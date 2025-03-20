use crate::app::components::header::Header;
use leptos::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    view! {
        <Header />
        <div class="mt-20 px-20">
            <p>This is the Dashboard</p>
        </div>
    }
}
