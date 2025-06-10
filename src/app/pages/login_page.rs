use crate::app::components::auth::enhanced_login_form::EnhancedLoginForm;
use crate::app::components::auth::login_form::{LoginForm, RegisterForm};
use crate::app::middleware::global_settings::use_settings;
use crate::app::models::user::UserJwt;
use leptos::*;
use leptos_router::*;

#[component]
pub fn LoginPage() -> impl IntoView {
    let (show_register, set_show_register) = create_signal(false);
    let current_user = use_context::<ReadSignal<Option<UserJwt>>>().unwrap();
    let navigate = use_navigate();

    // Get settings to check if student protections are enabled
    let (settings, _) = use_settings();
    let student_protections_enabled = move || settings.get().student_protections;

    // If already logged in, redirect to home
    create_effect(move |_| {
        if current_user.get().is_some() {
            navigate("/", Default::default());
        }
    });

    view! {
        <div class="max-w-md mx-auto mt-10 bg-[#f9f9f8]">
            {move || {
                if show_register.get() {
                    view! {
                        <RegisterForm />
                        <div class="mt-4 text-center">
                            <span>"Already have an account? "</span>
                            <button
                                class="text-blue-500 hover:underline"
                                on:click=move |_| set_show_register.set(false)
                            >
                                "Login"
                            </button>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        // Conditionally render the appropriate login form
                        {move || {
                            if student_protections_enabled() {
                                view! { <EnhancedLoginForm /> }.into_view()
                            } else {
                                view! { <LoginForm /> }.into_view()
                            }
                        }}

                        <div class="mt-4 text-center">
                            <div class="flex justify-center">
                                <button
                                    class="text-blue-500 hover:underline"
                                    on:click=move |_| set_show_register.set(true)
                                >
                                    "Register"
                                </button>
                                <a href="/forgot-password">
                                    <button
                                        class="text-red-500 hover:underline ml-4"
                                    >
                                        "Forgot Password?"
                                    </button>
                                </a>
                            </div>
                        </div>
                    }.into_view()
                }
            }}
        </div>
    }
}
