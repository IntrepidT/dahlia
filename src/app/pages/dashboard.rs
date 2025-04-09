use crate::app::components::dashboard::scores_ledger::ScoresLedger;
use crate::app::components::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::header::Header;
use leptos::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    let (selected_view, set_selected_view) = create_signal(SidebarSelected::Overview);

    view! {
        <Header />
        <div class="flex h-full">
            <DashboardSidebar
                selected_item=selected_view
                set_selected_item=set_selected_view
            />
            <main class="flex-1 mt-16 ml-20 px-10">
                {move || match selected_view() {
                    SidebarSelected::Overview => view! {
                        <div>
                            <div class="text-2xl font-bold">
                                Overview
                                <div class="flex-1 w-full shadow-lg border-gray border-2 h-[20rem] rounded-lg mt-2">
                                    <h1 class="text-base font-bold text-xl ml-2 p-2">
                                        Today
                                    </h1>
                                    <hr class="text-sm text-gray" />
                                </div>
                            </div>
                            <div class="text-2xl font-bold mt-5 ">
                                <div class="flex-1 w-full h-[20rem] rounded-lg mt-2">
                                    <ScoresLedger />
                                </div>
                            </div>
                        </div>
                    },
                    SidebarSelected::Analytics => view! {
                        <div>
                            <div class="text-2xl font-bold">
                                Analytics Dashboard
                                <p class="text-base font-normal mt-4">
                                    Explore detailed performance analytics and insights.
                                </p>
                            </div>
                        </div>
                    },
                    SidebarSelected::Settings => view! {
                        <div>
                            <div class="text-2xl font-bold">
                                Account Settings
                                <p class="text-base font-normal mt-4">
                                    Customize your account preferences and settings.
                                </p>
                            </div>
                        </div>
                    }
                }}
            </main>
        </div>
    }
}
