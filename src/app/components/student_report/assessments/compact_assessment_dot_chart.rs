use crate::app::components::data_processing::{AssessmentSummary, TestDetail};
use crate::app::models::test::Test;
use crate::app::server_functions::assessments::get_test_sequence;
use leptos::*;
use std::collections::HashMap;

#[component]
pub fn CompactProgressChart(
    assessment: AssessmentSummary,
    test_details: Vec<TestDetail>,
    tests_resource: Resource<(), Option<Vec<Test>>>,
) -> impl IntoView {
    // Get the complete test sequence for this assessment
    let test_sequence_resource = create_resource(
        move || assessment.assessment_id.clone(),
        |assessment_id| async move {
            match get_test_sequence(assessment_id).await {
                Ok(sequence) => {
                    log::info!(
                        "Successfully loaded test sequence with {} tests",
                        sequence.len()
                    );
                    Some(sequence)
                }
                Err(e) => {
                    log::error!("Failed to get test sequence: {}", e);
                    None
                }
            }
        },
    );

    // Group test details by test_id and sort by attempt
    let grouped_tests = create_memo(move |_| {
        let mut groups: HashMap<String, Vec<TestDetail>> = HashMap::new();

        for test in test_details.clone() {
            groups.entry(test.test_id.clone()).or_default().push(test);
        }

        // Sort each group by attempt number
        for attempts in groups.values_mut() {
            attempts.sort_by_key(|t| t.attempt);
        }

        groups
    });

    // Create the ordered test sequence including untaken tests
    let chart_data = create_memo(move |_| {
        let groups = grouped_tests.get();
        let test_sequence = test_sequence_resource.get().unwrap_or(None);
        let all_tests = tests_resource.get().unwrap_or(None);

        log::info!(
            "Building chart data - test_sequence loaded: {}, attempted tests: {}",
            test_sequence.is_some(),
            groups.len()
        );

        // Get the ordered test sequence
        let ordered_test_ids = if let Some(sequence) = test_sequence {
            log::info!("Using database test sequence with {} tests", sequence.len());
            // Use the sequence from the database (test_id, test_name)
            sequence
                .into_iter()
                .map(|(test_id, _test_name)| test_id)
                .collect()
        } else {
            log::warn!("No test sequence available, falling back to attempted tests");

            // Fallback to just the attempted tests from test_details
            log::info!("Using test_details as fallback");
            let mut attempted: Vec<String> = groups.keys().cloned().collect();
            attempted.sort();
            attempted
        };

        log::info!(
            "Final ordered test sequence has {} tests",
            ordered_test_ids.len()
        );

        let max_attempts = groups
            .values()
            .map(|attempts| attempts.len())
            .max()
            .unwrap_or(1);

        let total_tests = ordered_test_ids.len();

        // Calculate completion stats (tests that have at least one attempt with score > 0)
        let completed_tests = groups
            .values()
            .filter(|attempts| attempts.iter().any(|t| t.score > 0))
            .count();

        // Create a map of test_id to test_name for untaken tests
        let test_names: HashMap<String, String> = if let Some(tests) = all_tests {
            tests
                .iter()
                .map(|t| (t.test_id.clone(), t.name.clone()))
                .collect()
        } else {
            HashMap::new()
        };

        (
            groups,
            ordered_test_ids,
            max_attempts,
            total_tests,
            completed_tests,
            test_names,
        )
    });

    // Helper function to get benchmark-based color
    let get_dot_color = move |test: &TestDetail| -> (String, String, String) {
        // Get benchmark categories from tests_resource
        let benchmark_categories = tests_resource
            .get()
            .unwrap_or(None)
            .and_then(|tests| tests.iter().find(|t| t.test_id == test.test_id).cloned())
            .and_then(|t| t.benchmark_categories);

        let percentage = if test.total_possible > 0 {
            (test.score as f32 / test.total_possible as f32) * 100.0
        } else {
            0.0
        };

        if test.score == 0 {
            return (
                "bg-gray-200".to_string(),
                "ring-gray-400".to_string(),
                "opacity-40".to_string(),
            );
        }

        // Use benchmark categories if available
        if let Some(categories) = benchmark_categories {
            for category in &categories {
                if category.contains(test.score) {
                    let color = category.get_color();
                    // Convert hex color to Tailwind classes
                    let bg_class = hex_to_tailwind_bg(&color);
                    let ring_class = bg_class
                        .replace("bg-", "ring-")
                        .replace("-500", "-600")
                        .replace("-400", "-500");
                    return (bg_class, ring_class, "".to_string());
                }
            }
        }

        // Fallback to percentage-based colors (GitHub-style)
        let (bg_class, ring_class) = if percentage >= 90.0 {
            ("bg-emerald-500", "ring-emerald-600")
        } else if percentage >= 80.0 {
            ("bg-emerald-400", "ring-emerald-500")
        } else if percentage >= 70.0 {
            ("bg-yellow-400", "ring-yellow-500")
        } else if percentage >= 60.0 {
            ("bg-orange-400", "ring-orange-500")
        } else {
            ("bg-red-400", "ring-red-500")
        };

        (bg_class.to_string(), ring_class.to_string(), "".to_string())
    };

    view! {
        <div class="w-full">
            <div class="flex items-center justify-between mb-3">
                <div class="flex items-center space-x-2">
                    <div class="w-2 h-2 bg-blue-500 rounded-full"></div>
                    <span class="text-sm font-medium text-gray-700">"Test Progress"</span>
                </div>
                <span class="text-xs text-gray-500">
                    {move || {
                        let (_, ordered_test_ids, _, _, _, _) = chart_data.get();
                        format!("{} tests", ordered_test_ids.len())
                    }}
                </span>
            </div>

            <div class="relative bg-gradient-to-br from-gray-50 to-gray-100 border border-gray-200 rounded-lg p-4">
                <Suspense fallback=move || view! {
                    <div class="text-center py-4 text-gray-500 text-xs">
                        "Loading test sequence..."
                    </div>
                }>
                    {move || {
                        let (groups, ordered_test_ids, max_attempts, total_tests, completed_tests, test_names) = chart_data.get();

                        if total_tests == 0 {
                            return view! {
                                <div class="text-center py-8 text-gray-500">
                                    <div class="w-12 h-12 mx-auto mb-3 bg-gray-200 rounded-full flex items-center justify-center">
                                        <svg class="w-6 h-6 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                                        </svg>
                                    </div>
                                    <div class="text-sm font-medium text-gray-600 mb-1">"No Test Sequence"</div>
                                    <div class="text-xs text-gray-500">"No tests found for this assessment"</div>
                                </div>
                            }.into_view();
                        }

                        // Responsive sizing based on number of tests - ensure no overlap
                        let (dot_size, gap_size, show_y_axis) = if total_tests <= 6 {
                            (16, 6, max_attempts > 1)
                        } else if total_tests <= 12 {
                            (14, 4, max_attempts > 1)
                        } else if total_tests <= 20 {
                            (12, 3, false)
                        } else {
                            (10, 2, false)
                        };

                        view! {
                            <div class="space-y-4">
                                // Chart grid
                                <div class="flex items-end">
                                    // Y-axis (attempts) - only show if multiple attempts and space allows
                                    {if show_y_axis {
                                        view! {
                                            <div class="w-8 mr-3 flex flex-col justify-end" style=format!("margin-bottom: {}px", dot_size / 2)>
                                                // Attempts label at the top
                                                <div class="text-xs font-medium text-gray-500 mb-2 text-center">"Attempts"</div>
                                                // Numbers from max down to 1 (so 1 appears at bottom)
                                                <div class="space-y-1 flex flex-col">
                                                    {(1..=max_attempts).rev().map(|attempt| {
                                                        view! {
                                                            <div
                                                                class="text-xs text-gray-400 text-right leading-none font-mono"
                                                                style=format!("height: {}px; display: flex; align-items: center; justify-content: flex-end", dot_size + gap_size)
                                                            >
                                                                {attempt.to_string()}
                                                            </div>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            </div>
                                        }.into_view()
                                    } else {
                                        view! { <div class="w-0"></div> }.into_view()
                                    }}

                                    // Chart dots container
                                    <div class="flex-1 overflow-x-auto">
                                        <div class="flex items-end" style=format!("gap: {}px", gap_size)>
                                            {ordered_test_ids.iter().enumerate().map(|(index, test_id)| {
                                                let test_attempts = groups.get(test_id).cloned().unwrap_or_default();

                                                // Get test name - prioritize from attempts, then from test_names map, then fallback
                                                let test_name = test_attempts.first()
                                                    .map(|t| t.test_name.clone())
                                                    .or_else(|| test_names.get(test_id).cloned())
                                                    .or_else(|| {
                                                        // Try to get test name from tests_resource as final fallback
                                                        tests_resource.get()
                                                            .unwrap_or(None)
                                                            .and_then(|tests| {
                                                                tests.iter()
                                                                    .find(|t| &t.test_id == test_id)
                                                                    .map(|t| t.name.clone())
                                                            })
                                                    })
                                                    .unwrap_or_else(|| format!("Test {}", index + 1));

                                                // Determine if this test has been attempted at all
                                                let has_attempts = !test_attempts.is_empty();

                                                view! {
                                                    <div
                                                        class="flex flex-col hover:bg-white hover:bg-opacity-60 rounded-md p-1 transition-all duration-200"
                                                        style=format!("gap: {}px", gap_size / 2)
                                                        title=format!("{}{}", test_name, if !has_attempts { " (Not yet attempted)" } else { "" })
                                                    >
                                                        {(1..=max_attempts).rev().map(|attempt| {
                                                            let attempt_i32 = attempt as i32;
                                                            let dot_info = test_attempts.iter()
                                                                .find(|t| t.attempt == attempt_i32)
                                                                .map(|test| {
                                                                    let percentage = if test.total_possible > 0 {
                                                                        (test.score as f32 / test.total_possible as f32) * 100.0
                                                                    } else {
                                                                        0.0
                                                                    };

                                                                    let (bg_class, ring_class, opacity) = get_dot_color(test);

                                                                    let tooltip = format!("{}\nAttempt {}: {}/{} ({}%)\nDate: {}",
                                                                        test.test_name,
                                                                        attempt_i32,
                                                                        test.score,
                                                                        test.total_possible,
                                                                        percentage as i32,
                                                                        test.date_administered.format("%m/%d/%Y")
                                                                    );

                                                                    (bg_class, ring_class, opacity, tooltip)
                                                                });

                                                            match dot_info {
                                                                Some((bg_class, ring_class, opacity, tooltip)) => view! {
                                                                    <div
                                                                        class=format!("rounded-sm cursor-pointer hover:ring-2 hover:{} hover:scale-110 transition-all duration-200 shadow-sm {} {}", ring_class, bg_class, opacity)
                                                                        style=format!("width: {}px; height: {}px", dot_size, dot_size)
                                                                        title=tooltip
                                                                    ></div>
                                                                },
                                                                None => {
                                                                    // Show different styles based on whether test has been attempted
                                                                    if !has_attempts && attempt == 1 {
                                                                        // Untaken test in sequence - show dotted outline with better visibility
                                                                        view! {
                                                                            <div
                                                                                class="rounded-sm border-2 border-dashed border-blue-300 bg-blue-50 hover:bg-blue-100 hover:border-blue-400 transition-all duration-200"
                                                                                style=format!("width: {}px; height: {}px", dot_size, dot_size)
                                                                                title=format!("{}\nNext in sequence - Not yet attempted", test_name)
                                                                            ></div>
                                                                        }
                                                                    } else if has_attempts && attempt > test_attempts.len() {
                                                                        // Additional attempt slots for attempted tests - light gray
                                                                        view! {
                                                                            <div
                                                                                class="bg-gray-100 rounded-sm opacity-30 border border-gray-200"
                                                                                style=format!("width: {}px; height: {}px", dot_size, dot_size)
                                                                                title=format!("{}\nAttempt {} - Available", test_name, attempt_i32)
                                                                            ></div>
                                                                        }
                                                                    } else {
                                                                        // Empty space for higher attempts of untaken tests
                                                                        view! {
                                                                            <div
                                                                                style=format!("width: {}px; height: {}px", dot_size, dot_size)
                                                                            ></div>
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    </div>
                                </div>

                                // Summary stats and legend
                                <div class="bg-white bg-opacity-70 rounded-md p-2 border border-gray-200">
                                    <div class="flex items-center justify-between">
                                        <div class="flex items-center space-x-4">
                                            // Completion indicator
                                            <div class="flex items-center space-x-2">
                                                <div class="flex items-center space-x-1">
                                                    <div class="w-2 h-2 bg-emerald-500 rounded-full"></div>
                                                    <span class="text-sm font-medium text-gray-700">
                                                        {format!("{}/{}", completed_tests, total_tests)}
                                                    </span>
                                                </div>
                                                <span class="text-xs text-gray-500">"completed"</span>
                                            </div>

                                            // Progress percentage
                                            <div class="text-xs text-gray-600 bg-gray-100 px-2 py-1 rounded-full">
                                                {format!("{}%",
                                                    if total_tests > 0 { (completed_tests * 100) / total_tests } else { 0 }
                                                )}
                                            </div>

                                            // Pending tests indicator - updated styling
                                            <div class="flex items-center space-x-1">
                                                <div class="w-2 h-2 border-2 border-dashed border-blue-300 bg-blue-50 rounded-sm"></div>
                                                <span class="text-xs text-gray-500">
                                                    {format!("{} pending", total_tests - completed_tests)}
                                                </span>
                                            </div>
                                        </div>

                                        // Mini legend
                                        <div class="flex items-center space-x-2">
                                            <span class="text-xs text-gray-500">"Performance:"</span>
                                            <div class="flex space-x-1">
                                                <div class="w-2 h-2 bg-red-400 rounded-sm border border-red-500" title="Below 60%"></div>
                                                <div class="w-2 h-2 bg-orange-400 rounded-sm border border-orange-500" title="60-69%"></div>
                                                <div class="w-2 h-2 bg-yellow-400 rounded-sm border border-yellow-500" title="70-79%"></div>
                                                <div class="w-2 h-2 bg-emerald-400 rounded-sm border border-emerald-500" title="80-89%"></div>
                                                <div class="w-2 h-2 bg-emerald-500 rounded-sm border border-emerald-600" title="90%+"></div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }.into_view()
                    }}
                </Suspense>
            </div>
        </div>
    }
}

// Helper function to convert hex colors to Tailwind classes
fn hex_to_tailwind_bg(hex: &str) -> String {
    match hex.to_lowercase().as_str() {
        "#22c55e" | "#16a34a" | "#15803d" => "bg-green-500",
        "#eab308" | "#ca8a04" | "#a16207" => "bg-yellow-500",
        "#f97316" | "#ea580c" | "#c2410c" => "bg-orange-500",
        "#ef4444" | "#dc2626" | "#b91c1c" => "bg-red-500",
        "#3b82f6" | "#2563eb" | "#1d4ed8" => "bg-blue-500",
        "#8b5cf6" | "#7c3aed" | "#6d28d9" => "bg-purple-500",
        "#06b6d4" | "#0891b2" | "#0e7490" => "bg-cyan-500",
        "#10b981" | "#059669" | "#047857" => "bg-emerald-500",
        _ => "bg-gray-500", // fallback
    }
    .to_string()
}
