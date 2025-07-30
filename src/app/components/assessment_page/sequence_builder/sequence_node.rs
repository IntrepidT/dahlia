use crate::app::components::assessment_page::shared::{
    hooks::UseSequenceBuilder, types::get_behavior_display_props,
};
use crate::app::models::assessment_sequences::{
    SequenceBehavior, TestSequenceItem, VariationLevel,
};
use crate::app::models::test::Test;
use leptos::*;
use uuid::Uuid;

#[component]
pub fn SequenceNode(
    seq_item: TestSequenceItem,
    all_tests: Vec<Test>,
    index: usize,
    sequence_builder: UseSequenceBuilder,
    current_sequence: Signal<Vec<TestSequenceItem>>,
    on_sequence_change: impl Fn(Vec<TestSequenceItem>) + 'static + Copy,
) -> impl IntoView {
    let item_test_id = seq_item.test_id;
    let has_variations = seq_item
        .variation_levels
        .as_ref()
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    let variations = seq_item.variation_levels.clone().unwrap_or_default();

    let (node_color, icon, border_color, behavior_name) =
        get_behavior_display_props(&seq_item.sequence_behavior);

    let test = all_tests
        .iter()
        .find(|t| Uuid::parse_str(&t.test_id).unwrap_or_default() == item_test_id);
    let test_name = test
        .map(|t| t.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let short_name = if test_name.len() > 16 {
        format!("{}...", &test_name[0..13])
    } else {
        test_name.clone()
    };

    let dragging_item = sequence_builder.state.with(|s| s.dragging_item);
    let (show_details, set_show_details) = create_signal(false);

    let handle_drag_start = move |ev: leptos::ev::DragEvent| {
        ev.stop_propagation();
        if let Some(dt) = ev.data_transfer() {
            let _ = dt.set_data("text/plain", &index.to_string());
            let _ = dt.set_effect_allowed("move");
        }
        sequence_builder
            .set_state
            .update(|s| s.dragging_item = Some(index));
    };

    let handle_drop = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
        ev.stop_propagation();

        if let Some(dt) = ev.data_transfer() {
            if let Ok(data) = dt.get_data("text/plain") {
                if let Ok(source_index) = data.parse::<usize>() {
                    if source_index != index {
                        let new_sequence = sequence_builder.reorder_sequence.call((
                            source_index,
                            index,
                            current_sequence.get(),
                        ));
                        on_sequence_change(new_sequence);
                    }
                }
            }
        }
        sequence_builder
            .set_state
            .update(|s| s.dragging_item = None);
    };

    let remove_test = move |_| {
        let new_sequence = sequence_builder
            .remove_from_sequence
            .call((item_test_id, current_sequence.get()));
        on_sequence_change(new_sequence);
    };

    view! {
        <div class="flex flex-col items-center w-48 min-h-96">
            // Main Test Node
            <div
                class="sequence-node relative group cursor-move transition-all duration-200 hover:scale-105 z-10"
                class:opacity-50={move || dragging_item == Some(index)}
                draggable="true"
                on:dragstart=handle_drag_start
                on:dragover=move |ev: leptos::ev::DragEvent| {
                    ev.prevent_default();
                    ev.stop_propagation();
                }
                on:drop=handle_drop
                on:dragend=move |_| {
                    sequence_builder.set_state.update(|s| s.dragging_item = None);
                }
            >
                <div
                    class="w-20 h-20 rounded-full flex flex-col items-center justify-center text-white font-bold shadow-xl hover:shadow-2xl transition-shadow duration-200 relative"
                    style=format!("background: linear-gradient(135deg, {}, {}); border: 3px solid {}", node_color, node_color, border_color)
                >
                    <div class="text-lg mb-1">{icon}</div>
                    <div class="text-xs font-bold bg-black bg-opacity-20 rounded-full w-5 h-5 flex items-center justify-center">
                        {seq_item.sequence_order}
                    </div>
                </div>

                // Action buttons
                <div class="absolute -top-2 -right-2 opacity-0 group-hover:opacity-100 transition-opacity flex space-x-1 z-20">
                    <button
                        type="button"
                        class="w-6 h-6 bg-blue-500 text-white rounded-full text-xs hover:bg-blue-600 transition-colors shadow-md"
                        on:click=move |ev| {
                            ev.stop_propagation();
                            set_show_details.update(|val| *val = !*val);
                        }
                        title="View details"
                    >
                        "ⓘ"
                    </button>
                    <button
                        type="button"
                        class="w-6 h-6 bg-red-500 text-white rounded-full text-xs hover:bg-red-600 transition-colors shadow-md"
                        on:click=remove_test
                        title="Remove test"
                    >
                        "×"
                    </button>
                </div>
            </div>

            // Node Info
            <div class="mt-4 text-center w-full px-2">
                <div class="text-sm font-semibold text-gray-800 truncate mb-1" title={test_name.clone()}>
                    {short_name}
                </div>
                <div class="text-xs text-gray-600 bg-gray-100 rounded-full px-2 py-1 mb-1">
                    {behavior_name}
                </div>
                {if seq_item.sequence_behavior == SequenceBehavior::Attainment {
                    view! {
                        <div class="text-xs text-green-700 font-medium bg-green-100 rounded-full px-2 py-1 mb-1">
                            "Requires "{seq_item.required_score.unwrap_or(70)}"%"
                        </div>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }}
                {if has_variations {
                    view! {
                        <div class="text-xs text-orange-600 font-bold bg-orange-100 rounded-full px-2 py-1">
                            {format!("{} Variation{}", variations.len(), if variations.len() == 1 { "" } else { "s" })}
                        </div>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }}
            </div>

            // Variation Stack
            {if has_variations {
                view! {
                    <div class="mt-6 flex flex-col items-center">
                        <div class="bg-red-100 text-red-700 px-3 py-1 rounded-full text-xs font-bold border border-red-200 mb-3">
                            "ON FAIL ↓"
                        </div>
                        <VariationStack variations=variations all_tests=all_tests item_test_id=item_test_id />
                    </div>
                }.into_view()
            } else {
                view! { <div class="h-12"></div> }.into_view()
            }}

            // Details Panel
            <Show when=move || show_details.get()>
                <div class="mt-8 bg-white border border-gray-200 rounded-lg shadow-xl p-4 w-full max-w-sm z-30 relative">
                    <h6 class="font-semibold text-gray-800 mb-3 text-center">"Test Details"</h6>
                    <div class="space-y-3 text-sm">
                        <div class="bg-gray-50 p-2 rounded">
                            <div class="font-medium text-gray-700">"Name:"</div>
                            <div class="text-gray-900">{test_name.clone()}</div>
                        </div>
                        <div class="grid grid-cols-2 gap-2 text-xs">
                            <div class="bg-gray-50 p-2 rounded">
                                <div class="font-medium text-gray-700">"Behavior:"</div>
                                <div class="text-gray-900">{behavior_name}</div>
                            </div>
                            <div class="bg-gray-50 p-2 rounded">
                                <div class="font-medium text-gray-700">"Order:"</div>
                                <div class="text-gray-900">{seq_item.sequence_order}</div>
                            </div>
                        </div>
                    </div>
                    <div class="mt-4 text-center">
                        <button
                            type="button"
                            class="text-xs bg-gray-100 text-gray-700 px-3 py-1 rounded hover:bg-gray-200 transition-colors"
                            on:click=move |_| set_show_details.set(false)
                        >
                            "Close Details"
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn VariationStack(
    variations: Vec<VariationLevel>,
    all_tests: Vec<Test>,
    item_test_id: Uuid,
) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center space-y-6 w-full">
            {variations.iter().enumerate().map(|(var_index, variation)| {
                let var_test = all_tests.iter().find(|t| {
                    Uuid::parse_str(&t.test_id).unwrap_or_default() == variation.test_id
                });
                let var_name = var_test.map(|t| t.name.clone()).unwrap_or_else(|| "Unknown".to_string());
                let is_same_test = variation.test_id == item_test_id;
                let display_icon = if is_same_test { "📋" } else { "🔄" };
                let var_short_name = if is_same_test {
                    "Same Test".to_string()
                } else if var_name.len() > 14 {
                    format!("{}...", &var_name[0..11])
                } else {
                    var_name.clone()
                };

                let (var_color, var_border) = match variation.level {
                    1 => ("#fb923c", "#ea580c"),
                    2 => ("#f97316", "#c2410c"),
                    3 => ("#ea580c", "#9a3412"),
                    _ => ("#f97316", "#c2410c")
                };

                view! {
                    <div class="relative group w-full flex flex-col items-center">
                        <div
                            class="w-16 h-16 rounded-full flex flex-col items-center justify-center text-white font-bold shadow-lg hover:shadow-xl transition-all duration-200 hover:scale-105 relative"
                            style=format!("background: linear-gradient(135deg, {}, {}); border: 2px solid {}", var_color, var_border, var_border)
                            title=format!("Level {} Variation: {} ({}% required)", variation.level, var_name, variation.required_score.unwrap_or(60))
                        >
                            <div class="text-sm font-bold">"L"{variation.level}</div>
                            <div class="text-xs bg-black bg-opacity-20 rounded px-1">
                                {variation.required_score.unwrap_or(60)}"%"
                            </div>
                        </div>

                        <div class="mt-3 text-center w-full px-2">
                            <div class="text-xs font-semibold text-orange-800 truncate mb-1" title={var_name.clone()}>
                                {var_short_name}
                            </div>
                            <div class="text-xs text-orange-600 bg-orange-100 rounded-full px-2 py-1 mb-1">
                                {if is_same_test {
                                    format!("{} Retry L{}", display_icon, variation.level)
                                } else {
                                    format!("{} {}", display_icon, variation.description.clone())
                                }}
                            </div>
                            <div class="text-xs text-orange-700 font-medium">
                                "Level "{variation.level}" • "{variation.max_attempts.unwrap_or(2)}" attempts"
                            </div>
                        </div>

                        {if var_index < variations.len() - 1 {
                            view! {
                                <div class="flex flex-col items-center mt-4">
                                    <div class="bg-red-100 text-red-600 px-2 py-1 rounded-full text-xs font-medium border border-red-200 mb-2">
                                        "STILL FAIL ↓"
                                    </div>
                                </div>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }}
                    </div>
                }
            }).collect_view()}

            <div class="mt-6 flex flex-col items-center">
                <div class="bg-purple-100 text-purple-700 px-4 py-2 rounded-lg text-xs font-bold border border-purple-200 mb-2 text-center">
                    "🚨 TEACHER INTERVENTION"
                </div>
                <div class="text-xs text-gray-600 text-center max-w-40">
                    "All remediation attempts exhausted"
                </div>
            </div>
        </div>
    }
}
