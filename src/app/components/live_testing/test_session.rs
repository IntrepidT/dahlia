use super::{
    connection_status::ConnectionStatusIndicator,
    navigation::NavigationControls,
    participants_list::ParticipantsList,
    question_card::QuestionCard,
    test_controls::TestControls,
    types::*,
    websocket_handler::{use_websocket_connection, WebSocketActions},
};
use crate::app::models::question::Question;
use crate::app::models::score::CreateScoreRequest;
use crate::app::models::test::Test;
use crate::app::models::user::SessionUser;
use crate::app::server_functions::websocket_sessions::cleanup_teacher_session_endpoint;
use crate::app::server_functions::{
    questions::get_questions, scores::add_score, tests::get_tests, websocket_sessions,
};
use leptos::*;
use leptos_router::*;
use log;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

#[component]
pub fn RealtimeTestSession() -> impl IntoView {
    // Get test_id from URL parameters - FIXED: Create proper signal instead of closure
    let params = use_params_map();

    // Create a memo that properly implements signals
    let test_id_memo = create_memo(move |_| {
        params.with(|params| params.get("test_id").cloned().unwrap_or_default())
    });

    let user = use_context::<ReadSignal<Option<SessionUser>>>().expect("AuthProvider not Found");

    // Initialize state
    let (room_id, set_room_id) = create_signal::<Option<Uuid>>(None);
    let (role, set_role) = create_signal(Role::Student);
    let (connected_students, set_connected_students) =
        create_signal::<Vec<ConnectedStudent>>(Vec::new());
    let (connection_status, set_connection_status) = create_signal(ConnectionStatus::Disconnected);
    let (error_message, set_error_message) = create_signal(None::<String>);
    let (current_card_index, set_current_card_index) = create_signal(0);
    let (responses, set_responses) = create_signal(HashMap::<i32, QuestionResponse>::new());
    let (selected_student_id, set_selected_student_id) = create_signal(None::<i32>);
    let (is_test_active, set_is_test_active) = create_signal(false);
    let (is_submitted, set_is_submitted) = create_signal(false);
    let (remaining_time, set_remaining_time) = create_signal::<Option<i32>>(None);
    let (should_disable_inputs, set_should_disable_inputs) = create_signal(true);

    // Initialize role based on user - Enhanced logging
    create_effect(move |_| {
        log::info!("=== Role Assignment Effect Triggered ===");

        if let Some(current_user) = user() {
            log::info!("User session data available:");
            log::info!("  - User ID: {}", current_user.id);
            log::info!("  - Username: {}", current_user.username);
            log::info!("  - Role: {:?}", current_user.role);
            log::info!("  - is_admin(): {}", current_user.is_admin());
            log::info!("  - is_teacher(): {}", current_user.is_teacher());

            if current_user.is_admin() {
                log::info!("🔑 Setting role to Teacher (Admin privileges)");
                set_role(Role::Teacher);
            } else if current_user.is_teacher() {
                log::info!("🍎 Setting role to Teacher (Teacher privileges)");
                set_role(Role::Teacher);
            } else {
                log::info!("📚 Setting role to Student (role: {:?})", current_user.role);
                set_role(Role::Student);
            }
        } else {
            log::warn!("❌ No user session data available - setting role to Unknown");
            set_role(Role::Unknown);
        }

        log::info!("=== Role Assignment Effect Complete ===");
    });

    // Enhanced role effect with input disable logic
    create_effect(move |_| {
        let current_role = role.get();
        let should_disable = matches!(current_role, Role::Student | Role::Unknown);

        log::info!("=== Input Disable Effect ===");
        log::info!("Current role: {:?}", current_role);
        log::info!("Should disable inputs: {}", should_disable);
        log::info!("========================");

        set_should_disable_inputs.set(should_disable);
    });

    // Fetch test details and questions - FIXED: Use the memo signal
    let test_details = create_resource(test_id_memo, move |tid| async move {
        if tid.is_empty() {
            log::warn!("No test ID provided in the URL");
            return None;
        }
        match get_tests().await {
            Ok(tests) => tests.into_iter().find(|test| test.test_id == tid),
            Err(e) => {
                log::error!("Failed to fetch test details: {}", e);
                None
            }
        }
    });

    let questions = create_resource(test_id_memo, move |tid| async move {
        if tid.is_empty() {
            log::warn!("No test ID provided in the URL");
            return Vec::new();
        }
        match get_questions(tid).await {
            Ok(questions) => questions,
            Err(e) => {
                log::error!("Failed to fetch questions: {}", e);
                Vec::new()
            }
        }
    });

    // WebSocket connection and actions - FIXED: Use memo signal
    #[cfg(feature = "hydrate")]
    let ws_actions = use_websocket_connection(
        room_id.into(),
        user.into(),
        set_connection_status,
        set_error_message,
        set_role,
        set_connected_students,
        set_responses,
        set_current_card_index,
        set_remaining_time,
        set_is_test_active,
        set_is_submitted,
        test_id_memo.into(), // Use the memo signal here - this should work now
        room_id.into(),
        questions.into(),
    );

    //heartbeat system
    #[cfg(feature = "hydrate")]
    {
        create_effect(move |_| {
            if matches!(connection_status.get(), ConnectionStatus::Connected) {
                // Send heartbeat every 30 seconds
                let send_heartbeat = ws_actions.send_heartbeat.clone();
                let interval_handle = super::utils::set_interval_with_handle(
                    move || {
                        send_heartbeat.call(());
                    },
                    std::time::Duration::from_secs(30),
                );

                on_cleanup(move || {
                    if let Ok(handle) = interval_handle {
                        handle.clear();
                    }
                });
            }
        });
    }

    //detect tab close for user
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;

        create_effect(move |_| {
            if matches!(role.get(), Role::Teacher) {
                let user_for_cleanup = user.clone();

                // Only set up the event listener, don't trigger cleanup
                let beforeunload_closure =
                    Closure::wrap(Box::new(move |_: web_sys::BeforeUnloadEvent| {
                        if let Some(current_user) = user_for_cleanup.get_untracked() {
                            if let Some(window) = web_sys::window() {
                                let navigator = window.navigator();
                                let cleanup_data =
                                    format!(r#"{{"teacher_id": {}}}"#, current_user.id);

                                let _ = navigator.send_beacon_with_opt_str(
                                    "/api/CleanupTeacherSession",
                                    Some(&cleanup_data),
                                );
                            }
                        }
                    })
                        as Box<dyn FnMut(web_sys::BeforeUnloadEvent)>);

                if let Some(window) = web_sys::window() {
                    let _ = window.add_event_listener_with_callback(
                        "beforeunload",
                        beforeunload_closure.as_ref().unchecked_ref(),
                    );
                }

                beforeunload_closure.forget();
            }
        });
    }

    // Submit handler
    let handle_submit = create_action(move |_: &()| async move {
        let current_responses = responses.get();
        let current_test_id = test_id_memo.get(); // Use memo instead of closure
        let student_id = selected_student_id.get().unwrap_or(0);
        let evaluator = user().map(|u| u.id.to_string()).unwrap_or_default();
        let test_variant = 1;

        let mut test_scores = Vec::new();
        let mut comments = Vec::new();

        if let Some(questions) = questions.get() {
            let mut sorted_questions = questions.clone();
            sorted_questions.sort_by_key(|q| q.qnumber);

            for question in sorted_questions {
                if let Some(response) = current_responses.get(&question.qnumber) {
                    let score = if response.answer == question.correct_answer {
                        question.point_value
                    } else {
                        0
                    };
                    test_scores.push(score);
                    comments.push(response.comment.clone());
                } else {
                    test_scores.push(0);
                    comments.push(String::new());
                }
            }
        }

        let score_request = CreateScoreRequest {
            student_id,
            test_id: current_test_id,
            test_scores,
            comments,
            test_variant,
            evaluator,
        };

        match add_score(score_request).await {
            Ok(score) => {
                log::info!(
                    "Successfully submitted score for student {}",
                    score.student_id
                );
                #[cfg(feature = "hydrate")]
                ws_actions.end_test.call(());
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to submit score: {}", e);
                Err(e)
            }
        }
    });

    // Create or join session - Enhanced error handling
    create_effect(move |_| {
        let tid = test_id_memo.get(); // Use memo instead of closure
        let test_name = match &test_details() {
            Some(Some(test)) => test.name.clone(),
            _ => "Unknown Test".to_string(),
        };

        if !tid.is_empty() {
            spawn_local(async move {
                log::info!("Attempting to create or join session for test: {}", tid);

                match websocket_sessions::get_test_sessions_by_test_id(tid.clone()).await {
                    Ok(sessions) => {
                        log::info!(
                            "Found {} existing sessions for test {}",
                            sessions.len(),
                            tid
                        );

                        if let Some(active_session) = sessions.iter().find(|s| {
                            let now = chrono::Utc::now();
                            let active_threshold = now - chrono::Duration::minutes(5);
                            s.last_active > active_threshold
                                && s.start_time.is_none()
                                && s.end_time.is_none()
                        }) {
                            log::info!("Joining existing active session: {}", active_session.id);
                            set_room_id.set(Some(active_session.id));
                        } else {
                            log::info!("Creating new session for test: {}", test_name);
                            let request =
                                crate::app::models::websocket_session::CreateSessionRequest {
                                    name: format!("Test Session for {}", test_name),
                                    description: Some(format!("Test session for {}", tid)),
                                    session_type: Some(
                                        crate::app::models::websocket_session::SessionType::Test,
                                    ),
                                    test_id: Some(tid.clone()),
                                    max_users: Some(30),
                                    is_private: Some(false),
                                    password: None,
                                    metadata: None,
                                    teacher_id: Some(
                                        user().map(|u| u.id as i32).unwrap_or_default(),
                                    ),
                                };

                            match websocket_sessions::create_session(request).await {
                                Ok(session) => {
                                    log::info!("Created new session: {}", session.id);
                                    set_room_id.set(Some(session.id));
                                }
                                Err(e) => {
                                    log::error!("Failed to create session: {}", e);
                                    set_error_message
                                        .set(Some(format!("Failed to create session: {}", e)));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to fetch test sessions: {}", e);
                        set_error_message.set(Some(format!("Failed to fetch sessions: {}", e)));
                    }
                }
            });
        }
    });

    // Format remaining time
    let formatted_time = move || {
        if let Some(seconds) = remaining_time.get() {
            let minutes = seconds / 60;
            let seconds = seconds % 60;
            format!("{}:{:02}", minutes, seconds)
        } else {
            "".to_string()
        }
    };

    // Calculate percentage of answered questions
    let calculate_answered_percentage = create_memo(move |_| {
        let answered_count = responses.with(|r| {
            questions
                .get()
                .map(|q| {
                    q.iter()
                        .filter(|question| {
                            r.get(&question.qnumber)
                                .map(|resp| !resp.answer.trim().is_empty())
                                .unwrap_or(false)
                        })
                        .count() as f32
                })
                .unwrap_or(0.0)
        });

        let total = questions.get().map(|q| q.len() as f32).unwrap_or(1.0);
        if total > 0.0 {
            (answered_count / total) * 100.0
        } else {
            0.0
        }
    });

    //periodic heartbeat to keep WebSocket connection alive
    #[cfg(feature = "hydrate")]
    {
        create_effect(move |_| {
            if matches!(connection_status.get(), ConnectionStatus::Connected) {
                // Send heartbeat every 30 seconds using the WebSocketActions
                let send_heartbeat = ws_actions.send_heartbeat.clone();
                let interval_handle = super::utils::set_interval_with_handle(
                    move || {
                        send_heartbeat.call(()); // Just call the callback, no direct WebSocket access
                    },
                    std::time::Duration::from_secs(30),
                );

                on_cleanup(move || {
                    if let Ok(handle) = interval_handle {
                        handle.clear();
                    }
                });
            }
        });
    }

    // Cleanup on unmount
    #[cfg(feature = "hydrate")]
    on_cleanup(move || {
        log::info!("Component unmounting - cleaning up session");

        if let (Some(current_user), true) = (
            user.get_untracked(),
            matches!(role.get_untracked(), Role::Teacher),
        ) {
            spawn_local(async move {
                if let Err(e) =
                    cleanup_teacher_session_endpoint(current_user.id.try_into().unwrap()).await
                {
                    log::error!("Failed to cleanup teacher session on unmount: {}", e);
                }
            });
        }
    });

    view! {
        <div class="p-4 max-w-screen h-screen overflow-y-auto bg-gray-50 mx-auto">
            {/* Header */}
            <div class="text-center mb-8">
                <h2 class="text-2xl font-bold text-gray-800">
                    {move || match &test_details.get() {
                        Some(Some(test)) => format!("Realtime Test Session: {}", test.name.clone()),
                        _ => format!("Test Session: {}", test_id_memo.get())
                    }}
                </h2>
                <div class="mt-2 text-sm text-gray-600">
                    {move || match role.get() {
                        Role::Teacher => "You are the teacher for this session",
                        Role::Student => "You are a student in this session",
                        Role::Unknown => "Connecting to session..."
                    }}
                </div>
            </div>

            {/* Connection Status - FIXED: Explicit signal conversions */}
            <ConnectionStatusIndicator
                connection_status=Signal::derive(move || connection_status.get())
                error_message=Signal::derive(move || error_message.get())
            />

            {/* Session Status */}
            <div class="flex justify-between items-center mb-6 max-w-4xl mx-auto">
                <div class="text-sm text-gray-600">
                    <span class="font-medium">"Session ID: "</span>
                    {move || room_id.get().map(|id| id.to_string()).unwrap_or_else(|| "Connecting...".to_string())}
                </div>
                <div class="text-sm text-gray-600">
                    <span class="font-medium">"Status: "</span>
                    {move || if is_test_active.get() { "Active" } else { "Waiting" }}
                </div>
                <div class="text-sm text-gray-600">
                    <span class="font-medium">"Time: "</span>
                    {move || formatted_time()}
                </div>
            </div>

            {/* Teacher Controls - FIXED: Explicit signal conversions */}
            <TestControls
                role=Signal::derive(move || role.get())
                is_test_active=Signal::derive(move || is_test_active.get())
                is_submitted=Signal::derive(move || is_submitted.get())
                connection_status=Signal::derive(move || connection_status.get())
                selected_student_id=Signal::derive(move || selected_student_id.get())
                room_id=Signal::derive(move || room_id.get())
                test_id=Signal::derive(move || test_id_memo.get())
                set_selected_student_id=set_selected_student_id
                start_test={
                    #[cfg(feature = "hydrate")]
                    {ws_actions.start_test}
                    #[cfg(not(feature = "hydrate"))]
                    {Callback::new(|_| {})}
                }
                end_test={
                    #[cfg(feature = "hydrate")]
                    {ws_actions.end_test}
                    #[cfg(not(feature = "hydrate"))]
                    {Callback::new(|_| {})}
                }
            />

            {/* Participants List - FIXED: Explicit signal conversions */}
            <ParticipantsList
                connected_students=Signal::derive(move || connected_students.get())
                role=Signal::derive(move || role.get())
            />

            {/* Test Content */}
            <Show when=move || is_test_active.get() || matches!(role.get(), Role::Teacher)>
                <Suspense fallback=move || view! {
                    <div class="flex justify-center items-center h-64">
                        <div class="animate-pulse bg-white rounded-lg shadow-md w-full max-w-2xl h-64 flex items-center justify-center">
                            <p class="text-gray-400">"Loading questions..."</p>
                        </div>
                    </div>
                }>
                    {move || match questions.get() {
                        None => view! {<div class="text-center py-8">"Loading..."</div>}.into_view(),
                        Some(questions_vec) if questions_vec.is_empty() => {
                            view! {<div class="text-center py-8 text-red-500">"No questions found for this test ID."</div>}.into_view()
                        },
                        Some(questions_vec) => {
                            let total_questions = questions_vec.len();
                            let current_question = create_memo(move |_| {
                                questions_vec.get(current_card_index.get()).cloned().unwrap_or_else(|| {
                                    questions_vec.first().cloned().unwrap()
                                })
                            });

                            view! {
                                <div class="flex flex-col items-center justify-center">
                                    {/* Progress Bar */}
                                    <div class="w-full max-w-2xl mb-4">
                                        <div class="flex justify-between mb-1 text-xs text-gray-700">
                                            <span>"Progress"</span>
                                            <span>{move || format!("{:.1}%", calculate_answered_percentage())}</span>
                                        </div>
                                        <div class="mb-4 w-full bg-gray-200 rounded-full h-2.5">
                                            <div
                                                class="bg-gradient-to-r from-blue-500 to-purple-600 h-2.5 rounded-full transition-all duration-1500 ease-in"
                                                style=move || format!("width: {}%", calculate_answered_percentage())
                                            ></div>
                                        </div>
                                    </div>

                                    {/* Card Counter */}
                                    <div class="text-center mb-4">
                                        <span class="inline-flex items-center justify-center bg-white text-sm font-medium text-gray-700 px-3 py-1 rounded-full shadow-sm border border-gray-200">
                                            {move || current_card_index.get() + 1}
                                            " / "
                                            {total_questions}
                                            <span class="ml-2 text-purple-600 font-semibold">
                                                {move || current_question().point_value}
                                                " pts"
                                            </span>
                                        </span>
                                    </div>

                                    {/* Question Card - FIXED: Explicit signal conversions */}
                                    <QuestionCard
                                        question=current_question()
                                        role=Signal::derive(move || role.get())
                                        responses=Signal::derive(move || responses.get())
                                        should_disable_inputs=Signal::derive(move || should_disable_inputs.get())
                                        on_answer_change={
                                            #[cfg(feature = "hydrate")]
                                            {ws_actions.handle_answer_change}
                                            #[cfg(not(feature = "hydrate"))]
                                            {Callback::new(|_| {})}
                                        }
                                        on_comment_change={
                                            #[cfg(feature = "hydrate")]
                                            {ws_actions.handle_comment_change}
                                            #[cfg(not(feature = "hydrate"))]
                                            {Callback::new(|_| {})}
                                        }
                                    />

                                    {/* Navigation Controls - FIXED: Explicit signal conversions */}
                                    <NavigationControls
                                        role=Signal::derive(move || role.get())
                                        is_test_active=Signal::derive(move || is_test_active.get())
                                        is_submitted=Signal::derive(move || is_submitted.get())
                                        should_disable_inputs=Signal::derive(move || should_disable_inputs.get())
                                        current_card_index=Signal::derive(move || current_card_index.get())
                                        total_questions=Signal::derive(move || total_questions)
                                        selected_student_id=Signal::derive(move || selected_student_id.get())
                                        on_previous={
                                            #[cfg(feature = "hydrate")]
                                            {ws_actions.go_to_previous_card}
                                            #[cfg(not(feature = "hydrate"))]
                                            {Callback::new(|_| {})}
                                        }
                                        on_next={
                                            #[cfg(feature = "hydrate")]
                                            {ws_actions.go_to_next_card}
                                            #[cfg(not(feature = "hydrate"))]
                                            {Callback::new(|_| {})}
                                        }
                                        on_submit=Callback::new(move |_| handle_submit.dispatch(()))
                                    />

                                    {/* Submission Status */}
                                    <Show when=move || is_submitted.get()>
                                        <div class="mt-8 text-center">
                                            <div class="inline-flex items-center px-4 py-2 rounded-full bg-green-100 text-green-800 mb-4">
                                                <span class="mr-2">"✓"</span>
                                                "Assessment submitted successfully!"
                                            </div>
                                            <div>
                                                <button
                                                    class="px-5 py-2 mt-2 bg-gray-800 text-white rounded-lg hover:bg-gray-700 transition-colors"
                                                    on:click=move |_| {
                                                        let navigate = leptos_router::use_navigate();
                                                        navigate("/dashboard", Default::default());
                                                    }
                                                >
                                                    "Return to Dashboard"
                                                </button>
                                            </div>
                                        </div>
                                    </Show>
                                </div>
                            }.into_view()
                        }
                    }}
                </Suspense>
            </Show>

            {/* Session Join Information (when test is not active and for students) */}
            <Show when=move || !is_test_active.get() && matches!(role.get(), Role::Student)>
                <div class="flex flex-col items-center justify-center py-12 max-w-md mx-auto">
                    <div class="bg-white p-8 rounded-lg shadow-md w-full text-center">
                        <h3 class="text-xl font-medium mb-4">"Waiting for Test to Start"</h3>
                        <p class="text-gray-600 mb-6">"Your teacher will start the test soon. Please stand by."</p>
                        <div class="animate-pulse flex justify-center">
                            <div class="h-4 w-4 bg-blue-400 rounded-full mr-1"></div>
                            <div class="h-4 w-4 bg-blue-500 rounded-full mr-1 animation-delay-200"></div>
                            <div class="h-4 w-4 bg-blue-600 rounded-full animation-delay-400"></div>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
