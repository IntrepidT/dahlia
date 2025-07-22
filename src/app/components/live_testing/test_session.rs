use super::{
    connection_status::ConnectionStatusIndicator,
    navigation::NavigationControls,
    participants_list::ParticipantsList,
    question_card::QuestionCard,
    test_controls::TestControls,
    types::*,
    websocket_handler::{use_websocket_connection, WebSocketActions},
};
use crate::app::models::question::{Question, QuestionType};
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

// Helper function using your existing create_or_join_session - FIXED
async fn create_new_session_with_existing_function(
    tid: String,
    test_name: String,
    teacher_id: i32,
    set_room_id: WriteSignal<Option<Uuid>>,
    set_error_message: WriteSignal<Option<String>>,
) {
    let request = crate::app::models::websocket_session::CreateSessionRequest {
        name: format!("Test Session for {}", test_name),
        description: Some(format!("Test session for {}", tid)),
        session_type: Some(crate::app::models::websocket_session::SessionType::Test),
        test_id: Some(tid.clone()),
        max_users: Some(30),
        is_private: Some(false),
        password: None,
        metadata: None,
        teacher_id: Some(teacher_id),
    };

    match websocket_sessions::create_or_join_session(request).await {
        Ok(session) => {
            log::info!("Created/joined session: {}", session.id);
            set_room_id.set(Some(session.id)); // FIXED: Remove Uuid::parse_str()
        }
        Err(e) => {
            log::error!("Failed to create/join session: {}", e);
            set_error_message.set(Some(format!("Failed to create session: {}", e)));
        }
    }
}

