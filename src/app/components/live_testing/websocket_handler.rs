use super::types::*;
use crate::app::models::question::Question;
use crate::app::models::user::SessionUser;
use crate::app::server_functions::websocket_sessions::cleanup_teacher_session_endpoint;
use crate::app::websockets::lobby::AnonymousStudent;
use leptos::*;
use log;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration as StdDuration;
use uuid::Uuid;

#[cfg(feature = "hydrate")]
use {
    wasm_bindgen::{closure::Closure, JsCast, JsValue},
    web_sys::{CloseEvent, MessageEvent, WebSocket},
};

pub struct WebSocketActions {
    pub start_test: Callback<()>,
    pub end_test: Callback<()>,
    pub go_to_next_card: Callback<()>,
    pub go_to_previous_card: Callback<()>,
    pub handle_answer_change: Callback<(i32, String)>,
    pub handle_comment_change: Callback<(i32, String)>,
    pub handle_weighted_selection: Callback<(i32, Vec<String>)>, // New for weighted multiple choice
    pub request_participants: Callback<()>,
    pub send_heartbeat: Callback<()>,
    pub join_as_anonymous_student: Callback<AnonymousStudent>,
}

#[cfg(feature = "hydrate")]
pub fn use_websocket_connection(
    room_id: Signal<Option<Uuid>>,
    user: Signal<Option<SessionUser>>,
    set_connection_status: WriteSignal<ConnectionStatus>,
    set_error_message: WriteSignal<Option<String>>,
    set_role: WriteSignal<Role>,
    set_connected_students: WriteSignal<Vec<ConnectedStudent>>,
    set_responses: WriteSignal<HashMap<i32, QuestionResponse>>,
    set_current_card_index: WriteSignal<usize>,
    set_remaining_time: WriteSignal<Option<i32>>,
    set_is_test_active: WriteSignal<bool>,
    set_is_submitted: WriteSignal<bool>,
    test_id: Signal<String>,
    session_room_id: Signal<Option<Uuid>>,
    questions: Signal<Option<Vec<Question>>>,
) -> WebSocketActions {
    use gloo_timers::future::TimeoutFuture;
    let (ws, set_ws) = create_signal::<Option<WebSocket>>(None);

    // Connect to WebSocket when room_id changes
    create_effect(move |_| {
        if let Some(session_id) = room_id.get() {
            spawn_local(async move {
                //wait for any previous connection to close
                gloo_timers::future::TimeoutFuture::new(200).await;

                connect_to_session(
                    session_id,
                    set_ws,
                    user,
                    set_connection_status,
                    set_error_message,
                    set_role,
                    set_connected_students,
                    set_responses,
                    set_current_card_index,
                    set_remaining_time,
                    set_is_test_active,
                    set_is_submitted,
                    ws,
                );
            });
        }
    });

    // Send message through WebSocket
    let send_ws_message = move |message: String| {
        if let Some(socket) = ws.get() {
            match socket.send_with_str(&message) {
                Ok(_) => log::debug!("Sent WebSocket message: {}", message),
                Err(e) => log::error!("Failed to send WebSocket message: {:?}", e),
            }
        }
    };

    // Heartbeat sender to keep connection alive
    let send_heartbeat = Callback::new(move |_| {
        let heartbeat_message = json!({
            "type": "heartbeat",
            "timestamp": chrono::Utc::now().timestamp()
        })
        .to_string();

        send_ws_message(heartbeat_message);
    });

    // Start test handler
    let start_test = Callback::new(move |_| {
        if !test_id.get().is_empty() {
            let message = json!({
                "type": "test_message",
                "test_message_type": "start_test",
                "payload": {
                    "test_id": test_id.get()
                }
            })
            .to_string();

            send_ws_message(message);

            // Start timer
            let duration_minutes = 60;
            set_remaining_time.set(Some(duration_minutes * 60));

            // Start timer countdown
            let ws_clone = ws.clone();
            let _timer_handle = super::utils::set_interval_with_handle(
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

                                    if let Some(socket) = ws_clone.get() {
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

                                if let Some(socket) = ws_clone.get() {
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
    });

    // End test handler
    let end_test = Callback::new(move |_| {
        let test_id_value = test_id.get().clone();

        let end_message = json!({
            "type": "test_message",
            "test_message_type": "end_test",
            "payload": {}
        })
        .to_string();

        send_ws_message(end_message);
    });

    // Navigation handlers
    let go_to_next_card = Callback::new(move |_| {
        set_current_card_index.update(|index| {
            if let Some(questions_vec) = questions.get() {
                *index = (*index + 1).min(questions_vec.len() - 1);

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
    });

    let go_to_previous_card = Callback::new(move |_| {
        set_current_card_index.update(|index| {
            *index = index.saturating_sub(1);

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
    });

    // Answer change handler (for regular questions)
    let handle_answer_change = Callback::new(move |(qnumber, value): (i32, String)| {
        // Update local state
        set_responses.update(|r| {
            let response = r.entry(qnumber).or_insert(QuestionResponse::new());
            response.answer = value.clone();
        });

        // Send to server
        let answer_message = json!({
            "type": "test_message",
            "test_message_type": "submit_answer",
            "payload": {
                "question_id": qnumber,
                "answer": value,
                "answer_type": "regular"
            }
        })
        .to_string();

        send_ws_message(answer_message);
    });

    let handle_weighted_selection =
        Callback::new(move |(qnumber, selected_options): (i32, Vec<String>)| {
            // Update local state
            set_responses.update(|r| {
                let response = r.entry(qnumber).or_insert(QuestionResponse::new());
                response.selected_options = Some(selected_options.clone());
                // Also update answer field with JSON for compatibility
                response.answer = serde_json::to_string(&selected_options).unwrap_or_default();
            });

            // Send to server
            let weighted_message = json!({
                "type": "test_message",
                "test_message_type": "submit_answer",
                "payload": {
                    "question_id": qnumber,
                    "selected_options": selected_options,
                    "answer_type": "weighted_multiple_choice"
                }
            })
            .to_string();

            send_ws_message(weighted_message);
        });

    // Comment change handler
    let handle_comment_change = Callback::new(move |(qnumber, value): (i32, String)| {
        set_responses.update(|r| {
            let response = r.entry(qnumber).or_insert(QuestionResponse::new());
            response.comment = value.clone();
        });

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
    });

    let join_as_anonymous_student = Callback::new(move |student_data: AnonymousStudent| {
        let join_message = json!({
            "type": "anonymous_student_join",
            "name": student_data.name,
            "student_id": student_data.id,
            "timestamp": chrono::Utc::now().timestamp()
        })
        .to_string();

        send_ws_message(join_message);

        log::info!(
            "Sent anonymous student join message for: {}",
            student_data.name
        );
    });

    // Request participants
    let request_participants = Callback::new(move |_| {
        let request = json!({
            "type": "request_participants"
        })
        .to_string();

        send_ws_message(request);
    });

    // Cleanup on unmount
    on_cleanup(move || {
        if let Some(socket) = ws.get() {
            let _ = socket.close();
        }
    });

    WebSocketActions {
        start_test,
        end_test,
        go_to_next_card,
        go_to_previous_card,
        handle_answer_change,
        handle_comment_change,
        handle_weighted_selection,
        request_participants,
        send_heartbeat,
        join_as_anonymous_student,
    }
}

#[cfg(feature = "hydrate")]
fn connect_to_session(
    session_id: Uuid,
    set_ws: WriteSignal<Option<WebSocket>>,
    user: Signal<Option<SessionUser>>,
    set_connection_status: WriteSignal<ConnectionStatus>,
    set_error_message: WriteSignal<Option<String>>,
    set_role: WriteSignal<Role>,
    set_connected_students: WriteSignal<Vec<ConnectedStudent>>,
    set_responses: WriteSignal<HashMap<i32, QuestionResponse>>,
    set_current_card_index: WriteSignal<usize>,
    set_remaining_time: WriteSignal<Option<i32>>,
    set_is_test_active: WriteSignal<bool>,
    set_is_submitted: WriteSignal<bool>,
    ws: ReadSignal<Option<WebSocket>>,
) {
    spawn_local(async move {
        let protocol = if web_sys::window().unwrap().location().protocol().unwrap() == "https:" {
            "wss"
        } else {
            "ws"
        };
        let host = web_sys::window().unwrap().location().host().unwrap();
        let ws_url = format!("{protocol}://{host}/api/ws/{session_id}");

        log::info!("Connecting to WebSocket at: {}", ws_url);

        // Close any existing connection
        if let Some(existing_ws) = ws.get_untracked() {
            let _ = existing_ws.close();
        }

        set_connection_status.set(ConnectionStatus::Connecting);
        set_error_message.set(None);
        set_connected_students.set(Vec::new());

        match WebSocket::new(&ws_url) {
            Ok(websocket) => {
                setup_websocket_handlers(
                    websocket.clone(),
                    user,
                    set_connection_status,
                    set_error_message,
                    set_role,
                    set_connected_students,
                    set_responses,
                    set_current_card_index,
                    set_remaining_time,
                    set_is_test_active,
                    set_is_submitted,
                    ws,
                );

                set_ws.set(Some(websocket));
            }
            Err(err) => {
                let error_msg = format!("WebSocket connection failed: {:?}", err);
                log::error!("{}", error_msg);
                set_connection_status.set(ConnectionStatus::Error);
                set_error_message.set(Some(error_msg));
            }
        }
    });
}

#[cfg(feature = "hydrate")]
fn setup_websocket_handlers(
    websocket: WebSocket,
    user: Signal<Option<SessionUser>>,
    set_connection_status: WriteSignal<ConnectionStatus>,
    set_error_message: WriteSignal<Option<String>>,
    set_role: WriteSignal<Role>,
    set_connected_students: WriteSignal<Vec<ConnectedStudent>>,
    set_responses: WriteSignal<HashMap<i32, QuestionResponse>>,
    set_current_card_index: WriteSignal<usize>,
    set_remaining_time: WriteSignal<Option<i32>>,
    set_is_test_active: WriteSignal<bool>,
    set_is_submitted: WriteSignal<bool>,
    ws: ReadSignal<Option<WebSocket>>,
) {
    // Setup message handler
    let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
        if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
            let message = text.as_string().unwrap();
            log::info!("WebSocket message received: {}", message);

            match serde_json::from_str::<Value>(&message) {
                Ok(json_value) => {
                    handle_websocket_message(
                        json_value,
                        set_role,
                        set_connected_students,
                        set_responses,
                        set_current_card_index,
                        set_remaining_time,
                        set_is_test_active,
                        set_is_submitted,
                    );
                }
                Err(err) => {
                    log::error!(
                        "Failed to parse WebSocket message: {:?}. Raw message: {}",
                        err,
                        message
                    );
                }
            }
        }
    }) as Box<dyn FnMut(MessageEvent)>);

    // Setup onopen handler
    let onopen_callback = Closure::wrap(Box::new(move |_| {
        log::info!("🎉 WebSocket connection established");
        set_connection_status.set(ConnectionStatus::Connected);
        set_error_message.set(None);

        // ROBUST: Try to send user_info with retries
        spawn_local(async move {
            let max_attempts = 5;

            for attempt in 1..=max_attempts {
                log::info!("📤 Attempting to send user_info (attempt {})", attempt);

                if let Some(current_user) = user.get_untracked() {
                    log::info!("✅ User context available on attempt {}", attempt);

                    let user_info = json!({
                        "type": "user_info",
                        "user_id": current_user.id,
                        "role": current_user.role,
                        "is_teacher": current_user.is_teacher(),
                        "is_admin": current_user.is_admin(),
                        "is_reconnecting": true,
                    });

                    if let Some(socket) = ws.get() {
                        match socket.send_with_str(&user_info.to_string()) {
                            Ok(_) => {
                                log::info!("✅ user_info sent successfully on attempt {}", attempt);

                                // Send participants request after successful user_info
                                gloo_timers::future::TimeoutFuture::new(500).await;

                                let participants_request = json!({
                                    "type": "request_participants"
                                })
                                .to_string();

                                let _ = socket.send_with_str(&participants_request);
                                log::info!("✅ Participants request sent");
                                return; // Success!
                            }
                            Err(e) => {
                                log::error!(
                                    "❌ Failed to send user_info on attempt {}: {:?}",
                                    attempt,
                                    e
                                );
                            }
                        }
                    } else {
                        log::error!("❌ No WebSocket available on attempt {}", attempt);
                    }
                } else {
                    log::warn!("⏳ User context not available on attempt {}", attempt);
                }

                // Wait before retry (increasing delay)
                gloo_timers::future::TimeoutFuture::new(300 * attempt as u32).await;
            }

            log::error!(
                "❌ Failed to send user_info after {} attempts",
                max_attempts
            );
        });
    }) as Box<dyn FnMut(JsValue)>);

    // Setup onclose handler
    let onclose_callback = Closure::wrap(Box::new(move |e: CloseEvent| {
        log::info!("WebSocket closed: {} - {}", e.code(), e.reason());
        set_connection_status.set(ConnectionStatus::Disconnected);

        // Only cleanup if this was an established session (not initial connection failure)
        if e.code() != 1006 && e.code() != 1000 {
            // Avoid cleanup on normal close or connection never established
            if let Some(current_user) = user.get_untracked() {
                if current_user.is_teacher() || current_user.is_admin() {
                    spawn_local(async move {
                        if let Err(e) =
                            cleanup_teacher_session_endpoint(current_user.id.try_into().unwrap())
                                .await
                        {
                            log::error!(
                                "Failed to cleanup teacher session on WebSocket close: {}",
                                e
                            );
                        }
                    });
                }
            }
        }
    }) as Box<dyn FnMut(CloseEvent)>);

    // Setup onerror handler
    let onerror_callback = Closure::wrap(Box::new(move |_e| {
        let error_msg = "WebSocket connection failed".to_string();
        log::error!("WebSocket error occurred: {}", error_msg);
        set_connection_status.set(ConnectionStatus::Error);
        set_error_message.set(Some(error_msg));
    }) as Box<dyn FnMut(JsValue)>);

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
}

#[cfg(feature = "hydrate")]
fn handle_websocket_message(
    json_value: Value,
    set_role: WriteSignal<Role>,
    set_connected_students: WriteSignal<Vec<ConnectedStudent>>,
    set_responses: WriteSignal<HashMap<i32, QuestionResponse>>,
    set_current_card_index: WriteSignal<usize>,
    set_remaining_time: WriteSignal<Option<i32>>,
    set_is_test_active: WriteSignal<bool>,
    set_is_submitted: WriteSignal<bool>,
) {
    if let Some(msg_type) = json_value.get("type").and_then(|t| t.as_str()) {
        log::info!("Processing message type: {}", msg_type);

        match msg_type {
            "role_assigned" => {
                if let Some(role_str) = json_value.get("role").and_then(|r| r.as_str()) {
                    match role_str {
                        "teacher" => set_role.set(Role::Teacher),
                        "student" => set_role.set(Role::Student),
                        _ => set_role.set(Role::Unknown),
                    }
                }
            }
            "session_reset" => {
                log::info!("Session reset by teacher - clearing local state");
                set_responses.set(HashMap::new());
                set_current_card_index.set(0);
                set_is_test_active.set(false);
                set_is_submitted.set(false);
                set_remaining_time.set(None);
            }
            "participants_list" => {
                if let Some(participants) =
                    json_value.get("participants").and_then(|p| p.as_array())
                {
                    let connected_list: Vec<ConnectedStudent> = participants
                        .iter()
                        .filter_map(|p| {
                            let id = p.get("id")?.as_str()?;
                            let name = p.get("name")?.as_str().unwrap_or("Unknown");
                            let user_type = p.get("type")?.as_str().unwrap_or("User");
                            let status = p.get("status")?.as_str().unwrap_or("Connected");

                            Some(ConnectedStudent {
                                student_id: id.to_string(),
                                name: format!("{} ({})", name, user_type),
                                status: status.to_string(),
                            })
                        })
                        .collect();

                    set_connected_students.set(connected_list);
                }
            }
            "student_joined" | "user_joined" => {
                // Handle new participants joining
                let is_student = msg_type == "student_joined";
                let id_field = if is_student { "student_id" } else { "id" };
                let data_field = if is_student {
                    "student_data"
                } else {
                    "user_data"
                };
                let name_field = if is_student { "name" } else { "username" };

                if let Some(user_id) = json_value.get(id_field).and_then(|s| s.as_str()) {
                    if let Some(user_data) = json_value.get(data_field) {
                        let name = user_data
                            .get(name_field)
                            .and_then(|n| n.as_str())
                            .unwrap_or("Unknown");

                        let user_type = if is_student { "Student" } else { "Teacher" };
                        let display_name = format!("{} ({})", name, user_type);

                        set_connected_students.update(|students| {
                            if let Some(pos) = students.iter().position(|s| s.student_id == user_id)
                            {
                                students[pos].status = "Connected".to_string();
                                students[pos].name = display_name;
                            } else {
                                students.push(ConnectedStudent {
                                    student_id: user_id.to_string(),
                                    name: display_name,
                                    status: "Connected".to_string(),
                                });
                            }
                        });
                    }
                }
            }
            "anonymous_student_joined" => {
                // Handle confirmation that anonymous student has joined
                if let Some(student_id) = json_value.get("student_id").and_then(|s| s.as_str()) {
                    if let Some(name) = json_value.get("name").and_then(|n| n.as_str()) {
                        let display_name = format!("{} (Anonymous Student)", name);

                        set_connected_students.update(|students| {
                            if let Some(pos) =
                                students.iter().position(|s| s.student_id == student_id)
                            {
                                students[pos].status = "Connected".to_string();
                                students[pos].name = display_name;
                            } else {
                                students.push(ConnectedStudent {
                                    student_id: student_id.to_string(),
                                    name: display_name,
                                    status: "Connected".to_string(),
                                });
                            }
                        });

                        log::info!("Anonymous student joined: {} ({})", name, student_id);
                    }
                }
            }
            "student_left" | "user_left" => {
                let is_student = msg_type == "student_left";
                let id_field = if is_student { "student_id" } else { "id" };

                if let Some(user_id) = json_value.get(id_field).and_then(|s| s.as_str()) {
                    set_connected_students.update(|students| {
                        if let Some(pos) = students.iter().position(|s| s.student_id == user_id) {
                            students[pos].status = "Disconnected".to_string();
                        }
                    });
                }
            }
            "student_answer" => {
                if let Some(answer_data) = json_value.get("answer_data") {
                    if let Some(qnumber) = answer_data.get("question_id").and_then(|q| q.as_i64()) {
                        let qnumber = qnumber as i32;

                        set_responses.update(|r| {
                            let response = r.entry(qnumber).or_insert(QuestionResponse::new());

                            // Handle different answer types
                            if let Some(answer_type) =
                                answer_data.get("answer_type").and_then(|t| t.as_str())
                            {
                                match answer_type {
                                    "weighted_multiple_choice" => {
                                        if let Some(selected_options) = answer_data
                                            .get("selected_options")
                                            .and_then(|opts| opts.as_array())
                                        {
                                            let options: Vec<String> = selected_options
                                                .iter()
                                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                                .collect();
                                            response.selected_options = Some(options.clone());
                                            response.answer =
                                                serde_json::to_string(&options).unwrap_or_default();
                                        }
                                    }
                                    _ => {
                                        if let Some(answer) =
                                            answer_data.get("answer").and_then(|a| a.as_str())
                                        {
                                            response.answer = answer.to_string();
                                        }
                                    }
                                }
                            } else {
                                // Fallback for regular answers
                                if let Some(answer) =
                                    answer_data.get("answer").and_then(|a| a.as_str())
                                {
                                    response.answer = answer.to_string();
                                }
                            }
                        });
                    }
                }
            }
            "focus_question" => {
                if let Some(question_data) = json_value.get("question_data") {
                    if let Some(index) = question_data.get("index").and_then(|i| i.as_i64()) {
                        set_current_card_index.set(index as usize);
                    }
                }
            }
            "time_update" => {
                if let Some(time_data) = json_value.get("time_data") {
                    if let Some(remaining) = time_data.get("remaining").and_then(|r| r.as_i64()) {
                        set_remaining_time.set(Some(remaining as i32));
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
            "heartbeat" => {
                log::debug!("Received heartbeat from client");
            }
            _ => {
                log::debug!("Unhandled message type: {}", msg_type);
            }
        }
    }
}

// Non-hydrate version (no-op)
#[cfg(not(feature = "hydrate"))]
pub fn use_websocket_connection(
    _room_id: Signal<Option<Uuid>>,
    _user: Signal<Option<SessionUser>>,
    _set_connection_status: WriteSignal<ConnectionStatus>,
    _set_error_message: WriteSignal<Option<String>>,
    _set_role: WriteSignal<Role>,
    _set_connected_students: WriteSignal<Vec<ConnectedStudent>>,
    _set_responses: WriteSignal<HashMap<i32, QuestionResponse>>,
    _set_current_card_index: WriteSignal<usize>,
    _set_remaining_time: WriteSignal<Option<i32>>,
    _set_is_test_active: WriteSignal<bool>,
    _set_is_submitted: WriteSignal<bool>,
    _test_id: Signal<String>,
    _session_room_id: Signal<Option<Uuid>>,
    _questions: Signal<Option<Vec<Question>>>,
) -> WebSocketActions {
    WebSocketActions {
        start_test: Callback::new(|_| {}),
        end_test: Callback::new(|_| {}),
        go_to_next_card: Callback::new(|_| {}),
        go_to_previous_card: Callback::new(|_| {}),
        handle_answer_change: Callback::new(|_| {}),
        handle_comment_change: Callback::new(|_| {}),
        handle_weighted_selection: Callback::new(|_| {}), // NEW
        request_participants: Callback::new(|_| {}),
        send_heartbeat: Callback::new(|_| {}),
        join_as_anonymous_student: Callback::new(|_| {}),
    }
}
