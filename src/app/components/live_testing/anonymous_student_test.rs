use super::types::{ConnectionStatus, QuestionResponse, Role};
use crate::app::models::question::{Question, QuestionType};
use crate::app::server_functions::{questions::get_questions, tests::get_tests};
use leptos::*;
use leptos_router::*;
use log;
use std::collections::HashMap;
use uuid::Uuid; // Import QuestionResponse from types

#[cfg(feature = "hydrate")]
use {
    serde_json::{json, Value},
    wasm_bindgen::{closure::Closure, JsCast, JsValue},
    web_sys::{CloseEvent, MessageEvent, WebSocket},
};

#[component]
pub fn AnonymousStudentTest() -> impl IntoView {
    let params = use_params_map();
    let test_id = move || params.with(|params| params.get("test_id").cloned().unwrap_or_default());
    let session_id_str =
        move || params.with(|params| params.get("session_id").cloned().unwrap_or_default());

    let (student_name, set_student_name) = create_signal(String::new());
    let (student_id_input, set_student_id_input) = create_signal(String::new());
    let (has_joined, set_has_joined) = create_signal(false);
    let (error_message, set_error_message) = create_signal(None::<String>);
    let (connection_status, set_connection_status) = create_signal(ConnectionStatus::Disconnected);
    let (is_test_active, set_is_test_active) = create_signal(false);
    let (current_card_index, set_current_card_index) = create_signal(0);
    let (responses, set_responses) = create_signal(HashMap::<i32, QuestionResponse>::new());
    let (remaining_time, set_remaining_time) = create_signal::<Option<i32>>(None);

    #[cfg(feature = "hydrate")]
    let (ws, set_ws) = create_signal::<Option<WebSocket>>(None);

    // Get session ID as UUID
    let session_id = create_memo(move |_| Uuid::parse_str(&session_id_str()).ok());

    // Fetch test details and questions
    let test_details = create_resource(test_id.clone(), move |tid| async move {
        if tid.is_empty() {
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

    // Join test function
    let join_test = move |_| {
        let name = student_name.get().trim().to_string();
        let id = student_id_input.get().trim().to_string();

        if name.is_empty() {
            set_error_message.set(Some("Please enter your name".to_string()));
            return;
        }

        if id.is_empty() {
            set_error_message.set(Some("Please enter your student ID".to_string()));
            return;
        }

        set_error_message.set(None);
        set_has_joined.set(true);
    };

    // WebSocket connection for anonymous students
    #[cfg(feature = "hydrate")]
    let connect_to_session = create_action(move |session_uuid: &Uuid| {
        let session_uuid = *session_uuid;
        let student_name_val = student_name.get();
        let student_id_val = student_id_input.get();
        let test_id_val = test_id();

        async move {
            let protocol = if web_sys::window().unwrap().location().protocol().unwrap() == "https:"
            {
                "wss"
            } else {
                "ws"
            };
            let host = web_sys::window().unwrap().location().host().unwrap();
            let ws_url = format!("{protocol}://{host}/api/ws/{session_uuid}");

            log::info!("Anonymous student connecting to WebSocket at: {}", ws_url);

            set_connection_status.set(ConnectionStatus::Connecting);
            set_error_message.set(None);

            match WebSocket::new(&ws_url) {
                Ok(websocket) => {
                    // Setup message handler
                    let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                        if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                            let message = text.as_string().unwrap();
                            log::info!("Anonymous student received message: {}", message);

                            match serde_json::from_str::<Value>(&message) {
                                Ok(json_value) => {
                                    if let Some(msg_type) =
                                        json_value.get("type").and_then(|t| t.as_str())
                                    {
                                        match msg_type {
                                            "role_assigned" => {
                                                log::info!("Anonymous student role assigned");
                                            }
                                            "test_started" => {
                                                log::info!("Test started for anonymous student");
                                                set_is_test_active.set(true);
                                            }
                                            "test_ended" => {
                                                log::info!("Test ended for anonymous student");
                                                set_is_test_active.set(false);
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
                                            "teacher_comment" => {
                                                // Handle teacher comments if needed
                                                log::info!(
                                                    "Received teacher comment: {:?}",
                                                    json_value
                                                );
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
                    let onopen_callback = Closure::wrap(Box::new(move |_| {
                        log::info!("Anonymous student WebSocket connection established");
                        set_connection_status.set(ConnectionStatus::Connected);
                        set_error_message.set(None);

                        // Send anonymous student join message
                        let student_join_msg = json!({
                            "type": "anonymous_student_join",
                            "student_name": student_name_val,
                            "student_id": student_id_val,
                            "test_id": test_id_val
                        })
                        .to_string();

                        if let Some(socket) = ws.get() {
                            let _ = socket.send_with_str(&student_join_msg);
                            log::info!("Sent anonymous student join message");
                        }
                    })
                        as Box<dyn FnMut(JsValue)>);

                    // Setup onclose handler
                    let onclose_callback = Closure::wrap(Box::new(move |e: CloseEvent| {
                        log::info!(
                            "Anonymous student WebSocket closed: {} - {}",
                            e.code(),
                            e.reason()
                        );
                        set_connection_status.set(ConnectionStatus::Disconnected);
                    })
                        as Box<dyn FnMut(CloseEvent)>);

                    // Setup onerror handler
                    let onerror_callback = Closure::wrap(Box::new(move |_e| {
                        let error_msg = "WebSocket connection failed".to_string();
                        log::error!("Anonymous student WebSocket error: {}", error_msg);
                        set_connection_status.set(ConnectionStatus::Error);
                        set_error_message.set(Some(error_msg));
                    })
                        as Box<dyn FnMut(JsValue)>);

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

    // Connect when student joins
    create_effect(move |_| {
        if has_joined.get() {
            if let Some(session_uuid) = session_id.get() {
                #[cfg(feature = "hydrate")]
                connect_to_session.dispatch(session_uuid);
            } else {
                set_error_message.set(Some("Invalid session ID".to_string()));
            }
        }
    });

    // Handle answer submission
    #[cfg(feature = "hydrate")]
    let handle_answer_change = move |qnumber: i32, value: String| {
        // Update local state
        set_responses.update(|r| {
            let response = r.entry(qnumber).or_insert(QuestionResponse {
                answer: String::new(),
                comment: String::new(),
            });
            response.answer = value.clone();
        });

        // Send answer to teacher
        let answer_message = json!({
            "type": "test_message",
            "test_message_type": "submit_answer",
            "payload": {
                "question_id": qnumber,
                "answer": value
            }
        })
        .to_string();

        if let Some(socket) = ws.get() {
            let _ = socket.send_with_str(&answer_message);
        }
    };

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

    view! {
        <div class="min-h-screen bg-gray-50">
            <Show when=move || !has_joined.get()>
                // Student join form
                <div class="flex items-center justify-center min-h-screen">
                    <div class="bg-white p-8 rounded-lg shadow-md max-w-md w-full mx-4">
                        <div class="text-center mb-6">
                            <h2 class="text-2xl font-bold text-gray-800">"Join Test Session"</h2>
                            <p class="text-gray-600 mt-2">
                                {move || match &test_details.get() {
                                    Some(Some(test)) => format!("Test: {}", test.name),
                                    _ => format!("Test ID: {}", test_id())
                                }}
                            </p>
                        </div>

                        <Show when=move || error_message.get().is_some()>
                            <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
                                {move || error_message.get().unwrap_or_default()}
                            </div>
                        </Show>

                        <div class="space-y-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">
                                    "Your Name"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
                                    placeholder="Enter your full name"
                                    prop:value=move || student_name.get()
                                    on:input=move |ev| set_student_name.set(event_target_value(&ev))
                                />
                            </div>

                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">
                                    "Student ID"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
                                    placeholder="Enter your student ID"
                                    prop:value=move || student_id_input.get()
                                    on:input=move |ev| set_student_id_input.set(event_target_value(&ev))
                                />
                            </div>

                            <button
                                class="w-full px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors font-medium"
                                on:click=join_test
                            >
                                "Join Test"
                            </button>
                        </div>

                        <div class="mt-6 text-center">
                            <div class="text-xs text-gray-500 space-y-1">
                                <p>"💡 No account needed - just enter your info above"</p>
                                <p>"🔒 Your teacher will see your responses in real-time"</p>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || has_joined.get()>
                // Test interface for anonymous student
                <div class="p-4 max-w-screen h-screen overflow-y-auto bg-gray-50 mx-auto">
                    {/* Header */}
                    <div class="text-center mb-8">
                        <h2 class="text-2xl font-bold text-gray-800">
                            {move || match &test_details.get() {
                                Some(Some(test)) => format!("Test: {}", test.name),
                                _ => "Test Session".to_string()
                            }}
                        </h2>
                        <div class="mt-2 text-sm text-gray-600">
                            "Welcome, " {move || student_name.get()} " (ID: " {move || student_id_input.get()} ")"
                        </div>
                    </div>

                    {/* Connection Status */}
                    <div class="flex justify-center mb-4">
                        <div class="flex items-center space-x-2 px-3 py-1 rounded-full text-sm"
                             class:bg-green-100={move || matches!(connection_status.get(), ConnectionStatus::Connected)}
                             class:text-green-800={move || matches!(connection_status.get(), ConnectionStatus::Connected)}
                             class:bg-yellow-100={move || matches!(connection_status.get(), ConnectionStatus::Connecting)}
                             class:text-yellow-800={move || matches!(connection_status.get(), ConnectionStatus::Connecting)}
                             class:bg-red-100={move || matches!(connection_status.get(), ConnectionStatus::Error)}
                             class:text-red-800={move || matches!(connection_status.get(), ConnectionStatus::Error)}
                             class:bg-gray-100={move || matches!(connection_status.get(), ConnectionStatus::Disconnected)}
                             class:text-gray-800={move || matches!(connection_status.get(), ConnectionStatus::Disconnected)}>
                            <div class="w-2 h-2 rounded-full"
                                 class:bg-green-500={move || matches!(connection_status.get(), ConnectionStatus::Connected)}
                                 class:bg-yellow-500={move || matches!(connection_status.get(), ConnectionStatus::Connecting)}
                                 class:bg-red-500={move || matches!(connection_status.get(), ConnectionStatus::Error)}
                                 class:bg-gray-500={move || matches!(connection_status.get(), ConnectionStatus::Disconnected)}></div>
                            <span>{move || match connection_status.get() {
                                ConnectionStatus::Connected => "Connected",
                                ConnectionStatus::Connecting => "Connecting...",
                                ConnectionStatus::Error => "Connection Error",
                                ConnectionStatus::Disconnected => "Disconnected"
                            }}</span>
                        </div>
                    </div>

                    {/* Status and Timer */}
                    <div class="flex justify-center items-center mb-6 space-x-8">
                        <div class="text-sm text-gray-600">
                            <span class="font-medium">"Status: "</span>
                            {move || if is_test_active.get() { "Test Active" } else { "Waiting for Teacher" }}
                        </div>
                        <Show when=move || !formatted_time().is_empty()>
                            <div class="text-sm text-gray-600">
                                <span class="font-medium">"Time Remaining: "</span>
                                {move || formatted_time()}
                            </div>
                        </Show>
                    </div>

                    {/* Test Content */}
                    <Show when=move || is_test_active.get()>
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
                                    view! {<div class="text-center py-8 text-red-500">"No questions found."</div>}.into_view()
                                },
                                Some(questions_vec) => {
                                    // FIXED: Clone questions_vec to avoid borrow after move
                                    let questions_len = questions_vec.len();
                                    let questions_clone = questions_vec.clone();

                                    let current_question = create_memo(move |_| {
                                        questions_clone.get(current_card_index.get()).cloned().unwrap_or_else(|| {
                                            questions_clone.first().cloned().unwrap()
                                        })
                                    });

                                    view! {
                                        <div class="flex flex-col items-center justify-center">
                                            {/* Card Counter */}
                                            <div class="text-center mb-4">
                                                <span class="inline-flex items-center justify-center bg-white text-sm font-medium text-gray-700 px-3 py-1 rounded-full shadow-sm border border-gray-200">
                                                    {move || current_card_index.get() + 1}
                                                    " / "
                                                    {questions_len}
                                                    <span class="ml-2 text-purple-600 font-semibold">
                                                        {move || current_question().point_value}
                                                        " pts"
                                                    </span>
                                                </span>
                                            </div>

                                            {/* Question Card */}
                                            <div class="bg-white rounded-xl shadow-lg overflow-hidden w-full max-w-2xl" style="min-height: 400px;">
                                                <div class="p-8 flex flex-col justify-start items-center w-full h-full">
                                                    {/* Question */}
                                                    <div class="text-center w-full mb-6">
                                                        <p class="text-3xl font-bold text-gray-800 break-words">
                                                            {move || current_question().word_problem.clone()}
                                                        </p>
                                                    </div>

                                                    {/* Answer Input */}
                                                    <div class="w-full mt-4">
                                                        {move || {
                                                            let q = current_question();
                                                            match q.question_type {
                                                                QuestionType::MultipleChoice => view! {
                                                                    <div class="space-y-2">
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
                                                                                            on:change=move |ev| {
                                                                                                #[cfg(feature = "hydrate")]
                                                                                                {
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
                                                                }.into_view(),
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
                                                                                on:click=move |_| {
                                                                                    #[cfg(feature = "hydrate")]
                                                                                    handle_answer_change(qnumber, "true".to_string());
                                                                                }
                                                                            >
                                                                                "True"
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
                                                                                on:click=move |_| {
                                                                                    #[cfg(feature = "hydrate")]
                                                                                    handle_answer_change(qnumber, "false".to_string());
                                                                                }
                                                                            >
                                                                                "False"
                                                                            </button>
                                                                        </div>
                                                                    }.into_view()
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
                                                                                on:input=move |ev| {
                                                                                    #[cfg(feature = "hydrate")]
                                                                                    {
                                                                                        let value = event_target_value(&ev);
                                                                                        handle_answer_change(qnumber, value);
                                                                                    }
                                                                                }
                                                                                placeholder="Enter your answer here..."
                                                                                rows="4"
                                                                            ></textarea>
                                                                        </div>
                                                                    }.into_view()
                                                                }
                                                            }
                                                        }}
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_view()
                                }
                            }}
                        </Suspense>
                    </Show>

                    {/* Waiting Message */}
                    <Show when=move || !is_test_active.get()>
                        <div class="flex flex-col items-center justify-center py-12 max-w-md mx-auto">
                            <div class="bg-white p-8 rounded-lg shadow-md w-full text-center">
                                <h3 class="text-xl font-medium mb-4">"Waiting for Test to Start"</h3>
                                <p class="text-gray-600 mb-6">"Your teacher will start the test soon. Please stay on this page."</p>
                                <div class="animate-pulse flex justify-center">
                                    <div class="h-4 w-4 bg-blue-400 rounded-full mr-1"></div>
                                    <div class="h-4 w-4 bg-blue-500 rounded-full mr-1 animation-delay-200"></div>
                                    <div class="h-4 w-4 bg-blue-600 rounded-full animation-delay-400"></div>
                                </div>
                            </div>
                        </div>
                    </Show>

                    {/* Error Message */}
                    <Show when=move || error_message.get().is_some()>
                        <div class="max-w-4xl mx-auto mb-6">
                            <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
                                <strong>"Error: "</strong>
                                {move || error_message.get().unwrap_or_default()}
                            </div>
                        </div>
                    </Show>
                </div>
            </Show>
        </div>
    }
}
