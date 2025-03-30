use crate::app::models::score::{DeleteScoreRequest, Score};
use crate::app::server_functions::scores::{delete_score, get_scores};
use chrono::DateTime;
use leptos::*;

#[component]
pub fn ScoresLedger() -> impl IntoView {
    // Create resource for fetching scores from the database
    let scores_resource = create_resource(
        || (),
        |_| async {
            match get_scores().await {
                Ok(scores) => Ok(scores),
                Err(e) => {
                    log::error!("Failed to load scores: {}", e);
                    Err(ServerFnError::new("Failed to load scores"))
                }
            }
        },
    );

    // Function to format date
    let format_date =
        |date: DateTime<chrono::Utc>| -> String { date.format("%b %d, %Y").to_string() };

    // Function to format time
    let format_time = |date: DateTime<chrono::Utc>| -> String { date.format("%H:%M").to_string() };

    // Function to calculate percentage
    let calculate_percentage = |test_scores: &Vec<i32>| -> String {
        let score: i32 = test_scores.iter().sum();
        let max_score = test_scores.len() as i32;
        if max_score > 0 {
            format!("{:.1}%", (score as f64 / max_score as f64 * 100.0))
        } else {
            "N/A".to_string()
        }
    };

    // Function to determine badge color based on score percentage
    let get_badge_color = |test_scores: &Vec<i32>| -> &'static str {
        let score: i32 = test_scores.iter().sum();
        let max_score = test_scores.len() as i32;

        if max_score <= 0 {
            return "bg-gray-100 text-gray-800";
        }

        let percentage = (score as f64 / max_score as f64) * 100.0;

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
    let format_score = |test_scores: &Vec<i32>| -> String {
        let score: i32 = test_scores.iter().sum();
        let max_score = test_scores.len() as i32;
        format!("{} / {}", score, max_score)
    };

    view! {
        <div class="w-full">
            <div class="flex items-center justify-between mb-4">
                <h2 class="text-xl font-medium">Recent Scores</h2>
                <div>
                    <button class="text-indigo-600 hover:text-indigo-800 text-sm font-medium">
                        View all
                    </button>
                </div>
            </div>

            <div class="bg-white overflow-hidden shadow-sm sm:rounded-lg border border-gray-200">
                <div class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-gray-200">
                        <thead class="bg-gray-50">
                            <tr>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                    Student ID
                                </th>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                    Test
                                </th>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                    Date
                                </th>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                    Score
                                </th>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                    Percentage
                                </th>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                    Evaluator
                                </th>
                                <th scope="col" class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                                    Actions
                                </th>
                            </tr>
                        </thead>
                        <tbody class="bg-white divide-y divide-gray-200">
                            {move || {
                                scores_resource.get().map(|result| {
                                    match result {
                                        Ok(scores) => {
                                            if scores.is_empty() {
                                                view! {
                                                    <tr>
                                                        <td colspan="7" class="px-6 py-4 text-center text-sm text-gray-500">
                                                            No scores found.
                                                        </td>
                                                    </tr>
                                                }
                                                .into_view()
                                            } else {
                                                scores.iter().map(|score| {
                                                    let student_id = score.student_id;
                                                    let test_id = score.test_id.clone();
                                                    let test_variant = score.test_variant;

                                                    // Create delete request for this score
                                                    let delete_req = DeleteScoreRequest {
                                                        student_id,
                                                        test_id: test_id.clone(),
                                                        test_variant,
                                                    };


                                                    view! {
                                                        <tr class="hover:bg-gray-50">
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                                                                {score.student_id}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                                                <div class="flex flex-col">
                                                                    <span>{&score.test_id}</span>
                                                                    <span class="text-xs text-gray-400">{"Variant: "}{score.test_variant}</span>
                                                                </div>
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                                                <div class="flex flex-col">
                                                                    <span>{format_date(score.date_administered)}</span>
                                                                    <span class="text-xs text-gray-400">{format_time(score.date_administered)}</span>
                                                                </div>
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                                                {format_score(&score.test_scores)}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap">
                                                                <span class={"px-2 inline-flex text-xs leading-5 font-semibold rounded-full ".to_string() + get_badge_color(&score.test_scores)}>
                                                                    {calculate_percentage(&score.test_scores)}
                                                                </span>
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                                                {&score.evaluator}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                                                <button class="text-indigo-600 hover:text-indigo-900 mr-3">
                                                                    View
                                                                </button>
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
                                                    <td colspan="7" class="px-6 py-4 text-center text-sm text-gray-500">
                                                        Failed to load scores. Please try again later.
                                                    </td>
                                                </tr>
                                            }
                                            .into_view()
                                        }
                                    }
                                }).unwrap_or_else(|| view! {
                                    <tr>
                                        <td colspan="7" class="px-6 py-4 text-center text-sm text-gray-500">
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
