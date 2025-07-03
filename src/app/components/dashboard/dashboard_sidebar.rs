use crate::app::components::settings_modal::SettingsModal;
use icondata::IoChatbubbleEllipsesOutline;
use icondata::{
    AiApiOutlined, AiBarChartOutlined, AiHomeOutlined, AiSettingOutlined, ChStack, FiBook,
};
// Add new imports for additional icons, including pin/unpin icons
use crate::app::components::ShowAdministerTestModal;
use crate::app::models::{setting_data::UserSettings, user::SessionUser};
use crate::app::server_functions::user_settings::get_user_settings;
use icondata::{
    AiCoffeeOutlined,
    AiDashboardOutlined,
    BiClipboardRegular,
    IoPeopleOutline,
    // Add pin/unpin icons
    IoPinOutline,
    IoPinSharp,
    IoPricetagOutline,
    IoSettingsOutline,
};
use leptos::ev::MouseEvent;
use leptos::*;
use leptos_icons::Icon;
use leptos_router::*;
use std::time::Duration;

#[derive(Clone, PartialEq, Debug)]
pub enum SidebarSelected {
    Overview,
    Analytics,
    Settings,
    Chat,
    Dashboard,
    StudentView,
    TeacherView,
    AdministerTest,
    AdminDashboard,
    Live,
    Gradebook,
    Assessments,
}
const GRAY_COLOR: &str = "text-[#DADADA]";
const BLUE_COLOR: &str = "text-[#2E3A59]";
const BLUE_HUE: &str = "bg-blue-100";
const BG_BLUE: &str = "bg-[#2E3A59]";
const WHITE_COLOR: &str = "text-[#F9F9F8]";

