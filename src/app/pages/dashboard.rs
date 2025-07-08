use crate::app::components::auth::server_auth_components::ServerAuthGuard;
use crate::app::components::auth::test_saml::SamlTestButton;
use crate::app::components::dashboard::dashboard_deanonymizer::DashboardDeanonymizer;
use crate::app::components::dashboard::scores_ledger::ScoresLedger;
use crate::app::components::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::header::Header;
use crate::app::models::user::{SessionUser, UserRole};
use crate::app::server_functions::saml_auth::{create_saml_config, get_saml_institutions};
use leptos::*;
use leptos_router::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    view! {
        <ServerAuthGuard page_path="/dashboard">
            <DashboardContent />
        </ServerAuthGuard>
    }
}

#[component]
fn DashboardContent() -> impl IntoView {
    let (selected_view, set_selected_view) = create_signal(SidebarSelected::Overview);
    let location = use_location();

    create_effect(move |_| {
        let path = location.pathname.get();
        if path.starts_with("/dashboard") {
            set_selected_view(SidebarSelected::Overview);
        } else if path.starts_with("/studentview") {
            set_selected_view(SidebarSelected::StudentView);
        } else if path.starts_with("/teachers") {
            set_selected_view(SidebarSelected::TeacherView);
        } else if path.starts_with("/testsessions") {
            set_selected_view(SidebarSelected::Live);
        }
    });

    view! {
        <div class="bg-[#F9F9F8] h-full">
            <Header />
            <div class="flex h-full">
                <DashboardSidebar
                    selected_item=selected_view
                    set_selected_item=set_selected_view
                />
                <main class="flex-1 ml-20 px-10 mt-5">
                    {move || match selected_view() {
                        SidebarSelected::Overview => view! {
                            <div>
                                <DashboardDeanonymizer />
                                <div class="flex justify-between items-center my-4">
                                    <div class="text-2xl font-bold text-[#2E3A59]">
                                        "Overview"
                                    </div>
                                    //<SamlTestButton /> this was used for testing SAML, can be
                                //moved or removed later
                                </div>
                                <div class="flex gap-4 w-full">
                                    <div class="flex-1 w-1/2">
                                        <div class="shadow-lg border-gray border-2 h-[20rem] rounded-lg">
                                            <h1 class="text-base font-bold text-xl ml-2 p-2 text-[#2E3A59]">
                                                Today
                                            </h1>
                                            <hr class="text-sm text-gray" />
                                        </div>
                                    </div>
                                    <div class="flex-1 w-1/2">
                                        <div class="shadow-lg border-gray border-2 h-[20rem] rounded-lg">
                                            <h1 class="text-base font-bold text-xl ml-2 p-2 text-[#2E3A59]">
                                                Logs
                                            </h1>
                                            <hr class="text-sm text-gray" />
                                        </div>
                                    </div>
                                </div>
                                <div class="text-2xl font-bold mt-5 ">
                                    <div class="flex-1 w-full h-[20rem] rounded-lg mt-2">
                                        <Suspense fallback=move || view! {
                                            <div class="flex justify-center items-center h-40">
                                                <svg class="animate-spin h-6 w-6 text-indigo-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                                </svg>
                                                <span class="ml-2 text-[#2E3A59]">Loading scores...</span>
                                            </div>
                                        }>
                                            <ScoresLedger />
                                        </Suspense>
                                    </div>
                                </div>
                            </div>
                        },
                        _ => view! {
                            <div class="text-2xl font-bold text-[#2E3A59]">
                                "Admin-only content"
                            </div>
                        }
                    }}
                </main>
            </div>
        </div>
    }
}
