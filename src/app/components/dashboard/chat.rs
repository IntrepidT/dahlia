use leptos::*;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket};

// Import the server functions
use crate::app::models::websocket_session::{CreateSessionRequest, SessionSummary};
use crate::app::server_functions::websocket_sessions::{
    create_session, get_session, join_session, leave_session, list_active_sessions,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    sender_id: String,
    content: String,
    timestamp: String,
}

#[component]
pub fn Chat() -> impl IntoView {
    let (messages, set_messages) = create_signal(Vec::<ChatMessage>::new());
    let (current_message, set_current_message) = create_signal(String::new());
    let (connected, set_connected) = create_signal(false);
    let (user_id, set_user_id) = create_signal(String::new());

    // Room joining/creation states
    let (room_id, set_room_id) = create_signal(String::new());
    let (join_room_id, set_join_room_id) = create_signal(String::new());
    let (is_in_room, set_is_in_room) = create_signal(false);
    let (room_name, set_room_name) = create_signal(String::new());
    let (room_description, set_room_description) = create_signal(String::new());
    let (max_users, set_max_users) = create_signal(0);
    let (is_private, set_is_private) = create_signal(false);
    let (room_password, set_room_password) = create_signal(String::new());
    let (join_password, set_join_password) = create_signal(String::new());
    let (error_message, set_error_message) = create_signal(String::new());

    // Active sessions
    let (active_sessions, set_active_sessions) = create_signal(Vec::<SessionSummary>::new());
    let (loading_sessions, set_loading_sessions) = create_signal(false);

    // UI state
    let (active_tab, set_active_tab) = create_signal("join"); // "join" or "create"

    // Create a shared reference to store the WebSocket
    let socket_ref = Rc::new(RefCell::new(None::<WebSocket>));
    let socket_ref_for_cleanup = Rc::clone(&socket_ref);
    let socket_ref_for_send = Rc::clone(&socket_ref);
    let socket_ref_for_leave = Rc::clone(&socket_ref);

    // Define connect_to_websocket action before using it
    let connect_to_websocket = create_action(move |room_id_str: &String| {
        let room_id_val = room_id_str.clone();
        let socket_ref_clone = Rc::clone(&socket_ref);
        let set_messages = set_messages.clone();
        let set_connected = set_connected.clone();
        let set_is_in_room = set_is_in_room.clone();
        let set_error_message = set_error_message.clone();
        let set_user_id = set_user_id.clone();
        let user_id = user_id.clone();

        async move {
            // Disconnect from any existing connection
            if let Some(socket) = socket_ref_clone.borrow_mut().take() {
                let _ = socket.close();
            }

            set_messages.update(|msgs| msgs.clear());
            set_connected.set(false);
            set_error_message.set(String::new());

            log::info!("Establishing WebSocket connection to room: {}", room_id_val);

            let ws_url = format!(
                "ws://{}/ws/{}",
                window().location().host().unwrap(),
                room_id_val
            );

            match WebSocket::new(&ws_url) {
                Ok(socket) => {
                    // Set up WebSocket event handlers
                    let set_connected_clone = set_connected.clone();
                    let set_is_in_room_clone = set_is_in_room.clone();
                    let on_open = Closure::wrap(Box::new(move |_| {
                        set_connected_clone.set(true);
                        set_is_in_room_clone.set(true);
                        log::info!("WebSocket connection established");
                    }) as Box<dyn FnMut(JsValue)>);
                    socket.set_onopen(Some(on_open.as_ref().unchecked_ref()));
                    on_open.forget();

                    let set_connected_clone = set_connected.clone();
                    let set_is_in_room_clone = set_is_in_room.clone();
                    let on_close = Closure::wrap(Box::new(move |_| {
                        set_connected_clone.set(false);
                        log::info!("WebSocket connection closed");
                    }) as Box<dyn FnMut(JsValue)>);
                    socket.set_onclose(Some(on_close.as_ref().unchecked_ref()));
                    on_close.forget();

                    let set_error_message_clone = set_error_message.clone();
                    let on_error = Closure::wrap(Box::new(move |e| {
                        set_error_message_clone.set(format!("WebSocket error: Connection failed"));
                        log::error!("WebSocket error: {:?}", e);
                    }) as Box<dyn FnMut(JsValue)>);
                    socket.set_onerror(Some(on_error.as_ref().unchecked_ref()));
                    on_error.forget();

                    let set_messages_clone = set_messages.clone();
                    let set_user_id_clone = set_user_id.clone();

                    let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
                        let data = e.data().as_string().unwrap();

                        // Check if this is the initial ID message
                        if data.starts_with("your id is ") {
                            let id = data.replace("your id is ", "");
                            set_user_id_clone.set(id);
                            return;
                        }

                        // Otherwise treat as a chat message
                        let parts: Vec<&str> = data.split(": ").collect();
                        let (sender_id, content) = if parts.len() > 1 {
                            (parts[0].to_string(), parts[1..].join(": "))
                        } else {
                            ("System".to_string(), data)
                        };

                        let now = js_sys::Date::new_0();
                        let timestamp = now.to_locale_time_string("en-US");

                        let new_msg = ChatMessage {
                            sender_id,
                            content,
                            timestamp: timestamp.into(),
                        };

                        set_messages_clone.update(|msgs| {
                            msgs.push(new_msg);
                        });
                    })
                        as Box<dyn FnMut(MessageEvent)>);

                    socket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
                    on_message.forget();

                    // Store the socket
                    *socket_ref_clone.borrow_mut() = Some(socket);

                    // Update room state
                    set_is_in_room.set(true);
                }
                Err(err) => {
                    set_error_message.set(format!("Failed to create WebSocket connection"));
                    log::error!("Failed to create WebSocket: {:?}", err);
                }
            }
        }
    });

    // Fetch active sessions
    let fetch_sessions = create_action(move |_: &()| {
        let set_active_sessions = set_active_sessions.clone();
        let set_loading_sessions = set_loading_sessions.clone();
        let set_error_message = set_error_message.clone();

        async move {
            set_loading_sessions.set(true);
            set_error_message.set(String::new());

            match list_active_sessions().await {
                Ok(sessions) => {
                    set_active_sessions.set(sessions);
                }
                Err(err) => {
                    set_error_message.set(format!("Failed to load sessions: {}", err));
                }
            }

            set_loading_sessions.set(false);
        }
    });

    // Create a new room using the server API
    let create_new_room = create_action(move |_: &()| {
        let room_name_val = room_name.get();
        let room_desc_val = room_description.get();
        let max_users_val = max_users.get();
        let is_private_val = is_private.get();
        let password_val = room_password.get();

        let set_error_message = set_error_message.clone();
        let set_room_id = set_room_id.clone();
        let connect_ws = connect_to_websocket.clone();

        async move {
            // Validate input
            if room_name_val.trim().is_empty() {
                set_error_message.set("Room name cannot be empty".into());
                return;
            }

            let request = CreateSessionRequest {
                name: room_name_val.clone(),
                description: if room_desc_val.trim().is_empty() {
                    None
                } else {
                    Some(room_desc_val)
                },
                max_users: if max_users_val <= 0 {
                    None
                } else {
                    Some(max_users_val)
                },
                is_private: Some(is_private_val),
                password: if password_val.trim().is_empty() {
                    None
                } else {
                    Some(password_val)
                },
                metadata: None,
            };

            match create_session(request).await {
                Ok(session) => {
                    log::info!("Created new room: {}", session.id);
                    // Connect to the websocket
                    set_room_id.set(session.id.to_string());
                    connect_ws.dispatch(session.id.to_string());
                }
                Err(err) => {
                    set_error_message.set(format!("Failed to create room: {}", err));
                }
            }
        }
    });

    // Join existing room by ID
    let join_room_by_id = create_action(move |_: &()| {
        let room_id_val = join_room_id.get();
        let password_val = join_password.get();
        let set_error_message = set_error_message.clone();
        let set_room_id = set_room_id.clone();
        let connect_ws = connect_to_websocket.clone();

        async move {
            // Validate UUID
            if room_id_val.trim().is_empty() {
                set_error_message.set("Room ID is required".into());
                return;
            }

            if let Err(_) = Uuid::parse_str(&room_id_val) {
                set_error_message.set("Invalid room ID format".into());
                return;
            }

            // Try to join the session
            match join_session(
                room_id_val.clone(),
                if password_val.is_empty() {
                    None
                } else {
                    Some(password_val)
                },
            )
            .await
            {
                Ok(_) => {
                    // Connect to websocket
                    set_room_id.set(room_id_val.clone());
                    connect_ws.dispatch(room_id_val);
                }
                Err(err) => {
                    set_error_message.set(format!("Failed to join room: {}", err));
                }
            }
        }
    });

    // Join from active sessions list
    let join_from_list = create_action(move |session_id: &String| {
        let room_id_val = session_id.clone();
        let password_val = join_password.get();
        let set_error_message = set_error_message.clone();
        let set_room_id = set_room_id.clone();
        let connect_ws = connect_to_websocket.clone();

        async move {
            // Try to join the session
            match join_session(
                room_id_val.clone(),
                if password_val.is_empty() {
                    None
                } else {
                    Some(password_val)
                },
            )
            .await
            {
                Ok(_) => {
                    // Connect to websocket
                    set_room_id.set(room_id_val.clone());
                    connect_ws.dispatch(room_id_val);
                }
                Err(err) => {
                    set_error_message.set(format!("Failed to join room: {}", err));
                }
            }
        }
    });

    // Send message function
    let send_message = move |e: ev::SubmitEvent| {
        e.prevent_default();
        let msg = current_message.get();
        if !msg.trim().is_empty() && connected.get() {
            if let Some(socket) = socket_ref_for_send.borrow().as_ref() {
                if let Err(err) = socket.send_with_str(&msg) {
                    log::error!("Failed to send message: {:?}", err);
                } else {
                    set_current_message.set("".to_string());
                }
            }
        }
    };

    // Function to leave current room
    let leave_room = create_action(move |_: &()| {
        let room_id_val = room_id.get();
        let socket_ref_for_leave = Rc::clone(&socket_ref_for_leave);
        let set_is_in_room = set_is_in_room.clone();
        let set_connected = set_connected.clone();
        let set_messages = set_messages.clone();

        async move {
            // Close WebSocket connection
            if let Some(socket) = socket_ref_for_leave.borrow_mut().take() {
                let _ = socket.close();
            }

            // Notify server we're leaving
            if !room_id_val.is_empty() {
                let _ = leave_session(room_id_val).await;
            }

            // Update UI state
            set_is_in_room.set(false);
            set_connected.set(false);
            set_messages.update(|msgs| msgs.clear());
        }
    });

    // Refresh active sessions periodically
    let refresh_timer = use_interval(
        move || {
            if !is_in_room.get() {
                fetch_sessions.dispatch(());
            }
        },
        10000, // Refresh every 10 seconds
    );

    // Load active sessions on component mount
    create_effect(move |_| {
        fetch_sessions.dispatch(());
    });

    // Clean up when the component is unmounted
    on_cleanup(move || {
        log::info!("Cleaning up WebSocket connection");
        if let Some(socket) = socket_ref_for_cleanup.borrow_mut().take() {
            let _ = socket.close();
        }
    });

    view! {
        <div class="flex flex-col h-full border rounded-lg shadow">
            {move || {
                if !is_in_room.get() {
                    view! {
                        <div class="bg-white rounded-lg overflow-hidden">
                            <div class="p-6 border-b">
                                <h2 class="text-xl font-semibold text-gray-800">Chat Rooms</h2>
                            </div>

                            // Error message display
                            {move || {
                                let error = error_message.get();
                                if !error.is_empty() {
                                    view! {
                                        <div class="mx-6 mt-4 bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded relative">
                                            <span class="block sm:inline">{error}</span>
                                            <button
                                                class="absolute top-0 right-0 px-4 py-3"
                                                on:click=move |_| set_error_message.set(String::new())
                                            >
                                                <span class="text-red-700">&times;</span>
                                            </button>
                                        </div>
                                    }
                                } else {
                                    view! { <div></div> }
                                }
                            }}

                            // Tab navigation
                            <div class="flex mx-6 mt-6 border-b">
                                <button
                                    class=move || format!("py-2 px-4 font-medium text-sm {} {}",
                                        if active_tab.get() == "join" { "text-blue-600 border-b-2 border-blue-600" }
                                        else { "text-gray-500 hover:text-gray-700" },
                                        if active_tab.get() == "join" { "font-medium" } else { "" }
                                    )
                                    on:click=move |_| set_active_tab.set("join")
                                >
                                    Join a Room
                                </button>
                                <button
                                    class=move || format!("py-2 px-4 font-medium text-sm {} {}",
                                        if active_tab.get() == "create" { "text-blue-600 border-b-2 border-blue-600" }
                                        else { "text-gray-500 hover:text-gray-700" },
                                        if active_tab.get() == "create" { "font-medium" } else { "" }
                                    )
                                    on:click=move |_| set_active_tab.set("create")
                                >
                                    Create a Room
                                </button>
                            </div>

                            <div class="p-6">
                                {move || {
                                    if active_tab.get() == "join" {
                                        view! {
                                            <div>
                                                // Active sessions section
                                                <div class="mb-8">
                                                    <div class="flex justify-between items-center mb-4">
                                                        <h3 class="text-lg font-medium text-gray-900">Available Rooms</h3>
                                                        <button
                                                            on:click=move |_| fetch_sessions.dispatch(())
                                                            class="text-blue-600 hover:text-blue-800 text-sm flex items-center"
                                                        >
                                                            Refresh
                                                        </button>
                                                    </div>

                                                    {move || {
                                                        if loading_sessions.get() {
                                                            view! {
                                                                <div class="text-center py-8">
                                                                    <p class="text-gray-500">Loading available rooms...</p>
                                                                </div>
                                                            }.into_view()
                                                        } else {
                                                            let sessions = active_sessions.get();
                                                            if sessions.is_empty() {
                                                                view! {
                                                                    <div class="text-center py-8 bg-gray-50 border border-gray-100 rounded">
                                                                        <p class="text-gray-500">No active rooms found</p>
                                                                        <p class="text-sm text-gray-400 mt-1">Create a new room to get started</p>
                                                                    </div>
                                                                }.into_view()
                                                            } else {
                                                                view! {
                                                                    <div class="grid gap-3">
                                                                        {sessions.into_iter().map(|session| {
                                                                            let id = session.id.to_string();
                                                                            let name = session.name.clone();
                                                                            let users = session.current_users;
                                                                            let has_password = session.password_required;
                                                                            let join_from_list_clone = join_from_list.clone();
                                                                            let set_join_password_clone = set_join_password.clone();

                                                                            view! {
                                                                                <div class="p-4 border rounded-md bg-white hover:bg-gray-50 transition-colors">
                                                                                    <div class="flex justify-between items-center">
                                                                                        <div>
                                                                                            <h4 class="font-medium text-gray-900">{name}</h4>
                                                                                            <div class="flex items-center mt-1 text-sm text-gray-600">
                                                                                                <span class="mr-3">{format!("{} users", users)}</span>
                                                                                                {if has_password {
                                                                                                    view! {
                                                                                                        <span class="flex items-center text-amber-600">
                                                                                                            <span class="mr-1">"🔒"</span> Password required
                                                                                                        </span>
                                                                                                    }.into_view()
                                                                                                } else {
                                                                                                    view! {}.into_view()
                                                                                                }}
                                                                                            </div>
                                                                                            {if let Some(desc) = session.description {
                                                                                                view! { <p class="text-sm mt-2 text-gray-600">{desc}</p> }.into_view()
                                                                                            } else {
                                                                                                view! {}.into_view()
                                                                                            }}
                                                                                            <div class="text-xs text-gray-500 mt-2">
                                                                                                {"ID: "}{id.clone()}
                                                                                            </div>
                                                                                        </div>
                                                                                        {if has_password {
                                                                                            let id_clone = id.clone();
                                                                                            view! {
                                                                                                <div class="flex flex-col items-end space-y-2">
                                                                                                    <input
                                                                                                        type="password"
                                                                                                        prop:value=move || join_password.get()
                                                                                                        on:input=move |ev| {
                                                                                                            set_join_password_clone.set(event_target_value(&ev));
                                                                                                        }
                                                                                                        class="p-2 text-sm border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                                                                                                        placeholder="Enter password"
                                                                                                    />
                                                                                                    <button
                                                                                                        on:click=move |_| join_from_list_clone.dispatch(id_clone.clone())
                                                                                                        class="bg-blue-600 text-white px-4 py-2 text-sm rounded hover:bg-blue-700 transition-colors"
                                                                                                    >
                                                                                                        Join Room
                                                                                                    </button>
                                                                                                </div>
                                                                                            }.into_view()
                                                                                        } else {
                                                                                            let id_clone = id.clone();
                                                                                            view! {
                                                                                                <button
                                                                                                    on:click=move |_| join_from_list_clone.dispatch(id_clone.clone())
                                                                                                    class="bg-blue-600 text-white px-4 py-2 text-sm rounded hover:bg-blue-700 transition-colors"
                                                                                                >
                                                                                                    Join Room
                                                                                                </button>
                                                                                            }.into_view()
                                                                                        }}
                                                                                    </div>
                                                                                </div>
                                                                            }
                                                                        }).collect_view()}
                                                                    </div>
                                                                }.into_view()
                                                            }
                                                        }
                                                    }}
                                                </div>

                                                // Direct join by ID section
                                                <div class="mt-8 pt-6 border-t">
                                                    <h3 class="text-base font-medium text-gray-900 mb-4">Join by Room ID</h3>
                                                    <div class="space-y-4">
                                                        <div>
                                                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                                                Room ID
                                                            </label>
                                                            <input
                                                                type="text"
                                                                prop:value=move || join_room_id.get()
                                                                on:input=move |ev| {
                                                                    set_join_room_id.set(event_target_value(&ev));
                                                                }
                                                                class="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                                                                placeholder="Enter room ID"
                                                            />
                                                        </div>
                                                        <div>
                                                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                                                Password (if required)
                                                            </label>
                                                            <input
                                                                type="password"
                                                                prop:value=move || join_password.get()
                                                                on:input=move |ev| {
                                                                    set_join_password.set(event_target_value(&ev));
                                                                }
                                                                class="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                                                                placeholder="Password (if required)"
                                                            />
                                                        </div>
                                                        <button
                                                            on:click=move |_| join_room_by_id.dispatch(())
                                                            class="w-full bg-blue-600 text-white py-2 rounded hover:bg-blue-700 transition-colors"
                                                        >
                                                            Join Room
                                                        </button>
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    } else {
                                        view! {
                                            <div class="space-y-4">
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                                        Room Name <span class="text-red-500">*</span>
                                                    </label>
                                                    <input
                                                        type="text"
                                                        prop:value=move || room_name.get()
                                                        on:input=move |ev| {
                                                            set_room_name.set(event_target_value(&ev));
                                                        }
                                                        class="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                                                        placeholder="Enter a name for your room"
                                                    />
                                                </div>
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                                        Description
                                                    </label>
                                                    <textarea
                                                        prop:value=move || room_description.get()
                                                        on:input=move |ev| {
                                                            set_room_description.set(event_target_value(&ev));
                                                        }
                                                        class="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                                                        placeholder="Describe your room (optional)"
                                                        rows="3"
                                                    ></textarea>
                                                </div>
                                                <div class="flex gap-4">
                                                    <div class="w-1/2">
                                                        <label class="block text-sm font-medium text-gray-700 mb-1">
                                                            Max Users
                                                        </label>
                                                        <input
                                                            type="number"
                                                            prop:value=move || max_users.get()
                                                            on:input=move |ev| {
                                                                set_max_users.set(event_target_value(&ev).parse::<i32>().unwrap_or(0));
                                                            }
                                                            class="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                                                            placeholder="0 = unlimited"
                                                        />
                                                    </div>
                                                    <div class="w-1/2">
                                                        <label class="block text-sm font-medium text-gray-700 mb-1">
                                                            Room Visibility
                                                        </label>
                                                        <div class="flex items-center mt-2">
                                                            <input
                                                                type="checkbox"
                                                                id="is-private"
                                                                prop:checked=move || is_private.get()
                                                                on:change=move |ev| {
                                                                    set_is_private.set(event_target_checked(&ev));
                                                                }
                                                                class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                                                            />
                                                            <label for="is-private" class="ml-2 block text-sm text-gray-700">
                                                                Private Room
                                                            </label>
                                                        </div>
                                                    </div>
                                                </div>
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                                        Password
                                                    </label>
                                                    <input
                                                        type="password"
                                                        prop:value=move || room_password.get()
                                                        on:input=move |ev| {
                                                            set_room_password.set(event_target_value(&ev));
                                                        }
                                                        class="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                                                        placeholder="Set password (optional)"
                                                    />
                                                </div>
                                                <button
                                                    on:click=move |_| create_new_room.dispatch(())
                                                    class="w-full mt-4 bg-blue-600 text-white py-2 rounded hover:bg-blue-700 transition-colors"
                                                >
                                                    Create Room
                                                </button>
                                            </div>
                                        }
                                    }
                                }}
                            </div>
                        </div>
                    }
                } else {
                    // Chat room interface when user is in a room
                    view! {
                        <div class="flex flex-col h-full">
                            // Chat room header
                            <div class="bg-white p-4 border-b flex justify-between items-center">
                                <div>
                                    <h2 class="text-lg font-semibold text-gray-800">
                                        {move || {
                                            if !room_name.get().is_empty() {
                                                room_name.get()
                                            } else {
                                                format!("Chat Room: {}", room_id.get())
                                            }
                                        }}
                                    </h2>
                                    <div class="text-sm text-gray-500 flex items-center">
                                        <span class="mr-2">
                                            {move || {
                                                if connected.get() {
                                                    "Connected"
                                                } else {
                                                    "Disconnected"
                                                }
                                            }}
                                        </span>
                                        <div class={move || {
                                            format!("w-2 h-2 rounded-full mr-2 {}",
                                                if connected.get() { "bg-green-500" } else { "bg-red-500" }
                                            )
                                            }}>
                                        </div>
                                        <span>{"Your ID: "}{move || user_id.get()}</span>
                                    </div>
                                </div>
                                <button
                                    on:click=move |_| leave_room.dispatch(())
                                    class="bg-gray-100 text-gray-700 py-2 px-4 rounded hover:bg-gray-200 transition-colors"
                                >
                                    Leave Room
                                </button>
                            </div>

                            // Chat messages area
                            <div class="flex-grow overflow-y-auto p-4 bg-gray-50" id="messages">
                                <div class="space-y-3">
                                    {move || {
                                        messages.get().into_iter().map(|msg| {
                                            let is_self = msg.sender_id == user_id.get();
                                            let sender_name = if is_self { "You".to_string() } else { msg.sender_id.clone() };

                                            view! {
                                                <div class={move || {
                                                    format!("flex {}",
                                                        if is_self { "justify-end" } else { "justify-start" }
                                                    )
                                                }}>
                                                    <div class={move || {
                                                        format!("max-w-3/4 rounded-lg px-4 py-2 {}",
                                                            if is_self { "bg-blue-600 text-white" } else { "bg-white border" }
                                                        )
                                                    }}>
                                                        <div class="flex justify-between items-center mb-1">
                                                            <span class={move || {
                                                                format!("font-medium text-sm {}",
                                                                    if is_self { "text-blue-100" } else { "text-gray-700" }
                                                                )
                                                            }}>
                                                                {sender_name}
                                                            </span>
                                                            <span class={move || {
                                                                format!("text-xs ml-2 {}",
                                                                    if is_self { "text-blue-100" } else { "text-gray-500" }
                                                                )
                                                            }}>
                                                                {msg.timestamp.clone()}
                                                            </span>
                                                        </div>
                                                        <p class={move || {
                                                            if is_self { "text-white" } else { "text-gray-800" }
                                                        }}>
                                                            {msg.content.clone()}
                                                        </p>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()
                                    }}
                                </div>
                            </div>

                            // Message input form
                            <div class="p-4 bg-white border-t">
                                <form on:submit=send_message.clone() class="flex gap-2">
                                    <input
                                        type="text"
                                        prop:value=move || current_message.get()
                                        on:input=move |ev| {
                                            set_current_message.set(event_target_value(&ev));
                                        }
                                        class="flex-grow p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                                        placeholder="Type a message..."
                                        disabled=move || !connected.get()
                                    />
                                    <button
                                        type="submit"
                                        class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 transition-colors disabled:bg-blue-300"
                                        disabled=move || !connected.get() || current_message.get().trim().is_empty()
                                    >
                                        Send
                                    </button>
                                </form>
                            </div>
                        </div>
                    }
                }
            }}
        </div>
    }
}

fn use_interval<F>(f: F, ms: u32) -> impl Drop
where
    F: FnMut() + 'static,
{
    let window = web_sys::window().expect("no global `window` exists");
    let callback = Closure::wrap(Box::new(f) as Box<dyn FnMut()>);
    let interval_id = window
        .set_interval_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            ms as i32,
        )
        .expect("failed to set interval");

    // Keep the closure alive for the lifetime of the interval
    callback.forget();

    // Return a guard that will clear the interval when dropped
    IntervalGuard {
        window,
        interval_id,
    }
}

struct IntervalGuard {
    window: web_sys::Window,
    interval_id: i32,
}

impl Drop for IntervalGuard {
    fn drop(&mut self) {
        self.window.clear_interval_with_handle(self.interval_id);
    }
}
