use crate::app::models::question::QuestionType;
use crate::app::models::score::CreateScoreRequest;
use crate::app::models::student::Student;
use crate::app::models::test::Test;
use crate::app::models::user::User;
use crate::app::models::websocket_session::{CreateSessionRequest, SessionSummary, SessionType};
use crate::app::server_functions::questions::get_questions;
use crate::app::server_functions::scores::add_score;
use crate::app::server_functions::students::get_students;
use crate::app::server_functions::{tests::get_tests, websocket_sessions};
use chrono::{DateTime, Duration, Utc};
use leptos::ev::ErrorEvent;
use leptos::*;
use leptos_router::*;
use log::{error, info, warn};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration as StdDuration;
use uuid::Uuid;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{CloseEvent, MessageEvent, WebSocket};

#[derive(Clone, Copy, PartialEq, Debug)]
enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

#[derive(Debug, Clone)]
struct QuestionResponse {
    answer: String,
    comment: String,
}

#[derive(Debug, Clone)]
enum Role {
    Teacher,
    Student,
    Unknown,
}

#[derive(Debug, Clone)]
struct ConnectedStudent {
    student_id: String,
    name: String,
    status: String,
}

#[component]
pub fn RealtimeTestSession() -> impl IntoView {
    // Get test_id from URL parameters
    let params = use_params_map();
    let test_id = move || params.with(|params| params.get("test_id").cloned().unwrap_or_default());
    let user = use_context::<ReadSignal<Option<User>>>().expect("AuthProvider not Found");

    // WebSocket state
    let (ws, set_ws) = create_signal::<Option<WebSocket>>(None);
    let (room_id, set_room_id) = create_signal::<Option<Uuid>>(None);
    let (role, set_role) = create_signal(Role::Student);
    let (connected_students, set_connected_students) =
        create_signal::<Vec<ConnectedStudent>>(Vec::new());
    let (connection_status, set_connection_status) = create_signal(ConnectionStatus::Disconnected);
    let (error_message, set_error_message) = create_signal(None::<String>);

    let (should_disable_inputs, set_should_disable_inputs) = create_signal(true);

    if user().expect("Unwrapping a user").is_admin()
        || user().expect("Unwrapping a user").is_teacher()
    {
        set_role(Role::Teacher);
    } else {
        set_role(Role::Student);
    };

    create_effect(move |_| {
        let current_role = role.get();
        match current_role {
            Role::Teacher => set_should_disable_inputs.set(false),
            Role::Student => set_should_disable_inputs.set(true),
            Role::Unknown => set_should_disable_inputs.set(true),
        }
    });

    // Test session state
    let test_details = create_resource(test_id.clone(), move |tid| async move {
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

    let questions = create_resource(test_id, move |tid| async move {
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

    let (current_card_index, set_current_card_index) = create_signal(0);
    let (responses, set_responses) = create_signal(HashMap::<i32, QuestionResponse>::new());
    let (selected_student_id, set_selected_student_id) = create_signal(None::<i32>);
    let (is_test_active, set_is_test_active) = create_signal(false);
    let (is_submitted, set_is_submitted) = create_signal(false);

    // Timer for test duration
    let (remaining_time, set_remaining_time) = create_signal::<Option<i32>>(None);

    // Get evaluator ID
    let evaluator_id = create_memo(move |_| match user.get() {
        Some(user_data) => user_data.id.to_string(),
        None => "0".to_string(),
    });

    // Handle WebSocket connection
    let connect_to_session = create_action(move |session_id: &Uuid| {
        let session_id = *session_id;
        let protocol = if web_sys::window().unwrap().location().protocol().unwrap() == "https:" {
            "wss"
        } else {
            "ws"
        };
        let host = web_sys::window().unwrap().location().host().unwrap();
        let ws_url = format!("{protocol}://{host}/api/ws/{session_id}");

        // Clone necessary signals for the async block
        let set_role = set_role.clone();
        let set_connected_students = set_connected_students.clone();
        let set_responses = set_responses.clone();
        let set_current_card_index = set_current_card_index.clone();
        let set_remaining_time = set_remaining_time.clone();
        let set_is_test_active = set_is_test_active.clone();
        let set_is_submitted = set_is_submitted.clone();
        let set_connection_status = set_connection_status.clone(); // Add a new signal for connection status
        let set_error_message = set_error_message.clone(); // Add error message signal

        async move {
            log::info!("Connecting to WebSocket at: {}", ws_url);

            // Close any existing connection
            if let Some(ws) = ws.get_untracked() {
                let _ = ws.close();
            }

            // Reset connection-dependent state
            set_connection_status.set(ConnectionStatus::Connecting);
            set_error_message.set(None);

            match WebSocket::new(&ws_url) {
                Ok(websocket) => {
                    // Setup message handler
                    let ws_clone = websocket.clone();
                    let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                        if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                            let message = text.as_string().unwrap();
                            log::debug!("WebSocket message received: {}", message);

                            match serde_json::from_str::<Value>(&message) {
                                Ok(json_value) => {
                                    if let Some(msg_type) =
                                        json_value.get("type").and_then(|t| t.as_str())
                                    {
                                        match msg_type {
                                            "role_assigned" => {
                                                log::info!("Assigning role");
                                                if let Some(role_str) =
                                                    json_value.get("role").and_then(|r| r.as_str())
                                                {
                                                    match role_str {
                                                        "teacher" => set_role.set(Role::Teacher),
                                                        "student" => set_role.set(Role::Student),
                                                        _ => set_role.set(Role::Unknown),
                                                    }
                                                }
                                            }
                                            "test_data" => {
                                                if let Some(questions_array) = json_value
                                                    .get("questions")
                                                    .and_then(|q| q.as_array())
                                                {
                                                    let qs: Vec<
                                                        crate::app::models::question::Question,
                                                    > = questions_array
                                                        .iter()
                                                        .filter_map(|q| {
                                                            serde_json::from_value(q.clone()).ok()
                                                        })
                                                        .collect();
                                                }
                                            }
                                            "student_joined" | "user_joined" => {
                                                let is_student = msg_type == "student_joined";
                                                let log_msg = if is_student {
                                                    "student joined"
                                                } else {
                                                    "user joined"
                                                };
                                                log::info!(
                                                    "Received {} message {:?}",
                                                    log_msg,
                                                    json_value
                                                );

                                                // Extract ID field based on message type
                                                let id_field =
                                                    if is_student { "student_id" } else { "id" };
                                                let data_field = if is_student {
                                                    "student_data"
                                                } else {
                                                    "user_data"
                                                };
                                                let name_field =
                                                    if is_student { "name" } else { "username" };

                                                if let Some(user_id) = json_value
                                                    .get(id_field)
                                                    .and_then(|s| s.as_str())
                                                {
                                                    if let Some(user_data) =
                                                        json_value.get(data_field)
                                                    {
                                                        let name = user_data
                                                            .get(name_field)
                                                            .and_then(|n| n.as_str())
                                                            .unwrap_or("Unknown");

                                                        set_connected_students.update(|students| {
                                                            // Check if student already exists
                                                            if let Some(pos) =
                                                                students.iter().position(|s| {
                                                                    s.student_id == user_id
                                                                })
                                                            {
                                                                students[pos].status =
                                                                    "Connected".to_string();
                                                            } else {
                                                                students.push(ConnectedStudent {
                                                                    student_id: user_id.to_string(),
                                                                    name: name.to_string(),
                                                                    status: "Connected".to_string(),
                                                                });
                                                            }
                                                        });
                                                    }
                                                }
                                            }
                                            "student_left" | "user_left" => {
                                                let is_student = msg_type == "student_left";
                                                let log_msg = if is_student {
                                                    "student left"
                                                } else {
                                                    "user left"
                                                };
                                                log::info!(
                                                    "Received {} message {:?}",
                                                    log_msg,
                                                    json_value
                                                );

                                                // Extract ID field based on message type
                                                let id_field =
                                                    if is_student { "student_id" } else { "id" };

                                                if let Some(user_id) = json_value
                                                    .get(id_field)
                                                    .and_then(|s| s.as_str())
                                                {
                                                    set_connected_students.update(|students| {
                                                        if let Some(pos) = students
                                                            .iter()
                                                            .position(|s| s.student_id == user_id)
                                                        {
                                                            students[pos].status =
                                                                "Disconnected".to_string();
                                                        }
                                                    });
                                                }
                                            }
                                            "student_answer" => {
                                                if let Some(answer_data) =
                                                    json_value.get("answer_data")
                                                {
                                                    if let (Some(qnumber), Some(answer)) = (
                                                        answer_data
                                                            .get("question_id")
                                                            .and_then(|q| q.as_i64()),
                                                        answer_data
                                                            .get("answer")
                                                            .and_then(|a| a.as_str()),
                                                    ) {
                                                        set_responses.update(|r| {
                                                            let qnumber = qnumber as i32;
                                                            let response = r
                                                                .entry(qnumber)
                                                                .or_insert(QuestionResponse {
                                                                    answer: String::new(),
                                                                    comment: String::new(),
                                                                });
                                                            response.answer = answer.to_string();
                                                        });
                                                    }
                                                }
                                            }
                                            "focus_question" => {
                                                if let Some(question_data) =
                                                    json_value.get("question_data")
                                                {
                                                    if let Some(index) = question_data
                                                        .get("index")
                                                        .and_then(|i| i.as_i64())
                                                    {
                                                        set_current_card_index.set(index as usize);
                                                    }
                                                }
                                            }
                                            "time_update" => {
                                                if let Some(time_data) = json_value.get("time_data")
                                                {
                                                    if let Some(remaining) = time_data
                                                        .get("remaining")
                                                        .and_then(|r| r.as_i64())
                                                    {
                                                        set_remaining_time
                                                            .set(Some(remaining as i32));
                                                    }
                                                }
                                            }
                                            "test_started" => {
                                                set_is_test_active.set(true);
                                            }
                                            "test_ended" => {
                                                set_is_test_active.set(false);
                                                set_is_submitted.set(true);
                                            }
                                            _ => {
                                                log::debug!("Unhandled message type: {}", msg_type);
                                            }
                                        }
                                    }
                                }
                                Err(err) => {
                                    log::error!("Failed to parse WebSocket message: {:?}", err);
                                }
                            }
                        }
                    })
                        as Box<dyn FnMut(MessageEvent)>);

                    // Setup onopen handler
                    let set_connection_status_clone = set_connection_status.clone();
                    let onopen_callback = Closure::wrap(Box::new(move |_| {
                        log::info!("WebSocket connection established");
                        set_connection_status_clone.set(ConnectionStatus::Connected);
                    })
                        as Box<dyn FnMut(JsValue)>);

                    // Setup onclose handler
                    let set_connection_status_clone = set_connection_status.clone();
                    let onclose_callback = Closure::wrap(Box::new(move |e: CloseEvent| {
                        log::info!("WebSocket closed: {} - {}", e.code(), e.reason());
                        set_connection_status_clone.set(ConnectionStatus::Disconnected);
                    })
                        as Box<dyn FnMut(CloseEvent)>);

                    // Setup onerror handler
                    let set_connection_status_clone = set_connection_status.clone();
                    let set_error_message_clone = set_error_message.clone();
                    let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
                        let error_msg = e.message();
                        log::error!("WebSocket error occurred: {}", error_msg);
                        set_connection_status_clone.set(ConnectionStatus::Error);
                        set_error_message_clone
                            .set(Some(format!("Connection error: {}", error_msg)));
                    })
                        as Box<dyn FnMut(ErrorEvent)>);

                    // Set event handlers
                    websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
                    websocket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
                    websocket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
                    websocket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));

                    // Store callbacks to prevent them from being dropped
                    onmessage_callback.forget();
                    onopen_callback.forget();
                    onclose_callback.forget();
                    onerror_callback.forget();

                    // Store the websocket
                    set_ws.set(Some(websocket));
                    Ok(())
                }
                Err(err) => {
                    let error_msg = format!("WebSocket connection failed: {:?}", err);
                    log::error!("{}", error_msg);
                    set_connection_status.set(ConnectionStatus::Error);
                    set_error_message.set(Some(error_msg));
                    Err(())
                }
            }
        }
    });

    // Create or join session
    create_effect(move |_| {
        let tid = test_id();
        if !tid.is_empty() {
            spawn_local(async move {
                // First check if there's an active session for this test
                match websocket_sessions::get_test_sessions_by_test_id(tid.clone()).await {
                    Ok(sessions) => {
                        if let Some(active_session) = sessions.iter().find(|s| {
                            let now = Utc::now();
                            let active_threshold = now - chrono::Duration::minutes(5);
                            s.last_active > active_threshold
                                && s.start_time.is_none()
                                && s.end_time.is_none()
                        }) {
                            // Join existing session
                            set_room_id.set(Some(active_session.id));
                            if let Some(session_uuid) = Some(active_session.id) {
                                connect_to_session.dispatch(session_uuid);
                            }
                        } else {
                            // Create new session
                            let request = CreateSessionRequest {
                                name: format!("Test Session for {}", tid),
                                description: Some(format!(
                                    "Interactive test session for test {}",
                                    tid
                                )),
                                session_type: Some(SessionType::Test),
                                test_id: Some(tid),
                                max_users: Some(30),
                                is_private: Some(false),
                                password: None,
                                metadata: None,
                            };

                            match websocket_sessions::create_session(request).await {
                                Ok(session) => {
                                    set_room_id.set(Some(session.id));
                                    if let Some(session_uuid) = Some(session.id) {
                                        connect_to_session.dispatch(session_uuid);
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to create session: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to fetch test sessions: {}", e);
                    }
                }
            });
        }
    });

    // Send message through WebSocket
    let send_ws_message = move |message: String| {
        if let Some(socket) = ws.get() {
            let _ = socket.send_with_str(&message);
        }
    };

    // Start test handler
    let start_test = move |_| {
        if !test_id().is_empty() {
            let message = json!({
                "type": "test_message",
                "test_message_type": "start_test",
                "payload": {
                    "test_id": test_id()
                }
            })
            .to_string();

            send_ws_message(message);

            // Also start a timer for the test
            let duration_minutes = 60; // Default to 60 minutes
            set_remaining_time.set(Some(duration_minutes * 60));

            // Start timer countdown
            let timer_handle = set_interval_with_handle(
                move || {
                    set_remaining_time.update(|time| {
                        if let Some(t) = time {
                            if *t > 0 {
                                *t -= 1;

                                // Send time update every minute
                                if *t % 60 == 0 {
                                    let time_message = json!({
                                        "type": "test_message",
                                        "test_message_type": "time_update",
                                        "payload": {
                                            "remaining": *t
                                        }
                                    })
                                    .to_string();

                                    if let Some(socket) = ws.get() {
                                        let _ = socket.send_with_str(&time_message);
                                    }
                                }
                            } else {
                                // Time's up, end the test
                                let end_message = json!({
                                    "type": "test_message",
                                    "test_message_type": "end_test",
                                    "payload": {}
                                })
                                .to_string();

                                if let Some(socket) = ws.get() {
                                    let _ = socket.send_with_str(&end_message);
                                }
                            }
                        }
                    });
                },
                StdDuration::from_secs(1),
            )
            .expect("Could not create interval");
        }
    };

    // End test handler
    let end_test = move |_| {
        let end_message = json!({
            "type": "test_message",
            "test_message_type": "end_test",
            "payload": {}
        })
        .to_string();

        send_ws_message(end_message);

        // Also update the test status in the database
        if let Some(room_uuid) = room_id.get() {
            spawn_local(async move {
                match websocket_sessions::end_test_session(room_uuid.to_string()).await {
                    Ok(_) => {
                        log::info!("Test session ended successfully");
                    }
                    Err(e) => {
                        log::error!("Failed to end test session: {}", e);
                    }
                }
            });
        }
    };

    // Navigation handlers for teacher
    let go_to_next_card = move |_| {
        set_current_card_index.update(|index| {
            if let Some(questions_vec) = questions.get() {
                *index = (*index + 1).min(questions_vec.len() - 1);

                // Notify all clients about the question change
                let focus_message = json!({
                    "type": "test_message",
                    "test_message_type": "question_focus",
                    "payload": {
                        "index": *index
                    }
                })
                .to_string();

                send_ws_message(focus_message);
            }
        });
    };

    let go_to_previous_card = move |_| {
        set_current_card_index.update(|index| {
            *index = index.saturating_sub(1);

            // Notify all clients about the question change
            let focus_message = json!({
                "type": "test_message",
                "test_message_type": "question_focus",
                "payload": {
                    "index": *index
                }
            })
            .to_string();

            send_ws_message(focus_message);
        });
    };

    // Handle answer submitted by student
    let handle_answer_change = move |qnumber: i32, value: String| {
        // If student, send to teacher
        if matches!(role.get(), Role::Student) {
            let answer_message = json!({
                "type": "test_message",
                "test_message_type": "submit_answer",
                "payload": {
                    "question_id": qnumber,
                    "answer": value
                }
            })
            .to_string();

            send_ws_message(answer_message);
        } else {
            // If teacher, just update local state
            set_responses.update(|r| {
                let response = r.entry(qnumber).or_insert(QuestionResponse {
                    answer: String::new(),
                    comment: String::new(),
                });
                response.answer = value;
            });
        }
    };

    // Handle comment change
    let handle_comment_change = move |qnumber: i32, value: String| {
        let value_clone = value.clone();
        set_responses.update(|r| {
            let response = r.entry(qnumber).or_insert(QuestionResponse {
                answer: String::new(),
                comment: String::new(),
            });
            response.comment = value_clone;
        });

        // If teacher, broadcast the comment
        if matches!(role.get(), Role::Teacher) {
            let comment_message = json!({
                "type": "test_message",
                "test_message_type": "teacher_comment",
                "payload": {
                    "question_id": qnumber,
                    "comment": value
                }
            })
            .to_string();

            send_ws_message(comment_message);
        }
    };

    // Submit handler for teacher
    let handle_submit = create_action(move |_: &()| async move {
        let current_responses = responses.get();
        let current_test_id = test_id();

        let student_id = selected_student_id.get().unwrap_or(0);
        let evaluator = evaluator_id();
        let test_variant = 1;

        // Collect all scores and comments
        let mut test_scores = Vec::new();
        let mut comments = Vec::new();

        if let Some(questions) = questions.get() {
            // Sort questions by qnumber to ensure correct order
            let mut sorted_questions = questions.clone();
            sorted_questions.sort_by_key(|q| q.qnumber);

            for question in sorted_questions {
                if let Some(response) = current_responses.get(&question.qnumber) {
                    // Calculate score for this question
                    let score = if response.answer == question.correct_answer {
                        question.point_value
                    } else {
                        0
                    };
                    test_scores.push(score);
                    comments.push(response.comment.clone());
                } else {
                    // If no response, push 0 score and empty comment
                    test_scores.push(0);
                    comments.push(String::new());
                }
            }
        }

        // Create score request
        let score_request = CreateScoreRequest {
            student_id,
            test_id: current_test_id,
            test_scores,
            comments,
            test_variant,
            evaluator,
        };

        // Submit score to server
        match add_score(score_request).await {
            Ok(score) => {
                log::info!(
                    "Successfully submitted score for student {}",
                    score.student_id
                );

                // End the test after submitting
                end_test(());

                Ok(())
            }
            Err(e) => {
                log::error!("Failed to submit score: {}", e);
                Err(e)
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
    on_cleanup(move || {
        if let Some(socket) = ws.get() {
            socket.close().ok();
        }
    });

    view! {
        <div class="p-4 max-w-screen h-screen overflow-y-auto bg-gray-50 mx-auto">
            {/* Header */}
            <div class="text-center mb-8">
                <h2 class="text-2xl font-bold text-gray-800">
                    {move || match &test_details.get() {
                        Some(Some(test)) => format!("Realtime Test Session: {}", test.name.clone()),
                        _ => test_id()
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

            {/* Session Status */}
            <div class="flex justify-between items-center mb-6 max-w-4xl mx-auto">
                <div class="text-sm text-gray-600">
                    <span class="font-medium">Session ID: </span>
                    {move || room_id.get().map(|id| id.to_string()).unwrap_or_else(|| "Connecting...".to_string())}
                </div>
                <div class="text-sm text-gray-600">
                    <span class="font-medium">Status: </span>
                    {move || if is_test_active.get() { "Active" } else { "Waiting" }}
                </div>
                <div class="text-sm text-gray-600">
                    <span class="font-medium">Time: </span>
                    {move || formatted_time()}
                </div>
            </div>

            {/* Teacher Controls */}
            <Show when=move || matches!(role.get(), Role::Teacher)>
                <div class="mb-8 flex flex-wrap gap-4 justify-center">
                    <Show when=move || !is_test_active.get() && !is_submitted.get()>
                        <div class="w-full md:w-1/2 mb-4">
                            <StudentSelect set_selected_student_id=set_selected_student_id />
                        </div>
                        <button
                            class="px-5 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
                            on:click=start_test
                            disabled=move || selected_student_id.get().is_none()
                        >
                            "Start Test Session"
                        </button>
                    </Show>
                    <Show when=move || is_test_active.get() && !is_submitted.get()>
                        <button
                            class="px-5 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
                            on:click=move |_| end_test(())
                        >
                            "End Test Session"
                        </button>
                    </Show>
                </div>
            </Show>

            {/* Connected Students (Teacher View) */}
            <Show when=move || matches!(role.get(), Role::Teacher)>
                <div class="mb-8 max-w-4xl mx-auto">
                    <h3 class="text-lg font-medium mb-2">Connected Students</h3>
                    <div class="bg-white shadow-sm rounded-lg p-4">
                        <Show when=move || !connected_students.get().is_empty() fallback=|| view! {
                            <p class="text-gray-500 text-center py-2">"No students connected"</p>
                        }>
                            <ul class="divide-y divide-gray-200">
                                <For
                                    each=move || connected_students.get()
                                    key=|student| student.student_id.clone()
                                    children=move |student| {
                                        let status_for_class = student.status.clone();
                                        let status_for_display = student.status.clone();

                                        view! {
                                            <li class="py-2 flex justify-between items-center">
                                                <span>{student.name} ({student.student_id})</span>
                                                <span class=move || {
                                                    format!("text-sm px-2 py-1 rounded-full {}",
                                                    if student.status == "Connected" { "bg-green-100 text-green-800" }
                                                    else { "bg-red-100 text-red-800" })
                                                }>
                                                    {status_for_display}
                                                </span>
                                            </li>
                                        }
                                    }
                                />
                            </ul>
                        </Show>
                    </div>
                </div>
            </Show>

            {/* Test Content - Only show if test is active or teacher */}
            <Show when=move || is_test_active.get() || matches!(role.get(), Role::Teacher)>
                <Suspense
                    fallback=move || view! { <div class="flex justify-center items-center h-64">
                        <div class="animate-pulse bg-white rounded-lg shadow-md w-full max-w-2xl h-64 flex items-center justify-center">
                            <p class="text-gray-400">"Loading questions..."</p>
                        </div>
                    </div> }
                >
                    {move || match (questions.get(), test_details.get()) {
                        (None, _) => view! {<div class="text-center py-8">"Loading..."</div>}.into_view(),
                        (Some(questions), _) if questions.is_empty() => {
                            view! {<div class="text-center py-8 text-red-500">"No questions found for this test ID."</div>}.into_view()
                        },
                        (Some(questions), _) => {
                            let total_questions = questions.len();

                            let current_question = create_memo(move |_| {
                                questions.get(current_card_index.get()).cloned().unwrap_or_else(|| {
                                    log::warn!("Question index out of bounds");
                                    questions.first().cloned().unwrap_or_else(|| panic!("No questions available"))
                                })
                            });

                            view! {
                                <div class="flex flex-col items-center justify-center">
                                    {/* Progress Bar */}
                                    <div class="w-full max-w-2xl mb-4">
                                        <div class="flex justify-between mb-1 text-xs text-gray-700">
                                            <span>Progress</span>
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
                                            {" / "}
                                            {total_questions}
                                            <span class="ml-2 text-purple-600 font-semibold">
                                                {move || current_question().point_value}
                                                {" pts"}
                                            </span>
                                        </span>
                                    </div>

                                    {/* Flash Card */}
                                    <div class="bg-white rounded-xl shadow-lg overflow-hidden w-full max-w-2xl" style="min-height: 450px;">
                                        <div class="p-8 flex flex-col justify-start items-center w-full h-full overflow-y-auto">
                                            {/* Question Section */}
                                            <div class="text-center w-full overflow-auto mb-6">
                                                <p class="text-4xl sm:text-3xl font-bold text-gray-800 break-words mb-8">
                                                    {move || current_question().word_problem.clone()}
                                                </p>
                                            </div>

                                            {/* Answer Section */}
                                            <Show when=move || matches!(role.get(), Role::Teacher)>
                                                <div class="w-full mt-2">
                                                    <label class="block text-sm font-medium text-gray-700 mb-2">
                                                        "Your Answer:"
                                                    </label>
                                                    {move || {
                                                        let q = current_question();
                                                        match q.question_type {
                                                            QuestionType::MultipleChoice => view! {
                                                                <div class="space-y-2 max-h-48 overflow-y-auto">
                                                                    <For
                                                                        each=move || q.options.clone()
                                                                        key=|option| option.clone()
                                                                        children=move |option| {
                                                                            let option_value = option.clone();
                                                                            let option_value_clone = option_value.clone();
                                                                            let qnumber = q.qnumber;
                                                                            let is_checked = create_memo(move |_| {
                                                                                responses.with(|r| {
                                                                                    r.get(&qnumber)
                                                                                     .map(|resp| resp.answer == option_value_clone.clone())
                                                                                     .unwrap_or(false)
                                                                                })
                                                                            });

                                                                            view! {
                                                                                <label class="flex items-center p-3 rounded-lg border border-gray-200 hover:border-blue-400 hover:bg-blue-50 transition-colors cursor-pointer">
                                                                                    <input
                                                                                        type="radio"
                                                                                        name=format!("q_{}", qnumber)
                                                                                        value=option_value.clone()
                                                                                        class="h-4 w-4 text-blue-600 focus:ring-blue-500"
                                                                                        prop:checked=move || is_checked()
                                                                                        prop:disabled=should_disable_inputs.get()
                                                                                        on:change=move |ev| {
                                                                                            if !should_disable_inputs.get() {
                                                                                                let value = event_target_value(&ev);
                                                                                                handle_answer_change(qnumber, value);
                                                                                            }
                                                                                        }
                                                                                    />
                                                                                    <span class="ml-2 break-words">{option_value}</span>
                                                                                </label>
                                                                            }
                                                                        }
                                                                    />
                                                                </div>
                                                            },
                                                            QuestionType::TrueFalse => {
                                                                let qnumber = q.qnumber;
                                                                let is_true = create_memo(move |_| {
                                                                    responses.with(|r| {
                                                                        r.get(&qnumber)
                                                                         .map(|resp| resp.answer == "true")
                                                                         .unwrap_or(false)
                                                                    })
                                                                });
                                                                let is_false = create_memo(move |_| {
                                                                    responses.with(|r| {
                                                                        r.get(&qnumber)
                                                                         .map(|resp| resp.answer == "false")
                                                                         .unwrap_or(false)
                                                                    })
                                                                });

                                                                view! {
                                                                    <div class="w-full flex flex-col sm:flex-row gap-4 items-center justify-center">
                                                                        <button
                                                                            type="button"
                                                                            class="px-6 py-3 w-full rounded-lg font-medium text-center transition-colors"
                                                                            class:bg-white={move || !is_true()}
                                                                            class:text-gray-800={move || !is_true()}
                                                                            class:border-gray-200={move || !is_true()}
                                                                            class:border={move || !is_true()}
                                                                            class:bg-green-500={move || is_true()}
                                                                            class:text-white={move || is_true()}
                                                                            class:border-transparent={move || is_true()}
                                                                            class:cursor-not-allowd={should_disable_inputs()}
                                                                            on:click=move |_| {
                                                                                if !should_disable_inputs.get() {
                                                                                    handle_answer_change(qnumber, "true".to_string());
                                                                                }
                                                                            }
                                                                        >
                                                                            "Yes"
                                                                        </button>
                                                                        <button
                                                                            type="button"
                                                                            class="px-6 py-3 w-full rounded-lg font-medium text-center transition-colors"
                                                                            class:bg-white={move || !is_false()}
                                                                            class:text-gray-800={move || !is_false()}
                                                                            class:border-gray-200={move || !is_false()}
                                                                            class:border={move || !is_false()}
                                                                            class:bg-red-500={move || is_false()}
                                                                            class:text-white={move || is_false()}
                                                                            class:border-transparent={move || is_false()}
                                                                            class:cursor-not-allowed={should_disable_inputs()}
                                                                            on:click=move |_| {
                                                                                if !should_disable_inputs.get() {
                                                                                    handle_answer_change(qnumber, "false".to_string());
                                                                                }
                                                                            }
                                                                        >
                                                                            "No"
                                                                        </button>
                                                                    </div>
                                                                }
                                                            },
                                                            _ => {
                                                                let qnumber = q.qnumber;
                                                                let answer_value = create_memo(move |_| {
                                                                    responses.with(|r| {
                                                                        r.get(&qnumber)
                                                                         .map(|resp| resp.answer.clone())
                                                                         .unwrap_or_default()
                                                                    })
                                                                });

                                                                view! {
                                                                    <div>
                                                                        <textarea
                                                                            class="w-full p-3 border border-gray-200 rounded-lg focus:ring-blue-500 focus:border-blue-500"
                                                                            prop:value=move || answer_value()
                                                                            prop:disabled={should_disable_inputs()}
                                                                            on:input=move |ev| {
                                                                                if !should_disable_inputs.get() {
                                                                                    let value = event_target_value(&ev);
                                                                                    handle_answer_change(qnumber, value);
                                                                                }
                                                                            }
                                                                            placeholder="Enter your answer here..."
                                                                            rows="3"
                                                                        ></textarea>
                                                                    </div>
                                                                }
                                                            }
                                                        }
                                                    }}
                                                </div>
                                            </Show>

                                            {/* Teacher Comments Section - Only shown for teachers */}
                                            <Show when=move || matches!(role.get(), Role::Teacher)>
                                                <div class="w-full mt-4">
                                                    <label class="block text-sm font-medium text-gray-700 mb-2">
                                                        "Teacher Comments:"
                                                    </label>
                                                    {move || {
                                                        let qnumber = current_question().qnumber;
                                                        let comment_value = create_memo(move |_| {
                                                            responses.with(|r| {
                                                                r.get(&qnumber)
                                                                    .map(|resp| resp.comment.clone())
                                                                    .unwrap_or_default()
                                                            })
                                                        });
                                                        view! {
                                                            <div>
                                                                <textarea
                                                                    class="w-full p-3 border border-gray-200 rounded-lg focus:ring-blue-500 focus:border-blue-500"
                                                                    prop:value=move || comment_value()
                                                                    on:input=move |ev| {
                                                                        let value = event_target_value(&ev);
                                                                        handle_comment_change(qnumber, value);
                                                                    }
                                                                    placeholder="Add teacher comments or notes here..."
                                                                    rows="2"
                                                                ></textarea>
                                                            </div>
                                                        }
                                                    }}
                                                </div>
                                            </Show>
                                        </div>
                                    </div>

                                    {/* Navigation Buttons */}
                                    <Show when=move || is_test_active.get() || matches!(role.get(), Role::Teacher)>
                                        <div class="flex flex-wrap items-center justify-center gap-4 mt-8">
                                            <button
                                                class="flex items-center justify-center px-5 py-2 bg-white border border-gray-200 rounded-lg shadow-sm text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                                                disabled=move || (current_card_index.get() == 0 || should_disable_inputs())
                                                on:click=go_to_previous_card
                                            >
                                                <span class="mr-1">"←"</span>
                                                "Previous"
                                            </button>

                                            {move || {
                                                let is_last = current_card_index.get() == total_questions - 1;

                                                if is_last && matches!(role.get(), Role::Teacher) && is_test_active.get() && !is_submitted.get() {
                                                    view! {
                                                        <button
                                                            class="flex items-center justify-center px-5 py-2 bg-gradient-to-r from-blue-600 to-purple-600 text-white rounded-lg shadow-sm hover:from-blue-700 hover:to-purple-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                                            on:click=move |_| {
                                                                handle_submit.dispatch(());
                                                            }
                                                            disabled=move || (selected_student_id.get().is_none() || should_disable_inputs())
                                                        >
                                                            "Submit Assessment"
                                                            <span class="ml-1">"✓"</span>
                                                        </button>
                                                    }.into_view()
                                                } else if !is_last {
                                                    view! {
                                                        <button
                                                            class="flex items-center justify-center px-5 py-2 bg-gradient-to-r from-blue-600 to-purple-600 text-white rounded-lg shadow-sm hover:from-blue-700 hover:to-purple-700 transition-colors"
                                                            on:click=go_to_next_card
                                                            disabled=move || {should_disable_inputs}
                                                        >
                                                            "Next"
                                                            <span class="ml-1">"→"</span>
                                                        </button>
                                                    }.into_view()
                                                } else {
                                                    view! { <div></div> }.into_view()
                                                }
                                            }}
                                        </div>
                                    </Show>

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
                                                        let navigate=leptos_router::use_navigate();
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
                        <h3 class="text-xl font-medium mb-4">Waiting for Test to Start</h3>
                        <p class="text-gray-600 mb-6">Your teacher will start the test soon. Please stand by.</p>
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

// Student select component remains the same from original flash card set
#[component]
pub fn StudentSelect(set_selected_student_id: WriteSignal<Option<i32>>) -> impl IntoView {
    let (students, set_students) = create_signal(Vec::new());
    let get_students_action = create_action(|_: &()| async move {
        match get_students().await {
            Ok(fetched_students) => fetched_students,
            Err(e) => {
                log::error!("Failed to fetch students: {}", e);
                Vec::new()
            }
        }
    });

    // Dispatch action only once on component mount
    create_effect(move |_| {
        get_students_action.dispatch(());
    });

    // Update students when data is received
    create_effect(move |_| {
        if let Some(result) = get_students_action.value().get() {
            set_students.set(result);
        }
    });

    view! {
        <div class="mb-4 max-w-[20rem]">
            <label class="block text-sm font-medium mb-2">"Select Student:"</label>
            <select
                class="w-full p-2 border rounded-md"
                on:change=move |ev| {
                    let value = event_target_value(&ev).parse().ok();
                    set_selected_student_id.set(value);
                }
            >
                <option value="">"Select a student..."</option>
                {move || students.get().into_iter().map(|student| {
                    view! {
                        <option value={student.student_id.to_string()}>
                            {format!("{} {} - {}", student.firstname, student.lastname, student.student_id)}
                        </option>
                    }
                }).collect_view()}
            </select>
        </div>
    }
}