#[component]
pub fn DashboardSidebar(
    selected_item: ReadSignal<SidebarSelected>,
    set_selected_item: WriteSignal<SidebarSelected>,
) -> impl IntoView {
    // Get the current user ID from use_context
    let current_user =
        use_context::<ReadSignal<Option<SessionUser>>>().expect("AuthProvider not Found");
    let user_settings_resource = create_resource(
        move || current_user.get().map(|user| user.id),
        move |id| async move {
            match id {
                Some(user_id) => match get_user_settings(user_id).await {
                    Ok(settings) => Some(settings),
                    Err(e) => {
                        log::error!("Failed to fetch user settings: {}", e);
                        // Return default settings if fetch fails
                        Some(UserSettings::default())
                    }
                },
                None => Some(UserSettings::default()), // Default settings for no user
            }
        },
    );
    let user_id = current_user.get().map(|user| user.id).unwrap_or_default();

    // Handle current route for active styling
    let (current_path, set_current_path) = create_signal(String::new());

    // Effect to track current route
    create_effect(move |_| {
        if let Some(route_context) = use_context::<RouterContext>() {
            set_current_path(route_context.pathname().get());
        } else {
            set_current_path(String::from("/"));
        }
    });

    let (is_expanded, set_is_expanded) = create_signal(false);
    let (is_navigating, set_is_navigating) = create_signal(false);
    create_effect(move |_| {
        let _current_path = current_path();
        set_is_navigating.set(true);
        set_timeout(
            move || {
                set_is_navigating.set(false);
            },
            Duration::from_millis(500),
        );
    });
    let (show_administer_modal, set_show_administer_modal) = create_signal(false);
    let (show_settings, set_show_settings) = create_signal(false);

    // New signal for pinned state
    let (is_pinned_closed, set_is_pinned_closed) = create_signal(false);
    create_effect(move |_| {
        if let Some(Some(settings)) = user_settings_resource.get() {
            set_is_pinned_closed.set(settings.ui.pinned_sidebar);
        }
    });

    // Handle window size for responsive behavior
    let (is_small_screen, set_is_small_screen) = create_signal(false);

    // Computed position for dropdown modal based on screen size
    let modal_position = move || {
        let base_left = if is_pinned_closed() {
            if is_small_screen() {
                "5rem"
            } else {
                "5.5rem"
            }
        } else {
            if is_small_screen() {
                "12rem"
            } else {
                "16rem"
            }
        };

        format!("left: {}; top: 12rem;", base_left)
    };

    // Handle mouseenter event with consideration for pinned state
    let handle_mouseenter = move |_| {
        if !is_pinned_closed.get() && !is_navigating.get() && user_settings_resource.get().is_some()
        {
            set_is_expanded(true);
        }
    };

    // Handle mouseleave event with consideration for pinned state
    let handle_mouseleave = move |_| {
        // Only close the sidebar if we're not hovering the modal and not pinned
        if !show_administer_modal.get() && !is_pinned_closed.get() {
            set_is_expanded(false);
        }
    };

    // Toggle pinned state
    let toggle_pinned = move |_| {
        let new_value = !is_pinned_closed.get();
        set_is_pinned_closed.set(new_value);

        if new_value {
            set_is_expanded.set(false);
            set_show_administer_modal.set(false);
        }
    };

    // Handle administer test click with consideration for pinned state
    let handle_administer_click = move |_| {
        set_show_administer_modal.update(|v| *v = !*v);
    };

    view! {
        <div class="relative">
            <div
                class="fixed left-0 top-20 h-[calc(100vh-4rem)] bg-[#F9F9F8] shadow-lg transition-all duration-300 ease-in-out z-40"
                class:w-16={move || !is_expanded() && is_small_screen()}
                class:w-20={move || !is_expanded() && !is_small_screen()}
                class:w-48={move || is_expanded() && is_small_screen()}
                class:w-64={move || is_expanded() && !is_small_screen()}
                on:mouseenter=handle_mouseenter
                on:mouseleave=handle_mouseleave
            >
                <div class="flex flex-col h-full p-4 overflow-y-auto">
                    <div class="space-y-4 flex-1">
                        // Transferred navigation items from header
                        <SidebarNavLink
                            icon=AiDashboardOutlined
                            label="Dashboard (beta)"
                            path="/dashboard"
                            is_expanded=is_expanded.into()
                            is_active=Signal::derive(move || current_path().starts_with("/dashboard"))
                            is_small_screen=is_small_screen.into()
                        />
                        <SidebarNavLink
                            icon=IoPeopleOutline
                            label="Student View"
                            path="/studentview"
                            is_expanded=is_expanded.into()
                            is_active=Signal::derive(move || current_path().starts_with("/studentview"))
                            is_small_screen=is_small_screen.into()
                        />
                        <SidebarNavLink
                            icon=AiCoffeeOutlined
                            label="Teacher View"
                            path="/teachers"
                            is_expanded=is_expanded.into()
                            is_active=Signal::derive(move || current_path().starts_with("/teachers"))
                            is_small_screen=is_small_screen.into()
                        />
                        <Show when=move || current_user.get().map(|user| user.is_admin()).unwrap_or(false)>
                            <SidebarNavLink
                                icon=BiClipboardRegular
                                label="Admin Dashboard (beta)"
                                path="/admindashboard"
                                is_expanded=is_expanded.into()
                                is_active=Signal::derive(move || current_path().starts_with("/classrooms"))
                                is_small_screen=is_small_screen.into()
                            />
                        </Show>
                        <SidebarNavLink
                            icon=AiApiOutlined
                            label="Join Live Session"
                            path="/testsessions"
                            is_expanded=is_expanded.into()
                            is_active=Signal::derive(move || current_path().starts_with("/testsessions"))
                            is_small_screen=is_small_screen.into()
                        />
                        <SidebarNavLink
                            icon=ChStack
                            label="Assessments"
                            path="/assessments"
                            is_expanded=is_expanded.into()
                            is_active=Signal::derive(move || current_path().starts_with("/assessments"))
                            is_small_screen=is_small_screen.into()
                        />
                        <SidebarNavLink
                            icon=FiBook
                            label="Gradebook (beta)"
                            path="/gradebook"
                            is_expanded=is_expanded.into()
                            is_active=Signal::derive(move || current_path().starts_with("/gradebook"))
                            is_small_screen=is_small_screen.into()
                        />

                        // Administer Test item with dropdown - Updated click handler
                        <div
                            class="flex items-center cursor-pointer hover:bg-[#DADADA] p-2 rounded-md transition-colors"
                            on:click=handle_administer_click
                        >
                            <Icon
                                icon=IoPricetagOutline
                                class="w-6 h-6 flex-shrink-0 text-[#2E3A59]"
                            />
                            <div class="overflow-hidden whitespace-nowrap">
                                <Show
                                    when=move || is_expanded()
                                    fallback=|| view! { <></> }
                                >
                                    <div class="flex items-center justify-between ml-2 w-full">
                                        <span class="font-semibold text-sm sm:text-base">"Administer Test"</span>
                                    </div>
                                </Show>
                            </div>
                        </div>
                    </div>

                    <div class="space-y-4 mt-auto pb-4">

                        // Divider
                        <div class="border-t border-[#DADADA] my-2"></div>

                        // Pin/Unpin button
                        <div
                            class="flex items-center cursor-pointer hover:bg-[#DADADA] p-2 rounded-md transition-colors"
                            on:click=toggle_pinned
                            title=move || if is_pinned_closed() { "Unpin sidebar (allow expand on hover)" } else { "Pin sidebar closed" }
                        >
                            <Icon
                                icon={if is_pinned_closed() { IoPinSharp } else { IoPinOutline }}
                                class="w-5 h-5 text-[#2E3A59] flex-shrink-0"
                            />
                            <Show
                                when=move || is_expanded()
                                fallback=|| view! { <></> }
                            >
                                <span class="ml-2 font-semibold text-sm sm:text-base">
                                    {move || if is_pinned_closed() { "Unpin" } else { "Pin Sidebar" }}
                                </span>
                            </Show>
                        </div>

                        <button
                            class="flex items-center cursor-pointer hover:bg-[#DADADA] p-2 rounded-md transition-colors"
                            on:click=move |_| set_show_settings.set(true)
                        >
                            <Icon
                                icon=IoSettingsOutline
                                class="w-6 h-6 text[#2E3A59] flex-shrink-0"
                            />
                            <div class="overflow-hidden whitespace-nowrap">
                                <Show
                                    when=move || is_expanded()
                                    fallback=|| view! { <></> }
                                >
                                    <span class="ml-2 font-semibold text-sm sm:text-base">
                                        {"Settings (beta)"}
                                    </span>
                                </Show>
                            </div>
                        </button>

                    </div>
                </div>
            </div>

            // Modal Dropdown - Improved display logic for pinned state
            <Show when=move || show_administer_modal()>
                <div
                    class="fixed z-50 hover-area"
                    style=move || modal_position()
                    on:mouseenter=move |_| {
                        // Keep sidebar expanded when hovering modal
                        if !is_pinned_closed() {
                            set_show_administer_modal(true);
                        }
                    }
                    on:mouseleave=move |_| {
                        // If pinned closed, hide modal but keep sidebar collapsed
                        set_show_administer_modal(false);
                        if !is_pinned_closed() {
                            set_is_expanded(false);
                        }
                    }
                >
                    <ShowAdministerTestModal set_if_show_modal=set_show_administer_modal />
                </div>
            </Show>

            <Show when=move || show_settings()>
                <SettingsModal
                    show=show_settings
                    on_close=move |_| set_show_settings.set(false)
                    user_id=user_id
                />
            </Show>
        </div>
    }
}

