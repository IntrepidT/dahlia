use super::variation_manager::VariationManager;
use crate::app::components::assessment_page::shared::{
    hooks::UseSequenceBuilder,
    types::{is_variation_test, SequenceBuilderState},
};
use crate::app::models::assessment_sequences::{
    SequenceBehavior, TestSequenceItem, VariationLevel,
};
use crate::app::models::test::Test;
use leptos::*;
use strum::IntoEnumIterator;
use uuid::Uuid;

#[component]
pub fn TestAddPanel(
    tests_resource: Resource<(), Result<Vec<Test>, ServerFnError>>,
    sequence_builder: UseSequenceBuilder,
    current_sequence: Signal<Vec<TestSequenceItem>>,
    on_sequence_change: impl Fn(Vec<TestSequenceItem>) + 'static + Copy,
) -> impl IntoView {
    let (selected_test_for_sequence, set_selected_test_for_sequence) =
        create_signal::<Option<Uuid>>(None);
    let (sequence_behavior, set_sequence_behavior) = create_signal(SequenceBehavior::Node);
    let (required_score, set_required_score) = create_signal::<Option<i32>>(None);
    let (show_variations_panel, set_show_variations_panel) = create_signal(false);

    // CRITICAL FIX: Add variations state at panel level to capture them
    let (current_variations, set_current_variations) = create_signal::<Vec<VariationLevel>>(vec![]);

    let add_test_to_sequence = move |_| {
        if let Some(test_id) = selected_test_for_sequence.get() {
            let order = sequence_builder.state.get().sequence_counter;

            // CRITICAL FIX: Create the test item with the captured variations
            let new_item = match sequence_behavior.get() {
                SequenceBehavior::Attainment => {
                    let mut item = TestSequenceItem::new_attainment(
                        test_id,
                        order,
                        required_score.get().unwrap_or(70),
                        None,
                        None,
                    );
                    // CRITICAL FIX: Add the variations that were built up in the panel
                    let variations = current_variations.get();
                    if !variations.is_empty() {
                        item.variation_levels = Some(variations);
                    }
                    item
                }
                SequenceBehavior::Node => TestSequenceItem::new_node(test_id, order),
                SequenceBehavior::Optional => TestSequenceItem::new_optional(test_id, order),
                SequenceBehavior::Diagnostic => TestSequenceItem::new_diagnostic(test_id, order),
                SequenceBehavior::Remediation => {
                    TestSequenceItem::new_remediation(test_id, order, vec![])
                }
                SequenceBehavior::Branching => {
                    TestSequenceItem::new_branching(test_id, order, vec![])
                }
            };

            let mut current_seq = current_sequence.get();
            current_seq.push(new_item);
            current_seq.sort_by_key(|item| item.sequence_order);
            on_sequence_change(current_seq);

            // Update the sequence counter
            sequence_builder
                .set_state
                .update(|s| s.sequence_counter = order + 1);

            // Reset form
            set_selected_test_for_sequence.set(None);
            set_sequence_behavior.set(SequenceBehavior::Node);
            set_required_score.set(None);
            set_show_variations_panel.set(false);
            set_current_variations.set(vec![]); // CRITICAL FIX: Reset variations
        }
    };

    // Get available tests (not already used in sequence)
    let get_available_tests = move || -> Vec<Test> {
        let all_tests = tests_resource
            .get()
            .map(|r| r.ok())
            .flatten()
            .unwrap_or_default();
        let current_seq = current_sequence.get();
        let used_test_ids: std::collections::HashSet<Uuid> =
            current_seq.iter().map(|item| item.test_id).collect();

        all_tests
            .into_iter()
            .filter(|test| {
                let test_uuid = Uuid::parse_str(&test.test_id).unwrap_or_default();
                !used_test_ids.contains(&test_uuid) && !is_variation_test(test)
            })
            .collect()
    };

    view! {
        <div class="add-test-panel bg-gray-50 border border-gray-200 rounded-lg p-4 mb-6">
            <h5 class="text-gray-700 font-medium mb-3">"Add Test to Sequence"</h5>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
                <div>
                    <label class="block text-sm font-medium text-gray-600 mb-1">"Test"</label>
                    <select
                        class="w-full px-3 py-2 border border-gray-300 rounded-md bg-white text-gray-900 focus:ring-2 focus:ring-blue-500"
                        prop:value={move || selected_test_for_sequence.get().map(|id| id.to_string()).unwrap_or_default()}
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            if value.is_empty() {
                                set_selected_test_for_sequence.set(None);
                            } else if let Ok(uuid) = Uuid::parse_str(&value) {
                                set_selected_test_for_sequence.set(Some(uuid));
                            }
                        }
                    >
                        <option value="">"Select a test"</option>
                        {move || {
                            get_available_tests().into_iter().map(|test| {
                                view! {
                                    <option value=test.test_id.clone() class="text-gray-900">
                                        {test.name.clone()}
                                    </option>
                                }
                            }).collect_view()
                        }}
                    </select>
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-600 mb-1">"Behavior"</label>
                    <select
                        class="w-full px-3 py-2 border border-gray-300 rounded-md bg-white text-gray-900 focus:ring-2 focus:ring-blue-500"
                        prop:value={move || format!("{:?}", sequence_behavior.get())}
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            match value.as_str() {
                                "Node" => {
                                    set_sequence_behavior.set(SequenceBehavior::Node);
                                    set_show_variations_panel.set(false);
                                },
                                "Attainment" => {
                                    set_sequence_behavior.set(SequenceBehavior::Attainment);
                                    set_show_variations_panel.set(true);
                                },
                                "Optional" => {
                                    set_sequence_behavior.set(SequenceBehavior::Optional);
                                    set_show_variations_panel.set(false);
                                },
                                "Diagnostic" => {
                                    set_sequence_behavior.set(SequenceBehavior::Diagnostic);
                                    set_show_variations_panel.set(false);
                                },
                                "Remediation" => {
                                    set_sequence_behavior.set(SequenceBehavior::Remediation);
                                    set_show_variations_panel.set(false);
                                },
                                "Branching" => {
                                    set_sequence_behavior.set(SequenceBehavior::Branching);
                                    set_show_variations_panel.set(false);
                                },
                                _ => {}
                            }
                        }
                    >
                        <option value="" class="text-gray-900">"Select Node Behavior"</option>
                        {SequenceBehavior::iter().map(|behavior| {
                            view! {
                                <option value=format!("{:?}", behavior) class="text-gray-900">
                                    {format!("{:?}", behavior)}
                                </option>
                            }
                        }).collect::<Vec<_>>()}
                    </select>
                </div>

                <Show when=move || matches!(sequence_behavior.get(), SequenceBehavior::Attainment)>
                    <div>
                        <label class="block text-sm font-medium text-gray-600 mb-1">"Required Score"</label>
                        <input
                            type="number"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md bg-white text-gray-900 focus:ring-2 focus:ring-blue-500"
                            placeholder="70"
                            prop:value={move || required_score.get().unwrap_or(70)}
                            on:input=move |ev| {
                                if let Ok(score) = event_target_value(&ev).parse::<i32>() {
                                    set_required_score.set(Some(score));
                                }
                            }
                        />
                    </div>
                </Show>
            </div>

            <Show when=move || show_variations_panel.get() && matches!(sequence_behavior.get(), SequenceBehavior::Attainment)>
                <VariationManager
                    tests_resource=tests_resource
                    current_sequence=current_sequence
                    variations=current_variations
                    set_variations=set_current_variations
                />
            </Show>

            <div class="flex justify-end">
                <button
                    type="button"
                    class="bg-blue-600 text-white px-6 py-2 rounded-md hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                    on:click=add_test_to_sequence
                    disabled=move || selected_test_for_sequence.get().is_none()
                >
                    "Add to Sequence"
                </button>
            </div>
        </div>
    }
}
