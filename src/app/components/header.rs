use leptos::prelude::*;
use leptos_router::components::*;

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="sticky top-0 z-50 w-full bg-[#F9F9F8] backdrop-blur bg-opacity-90">
            <div class="max-w-full mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex justify-between items-center h-20">
                    // Logo and brand name
                    <div class="flex items-center transform hover:scale-105 transition-transform duration-200">
                        <a
                            href="/dashboard"
                            class="flex items-center"
                        >
                            <div class="rounded-lg">
                                <img
                                    src="/assets/teapot2.png"
                                    alt="Teapot Testing"
                                    class="h-24 w-auto"
                                />
                            </div>
                            <div class="hidden sm:block">
                                <div class="font-montserrat text-4xl font-bold text-[#2E3A59] leading-tight">
                                    "teapot v2"
                                </div>
                            </div>
                        </a>
                    </div>

                    // User Account
                    <div class="flex items-center transform hover:scale-105 transition-transform duration-200">
                        <a
                            href="/myaccount"
                            class="flex items-center space-x-2 bg-[#2E3A59] hover:bg-opacity-80 text-[#F9F9F8] px-4 py-2 rounded-lg transition-colors duration-200 text-sm font-medium"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
                                <circle cx="12" cy="7" r="4"></circle>
                            </svg>
                        </a>
                    </div>

                    // Mobile menu button - hidden on desktop
                    <div class="md:hidden flex items-center">
                        <button class="text-[#DADADA] hover:text-[#2E3A59] focus:outline-none transition-colors duration-200">
                            <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M4 6h16M4 12h16m-7 6h7"
                                >
                                </path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        </header>
    }
}
