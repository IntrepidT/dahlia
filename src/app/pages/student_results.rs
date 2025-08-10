use crate::app::components::dashboard::color_utils::ColorUtils;
use crate::app::components::dashboard::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::dashboard::scores_ledger::ScoreUtils;
use crate::app::components::data_processing::{
    AssessmentProgressChart, AssessmentRadarChart, AssessmentSummary, PerformanceDistributionChart,
    Progress, StudentResultsSummary, TestAreaPerformanceChart, TestDetail, TestScoresTimelineChart,
};
use crate::app::components::enhanced_login_form::{
    use_student_mapping_service, DeAnonymizedStudent, StudentMappingService,
};
use crate::app::components::header::Header;
use crate::app::components::student_report::assessments::progress_overview_tab::ProgressOverviewTab;
use crate::app::components::student_report::overview::{OverviewTab, SortOption, TimeFrame};
use crate::app::components::student_report::sequence_progress_bar::{
    CompactStripeProgress, StripeProgressBar,
};
use crate::app::components::student_report::sequence_web::SequenceWeb;
use crate::app::middleware::global_settings::use_settings;
use crate::app::models::test::Test;
use crate::app::server_functions::data_wrappers::get_student_results_server;
use crate::app::server_functions::tests::get_tests;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use std::collections::HashSet;
use uuid::Uuid;

