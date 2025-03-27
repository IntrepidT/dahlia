use icondata::{AiBarChartOutlined, AiHomeOutlined, AiSettingOutlined};
use leptos::ev::MouseEvent;
use leptos::*;
use leptos_icons::Icon;

#[derive(Clone, PartialEq, Debug)]
pub enum SidebarSelected {
    Overview,
    Analytics,
    Settings,
}

#[component]
pub fn DashboardSidebar(
    selected_item: ReadSignal<SidebarSelected>,
    set_selected_item: WriteSignal<SidebarSelected>,
) -> impl IntoView {
    let (is_expanded, set_is_expanded) = create_signal(false);

    view! {
        <div
            class="fixed left-0 top-16 h-[calc(100vh-4rem)] bg-white shadow-lg transition-all duration-300 ease-in-out z-40"
            class:w-20={move || !is_expanded()}
            class:w-64={move || is_expanded()}
            on:mouseenter=move |_| set_is_expanded(true)
            on:mouseleave=move |_| set_is_expanded(false)
        >
            <div class="flex flex-col h-full p-4 overflow-y-auto">
                <div class="space-y-4">
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
                        icon=AiSettingOutlined
                        label="Settings"
                        description="Customize your account preferences"
                        is_expanded=is_expanded.into()
                        is_selected=Signal::derive(move || selected_item() == SidebarSelected::Settings)
                        on_click=move |_| set_selected_item(SidebarSelected::Settings)
                    />
                </div>
            </div>
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
            class="flex items-center cursor-pointer hover:bg-gray-100 p-2 rounded-md transition-colors"
            class:bg-blue-100=move || is_selected()
            on:click=on_click
        >
            <Icon
                icon=icon
                class="w-6 h-6 mr-4 text-gray-600"
            />
            <div class="overflow-hidden">
                <Show
                    when=move || is_expanded()
                    fallback=|| view! { <></> }
                >
                    <div class="flex flex-col">
                        <span
                            class="font-semibold"
                            class:text-gray-800=move || !is_selected.get()
                            class:text-blue-800=move || is_selected.get()
                        >
                            {label}
                        </span>
                        <span class="text-xs text-gray-500">{description}</span>
                    </div>
                </Show>
            </div>
        </div>
    }
}
