use crate::app::components::assessment_page::shared::types::get_behavior_display_props;
use crate::app::models::assessment_sequences::{SequenceBehavior, TestSequenceItem};
use crate::app::models::test::Test;
use leptos::prelude::*;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn SequenceVisualization(sequence: Vec<TestSequenceItem>, tests: Vec<Test>) -> impl IntoView {
    let sequence = signal(sequence).0;
    let tests = signal(tests).0;

    view! {
        <div class="relative">
            <div class="flex flex-wrap gap-4 items-center">
                {move || {
                    let seq = sequence.get();
                    let all_tests = tests.get();

                    seq.iter().enumerate().map(|(index, item)| {
                        let test = all_tests.iter().find(|t| {
                            Uuid::parse_str(&t.test_id).unwrap_or_default() == item.test_id
                        });

                        let (bg_color, icon, border_color, _) = get_behavior_display_props(&item.sequence_behavior);

                        view! {
                            <div class="flex items-center">
                                <div class=format!("relative p-3 rounded-lg border-2 {} {} min-w-32",
                                    get_bg_class(&item.sequence_behavior),
                                    get_border_class(&item.sequence_behavior))>
                                    <div class="flex items-center space-x-2">
                                        <span class="text-lg">{icon}</span>
                                        <div>
                                            <div class="text-xs font-medium">
                                                {test.map(|t| t.name.clone()).unwrap_or_else(|| "Unknown".to_string())}
                                            </div>
                                            <div class="text-xs text-gray-500">
                                                {format!("{:?}", item.sequence_behavior)}
                                            </div>
                                        </div>
                                    </div>
                                    <div class="absolute -top-2 -left-2 w-6 h-6 bg-[#2E3A59] text-white rounded-full flex items-center justify-center text-xs font-bold">
                                        {item.sequence_order}
                                    </div>
                                </div>

                                // Connection arrow
                                {if index < seq.len() - 1 {
                                    view! {
                                        <div class="mx-2 text-gray-400">
                                            <svg width="24" height="16" viewBox="0 0 24 16" fill="currentColor">
                                                <path d="M16 8l-4-4v3H0v2h12v3l4-4z"/>
                                            </svg>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <div></div> }.into_any()
                                }}
                            </div>
                        }
                    }).collect_view()
                }}
            </div>
        </div>
    }
}

fn get_bg_class(behavior: &SequenceBehavior) -> &'static str {
    match behavior {
        SequenceBehavior::Node => "bg-blue-50",
        SequenceBehavior::Attainment => "bg-green-50",
        SequenceBehavior::Optional => "bg-gray-50",
        SequenceBehavior::Diagnostic => "bg-purple-50",
        SequenceBehavior::Remediation => "bg-orange-50",
        SequenceBehavior::Branching => "bg-yellow-50",
    }
}

fn get_border_class(behavior: &SequenceBehavior) -> &'static str {
    match behavior {
        SequenceBehavior::Node => "border-blue-200",
        SequenceBehavior::Attainment => "border-green-200",
        SequenceBehavior::Optional => "border-gray-200",
        SequenceBehavior::Diagnostic => "border-purple-200",
        SequenceBehavior::Remediation => "border-orange-200",
        SequenceBehavior::Branching => "border-yellow-200",
    }
}
