use super::sequence_node::SequenceNode;
use crate::app::components::assessment_page::shared::hooks::UseSequenceBuilder;
use crate::app::models::assessment_sequences::TestSequenceItem;
use crate::app::models::test::Test;
use leptos::prelude::*;

#[component]
pub fn SequenceFlow(
    sequence: Signal<Vec<TestSequenceItem>>,
    tests_resource: Resource<Result<Vec<Test>, ServerFnError>>,
    sequence_builder: UseSequenceBuilder,
    on_sequence_change: impl Fn(Vec<TestSequenceItem>) + 'static + Copy + Send,
) -> impl IntoView {
    view! {
        <div class="sequence-flow-container bg-white border-2 border-dashed border-gray-300 rounded-lg p-6 min-h-96">
            <div class="flex items-center justify-between mb-6">
                <h5 class="text-gray-700 font-medium">"Visual Sequence Flow with Vertical Variation Stacks"</h5>
                <div class="text-sm text-gray-500">
                    {move || {
                        let count = sequence.get().len();
                        format!("{} test{} in sequence", count, if count == 1 { "" } else { "s" })
                    }}
                </div>
            </div>

            {move || {
                let seq = sequence.get();
                if seq.is_empty() {
                    view! {
                        <div class="flex items-center justify-center h-48 text-gray-500 text-sm">
                            <div class="text-center">
                                <div class="mb-4 text-4xl">"🔄"</div>
                                <div class="text-lg font-medium mb-2">"No tests in sequence yet"</div>
                                <div class="text-gray-400">"Add tests above to build your assessment flow"</div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    let all_tests = tests_resource.get().map(|r| r.ok()).flatten().unwrap_or_default();

                    view! {
                        <div class="overflow-x-auto pb-4">
                            <div class="flex items-start gap-8 min-w-fit" style="min-width: max-content;">
                                {seq.iter().enumerate().map(|(index, seq_item)| {
                                    view! {
                                        <div class="flex items-start shrink-0">
                                            <SequenceNode
                                                seq_item=seq_item.clone()
                                                all_tests=all_tests.clone()
                                                index=index
                                                sequence_builder=sequence_builder.clone()
                                                current_sequence=sequence
                                                on_sequence_change=on_sequence_change
                                            />

                                            {if index < seq.len() - 1 {
                                                view! {
                                                    <div class="flex items-center justify-center h-20 mx-4 mt-16">
                                                        <div class="flex flex-col items-center">
                                                            <div class="bg-blue-100 text-blue-700 px-2 py-1 rounded-full text-xs font-medium mb-1">
                                                                "PASS"
                                                            </div>
                                                            <svg width="32" height="16" viewBox="0 0 32 16" fill="currentColor" class="text-blue-500">
                                                                <path d="M24 8l-4-4v3H4v2h16v3l4-4z"/>
                                                            </svg>
                                                        </div>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div class="flex items-center justify-center h-20 mx-4 mt-16">
                                                        <div class="bg-green-100 text-green-700 px-3 py-2 rounded-full text-sm font-medium">
                                                            "🎯 COMPLETE"
                                                        </div>
                                                    </div>
                                                }.into_any()
                                            }}
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>

                        <SequenceFlowLegend />
                    }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn SequenceFlowLegend() -> impl IntoView {
    view! {
        <div class="mt-8 grid grid-cols-1 lg:grid-cols-2 gap-6">
            <div class="bg-blue-50 rounded-lg border border-blue-200 p-4">
                <h6 class="text-sm font-semibold mb-3 text-blue-900">"Test Behaviors"</h6>
                <div class="space-y-2">
                    <LegendItem color="bg-green-500" label="Attainment (✓)" description="Requires specific score to pass" />
                    <LegendItem color="bg-blue-500" label="Node (→)" description="Simple progression test" />
                    <LegendItem color="bg-gray-500" label="Optional (?)" description="Can be skipped" />
                    <LegendItem color="bg-purple-500" label="Diagnostic (📊)" description="Assessment only" />
                </div>
            </div>

            <div class="bg-orange-50 rounded-lg border border-orange-200 p-4">
                <h6 class="text-sm font-semibold mb-3 text-orange-900">"Variation System"</h6>
                <div class="space-y-2">
                    <LegendItem color="bg-orange-400" label="Level 1" description="First remediation attempt" />
                    <LegendItem color="bg-orange-500" label="Level 2" description="Second remediation attempt" />
                    <LegendItem color="bg-orange-600" label="Level 3" description="Final remediation attempt" />
                </div>
                <div class="mt-3 p-2 bg-orange-100 rounded text-xs text-orange-800">
                    <span class="font-medium">"Flow:"</span>
                    " Student fails main → L1 → L2 → L3 → Teacher intervention"
                </div>
            </div>
        </div>
    }
}

#[component]
fn LegendItem(
    color: &'static str,
    label: &'static str,
    description: &'static str,
) -> impl IntoView {
    view! {
        <div class="flex items-center text-xs text-blue-800">
            <div class=format!("w-4 h-4 rounded-full {} mr-3", color)></div>
            <div class="flex-1">
                <span class="font-medium">{label}</span>
                <div class="text-blue-600">{description}</div>
            </div>
        </div>
    }
}
