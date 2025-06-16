use crate::app::models::score::{DeleteScoreRequest, Score};
use crate::app::server_functions::scores::{delete_score, get_scores};
use crate::app::server_functions::students::get_students;
use crate::app::server_functions::tests::get_tests;
use chrono::DateTime;
use leptos::*;

#[component]
pub fn ScoresLedger() -> impl IntoView {
    let navigate = leptos_router::use_navigate();
    // Create resource for fetching scores from the database
    let scores_resource = create_local_resource(
        || (),
        |_| async {
            match get_scores().await {
                Ok(mut scores) => {
                    if scores.len() > 4 {
                        scores.truncate(4);
                    }
                    Ok(scores)
                }
                Err(e) => {
                    log::error!("Failed to load scores: {}", e);
                    Err(ServerFnError::new("Failed to load scores"))
                }
            }
        },
    );

    let students_resource = create_local_resource(
        || (),
        |_| async {
            match get_students().await {
                Ok(students) => Ok(students),
                Err(e) => {
                    log::error!("Failed to load students: {}", e);
                    Err(ServerFnError::new("Failed to load students"))
                }
            }
        },
    );

    let tests_resource = create_local_resource(
        || (),
        |_| async {
            match get_tests().await {
                Ok(mut tests) => Ok(tests),
                Err(e) => {
                    log::error!("Failed to load tests: {}", e);
                    Err(ServerFnError::new("Failed to load tests"))
                }
            }
        },
    );
    //helper function to get test name
    let get_test_name = move |test_id: String| -> String {
        if let Some(Ok(tests)) = tests_resource.get() {
            if let Some(test) = tests.iter().find(|t| t.test_id == test_id) {
                return format!("{}", test.name);
            }
        }
        "Unknown Test".to_string()
    };
    //helper function to get a max score for test
    let get_max_score = move |test_id: String| -> i32 {
        if let Some(Ok(tests)) = tests_resource.get() {
            if let Some(test) = tests.iter().find(|t| t.test_id == *test_id) {
                return test.score;
            }
        }
        0
    };

    //helper function to get student name
    let get_student_name = move |student_id: i32| -> String {
        if let Some(Ok(students)) = students_resource.get() {
            if let Some(student) = students.iter().find(|s| s.student_id == student_id) {
                return format!(
                    "{} {}",
                    student.firstname.as_ref().unwrap_or(&"Unknown".to_string()),
                    student.lastname.as_ref().unwrap_or(&"Student".to_string())
                );
            }
        }
        "Unknown Student".to_string()
    };

    let (expanded_view, set_expanded_view) = create_signal(false);

    let toggle_expanded_view = move |_| {
        set_expanded_view.update(|val| *val = !*val);
    };

    // Function to format date
    let format_date =
        |date: DateTime<chrono::Utc>| -> String { date.format("%b %d, %Y").to_string() };

    // Function to format time
    let format_time = |date: DateTime<chrono::Utc>| -> String { date.format("%H:%M").to_string() };

    // Function to calculate percentage
    let calculate_percentage = move |test_scores: &Vec<i32>, test_id: String| -> String {
        let score: i32 = test_scores.iter().sum();
        let max_score = get_max_score(test_id.clone());

        if max_score > 0 {
            format!("{:.1}%", (score as f64 / max_score as f64 * 100.0))
        } else {
            "N/A".to_string()
        }
    };

    let get_benchmark_label = move |test_scores: &Vec<i32>, test_id: &String| -> String {
        let score: i32 = test_scores.iter().sum();
        let max_score = get_max_score(test_id.clone());

        if max_score <= 0 {
            return "N/A".to_string();
        }

        let percentage = (score as f64 / max_score as f64) * 100.0;

        if let Some(Ok(tests)) = tests_resource.get() {
            if let Some(test) = tests.iter().find(|t| t.test_id == *test_id) {
                if let Some(benchmark_categories) = &test.benchmark_categories {
                    for category in benchmark_categories {
                        let min_percent = category.min as f64;
                        let max_percent = category.max as f64;
                        if percentage >= min_percent && percentage <= max_percent {
                            return category.label.clone();
                        }
                    }
                }
            }
        }

        if percentage >= 90.0 {
            "Excellent".to_string()
        } else if percentage >= 80.0 {
            "Good".to_string()
        } else if percentage >= 70.0 {
            "Satisfactory".to_string()
        } else {
            "Needs Improvement".to_string()
        }
    };

    // Function to determine badge color based on score percentage
    let get_badge_color = move |test_scores: &Vec<i32>, test_id: String| -> &'static str {
        let score: i32 = test_scores.iter().sum();
        let max_score = get_max_score(test_id.clone());

        if max_score <= 0 {
            return "bg-gray-100 text-gray-800";
        }

        let percentage = (score as f64 / max_score as f64) * 100.0;

        if let Some(Ok(tests)) = tests_resource.get() {
            if let Some(test) = tests.iter().find(|t| t.test_id == *test_id) {
                if let Some(benchmark_categories) = &test.benchmark_categories {
                    for category in benchmark_categories {
                        let min_percent = category.min as f64;
                        let max_percent = category.max as f64;
                        if percentage >= min_percent && percentage <= max_percent {
                            if min_percent >= 85.0 {
                                return "bg-green-100 text-green-800";
                            } else if min_percent >= 65.0 {
                                return "bg-yellow-100 text-yellow-800";
                            } else {
                                return "bg-red-100 text-red-800";
                            }
                        }
                    }
                }
            }
        }

        if percentage >= 90.0 {
            "bg-green-100 text-green-800"
        } else if percentage >= 70.0 {
            "bg-blue-100 text-blue-800"
        } else if percentage >= 60.0 {
            "bg-yellow-100 text-yellow-800"
        } else {
            "bg-red-100 text-red-800"
        }
    };

    // Calculate score value from test_scores
    let format_score = |test_scores: &Vec<i32>, test_id: String| -> String {
        let score: i32 = test_scores.iter().sum();
        let max_score = get_max_score(test_id.clone());
        format!("{} / {}", score, max_score)
    };

    view! {
        <div class={move || {
            if expanded_view() {
                "fixed inset-0 z-50 bg-[#F9F9F8] flex flex-col p-5"
            } else {
                "w-full"
            }
        }}>
            <div class="flex items-center justify-between mb-2 p-2">
                <h2 class="text-xl font-bold">Recent Scores</h2>
                <div>
                    <button
                        class="text-indigo-600 hover:text-indigo-800 text-sm font-medium"
                        on:click=toggle_expanded_view
                    >
                        {move || if expanded_view() {"Collapse"} else {"View all"}}
                    </button>
                </div>
            </div>

            <div class={move ||{
                let base_classes = "bg-[#F9F9F8] overflow-hidden shadow-lg sm:rounded-lg border border-gray-200";
                if expanded_view() {
                    format!("{} flex-grow overflow-hidden", base_classes)
                } else {
                    base_classes.to_string()
                }
            }}>
                <div class={move || {
                    let base_classes = "overflow-x-auto overflow-y-auto";
                    if expanded_view() {
                        format!("{} h-full", base_classes)
                    } else {
                        format!("{} max-h-80", base_classes)
                    }
                }}>
                    <table class="min-w-full divide-y divide-[#DADADA]">
                        <thead class="bg-[#DADADA]">
                            <tr>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-[#2E3A59] uppercase tracking-wider">
                                    Student ID
                                </th>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-[#2E3A59] uppercase tracking-wider">
                                    Student Name
                                </th>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-[#2E3A59] uppercase tracking-wider">
                                    Test
                                </th>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-[#2E3A59] uppercase tracking-wider">
                                    Date
                                </th>
                                /*<th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                    Score
                                </th>*/
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-[#2E3A59] uppercase tracking-wider">
                                    Percentage
                                </th>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-[#2E3A59] uppercase tracking-wider">
                                    Benchmark
                                </th>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-[#2E3A59] uppercase tracking-wider">
                                    Evaluator
                                </th>
                                <th scope="col" class="px-6 py-3 text-right text-xs font-medium text-[#2E3A59] uppercase tracking-wider">
                                    Actions
                                </th>
                            </tr>
                        </thead>
                        <tbody class="bg-[#F9F9F8] divide-y divide-[#DADADA]">
                            {move || {
                                scores_resource.get().map(|result| {
                                    match result {
                                        Ok(scores) => {
                                            if scores.is_empty() {
                                                view! {
                                                    <tr>
                                                        <td colspan="7" class="px-6 py-4 text-center text-sm text-[#2E3A59]">
                                                            No scores found.
                                                        </td>
                                                    </tr>
                                                }
                                                .into_view()
                                            } else {
                                                scores.iter().rev().map(|score| {
                                                    let student_id = score.student_id;
                                                    let test_id = score.test_id.clone();
                                                    let test_variant = score.test_variant;
                                                    let attempt = score.attempt;


                                                    // Create delete request for this score
                                                    let delete_req = DeleteScoreRequest {
                                                        student_id,
                                                        test_id: test_id.clone(),
                                                        test_variant,
                                                        attempt,
                                                    };


                                                    view! {
                                                        <tr class="hover:bg-gray-50">
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-[#2E3A59]">
                                                                {score.student_id}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-[#2E3A59]">
                                                                {get_student_name(score.student_id)}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-[#2E3A59]">
                                                                <div class="flex flex-col">
                                                                    <span>{get_test_name(score.test_id.clone())}</span>
                                                                    <span class="text-xs text-[#2E3A59]">{"Variant: "}{score.test_variant}</span>
                                                                </div>
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-[#2E3A59]">
                                                                <div class="flex flex-col">
                                                                    <span>{format_date(score.date_administered)}</span>
                                                                    <span class="text-xs text-[#2E3A59]">{format_time(score.date_administered)}</span>
                                                                </div>
                                                            </td>
                                                            /*<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                                                {format_score(&score.test_scores)}
                                                            </td>*/
                                                            <td class="px-6 py-4 whitespace-nowrap">
                                                                <span class={"px-2 inline-flex text-xs leading-5 font-semibold rounded-full ".to_string() + get_badge_color(&score.test_scores, score.test_id.clone())}>
                                                                    {calculate_percentage(&score.test_scores, score.test_id.clone())}
                                                                </span>
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap">
                                                                <span class={"px-2 inline-flex text-xs leading-5 font-semibold rounded-full".to_string() + get_badge_color(&score.test_scores, score.test_id.clone())}>
                                                                    {get_benchmark_label(&score.test_scores, &score.test_id)}
                                                                </span>
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-[#2E3A59]">
                                                                {&score.evaluator}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                                                {
                                                                    let nav = navigate.clone();
                                                                    let test_id = score.test_id.clone();
                                                                    let student_id = score.student_id;
                                                                    let test_variant = score.test_variant.clone();
                                                                    let attempt = score.attempt.clone();

                                                                    view! {
                                                                        <button
                                                                            class="text-indigo-600 hover:text-indigo-900 mr-3"
                                                                            on:click=move |_| {
                                                                                nav(&format!("/reviewtest/{}/{}/{}/{}", test_id, student_id, test_variant, attempt), Default::default());
                                                                            }
                                                                        >
                                                                            View
                                                                        </button>
                                                                    }
                                                                }
                                                            </td>
                                                        </tr>
                                                    }
                                                })
                                                .collect_view()
                                            }
                                        }
                                        Err(_) => {
                                            view! {
                                                <tr>
                                                    <td colspan="7" class="px-6 py-4 text-center text-sm text-[#2E3A59]">
                                                        Failed to load scores. Please try again later.
                                                    </td>
                                                </tr>
                                            }
                                            .into_view()
                                        }
                                    }
                                }).unwrap_or_else(|| view! {
                                    <tr>
                                        <td colspan="7" class="px-6 py-4 text-center text-sm text-[#2E3A59]">
                                            <div class="flex justify-center items-center">
                                                <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-indigo-500" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                                </svg>
                                                Loading scores...
                                            </div>
                                        </td>
                                    </tr>
                                }.into_view())
                            }}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}
