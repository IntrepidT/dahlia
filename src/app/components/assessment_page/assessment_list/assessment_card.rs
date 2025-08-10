use crate::app::components::assessment_page::assessment_list::sequence_visualization::SequenceVisualization;
use crate::app::components::test_item::TestItem;
use crate::app::models::assessment::Assessment;
use crate::app::models::test::Test;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn AssessmentCard(
    assessment: Assessment,
    tests: Vec<Test>,
    on_edit: impl Fn(Assessment) + 'static + Copy,
    on_delete: impl Fn(Uuid) + 'static + Copy,
) -> impl IntoView {
    let assessment_clone = assessment.clone();
    let assessment_id = assessment.id;
    let assessment_name = assessment.name.clone();
    let assessment_subject = assessment.subject.clone();
    let assessment_grade = assessment.grade.clone();
    let (expanded, set_expanded) = signal(false);

    let uses_sequences = assessment.test_sequence.is_some()
        && !assessment.test_sequence.as_ref().unwrap().is_empty();

    // Combined style computation
    let expandable_style = move || {
        let max_height = if expanded.get() { "800px" } else { "0" };
        let opacity = if expanded.get() { "1" } else { "0" };
        format!("max-height: {}; opacity: {}", max_height, opacity)
    };

    view! {
        <div class="bg-white rounded-lg shadow-sm border border-gray-100 hover:shadow-md transition-shadow overflow-hidden">
            <button
                class="w-full text-left p-4 focus:outline-none"
                on:click=move |_| set_expanded.update(|val| *val = !*val)
            >
                <div class="flex justify-between items-center">
                    <div>
                        <h3 class="font-medium">{assessment_name.clone()}</h3>
                        <div class="flex items-center space-x-2 text-sm text-gray-500">
                            <span>"("{assessment_subject.as_ref().map(|s| s.to_string()).unwrap_or_else(|| "No Subject".to_string())}")"</span>
                            {if uses_sequences {
                                view! { <span class="bg-blue-100 text-blue-800 px-2 py-1 rounded text-xs">"Sequenced"</span> }.into_any()
                            } else {
                                view! { <span class="bg-gray-100 text-gray-600 px-2 py-1 rounded text-xs">"Legacy"</span> }.into_any()
                            }}
                        </div>
                    </div>
                    <div class="flex items-center space-x-2">
                        <div class="text-sm text-gray-500">
                            {assessment_grade.map(|g| format!("{:?}", g)).unwrap_or_else(|| "Any".to_string())}
                        </div>
                        <button
                            class="text-xs px-3 py-1 bg-blue-100 rounded-full text-blue-600 hover:bg-blue-200 transition-colors"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                on_edit(assessment_clone.clone());
                            }
                        >
                            "Edit"
                        </button>
                        <button
                            class="text-xs px-3 py-1 bg-red-50 rounded-full text-red-600 hover:bg-red-100 transition-colors"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                on_delete(assessment_id);
                            }
                        >
                            "Delete"
                        </button>
                    </div>
                </div>
            </button>

            // Expandable details section - FIXED: Combined styles
            <div
                class="border-t border-gray-100 overflow-hidden transition-all duration-300 ease-in-out"
                style=expandable_style
            >
                <div class="p-4">
                    <AssessmentDetails
                        assessment=assessment.clone()
                        tests=tests.clone()
                        uses_sequences=uses_sequences
                    />
                </div>
            </div>
        </div>
    }
}

#[component]
fn AssessmentDetails(
    assessment: Assessment,
    tests: Vec<Test>,
    uses_sequences: bool,
) -> impl IntoView {
    if uses_sequences {
        view! {
            <div>
                <h4 class="text-sm font-medium mb-3">"Test Sequence Flow"</h4>
                <SequenceVisualization
                    sequence={assessment.test_sequence.clone().unwrap_or_default()}
                    tests={tests}
                />
            </div>
        }
        .into_any()
    } else {
        view! {
            <div>
                <h4 class="text-sm font-medium mb-2">"Tests"</h4>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
                    {assessment.tests.iter().map(|test_id| {
                        if let Some(test) = tests.iter().find(|t| {
                            Uuid::parse_str(&t.test_id).unwrap_or_default() == *test_id
                        }) {
                            view! {
                                <TestItem
                                    test=test.clone()
                                    test_id=test.test_id.clone()
                                    test_name=test.name.clone()
                                />
                            }.into_any()
                        } else {
                            view! { <div>"Unknown Test"</div> }.into_any()
                        }
                    }).collect_view()}
                </div>
            </div>
        }
        .into_any()
    }
}
