use crate::app::components::data_processing::{AssessmentSummary, TestDetail};
use crate::app::components::student_report::assessments::test_card::TestCard;
use crate::app::models::test::Test;
use leptos::prelude::*;
use leptos::prelude::*;

#[component]
pub fn ExpandedTestList(
    assessment: AssessmentSummary,
    tests_resource: Resource<Option<Vec<Test>>>,
    #[prop(default = false)] show_detailed_test_info: bool,
) -> impl IntoView {
    let test_details = assessment.test_details.clone();
    let assessment_name = assessment.assessment_name.clone();

    // Sort tests by date (most recent first)
    let sorted_tests = {
        let mut tests = test_details;
        tests.sort_by(|a, b| b.date_administered.cmp(&a.date_administered));
        tests
    };

    view! {
        <div class="mt-4 bg-gray-50 rounded-lg border border-gray-200 p-4">
            <div class="flex items-center justify-between mb-4">
                <h4 class="text-lg font-semibold text-gray-900">
                    {format!("{} - Test Details", assessment_name)}
                </h4>
                <div class="flex items-center gap-2 text-sm text-gray-600">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                    </svg>
                    <span>{sorted_tests.len()} {if sorted_tests.len() == 1 { "test" } else { "tests" }}</span>
                </div>
            </div>

            {if sorted_tests.is_empty() {
                view! {
                    <div class="text-center py-8">
                        <div class="w-12 h-12 mx-auto mb-4 bg-gray-200 rounded-full flex items-center justify-center">
                            <svg class="w-6 h-6 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                            </svg>
                        </div>
                        <h3 class="text-lg font-medium text-gray-900 mb-1">No tests completed</h3>
                        <p class="text-gray-500">Tests for this assessment will appear here once completed.</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="space-y-3">
                        // Summary Stats
                        <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-4 p-4 bg-white rounded-lg border border-gray-200">
                            <div class="text-center">
                                <div class="text-2xl font-bold text-gray-900">
                                    {
                                        let total_score: i32 = sorted_tests.iter().map(|t| t.score).sum();
                                        total_score
                                    }
                                </div>
                                <div class="text-sm text-gray-600">Total Points</div>
                            </div>
                            <div class="text-center">
                                <div class="text-2xl font-bold text-gray-900">
                                    {
                                        let avg_percentage = if !sorted_tests.is_empty() {
                                            sorted_tests.iter()
                                                .map(|t| (t.score as f32 / t.total_possible as f32 * 100.0))
                                                .sum::<f32>() / sorted_tests.len() as f32
                                        } else {
                                            0.0
                                        };
                                        format!("{:.1}%", avg_percentage)
                                    }
                                </div>
                                <div class="text-sm text-gray-600">Average Score</div>
                            </div>
                            <div class="text-center">
                                <div class="text-2xl font-bold text-gray-900">
                                    {
                                        if let Some(latest_test) = sorted_tests.first() {
                                            latest_test.date_administered.format("%m/%d").to_string()
                                        } else {
                                            "N/A".to_string()
                                        }
                                    }
                                </div>
                                <div class="text-sm text-gray-600">Latest Test</div>
                            </div>
                        </div>

                        // Test Cards Grid
                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                            {sorted_tests.clone().into_iter().map(|test| {
                                view! {
                                    <TestCard
                                        test_detail=test
                                        tests_resource=tests_resource
                                        show_detailed_info=show_detailed_test_info
                                    />
                                }
                            }).collect::<Vec<_>>()}
                        </div>

                        // Performance Insights
                        <div class="mt-6 p-4 bg-blue-50 rounded-lg border border-blue-200">
                            <h5 class="font-medium text-blue-900 mb-2 flex items-center gap-2">
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                                Performance Insights
                            </h5>
                            <div class="text-sm text-blue-800">
                                {
                                    let test_count = sorted_tests.len();
                                    let high_scores = sorted_tests.iter()
                                        .filter(|t| (t.score as f32 / t.total_possible as f32) >= 0.8)
                                        .count();
                                    let low_scores = sorted_tests.iter()
                                        .filter(|t| (t.score as f32 / t.total_possible as f32) < 0.6)
                                        .count();

                                    if high_scores > test_count / 2 {
                                        "Strong performance across most tests. Consider advancing to more challenging material."
                                    } else if low_scores > test_count / 2 {
                                        "Several tests show room for improvement. Consider reviewing foundational concepts."
                                    } else {
                                        "Mixed performance indicates good progress with some areas needing attention."
                                    }
                                }
                            </div>
                        </div>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
