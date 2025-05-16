use crate::app::models::user::User;
use crate::app::server_functions::auth::{get_current_user, login, logout, register};
use leptos::*;
use leptos_router::use_navigate;
use log::{debug, error, log};
use serde::Serialize;
#[cfg(feature = "ssr")]
use {
    lettre::transport::smtp::authentication::Credentials,
    lettre::{message::Message, SmtpTransport, Transport},
};

#[component]
pub fn AuthProvider(children: Children) -> impl IntoView {
    let (current_user, set_current_user) = create_signal::<Option<User>>(None);
    let (loading, set_loading) = create_signal(true);

    // Load the current user on component mount
    create_effect(move |_| {
        set_loading.set(true);
        logging::log!("AuthProvider: Loading current user");

        spawn_local(async move {
            match get_current_user().await {
                Ok(user) => {
                    logging::log!("AuthProvider: User loaded: {:?}", user);
                    set_current_user.set(user);
                }
                Err(err) => {
                    logging::log!("AuthProvider: Error loading user: {:?}", err);
                    set_current_user.set(None);
                }
            }
            set_loading.set(false);
        });
    });

    // Add an effect to log whenever the current_user changes
    create_effect(move |_| {
        let user = current_user.get();
        logging::log!("AuthProvider: Current user updated: {:?}", user);
    });

    provide_context(current_user);
    provide_context(set_current_user);
    provide_context(loading);

    children()
}