#[component]
pub fn RealtimeTestSession() -> impl IntoView {
    // Get test_id from URL parameters
    let params = use_params_map();
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
    let (show_participants, set_show_participants) = create_signal(false);

    // Initialize role based on user
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

    // Fetch test details and questions
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
            Ok(mut questions) => {
                // Sort questions by qnumber to ensure consistent ordering
                questions.sort_by_key(|q| q.qnumber);
                questions
            }
            Err(e) => {
                log::error!("Failed to fetch questions: {}", e);
                Vec::new()
            }
        }
    });

    // WebSocket connection and actions
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
        test_id_memo.into(),
        room_id.into(),
        questions.into(),
    );

    // Heartbeat system
    #[cfg(feature = "hydrate")]
    {
        create_effect(move |_| {
            if matches!(connection_status.get(), ConnectionStatus::Connected) {
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

    // Detect tab close for user
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;

        create_effect(move |_| {
            if matches!(role.get(), Role::Teacher) {
                let user_for_cleanup = user.clone();

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

    // Submit handler with weighted multiple choice support
    let handle_submit = create_action(move |_: &()| async move {
        let current_responses = responses.get();
        let current_test_id = test_id_memo.get();
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
                    let score = match question.question_type {
                        QuestionType::WeightedMultipleChoice => {
                            // Calculate weighted score
                            if let Some(ref selected_opts) = response.selected_options {
                                question.calculate_weighted_score(selected_opts)
                            } else {
                                0
                            }
                        }
                        _ => {
                            // Regular scoring logic
                            if response.answer == question.correct_answer {
                                question.point_value
                            } else {
                                0
                            }
                        }
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

    // Create or join session
    create_effect(move |_| {
        let tid = test_id_memo.get();
        let test_name = match &test_details() {
            Some(Some(test)) => test.name.clone(),
            _ => "Unknown Test".to_string(),
        };

        if !tid.is_empty() {
            if let Some(current_user) = user() {
                let teacher_id = current_user.id as i32;

                spawn_local(async move {
                    log::info!("Checking for existing teacher sessions for test: {}", tid);

                    match websocket_sessions::get_teacher_active_session(teacher_id).await {
                        Ok(Some(existing_session)) => {
                            // Check if it's for the same test
                            if existing_session
                                .test_id
                                .as_ref()
                                .map_or(false, |test_id| test_id == &tid)
                            {
                                log::info!(
                                    "Found existing session for same test - taking over: {}",
                                    existing_session.id
                                );
                                set_room_id.set(Some(existing_session.id)); // FIXED: session.id is already Uuid
                            } else {
                                log::info!("Found session for different test - cleaning up and creating new");
                                // Clean up old session
                                let _ = websocket_sessions::cleanup_teacher_session_endpoint(
                                    teacher_id,
                                )
                                .await;

                                // Create new session using your existing create_or_join_session
                                create_new_session_with_existing_function(
                                    tid,
                                    test_name,
                                    teacher_id,
                                    set_room_id,
                                    set_error_message,
                                )
                                .await;
                            }
                        }
                        Ok(None) => {
                            log::info!("No existing session found - creating new");
                            create_new_session_with_existing_function(
                                tid,
                                test_name,
                                teacher_id,
                                set_room_id,
                                set_error_message,
                            )
                            .await;
                        }
                        Err(e) => {
                            log::error!("Error checking for teacher session: {}", e);
                            create_new_session_with_existing_function(
                                tid,
                                test_name,
                                teacher_id,
                                set_room_id,
                                set_error_message,
                            )
                            .await;
                        }
                    }
                });
            }
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
        <div class="min-h-screen bg-gray-50">
            {/* Minimal Top Bar */}
            <div class="sticky top-0 z-10 bg-white/80 backdrop-blur-md border-b border-gray-100">
                <div class="max-w-5xl mx-auto px-6 py-3">
                    <div class="flex items-center justify-between">
                        {/* Left: Student Select (Teacher only) */}
                        <div class="flex-shrink-0">
                            <Show when=move || matches!(role.get(), Role::Teacher)>
                                <super::student_select::StudentSelect set_selected_student_id=set_selected_student_id />
                            </Show>
                        </div>

                        {/* Center: Test Title */}
                        <div class="flex-1 text-center px-8">
                            <h1 class="text-lg font-medium text-gray-900 truncate">
                                {move || match &test_details.get() {
                                    Some(Some(test)) => format!("Live Test: {}", test.name),
                                    _ => format!("Live Test: {}", test_id_memo.get())
                                }}
                            </h1>
                        </div>

                        {/* Right: Role and Status */}
                        <div class="flex items-center gap-3">
                            <div class="text-sm text-gray-500 font-medium hidden sm:block">
                                {move || match role.get() {
                                    Role::Teacher => "Teacher",
                                    Role::Student => "Student",
                                    Role::Unknown => "Connecting..."
                                }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="max-w-5xl mx-auto px-6 py-4">
                {/* Connection Status */}
                <ConnectionStatusIndicator
                    connection_status=Signal::derive(move || connection_status.get())
                    error_message=Signal::derive(move || error_message.get())
                />

                {/* Session Status */}
                <div class="flex justify-center items-center mb-3 space-x-8 text-sm">
                    <Show when=move || !formatted_time().is_empty()>
                        <div class="text-gray-600">
                            <span class="font-medium">"Time: "</span>
                            {move || formatted_time()}
                        </div>
                    </Show>
                </div>

                {/* Teacher Controls */}
                <div class="mb-4">
                    <TestControls
                        role=Signal::derive(move || role.get())
                        is_test_active=Signal::derive(move || is_test_active.get())
                        is_submitted=Signal::derive(move || is_submitted.get())
                        connection_status=Signal::derive(move || connection_status.get())
                        selected_student_id=Signal::derive(move || selected_student_id.get())
                        room_id=Signal::derive(move || room_id.get())
                        test_id=Signal::derive(move || test_id_memo.get())
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
                </div>

                {/* Compact Participants Dropdown */}
                <div class="mb-4">
                    <button
                        class="w-full flex items-center justify-between px-4 py-2 bg-white border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors"
                        on:click=move |_| set_show_participants.update(|show| *show = !*show)
                    >
                        <div class="flex items-center gap-3">
                            <div class="w-2 h-2 bg-green-400 rounded-full"></div>
                            <span class="font-medium text-gray-700">
                                {move || format!("{} Connected", connected_students.get().len())}
                            </span>
                        </div>
                        <svg
                            class=move || format!("w-4 h-4 text-gray-400 transition-transform {}",
                                if show_participants.get() { "rotate-180" } else { "" })
                            fill="none" stroke="currentColor" viewBox="0 0 24 24"
                        >
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                        </svg>
                    </button>

                    <Show when=move || show_participants.get()>
                        <div class="mt-2 p-3 bg-white border border-gray-200 rounded-lg shadow-sm">
                            <ParticipantsList
                                connected_students=Signal::derive(move || connected_students.get())
                                role=Signal::derive(move || role.get())
                            />
                        </div>
                    </Show>
                </div>

                {/* Test Content */}
                <Show when=move || is_test_active.get() || matches!(role.get(), Role::Teacher)>
                    <Suspense fallback=move || view! {
                        <div class="flex items-center justify-center h-96">
                            <div class="flex flex-col items-center gap-4">
                                <div class="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
                                <p class="text-gray-500 text-sm">"Loading questions..."</p>
                            </div>
                        </div>
                    }>
                        {move || match questions.get() {
                            None => view! {
                                <div class="text-center py-8">"Loading..."</div>
                            }.into_view(),
                            Some(questions_vec) if questions_vec.is_empty() => {
                                view! {
                                    <div class="flex items-center justify-center h-96">
                                        <div class="text-center">
                                            <div class="w-16 h-16 bg-red-50 rounded-full flex items-center justify-center mx-auto mb-4">
                                                <svg class="w-8 h-8 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.732-.833-2.5 0L4.268 18.5c-.77.833.192 2.5 1.732 2.5z"></path>
                                                </svg>
                                            </div>
                                            <p class="text-gray-500">"No questions found for this test."</p>
                                        </div>
                                    </div>
                                }.into_view()
                            },
                            Some(questions_vec) => {
                                let total_questions = questions_vec.len();
                                let current_question = create_memo(move |_| {
                                    questions_vec.get(current_card_index.get()).cloned().unwrap_or_else(|| {
                                        questions_vec.first().cloned().unwrap()
                                    })
                                });

                                view! {
                                    <div class="space-y-6">
                                        {/* Progress Section */}
                                        <div class="text-center space-y-2">
                                            {/* Progress Bar */}
                                            <div class="w-full max-w-md mx-auto">
                                                <div class="bg-gray-100 rounded-full h-1">
                                                    <div
                                                        class="bg-gradient-to-r from-blue-500 to-indigo-600 h-1 rounded-full transition-all duration-700 ease-out"
                                                        style=move || format!("width: {}%", calculate_answered_percentage())
                                                    ></div>
                                                </div>
                                            </div>

                                            {/* Question Counter */}
                                            <div class="flex items-center justify-center gap-6 text-sm">
                                                <span class="text-gray-500">
                                                    "Question " {move || current_card_index.get() + 1} " of " {total_questions}
                                                </span>
                                                <span class="px-3 py-1 bg-indigo-50 text-indigo-700 rounded-full font-medium">
                                                    {move || current_question().point_value} " points"
                                                </span>
                                            </div>
                                        </div>

                                        {/* Question Card */}
                                        <div class="max-w-4xl mx-auto">
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
                                                on_weighted_selection={
                                                    #[cfg(feature = "hydrate")]
                                                    {ws_actions.handle_weighted_selection}
                                                    #[cfg(not(feature = "hydrate"))]
                                                    {Callback::new(|_| {})}
                                                }
                                            />
                                        </div>

                                        {/* Navigation Controls */}
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
                                            <div class="text-center pt-4">
                                                <div class="inline-flex items-center gap-3 px-6 py-3 bg-green-50 border border-green-200 rounded-lg text-green-800">
                                                    <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                                                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"></path>
                                                    </svg>
                                                    "Assessment submitted successfully!"
                                                </div>
                                                <div class="mt-4">
                                                    <button
                                                        class="px-5 py-2 bg-gray-800 text-white rounded-lg hover:bg-gray-700 transition-colors"
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

                {/* Waiting Message for Students */}
                <Show when=move || !is_test_active.get() && matches!(role.get(), Role::Student)>
                    <div class="flex flex-col items-center justify-center py-12 max-w-md mx-auto">
                        <div class="bg-white p-8 rounded-lg shadow-md w-full text-center">
                            <h3 class="text-xl font-medium mb-4">"Waiting for Test to Start"</h3>
                            <p class="text-gray-600 mb-6">"Your teacher will start the test soon. Please stay connected."</p>
                            <div class="animate-pulse flex justify-center">
                                <div class="h-4 w-4 bg-blue-400 rounded-full mr-1"></div>
                                <div class="h-4 w-4 bg-blue-500 rounded-full mr-1 animation-delay-200"></div>
                                <div class="h-4 w-4 bg-blue-600 rounded-full animation-delay-400"></div>
                            </div>
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}
