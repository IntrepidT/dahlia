use crate::app::components::Header;
use crate::app::models::user::SessionUser;
use leptos::*;
use leptos_router::*;

#[component]
pub fn HomePage() -> impl IntoView {
    // Access the auth context signals directly
    let current_user =
        use_context::<ReadSignal<Option<SessionUser>>>().expect("Auth context not found");
    let loading = use_context::<ReadSignal<bool>>().expect("Auth context not found");

    // Create a view that will check authentication status and redirect if needed
    view! {
        <Suspense fallback=move || view! { <p>"Loading..."</p> }>
            {move || {
                // If not loading and user is already logged in, redirect to dashboard
                if !loading.get() && current_user.get().is_some() {
                    view! { <Redirect path="/dashboard"/> }
                } else {
                    // Otherwise, show the normal homepage content
                    view! {
                        <div class="bg-[#F9F9F8] h-screen overflow-hidden flex flex-col">
                            <Header />
                            <main id="main-content" role="main" class="flex-1 min-h-0">
                                <div class="max-w-8xl mx-auto px-10 h-full">
                                    <div class="h-5/6 items-center justify-center mt-20 rounded-2xl flex-col relative overflow-hidden">
                                        // Multi-layered background for smooth loading
                                        <div class="absolute inset-0 bg-gradient-to-br from-blue-600 via-purple-600 to-indigo-700"></div>
                                        <div class="absolute inset-0 bg-[url('/assets/home23.png')] bg-cover bg-center"></div>

                                        <div class="h-5/6 pt-20 ml-20 mt-30 relative z-10">
                                            <h1 class="text-5xl font-extrabold text-left text-white mt-20 mb-10 drop-shadow-lg">
                                                Simplifying<br/>Standardized<br/>Testing.
                                            </h1>
                                            <p class="text-2xl font-semibold text-left text-white mt-10 drop-shadow-md">
                                                Bringing your testing needs together,<br/>
                                                so you can pursue what is important.
                                            </p>
                                            <div class="flex relative mt-10 font-base justify-start text-center text-white">
                                                <A href="/login" class="font-semibold text-center text-white">
                                                    <div class="bg-[#2E3A59] rounded-2xl border-white border-1 pl-3 pr-3 py-3 hover:bg-[#3a4660] transition-colors shadow-lg">
                                                        "Log in"
                                                        <img src="/assets/arrow.png" alt="arrow" class="inline h-6 w-6 pb-1" />
                                                    </div>
                                                </A>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </main>
                        </div>
                    }.into_view()
                }
            }}
        </Suspense>
    }
}
