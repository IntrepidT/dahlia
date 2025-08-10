use crate::app::components::{ShowTestModal, ToastMessage};
use crate::app::models::{Test, TestType};
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::hooks::use_navigate;
use leptos_router::hooks::*;
use std::cell::RefCell;
use std::sync::Arc;

const DISPLAY_TEST_STYLE: &str = "group block w-full overflow-hidden border border-gray-200 rounded-lg bg-white shadow-sm hover:shadow-md transition-all duration-300";
const DISPLAY_TEST_EDIT_STYLE: &str = "group block w-full overflow-hidden rounded-lg bg-[#F44336] bg-opacity-40 hover:scale-105 hover:-translate-y-1 transition-all duration-300";

const IMG_SRC: &str = "/assets/math.png";

const CAPTION_STYLE: &str = "text-lg font-medium text-[#2E3A59]";
const INFO_STYLE: &str = "text-sm text-gray-600";
const SCORE_STYLE: &str = "text-md font-semibold text-[#2E3A59]";

#[component]
pub fn MathTestDisplay(
    test: Arc<Test>,
    test_resource: Resource<Result<Vec<Test>, ServerFnError>>,
    set_if_show_toast: WriteSignal<bool>,
    set_toast_message: WriteSignal<ToastMessage>,
    editing_mode: ReadSignal<bool>,
    on_delete: Option<Callback<String>>,
    show_delete_mode: ReadSignal<bool>,
    show_variations: Option<bool>,
    all_tests: Option<ReadSignal<Vec<Test>>>,
) -> impl IntoView {
    let edit_test = test.clone();
    let (show_options_modal, set_show_options_modal) = signal(false);
    let (show_variations_panel, set_show_variations_panel) = signal(false);

    // Determine if this is a variation
    let is_variation = test.name.contains(" - ")
        && (test.name.to_lowercase().contains("remediation")
            || test.name.to_lowercase().contains("advanced")
            || test.name.to_lowercase().contains("easy")
            || test.name.to_lowercase().contains("hard")
            || test.comments.to_lowercase().contains("variation:"));

    // Extract base name and variation type
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

    // Find related variations if all_tests is provided
    let related_variations = Memo::new({
        let test_clone = test.clone();
        let base_name_clone = base_name.clone();
        let test_id_clone = test.test_id.clone();

        move |_| {
            if let Some(all_tests_signal) = all_tests {
                let all = all_tests_signal.get();
                let current_base = if is_variation {
                    base_name_clone.clone()
                } else {
                    test_clone.name.clone()
                };

                all.into_iter()
                    .filter(|t| {
                        let t_base = if t.name.contains(" - ") {
                            t.name.split(" - ").next().unwrap_or(&t.name).to_string()
                        } else {
                            t.name.clone()
                        };
                        t_base == current_base && t.test_id != test_id_clone
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        }
    });

    // Handle showing selection modal instead of direct navigation
    let on_show_info = move |_| {
        if editing_mode() {
            // In editing mode, navigate directly to test builder
            let test_id = edit_test.test_id.clone();
            let navigate = use_navigate();
            navigate(&format!("/testbuilder/{}", test_id), Default::default());
        } else {
            // Show options modal in normal mode
            set_show_options_modal.set(true);
        }
    };

    let styling = move || {
        if editing_mode() {
            DISPLAY_TEST_EDIT_STYLE
        } else {
            DISPLAY_TEST_STYLE
        }
    };

    // Clone test for school_year_display closure
    let sy_test = test.clone();
    let school_year_display = move || match &sy_test.school_year {
        Some(year) if !year.is_empty() => year.clone(),
        _ => "Not specified".to_string(),
    };

    // Clone test for grade_level_display closure
    let gl_test = test.clone();
    let grade_level_display = move || match &gl_test.grade_level {
        Some(grade) => format!("{:?}", grade),
        None => "Not specified".to_string(),
    };

    // Clone test for benchmark_info closure
    let bm_test = test.clone();
    let benchmark_info = move || {
        match &bm_test.benchmark_categories {
            Some(categories) if !categories.is_empty() => {
                let category = &categories[0]; // Display first category info
                format!(
                    "Range: {} - {} ({})",
                    category.min, category.max, &category.label
                )
            }
            _ => "No benchmark data".to_string(),
        }
    };

    // Clone test for comments display
    let comments_test = test.clone();
    let content_test = test.clone();
    let has_comments = move || !comments_test.comments.is_empty();
    let comments_content = move || content_test.comments.clone();

    // Create score value for display
    let score_value = test.score.to_string();
    let test_name = test.name.clone();

    // Clone test for delete functionality
    let delete_test = test.clone();

    // Create separate variables for each navigation type
    let realtime_test = test.clone();
    let individual_test = test.clone();
    let grid_test = test.clone();
    let set_modal = set_show_options_modal.clone();

    let on_realtime_click = move |_| {
        let test_id = realtime_test.test_id.clone();
        let navigate = use_navigate();
        navigate(&format!("/test-session/{}", test_id), Default::default());
        set_modal.set(false);
    };

    let on_individual_click = move |_| {
        let test_id = individual_test.test_id.clone();
        let navigate = use_navigate();
        navigate(&format!("/flashcardset/{}", test_id), Default::default());
        set_modal.set(false);
    };

    let on_grid_test_click = move |_| {
        let test_id = grid_test.test_id.clone();
        let navigate = use_navigate();
        navigate(&format!("/gridtest/{}", test_id), Default::default());
        set_modal.set(false);
    };

    let on_cancel_click = move |_| {
        set_show_options_modal.set(false);
    };

    let on_show_variations = move |_| {
        set_show_variations_panel.set(!show_variations_panel());
    };

    view! {
        <div class="z-auto relative h-full">
            <button
                on:click=on_show_info
                class="w-full text-left"
            >
                <div class=styling>
                    <div class="flex items-center p-4">
                        <div class="w-12 h-12 flex-shrink-0">
                            <img src=IMG_SRC class="w-full h-full object-cover" />
                        </div>
                        <div class="ml-4 flex-grow">
                            <div class="flex items-center space-x-2">
                                <p class=CAPTION_STYLE>
                                    {if is_variation { base_name.clone() } else { test_name.clone() }}
                                </p>
                                {if is_variation {
                                    view! {
                                        <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                                            {variation_type.clone()}
                                        </span>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }}
                            </div>
                            <div class="mt-2 space-y-1">
                                <p class=SCORE_STYLE>
                                    "Total Score: " {score_value}
                                </p>
                                <p class=INFO_STYLE>
                                    "School Year: " {school_year_display}
                                </p>
                                <p class=INFO_STYLE>
                                    "Grade Level: " {grade_level_display}
                                </p>
                                {move || {
                                    if has_comments() {
                                        view! {
                                            <div>
                                                <p class=INFO_STYLE>
                                                    "Notes: " {comments_content()}
                                                </p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }
                                }}
                                <p class="text-xs text-gray-500 italic mt-1">
                                    {benchmark_info}
                                </p>

                                // Show variations count if applicable
                                {move || {
                                    let variations = related_variations.get();
                                    if !variations.is_empty() && show_variations.unwrap_or(false) {
                                        view! {
                                            <div class="flex items-center space-x-2 mt-2">
                                                <button
                                                    class="text-xs bg-gray-100 hover:bg-gray-200 text-gray-700 px-2 py-1 rounded transition-colors"
                                                    on:click=move |e| {
                                                        e.stop_propagation();
                                                        on_show_variations(e);
                                                    }
                                                >
                                                    {format!("{} variation(s)", variations.len())}
                                                </button>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }
                                }}
                            </div>
                        </div>
                        <div class="flex-shrink-0 ml-2">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-gray-400" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd" />
                            </svg>
                        </div>
                    </div>
                </div>
            </button>

            // Variations panel (expandable)
            {move || {
                if show_variations_panel() && show_variations.unwrap_or(false) {
                    let variations = related_variations.get();
                    view! {
                        <div class="bg-gray-50 border-t border-gray-200 px-4 py-3">
                            <h4 class="text-sm font-medium text-gray-700 mb-2">Related Tests:</h4>
                            <div class="space-y-2">
                                <For
                                    each=move || variations.clone()
                                    key=|variation| variation.test_id.clone()
                                    children=move |variation: Test| {
                                        let var_clone = variation.clone();
                                        let is_var = variation.name.contains(" - ");
                                        let display_name = if is_var {
                                            variation.name.split(" - ").nth(1).unwrap_or("Variation").to_string()
                                        } else {
                                            "Base Test".to_string()
                                        };

                                        view! {
                                            <div class="flex items-center justify-between bg-white rounded px-3 py-2 border border-gray-200">
                                                <div class="flex items-center space-x-2">
                                                    <span class="text-sm font-medium text-gray-900">
                                                        {display_name}
                                                    </span>
                                                    <span class="text-xs text-gray-500">
                                                        "({variation.score} pts)"
                                                    </span>
                                                </div>
                                                <div class="flex space-x-2">
                                                    <button
                                                        class="text-xs bg-blue-100 hover:bg-blue-200 text-blue-800 px-2 py-1 rounded transition-colors"
                                                        on:click=move |e| {
                                                            e.stop_propagation();
                                                            let test_id = var_clone.test_id.clone();
                                                            let navigate = use_navigate();
                                                            navigate(&format!("/testbuilder/{}", test_id), Default::default());
                                                        }
                                                    >
                                                        "Edit"
                                                    </button>
                                                    <button
                                                        class="text-xs bg-green-100 hover:bg-green-200 text-green-800 px-2 py-1 rounded transition-colors"
                                                        on:click=move |e| {
                                                            e.stop_propagation();
                                                            // Navigate to test taking interface
                                                            let test_id = variation.test_id.clone();
                                                            let navigate = use_navigate();
                                                            navigate(&format!("/test-session/{}", test_id), Default::default());
                                                        }
                                                    >
                                                        "Use"
                                                    </button>
                                                </div>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                            <div class="mt-3 pt-2 border-t border-gray-200">
                                <button
                                    class="text-xs bg-blue-600 hover:bg-blue-700 text-white px-3 py-1 rounded transition-colors"
                                    on:click=move |e| {
                                        e.stop_propagation();
                                        let navigate = use_navigate();
                                        navigate("/test-variations", Default::default());
                                    }
                                >
                                    "Manage All Variations"
                                </button>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}

            // Test Options Modal (existing code)
            {move || {
                if show_options_modal() {
                    view! {
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
                                    }.into_any()
                                } else {
                                    view! { <div></div> }.into_any()
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
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}

            // Delete button (existing code)
            {move || {
                if show_delete_mode() && on_delete.is_some() {
                    let test_id = delete_test.test_id.clone();

                    let delete_action = move |_| {
                        if let Some(delete_fn) = on_delete.clone() {
                            delete_fn.run(test_id.clone());
                        }
                    };

                    view! {
                        <div class="absolute top-2 right-2 z-10">
                            <button
                                class="bg-red-600 text-white p-2 rounded-full shadow-md hover:bg-red-700 transition-colors"
                                on:click=delete_action
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
                                </svg>
                            </button>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>
    }
}
