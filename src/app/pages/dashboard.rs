use crate::app::components::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::header::Header;
use leptos::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    let (selected_view, set_selected_view) = create_signal(SidebarSelected::Overview);

    view! {
        <Header />
        <div class="flex">
            <DashboardSidebar
                selected_item=selected_view
                set_selected_item=set_selected_view
            />
            <main class="flex-1 mt-16 ml-20 px-10">
                {move || match selected_view() {
                    SidebarSelected::Overview => view! {
                        <div class="text-2xl font-bold">
                            Dashboard Overview
                            <p class="text-base font-normal mt-4">
                                Welcome to your dashboard. Here you can view key metrics and insights.
                            </p>
                        </div>
                    },
                    SidebarSelected::Analytics => view! {
                        <div class="text-2xl font-bold">
                            Analytics Dashboard
                            <p class="text-base font-normal mt-4">
                                Explore detailed performance analytics and insights.
                            </p>
                        </div>
                    },
                    SidebarSelected::Settings => view! {
                        <div class="text-2xl font-bold">
                            Account Settings
                            <p class="text-base font-normal mt-4">
                                Customize your account preferences and settings.
                            </p>
                        </div>
                    }
                }}
            </main>
        </div>
    }
}
