use crate::app::models::test::Test;
use leptos::*;
use std::rc::Rc;

#[component]
pub fn SelectTestModal(
    test: Rc<Test>,
    show_modal: ReadSignal<bool>,
    set_show_modal: WriteSignal<bool>,
) -> impl IntoView {
    // Create separate variables for each navigation type to avoid borrow checker issues
    let realtime_test = test.clone();
    let individual_test = test.clone();
    let grid_test = test.clone();

    // Event handlers for each type of test navigation
    let on_realtime_click = move |_| {
        let test_id = realtime_test.test_id.clone();
        let navigate = leptos_router::use_navigate();
        navigate(&format!("/test-session/{}", test_id), Default::default());
        set_show_modal.set(false);
    };

    let on_individual_click = move |_| {
        let test_id = individual_test.test_id.clone();
        let navigate = leptos_router::use_navigate();
        navigate(&format!("/flashcardset/{}", test_id), Default::default());
        set_show_modal.set(false);
    };

    let on_grid_test_click = move |_| {
        let test_id = grid_test.test_id.clone();
        let navigate = leptos_router::use_navigate();
        navigate(&format!("/gridtest/{}", test_id), Default::default());
        set_show_modal.set(false);
    };

    // Event handler for cancel button
    let on_cancel_click = move |_| {
        set_show_modal.set(false);
    };

    // Check if this is a variation test for display purposes
    let is_variation = test.name.contains(" - ")
        && (test.name.to_lowercase().contains("randomized")
            || test.name.to_lowercase().contains("distinct")
            || test.name.to_lowercase().contains("practice")
            || test.comments.to_lowercase().contains("variation:"));

    // Extract base name and variation type for display
    let (base_name, variation_type) = if is_variation {
        let parts: Vec<&str> = test.name.split(" - ").collect();
        (
            parts
                .get(0)
                .map(|s| s.to_string())
                .unwrap_or_else(|| test.name.clone()),
            parts.get(1).map_or("Variation", |v| v).to_string(),
        )
    } else {
        (test.name.clone(), String::new())
    };

    view! {
        <Show when=move || show_modal()>
            <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                <div class="bg-white rounded-lg shadow-xl p-6 max-w-md w-full mx-4">
                    <h3 class="text-xl font-semibold text-gray-800 mb-4">Choose Test Mode</h3>

                    // Show variation info in modal if applicable
                    {if is_variation {
                        view! {
                            <div class="mb-4 p-3 bg-blue-50 rounded-lg border border-blue-200">
                                <p class="text-sm text-blue-800">
                                    "Using " <strong>{variation_type.clone()}</strong> " version of " <strong>{base_name.clone()}</strong>
                                </p>
                            </div>
                        }
                    } else {
                        view! { <div></div> }
                    }}

                    <div class="space-y-4">
                        <button
                            class="w-full p-3 bg-blue-600 text-white rounded-lg flex items-center justify-between hover:bg-blue-700 transition-colors"
                            on:click=on_realtime_click.clone()
                        >
                            <span class="text-lg">Real-time Live Testing</span>
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-12a1 1 0 10-2 0v4a1 1 0 00.293.707l2.828 2.829a1 1 0 101.415-1.415L11 9.586V6z" clip-rule="evenodd" />
                            </svg>
                        </button>

                        <button
                            class="w-full p-3 bg-green-600 text-white rounded-lg flex items-center justify-between hover:bg-green-700 transition-colors"
                            on:click=on_individual_click.clone()
                        >
                            <span class="text-lg">Individual Flashcard Test</span>
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                <path d="M9 2a1 1 0 000 2h2a1 1 0 100-2H9z" />
                                <path fill-rule="evenodd" d="M4 5a2 2 0 012-2 3 3 0 003 3h2a3 3 0 003-3 2 2 0 012 2v11a2 2 0 01-2 2H6a2 2 0 01-2-2V5zm3 4a1 1 0 000 2h.01a1 1 0 100-2H7zm3 0a1 1 0 000 2h3a1 1 0 100-2h-3zm-3 4a1 1 0 100 2h.01a1 1 0 100-2H7zm3 0a1 1 0 100 2h3a1 1 0 100-2h-3z" clip-rule="evenodd" />
                            </svg>
                        </button>

                        <button
                            class="w-full p-3 bg-purple-600 text-white rounded-lg flex items-center justify-between hover:bg-purple-700 transition-colors"
                            on:click=on_grid_test_click.clone()
                        >
                            <span class="text-lg">Grid Test</span>
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                <path d="M9 2a1 1 0 000 2h2a1 1 0 100-2H9z" />
                                <path fill-rule="evenodd" d="M4 5a2 2 0 012-2 3 3 0 003 3h2a3 3 0 003-3 2 2 0 012 2v11a2 2 0 01-2 2H6a2 2 0 01-2-2V5zm3 4a1 1 0 000 2h.01a1 1 0 100-2H7zm3 0a1 1 0 000 2h3a1 1 0 100-2h-3zm-3 4a1 1 0 100 2h.01a1 1 0 100-2H7zm3 0a1 1 0 100 2h3a1 1 0 100-2h-3z" clip-rule="evenodd" />
                            </svg>
                        </button>
                    </div>

                    <div class="mt-6 flex justify-end">
                        <button
                            class="px-4 py-2 bg-gray-200 rounded-md hover:bg-gray-300 transition-colors"
                            on:click=on_cancel_click
                        >
                            Cancel
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
