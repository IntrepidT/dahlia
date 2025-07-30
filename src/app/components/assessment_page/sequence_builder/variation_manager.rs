use crate::app::components::assessment_page::shared::types::is_variation_test;
use crate::app::models::assessment_sequences::{TestSequenceItem, VariationLevel};
use crate::app::models::test::Test;
use leptos::*;
use uuid::Uuid;

#[component]
pub fn VariationManager(
    tests_resource: Resource<(), Result<Vec<Test>, ServerFnError>>,
    current_sequence: Signal<Vec<TestSequenceItem>>,
    variations: ReadSignal<Vec<VariationLevel>>,
    set_variations: WriteSignal<Vec<VariationLevel>>,
    main_test_id: Option<Uuid>,
) -> impl IntoView {
    let (editing_variation_index, set_editing_variation_index) =
        create_signal::<Option<usize>>(None);

    // Form fields for adding/editing variations
    let (var_test_id, set_var_test_id) = create_signal::<Option<Uuid>>(None);
    let (var_level, set_var_level) = create_signal(1);
    let (var_description, set_var_description) = create_signal(String::new());
    let (var_required_score, set_var_required_score) = create_signal(60);
    let (var_max_attempts, set_var_max_attempts) = create_signal(2);
    let (use_same_test, set_use_same_test) = create_signal(true);

    // Get available variation tests
    let get_available_variation_tests = move || -> Vec<Test> {
        let all_tests = tests_resource
            .get()
            .map(|r| r.ok())
            .flatten()
            .unwrap_or_default();
        /*let current_seq = current_sequence.get();
        let current_variations = variations.get();

        let used_test_ids: std::collections::HashSet<Uuid> = current_seq
            .iter()
            .flat_map(|item| {
                let mut ids = vec![item.test_id];
                if let Some(variations) = &item.variation_levels {
                    ids.extend(variations.iter().map(|v| v.test_id));
                }
                ids
            })
            .chain(current_variations.iter().map(|v| v.test_id))
            .collect();*/

        all_tests
            .into_iter()
            /*.filter(|test| {
                let test_uuid = Uuid::parse_str(&test.test_id).unwrap_or_default();
                !used_test_ids.contains(&test_uuid) && is_variation_test(test)
            })*/
            .collect()
    };

    let reset_variation_form = move || {
        set_var_test_id.set(None);
        set_var_level.set(1);
        set_var_description.set(String::new());
        set_var_required_score.set(60);
        set_var_max_attempts.set(2);
        set_editing_variation_index.set(None);
        set_use_same_test.set(true);
    };

    let save_variation = move |_| {
        let final_test_id = if use_same_test.get() {
            main_test_id.unwrap_or_else(|| var_test_id.get().unwrap_or_default())
        } else {
            var_test_id.get().unwrap_or_default()
        };

        if final_test_id != Uuid::default() {
            let new_variation = VariationLevel {
                level: var_level.get(),
                test_id: final_test_id,
                required_score: Some(var_required_score.get()),
                max_attempts: Some(var_max_attempts.get()),
                description: var_description.get(),
            };

            let mut current_variations = variations.get();

            if let Some(index) = editing_variation_index.get() {
                current_variations[index] = new_variation;
            } else {
                current_variations.push(new_variation);
            }

            current_variations.sort_by_key(|v| v.level);
            set_variations.set(current_variations);
            reset_variation_form();
        }
    };

    let edit_variation = move |index: usize| {
        let variation_list = variations.get();
        if let Some(variation) = variation_list.get(index) {
            let is_same_test = main_test_id
                .map(|main_id| main_id == variation.test_id)
                .unwrap_or(false);

            set_var_test_id.set(Some(variation.test_id));
            set_var_level.set(variation.level);
            set_var_description.set(variation.description.clone());
            set_var_required_score.set(variation.required_score.unwrap_or(60));
            set_var_max_attempts.set(variation.max_attempts.unwrap_or(2));
            set_editing_variation_index.set(Some(index));
            set_use_same_test.set(is_same_test);
        }
    };

    let remove_variation = move |index: usize| {
        let mut variation_list = variations.get(); // CRITICAL FIX: Use passed variations
        variation_list.remove(index);
        set_variations.set(variation_list); // CRITICAL FIX: Update passed variations
        reset_variation_form();
    };

    view! {
        <div class="bg-orange-50 border border-orange-200 rounded-lg p-4 mb-4">
            <div class="flex items-center justify-between mb-3">
                <h6 class="text-orange-800 font-medium">"Multi-Level Variation Tests (On Fail)"</h6>
            </div>
            <p class="text-xs text-orange-700 mb-4">
                "Students will progress through these variations vertically if they fail the main test. Maximum 3 levels."
            </p>

            <VariationStackDisplay
                variations=variations
                tests_resource=tests_resource
                main_test_id=main_test_id
                on_edit=edit_variation
                on_remove=remove_variation
            />

            // Add/Edit Variation Form
            <div class="bg-white rounded-lg border border-orange-200 p-4">
                <div class="text-sm font-medium text-orange-800 mb-3">
                    {move || if editing_variation_index.get().is_some() { "Edit Variation" } else { "Add New Variation" }}
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                    <div class="col-span-2 mb-4">
                        <div class="flex items-center space-x-2">
                            <label class="inline-flex items-center">
                                <input
                                    type="radio"
                                    name="test_selection_mode"
                                    class="form-radio h-4 w-4 text-orange-600"
                                    prop:checked={move || use_same_test.get()}
                                    on:change=move |_| set_use_same_test.set(true)
                                />
                                <span class="ml-2 text-sm text-orange-700">"Use same test as main (different difficult/attempts)"</span>
                            </label>
                            <label class="inline-flex items-center">
                                <input
                                    type="radio"
                                    name="test_selection_mode"
                                    class="form-radio h-4 w-4 text-orange-600"
                                    prop:checked={move || !use_same_test.get()}
                                    on:change=move |_| set_use_same_test.set(false)
                                />
                                <span class="ml-2 text-sm text-orange-700">"Use different test"</span>
                            </label>
                        </div>
                    </div>

                    <Show when=move || !use_same_test.get()>
                        <div>
                            <label class="block text-xs font-medium text-orange-700 mb-1">"Variation Test"</label>
                            <select
                                class="w-full px-3 py-2 border border-orange-300 rounded-md bg-white text-gray-900 focus:ring-2 focus:ring-orange-500 text-sm"
                                prop:value={move || var_test_id.get().map(|id| id.to_string()).unwrap_or_default()}
                                on:change=move |ev| {
                                    let value = event_target_value(&ev);
                                    if value.is_empty() {
                                        set_var_test_id.set(None);
                                    } else if let Ok(uuid) = Uuid::parse_str(&value) {
                                        set_var_test_id.set(Some(uuid));
                                    }
                                }
                            >
                                <option value="">"Select variation test"</option>
                                {move || {
                                    get_available_variation_tests().into_iter().map(|test| {
                                        view! {
                                            <option value=test.test_id.clone() class="text-gray-900">
                                                {test.name.clone()}
                                            </option>
                                        }
                                    }).collect_view()
                                }}
                            </select>
                        </div>
                    </Show>

                    <Show when=move || use_same_test.get()>
                        <div>
                            <label class="block text-xs font-medium text-orange-700 mb-1">"Using Main Test"</label>
                            <div class="w-full px-3 py-2 border border-orange-300 rounded-md bg-orange-50 text-orange-800 text-sm">
                                {move || {
                                    if let Some(main_id) = main_test_id {
                                        let all_tests = tests_resource
                                            .get()
                                            .map(|r| r.ok())
                                            .flatten()
                                            .unwrap_or_default();
                                        all_tests.iter()
                                            .find(|t| Uuid::parse_str(&t.test_id).unwrap_or_default() == main_id)
                                            .map(|t|format!("📋 {} (Main Test)", t.name))
                                            .unwrap_or_else(|| "Main Test".to_string())
                                    } else {
                                        "Main Test".to_string()
                                    }
                                }}
                            </div>
                        </div>
                    </Show>

                    <div>
                        <label class="block text-xs font-medium text-orange-700 mb-1">"Level (1-3)"</label>
                        <select
                            class="w-full px-3 py-2 border border-orange-300 rounded-md bg-white text-gray-900 focus:ring-2 focus:ring-orange-500 text-sm"
                            prop:value={move || var_level.get().to_string()}
                            on:change=move |ev| {
                                if let Ok(level) = event_target_value(&ev).parse::<i32>() {
                                    set_var_level.set(level);
                                }
                            }
                        >
                            <option value="1">"Level 1 (First try)"</option>
                            <option value="2">"Level 2 (Second try)"</option>
                            <option value="3">"Level 3 (Final try)"</option>
                        </select>
                    </div>

                    <div>
                        <label class="block text-xs font-medium text-orange-700 mb-1">"Description"</label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 border border-orange-300 rounded-md bg-white text-gray-900 focus:ring-2 focus:ring-orange-500 text-sm"
                            placeholder="e.g., Practice Mode, Guided Version"
                            prop:value={move || var_description.get()}
                            on:input=move |ev| set_var_description.set(event_target_value(&ev))
                        />
                    </div>

                    <div>
                        <label class="block text-xs font-medium text-orange-700 mb-1">"Required Score %"</label>
                        <input
                            type="number"
                            min="0"
                            max="100"
                            class="w-full px-3 py-2 border border-orange-300 rounded-md bg-white text-gray-900 focus:ring-2 focus:ring-orange-500 text-sm"
                            prop:value={move || var_required_score.get()}
                            on:input=move |ev| {
                                if let Ok(score) = event_target_value(&ev).parse::<i32>() {
                                    set_var_required_score.set(score);
                                }
                            }
                        />
                    </div>
                </div>

                <div class="flex justify-between items-center">
                    <div class="text-xs text-orange-600">
                        {move || {
                            let count = variations.get().len(); // CRITICAL FIX: Use passed variations
                            if count >= 3 && editing_variation_index.get().is_none() {
                                "Maximum 3 variations reached".to_string()
                            } else {
                                format!("{}/3 variations used", count)
                            }
                        }}
                    </div>

                    <div class="flex space-x-2">
                        <Show when=move || editing_variation_index.get().is_some()>
                            <button
                                type="button"
                                class="px-3 py-1 text-xs bg-gray-100 text-gray-700 rounded hover:bg-gray-200 transition-colors"
                                on:click=move |_| reset_variation_form()
                            >
                                "Cancel"
                            </button>
                        </Show>

                        <button
                            type="button"
                            class="px-3 py-1 text-xs bg-orange-600 text-white rounded hover:bg-orange-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                            on:click=save_variation
                            disabled=move || {
                                let has_test = use_same_test.get() || var_test_id.get().is_some();
                                let has_description = !var_description.get().trim().is_empty();
                                let count = variations.get().len();
                                let is_editing = editing_variation_index.get().is_some();

                                !has_test || !has_description || (count >= 3 && !is_editing)
                            }
                        >
                            {move || if editing_variation_index.get().is_some() { "Update" } else { "Add Variation" }}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn VariationStackDisplay(
    variations: ReadSignal<Vec<VariationLevel>>,
    tests_resource: Resource<(), Result<Vec<Test>, ServerFnError>>,
    main_test_id: Option<Uuid>,
    on_edit: impl Fn(usize) + 'static + Copy,
    on_remove: impl Fn(usize) + 'static + Copy,
) -> impl IntoView {
    view! {
        <div class="mb-4">
            <div class="text-sm font-medium text-orange-800 mb-2">"Current Variation Stack"</div>
            <div class="bg-white rounded-lg border border-orange-200 p-3">
                {move || {
                    variations.with(|variation_list| {
                        if variation_list.is_empty() {
                            view! {
                                <div class="text-center text-gray-500 py-4">
                                    <div class="mb-2">"📝"</div>
                                    <div class="text-xs">"No variations added yet"</div>
                                    <div class="text-xs text-gray-400">"Add up to 3 levels below"</div>
                                </div>
                            }.into_view()
                        } else {
                            let all_tests = tests_resource.get().map(|r| r.ok()).flatten().unwrap_or_default();

                            view! {
                                <div class="flex flex-col items-center space-y-3">
                                    {variation_list.iter().enumerate().map(|(index, variation)| {
                                        let test = all_tests.iter().find(|t| {
                                            Uuid::parse_str(&t.test_id).unwrap_or_default() == variation.test_id
                                        });
                                        let test_name = test.map(|t| t.name.clone()).unwrap_or_else(|| "Unknown".to_string());

                                        // Check if this variation uses the same test as main
                                        let is_same_test = main_test_id
                                            .map(|main_id| main_id == variation.test_id)
                                            .unwrap_or(false);

                                        let level_color = match variation.level {
                                            1 => "#fb923c",
                                            2 => "#f97316",
                                            3 => "#ea580c",
                                            _ => "#f97316"
                                        };

                                        view! {
                                            <div class="relative group">
                                                <div
                                                    class="w-16 h-16 rounded-full flex flex-col items-center justify-center text-white font-bold text-xs shadow-md border-2"
                                                    style=format!("background-color: {}; border-color: {}", level_color, level_color)
                                                >
                                                    <div class="text-sm">"L"{variation.level}</div>
                                                    <div class="text-xs">{variation.required_score.unwrap_or(60)}"%"</div>
                                                </div>

                                                <div class="absolute -bottom-12 left-1/2 transform -translate-x-1/2 text-center">
                                                    <div class="text-xs font-medium text-orange-700 whitespace-nowrap max-w-20 truncate">
                                                        {if is_same_test {
                                                            format!("📋 {}", test_name)
                                                        } else {
                                                            format!("🔄 {}", test_name)
                                                        }}
                                                    </div>
                                                    <div class="text-xs text-orange-600">{variation.description.clone()}</div>
                                                </div>

                                                <div class="absolute -top-2 -right-2 opacity-0 group-hover:opacity-100 transition-opacity flex space-x-1">
                                                    <button
                                                        type="button"
                                                        class="w-5 h-5 bg-blue-500 text-white rounded-full text-xs hover:bg-blue-600 transition-colors"
                                                        on:click=move |_| on_edit(index)
                                                        title="Edit"
                                                    >
                                                        "✎"
                                                    </button>
                                                    <button
                                                        type="button"
                                                        class="w-5 h-5 bg-red-500 text-white rounded-full text-xs hover:bg-red-600 transition-colors"
                                                        on:click=move |_| on_remove(index)
                                                        title="Remove"
                                                    >
                                                        "×"
                                                    </button>
                                                </div>
                                            </div>

                                            {if index < variation_list.len() - 1 {
                                                view! {
                                                    <div class="text-orange-400">
                                                        <svg width="16" height="20" viewBox="0 0 16 20" fill="currentColor">
                                                            <path d="M8 18l-4-4h3V2h2v12h3l-4 4z"/>
                                                        </svg>
                                                    </div>
                                                }.into_view()
                                            } else {
                                                view! {}.into_view()
                                            }}
                                        }
                                    }).collect_view()}

                                    {if !variation_list.is_empty() {
                                        view! {
                                            <div class="mt-2 px-3 py-1 bg-gray-100 rounded-full">
                                                <div class="text-xs text-gray-600">"Teacher Intervention"</div>
                                            </div>
                                        }.into_view()
                                    } else {
                                        view! {}.into_view()
                                    }}
                                </div>
                            }.into_view()
                        }
                    })
                }}
            </div>
        </div>
    }
}
