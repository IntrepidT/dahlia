use crate::app::components::auth::LogoutButton;
use crate::app::components::Header;
use crate::app::models::user::User;
use crate::app::server_functions::auth::{get_current_user, Logout};
use leptos::*;

#[component]
pub fn MyAccount() -> impl IntoView {
    // Get the current user from context (provided by AuthProvider)
    let current_user = use_context::<ReadSignal<Option<User>>>()
        .expect("AuthProvider should provide current_user");
    let loading = use_context::<ReadSignal<bool>>().expect("AuthProvider should provide loading");

    view! {
        <Header/>
        <div class="bg-[#00356B] text-white w-full max-w-[64rem] mx-auto p-6 rounded-md my-4">
            <h1 class="text-2xl font-bold mb-6">"My Account"</h1>

            {move || {
                if loading.get() {
                    view! { <div class="text-center p-8">"Loading..."</div> }
                } else if let Some(user) = current_user.get() {
                    view! {
                        <div class="bg-white text-gray-800 p-4 rounded-md shadow-md">
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div class="mb-4">
                                    <h2 class="text-xl font-semibold text-[#00356B] mb-2">"Account Information"</h2>

                                    <div class="mb-2">
                                        <label class="block text-gray-600 text-sm font-medium">"Username:"</label>
                                        <p class="font-medium">{user.username.clone()}</p>
                                    </div>

                                    <div class="mb-2">
                                        <label class="block text-gray-600 text-sm font-medium">"Email:"</label>
                                        <p class="font-medium">{user.email.clone()}</p>
                                    </div>

                                    <div class="mb-2">
                                        <label class="block text-gray-600 text-sm font-medium">"User ID:"</label>
                                        <p class="font-medium">{user.id.to_string()}</p>
                                    </div>

                                    <div class="mb-2">
                                        <label class="block text-gray-600 text-sm font-medium">"Role:"</label>
                                        <p class="font-medium">
                                            {user.role.clone()}
                                            <span class="ml-2 text-xs bg-blue-100 text-blue-800 py-1 px-2 rounded-full">
                                                {
                                                    if user.is_admin() {
                                                        "Administrator"
                                                    } else if user.is_teacher() {
                                                        "Teacher"
                                                    } else {
                                                        "Standard User"
                                                    }
                                                }
                                            </span>
                                        </p>
                                    </div>
                                </div>

                                <div>
                                    <h2 class="text-xl font-semibold text-[#00356B] mb-2">"Account Actions"</h2>

                                    <a href="/update-profile" class="block text-center w-full bg-[#00356B] text-white py-2 px-4 rounded-md mb-2 hover:bg-[#002347] transition">
                                        "Update Profile"
                                    </a>

                                    <a href="/change-password" class="block text-center w-full bg-yellow-600 text-white py-2 px-4 rounded-md mb-2 hover:bg-yellow-700 transition">
                                        "Change Password"
                                    </a>

                                    {
                                        if user.is_admin() {
                                            view! {
                                                <div>
                                                    <a href="/admin-dashboard" class="block text-center w-full bg-purple-600 text-white py-2 px-4 rounded-md mb-2 hover:bg-purple-700 transition">
                                                        "Admin Dashboard"
                                                    </a>
                                                </div>
                                            }
                                        } else {
                                            view! { <div></div> }
                                        }
                                    }

                                    <div class="w-full">
                                        <LogoutButton/>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                } else {
                    view! {
                        <div class="bg-white text-gray-800 p-8 rounded-md shadow-md text-center">
                            <p class="mb-4">"You are not logged in."</p>
                            <a href="/login" class="bg-[#00356B] text-white py-2 px-4 rounded-md hover:bg-[#002347] transition">
                                "Log In"
                            </a>
                        </div>
                    }
                }
            }}
        </div>
    }
}
