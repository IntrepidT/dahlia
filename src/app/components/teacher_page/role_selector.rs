use crate::app::models::user::{User, UserRole};
use crate::app::server_functions::users::update_user_permissions;
use leptos::*;

#[component]
pub fn RoleSelector(
    user: User,
    current_user_role: UserRole,
    current_user_id: i64,
    #[prop(into)] on_role_updated: Callback<()>,
) -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);
    let (is_updating, set_is_updating) = create_signal(false);

    // Check if this is the current user (to prevent self-demotion)
    let is_current_user = user.id == current_user_id;

    // Determine which roles the current user can assign
    let available_roles = move || -> Vec<UserRole> {
        let mut roles = match current_user_role {
            UserRole::SuperAdmin => vec![
                UserRole::Guest,
                UserRole::User,
                UserRole::Teacher,
                UserRole::Admin,
                UserRole::SuperAdmin,
            ],
            UserRole::Admin => vec![
                UserRole::Guest,
                UserRole::User,
                UserRole::Teacher,
                UserRole::Admin,
            ],
            _ => vec![], // Only admins and superadmins can change roles
        };

        // If this is the current user, remove roles that would be a demotion
        if is_current_user {
            roles.retain(|&role| {
                // Keep roles that are equal or higher than current role
                match (current_user_role, role) {
                    (UserRole::SuperAdmin, _) => role == UserRole::SuperAdmin,
                    (UserRole::Admin, _) => matches!(role, UserRole::Admin | UserRole::SuperAdmin),
                    (UserRole::Teacher, _) => matches!(
                        role,
                        UserRole::Teacher | UserRole::Admin | UserRole::SuperAdmin
                    ),
                    (UserRole::User, _) => matches!(
                        role,
                        UserRole::User | UserRole::Teacher | UserRole::Admin | UserRole::SuperAdmin
                    ),
                    (UserRole::Guest, _) => true, // Guests can be promoted to anything
                }
            });
        }

        roles
    };

    let can_manage_roles =
        move || -> bool { matches!(current_user_role, UserRole::Admin | UserRole::SuperAdmin) };

    let update_role = create_action(move |new_role: &UserRole| {
        let new_role = *new_role;
        let user_id = user.id;

        async move {
            set_is_updating(true);

            match update_user_permissions(user_id, new_role).await {
                Ok(_) => {
                    log::info!("Successfully updated user role");
                    on_role_updated(());
                    set_is_open(false);
                }
                Err(e) => {
                    log::error!("Failed to update user role: {:?}", e);
                }
            }

            set_is_updating(false);
        }
    });

    let role_badge_style = move |role: &UserRole| -> String {
        let base_style = "px-2 py-1 text-xs font-medium rounded-md border";
        let color_style = match role {
            UserRole::SuperAdmin => "text-red-700 bg-red-100 border-red-300",
            UserRole::Admin => "text-orange-700 bg-orange-100 border-orange-300",
            UserRole::Teacher => "text-blue-700 bg-blue-100 border-blue-300",
            UserRole::User => "text-green-700 bg-green-100 border-green-300",
            UserRole::Guest => "text-gray-700 bg-gray-100 border-gray-300",
        };
        format!("{} {}", base_style, color_style)
    };

    let can_edit_this_user = move || -> bool {
        can_manage_roles() && (!is_current_user || current_user_role == UserRole::SuperAdmin)
    };

    view! {
        <div class="relative inline-block">
            {move || if can_edit_this_user() {
                view! {
                    <div>
                        <button
                            class=format!(
                                "{} {} cursor-pointer select-none",
                                role_badge_style(&user.role),
                                if is_updating() { "opacity-50" } else { "hover:opacity-80" }
                            )
                            on:click=move |e| {
                                e.stop_propagation();
                                set_is_open(!is_open());
                            }
                            disabled=move || is_updating()
                            title=move || if is_current_user { "Edit your role (limited options)" } else { "Change user role" }
                        >
                            <span class="flex items-center gap-1">
                                {move || if is_updating() {
                                    "Updating...".to_string()
                                } else {
                                    user.role.to_string()
                                }}
                                <svg class="w-3 h-3 ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                                </svg>
                            </span>
                        </button>

                        // Dropdown menu with better positioning
                        <Show when=move || is_open()>
                            <div class="absolute right-0 mt-1 w-36 bg-white rounded-md shadow-lg border border-gray-200 z-50 overflow-hidden">
                                <div class="py-1">
                                    {available_roles().into_iter().map(|role| {
                                        let role_display = role.to_string();
                                        let is_current = role == user.role;
                                        let role_for_click = role;

                                        view! {
                                            <button
                                                class=move || format!(
                                                    "w-full text-left px-3 py-2 text-xs font-medium transition-colors flex items-center gap-2 {}",
                                                    if is_current {
                                                        "bg-gray-100 text-gray-900 cursor-default"
                                                    } else {
                                                        "text-gray-700 hover:bg-gray-50 hover:text-gray-900"
                                                    }
                                                )
                                                on:click=move |e| {
                                                    e.stop_propagation();
                                                    if !is_current {
                                                        update_role.dispatch(role_for_click);
                                                    }
                                                }
                                                disabled=move || is_current || is_updating()
                                            >
                                                <span class=format!("inline-block w-2 h-2 rounded-full {}",
                                                    match role {
                                                        UserRole::SuperAdmin => "bg-red-500",
                                                        UserRole::Admin => "bg-orange-500",
                                                        UserRole::Teacher => "bg-blue-500",
                                                        UserRole::User => "bg-green-500",
                                                        UserRole::Guest => "bg-gray-500",
                                                    }
                                                )></span>
                                                <span>{role_display}</span>
                                                {if is_current {
                                                    view! { <span class="text-gray-500 ml-auto">"✓"</span> }.into_view()
                                                } else {
                                                    view! {}.into_view()
                                                }}
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                        </Show>
                    </div>
                }.into_view()
            } else {
                // Non-admin users or users viewing themselves (non-superadmin) see a static badge
                view! {
                    <span class=role_badge_style(&user.role)>
                        {user.role.to_string()}
                        {if is_current_user && can_manage_roles() {
                            view! { <span class="ml-1 text-xs opacity-60">"(you)"</span> }.into_view()
                        } else {
                            view! {}.into_view()
                        }}
                    </span>
                }.into_view()
            }}

            // Click outside to close dropdown
            <Show when=move || is_open()>
                <div
                    class="fixed inset-0 z-40"
                    on:click=move |_| set_is_open(false)
                ></div>
            </Show>
        </div>
    }
}
