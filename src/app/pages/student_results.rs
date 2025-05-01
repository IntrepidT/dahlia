use crate::app::components::dashboard::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::header::Header;
use crate::app::models::assessment::Assessment;
use crate::app::models::score::Score;
use crate::app::models::student::Student;
use crate::app::models::test::Test;
use crate::app::server_functions::assessments::get_assessments;
use crate::app::server_functions::scores::get_scores;
use crate::app::server_functions::students::get_student;
use crate::app::server_functions::tests::get_tests;
// Import the chart rendering functions
use crate::app::components::data_charts::{
    render_overall_progress, render_score_distribution, render_test_distribution, render_test_plot,
};
use leptos::*;
use leptos_router::use_params_map;
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

    // Resource to fetch student data
    let student_resource = create_resource(
        move || student_id(),
        |id| async move {
            if id > 0 {
                get_student(id).await.ok()
            } else {
                None
            }
        },
    );

    // Resources for assessments, tests, and scores
    let assessments_resource = create_resource(
        || (),
        |_| async move { get_assessments().await.unwrap_or_default() },
    );

    let tests_resource = create_resource(
        || (),
        |_| async move { get_tests().await.unwrap_or_default() },
    );

    let scores_resource = create_resource(
        || (),
        |_| async move { get_scores().await.unwrap_or_default() },
    );

    // Signal to track which assessment is expanded
    let (expanded_assessment, set_expanded_assessment) = create_signal::<Option<String>>(None);

    // Function to filter scores for the current student
    let student_scores = move || {
        scores_resource
            .get()
            .map(|scores| {
                scores
                    .iter()
                    .filter(|score| score.student_id == student_id())
                    .cloned()
                    .collect::<Vec<Score>>()
            })
            .unwrap_or_default()
    };

    // Group scores by assessment
    let scores_by_assessment = move || {
        let all_scores = student_scores();
        let all_tests = tests_resource.get().unwrap_or_default();
        let all_assessments = assessments_resource.get().unwrap_or_default();

        // Create a map of test_id to Assessment
        let mut test_to_assessment_map = std::collections::HashMap::new();
        for assessment in all_assessments.iter() {
            for test_id in &assessment.tests {
                test_to_assessment_map.insert(test_id.clone(), assessment);
            }
        }

        // Group scores by assessment
        let mut result = std::collections::HashMap::new();
        for score in all_scores {
            let test = all_tests.iter().find(|t| t.test_id == score.test_id);
            if let Some(test) = test {
                if let Some(assessment) = test_to_assessment_map.get(
                    &Uuid::parse_str(&score.test_id).expect("Failed conversion string -> Uuid"),
                ) {
                    let entry = result
                        .entry(assessment.id.to_string())
                        .or_insert_with(Vec::new);
                    entry.push((score, test.clone()));
                }
            }
        }

        result
    };

    // Function to calculate average score for an assessment
    let calculate_assessment_avg = move |assessment_id: &str| {
        let scores_map = scores_by_assessment();
        if let Some(scores) = scores_map.get(assessment_id) {
            let total: i32 = scores.iter().map(|(score, _)| score.get_total()).sum();
            let count = scores.len() as i32;
            if count > 0 {
                total / count
            } else {
                0
            }
        } else {
            0
        }
    };

    view! {
        <Header />
        <div class="p-6 max-w-6xl mx-auto">
            // Student Information Section
            <Suspense fallback=move || view! { <div class="text-center p-4">"Loading student data..."</div> }>
                {move || student_resource.get().map(|student_opt| {
                    match student_opt {
                        Some(student) => view! {
                            <div class="bg-white rounded-lg shadow-md p-6 mb-6">
                                <h1 class="text-2xl font-bold mb-4">"Test Results for " {student.firstname.clone()} " " {student.lastname.clone()}</h1>
                                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                                    <div class="bg-gray-50 p-4 rounded">
                                        <h3 class="font-semibold text-gray-700">"Student ID"</h3>
                                        <p>{student.student_id}</p>
                                    </div>
                                    <div class="bg-gray-50 p-4 rounded">
                                        <h3 class="font-semibold text-gray-700">"Grade Level"</h3>
                                        <p>{student.grade.to_string()}</p>
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
                        let assessments = assessments_resource.get().unwrap_or_default();
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
                                                <th class="py-2 px-4 text-left">"Average Score"</th>
                                                <th class="py-2 px-4 text-left">"Grade Level"</th>
                                                <th class="py-2 px-4 text-left">"Action"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {assessments.into_iter().map(|assessment| {
                                                let assessment_id = assessment.id.to_string();
                                                let assessment_id_for_button = assessment_id.clone();
                                                let assessment_id_for_details = assessment_id.clone();

                                                view! {
                                                    <tr class="border-t hover:bg-gray-50">
                                                        <td class="py-3 px-4">{assessment.name}</td>
                                                        <td class="py-3 px-4">{format!("{:?}", assessment.subject)}</td>
                                                        <td class="py-3 px-4">{calculate_assessment_avg(&assessment_id)}</td>
                                                        <td class="py-3 px-4">{assessment.grade.map(|g| format!("{:?}", g)).unwrap_or_else(|| "Any".to_string())}</td>
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
                                                                    if expanded_assessment.get() == Some(assessment_id.clone()) {
                                                                        "Hide Details"
                                                                    } else {
                                                                        "Show Details"
                                                                    }
                                                                }}
                                                            </button>
                                                        </td>
                                                    </tr>

                                                    // Expanded details section
                                                    {move || {
                                                        if expanded_assessment.get() == Some(assessment_id_for_details.clone()) {
                                                            let scores_map = scores_by_assessment();
                                                            let assessment_scores = scores_map.get(&assessment_id_for_details).cloned().unwrap_or_default();

                                                            view! {
                                                                <div>
                                                                    <tr>
                                                                        <td colspan="5" class="py-4 px-6 bg-gray-50">
                                                                            <h3 class="font-semibold mb-2">{"Subtests Performance"}</h3>
                                                                            {if assessment_scores.is_empty() {
                                                                                view! { <div><p class="text-gray-500">"No test data available for this assessment"</p></div> }
                                                                            } else {
                                                                                view! {
                                                                                    <div class="overflow-x-auto">
                                                                                        <table class="min-w-full bg-white border">
                                                                                            <thead class="bg-gray-100">
                                                                                                <tr>
                                                                                                    <th class="py-2 px-3 text-left text-sm">"Test Name"</th>
                                                                                                    <th class="py-2 px-3 text-left text-sm">"Score"</th>
                                                                                                    <th class="py-2 px-3 text-left text-sm">"Test Area"</th>
                                                                                                    <th class="py-2 px-3 text-left text-sm">"Date Taken"</th>
                                                                                                    <th class="py-2 px-3 text-left text-sm">"Comments"</th>
                                                                                                </tr>
                                                                                            </thead>
                                                                                            <tbody>
                                                                                                {assessment_scores.iter().map(|(score, test)| {
                                                                                                    // Clone score for use in closures
                                                                                                    let score_for_class = score.clone();

                                                                                                    view! {
                                                                                                        <tr class="border-t">
                                                                                                            <td class="py-2 px-3 text-sm">{test.name.clone()}</td>
                                                                                                            <td class="py-2 px-3 text-sm">
                                                                                                                <span class=move || {
                                                                                                                    let total = score_for_class.get_total();
                                                                                                                    if total >= 80 {
                                                                                                                        "text-green-600 font-semibold"
                                                                                                                    } else if total >= 60 {
                                                                                                                        "text-yellow-600 font-semibold"
                                                                                                                    } else {
                                                                                                                        "text-red-600 font-semibold"
                                                                                                                    }
                                                                                                                }>
                                                                                                                    {score.get_total()}
                                                                                                                </span>
                                                                                                            </td>
                                                                                                            <td class="py-2 px-3 text-sm">{format!("{:?}", test.testarea)}</td>
                                                                                                            <td class="py-2 px-3 text-sm">{format!("{:?}", score.date_administered)}</td>
                                                                                                            <td class="py-2 px-3 text-sm">{test.comments.clone()}</td>
                                                                                                        </tr>
                                                                                                    }
                                                                                                }).collect::<Vec<_>>()}
                                                                                            </tbody>
                                                                                        </table>
                                                                                    </div>
                                                                                }
                                                                            }}

                                                                            // Performance graphs for tests in this assessment
                                                                            <div class="mt-6 grid grid-cols-1 md:grid-cols-2 gap-4">
                                                                                {assessment_scores.iter().map(|(score, test)| {
                                                                                    let score_clone = score.clone();
                                                                                    let test_clone = test.clone();
                                                                                    let test_score_data = vec![(score_clone, test_clone)];
                                                                                    let test_id = test.test_id.clone();
                                                                                    let test_name = test.name.clone();

                                                                                    // Render individual test plot
                                                                                    render_test_plot(test_id, test_name, test_score_data)
                                                                                }).collect::<Vec<_>>()}
                                                                            </div>

                                                                            // Assessment distribution chart
                                                                            {
                                                                                let assessment_scores_clone = assessment_scores.clone();
                                                                                render_test_distribution(assessment_id_for_details.clone(), assessment_scores_clone)
                                                                            }
                                                                        </td>
                                                                    </tr>
                                                                </div>
                                                            }
                                                        } else {
                                                            view! { <div></div> }
                                                        }
                                                    }}
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                </div>
                            }
                        }
                    }}
                </Suspense>
            </div>

            // Overall Progress Section
            <div class="bg-white rounded-lg shadow-md p-6">
                <h2 class="text-xl font-bold mb-4">"Overall Progress"</h2>
                <Suspense fallback=move || view! { <div>"Loading progress data..."</div> }>
                    {move || {
                        // Create a derived signal for scores that can be reused multiple times
                        let scores_memo = create_memo(move |_| student_scores());

                        let scores = scores_memo.get();
                        if scores.is_empty() {
                            view! { <div class="text-gray-600">"No progress data available"</div> }
                        } else {
                            view! {
                                <div>
                                    // Render score distribution chart
                                    {render_score_distribution(scores.clone())}

                                    // Render overall progress chart
                                    {render_overall_progress(scores.clone())}

                                    <div class="mt-6">
                                        <h3 class="font-semibold mb-2">"Performance Summary"</h3>
                                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                                            <div class="bg-gray-50 p-4 rounded">
                                                <h4 class="text-sm font-medium text-gray-500">"Average Score"</h4>
                                                <p class="text-2xl font-bold">
                                                    {move || {
                                                        let scores = scores_memo.get();
                                                        let total: i32 = scores.iter().map(|s| s.get_total()).sum();
                                                        let count = scores.len() as i32;
                                                        if count > 0 { total / count } else { 0 }
                                                    }}
                                                </p>
                                            </div>
                                            <div class="bg-gray-50 p-4 rounded">
                                                <h4 class="text-sm font-medium text-gray-500">"Highest Score"</h4>
                                                <p class="text-2xl font-bold text-green-600">
                                                    {move || {
                                                        let scores = scores_memo.get();
                                                        scores.iter().map(|s| s.get_total()).max().unwrap_or(0)
                                                    }}
                                                </p>
                                            </div>
                                            <div class="bg-gray-50 p-4 rounded">
                                                <h4 class="text-sm font-medium text-gray-500">"Lowest Score"</h4>
                                                <p class="text-2xl font-bold text-red-600">
                                                    {move || {
                                                        let scores = scores_memo.get();
                                                        scores.iter().map(|s| s.get_total()).min().unwrap_or(0)
                                                    }}
                                                </p>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            }
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}
