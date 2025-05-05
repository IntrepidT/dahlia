use icondata::IoChatbubbleEllipsesOutline;
use icondata::{AiApiOutlined, AiBarChartOutlined, AiHomeOutlined, AiSettingOutlined, ChStack};
// Add new imports for additional icons, including pin/unpin icons
use crate::app::components::ShowAdministerTestModal;
use icondata::{
    AiCoffeeOutlined,
    AiDashboardOutlined,
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
    Live,
    Assessments,
}
const GRAY_COLOR: &str = "text-[#DADADA]";
const BLUE_COLOR: &str = "text-[#2E3A59]";
const BLUE_HUE: &str = "bg-blue-100";
const BG_BLUE: &str = "bg-[#2E3A59]";

#[component]
pub fn DashboardSidebar(
    selected_item: ReadSignal<SidebarSelected>,
    set_selected_item: WriteSignal<SidebarSelected>,
) -> impl IntoView {
    let (is_expanded, set_is_expanded) = create_signal(false);
    let (show_administer_modal, set_show_administer_modal) = create_signal(false);

    // New signal for pinned state
    let (is_pinned_closed, set_is_pinned_closed) = create_signal(false);

    // Handle window size for responsive behavior
    let (is_small_screen, set_is_small_screen) = create_signal(false);

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

    // Handle window resize events
    let window = window();

    // Function to check screen size
    let check_screen_size = move || {
        // Direct usage of window without trying to unwrap Result
        let width = window.inner_width().unwrap().as_f64().unwrap();
        set_is_small_screen(width < 768.0); // 768px is typical md breakpoint
    };

    // Check screen size on mount
    check_screen_size();

    // Set up resize event listener - use window_event_listener instead of use_event_listener
    // The returned handle will be automatically cleaned up when the component is removed
    let _cleanup = window_event_listener(ev::resize, move |_| {
        check_screen_size();
    });

    // We don't need an explicit on_cleanup as the WindowListenerHandle will be dropped automatically

    // Computed position for dropdown modal based on screen size
    let modal_position = move || {
        if is_small_screen() {
            "left: 12rem; top: 12rem;" // Adjusted position for small screens
        } else {
            "left: 16rem; top: 12rem;" // Original position
        }
    };

    // Handle mouseenter event with consideration for pinned state
    let handle_mouseenter = move |_| {
        if !is_pinned_closed() {
            set_is_expanded(true);
        }
    };

    // Handle mouseleave event with consideration for pinned state
    let handle_mouseleave = move |_| {
        // Only close the sidebar if we're not hovering the modal and not pinned
        if !show_administer_modal() && !is_pinned_closed() {
            set_is_expanded(false);
        }
    };

    // Toggle pinned state
    let toggle_pinned = move |_| {
        let new_value = !is_pinned_closed.get();
        set_is_pinned_closed.set(new_value);

        // If unpinning, we immediately expand the sidebar
        if !new_value {
            set_is_expanded(true);
        } else {
            // If pinning closed, we collapse the sidebar
            set_is_expanded(false);
        }
    };

    view! {
        <div class="relative">
            <div
                class="fixed left-0 top-16 h-[calc(100vh-4rem)] bg-[#F9F9F8] shadow-lg transition-all duration-300 ease-in-out z-40"
                class:w-16={move || !is_expanded() && is_small_screen()}
                class:w-20={move || !is_expanded() && !is_small_screen()}
                class:w-48={move || is_expanded() && is_small_screen()}
                class:w-64={move || is_expanded() && !is_small_screen()}
                on:mouseenter=handle_mouseenter
                on:mouseleave=handle_mouseleave
            >
                <div class="flex flex-col h-full p-4 overflow-y-auto">
                    <div class="space-y-4">
                        // Transferred navigation items from header
                        <SidebarNavLink
                            icon=AiDashboardOutlined
                            label="Dashboard"
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

                        // Administer Test item with dropdown
                        <div
                            class="flex items-center cursor-pointer hover:bg-[#DADADA] p-2 rounded-md transition-colors"
                            on:click=move |_| set_show_administer_modal.update(|v| *v = !*v)
                        >
                            <Icon
                                icon=IoPricetagOutline
                                class="w-6 h-6 flex-shrink-0 text-[#2E3A59]"
                            />
                            <div class="overflow-hidden">
                                <Show
                                    when=move || is_expanded()
                                    fallback=|| view! { <></> }
                                >
                                    <div class="flex items-center justify-between w-full">
                                        <span class="font-semibold text-sm sm:text-base">"Administer Test"</span>
                                        <span>
                                            <Show when=move || show_administer_modal()>
                                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                    <polyline points="18 15 12 9 6 15"></polyline>
                                                </svg>
                                            </Show>
                                            <Show when=move || !show_administer_modal()>
                                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                    <polyline points="6 9 12 15 18 9"></polyline>
                                                </svg>
                                            </Show>
                                        </span>
                                    </div>
                                </Show>
                            </div>
                        </div>

                        <SidebarNavLink
                            icon=IoSettingsOutline
                            label="Settings"
                            path="/settings"
                            is_expanded=is_expanded.into()
                            is_active=Signal::derive(move || current_path().starts_with("/settings"))
                            is_small_screen=is_small_screen.into()
                        />

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
                                    {move || if is_pinned_closed() { "Unpin" } else { "Pin" }}
                                </span>
                            </Show>
                        </div>

                        /*// Original sidebar items
                        <SidebarItem
                            icon=AiHomeOutlined
                            label="Overview"
                            description="View your main dashboard metrics"
                            is_expanded=is_expanded.into()
                            is_selected=Signal::derive(move || selected_item() == SidebarSelected::Overview)
                            on_click=move |_| set_selected_item(SidebarSelected::Overview)
                            is_small_screen=is_small_screen.into()
                        />
                        <SidebarItem
                            icon=AiBarChartOutlined
                            label="Analytics"
                            description="Deep dive into your performance data"
                            is_expanded=is_expanded.into()
                            is_selected=Signal::derive(move || selected_item() == SidebarSelected::Analytics)
                            on_click=move |_| set_selected_item(SidebarSelected::Analytics)
                            is_small_screen=is_small_screen.into()
                        />
                        <SidebarItem
                            icon=IoChatbubbleEllipsesOutline
                            label="Chat"
                            description="Communicate with your coworkers"
                            is_expanded=is_expanded.into()
                            is_selected=Signal::derive(move || selected_item() == SidebarSelected::Chat)
                            on_click=move |_| set_selected_item(SidebarSelected::Chat)
                            is_small_screen=is_small_screen.into()
                        />
                        <SidebarItem
                            icon=AiSettingOutlined
                            label="Settings"
                            description="Customize your account preferences"
                            is_expanded=is_expanded.into()
                            is_selected=Signal::derive(move || selected_item() == SidebarSelected::Settings)
                            on_click=move |_| set_selected_item(SidebarSelected::Settings)
                            is_small_screen=is_small_screen.into()
                        />
                        */
                    </div>
                </div>
            </div>

            // Modal Dropdown - Improved positioning with responsive adjustments
            <Show when=move || show_administer_modal() && is_expanded()>
                <div
                    class="fixed z-50 hover-area"
                    style=move || modal_position()
                    on:mouseenter=move |_| set_is_expanded(true) // Keep sidebar open when hovering modal
                    on:mouseleave=move |_| {
                        // Only close everything when leaving the modal and not pinned
                        if !is_pinned_closed() {
                            set_is_expanded(false);
                        }
                        set_show_administer_modal(false);
                    }
                >
                    <ShowAdministerTestModal set_if_show_modal=set_show_administer_modal />
                </div>
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
            <div class="overflow-hidden">
                <Show
                    when=move || is_expanded()
                    fallback=|| view! { <></> }
                >
                    <div class="flex flex-col">
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
                    class="w-6 h-6 text-[#2E3A59] flex-shrink-0 mr-2"
                />
                <div class="overflow-hidden">
                    <Show
                        when=move || is_expanded()
                        fallback=|| view! { <></> }
                    >
                        <span
                            class="font-semibold text-sm sm:text-base"
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
