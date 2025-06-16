use crate::app::components::dashboard::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
#[cfg(feature = "hydrate")]
use crate::app::components::data_charts::{
    render_overall_progress, render_score_distribution, render_test_distribution, render_test_plot,
};
use crate::app::components::data_processing::{
    AssessmentSummary, Progress, StudentResultsSummary, TestDetail,
};
use crate::app::components::header::Header;
use crate::app::server_functions::data_wrappers::get_student_results_server;
use leptos::*;
use leptos_router::use_params_map;
use std::collections::HashSet;
use uuid::Uuid;

#[component]
pub fn TestResultsPage() -> impl IntoView {
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
    let student_results_resource = create_resource(
        move || student_id(),
        |id| async move {
            if id > 0 {
                get_student_results_server(id).await.ok()
            } else {
                None
            }
        },
    );

    // Signal to track which assessment is expanded
    let (expanded_assessment, set_expanded_assessment) = create_signal::<Option<String>>(None);
    let (filter_test_name, set_filter_test_name) = create_signal::<Option<String>>(None);
    let (show_filters, set_show_filters) = create_signal::<bool>(false);

    view! {
        <Header />
        <div class="p-6 max-w-6xl mx-auto">
            // Student Information Section
            <Suspense fallback=move || view! { <div class="text-center p-4">"Loading student data..."</div> }>
                {move || student_results_resource.get().map(|results_opt| {
                    match results_opt {
                        Some(results) => view! {
                            <div class="bg-white rounded-lg shadow-md p-6 mb-6">
                                <h1 class="text-2xl font-bold mb-4">
                                    "Test Results for " {results.student.firstname.clone()} " " {results.student.lastname.clone()}
                                </h1>
                                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                                    <div class="bg-gray-50 p-4 rounded">
                                        <h3 class="font-semibold text-gray-700">"Student ID"</h3>
                                        <p>{results.student.student_id}</p>
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
                        },
                        None => view! { <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-6">
                            "Student not found or invalid ID"
                        </div> }
                    }
                })}
            </Suspense>

            // Performance Summary Section
            <div class="bg-white rounded-lg shadow-md p-6 mb-6">
                <h2 class="text-xl font-bold mb-4">"Performance Summary"</h2>
                <Suspense fallback=move || view! { <div>"Loading summary data..."</div> }>
                    {move || {
                        let results = student_results_resource.get().unwrap_or(None);
                        match results {
                            Some(data) => {
                                let assessments = data.assessment_summaries.clone();
                                let assessments_clone = assessments.clone();
                                if assessments.is_empty() {
                                    view! { <div class="text-gray-600">"No assessment data available"</div> }
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
                                                                        {format!("{} Details", assessment_name)}
                                                                    </h3>

                                                                    <div class="mb-4">
                                                                        <h4 class="font-semibold mb-2">{"Subtests Performance"}</h4>
                                                                        {if test_details_clone.is_empty() {
                                                                            view! { <div><p class="text-gray-500">"No test data available for this assessment"</p></div> }
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
                                                                                                let test_for_class = test.clone();
                                                                                                let performance_class = test.performance_class.clone();
                                                                                                let performance_class_for_style = performance_class.clone();

                                                                                                view! {
                                                                                                    <tr class="border-t">
                                                                                                        <td class="py-2 px-3 text-sm">{test.test_name.clone()}</td>
                                                                                                        <td class="py-2 px-3 text-sm">
                                                                                                            <span class=move || {
                                                                                                                let score_percentage = (test_for_class.score as f32 / test_for_class.total_possible as f32) * 100.0;
                                                                                                                if score_percentage >= 80.0 {
                                                                                                                    "text-green-600 font-semibold"
                                                                                                                } else if score_percentage >= 60.0 {
                                                                                                                    "text-yellow-600 font-semibold"
                                                                                                                } else {
                                                                                                                    "text-red-600 font-semibold"
                                                                                                                }
                                                                                                            }>
                                                                                                                {test.score}
                                                                                                            </span>
                                                                                                        </td>
                                                                                                        <td class="py-2 px-3 text-sm">{test.total_possible}</td>
                                                                                                        <td class="py-2 px-3 text-sm">{test.test_area.clone()}</td>
                                                                                                        <td class="py-2 px-3 text-sm">{format!("{}", test.date_administered.format("%Y-%m-%d"))}</td>
                                                                                                        <td class="py-2 px-3 text-sm">
                                                                                                            <span class=move || {
                                                                                                                if performance_class_for_style.contains("Above") || performance_class_for_style.contains("High") {
                                                                                                                    "px-2 py-1 bg-green-100 text-green-800 rounded-full text-xs"
                                                                                                                } else if performance_class_for_style.contains("Average") || performance_class_for_style.contains("On Track") {
                                                                                                                    "px-2 py-1 bg-blue-100 text-blue-800 rounded-full text-xs"
                                                                                                                } else if performance_class_for_style.contains("Below") || performance_class_for_style.contains("Risk") {
                                                                                                                    "px-2 py-1 bg-red-100 text-red-800 rounded-full text-xs"
                                                                                                                } else {
                                                                                                                    "px-2 py-1 bg-gray-100 text-gray-800 rounded-full text-xs"
                                                                                                                }
                                                                                                            }>
                                                                                                                {&performance_class.clone()}
                                                                                                            </span>
                                                                                                        </td>
                                                                                                    </tr>
                                                                                                }
                                                                                            }).collect::<Vec<_>>()}
                                                                                        </tbody>
                                                                                    </table>
                                                                                </div>
                                                                            }
                                                                        }}
                                                                    </div>

                                                                    // Performance charts for this assessment
                                                                    <div class="mt-6">
                                                                        <h4 class="font-semibold mb-2">"Performance Distribution"</h4>
                                                                        <div class="border rounded-lg p-4 bg-white">
                                                                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                                                                // Distribution chart
                                                                                <div class="h-64">
                                                                                    <h5 class="text-sm font-medium text-gray-500 mb-2">"Performance Categories"</h5>
                                                                                    <div class="flex space-x-2">
                                                                                        {distribution_data.iter().map(|(category, count)| {
                                                                                            let category_clone = category.clone();
                                                                                            let color = match category.as_str() {
                                                                                                cat if cat.contains("Above") || cat.contains("High") => "bg-green-500",
                                                                                                cat if cat.contains("Average") || cat.contains("On Track") => "bg-blue-500",
                                                                                                cat if cat.contains("Below") || cat.contains("Risk") => "bg-red-500",
                                                                                                _ => "bg-gray-500"
                                                                                            };
                                                                                            let width_percent = *count as f32 / test_details_len as f32 * 100.0;

                                                                                            view! {
                                                                                                <div class="flex flex-col items-center">
                                                                                                    <div class="w-16 relative h-48 bg-gray-200 rounded overflow-hidden">
                                                                                                        <div
                                                                                                            class={format!("{} absolute bottom-0 w-full", color)}
                                                                                                            style={format!("height: {}%", width_percent)}
                                                                                                        ></div>
                                                                                                    </div>
                                                                                                    <div class="text-xs mt-1">{category_clone}</div>
                                                                                                    <div class="text-xs font-bold">{count.to_string()}</div>
                                                                                                </div>
                                                                                            }
                                                                                        }).collect::<Vec<_>>()}
                                                                                    </div>
                                                                                </div>

                                                                                // Overall assessment rating
                                                                                <div class="flex flex-col items-center justify-center border rounded-lg p-4">
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
                                                                                        {&assessment_rating_clone}
                                                                                    </div>
                                                                                    <div class="mt-2 text-sm text-gray-600">
                                                                                        "Based on " {test_details_len} " completed tests"
                                                                                    </div>
                                                                                    <div class="mt-4 flex items-center">
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
                                                                </div>
                                                            }
                                                        } else {
                                                            view! { <div></div> }
                                                        }
                                                    }}
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }
                                }
                            },
                            None => view! { <div class="text-gray-600">"No assessment data available"</div> }
                        }
                    }}
                </Suspense>
            </div>
            <div class="bg-white rounded-lg shadow-md p-6 mb-6">
                <div class="flex justify-between items-center mb-4">
                    <h2 class="text-xl font-bold">"Test Score Ledger"</h2>
                    <button
                        class="px-3 py-1 bg-gray-200 text-gray-700 rounded hover:bg-gray-300 flex items-center"
                        on:click=move |_| set_show_filters.update(|v| *v = !*v)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
                        </svg>
                        {move || if show_filters.get() { "Hide Filters" } else { "Show Filters" }}
                    </button>
                </div>

                <Suspense fallback=move || view! { <div>"Loading test history data..."</div> }>
                    {move || {
                        let results = student_results_resource.get().unwrap_or(None);
                        match results {
                            Some(data) => {
                                let test_history = data.test_history.clone();

                                if test_history.is_empty() {
                                    view! { <div class="text-gray-600">"No test history available"</div> }
                                } else {
                                    // Extract unique test names for filter dropdown
                                    let unique_test_names: HashSet<String> = test_history.iter()
                                        .map(|entry| entry.test_name.clone())
                                        .collect();

                                    let test_names_vec: Vec<String> = unique_test_names.into_iter().collect();
                                    let test_names_for_filter = test_names_vec.clone();

                                    // Filter the test history based on selected filters
                                    let filtered_history = move || {
                                        let mut filtered = test_history.clone();

                                        // Apply test name filter if selected
                                        if let Some(name) = filter_test_name.get() {
                                            filtered = filtered.into_iter()
                                                .filter(|entry| entry.test_name == name)
                                                .collect();
                                        }

                                        // Sort by most recent first
                                        filtered.sort_by(|a, b| b.date_administered.cmp(&a.date_administered));

                                        filtered
                                    };

                                    view! {
                                        <div>
                                            // Filter controls
                                            {move || {
                                                if show_filters.get() {
                                                    view! {
                                                        <div class="mb-4 p-4 bg-gray-50 rounded-lg border">
                                                            <div class="flex flex-wrap gap-4 items-end">
                                                                <div class="max-w-xs">
                                                                    <label class="block text-sm font-medium text-gray-700 mb-1" for="test-filter">
                                                                        "Filter by Test"
                                                                    </label>
                                                                    <select
                                                                        id="test-filter"
                                                                        class="block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                                                                        on:change=move |ev| {
                                                                            let value = event_target_value(&ev);
                                                                            if value.is_empty() {
                                                                                set_filter_test_name(None);
                                                                            } else {
                                                                                set_filter_test_name(Some(value));
                                                                            }
                                                                        }
                                                                    >
                                                                        <option value="">"All Tests"</option>
                                                                        {test_names_for_filter.iter().map(|name| {
                                                                            view! { <option value={name.clone()}>{name.clone()}</option> }
                                                                        }).collect::<Vec<_>>()}
                                                                    </select>
                                                                </div>

                                                                <button
                                                                    class="px-3 py-2 bg-blue-100 text-blue-800 rounded hover:bg-blue-200"
                                                                    on:click=move |_| set_filter_test_name(None)
                                                                >
                                                                    "Clear Filters"
                                                                </button>
                                                            </div>
                                                        </div>
                                                    }
                                                } else {
                                                    view! { <div></div> }
                                                }
                                            }}

                                            // Test history table
                                            <div class="overflow-x-auto">
                                                <table class="min-w-full bg-white">
                                                    <thead class="bg-gray-100">
                                                        <tr>
                                                            <th class="py-2 px-4 text-left">"Test Name"</th>
                                                            <th class="py-2 px-4 text-left">"Taken"</th>
                                                            <th class="py-2 px-4 text-left">"Score"</th>
                                                            <th class="py-2 px-4 text-left">"Possible"</th>
                                                            <th class="py-2 px-4 text-left">"Percentage"</th>
                                                            <th class="py-2 px-4 text-left">"Performance"</th>
                                                            <th class="py-2 px-4 text-left">"Evaluator"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        {move || {
                                                            let entries = filtered_history();
                                                            if entries.is_empty() {
                                                                vec![view! {
                                                                    <tr>
                                                                        <td colspan="7" class="py-4 px-4 text-center text-gray-500">
                                                                            "No matching test records found"
                                                                        </td>
                                                                    </tr>
                                                                }]
                                                            } else {
                                                                entries.into_iter().map(|entry| {
                                                                    let score_percentage = (entry.score as f32 / entry.total_possible as f32) * 100.0;
                                                                    let performance_class = entry.performance_class.clone();
                                                                    let performance_class_for_style = performance_class.clone();

                                                                    view! {
                                                                        <tr class="border-t hover:bg-gray-50">
                                                                            <td class="py-3 px-4">{entry.test_name}</td>
                                                                            <td class="py-3 px-4">{format!("{}", entry.date_administered.format("%Y-%m-%d"))}</td>
                                                                            <td class="py-3 px-4">
                                                                                <span class=move || {
                                                                                    if score_percentage >= 80.0 {
                                                                                        "text-green-600 font-semibold"
                                                                                    } else if score_percentage >= 60.0 {
                                                                                        "text-yellow-600 font-semibold"
                                                                                    } else {
                                                                                        "text-red-600 font-semibold"
                                                                                    }
                                                                                }>
                                                                                    {entry.score}
                                                                                </span>
                                                                            </td>
                                                                            <td class="py-3 px-4">{entry.total_possible}</td>
                                                                            <td class="py-3 px-4">
                                                                                <div class="flex items-center">
                                                                                    <div class="w-24 bg-gray-200 rounded-full h-2 mr-2">
                                                                                        <div
                                                                                            class=move || {
                                                                                                if score_percentage >= 80.0 {
                                                                                                    "bg-green-600 h-2 rounded-full"
                                                                                                } else if score_percentage >= 60.0 {
                                                                                                    "bg-yellow-400 h-2 rounded-full"
                                                                                                } else {
                                                                                                    "bg-red-600 h-2 rounded-full"
                                                                                                }
                                                                                            }
                                                                                            style=format!("width: {}%", score_percentage.min(100.0))
                                                                                        ></div>
                                                                                    </div>
                                                                                    <span class="text-sm">{format!("{:.1}%", score_percentage)}</span>
                                                                                </div>
                                                                            </td>
                                                                            <td class="py-3 px-4">
                                                                                <span class=move || {
                                                                                    if performance_class_for_style.contains("Above") || performance_class_for_style.contains("High") {
                                                                                        "px-2 py-1 bg-green-100 text-green-800 rounded-full text-xs"
                                                                                    } else if performance_class_for_style.contains("Average") || performance_class_for_style.contains("On Track") {
                                                                                        "px-2 py-1 bg-blue-100 text-blue-800 rounded-full text-xs"
                                                                                    } else if performance_class_for_style.contains("Below") || performance_class_for_style.contains("Risk") {
                                                                                        "px-2 py-1 bg-red-100 text-red-800 rounded-full text-xs"
                                                                                    } else {
                                                                                        "px-2 py-1 bg-gray-100 text-gray-800 rounded-full text-xs"
                                                                                    }
                                                                                }>
                                                                                    {&performance_class}
                                                                                </span>
                                                                            </td>
                                                                            <td class="py-3 px-4">{entry.evaluator}</td>
                                                                        </tr>
                                                                    }
                                                                }).collect::<Vec<_>>()
                                                            }
                                                        }}
                                                    </tbody>
                                                </table>
                                            </div>
                                        </div>
                                    }
                                }
                            },
                            None => view! { <div class="text-gray-600">"No test history available"</div> }
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}
