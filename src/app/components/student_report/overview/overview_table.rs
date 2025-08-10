use crate::app::components::dashboard::color_utils::ColorUtils;
use crate::app::components::dashboard::scores_ledger::ScoreUtils;
use crate::app::components::data_processing::TestHistoryEntry;
use crate::app::components::overview::sort_selector::SortOption;
use crate::app::components::overview::time_frame_selector::TimeFrame;
use crate::app::models::test::Test;
use crate::app::server_functions::tests::get_tests;
use chrono::{DateTime, NaiveDate, Utc};
use leptos::prelude::*;
use leptos::prelude::*;

#[component]
pub fn OverviewTable(
    test_history: Vec<TestHistoryEntry>,
    #[prop(into)] search_query: ReadSignal<String>,
    #[prop(into)] selected_timeframe: ReadSignal<TimeFrame>,
    #[prop(into)] selected_sort: ReadSignal<SortOption>,
) -> impl IntoView {
    // Add resource to fetch test data for benchmark categories
    let tests_resource = LocalResource::new(move || async move {
        match get_tests().await {
            Ok(tests) => Some(tests),
            Err(e) => {
                log::error!("Failed to load tests: {}", e);
                None
            }
        }
    });

    let filtered_and_sorted_tests = Memo::new(move |_| {
        let mut tests = test_history.clone();
        let query = search_query.get().to_lowercase();
        let timeframe = selected_timeframe.get();
        let sort = selected_sort.get();

        // Filter by search query
        if !query.is_empty() {
            tests = tests
                .into_iter()
                .filter(|test| {
                    test.test_name.to_lowercase().contains(&query)
                        || test.performance_class.to_lowercase().contains(&query)
                        || test.evaluator.to_lowercase().contains(&query)
                })
                .collect();
        }

        // Filter by timeframe
        if timeframe != TimeFrame::AllTime {
            let days_back = match timeframe {
                TimeFrame::LastWeek => 7,
                TimeFrame::LastMonth => 30,
                TimeFrame::Last3Months => 90,
                TimeFrame::LastYear => 365,
                TimeFrame::AllTime => unreachable!(),
            };

            let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days_back);
            tests = tests
                .into_iter()
                .filter(|test| test.date_administered >= cutoff_date)
                .collect();
        }

        // Sort
        match sort {
            SortOption::DateDesc => {
                tests.sort_by(|a, b| b.date_administered.cmp(&a.date_administered))
            }
            SortOption::DateAsc => {
                tests.sort_by(|a, b| a.date_administered.cmp(&b.date_administered))
            }
            SortOption::ScoreDesc => tests.sort_by(|a, b| {
                let a_percent = (a.score as f32 / a.total_possible as f32) * 100.0;
                let b_percent = (b.score as f32 / b.total_possible as f32) * 100.0;
                b_percent
                    .partial_cmp(&a_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            SortOption::ScoreAsc => tests.sort_by(|a, b| {
                let a_percent = (a.score as f32 / a.total_possible as f32) * 100.0;
                let b_percent = (b.score as f32 / b.total_possible as f32) * 100.0;
                a_percent
                    .partial_cmp(&b_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            SortOption::TestNameAsc => tests.sort_by(|a, b| a.test_name.cmp(&b.test_name)),
            SortOption::TestNameDesc => tests.sort_by(|a, b| b.test_name.cmp(&a.test_name)),
        }

        tests
    });

    view! {
        <div class="bg-white rounded-xl border border-gray-200 shadow-sm flex flex-col" style="height: 400px; min-height: 400px;">
            {move || {
                let tests = filtered_and_sorted_tests.get();
                let test_count = tests.len();

                if tests.is_empty() {
                    view! {
                        <div class="flex-1 flex items-center justify-center p-12">
                            <div class="text-center">
                                <div class="w-12 h-12 mx-auto mb-4 bg-gray-100 rounded-full flex items-center justify-center">
                                    <svg class="w-6 h-6 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                    </svg>
                                </div>
                                <h3 class="text-lg font-medium text-gray-900 mb-1">"No tests found"</h3>
                                <p class="text-gray-500 text-sm">"Try adjusting your search or time frame filters."</p>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div>
                            // Header with test count - fixed at top
                            <div class="flex-shrink-0 px-6 py-3 bg-gray-50 border-b border-gray-200 rounded-t-xl">
                                <div class="flex items-center justify-between">
                                    <span class="text-sm font-medium text-gray-700">
                                        {format!("{} test{} found", test_count, if test_count == 1 { "" } else { "s" })}
                                    </span>
                                </div>
                            </div>

                            // Scrollable table content
                            <div class="flex-1 overflow-auto">
                                <div class="overflow-x-auto h-full">
                                    <table class="min-w-full divide-y divide-gray-200">
                                        <thead class="bg-gray-50 sticky top-0 z-10">
                                            <tr>
                                                <th scope="col" class="px-6 py-4 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                                    "Test"
                                                </th>
                                                <th scope="col" class="px-6 py-4 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                                    "Score"
                                                </th>
                                                <th scope="col" class="px-6 py-4 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                                    "Performance"
                                                </th>
                                                <th scope="col" class="px-6 py-4 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                                    "Evaluator"
                                                </th>
                                                <th scope="col" class="px-6 py-4 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                                    "Date"
                                                </th>
                                            </tr>
                                        </thead>
                                        <tbody class="bg-white divide-y divide-gray-200">
                                            {tests.into_iter().map(|test| {
                                                let score_percentage = (test.score as f32 / test.total_possible as f32) * 100.0;
                                                let evaluator_name = test.evaluator.clone();

                                                // Get test data for benchmark categories using test_id for accurate matching
                                                let test_data = tests_resource.get()
                                                    .and_then(|result| result)
                                                    .and_then(|tests| tests.iter().find(|t| t.test_id == test.test_id).cloned());

                                                let benchmark_categories = test_data.as_ref().and_then(|t| t.benchmark_categories.as_ref());

                                                // Get benchmark-based colors using ColorUtils
                                                let badge_classes = ColorUtils::get_badge_classes_for_score(
                                                    test.score,
                                                    test.total_possible,
                                                    benchmark_categories
                                                );
                                                let benchmark_label = ScoreUtils::get_benchmark_label(
                                                    test.score,
                                                    test.total_possible,
                                                    benchmark_categories
                                                );
                                                let score_text_color = ColorUtils::get_score_text_color_for_score(
                                                    test.score,
                                                    test.total_possible,
                                                    benchmark_categories
                                                );
                                                let progress_bar_color = ColorUtils::get_progress_bar_color_for_score(
                                                    test.score,
                                                    test.total_possible,
                                                    benchmark_categories
                                                );

                                                view! {
                                                    <tr class="hover:bg-gray-50 transition-colors duration-150">
                                                        <td class="px-6 py-4 whitespace-nowrap">
                                                            <div class="text-sm font-medium text-gray-900">
                                                                {test.test_name}
                                                            </div>
                                                        </td>
                                                        <td class="px-6 py-4 whitespace-nowrap">
                                                            <div class="flex items-center space-x-3">
                                                                <div class="flex-shrink-0">
                                                                    <span class={score_text_color}>
                                                                        {test.score}
                                                                    </span>
                                                                    <span class="text-sm text-gray-400 ml-1">
                                                                        "/" {test.total_possible}
                                                                    </span>
                                                                </div>
                                                                <div class="flex-1 min-w-0">
                                                                    <div class="w-16 bg-gray-200 rounded-full h-1.5">
                                                                        <div
                                                                            class={progress_bar_color}
                                                                            style=format!("width: {}%", score_percentage.min(100.0))
                                                                        ></div>
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        </td>
                                                        <td class="px-6 py-4 whitespace-nowrap">
                                                            <span class={format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}", badge_classes)}>
                                                                {benchmark_label}
                                                            </span>
                                                        </td>
                                                        <td class="px-6 py-4 whitespace-nowrap">
                                                            <div class="text-sm text-gray-900 font-medium">
                                                                {if evaluator_name.is_empty() {
                                                                    "Not specified".to_string()
                                                                } else {
                                                                    evaluator_name
                                                                }}
                                                            </div>
                                                        </td>
                                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                                            {format!("{}", test.date_administered.format("%b %d, %Y"))}
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