#[component]
fn SidebarItem(
    icon: icondata::Icon,
    label: &'static str,
    description: &'static str,
    is_expanded: Signal<bool>,
    is_selected: Signal<bool>,
    is_small_screen: Signal<bool>,
    on_click: impl Fn(MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <div
            class="flex items-center cursor-pointer hover:bg-[#DADADA] p-2 rounded-md transition-colors"
            class:bg-blue-100=move || is_selected()
            on:click=on_click
        >
            <Icon
                icon=icon
                class="w-6 h-6 text-[#2E3A59] flex-shrink-0"
            />
            <div class="overflow-hidden whitespace-nowrap">
                <Show
                    when=move || is_expanded()
                    fallback=|| view! { <></> }
                >
                    <div class="flex flex-col ml-2">
                        <span
                            class="font-semibold text-sm sm:text-base"
                            class:GRAY_COLOR=move || !is_selected.get()
                            class:BLUE_COLOR=move || is_selected.get()
                        >
                            {label}
                        </span>
                        <Show
                            when=move || !is_small_screen()
                            fallback=|| view! { <></> }
                        >
                            <span class="text-xs text-[#2E3A59]">{description}</span>
                        </Show>
                    </div>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn SidebarNavLink(
    icon: icondata::Icon,
    label: &'static str,
    path: &'static str,
    is_expanded: Signal<bool>,
    is_active: Signal<bool>,
    is_small_screen: Signal<bool>,
) -> impl IntoView {
    view! {
        <div
            class="rounded-lg"
            class:bg-blue-100=move || is_active()
        >
            <A
                href={path}
                class="flex items-center cursor-pointer hover:bg-[#DADADA] p-2 rounded-md transition-colors"
            >
                <Icon
                    icon=icon
                    class="w-6 h-6 text-[#2E3A59] flex-shrink-0"
                />
                <div class="overflow-hidden whitespace-nowrap">
                    <Show
                        when=move || is_expanded()
                        fallback=|| view! { <></> }
                    >
                        <span
                            class="ml-2 font-semibold text-sm sm:text-base"
                            class:GRAY_COLOR=move || !is_active.get()
                            class:BLUE_COLOR=move || is_active.get()
                        >
                            {label}
                        </span>
                    </Show>
                </div>
            </A>
        </div>
    }
}
