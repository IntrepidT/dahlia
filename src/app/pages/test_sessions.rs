use crate::app::components::dashboard::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::header::Header;
use crate::app::models::user::User;
use crate::app::models::websocket_session::{SessionSummary, SessionType};
use crate::app::server_functions::websocket_sessions;
use chrono::{DateTime, Duration, Utc};
use leptos::*;
use leptos_router::*;
use log::{error, info};
use std::collections::HashMap;

#[component]
pub fn TestSessionsList() -> impl IntoView {
    let user = use_context::<ReadSignal<Option<User>>>().expect("AuthProvider not Found");
    let navigate = use_navigate();

    // State for sessions
    let (sessions, set_sessions) = create_signal(Vec::<SessionSummary>::new());
    let (is_loading, set_is_loading) = create_signal(true);
    let (error_message, set_error_message) = create_signal(String::new());
    let (filter_value, set_filter_value) = create_signal(String::new());

    let (selected_view, set_selected_view) = create_signal(SidebarSelected::Live);

    // Action to fetch sessions
    let fetch_sessions = create_action(move |_: &()| async move {
        set_is_loading.set(true);
        set_error_message.set(String::new());

        match websocket_sessions::list_active_sessions().await {
            Ok(fetched_sessions) => {
                set_sessions.set(fetched_sessions);
                set_is_loading.set(false);
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to fetch sessions: {}", e);
                error!("{}", error_msg);
                set_error_message.set(error_msg);
                set_is_loading.set(false);
                Err(())
            }
        }
    });

    // Refresh sessions every 30 seconds
    create_effect(move |_| {
        fetch_sessions.dispatch(());

        let handle = set_interval_with_handle(
            move || {
                fetch_sessions.dispatch(());
            },
            std::time::Duration::from_secs(30),
        );

        on_cleanup(move || {
            if let Ok(handle) = handle {
                handle.clear();
            }
        });
    });

    // Filter sessions based on search term
    let filtered_sessions = create_memo(move |_| {
        let all_sessions = sessions.get();
        let filter = filter_value.get().to_lowercase();

        if filter.is_empty() {
            all_sessions
        } else {
            all_sessions
                .into_iter()
                .filter(|session| {
                    session.name.to_lowercase().contains(&filter)
                        || session
                            .description
                            .as_ref()
                            .unwrap_or(&String::new())
                            .to_lowercase()
                            .contains(&filter)
                        || session
                            .test_id
                            .as_ref()
                            .unwrap_or(&String::new())
                            .to_lowercase()
                            .contains(&filter)
                })
                .collect()
        }
    });

    // Join a session
    let join_session_action = create_action(move |session_id: &uuid::Uuid| {
        let session_id = *session_id;
        let sessions_value = sessions.clone();
        let navigate = use_navigate();

        async move {
            // Find the session to get its test_id
            let sessions_data = sessions_value.get();
            if let Some(session) = sessions_data.iter().find(|s| s.id == session_id) {
                if let Some(test_id) = &session.test_id {
                    // Navigate directly to the test-session route with the test_id
                    navigate(
                        &format!("/tests/{}/sessions/{}", test_id, session_id),
                        NavigateOptions::default(),
                    );
                }
            }
        }
    });

    // Format time since creation
    let format_time_ago = |created_at: DateTime<Utc>| {
        let now = Utc::now();
        let diff = now.signed_duration_since(created_at);

        if diff < Duration::minutes(1) {
            "just now".to_string()
        } else if diff < Duration::hours(1) {
            format!("{} minutes ago", diff.num_minutes())
        } else if diff < Duration::days(1) {
            format!("{} hours ago", diff.num_hours())
        } else {
            format!("{} days ago", diff.num_days())
        }
    };

    view! {
        <div class="min-h-screen flex flex-col bg-[#F9F9F8]">
            <Header />
            <DashboardSidebar
                selected_item=selected_view
                set_selected_item=set_selected_view
            />

            <div class="p-4 w-full h-screen overflow-y-auto bg-gray-50 mx-auto">
                {/* Header */}
                <div class="text-center mb-8">
                    <h2 class="text-2xl font-bold text-gray-800">
                        "Available Test Sessions"
                    </h2>
                    <p class="text-gray-600 mt-2">
                        "Join an active test session or wait for a teacher to start one"
                    </p>
                </div>

                {/* Search and filter */}
                <div class="max-w-4xl mx-auto mb-6">
                    <div class="bg-white shadow-sm rounded-lg p-4 flex">
                        <input
                            type="text"
                            placeholder="Search for test sessions..."
                            class="w-full px-4 py-2 rounded-md border border-gray-200 focus:ring-blue-500 focus:border-blue-500"
                            prop:value=move || filter_value.get()
                            on:input=move |ev| {
                                set_filter_value.set(event_target_value(&ev));
                            }
                        />
                        <button
                            class="ml-2 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
                            on:click=move |_| {
                                fetch_sessions.dispatch(());
                            }
                        >
                            "Refresh"
                        </button>
                    </div>
                </div>

                {/* Error message display */}
                <Show when=move || !error_message.get().is_empty()>
                    <div class="max-w-4xl mx-auto mb-6">
                        <div class="bg-red-100 border-l-4 border-red-500 text-red-700 p-4 rounded">
                            <p>{move || error_message.get()}</p>
                        </div>
                    </div>
                </Show>

                {/* Sessions list */}
                <div class="max-w-4xl mx-auto">
                    <Show when=move || !is_loading.get() fallback=|| view! {
                        <div class="flex justify-center items-center py-12">
                            <div class="animate-pulse flex flex-col items-center">
                                <div class="h-12 w-12 border-4 border-gray-300 border-t-blue-500 rounded-full animate-spin mb-4"></div>
                                <p class="text-gray-500">"Loading test sessions..."</p>
                            </div>
                        </div>
                    }>
                        <Show
                            when=move || !filtered_sessions.get().is_empty()
                            fallback=|| view! {
                                <div class="bg-white shadow-sm rounded-lg p-8 text-center">
                                    <p class="text-gray-500 text-lg mb-2">"No active test sessions found"</p>
                                    <p class="text-gray-400">"Check back later or ask your teacher to start a session"</p>
                                </div>
                            }
                        >
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <For
                                    each=move || filtered_sessions.get()
                                    key=|session| session.id
                                    children=move |session| {
                                        let session_id = session.id;
                                        let session_test_id_clone = session.test_id.clone();
                                        let created_time_ago = format_time_ago(session.created_at);
                                        let is_test_session = matches!(session.session_type, SessionType::Test);
                                        let session_type_label = match session.session_type {
                                            SessionType::Test => "Test",
                                            SessionType::Chat => "Chat"
                                        };

                                        view! {
                                            <div class="bg-white shadow-sm rounded-lg overflow-hidden border border-gray-200 hover:shadow-md transition-shadow">
                                                <div class="p-4">
                                                    <div class="flex justify-between items-start">
                                                        <h3 class="font-medium text-lg text-gray-800">{session.name}</h3>
                                                        <span class=move || format!(
                                                            "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {} {}",
                                                            if is_test_session { "bg-purple-100 text-purple-800" } else { "bg-blue-100 text-blue-800" },
                                                            "animate-pulse"
                                                        )>
                                                            "Active"
                                                        </span>
                                                    </div>
                                                    <div class="mt-1 text-sm text-gray-500">
                                                        <p>
                                                            "Created " {created_time_ago}
                                                        </p>
                                                        <Show when=move || session_test_id_clone.is_some()>
                                                            <p class="mt-1">
                                                                "Test ID: " {session.test_id.clone().unwrap_or_default()}
                                                            </p>
                                                        </Show>
                                                    </div>
                                                    <p class="mt-2 text-sm text-gray-600">
                                                        {session.description.unwrap_or_else(|| "No description provided".to_string())}
                                                    </p>
                                                    <div class="mt-4 flex items-center justify-between">
                                                        <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
                                                            {session_type_label}
                                                        </span>
                                                        <div class="flex space-x-2">
                                                            <button
                                                                class="px-4 py-2 bg-gradient-to-r from-blue-600 to-purple-600 text-white rounded-md hover:from-blue-700 hover:to-purple-700 transition-colors text-sm font-medium"
                                                                on:click=move |_| join_session_action.dispatch(session_id)
                                                            >
                                                                "Join Session"
                                                            </button>
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        </Show>
                    </Show>
                </div>

                /*{/* Create new test session button (only for teachers) */}
                <Show when=move || {
                    user.get().map(|u| u.is_admin() || u.is_teacher()).unwrap_or(false)
                }>
                    <div class="max-w-4xl mx-auto mt-8 flex justify-center">
                        <A
                            href="/tests"
                            class="px-5 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
                        >
                            "Create New Test Session"
                        </A>
                    </div>
                </Show>*/
            </div>
        </div>
    }
}
