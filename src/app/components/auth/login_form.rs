use crate::app::components::authorization_components::perform_post_login_redirect;
use crate::app::models::user::SessionUser;
use crate::app::server_functions::auth::{get_current_user, login, logout, register};
use leptos::prelude::*;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use log::{debug, error, log};
use serde::Serialize;
#[cfg(feature = "ssr")]
use {
    lettre::transport::smtp::authentication::Credentials,
    lettre::{message::Message, SmtpTransport, Transport},
};

#[component]
pub fn LoginForm() -> impl IntoView {
    let (username, set_username) = signal("".to_string());
    let (password, set_password) = signal("".to_string());
    let (error, set_error) = signal::<Option<String>>(None);
    let set_current_user = expect_context::<WriteSignal<Option<SessionUser>>>();

    let handle_submit = Action::new(move |_: &()| {
        let username = username.get();
        let password = password.get();

        async move {
            // Client-side validation for empty fields
            if username.trim().is_empty() || password.trim().is_empty() {
                set_error.set(Some("Username and password are required".to_string()));
                return;
            }

            // Debug log
            log::info!("Attempting login with username: {}", username);

            match login(username, password).await {
                Ok(response) => {
                    // Debug log the response
                    log::info!(
                        "Login response: success={}, message={}, user={:?}",
                        response.success,
                        response.message,
                        response.user
                    );

                    if response.success {
                        log::info!("Login successful, setting user");
                        set_current_user.set(response.user);
                        set_error.set(None);

                        // Use the simple redirect function
                        perform_post_login_redirect();
                    } else {
                        log::info!("Login failed: {}", response.message);
                        set_error.set(Some(response.message));
                    }
                }
                Err(err) => {
                    log::info!("Login error: {:?}", err);
                    set_error.set(Some(
                        "Login failed. Please check your credentials and try again.".to_string(),
                    ));
                }
            }
        }
    });

    view! {
        <div class="p-4 bg-white rounded shadow-md">
            <h2 class="text-2xl font-bold mb-4">"Login"</h2>

            {move || {
                error.get().map(|err| {
                    view! {
                        <div class="mb-4 p-2 bg-red-100 text-red-700 rounded">{err}</div>
                    }
                })
            }}

            <form on:submit=move |ev| {
                ev.prevent_default();
                handle_submit.dispatch(());
            }>
                <div class="mb-4">
                    <label class="block text-gray-700 mb-2" for="username">"Username"</label>
                    <input
                        id="username"
                        type="text"
                        class="w-full p-2 border rounded"
                        prop:value=move || username.get()
                        on:input=move |ev| {
                            set_username.set(event_target_value(&ev));
                        }
                    />
                </div>

                <div class="mb-4">
                    <label class="block text-gray-700 mb-2" for="password">"Password"</label>
                    <input
                        id="password"
                        type="password"
                        class="w-full p-2 border rounded"
                        prop:value=move || password.get()
                        on:input=move |ev| {
                            set_password.set(event_target_value(&ev));
                        }
                    />
                </div>

                <button
                    type="submit"
                    class="w-full p-2 bg-blue-500 text-white rounded hover:bg-blue-600"
                    prop:disabled=move || handle_submit.pending().get()
                >
                    {move || {
                        if handle_submit.pending().get() {
                            "Logging in..."
                        } else {
                            "Login"
                        }
                    }}
                </button>
            </form>
        </div>
    }
}

#[component]
pub fn RegisterForm() -> impl IntoView {
    let (username, set_username) = signal("".to_string());
    let (email, set_email) = signal("".to_string());
    let (password, set_password) = signal("".to_string());
    let (confirm_password, set_confirm_password) = signal("".to_string());
    let (error, set_error) = create_signal::<Option<String>>(None);
    let set_current_user = expect_context::<WriteSignal<Option<SessionUser>>>();

    let handle_submit = Action::new(move |_: &()| {
        let username = username.get();
        let email = email.get();
        let password = password.get();
        let confirm_password = confirm_password.get();

        async move {
            // Check passwords match first
            if password != confirm_password {
                set_error.set(Some("Passwords do not match".to_string()));
                return;
            }

            // Then proceed with registration
            match register(username, email, password).await {
                Ok(response) => {
                    if response.success {
                        set_current_user.set(response.user);
                        set_error.set(None);

                        // Use the simple redirect function
                        perform_post_login_redirect();
                    } else {
                        set_error.set(Some(response.message));
                    }
                }
                Err(_) => {
                    set_error.set(Some("An error occurred".to_string()));
                }
            }
        }
    });

    view! {
        <div class="p-4 bg-white rounded shadow-md">
            <h2 class="text-2xl font-bold mb-4">"Register"</h2>

            {move || {
                error.get().map(|err| {
                    view! {
                        <div class="mb-4 p-2 bg-red-100 text-red-700 rounded">{err}</div>
                    }
                })
            }}

            <form on:submit=move |ev| {
                ev.prevent_default();
                handle_submit.dispatch(());
            }>
                <div class="mb-4">
                    <label class="block text-gray-700 mb-2" for="username">"Username"</label>
                    <input
                        id="username"
                        type="text"
                        class="w-full p-2 border rounded"
                        prop:value=move || username.get()
                        on:input=move |ev| {
                            set_username.set(event_target_value(&ev));
                        }
                    />
                </div>

                <div class="mb-4">
                    <label class="block text-gray-700 mb-2" for="email">"Email"</label>
                    <input
                        id="email"
                        type="email"
                        class="w-full p-2 border rounded"
                        prop:value=move || email.get()
                        on:input=move |ev| {
                            set_email.set(event_target_value(&ev));
                        }
                    />
                </div>

                <div class="mb-4">
                    <label class="block text-gray-700 mb-2" for="password">"Password"</label>
                    <input
                        id="password"
                        type="password"
                        class="w-full p-2 border rounded"
                        prop:value=move || password.get()
                        on:input=move |ev| {
                            set_password.set(event_target_value(&ev));
                        }
                    />
                </div>

                <div class="mb-4">
                    <label class="block text-gray-700 mb-2" for="confirm-password">"Confirm Password"</label>
                    <input
                        id="confirm-password"
                        type="password"
                        class="w-full p-2 border rounded"
                        prop:value=move || confirm_password.get()
                        on:input=move |ev| {
                            set_confirm_password.set(event_target_value(&ev));
                        }
                    />
                </div>

                <button
                    type="submit"
                    class="w-full p-2 bg-green-500 text-white rounded hover:bg-green-600"
                    prop:disabled=move || handle_submit.pending().get()
                >
                    {move || {
                        if handle_submit.pending().get() {
                            "Registering..."
                        } else {
                            "Register"
                        }
                    }}
                </button>
            </form>
        </div>
    }
}

#[component]
pub fn LogoutButton() -> impl IntoView {
    let set_current_user = expect_context::<WriteSignal<Option<SessionUser>>>();

    let handle_logout = Action::new(move |_: &()| {
        async move {
            match logout().await {
                Ok(_) => {
                    set_current_user.set(None);
                }
                Err(_) => {
                    // Handle error
                }
            }
        }
    });

    view! {
        <button
            class="p-2 bg-red-500 text-white rounded hover:bg-red-600"
            on:click=move |_| {
                handle_logout.dispatch(());
            }
            prop:disabled=move || handle_logout.pending().get()
        >
            {move || {
                if handle_logout.pending().get() {
                    "Logging out..."
                } else {
                    "Logout"
                }
            }}
        </button>
    }
}
