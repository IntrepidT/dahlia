use crate::app::components::dashboard::chat::Chat;
use crate::app::components::dashboard::scores_ledger::ScoresLedger;
use crate::app::components::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::header::Header;
use crate::app::components::live_testing::live_test::RealtimeTestSession;
use leptos::*;
use leptos_router::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    let (selected_view, set_selected_view) = create_signal(SidebarSelected::Overview);

    // Listen for route changes to update sidebar selection accordingly
    let location = use_location();

    create_effect(move |_| {
        let path = location.pathname.get();
        // Update sidebar selection based on current path
        if path.starts_with("/dashboard") {
            set_selected_view(SidebarSelected::Overview);
        } else if path.starts_with("/studentview") {
            set_selected_view(SidebarSelected::StudentView);
        } else if path.starts_with("/teachers") {
            set_selected_view(SidebarSelected::TeacherView);
        } else if path.starts_with("/testsessions") {
            set_selected_view(SidebarSelected::Live);
        }
        // Note: AdministerTest doesn't have its own page, it's handled by the modal
    });

    view! {
        <div class="bg-[#F9F9F8] h-full">
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
                                <div class="text-2xl font-bold mb-2 text-[#2E3A59]">Overview</div>
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
                                        <ScoresLedger />
                                    </div>
                                </div>
                            </div>
                        },
                        SidebarSelected::Analytics => view! {
                            <div>
                                <div class="text-2xl font-bold text-[#2E3A59]">
                                    Analytics Dashboard
                                    <p class="text-base font-normal mt-4 text-[#2E3A59]">
                                        Explore detailed performance analytics and insights.
                                    </p>
                                </div>
                            </div>
                        },
                        SidebarSelected::Chat => view! {
                            <div>
                                <div class="text-2xl font-bold text-[#2E3A59]">
                                    Chat Logs
                                    <p class="text-base font-normal mt-4">
                                        Communicate with your co-workers.
                                    </p>
                                </div>
                                <div class="h-full">
                                    <Chat />
                                </div>
                            </div>
                        },
                        SidebarSelected::Settings => view! {
                            <div>
                                <div class="text-2xl font-bold text-[#2E3A59]">
                                    Account Settings
                                    <p class="text-base font-normal mt-4 text-[#2E3A59]">
                                        Customize your account preferences and settings.
                                    </p>
                                </div>
                            </div>
                        },
                        // Added handlers for the new navigation items
                        SidebarSelected::Dashboard => view! {
                            <div>
                                <div class="text-2xl font-bold text-[#2E3A59]">
                                    Main Dashboard
                                    <p class="text-base font-normal mt-4 text-[#2E3A59]">
                                        Your main dashboard and controls.
                                    </p>
                                </div>
                                {/* This is essentially a duplicate of Overview or could redirect to it */}
                                <div class="flex gap-4 w-full mt-4">
                                    <div class="flex-1 w-1/2">
                                        <div class="shadow-lg border-gray border-2 h-[20rem] rounded-lg">
                                            <h1 class="text-base font-bold text-xl ml-2 p-2 text-[#2E3A59]">
                                                Dashboard Summary
                                            </h1>
                                            <hr class="text-sm text-gray" />
                                        </div>
                                    </div>
                                    <div class="flex-1 w-1/2">
                                        <div class="shadow-lg border-gray border-2 h-[20rem] rounded-lg">
                                            <h1 class="text-base font-bold text-xl ml-2 p-2 text-[#2E3A59]">
                                                Recent Activity
                                            </h1>
                                            <hr class="text-sm text-gray" />
                                        </div>
                                    </div>
                                </div>
                            </div>
                        },
                        SidebarSelected::StudentView => view! {
                            <div>
                                <div class="text-2xl font-bold text-[#2E3A59]">
                                    Student View
                                    <p class="text-base font-normal mt-4 text-[#2E3A59]">
                                        Manage and view student data and performance.
                                    </p>
                                </div>
                                <div class="flex gap-4 w-full mt-4">
                                    <div class="flex-1">
                                        <div class="shadow-lg border-gray border-2 h-[20rem] rounded-lg">
                                            <h1 class="text-base font-bold text-xl ml-2 p-2 text-[#2E3A59]">
                                                Student List
                                            </h1>
                                            <hr class="text-sm text-gray" />
                                            <div class="p-4">
                                                <p class="text-[#2E3A59]">View and manage your students here.</p>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        },
                        SidebarSelected::TeacherView => view! {
                            <div>
                                <div class="text-2xl font-bold text-[#2E3A59]">
                                    Teacher View
                                    <p class="text-base font-normal mt-4 text-[#2E3A59]">
                                        Teacher management and collaboration tools.
                                    </p>
                                </div>
                                <div class="flex gap-4 w-full mt-4">
                                    <div class="flex-1">
                                        <div class="shadow-lg border-gray border-2 h-[20rem] rounded-lg">
                                            <h1 class="text-base font-bold text-xl ml-2 p-2 text-[#2E3A59]">
                                                Teacher Directory
                                            </h1>
                                            <hr class="text-sm text-gray" />
                                            <div class="p-4">
                                                <p class="text-[#2E3A59]">Connect with other teachers and staff.</p>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        },
                        SidebarSelected::AdministerTest => view! {
                            <div>
                                <div class="text-2xl font-bold text-[#2E3A59]">
                                    Administer Test
                                    <p class="text-base font-normal mt-4 text-[#2E3A59]">
                                        Create and administer tests to your students.
                                    </p>
                                </div>
                                <div class="flex gap-4 w-full mt-4">
                                    <div class="flex-1">
                                        <div class="shadow-lg border-gray border-2 h-[20rem] rounded-lg">
                                            <h1 class="text-base font-bold text-xl ml-2 p-2 text-[#2E3A59]">
                                                Test Management
                                            </h1>
                                            <hr class="text-sm text-gray" />
                                            <div class="p-4">
                                                <p class="text-[#2E3A59]">Please use the sidebar menu to select specific test options.</p>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        },
                        SidebarSelected::Live => view! {
                            <div>
                                <p>This is the dashboard component for Live testing</p>
                            </div>
                        },
                        SidebarSelected::Assessments => view! {
                            <div>
                                <p>This is the assessments component for grouping/organizing your tests</p>
                            </div>
                        },
                        SidebarSelected::Gradebook => view! {
                            <div>
                                <p>This is the gradebook component for viewing and managing student grades</p>
                            </div>
                        },
                        SidebarSelected::AdminDashboard => view! {
                            <div>
                                <p>This is the admin dashboard component</p>
                            </div>
                        },
                    }}
                </main>
            </div>
        </div>
    }
}
