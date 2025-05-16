use crate::app::server_functions::auth::{
    request_password_reset, reset_password, validate_reset_token,
};
use leptos::*;
use leptos_router::{use_params, Params};
use log::info;
use std::time::Duration;

#[component]
pub fn RequestPasswordResetForm() -> impl IntoView {
    let (email, set_email) = create_signal("".to_string());
    let (message, set_message) = create_signal::<Option<(String, bool)>>(None);

    // Create an action to handle the form submission
    let request_reset = create_action(move |_: &()| {
        let email = email.get();

        async move {
            match request_password_reset(email).await {
                Ok(response) => {
                    if response.success {
                        set_message.set(Some((response.message, true)));
                    } else {
                        set_message.set(Some((response.message, false)));
                    }
                }
                Err(err) => {
                    set_message.set(Some((format!("Error: {}", err), false)));
                }
            }
        }
    });

    view! {
        <div class="p-4 bg-white rounded shadow-md max-w-md mx-auto mt-10">
            <h2 class="text-2xl font-bold mb-4">"Reset Your Password"</h2>
            <p class="mb-4 text-gray-600">"Enter your email address and we will send you a link to reset your password."</p>

            {move || {
                message.get().map(|(msg, is_success)| {
                    let bg_class = if is_success { "bg-green-100 text-green-700" } else { "bg-red-100 text-red-700" };
                    view! {
                        <div class={format!("mb-4 p-3 rounded {}", bg_class)}>{msg}</div>
                    }
                })
            }}

            <form on:submit=move |ev| {
                ev.prevent_default();
                request_reset.dispatch(());
            }>
                <div class="mb-4">
                    <label class="block text-gray-700 mb-2" for="email">"Email"</label>
                    <input
                        id="email"
                        type="email"
                        class="w-full p-2 border rounded"
                        required="true"
                        prop:value=move || email.get()
                        on:input=move |ev| {
                            set_email.set(event_target_value(&ev));
                        }
                    />
                </div>

                <button
                    type="submit"
                    class="w-full p-2 bg-blue-500 text-white rounded hover:bg-blue-600"
                    prop:disabled=move || request_reset.pending().get()
                >
                    {move || {
                        if request_reset.pending().get() {
                            "Sending..."
                        } else {
                            "Send Reset Link"
                        }
                    }}
                </button>
            </form>

            <div class="mt-4 text-center">
                <a href="/login" class="text-blue-500 hover:underline">"Back to Login"</a>
            </div>
        </div>
    }
}

#[component]
pub fn ResetPasswordForm() -> impl IntoView {
    // Get the token from the URL
    #[derive(Params, PartialEq, Clone, Debug)]
    struct ResetParams {
        token: String,
    }

    let params = use_params::<ResetParams>();
    let token = move || params.get().map(|p| p.token).unwrap_or_default();

    let (password, set_password) = create_signal("".to_string());
    let (confirm_password, set_confirm_password) = create_signal("".to_string());
    let (message, set_message) = create_signal::<Option<(String, bool)>>(None);
    let (token_valid, set_token_valid) = create_signal(false);
    let (token_checked, set_token_checked) = create_signal(false);

    // Validate the token when the component mounts
    create_effect(move |_| {
        let token_value = token();
        if !token_value.is_empty() {
            spawn_local(async move {
                match validate_reset_token(token_value).await {
                    Ok(valid) => {
                        set_token_valid.set(valid);
                        if !valid {
                            set_message.set(Some(("Invalid or expired reset token. Please request a new password reset.".to_string(), false)));
                        }
                    }
                    Err(_) => {
                        set_token_valid.set(false);
                        set_message.set(Some(("Error validating reset token. Please try again or request a new password reset.".to_string(), false)));
                    }
                }
                set_token_checked.set(true);
            });
        }
    });

    // Create an action to handle the form submission
    let perform_reset = create_action(move |_: &()| {
        let password_value = password.get();
        let confirm_value = confirm_password.get();
        let token_value = token();

        async move {
            // Client-side validation
            if password_value.trim().is_empty() {
                set_message.set(Some(("Password cannot be empty".to_string(), false)));
                return;
            }

            if password_value != confirm_value {
                set_message.set(Some(("Passwords do not match".to_string(), false)));
                return;
            }

            if password_value.len() < 8 {
                set_message.set(Some((
                    "Password must be at least 8 characters".to_string(),
                    false,
                )));
                return;
            }

            // Server-side reset
            match reset_password(token_value, password_value).await {
                Ok(response) => {
                    if response.success {
                        set_message.set(Some((
                            format!("{}. Redirecting to login page...", response.message),
                            true,
                        )));
                        // Redirect to login page after 3 seconds
                        let navigate = leptos_router::use_navigate();
                        set_timeout(
                            move || {
                                navigate("/login", Default::default());
                            },
                            Duration::new(3000, 0),
                        );
                    } else {
                        set_message.set(Some((response.message, false)));
                    }
                }
                Err(err) => {
                    set_message.set(Some((format!("Error: {}", err), false)));
                }
            }
        }
    });

    view! {
        <div class="p-4 bg-white rounded shadow-md max-w-md mx-auto mt-10">
            <h2 class="text-2xl font-bold mb-4">"Set New Password"</h2>

            {move || {
                message.get().map(|(msg, is_success)| {
                    let bg_class = if is_success { "bg-green-100 text-green-700" } else { "bg-red-100 text-red-700" };
                    view! {
                        <div class={format!("mb-4 p-3 rounded {}", bg_class)}>{msg}</div>
                    }
                })
            }}

            {move || {
                if !token_checked.get() {
                    view! { <div class="text-center py-4">"Validating reset token..."</div> }.into_view()
                } else if token_valid.get() {
                    view! {
                        <form on:submit=move |ev| {
                            ev.prevent_default();
                            perform_reset.dispatch(());
                        }>
                            <div class="mb-4">
                                <label class="block text-gray-700 mb-2" for="password">"New Password"</label>
                                <input
                                    id="password"
                                    type="password"
                                    class="w-full p-2 border rounded"
                                    required="true"
                                    prop:value=move || password.get()
                                    on:input=move |ev| {
                                        set_password.set(event_target_value(&ev));
                                    }
                                />
                                <p class="text-xs text-gray-500 mt-1">"Must be at least 8 characters"</p>
                            </div>

                            <div class="mb-4">
                                <label class="block text-gray-700 mb-2" for="confirm-password">"Confirm New Password"</label>
                                <input
                                    id="confirm-password"
                                    type="password"
                                    class="w-full p-2 border rounded"
                                    required="true"
                                    prop:value=move || confirm_password.get()
                                    on:input=move |ev| {
                                        set_confirm_password.set(event_target_value(&ev));
                                    }
                                />
                            </div>

                            <button
                                type="submit"
                                class="w-full p-2 bg-blue-500 text-white rounded hover:bg-blue-600"
                                prop:disabled=move || perform_reset.pending().get()
                            >
                                {move || {
                                    if perform_reset.pending().get() {
                                        "Updating..."
                                    } else {
                                        "Set New Password"
                                    }
                                }}
                            </button>
                        </form>
                    }.into_view()
                } else {
                    view! {
                        <div class="text-center">
                            <p class="mb-4">"This password reset link is invalid or has expired."</p>
                            <a href="/forgot-password" class="text-blue-500 hover:underline">"Request a new reset link"</a>
                        </div>
                    }.into_view()
                }
            }}

            <div class="mt-4 text-center">
                <a href="/login" class="text-blue-500 hover:underline">"Back to Login"</a>
            </div>
        </div>
    }
}
