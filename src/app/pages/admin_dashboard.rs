use crate::app::components::dashboard::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::header::Header;
use leptos::*;
use leptos_router::*;

#[component]
pub fn AdminDashboard() -> impl IntoView {
    let (selected_view, set_selected_view) = create_signal(SidebarSelected::AdminDashboard);
    view! {
        <div class="min-h-screen bg-[#F9F9F8]">
            <Header />
            <DashboardSidebar
                selected_item=selected_view
                set_selected_item=set_selected_view
            />
            <div class="flex flex-1 ml-20 mt-20">
                <p>"This is the Classroom page"</p>
            </div>
        </div>
    }
}
