use crate::app::components::settings_modal::SettingsModal;
use icondata::IoChatbubbleEllipsesOutline;
use icondata::{
    AiApiOutlined, BsClipboardCheck, BsGraphUpArrow, ChNotesTick, FaChildrenSolid,
    LuLayoutDashboard, RiAdminUserFacesLine,
};
use leptos::prelude::*;
use leptos_router::path;
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
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::*;
use leptos_router::hooks::*;
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
    // Get the current user ID from expect_context
    let current_user = expect_context::<ReadSignal<Option<SessionUser>>>();
    let user_settings_resource = Resource::new(
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
    let (current_path, set_current_path) = signal(String::new());

    // Effect to track current route using use_location instead of Router context
    Effect::new(move |_| {
        let location = use_location();
        set_current_path(location.pathname.get());
    });

    let (is_expanded, set_is_expanded) = signal(false);
    let (is_navigating, set_is_navigating) = signal(false);
    Effect::new(move |_| {
        let _current_path = current_path();
        set_is_navigating.set(true);
        set_timeout(
            move || {
                set_is_navigating.set(false);
            },
            Duration::from_millis(500),
        );
    });
    let (show_administer_modal, set_show_administer_modal) = signal(false);
    let (show_settings, set_show_settings) = signal(false);

    // New signal for pinned state
    let (is_pinned_closed, set_is_pinned_closed) = signal(false);
    Effect::new(move |_| {
        if let Some(Some(settings)) = user_settings_resource.get() {
            set_is_pinned_closed.set(settings.ui.pinned_sidebar);
        }
    });

    // Handle window size for responsive behavior
    let (is_small_screen, set_is_small_screen) = signal(false);

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

    // Combined class computation for the main sidebar div
    let sidebar_class = move || {
        let base_classes = "fixed left-0 top-20 h-[calc(100vh-4rem)] bg-[#F9F9F8] shadow-lg transition-all duration-300 ease-in-out z-40";

        let width_class = if is_expanded() {
            if is_small_screen() {
                "w-48"
            } else {
                "w-64"
            }
        } else {
            if is_small_screen() {
                "w-16"
            } else {
                "w-20"
            }
        };

        format!("{} {}", base_classes, width_class)
    };

    view! {
        <div class="relative">
            <div
                class=sidebar_class
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
                            icon=FaChildrenSolid
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
                                icon=RiAdminUserFacesLine
                                label="Admin Dashboard (beta)"
                                path="/admindashboard"
                                is_expanded=is_expanded.into()
                                is_active=Signal::derive(move || current_path().starts_with("/classrooms"))
                                is_small_screen=is_small_screen.into()
                            />
                        </Show>
                        <SidebarNavLink
                            icon=ChNotesTick
                            label="Assessments"
                            path="/assessments"
                            is_expanded=is_expanded.into()
                            is_active=Signal::derive(move || current_path().starts_with("/assessments"))
                            is_small_screen=is_small_screen.into()
                        />
                        <SidebarNavLink
                            icon=BsGraphUpArrow
                            label="Gradebook (beta)"
                            path="/gradebook"
                            is_expanded=is_expanded.into()
                            is_active=Signal::derive(move || current_path().starts_with("/gradebook"))
                            is_small_screen=is_small_screen.into()
                        />
                        <SidebarNavLink
                            icon=BsClipboardCheck
                            label="Tests"
                            path="/test-manager"
                            is_expanded=is_expanded.into()
                            is_active=Signal::derive(move || current_path().starts_with("/test-manager"))
                            is_small_screen=is_small_screen.into()
                        />
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
                                attr:class="w-5 h-5 text-[#2E3A59] flex-shrink-0"
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
                                attr:class="w-6 h-6 text[#2E3A59] flex-shrink-0"
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
                    on_close=move || {
                        let _ = set_show_settings.set(false);
                    }
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
    // Combined class computation for the item div
    let item_class = move || {
        let base_classes =
            "flex items-center cursor-pointer hover:bg-[#DADADA] p-2 rounded-md transition-colors";
        if is_selected() {
            format!("{} bg-blue-100", base_classes)
        } else {
            base_classes.to_string()
        }
    };

    // Combined class computation for the span
    let span_class = move || {
        let base_class = "font-semibold text-sm sm:text-base";
        if is_selected.get() {
            format!("{} {}", base_class, BLUE_COLOR)
        } else {
            format!("{} {}", base_class, GRAY_COLOR)
        }
    };

    view! {
        <div
            class=item_class
            on:click=on_click
        >
            <Icon
                icon=icon
                attr:class="w-6 h-6 text-[#2E3A59] flex-shrink-0"
            />
            <div class="overflow-hidden whitespace-nowrap">
                <Show
                    when=move || is_expanded()
                    fallback=|| view! { <></> }
                >
                    <div class="flex flex-col ml-2">
                        <span class=span_class>
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
    // Combined class computation for the nav div
    let nav_class = move || {
        let base_class = "rounded-lg";
        if is_active() {
            format!("{} bg-blue-100", base_class)
        } else {
            base_class.to_string()
        }
    };

    // Combined class computation for the span
    let span_class = move || {
        let base_class = "ml-2 font-semibold text-sm sm:text-base";
        if is_active.get() {
            format!("{} {}", base_class, BLUE_COLOR)
        } else {
            format!("{} {}", base_class, GRAY_COLOR)
        }
    };

    view! {
        <div class=nav_class>
            <A
                href={path}
                attr:class="flex items-center cursor-pointer hover:bg-[#DADADA] p-2 rounded-md transition-colors"
            >
                <Icon
                    icon=icon
                    attr:class="w-6 h-6 text-[#2E3A59] flex-shrink-0"
                />
                <div class="overflow-hidden whitespace-nowrap">
                    <Show
                        when=move || is_expanded()
                        fallback=|| view! { <></> }
                    >
                        <span class=span_class>
                            {label}
                        </span>
                    </Show>
                </div>
            </A>
        </div>
    }
}
