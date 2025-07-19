use leptos::*;

#[component]
pub fn TestInstructions(instructions: Option<String>) -> impl IntoView {
    let (is_expanded, set_is_expanded) = create_signal(false);
    
    let has_instructions = instructions.as_ref().map(|i| !i.trim().is_empty()).unwrap_or(false);
    
    if !has_instructions {
        return view! { <div></div> }.into_view();
    }
    
    let instructions_text = instructions.unwrap_or_default();
    
    // Create a separate handler function
    let toggle_expanded = move |_| {
        set_is_expanded.update(|expanded| *expanded = !*expanded);
    };
    
    view! {
        <div class="mb-6 flex items-center justify-center">
            <div class="w-1/2 bg-blue-50 border-l-4 border-blue-400 overflow-hidden rounded-lg">
                <button
                    class="w-full px-4 py-3 text-left focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 rounded-t-lg hover:bg-blue-100 transition-colors"
                    on:click=toggle_expanded
                >
                    <div class="flex items-center justify-between">
                        <div class="flex items-center gap-3">
                            <div class="w-2 h-2 bg-blue-500 rounded-full"></div>
                            <h3 class="text-sm font-medium text-blue-800">
                                "Test Instructions"
                            </h3>
                        </div>
                        <div class="flex items-center gap-2">
                            <span class="text-xs text-blue-600 font-medium">
                                {move || if is_expanded() { "Hide" } else { "Show" }}
                            </span>
                            <svg 
                                class="w-4 h-4 text-blue-600 transition-transform duration-200"
                                class:rotate-180=move || is_expanded()
                                fill="none" 
                                stroke="currentColor" 
                                viewBox="0 0 24 24"
                            >
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                            </svg>
                        </div>
                    </div>
                </button>
                
                <Show when=move || is_expanded()>
                    <div class="px-4 pb-4 pt-2">
                        <div class="bg-white rounded border border-blue-200 p-4">
                            <div class="text-sm text-gray-700 whitespace-pre-wrap leading-relaxed">
                                {instructions_text.clone()}
                            </div>
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }.into_view()
}
