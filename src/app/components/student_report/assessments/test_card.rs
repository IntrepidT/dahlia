use crate::app::components::dashboard::color_utils::ColorUtils;
use crate::app::components::dashboard::scores_ledger::ScoreUtils;
use crate::app::components::data_processing::TestDetail;
use crate::app::models::test::Test;
use leptos::prelude::*;
use leptos::prelude::*;

#[component]
pub fn TestCard(
    test_detail: TestDetail,
    tests_resource: Resource<Option<Vec<Test>>>,
    #[prop(default = false)] show_detailed_info: bool,
) -> impl IntoView {
    let test_name = test_detail.test_name.clone();
    let score = test_detail.score;
    let total_possible = test_detail.total_possible;
    let test_area = test_detail.test_area.clone();
    let date_administered = test_detail.date_administered;
    let test_id = test_detail.test_id;

    // Calculate percentage
    let percentage = (score as f32 / total_possible as f32 * 100.0).min(100.0);

    // Get benchmark-based styling
    let get_styling = move || {
        let test_data_result = tests_resource
            .get()
            .and_then(|result| result)
            .and_then(|tests| tests.iter().find(|t| t.test_id == test_id).cloned());

        let benchmark_categories = test_data_result
            .as_ref()
            .and_then(|t| t.benchmark_categories.as_ref());

        let badge_classes =
            ColorUtils::get_badge_classes_for_score(score, total_possible, benchmark_categories);
        let score_text_color =
            ColorUtils::get_score_text_color_for_score(score, total_possible, benchmark_categories);
        let progress_bar_color = ColorUtils::get_progress_bar_color_for_score(
            score,
            total_possible,
            benchmark_categories,
        );
        let benchmark_label =
            ScoreUtils::get_benchmark_label(score, total_possible, benchmark_categories);

        (
            badge_classes,
            score_text_color,
            progress_bar_color,
            benchmark_label,
        )
    };

    view! {
        <div class="bg-white rounded-lg border border-gray-200 shadow-sm hover:shadow-md transition-all duration-200 hover:border-blue-200">
            <div class="p-5">
                // Header Section
                <div class="flex items-start justify-between mb-4">
                    <div class="flex-1">
                        <h4 class="text-base font-semibold text-gray-900 mb-1">
                            {test_name}
                        </h4>
                        <div class="flex items-center gap-3 text-sm text-gray-600">
                            <span class="flex items-center gap-1">
                                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                                </svg>
                                {test_area}
                            </span>
                            <span class="flex items-center gap-1">
                                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                                </svg>
                                {date_administered.format("%b %d, %Y").to_string()}
                            </span>
                        </div>
                    </div>
                    // Performance Badge
                    <div class={format!("px-2.5 py-1 rounded-full text-xs font-medium {}", get_styling().0)}>
                        {get_styling().3}
                    </div>
                </div>

                // Score Section
                <div class="mb-4">
                    <div class="flex items-end gap-2 mb-2">
                        <span class={format!("text-2xl font-bold {}", get_styling().1)}>
                            {score}
                        </span>
                        <span class="text-lg text-gray-500 mb-1">
                            / {total_possible}
                        </span>
                        <span class="text-sm text-gray-500 mb-1">
                            ({format!("{:.1}%", percentage)})
                        </span>
                    </div>

                    // Progress Bar
                    <div class="w-full bg-gray-200 rounded-full h-2">
                        <div
                            class={format!("{} h-2 rounded-full transition-all duration-300", get_styling().2)}
                            style=format!("width: {}%", percentage)
                        ></div>
                    </div>
                </div>

                // Detailed Information (conditional)
                {if show_detailed_info {
                    view! {
                        <div class="border-t border-gray-100 pt-4 space-y-3">
                            // Performance Indicators
                            <div class="grid grid-cols-2 gap-4 text-sm">
                                <div>
                                    <span class="text-gray-600">Score Range:</span>
                                    <div class="font-medium text-gray-900">
                                        {
                                            if percentage >= 90.0 {
                                                "Excellent (90-100%)"
                                            } else if percentage >= 80.0 {
                                                "Good (80-89%)"
                                            } else if percentage >= 70.0 {
                                                "Satisfactory (70-79%)"
                                            } else if percentage >= 60.0 {
                                                "Needs Improvement (60-69%)"
                                            } else {
                                                "Below Standards (<60%)"
                                            }
                                        }
                                    </div>
                                </div>
                                <div>
                                    <span class="text-gray-600">Status:</span>
                                    <div class={
                                        if percentage >= 70.0 {
                                            "font-medium text-green-600"
                                        } else if percentage >= 60.0 {
                                            "font-medium text-yellow-600"
                                        } else {
                                            "font-medium text-red-600"
                                        }
                                    }>
                                        {
                                            if percentage >= 70.0 {
                                                "Passing"
                                            } else if percentage >= 60.0 {
                                                "Marginal"
                                            } else {
                                                "Needs Support"
                                            }
                                        }
                                    </div>
                                </div>
                            </div>

                            // Additional Actions
                            <div class="flex items-center gap-2 pt-2">
                                <button class="text-xs text-blue-600 hover:text-blue-700 hover:underline transition-colors">
                                    View Details
                                </button>
                                <span class="text-gray-300">|</span>
                                <button class="text-xs text-gray-600 hover:text-gray-700 hover:underline transition-colors">
                                    Compare Scores
                                </button>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }}
            </div>
        </div>
    }
}
