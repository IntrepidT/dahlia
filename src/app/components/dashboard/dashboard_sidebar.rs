use icondata::IoChatbubbleEllipsesOutline;
use icondata::{AiApiOutlined, AiBarChartOutlined, AiHomeOutlined, AiSettingOutlined};
// Add new imports for additional icons
use crate::app::components::ShowAdministerTestModal;
use icondata::{AiCoffeeOutlined, AiDashboardOutlined, IoPeopleOutline, IoPricetagOutline};
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

    view! {
        <div class="relative">
            <div
                class="fixed left-0 top-16 h-[calc(100vh-4rem)] bg-[#F9F9F8] shadow-lg transition-all duration-300 ease-in-out z-40"
                class:w-20={move || !is_expanded()}
                class:w-64={move || is_expanded()}
                on:mouseenter=move |_| set_is_expanded(true)
                on:mouseleave=move |_| {
                    // Only close the sidebar if we're not hovering the modal
                    if !show_administer_modal() {
                        set_is_expanded(false);
                    }
                }
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
                        />
                        <SidebarNavLink
                            icon=IoPeopleOutline
                            label="Student View"
                            path="/studentview"
                            is_expanded=is_expanded.into()
                            is_active=Signal::derive(move || current_path().starts_with("/studentview"))
                        />
                        <SidebarNavLink
                            icon=AiCoffeeOutlined
                            label="Teacher View"
                            path="/teachers"
                            is_expanded=is_expanded.into()
                            is_active=Signal::derive(move || current_path().starts_with("/teachers"))
                        />
                        <SidebarNavLink
                            icon=AiApiOutlined
                            label="Join Live Session"
                            path="/testsessions"
                            is_expanded=is_expanded.into()
                            is_active=Signal::derive(move || current_path().starts_with("/testsessions"))
                        />

                        // Administer Test item with dropdown
                        <div
                            class="flex items-center cursor-pointer hover:bg-[#DADADA] p-2 rounded-md transition-colors"
                            on:click=move |_| set_show_administer_modal.update(|v| *v = !*v)
                        >
                            <Icon
                                icon=IoPricetagOutline
                                class="w-6 h-6 mr-4 flex-shrink-0 text-[#2E3A59]"
                            />
                            <div class="overflow-hidden">
                                <Show
                                    when=move || is_expanded()
                                    fallback=|| view! { <></> }
                                >
                                    <div class="flex items-center justify-between w-full">
                                        <span class="font-semibold text-[#2E3A59]">"Administer Test"</span>
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

                        // Divider
                        <div class="border-t border-[#DADADA] my-2"></div>

                        /*// Original sidebar items
                        <SidebarItem
                            icon=AiHomeOutlined
                            label="Overview"
                            description="View your main dashboard metrics"
                            is_expanded=is_expanded.into()
                            is_selected=Signal::derive(move || selected_item() == SidebarSelected::Overview)
                            on_click=move |_| set_selected_item(SidebarSelected::Overview)
                        />
                        <SidebarItem
                            icon=AiBarChartOutlined
                            label="Analytics"
                            description="Deep dive into your performance data"
                            is_expanded=is_expanded.into()
                            is_selected=Signal::derive(move || selected_item() == SidebarSelected::Analytics)
                            on_click=move |_| set_selected_item(SidebarSelected::Analytics)
                        />
                        <SidebarItem
                            icon=IoChatbubbleEllipsesOutline
                            label="Chat"
                            description="Communicate with your coworkers"
                            is_expanded=is_expanded.into()
                            is_selected=Signal::derive(move || selected_item() == SidebarSelected::Chat)
                            on_click=move |_| set_selected_item(SidebarSelected::Chat)
                        />
                        <SidebarItem
                            icon=AiSettingOutlined
                            label="Settings"
                            description="Customize your account preferences"
                            is_expanded=is_expanded.into()
                            is_selected=Signal::derive(move || selected_item() == SidebarSelected::Settings)
                            on_click=move |_| set_selected_item(SidebarSelected::Settings)
                        />
                        */
                    </div>
                </div>
            </div>

            // Modal Dropdown - Improved positioning
            <Show when=move || show_administer_modal() && is_expanded()>
                <div
                    class="fixed z-50 hover-area"
                    style="left: 16rem; top: 12rem;"
                    on:mouseenter=move |_| set_is_expanded(true) // Keep sidebar open when hovering modal
                    on:mouseleave=move |_| {
                        // Only close everything when leaving the modal
                        set_is_expanded(false);
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
                class="w-6 h-6 text-[#2E3A59] mr-4 flex-shrink-0"
            />
            <div class="overflow-hidden">
                <Show
                    when=move || is_expanded()
                    fallback=|| view! { <></> }
                >
                    <div class="flex flex-col">
                        <span
                            class="font-semibold"
                            class:GRAY_COLOR=move || !is_selected.get()
                            class:BLUE_COLOR=move || is_selected.get()
                        >
                            {label}
                        </span>
                        <span class="text-xs text-[#2E3A59]">{description}</span>
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
                    class="w-6 h-6 text-[#2E3A59] flex-shrink-0 mr-4"
                />
                <div class="overflow-hidden">
                    <Show
                        when=move || is_expanded()
                        fallback=|| view! { <></> }
                    >
                        <span
                            class="font-semibold"
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
