use crate::app::components::auth::{LoginForm, RegisterForm};
use crate::app::models::user::User;
use leptos::*;
use leptos_router::*;

#[component]
pub fn LoginPage() -> impl IntoView {
    let (show_register, set_show_register) = create_signal(false);
    let current_user = use_context::<ReadSignal<Option<User>>>().unwrap();
    let navigate = use_navigate();

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
                    }
                } else {
                    view! {
                        <LoginForm />
                        <div class="mt-4 text-center">
                            <span>"Don't have an account? "</span>
                            <button
                                class="text-blue-500 hover:underline"
                                on:click=move |_| set_show_register.set(true)
                            >
                                "Register"
                            </button>
                        </div>
                    }
                }
            }}
        </div>
    }
}