#[component]
pub fn LoginForm() -> impl IntoView {
    let (username, set_username) = create_signal("".to_string());
    let (password, set_password) = create_signal("".to_string());
    let (error, set_error) = create_signal::<Option<String>>(None);
    let set_current_user = use_context::<WriteSignal<Option<User>>>().unwrap();

    let handle_submit = create_action(move |_: &()| {
        let username = username.get();
        let password = password.get();

        async move {
            // Client-side validation for empty fields
            if username.trim().is_empty() || password.trim().is_empty() {
                set_error.set(Some("Username and password are required".to_string()));
                return;
            }

            // Debug log
            logging::log!("Attempting login with username: {}", username);

            match login(username, password).await {
                Ok(response) => {
                    // Debug log the response
                    logging::log!(
                        "Login response: success={}, message={}, user={:?}",
                        response.success,
                        response.message,
                        response.user
                    );

                    if response.success {
                        logging::log!("Login successful, setting user");
                        set_current_user.set(response.user);
                        set_error.set(None);
                    } else {
                        logging::log!("Login failed: {}", response.message);
                        set_error.set(Some(response.message));
                    }
                }
                Err(err) => {
                    logging::log!("Login error: {:?}", err);
                    set_error.set(Some(format!("Error: {:?}", err)));
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
    let (username, set_username) = create_signal("".to_string());
    let (email, set_email) = create_signal("".to_string());
    let (password, set_password) = create_signal("".to_string());
    let (confirm_password, set_confirm_password) = create_signal("".to_string());
    let (error, set_error) = create_signal::<Option<String>>(None);
    let set_current_user = use_context::<WriteSignal<Option<User>>>().unwrap();

    let handle_submit = create_action(move |_: &()| {
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
    let set_current_user = use_context::<WriteSignal<Option<User>>>().unwrap();

    let handle_logout = create_action(move |_: &()| {
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

#[component]
pub fn RequireAuth(children: Children) -> impl IntoView {
    let current_user = use_context::<ReadSignal<Option<User>>>().unwrap();
    let loading = use_context::<ReadSignal<bool>>().unwrap();

    let rendered_children = children();
    let navigate = use_navigate();

    create_effect(move |_| {
        if !loading.get() && current_user.get().is_none() {
            navigate("/login", Default::default());
        }
    });

    view! {
        {move || {
            if loading.get() {
                view! { <div>"Loading..."</div> }
            } else if current_user.get().is_some() {
                view!{ <div>{rendered_children.clone()}</div>}
            } else {
                view! { <div></div> }
            }
        }}
    }
}

#[component]
pub fn RequireRole(
    #[prop(default = "user".to_string())] role: String,
    children: Children,
) -> impl IntoView {
    let current_user = use_context::<ReadSignal<Option<User>>>().unwrap();
    let loading = use_context::<ReadSignal<bool>>().unwrap();

    let navigate = use_navigate();

    // Render the children once and store the result
    let rendered_children = children();
    let role_mimic = role.clone();

    create_effect(move |_| {
        if !loading.get() {
            if let Some(user) = current_user.get() {
                match role.clone().as_str() {
                    "admin" => {
                        if !user.is_admin() {
                            navigate("/", Default::default());
                        }
                    }
                    "teacher" => {
                        if !user.is_teacher() {
                            navigate("/", Default::default());
                        }
                    }
                    _ => {}
                }
            } else {
                navigate("/login", Default::default());
            }
        }
    });

    view! {
        {move || {
            if loading.get() {
                view! { <div>"Loading..."</div> }
            } else if let Some(user) = current_user.get() {
                match role_mimic.as_str() {
                    "admin" => {
                        if user.is_admin() {
                            view!{ <div>{rendered_children.clone()}</div> }
                        } else {
                            view! { <div>"Unauthorized"</div> }
                        }
                    }
                    "teacher" => {
                        if user.is_teacher() {
                            view!{ <div>{rendered_children.clone()}</div> }
                        } else {
                            view! { <div>"Unauthorized"</div> }
                        }
                    }
                    _ => view!{ <div>{rendered_children.clone()}</div> }
                }
            } else {
                view! { <div></div> }
            }
        }}
    }
}

#[derive(Serialize)]
struct EmailContext {
    reset_link: String,
    // Add more fields as needed for your template
}

#[cfg(feature = "ssr")]
pub async fn send_reset_email(email: &str, reset_token: &str) -> Result<(), String> {
    use reqwest::Client;
    use serde_json::{json, Value};

    // Configuration - in production these should come from environment variables
    let sendgrid_api_key = std::env::var("SENDGRID_API_KEY")
        .map_err(|_| "SENDGRID_API_KEY environment variable not set".to_string())?;
    let app_url = std::env::var("APP_URL")
        .unwrap_or_else(|_| "https://yourapp.com".to_string());
    let from_email = std::env::var("FROM_EMAIL")
        .unwrap_or_else(|_| "noreply@yourapp.com".to_string());
    
    // Determine whether to use sandbox mode based on environment
    let is_development = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()) != "production";
    
    // Create the reset link
    let reset_link = format!("{}/reset-password/{}", app_url, reset_token);

    // Build the SendGrid API request payload
    let mut payload = json!({
        "personalizations": [{
            "to": [{ "email": email }]
        }],
        "from": { "email": from_email },
        "subject": "Password Reset Instructions",
        "content": [{
            "type": "text/plain",
            "value": format!(
                "Click the link below to reset your password:\n\n{}\n\nThis link will expire in 24 hours.",
                reset_link
            )
        }]
    });
    
    // Only enable sandbox mode for development environment
    if is_development {
        // Add sandbox mode setting for development
        if let Some(payload_obj) = payload.as_object_mut() {
            payload_obj.insert(
                "mail_settings".to_string(),
                json!({
                    "sandbox_mode": {
                        "enable": true
                    }
                })
            );
            log::info!("Sending password reset email to {} (sandbox mode)", email);
        }
    } else {
        log::info!("Sending password reset email to {} (production mode)", email);
    }

    // Send the request to SendGrid API
    let client = Client::new();
    let res = client
        .post("https://api.sendgrid.com/v3/mail/send")
        .header("Authorization", format!("Bearer {}", sendgrid_api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to send request to SendGrid: {}", e))?;

    // Check the response
    if res.status().is_success() {
        if is_development {
            log::info!("Password reset email sent successfully to {} (sandbox mode)", email);
        } else {
            log::info!("Password reset email sent successfully to {}", email);
        }
        Ok(())
    } else {
        let status = res.status();
        let body = res.text().await.unwrap_or_else(|_| "No response body".to_string());
        error!("Failed to send email. Status: {}, Body: {}", status, body);
        Err(format!("Failed to send email. Status: {}", status))
    }
}