#[component]
pub fn TestResultsPage() -> impl IntoView {
    //Get global settings for anonymization
    let (settings, _) = use_settings();
    let anonymization_enabled = move || settings.get().student_protections;

    //Get student mapping service
    let (student_mapping_service, _) = use_student_mapping_service();

    // Get student ID from URL parameters
    let params = use_params_map();
    let student_id = move || {
        params.with(|params| {
            params
                .get("student_id")
                .and_then(|id| id.parse::<i32>().ok())
                .unwrap_or(0)
        })
    };

    // Resource to fetch consolidated student results data
    let student_results_resource = Resource::new(
        move || student_id(),
        |id| async move {
            if id > 0 {
                get_student_results_server(id).await.ok()
            } else {
                None
            }
        },
    );

    // Resource to fetch test data for benchmark categories - FIXED
    let tests_resource = LocalResource::new(|| async {
        match get_tests().await {
            Ok(tests) => Some(tests),
            Err(e) => {
                log::error!("Failed to load tests: {}", e);
                None
            }
        }
    });

    //Create enhanced student data with de-anonymization
    let enhanced_student_data = Memo::new(move |_| {
        student_results_resource
            .get()
            .unwrap_or(None)
            .map(|results| {
                if anonymization_enabled() {
                    let de_anon = DeAnonymizedStudent::from_student_with_mapping(
                        &results.student,
                        student_mapping_service.get().as_ref(),
                    );
                    (results, Some(de_anon))
                } else {
                    (results, None)
                }
            })
    });

    // Signal to track which assessment is expanded
    let (expanded_assessment, set_expanded_assessment) = create_signal::<Option<String>>(None);
    let (filter_test_name, set_filter_test_name) = create_signal::<Option<String>>(None);
    let (show_filters, set_show_filters) = create_signal::<bool>(false);
    let (view_mode, set_view_mode) = create_signal::<String>("overview".to_string());

    view! {
        <Header />
        <div class="p-5 max-w-7xl mx-auto">
            // Student Information Section
            <Suspense fallback=move || view! { <div class="text-center p-4">"Loading student data..."</div> }>
                {move || enhanced_student_data.get().map(|(results, de_anon_opt)| {
                    let (display_name, display_id) = if let Some(de_anon) = &de_anon_opt {
                        (de_anon.display_name.clone(), de_anon.display_id.clone())
                    } else {
                        let name = format!(
                            "{} {}",
                            results.student.firstname.clone().unwrap_or_else(|| "Unknown".to_string()),
                            results.student.lastname.clone().unwrap_or_else(|| "Student".to_string())
                        );
                        (name, results.student.student_id.to_string())
                    };

                    view! {
                        <div class="bg-white rounded-lg shadow-md p-6 mb-6">
                            <h1 class="text-2xl font-bold mb-4">
                                "Test Results for " {display_name}
                            </h1>
                            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                                <div class="bg-gray-50 p-4 rounded">
                                    <h3 class="font-semibold text-gray-700">"Student ID"</h3>
                                    <p>{display_id}</p>
                                </div>
                                <div class="bg-gray-50 p-4 rounded">
                                    <h3 class="font-semibold text-gray-700">"Grade Level"</h3>
                                    <p>{results.student.current_grade_level.to_string()}</p>
                                </div>
                                <div class="bg-gray-50 p-4 rounded">
                                    <h3 class="font-semibold text-gray-700">"School Year"</h3>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                }).unwrap_or_else(|| view! {
                    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-6">
                        "Student not found or invalid ID"
                    </div>
                }.into_any())}
            </Suspense>

            //View mode toggle
            <div class="flex justify-center mb-6">
                <div class="bg-white rounded-xl shadow-lg p-2 border border-slate-200">
                    <div class="flex space-x-2">
                        <button
                            class=move || {
                                if view_mode.get() == "overview" {
                                    "px-6 py-2 bg-blue-500 text-white rounded-lg font-medium transition-all duration-200"
                                } else {
                                    "px-6 py-2 text-slate-600 hover:text-slate-800 rounded-lg font-medium transition-all duration-200"
                                }
                            }
                            on:click=move |_| set_view_mode("overview".to_string())
                        >
                            "Overview"
                        </button>
                        <button
                            class=move || {
                                if view_mode.get() == "sequence" {
                                    "px-6 py-2 bg-blue-500 text-white rounded-lg font-medium transition-all duration-200"
                                } else {
                                    "px-6 py-2 text-slate-600 hover:text-slate-800 rounded-lg font-medium transition-all duration-200"
                                }
                            }
                            on:click=move |_| set_view_mode("sequence".to_string())
                        >
                            "Assessment Progress"
                        </button>
                        <button
                            class=move || {
                                if view_mode.get() == "detailed" {
                                    "px-6 py-2 bg-blue-500 text-white rounded-lg font-medium transition-all duration-200"
                                } else {
                                    "px-6 py-2 text-slate-600 hover:text-slate-800 rounded-lg font-medium transition-all duration-200"
                                }
                            }
                            on:click=move |_| set_view_mode("detailed".to_string())
                        >
                            "Detailed View"
                        </button>
                    </div>
                </div>
            </div>

            // Overview Section
            <Suspense fallback=move || view! {
                <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6 animate-pulse">
                    <div class="h-6 bg-gray-200 rounded w-1/4 mb-6"></div>
                    <div class="space-y-3">
                        <div class="h-4 bg-gray-200 rounded w-full"></div>
                        <div class="h-4 bg-gray-200 rounded w-3/4"></div>
                        <div class="h-4 bg-gray-200 rounded w-5/6"></div>
                    </div>
                </div>
            }>
                {move || {
                    let results = enhanced_student_data.get().map(|(results, _)| results);
                    match results {
                        Some(data) => {
                            let test_history = data.test_history.clone();

                            if view_mode.get() == "overview" {
                                // Create signals for the overview controls
                                let (search_query, set_search_query) = signal(String::new());
                                let (selected_timeframe, set_selected_timeframe) = signal(TimeFrame::AllTime);
                                let (selected_sort, set_selected_sort) = signal(SortOption::DateDesc);

                                // Filter and sort logic
                                let filtered_and_sorted_tests = Memo::new(move |_| {
                                    let mut tests = test_history.clone();
                                    let query = search_query.get().to_lowercase();
                                    let timeframe = selected_timeframe.get();
                                    let sort = selected_sort.get();

                                    // Filter by search query
                                    if !query.is_empty() {
                                        tests = tests.into_iter()
                                            .filter(|test| {
                                                test.test_name.to_lowercase().contains(&query) ||
                                                test.performance_class.to_lowercase().contains(&query) ||
                                                test.evaluator.to_lowercase().contains(&query)
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
                                        tests = tests.into_iter()
                                            .filter(|test| test.date_administered >= cutoff_date)
                                            .collect();
                                    }

                                    // Sort
                                    match sort {
                                        SortOption::DateDesc => tests.sort_by(|a, b| b.date_administered.cmp(&a.date_administered)),
                                        SortOption::DateAsc => tests.sort_by(|a, b| a.date_administered.cmp(&b.date_administered)),
                                        SortOption::ScoreDesc => tests.sort_by(|a, b| {
                                            let a_percent = (a.score as f32 / a.total_possible as f32) * 100.0;
                                            let b_percent = (b.score as f32 / b.total_possible as f32) * 100.0;
                                            b_percent.partial_cmp(&a_percent).unwrap_or(std::cmp::Ordering::Equal)
                                        }),
                                        SortOption::ScoreAsc => tests.sort_by(|a, b| {
                                            let a_percent = (a.score as f32 / a.total_possible as f32) * 100.0;
                                            let b_percent = (b.score as f32 / b.total_possible as f32) * 100.0;
                                            a_percent.partial_cmp(&b_percent).unwrap_or(std::cmp::Ordering::Equal)
                                        }),
                                        SortOption::TestNameAsc => tests.sort_by(|a, b| a.test_name.cmp(&b.test_name)),
                                        SortOption::TestNameDesc => tests.sort_by(|a, b| b.test_name.cmp(&a.test_name)),
                                    }

                                    tests
                                });

                                view! {
                                    <div class="space-y-6">
                                        // Header section
                                        <div class="flex flex-col space-y-4 sm:flex-row sm:items-center sm:justify-between sm:space-y-0">
                                            <div>
                                                <h2 class="text-2xl font-semibold text-gray-900">"Recent Tests"</h2>
                                                <p class="mt-1 text-sm text-gray-600">
                                                    "Track test performance and progress over time"
                                                </p>
                                            </div>
                                        </div>

                                        // Controls section
                                        <div class="flex flex-col space-y-4 sm:flex-row sm:items-center sm:space-y-0 sm:space-x-4">
                                            <div class="flex-1 max-w-md">
                                                // Search bar
                                                <div class="relative">
                                                    <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                                        <svg
                                                            class="h-4 w-4 text-gray-400"
                                                            fill="none"
                                                            viewBox="0 0 24 24"
                                                            stroke="currentColor"
                                                        >
                                                            <path
                                                                stroke-linecap="round"
                                                                stroke-linejoin="round"
                                                                stroke-width="2"
                                                                d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                                                            />
                                                        </svg>
                                                    </div>
                                                    <input
                                                        type="text"
                                                        placeholder="Search tests, evaluators..."
                                                        class="block w-full pl-10 pr-3 py-2.5 border border-gray-200 rounded-lg text-sm placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200 bg-white"
                                                        prop:value=search_query
                                                        on:input=move |ev| {
                                                            set_search_query(event_target_value(&ev));
                                                        }
                                                    />
                                                </div>
                                            </div>
                                            <div class="flex space-x-3">
                                                // Time frame selector
                                                <div class="relative">
                                                    <select
                                                        class="appearance-none bg-white border border-gray-200 rounded-lg px-4 py-2.5 pr-8 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200 cursor-pointer"
                                                        on:change=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            let timeframe = match value.as_str() {
                                                                "7" => TimeFrame::LastWeek,
                                                                "30" => TimeFrame::LastMonth,
                                                                "90" => TimeFrame::Last3Months,
                                                                "365" => TimeFrame::LastYear,
                                                                _ => TimeFrame::AllTime,
                                                            };
                                                            set_selected_timeframe(timeframe);
                                                        }
                                                    >
                                                        <option value="all">"All time"</option>
                                                        <option value="7">"Last 7 days"</option>
                                                        <option value="30">"Last 30 days"</option>
                                                        <option value="90">"Last 90 days"</option>
                                                        <option value="365">"Last year"</option>
                                                    </select>
                                                    <div class="absolute inset-y-0 right-0 flex items-center pr-2 pointer-events-none">
                                                        <svg class="h-4 w-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                                                        </svg>
                                                    </div>
                                                </div>
                                                // Sort selector
                                                <div class="relative">
                                                    <select
                                                        class="appearance-none bg-white border border-gray-200 rounded-lg px-4 py-2.5 pr-8 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200 cursor-pointer"
                                                        on:change=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            let sort = match value.as_str() {
                                                                "date_asc" => SortOption::DateAsc,
                                                                "score_desc" => SortOption::ScoreDesc,
                                                                "score_asc" => SortOption::ScoreAsc,
                                                                "name_asc" => SortOption::TestNameAsc,
                                                                "name_desc" => SortOption::TestNameDesc,
                                                                _ => SortOption::DateDesc,
                                                            };
                                                            set_selected_sort(sort);
                                                        }
                                                    >
                                                        <option value="date_desc">"Newest first"</option>
                                                        <option value="date_asc">"Oldest first"</option>
                                                        <option value="score_desc">"Highest score"</option>
                                                        <option value="score_asc">"Lowest score"</option>
                                                        <option value="name_asc">"Test name A-Z"</option>
                                                        <option value="name_desc">"Test name Z-A"</option>
                                                    </select>
                                                    <div class="absolute inset-y-0 right-0 flex items-center pr-2 pointer-events-none">
                                                        <svg class="h-4 w-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                                                        </svg>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // Table section
                                        <div class="bg-white rounded-xl border border-gray-200 shadow-sm flex flex-col" style="height: 400px; min-height: 400px;">
                                            {move || {
                                                let tests = filtered_and_sorted_tests.get();
                                                let test_count = tests.len();

                                                if tests.is_empty() {
                                                    view! {
                                                        <>
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
                                                        </>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <>
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
                                                        </>
                                                    }.into_any()
                                                }
                                            }}
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }
                        },
                        None => view! { <div></div> }.into_any()
                    }
                }}
            </Suspense>

            // Compact Progress Cards Section - Show in detailed view only
            <Suspense fallback=move || view! { <div>"Loading compact progress..."</div> }>
                {move || {
                    let results = enhanced_student_data.get().map(|(results, _)| results);
                    match results {
                        Some(data) => {
                            let assessments = data.assessment_summaries.clone();

                            if view_mode.get() == "detailed" && !assessments.is_empty() {
                                view! {
                                    <div class="mb-6">
                                        <h2 class="text-xl font-bold text-slate-800 mb-4">"Quick Progress Summary"</h2>
                                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                            {assessments.iter().map(|assessment| {
                                                view! {
                                                    <CompactStripeProgress
                                                        assessment_name={assessment.assessment_name.clone()}
                                                        current_score={assessment.current_score}
                                                        total_possible={assessment.total_possible}
                                                        test_details={assessment.test_details.clone()}
                                                    />
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }
                        },
                        None => view! { <div></div> }.into_any()
                    }
                }}
            </Suspense>

            // Performance Summary Section - Show only in detailed view
            {move || {
                if view_mode.get() == "detailed" {
                    view! {
                        <div class="bg-white rounded-lg shadow-md p-6 mb-6">
                            <h2 class="text-xl font-bold mb-4">"Performance Summary"</h2>
                            <Suspense fallback=move || view! { <div>"Loading summary data..."</div> }>
                                {move || {
                                    let results = enhanced_student_data.get().map(|(results, _)| results);
                                    match results {
                                        Some(data) => {
                                            let assessments = data.assessment_summaries.clone();
                                            let assessments_clone = assessments.clone();
                                            if assessments.is_empty() {
                                                view! { <div class="text-gray-600">"No assessment data available"</div> }.into_any()
                                            } else {
                                                view! {
                                                    <div class="overflow-x-auto">
                                                        <table class="min-w-full bg-white">
                                                            <thead class="bg-gray-100">
                                                                <tr>
                                                                    <th class="py-2 px-4 text-left">"Assessment Name"</th>
                                                                    <th class="py-2 px-4 text-left">"Subject"</th>
                                                                    <th class="py-2 px-4 text-left">"Total Possible"</th>
                                                                    <th class="py-2 px-4 text-left">"Current Score"</th>
                                                                    <th class="py-2 px-4 text-left">"Grade Level"</th>
                                                                    <th class="py-2 px-4 text-left">"Performance"</th>
                                                                    <th class="py-2 px-4 text-left">"Status"</th>
                                                                    <th class="py-2 px-4 text-left">"Action"</th>
                                                                </tr>
                                                            </thead>
                                                            <tbody>
                                                                {assessments.into_iter().map(|assessment| {
                                                                    let assessment_id = assessment.assessment_id.clone();
                                                                    let assessment_id_for_button = assessment_id.clone();
                                                                    let assessment_id_for_details = assessment_id.clone();
                                                                    let assessment_id_for_closure = assessment_id.clone();
                                                                    let progress_clone = assessment.progress.clone();

                                                                    // Pre-clone all the values that will be used inside closures
                                                                    let test_details = assessment.test_details.clone();
                                                                    let distribution_data = assessment.distribution_data.clone();
                                                                    let assessment_rating = assessment.assessment_rating.clone();
                                                                    let assessment_current_score = assessment.current_score;
                                                                    let assessment_total_possible = assessment.total_possible;
                                                                    let test_details_len = assessment.test_details.len();

                                                                    view! {
                                                                        <>
                                                                            <tr class="border-t hover:bg-gray-50">
                                                                                <td class="py-3 px-4">{assessment.assessment_name}</td>
                                                                                <td class="py-3 px-4">{assessment.subject}</td>
                                                                                <td class="py-3 px-4">
                                                                                    {assessment.total_possible.map(|score| score.to_string()).unwrap_or_else(|| "N/A".to_string())}
                                                                                </td>
                                                                                <td class="py-3 px-4">{assessment.current_score}</td>
                                                                                <td class="py-3 px-4">{assessment.grade_level.unwrap_or_else(|| "Any".to_string())}</td>

                                                                                <td class="py-3 px-4">{assessment.assessment_rating}</td>
                                                                                <td class="py-3 px-4">
                                                                                    <span class=move || {
                                                                                        match progress_clone {
                                                                                            Progress::Completed => "px-2 py-1 bg-green-100 text-green-800 rounded-full text-xs",
                                                                                            Progress::Ongoing => "px-2 py-1 bg-yellow-100 text-yellow-800 rounded-full text-xs",
                                                                                            Progress::NotStarted => "px-2 py-1 bg-gray-100 text-gray-800 rounded-full text-xs",
                                                                                        }
                                                                                    }>
                                                                                        {format!("{}", assessment.progress)}
                                                                                    </span>
                                                                                </td>
                                                                                <td class="py-3 px-4">
                                                                                    <button
                                                                                        class="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600"
                                                                                        on:click=move |_| {
                                                                                            if expanded_assessment.get() == Some(assessment_id_for_button.clone()) {
                                                                                                set_expanded_assessment(None);
                                                                                            } else {
                                                                                                set_expanded_assessment(Some(assessment_id_for_button.clone()));
                                                                                            }
                                                                                        }
                                                                                    >
                                                                                        {move || {
                                                                                            if expanded_assessment.get() == Some(assessment_id_for_closure.clone()) {
                                                                                                "Hide Details"
                                                                                            } else {
                                                                                                "Show Details"
                                                                                            }
                                                                                        }}
                                                                                    </button>
                                                                                </td>
                                                                            </tr>
                                                                        </>
                                                                    }
                                                                }).collect::<Vec<_>>()}
                                                            </tbody>
                                                        </table>

                                                        {/* Assessment details display outside of the main table */}
                                                        {assessments_clone.into_iter().map(|assessment| {
                                                            let assessment_id = assessment.assessment_id.clone();
                                                            let assessment_name = assessment.assessment_name.clone();

                                                            // Pre-clone all the values that will be used inside closures
                                                            let test_details = assessment.test_details.clone();
                                                            let distribution_data = assessment.distribution_data.clone();
                                                            let assessment_rating = assessment.assessment_rating.clone();
                                                            let assessment_current_score = assessment.current_score;
                                                            let assessment_total_possible = assessment.total_possible;
                                                            let test_details_len = assessment.test_details.len();

                                                            view! {
                                                                {move || {
                                                                    if expanded_assessment.get() == Some(assessment_id.clone()) {
                                                                        let test_details_clone = test_details.clone();
                                                                        // Create a local clone of assessment_rating to avoid moving it
                                                                        let assessment_rating_clone = assessment_rating.clone();
                                                                        let rating = assessment_rating_clone.clone();

                                                                        view! {
                                                                            <div class="mt-4 mb-8 bg-gray-50 border rounded-lg p-4 shadow">
                                                                                <h3 class="font-semibold text-lg mb-2 text-blue-600">
                                                                                    {format!("{} Details", assessment_name.clone())}
                                                                                </h3>

                                                                                <div class="mb-4">
                                                                                    <h4 class="font-semibold mb-2">{"Subtests Performance"}</h4>
                                                                                    {if test_details_clone.is_empty() {
                                                                                        view! { <div><p class="text-gray-500">"No test data available for this assessment"</p></div> }.into_any()
                                                                                    } else {
                                                                                        view! {
                                                                                            <div class="overflow-x-auto">
                                                                                                <table class="min-w-full bg-white border">
                                                                                                    <thead class="bg-gray-100">
                                                                                                        <tr>
                                                                                                            <th class="py-2 px-3 text-left text-sm">"Test Name"</th>
                                                                                                            <th class="py-2 px-3 text-left text-sm">"Score"</th>
                                                                                                            <th class="py-2 px-3 text-left text-sm">"Total"</th>
                                                                                                            <th class="py-2 px-3 text-left text-sm">"Test Area"</th>
                                                                                                            <th class="py-2 px-3 text-left text-sm">"Taken"</th>
                                                                                                            <th class="py-2 px-3 text-left text-sm">"Performance"</th>
                                                                                                        </tr>
                                                                                                    </thead>
                                                                                                    <tbody>
                                                                                                        {test_details_clone.iter().map(|test| {
                                                                                                            // Get test data for benchmark categories
                                                                                                            let test_data = tests_resource.get()
                                                                                                                .and_then(|result| result)
                                                                                                                .and_then(|tests| tests.iter().find(|t| t.test_id == test.test_id).cloned());

                                                                                                            let benchmark_categories = test_data.as_ref().and_then(|t| t.benchmark_categories.as_ref());

                                                                                                            // Get benchmark-based colors for the detailed view too
                                                                                                            let badge_classes = ColorUtils::get_badge_classes_for_score(
                                                                                                                test.score,
                                                                                                                test.total_possible,
                                                                                                                benchmark_categories
                                                                                                            );
                                                                                                            let score_text_color = ColorUtils::get_score_text_color_for_score(
                                                                                                                test.score,
                                                                                                                test.total_possible,
                                                                                                                benchmark_categories
                                                                                                            );

                                                                                                            view! {
                                                                                                                <tr class="border-t">
                                                                                                                    <td class="py-2 px-3 text-sm">{test.test_name.clone()}</td>
                                                                                                                    <td class="py-2 px-3 text-sm">
                                                                                                                        <span class={score_text_color}>
                                                                                                                            {test.score}
                                                                                                                        </span>
                                                                                                                    </td>
                                                                                                                    <td class="py-2 px-3 text-sm">{test.total_possible}</td>
                                                                                                                    <td class="py-2 px-3 text-sm">{test.test_area.clone()}</td>
                                                                                                                    <td class="py-2 px-3 text-sm">{format!("{}", test.date_administered.format("%Y-%m-%d"))}</td>
                                                                                                                    <td class="py-2 px-3 text-sm">
                                                                                                                        <span class={format!("px-2 py-1 rounded-full text-xs {}", badge_classes)}>
                                                                                                                            {ScoreUtils::get_benchmark_label(
                                                                                                                                test.score,
                                                                                                                                test.total_possible,
                                                                                                                                benchmark_categories
                                                                                                                            )}
                                                                                                                        </span>
                                                                                                                    </td>
                                                                                                                </tr>
                                                                                                            }
                                                                                                        }).collect::<Vec<_>>()}
                                                                                                    </tbody>
                                                                                                </table>
                                                                                            </div>
                                                                                        }.into_any()
                                                                                    }}
                                                                                </div>

                                                                                // Replaced old charts with new Chart.js component
                                                                                <div class="mt-6">
                                                                                    <h4 class="font-semibold mb-4">"Assessment Performance Charts"</h4>
                                                                                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
                                                                                        // Performance Distribution Chart
                                                                                        <PerformanceDistributionChart
                                                                                            distribution_data={distribution_data.clone()}
                                                                                            chart_id={format!("expanded-dist-{}", assessment_id)}
                                                                                            title="Performance Categories".to_string()
                                                                                        />

                                                                                        // Overall assessment rating display
                                                                                        <div class="bg-white rounded-lg border p-6 flex flex-col items-center justify-center">
                                                                                            <h5 class="text-sm font-medium text-gray-500 mb-2">"Overall Assessment Rating"</h5>
                                                                                            <div class=move || {
                                                                                                // Use the cloned value to avoid moving
                                                                                                let color_class = if rating.contains("Above") || rating.contains("High") {
                                                                                                    "text-4xl font-bold text-green-600"
                                                                                                } else if rating.contains("Average") || rating.contains("On Track") {
                                                                                                    "text-4xl font-bold text-blue-600"
                                                                                                } else if rating.contains("Below") || rating.contains("Risk") {
                                                                                                    "text-4xl font-bold text-red-600"
                                                                                                } else {
                                                                                                    "text-4xl font-bold text-gray-600"
                                                                                                };
                                                                                                color_class
                                                                                            }>
                                                                                                {assessment_rating_clone}
                                                                                            </div>
                                                                                            <div class="mt-2 text-sm text-gray-600">
                                                                                                "Based on " {test_details_len} " completed tests"
                                                                                            </div>
                                                                                            <div class="mt-4 flex items-center w-full">
                                                                                                <div class="w-full bg-gray-200 rounded-full h-2.5">
                                                                                                    <div
                                                                                                        class=move || {
                                                                                                            let score_percent = if let Some(total) = assessment_total_possible {
                                                                                                                (assessment_current_score as f32 / total as f32 * 100.0) as i32
                                                                                                            } else {
                                                                                                                0
                                                                                                            };

                                                                                                            if score_percent >= 80 {
                                                                                                                "bg-green-600 h-2.5 rounded-full"
                                                                                                            } else if score_percent >= 60 {
                                                                                                                "bg-yellow-400 h-2.5 rounded-full"
                                                                                                            } else {
                                                                                                                "bg-red-600 h-2.5 rounded-full"
                                                                                                            }
                                                                                                        }
                                                                                                        style=move || {
                                                                                                            let percent = if let Some(total) = assessment_total_possible {
                                                                                                                let p = (assessment_current_score as f32 / total as f32 * 100.0) as i32;
                                                                                                                p.min(100)
                                                                                                            } else {
                                                                                                                0
                                                                                                            };
                                                                                                            format!("width: {}%", percent)
                                                                                                        }
                                                                                                    ></div>
                                                                                                </div>
                                                                                            </div>
                                                                                            <div class="mt-1 text-xs">
                                                                                                {assessment_current_score}
                                                                                                {assessment_total_possible.map(|t| format!(" / {}", t)).unwrap_or_else(|| String::new())}
                                                                                            </div>
                                                                                        </div>
                                                                                    </div>
                                                                                </div>
                                                                            </div>
                                                                        }.into_any()
                                                                    } else {
                                                                        view! { <div></div> }.into_any()
                                                                    }
                                                                }}
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                }.into_any()
                                            }
                                        },
                                        None => view! { <div class="text-gray-600">"No assessment data available"</div> }.into_any()
                                    }
                                }}
                            </Suspense>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>
    }
}

// Helper function to calculate overall distribution across all assessments
fn calculate_overall_distribution(assessments: &[AssessmentSummary]) -> Vec<(String, i32)> {
    let mut distribution_map: std::collections::HashMap<String, i32> =
        std::collections::HashMap::new();

    for assessment in assessments {
        for (category, count) in &assessment.distribution_data {
            *distribution_map.entry(category.clone()).or_insert(0) += count;
        }
    }

    distribution_map.into_iter().collect()
}
