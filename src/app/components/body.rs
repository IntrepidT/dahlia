use leptos::*;
use crate::app::components::nav::NavBar;

#[component]
pub fn Body() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-100">
            <NavBar />
            <div class="container mx-auto p-4">
                <slot />
            </div>
        </div>
    }
}
