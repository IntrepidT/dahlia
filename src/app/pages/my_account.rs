use crate::app::components::auth::LogoutButton;
use crate::app::components::update_user_modal::UpdateProfileModal; // Import the new modal component
use crate::app::components::Header;
use crate::app::models::user::User;
use crate::app::models::user::UserJwt;
use crate::app::server_functions::auth::{get_current_user, Logout};
use crate::app::server_functions::users::get_user;
use leptos::*;

#[component]
pub fn MyAccount() -> impl IntoView {
    // State to control the visibility of the update profile modal
    let (show_update_modal, set_show_update_modal) = create_signal(false);

    // Get the current user from context (provided by AuthProvider)
    let current_user = use_context::<ReadSignal<Option<UserJwt>>>()
        .expect("AuthProvider should provide current_user");

    // Create a derived signal for user_id to avoid unnecessary refetching
    let user_id = create_memo(move |_| current_user.get().map(|user| user.id));

    // Fetch full user data when authenticated
    let user_resource = create_resource(
        move || user_id.get(),
        move |id| async move {
            match id {
                Some(user_id) => get_user(user_id).await.ok(),
                None => None,
            }
        },
    );

    // Handle opening the update profile modal
    let open_update_modal = move |_| {
        set_show_update_modal.set(true);
    };

    // Handle closing the update profile modal
    let close_update_modal = move |_| {
        set_show_update_modal.set(false);
    };

    // Handle successful profile update
    let on_profile_updated = move |_| {
        // Refetch user data to update the UI
        user_resource.refetch();
    };

    let loading = use_context::<ReadSignal<bool>>().expect("AuthProvider should provide loading");

    view! {
        <div class="bg-[#F9F9F8]">
            <Header/>
            <div class="bg-[#2E3A59] text-white w-full max-w-[64rem] mx-auto p-6 rounded-md my-4">
                <h1 class="text-2xl font-bold mb-6">"My Account"</h1>

                {move || {
                    if loading.get() {
                        view! { <div class="text-center p-8">"Loading..."</div> }
                    } else if let Some(user) = current_user.get() {
                        // Define reusable function to display user data safely
                        let display_user_field = |field_value: Option<String>| {
                            field_value.unwrap_or_else(|| "Not provided".to_string())
                        };

                        // Create a derived signal for the full user data
                        let user_data = move || user_resource.get().flatten();

                        view! {
                            <div>
                                <div class="bg-white text-[#2E3A59] p-4 rounded-md shadow-md">
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                        <div class="mb-4">
                                            <h2 class="text-xl font-semibold text-[#2E3A59] mb-2">"Account Information"</h2>

                                            <div class="mb-2">
                                                <label class="block text-[#2E3A59] text-sm font-medium">"Username:"</label>
                                                <p class="font-medium text-[#2E3A59]">{user.username.clone()}</p>
                                            </div>

                                            <div class="mb-2">
                                                <label class="block text-[#2E3A59] text-sm font-medium">"Firstname:"</label>
                                                <p class="font-medium text-[#2E3A59]">
                                                    {move || user_data().map(|u| u.first_name.clone()).unwrap_or(Some("Not provided".to_string()))}
                                                </p>
                                            </div>

                                            <div class="mb-2">
                                                <label class="block text-[#2E3A59] text-sm font-medium">"Lastname:"</label>
                                                <p class="font-medium text-[#2E3A59]">
                                                    {move || user_data().map(|u| u.last_name.clone()).unwrap_or(Some("Not provided".to_string()))}
                                                </p>
                                            </div>

                                            <div class="mb-2">
                                                <label class="block text-[#2E3A59] text-sm font-medium">"Email:"</label>
                                                <p class="font-medium text-[#2E3A59]">{user.email.clone()}</p>
                                            </div>

                                            <div class="mb-2">
                                                <label class="block text-[#2E3A59] text-sm font-medium">"User ID:"</label>
                                                <p class="font-medium text-[#2E3A59]">{user.id.to_string()}</p>
                                            </div>

                                            <div class="mb-2">
                                                <label class="block text-gray-600 text-sm font-medium">"Role:"</label>
                                                <p class="font-medium text-[#2E3A59]">
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

                                            <div class="mb-2">
                                                <label class="block text-gray-600 text-sm font-medium">"Email Verification:"</label>
                                                <p class="font-medium text-[#2E3A59]">
                                                    {move || user_data().map(|u| if u.email_verified { "✓" } else { "❌" }).unwrap_or("❌")}
                                                </p>
                                            </div>

                                            <div class="mb-2">
                                                <label class="block text-gray-600 text-sm font-medium">"Phone Verification:"</label>
                                                <p class="font-medium text-[#2E3A59]">
                                                    {move || user_data().map(|u| if u.phone_verified { "✓" } else { "❌" }).unwrap_or("❌")}
                                                </p>
                                            </div>

                                            <div class="mb-2">
                                                <label class="block text-gray-600 text-sm font-medium">"Phone Number: "</label>
                                                <p class="font-medium text-[#2E3A59]">
                                                    {move || user_data()
                                                        .and_then(|u| u.phone_number)
                                                        .unwrap_or_else(|| "Not provided".to_string())}
                                                </p>
                                            </div>
                                        </div>

                                        <div class="flex-1 flex-row">
                                            <h2 class="text-xl font-semibold text-[#2E3A59] mb-2">"Account Actions"</h2>

                                            <button
                                                on:click=open_update_modal
                                                class="block text-center w-[20rem] bg-[#2E3A59] text-white py-2 px-4 rounded-md mb-2 hover:bg-[#DADADA] transition"
                                            >
                                                "Update Profile"
                                            </button>

                                            <div class="w-[20rem]">
                                                <LogoutButton/>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                // Render the update profile modal, hidden by default
                                <UpdateProfileModal
                                    show=Signal::derive(move || show_update_modal.get())
                                    on_close=Callback::new(close_update_modal)
                                    on_success=Callback::new(on_profile_updated)
                                />
                            </div>
                        }
                    } else {
                        view! {
                            <div class="bg-[#F9F9F8] text-[#2E3A59] p-8 rounded-md shadow-md text-center">
                                <p class="mb-4">"You are not logged in."</p>
                                <a href="/login" class="bg-[#2E3A59] text-white py-2 px-4 rounded-md hover:bg-[#DADADA] transition">
                                    "Log In"
                                </a>
                            </div>
                        }
                    }
                }}
            </div>
        </div>
    }
}
