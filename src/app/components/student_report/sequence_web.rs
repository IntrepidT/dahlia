use crate::app::components::data_processing::{AssessmentSummary, Progress, TestDetail};
use leptos::*;
use std::collections::HashMap;

#[component]
pub fn SequenceWeb(assessment: AssessmentSummary, test_details: Vec<TestDetail>) -> impl IntoView {
    // Get ALL possible tests from the assessment, not just completed ones
    let all_assessment_tests = create_memo(move |_| {
        // Create a comprehensive list of all tests that should be in this assessment
        let mut all_tests: Vec<TestDetail> = Vec::new();

        // First, add all completed tests
        for test in &test_details {
            all_tests.push(test.clone());
        }

        // Then, we need to ensure we have placeholders for any missing tests
        // This would require access to the full assessment.tests list from the Assessment model
        // For now, we'll work with what we have and assume test_details contains all possible tests
        // even if some have score = 0

        // Sort by the order they should appear (you might want to customize this)
        all_tests.sort_by(|a, b| {
            // First by test area, then by test name
            a.test_area
                .cmp(&b.test_area)
                .then(a.test_name.cmp(&b.test_name))
        });

        all_tests
    });

    let total_tests = create_memo(move |_| all_assessment_tests.get().len());
    let completed_tests = create_memo(move |_| {
        all_assessment_tests
            .get()
            .iter()
            .filter(|t| t.score > 0)
            .count()
    });

    // Calculate overall progress percentage based on ALL possible tests
    let progress_percentage = create_memo(move |_| {
        let total = total_tests.get();
        let completed = completed_tests.get();
        if total > 0 {
            (completed as f32 / total as f32) * 100.0
        } else {
            0.0
        }
    });

    // Calculate overall score percentage
    let overall_score = create_memo(move |_| {
        let tests = all_assessment_tests.get();
        if !tests.is_empty() {
            let total_score: i32 = tests.iter().filter(|t| t.score > 0).map(|t| t.score).sum();
            let total_possible: i32 = tests
                .iter()
                .filter(|t| t.score > 0)
                .map(|t| t.total_possible)
                .sum();
            if total_possible > 0 {
                (total_score as f32 / total_possible as f32) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        }
    });

    // Clone values that will be moved into closures
    let progress_for_class = assessment.progress.clone();
    let progress_for_icon = assessment.progress.clone();
    let assessment_name_clone = assessment.assessment_name.clone();
    let subject_clone = assessment.subject.clone();

    view! {
        <div class="sequence-web-container bg-white border border-gray-200 rounded-xl shadow-sm hover:shadow-md transition-all duration-200">
            {/* Header Section */}
            <div class="px-6 py-4 border-b border-gray-100">
                <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-3">
                        <div class=move || {
                            match progress_for_class {
                                Progress::Completed => "flex items-center justify-center w-6 h-6 bg-emerald-100 text-emerald-600 rounded-full",
                                Progress::Ongoing => "flex items-center justify-center w-6 h-6 bg-blue-100 text-blue-600 rounded-full",
                                Progress::NotStarted => "flex items-center justify-center w-6 h-6 bg-gray-100 text-gray-500 rounded-full",
                            }
                        }>
                            <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                                {match progress_for_icon {
                                    Progress::Completed => view! {
                                        <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                                    },
                                    Progress::Ongoing => view! {
                                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z" clip-rule="evenodd"/>
                                    },
                                    Progress::NotStarted => view! {
                                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-12a1 1 0 10-2 0v4a1 1 0 00.293.707l2.828 2.829a1 1 0 101.415-1.415L11 9.586V6z" clip-rule="evenodd"/>
                                    },
                                }}
                            </svg>
                        </div>

                        <div>
                            <h3 class="text-lg font-semibold text-gray-900">{assessment_name_clone}</h3>
                            <p class="text-sm text-gray-500">{subject_clone}</p>
                        </div>
                    </div>

                    <div class="flex items-center space-x-6">
                        {/* Progress Stats */}
                        <div class="text-right">
                            <div class="text-sm font-medium text-gray-900">
                                {move || completed_tests.get()} " of " {move || total_tests.get()} " completed"
                            </div>
                            <div class="text-xs text-gray-500">
                                {move || format!("{:.0}% progress", progress_percentage.get())}
                            </div>
                        </div>

                        {/* Overall Score */}
                        <div class="text-right">
                            <div class=move || {
                                let score = overall_score.get();
                                if score >= 80.0 {
                                    "text-lg font-bold text-emerald-600"
                                } else if score >= 60.0 {
                                    "text-lg font-bold text-amber-600"
                                } else if score > 0.0 {
                                    "text-lg font-bold text-red-600"
                                } else {
                                    "text-lg font-bold text-gray-400"
                                }
                            }>
                                {move || {
                                    let score = overall_score.get();
                                    if score > 0.0 {
                                        format!("{:.0}%", score)
                                    } else {
                                        "—".to_string()
                                    }
                                }}
                            </div>
                            <div class="text-xs text-gray-500">Overall Score</div>
                        </div>
                    </div>
                </div>

                {/* Progress Bar */}
                <div class="mt-4">
                    <div class="flex items-center justify-between text-xs text-gray-500 mb-1">
                        <span>Assessment Progress</span>
                        <span>{move || format!("{:.0}%", progress_percentage.get())}</span>
                    </div>
                    <div class="w-full bg-gray-200 rounded-full h-2">
                        <div
                            class="bg-gradient-to-r from-blue-500 to-blue-600 h-2 rounded-full transition-all duration-1000 ease-out"
                            style=move || format!("width: {}%", progress_percentage.get())
                        ></div>
                    </div>
                </div>
            </div>

            {/* Test Sequence */}
            <div class="p-6">
                <TestSequence test_details={all_assessment_tests.get()} />
            </div>
        </div>
    }
}

#[component]
pub fn TestSequence(test_details: Vec<TestDetail>) -> impl IntoView {
    // Group tests by test name to handle multiple attempts
    let grouped_tests = create_memo(move |_| {
        let mut groups: HashMap<String, Vec<TestDetail>> = HashMap::new();

        for test in test_details.iter() {
            groups
                .entry(test.test_name.clone())
                .or_insert_with(Vec::new)
                .push(test.clone());
        }

        // Sort groups by first occurrence and attempts by attempt number
        let mut sorted_groups: Vec<(String, Vec<TestDetail>)> = groups.into_iter().collect();
        sorted_groups.sort_by_key(|(_, tests)| {
            tests
                .iter()
                .map(|t| t.date_administered)
                .min()
                .unwrap_or_default()
        });

        for (_, tests) in sorted_groups.iter_mut() {
            tests.sort_by_key(|t| t.attempt);
        }

        sorted_groups
    });

    // State for selected node
    let (selected_node, set_selected_node) = create_signal::<Option<String>>(None);

    // Find next available test (first test with score = 0)
    let next_test = create_memo(move |_| {
        let groups = grouped_tests.get();
        groups
            .iter()
            .find(|(_, tests)| {
                // Check if this test group has any completed attempts
                let has_completed = tests.iter().any(|t| t.score > 0);
                !has_completed // If no completed attempts, this is available
            })
            .map(|(name, _)| name.clone())
    });

    // Find most recently completed test
    let most_recent_completed = create_memo(move |_| {
        let groups = grouped_tests.get();
        groups
            .iter()
            .filter(|(_, tests)| tests.iter().any(|t| t.score > 0))
            .max_by_key(|(_, tests)| {
                tests
                    .iter()
                    .filter(|t| t.score > 0)
                    .map(|t| t.date_administered)
                    .max()
                    .unwrap_or_default()
            })
            .map(|(name, _)| name.clone())
    });

    view! {
        <div class="test-sequence">
            {/* Sequence Navigation */}
            <div class="relative">
                {/* Background Line */}
                <div class="absolute top-8 left-8 right-8 h-0.5 bg-gray-200"></div>

                {/* Progress Line */}
                <div
                    class="absolute top-8 left-8 h-0.5 bg-gradient-to-r from-blue-500 to-emerald-500 transition-all duration-1000 ease-out"
                    style=move || {
                        let groups = grouped_tests.get();
                        let total_groups = groups.len();

                        if total_groups <= 1 {
                            return "width: 0%".to_string();
                        }

                        let completed_groups = groups.iter()
                            .filter(|(_, tests)| tests.iter().any(|t| t.score > 0))
                            .count();

                        // Calculate progress as a percentage of total tests completed
                        let progress = if total_groups > 0 {
                            (completed_groups as f32 / total_groups as f32 * 100.0).min(100.0)
                        } else {
                            0.0
                        };

                        // Calculate the width to span between nodes, accounting for node spacing
                        let node_spacing_adjustment = if total_groups > 1 {
                            // Adjust for the spacing between nodes
                            progress * (1.0 - (32.0 / (total_groups as f32 * 80.0))) // Approximate adjustment
                        } else {
                            0.0
                        };

                        format!("width: calc({}% - 2rem)", node_spacing_adjustment.max(0.0))
                    }
                ></div>

                {/* Test Nodes */}
                <div class="flex justify-between items-start relative z-10">
                    {move || {
                        let groups = grouped_tests.get();
                        let next_available = next_test.get();
                        let most_recent = most_recent_completed.get();

                        groups.iter().enumerate().map(|(index, (test_name, attempts))| {
                            let test_name_clone = test_name.clone();
                            let test_name_for_click = test_name.clone();
                            let latest_attempt = attempts.iter().max_by_key(|t| t.attempt).unwrap();
                            let is_completed = latest_attempt.score > 0;
                            let is_next = Some(test_name.clone()) == next_available;
                            let is_most_recent = Some(test_name.clone()) == most_recent;
                            let has_multiple_attempts = attempts.len() > 1;

                            let score_percentage = if latest_attempt.total_possible > 0 {
                                (latest_attempt.score as f32 / latest_attempt.total_possible as f32) * 100.0
                            } else {
                                0.0
                            };

                            view! {
                                <div class="flex flex-col items-center space-y-2 cursor-pointer group"
                                     on:click=move |_| {
                                        if selected_node.get() == Some(test_name_for_click.clone()) {
                                            set_selected_node(None);
                                        } else {
                                            set_selected_node(Some(test_name_for_click.clone()));
                                        }
                                     }
                                >
                                    {/* Node */}
                                    <div class="relative">
                                        <div class=move || {
                                            let mut classes = "w-16 h-16 rounded-full flex items-center justify-center text-sm font-bold transition-all duration-200 border-4".to_string();

                                            if is_completed {
                                                if score_percentage >= 80.0 {
                                                    classes.push_str(" bg-emerald-50 border-emerald-500 text-emerald-700 group-hover:bg-emerald-100");
                                                } else if score_percentage >= 60.0 {
                                                    classes.push_str(" bg-amber-50 border-amber-500 text-amber-700 group-hover:bg-amber-100");
                                                } else {
                                                    classes.push_str(" bg-red-50 border-red-500 text-red-700 group-hover:bg-red-100");
                                                }

                                                // Add pulse animation for most recently completed
                                                if is_most_recent {
                                                    classes.push_str(" ring-4 ring-emerald-300 ring-opacity-75 animate-pulse");
                                                }
                                            } else if is_next {
                                                classes.push_str(" bg-blue-50 border-blue-500 text-blue-700 group-hover:bg-blue-100");
                                                // Add pulsing ring for next test
                                                classes.push_str(" ring-4 ring-blue-300 ring-opacity-75 animate-pulse");
                                            } else {
                                                classes.push_str(" bg-gray-50 border-gray-300 text-gray-600 group-hover:bg-gray-100");
                                            }

                                            if selected_node.get() == Some(test_name_clone.clone()) {
                                                classes.push_str(" ring-4 ring-blue-400 ring-opacity-100 scale-105");
                                            }

                                            classes
                                        }>
                                            {if is_completed {
                                                view! {
                                                    <div class="w-6 h-6">
                                                        <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
                                                            <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                                                        </svg>
                                                    </div>
                                                }
                                            } else if is_next {
                                                view! {
                                                    <div class="w-6 h-6">
                                                        <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
                                                            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z" clip-rule="evenodd"/>
                                                        </svg>
                                                    </div>
                                                }
                                            } else {
                                                view! {
                                                    <div class="w-6 h-6 flex items-center justify-center">
                                                        <span>{(index + 1).to_string()}</span>
                                                    </div>
                                                }
                                            }}
                                        </div>

                                        {/* Multiple Attempts Indicator */}
                                        {if has_multiple_attempts {
                                            view! {
                                                <div class="absolute -top-1 -right-1 w-6 h-6 bg-blue-500 text-white rounded-full flex items-center justify-center text-xs font-bold border-2 border-white">
                                                    {attempts.len()}
                                                </div>
                                            }
                                        } else {
                                            view! { <div></div> }
                                        }}

                                        {/* Score Badge */}
                                        {if is_completed {
                                            view! {
                                                <div class=move || {
                                                    let mut badge_classes = "absolute -bottom-2 left-1/2 transform -translate-x-1/2 px-2 py-1 rounded-full text-xs font-bold".to_string();
                                                    if score_percentage >= 80.0 {
                                                        badge_classes.push_str(" bg-emerald-600 text-white");
                                                    } else if score_percentage >= 60.0 {
                                                        badge_classes.push_str(" bg-amber-600 text-white");
                                                    } else {
                                                        badge_classes.push_str(" bg-red-600 text-white");
                                                    }
                                                    badge_classes
                                                }>
                                                    {format!("{}%", score_percentage as i32)}
                                                </div>
                                            }
                                        } else {
                                            view! { <div></div> }
                                        }}

                                        {/* Next Test Indicator */}
                                        {if is_next {
                                            view! {
                                                <div class="absolute -top-8 left-1/2 transform -translate-x-1/2">
                                                    <div class="bg-blue-500 text-white px-3 py-1 rounded-full text-xs font-bold animate-bounce shadow-lg">
                                                        "Next"
                                                    </div>
                                                </div>
                                            }
                                        } else {
                                            view! { <div></div> }
                                        }}

                                        {/* Most Recent Completed Indicator */}
                                        {if is_most_recent && is_completed {
                                            view! {
                                                <div class="absolute -top-8 left-1/2 transform -translate-x-1/2">
                                                    <div class="bg-emerald-500 text-white px-3 py-1 rounded-full text-xs font-bold shadow-lg">
                                                        "Latest"
                                                    </div>
                                                </div>
                                            }
                                        } else {
                                            view! { <div></div> }
                                        }}
                                    </div>

                                    {/* Test Name */}
                                    <div class="text-center max-w-20">
                                        <div class="text-sm font-medium text-gray-900 truncate" title=test_name.clone()>
                                            {if test_name.len() > 8 {
                                                format!("{}...", &test_name[..8])
                                            } else {
                                                test_name.clone()
                                            }}
                                        </div>
                                        {if is_completed {
                                            view! {
                                                <div class="text-xs text-gray-500">
                                                    {latest_attempt.score} "/" {latest_attempt.total_possible}
                                                </div>
                                            }
                                        } else if is_next {
                                            view! {
                                                <div class="text-xs text-blue-600 font-medium">
                                                    "Up Next"
                                                </div>
                                            }
                                        } else {
                                            view! {
                                                <div class="text-xs text-gray-500">
                                                    "Available"
                                                </div>
                                            }
                                        }}
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </div>
            </div>

            {/* Test Details Panel */}
            <TestDetailsPanel
                selected_test={selected_node.get()}
                test_groups={grouped_tests.get()}
            />
        </div>
    }
}

// Rest of the component remains the same...
#[component]
pub fn TestDetailsPanel(
    selected_test: Option<String>,
    test_groups: Vec<(String, Vec<TestDetail>)>,
) -> impl IntoView {
    view! {
        {move || {
            if let Some(test_name) = &selected_test {
                if let Some((_, attempts)) = test_groups.iter().find(|(name, _)| name == test_name) {
                    let latest_attempt = attempts.iter().max_by_key(|t| t.attempt).unwrap();
                    let is_completed = latest_attempt.score > 0;

                    view! {
                        <div class="mt-8 bg-gray-50 border border-gray-200 rounded-xl p-6 transition-all duration-300 ease-in-out">
                            {/* Header */}
                            <div class="flex items-center justify-between mb-6">
                                <div>
                                    <h4 class="text-xl font-semibold text-gray-900">{test_name.clone()}</h4>
                                    <p class="text-sm text-gray-600">{latest_attempt.test_area.clone()}</p>
                                </div>
                                <div class="flex items-center space-x-4">
                                    {if attempts.len() > 1 {
                                        view! {
                                            <div class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-blue-100 text-blue-800">
                                                {attempts.len()} " attempts"
                                            </div>
                                        }
                                    } else {
                                        view! { <div></div> }
                                    }}

                                    {if is_completed {
                                        view! {
                                            <div class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-emerald-100 text-emerald-800">
                                                "Completed"
                                            </div>
                                        }
                                    } else {
                                        view! {
                                            <div class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-gray-100 text-gray-800">
                                                "Not Started"
                                            </div>
                                        }
                                    }}
                                </div>
                            </div>

                            {/* Attempts Grid */}
                            <div class="grid gap-4">
                                {attempts.iter().enumerate().map(|(index, attempt)| {
                                    let is_latest = index == attempts.len() - 1;
                                    let attempt_completed = attempt.score > 0;
                                    let score_percentage = if attempt.total_possible > 0 {
                                        (attempt.score as f32 / attempt.total_possible as f32) * 100.0
                                    } else {
                                        0.0
                                    };

                                    view! {
                                        <div class=move || {
                                            let mut card_classes = "p-4 rounded-lg border-2 transition-all duration-200".to_string();
                                            if is_latest && attempt_completed {
                                                card_classes.push_str(" border-blue-200 bg-blue-50");
                                            } else if attempt_completed {
                                                card_classes.push_str(" border-gray-200 bg-white");
                                            } else {
                                                card_classes.push_str(" border-dashed border-gray-300 bg-gray-50");
                                            }
                                            card_classes
                                        }>
                                            {/* Rest of the TestDetailsPanel implementation remains the same */}
                                            <div class="flex items-center justify-between">
                                                <div class="flex items-center space-x-4">
                                                    <div class=move || {
                                                        if attempt_completed {
                                                            if score_percentage >= 80.0 {
                                                                "flex items-center justify-center w-10 h-10 bg-emerald-100 text-emerald-600 rounded-full"
                                                            } else if score_percentage >= 60.0 {
                                                                "flex items-center justify-center w-10 h-10 bg-amber-100 text-amber-600 rounded-full"
                                                            } else {
                                                                "flex items-center justify-center w-10 h-10 bg-red-100 text-red-600 rounded-full"
                                                            }
                                                        } else {
                                                            "flex items-center justify-center w-10 h-10 bg-gray-100 text-gray-500 rounded-full"
                                                        }
                                                    }>
                                                        {if attempt_completed {
                                                            view! {
                                                                <div class="w-5 h-5">
                                                                    <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                                                                        <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                                                                    </svg>
                                                                </div>
                                                            }
                                                        } else {
                                                            view! {
                                                                <div class="w-5 h-5 flex items-center justify-center">
                                                                    <span class="text-sm font-medium">{(index + 1).to_string()}</span>
                                                                </div>
                                                            }
                                                        }}
                                                    </div>

                                                    <div>
                                                        <div class="flex items-center space-x-2">
                                                            <span class="font-medium text-gray-900">
                                                                "Attempt " {attempt.attempt}
                                                            </span>
                                                            {if is_latest {
                                                                view! {
                                                                    <div class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                                                                        "Latest"
                                                                    </div>
                                                                }
                                                            } else {
                                                                view! { <div></div> }
                                                            }}
                                                        </div>
                                                        {if attempt_completed {
                                                            view! {
                                                                <div class="text-sm text-gray-600">
                                                                    "Completed on " {attempt.date_administered.format("%B %d, %Y").to_string()}
                                                                </div>
                                                            }
                                                        } else {
                                                            view! {
                                                                <div class="text-sm text-gray-500">
                                                                    "Not yet started"
                                                                </div>
                                                            }
                                                        }}
                                                    </div>
                                                </div>

                                                {if attempt_completed {
                                                    let attempt_clone = attempt.clone();
                                                    view! {
                                                        <div class="text-right">
                                                            <div class="flex items-center space-x-4">
                                                                <div>
                                                                    <div class="text-2xl font-bold text-gray-900">
                                                                        {attempt_clone.score} "/" {attempt_clone.total_possible}
                                                                    </div>
                                                                    <div class="text-sm text-gray-500">Score</div>
                                                                </div>
                                                                <div class=move || {
                                                                    let mut score_classes = "text-2xl font-bold".to_string();
                                                                    if score_percentage >= 80.0 {
                                                                        score_classes.push_str(" text-emerald-600");
                                                                    } else if score_percentage >= 60.0 {
                                                                        score_classes.push_str(" text-amber-600");
                                                                    } else {
                                                                        score_classes.push_str(" text-red-600");
                                                                    }
                                                                    score_classes
                                                                }>
                                                                    {format!("{}%", score_percentage as i32)}
                                                                </div>
                                                            </div>
                                                            <div class="mt-2">
                                                                {
                                                                    let performance_class = attempt_clone.performance_class.clone();
                                                                    let performance_class_for_text = performance_class.clone();
                                                                    view! {
                                                                        <span class=move || {
                                                                            let mut performance_classes = "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium".to_string();
                                                                            if performance_class.contains("Above") || performance_class.contains("High") {
                                                                                performance_classes.push_str(" bg-emerald-100 text-emerald-800");
                                                                            } else if performance_class.contains("Average") || performance_class.contains("On Track") {
                                                                                performance_classes.push_str(" bg-blue-100 text-blue-800");
                                                                            } else if performance_class.contains("Below") || performance_class.contains("Risk") {
                                                                                performance_classes.push_str(" bg-red-100 text-red-800");
                                                                            } else {
                                                                                performance_classes.push_str(" bg-gray-100 text-gray-800");
                                                                            }
                                                                            performance_classes
                                                                        }>
                                                                            {performance_class_for_text}
                                                                        </span>
                                                                    }
                                                                }
                                                            </div>
                                                        </div>
                                                    }
                                                } else {
                                                    view! {
                                                        <div class="flex space-x-2">
                                                            <button class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium text-sm">
                                                                "Start Test"
                                                            </button>
                                                        </div>
                                                    }
                                                }}
                                            </div>

                                            {/* Progress bar for completed attempts */}
                                            {if attempt_completed {
                                                view! {
                                                    <div class="mt-3">
                                                        <div class="w-full bg-gray-200 rounded-full h-2">
                                                            <div
                                                                class=move || {
                                                                    if score_percentage >= 80.0 {
                                                                        "bg-gradient-to-r from-emerald-500 to-emerald-600 h-2 rounded-full transition-all duration-1000"
                                                                    } else if score_percentage >= 60.0 {
                                                                        "bg-gradient-to-r from-amber-500 to-amber-600 h-2 rounded-full transition-all duration-1000"
                                                                    } else {
                                                                        "bg-gradient-to-r from-red-500 to-red-600 h-2 rounded-full transition-all duration-1000"
                                                                    }
                                                                }
                                                                style=format!("width: {}%", score_percentage)
                                                            ></div>
                                                        </div>
                                                    </div>
                                                }
                                            } else {
                                                view! { <div></div> }
                                            }}
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>

                            {/* Action Buttons */}
                            <div class="mt-6 flex justify-between items-center">
                                <div class="flex space-x-3">
                                    {if is_completed {
                                        view! {
                                            <div class="flex space-x-3">
                                                <button class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium text-sm">
                                                    "Retake Test"
                                                </button>
                                                <button class="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors font-medium text-sm">
                                                    "View Details"
                                                </button>
                                            </div>
                                        }
                                    } else {
                                        view! {
                                            <div class="flex space-x-3">
                                                <button class="px-4 py-2 bg-emerald-600 text-white rounded-lg hover:bg-emerald-700 transition-colors font-medium text-sm">
                                                    "Start Test"
                                                </button>
                                            </div>
                                        }
                                    }}
                                </div>

                                {/* Quick Stats */}
                                {if is_completed && attempts.len() > 1 {
                                    let best_score = attempts.iter()
                                        .filter(|a| a.score > 0)
                                        .map(|a| (a.score as f32 / a.total_possible as f32) * 100.0)
                                        .fold(0.0, f32::max);

                                    view! {
                                        <div class="text-right">
                                            <div class="text-sm text-gray-600">Best Score</div>
                                            <div class=move || {
                                                if best_score >= 80.0 {
                                                    "text-lg font-bold text-emerald-600"
                                                } else if best_score >= 60.0 {
                                                    "text-lg font-bold text-amber-600"
                                                } else {
                                                    "text-lg font-bold text-red-600"
                                                }
                                            }>
                                                {format!("{}%", best_score as i32)}
                                            </div>
                                        </div>
                                    }
                                } else {
                                    view! { <div></div> }
                                }}
                            </div>
                        </div>
                    }
                } else {
                    view! { <div></div> }
                }
            } else {
                view! { <div></div> }
            }
        }}
    }
}

// Legacy component aliases for backward compatibility
#[component]
pub fn EnhancedNodalSequence(
    assessment_name: String,
    test_details: Vec<TestDetail>,
    show_actions: bool,
) -> impl IntoView {
    // Convert to new format for compatibility
    let assessment = AssessmentSummary {
        assessment_id: "legacy".to_string(),
        assessment_name: assessment_name.clone(),
        subject: "Unknown".to_string(),
        grade_level: None,
        current_score: test_details.iter().map(|t| t.score).sum(),
        total_possible: Some(test_details.iter().map(|t| t.total_possible).sum()),
        progress: if test_details.iter().all(|t| t.score > 0) {
            Progress::Completed
        } else if test_details.iter().any(|t| t.score > 0) {
            Progress::Ongoing
        } else {
            Progress::NotStarted
        },
        assessment_rating: "Unknown".to_string(),
        test_details: test_details.clone(),
        distribution_data: Vec::new(),
    };

    view! {
        <SequenceWeb assessment={assessment} test_details={test_details} />
    }
}

#[component]
pub fn CompactNodalSequence(
    assessment_name: String,
    test_details: Vec<TestDetail>,
) -> impl IntoView {
    // Convert to new format for compatibility
    let assessment = AssessmentSummary {
        assessment_id: "legacy".to_string(),
        assessment_name: assessment_name.clone(),
        subject: "Unknown".to_string(),
        grade_level: None,
        current_score: test_details.iter().map(|t| t.score).sum(),
        total_possible: Some(test_details.iter().map(|t| t.total_possible).sum()),
        progress: if test_details.iter().all(|t| t.score > 0) {
            Progress::Completed
        } else if test_details.iter().any(|t| t.score > 0) {
            Progress::Ongoing
        } else {
            Progress::NotStarted
        },
        assessment_rating: "Unknown".to_string(),
        test_details: test_details.clone(),
        distribution_data: Vec::new(),
    };

    view! {
        <SequenceWeb assessment={assessment} test_details={test_details} />
    }
}

#[component]
pub fn CompactSequenceWeb(
    assessment_name: String,
    test_details: Vec<TestDetail>,
    show_actions: bool,
) -> impl IntoView {
    // Convert to new format for compatibility
    let assessment = AssessmentSummary {
        assessment_id: "legacy".to_string(),
        assessment_name: assessment_name.clone(),
        subject: "Unknown".to_string(),
        grade_level: None,
        current_score: test_details.iter().map(|t| t.score).sum(),
        total_possible: Some(test_details.iter().map(|t| t.total_possible).sum()),
        progress: if test_details.iter().all(|t| t.score > 0) {
            Progress::Completed
        } else if test_details.iter().any(|t| t.score > 0) {
            Progress::Ongoing
        } else {
            Progress::NotStarted
        },
        assessment_rating: "Unknown".to_string(),
        test_details: test_details.clone(),
        distribution_data: Vec::new(),
    };

    view! {
        <SequenceWeb assessment={assessment} test_details={test_details} />
    }
}
