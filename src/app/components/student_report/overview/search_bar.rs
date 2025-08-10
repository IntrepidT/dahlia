use leptos::prelude::*;
use leptos::prelude::*;

#[component]
pub fn SearchBar(
    #[prop(into)] search_query: ReadSignal<String>,
    #[prop(into)] set_search_query: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <div class="relative">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <svg 
                    class="h-4 w-4 text-gray-400" 
                    fill="none" 
                    viewBox="0 0 24 24" 
                    stroke="currentColor"
                >
                    <path 
                        stroke-linecap="round" 
                        stroke-linejoin="round" 
                        stroke-width="2" 
                        d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" 
                    />
                </svg>
            </div>
            <input
                type="text"
                placeholder="Search tests..."
                class="block w-full pl-10 pr-3 py-2.5 border border-gray-200 rounded-lg text-sm placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200 bg-white"
                prop:value=search_query
                on:input=move |ev| {
                    set_search_query(event_target_value(&ev));
                }
            />
        </div>
    }
}
