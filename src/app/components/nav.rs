use crate::app::components::auth::LogoutButton;
use crate::app::models::user::User;
use leptos::*;
use leptos_router::*;

#[component]
pub fn NavBar() -> impl IntoView {
    let current_user = use_context::<ReadSignal<Option<User>>>().unwrap();

    view! {
        <nav class="bg-blue-600 text-white p-4">
            <div class="container mx-auto flex justify-between items-center">
                <A href="/" class="text-xl font-bold">
                    "Dahlia"
                </A>

                <div class="flex space-x-4">
                    <A href="/" class="hover:text-blue-200">
                        "Home"
                    </A>

                    {move || {
                        if let Some(user) = current_user.get() {
                            // User is logged in
                            view! {
                                <div class="flex items-center space-x-4">
                                    <A href="/studentview" class="hover:text-blue-200">
                                        "Student View"
                                    </A>

                                    {if user.is_teacher() {
                                        view! {
                                            <div>
                                                <A href="/teachers" class="hover:text-blue-200">
                                                    "Teachers"
                                                </A>
                                                <A href="/admintest" class="hover:text-blue-200">
                                                    "Administer Test"
                                                </A>
                                            </div>
                                        }
                                    } else {
                                        view! { <div></div> }
                                    }}

                                    <A href="/myaccount" class="hover:text-blue-200">
                                        "My Account"
                                    </A>

                                    <span class="border-l border-blue-400 h-6 mx-2"></span>

                                    <span class="italic">
                                        {format!("Hi, {}", user.username)}
                                    </span>

                                    <LogoutButton />
                                </div>
                            }
                        } else {
                            // User is not logged in
                            view! {
                                <div>
                                    <A href="/login" class="hover:text-blue-200">
                                        "Login"
                                    </A>
                                </div>
                            }
                        }
                    }}
                </div>
            </div>
        </nav>
    }
}
