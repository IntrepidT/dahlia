use crate::app::models::user::{User, UserJwt};
use crate::app::server_functions::users::{get_user, update_user};
use leptos::*;

#[component]
pub fn UpdateProfileModal(
    #[prop(into)] show: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_success: Callback<()>,
) -> impl IntoView {
    // Get the current user from context (provided by AuthProvider)
    let current_user = use_context::<ReadSignal<Option<UserJwt>>>()
        .expect("AuthProvider should provide current_user");

    // Create a derived signal for user_id to avoid unnecessary refetching
    let user_id = create_memo(move |_| current_user.get().map(|user| user.id));

    // State for form inputs
    let (first_name, set_first_name) = create_signal(String::new());
    let (last_name, set_last_name) = create_signal(String::new());
    let (phone_number, set_phone_number) = create_signal(String::new());

    // State for feedback messages
    let (success_message, set_success_message) = create_signal(None::<String>);
    let (error_message, set_error_message) = create_signal(None::<String>);
    let (is_submitting, set_is_submitting) = create_signal(false);

    // Fetch current user data to pre-populate form
    let user_resource = create_resource(
        move || (user_id.get(), show.get()), // Refetch when modal is shown
        move |(id, _)| async move {
            match id {
                Some(user_id) => get_user(user_id).await.ok(),
                None => None,
            }
        },
    );

    // Pre-populate form when user data is fetched
    create_effect(move |_| {
        if let Some(Some(user)) = user_resource.get() {
            if let Some(first) = user.first_name.clone() {
                set_first_name(first);
            }
            if let Some(last) = user.last_name.clone() {
                set_last_name(last);
            }
            if let Some(phone) = user.phone_number.clone() {
                set_phone_number(phone);
            }
        }
    });

    // Reset form state when modal is closed
    create_effect(move |_| {
        if !show.get() {
            set_success_message(None);
            set_error_message(None);
            set_is_submitting(false);
        }
    });

    // Handle form submission
    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();

        // Clear previous messages
        set_success_message(None);
        set_error_message(None);
        set_is_submitting(true);

        let current_first_name = first_name.get();
        let current_last_name = last_name.get();
        let current_phone_number = phone_number.get();
        let on_success_callback = on_success.clone();

        // We need the original user data to update only the fields we want to change
        spawn_local(async move {
            if let Some(user_id) = user_id.get() {
                match get_user(user_id).await {
                    Ok(mut user) => {
                        // Update only the fields we want to change
                        user.first_name = Some(current_first_name);
                        user.last_name = Some(current_last_name);
                        user.phone_number = Some(current_phone_number);

                        // Send the update request
                        match update_user(user).await {
                            Ok(_) => {
                                set_success_message(Some(
                                    "Profile updated successfully!".to_string(),
                                ));
                                // Notify parent component of success
                                on_success_callback.call(());
                                // Wait briefly to show success message before closing
                                set_timeout(
                                    move || {
                                        on_close.call(());
                                    },
                                    std::time::Duration::from_millis(1500),
                                );
                            }
                            Err(e) => {
                                set_error_message(Some(format!("Failed to update profile: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        set_error_message(Some(format!("Failed to fetch user data: {}", e)));
                    }
                }
            } else {
                set_error_message(Some("User not authenticated".to_string()));
            }

            set_is_submitting(false);
        });
    };

    let close_modal = move |_| {
        on_close.call(());
    };

    // Only render modal content when shown
    view! {
        {move || if show.get() {
            view! {
                <div class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center overflow-y-auto">
                    <div class="bg-white text-[#2E3A59] p-6 rounded-md shadow-lg max-w-md w-full mx-4"
                         role="dialog"
                         aria-modal="true">
                        <div class="flex justify-between items-center mb-4">
                            <h2 class="text-xl font-bold text-[#2E3A59]">"Update Profile"</h2>
                            <button
                                type="button"
                                class="text-gray-400 hover:text-gray-600"
                                on:click=close_modal
                                aria-label="Close"
                            >
                                <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                                </svg>
                            </button>
                        </div>

                        <form on:submit=on_submit>
                            // Success message
                            {move || success_message.get().map(|msg| view! {
                                <div class="mb-4 p-2 bg-green-100 text-green-700 rounded">
                                    {msg}
                                </div>
                            })}

                            // Error message
                            {move || error_message.get().map(|msg| view! {
                                <div class="mb-4 p-2 bg-red-100 text-red-700 rounded">
                                    {msg}
                                </div>
                            })}

                            <div class="mb-4">
                                <label for="firstName" class="block text-[#2E3A59] text-sm font-medium mb-1">
                                    "First Name"
                                </label>
                                <input
                                    type="text"
                                    id="firstName"
                                    value={move || first_name.get()}
                                    on:input=move |ev| {
                                        set_first_name(event_target_value(&ev));
                                    }
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-[#2E3A59]"
                                />
                            </div>

                            <div class="mb-4">
                                <label for="lastName" class="block text-[#2E3A59] text-sm font-medium mb-1">
                                    "Last Name"
                                </label>
                                <input
                                    type="text"
                                    id="lastName"
                                    value={move || last_name.get()}
                                    on:input=move |ev| {
                                        set_last_name(event_target_value(&ev));
                                    }
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-[#2E3A59]"
                                />
                            </div>

                            <div class="mb-4">
                                <label for="phoneNumber" class="block text-[#2E3A59] text-sm font-medium mb-1">
                                    "Phone Number"
                                </label>
                                <input
                                    type="tel"
                                    id="phoneNumber"
                                    value={move || phone_number.get()}
                                    on:input=move |ev| {
                                        set_phone_number(event_target_value(&ev));
                                    }
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-[#2E3A59]"
                                />
                            </div>

                            <div class="flex items-center justify-between mt-6">
                                <button
                                    type="submit"
                                    class="bg-[#2E3A59] text-white py-2 px-6 rounded-md hover:bg-[#DADADA] transition disabled:opacity-50 disabled:cursor-not-allowed"
                                    disabled=move || is_submitting.get()
                                >
                                    {move || if is_submitting.get() { "Updating..." } else { "Update Profile" }}
                                </button>

                                <button
                                    type="button"
                                    on:click=close_modal
                                    class="text-[#2E3A59] hover:underline"
                                >
                                    "Cancel"
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            }
        } else {
            view! { <div> </div> }
        }}
    }
}
