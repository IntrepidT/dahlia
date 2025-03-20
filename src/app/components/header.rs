use crate::app::components::ShowAdministerTestModal;
use leptos::*;
use leptos_router::*;

#[component]
pub fn Header() -> impl IntoView {
    // State for modal and hover effects
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

    // Determine if a nav link is active
    let is_active = move |path: &str| current_path().starts_with(path);

    view! {
        <header class="sticky top-0 z-50 w-full bg-white shadow-sm">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex justify-between items-center h-16">
                    {/* Logo and brand name */}
                    <div class="flex items-center">
                        <A href="/dashboard" class="flex items-center space-x-3">
                            <div class="bg-[#00356b] p-1.5 rounded-lg">
                                <img
                                    src="/assets/dahliano.png"
                                    alt="Dahlia Software"
                                    class="h-10 w-10 object-contain"
                                />
                            </div>
                            <div>
                                <div class="text-lg font-semibold text-[#00356b]">"Dahlia Software"</div>
                                <div class="text-xs text-gray-500">"for Connie Le"</div>
                            </div>
                        </A>
                    </div>

                    {/* Main Navigation */}
                    <nav class="md:flex items-center space-x-6">
                        <A
                            href="/dashboard"
                            class=move || {
                                if is_active("/dashboard") {
                                    "text-[#00356b] font-medium border-b-2 border-[#00356b] pb-1"
                                } else {
                                    "text-gray-600 hover:text-[#00356b] hover:border-b-2 hover:border-gray-600 pb-1 transition-colors"
                                }
                            }
                        >
                            "Dashboard"
                        </A>
                        <A
                            href="/studentview"
                            class=move || {
                                if is_active("/studentview") {
                                    "text-[#00356b] font-medium border-b-2 border-[#00356b] pb-1"
                                } else {
                                    "text-gray-600 hover:text-[#00356b] hover:border-b-2 hover:border-gray-600 pb-1 transition-colors"
                                }
                            }
                        >
                            "Student View"
                        </A>
                        <A
                            href="/teachers"
                            class=move || {
                                if is_active("/teachers") {
                                    "text-[#00356b] font-medium border-b-2 border-[#00356b] pb-1"
                                } else {
                                    "text-gray-600 hover:text-[#00356b] hover:border-b-2 hover:border-gray-600 pb-1 transition-colors"
                                }
                            }
                        >
                            "Teacher View"
                        </A>

                        {/* Administer Test Dropdown */}
                        <div class="relative">
                            <button
                                on:click=move |_| set_show_administer_modal.update(|v| *v = !*v)
                                on:blur=move |_| set_show_administer_modal.set(false)
                                class="flex items-center text-gray-600 hover:text-[#00356b] hover:border-gray-600 pb-1 transition-colors"
                            >
                                <span>"Administer Test"</span>
                                <span class="ml-1">
                                    <Show when=move || show_administer_modal()>
                                        <img src="/assets/arrow_up.png" class="h-4 w-4" />
                                    </Show>
                                    <Show when=move || !show_administer_modal()>
                                        <img src="/assets/arrow_down.png" class="h-4 w-4" />
                                    </Show>
                                </span>
                            </button>

                            {/* Modal Dropdown */}
                            <Show when=move || show_administer_modal()>
                                <div class="absolute right-0 mt-2 rounded-md shadow-lg z-50">
                                    <ShowAdministerTestModal set_if_show_modal=set_show_administer_modal />
                                </div>
                            </Show>
                        </div>
                    </nav>

                    {/* User Account */}
                    <div class="flex items-center">
                        <A
                            href="/myaccount"
                            class="flex items-center space-x-2 bg-[#00356B] hover:bg-[#00457b] text-white px-4 py-2 rounded-lg transition-colors"
                        >
                            <span>"My Account"</span>
                            <img src="/assets/user.png" alt="User account" class="h-5 w-5" />
                        </A>
                    </div>

                    {/* Mobile menu button - hidden on desktop */}
                    <div class="md:hidden flex items-center">
                        <button class="text-gray-600 hover:text-[#00356b] focus:outline-none">
                            <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16m-7 6h7"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        </header>
    }
}
