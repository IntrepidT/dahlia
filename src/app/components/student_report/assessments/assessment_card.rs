use crate::app::components::dashboard::color_utils::ColorUtils;
use crate::app::components::data_processing::{AssessmentSummary, Progress, TestHistoryEntry};
use crate::app::components::student_report::assessments::compact_assessment_dot_chart::CompactProgressChart; // Fixed import path
use crate::app::models::test::Test;
use leptos::prelude::*;
use leptos::prelude::*;

#[component]
pub fn AssessmentCard(
    assessment: AssessmentSummary,
    tests_resource: Resource<Option<Vec<Test>>>,
    on_expand: Callback<String>,
    #[prop(default = false)] is_expanded: bool,
    #[prop(optional)] test_history: Option<Vec<TestHistoryEntry>>,
    #[prop(default = 1)] student_id: i32,
) -> impl IntoView {
    let assessment_id = assessment.assessment_id.clone();
    let assessment_name = assessment.assessment_name.clone();
    let current_score = assessment.current_score;
    let total_possible = assessment.total_possible;
    let progress = assessment.progress.clone();
    let assessment_rating = assessment.assessment_rating.clone();
    let subject = assessment.subject.clone();
    let grade_level = assessment.grade_level.clone();

    // Calculate completion percentage
    let completion_percentage = if let Some(total) = total_possible {
        (current_score as f32 / total as f32 * 100.0).min(100.0)
    } else {
        0.0
    };

    // Get progress status styling
    let (progress_color, progress_bg, progress_text) = match progress {
        Progress::Completed => ("text-green-700", "bg-green-100", "Completed"),
        Progress::Ongoing => ("text-yellow-700", "bg-yellow-100", "In Progress"),
        Progress::NotStarted => ("text-gray-700", "bg-gray-100", "Not Started"),
    };

    // Get rating color styling
    let rating_color = if assessment_rating.contains("Above") || assessment_rating.contains("High")
    {
        "text-green-600"
    } else if assessment_rating.contains("Average") || assessment_rating.contains("On Track") {
        "text-blue-600"
    } else if assessment_rating.contains("Below") || assessment_rating.contains("Risk") {
        "text-red-600"
    } else {
        "text-gray-600"
    };

    view! {
        <div class="bg-white rounded-xl border border-gray-200 shadow-sm hover:shadow-md transition-shadow duration-200">
            // Header Section
            <div class="p-6 border-b border-gray-100">
                <div class="flex items-start justify-between">
                    <div class="flex-1">
                        <h3 class="text-lg font-semibold text-gray-900 mb-2">
                            {assessment_name}
                        </h3>
                        <div class="flex items-center gap-4 text-sm text-gray-600">
                            <span class="flex items-center gap-1">
                                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.746 0 3.332.477 4.5 1.253v13C19.832 18.477 18.246 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
                                </svg>
                                {subject}
                            </span>
                            {grade_level.as_ref().map(|grade| view! {
                                <span class="flex items-center gap-1">
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
                                    </svg>
                                    {format!("Grade {}", grade)}
                                </span>
                            })}
                        </div>
                    </div>
                    <div class="flex items-center gap-3">
                        <span class={format!("px-3 py-1 rounded-full text-xs font-medium {} {}", progress_color, progress_bg)}>
                            {progress_text}
                        </span>
                        <button
                            class="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-50 rounded-lg transition-colors"
                            on:click=move |_| on_expand.run(assessment_id.clone())
                        >
                            <svg
                                class={format!("w-5 h-5 transition-transform duration-200 {}", if is_expanded { "rotate-180" } else { "" })}
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                            </svg>
                        </button>
                    </div>
                </div>
            </div>

            {if !is_expanded && !assessment.test_details.is_empty() {
                view! {
                    <div class="px-6 py-4 border-b border-gray-100">
                        <CompactProgressChart
                            assessment={assessment.clone()}
                            test_details={assessment.test_details.clone()}
                            tests_resource={tests_resource}
                        />
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            // Content Section
            <div class="p-6">
                <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                    // Score Section
                    <div class="space-y-3">
                        <h4 class="text-sm font-medium text-gray-700">Score Progress</h4>
                        <div class="flex items-end gap-2">
                            <span class="text-2xl font-bold text-gray-900">
                                {current_score}
                            </span>
                            {total_possible.map(|total| view! {
                                <span class="text-lg text-gray-500 mb-1">
                                    / {total}
                                </span>
                            })}
                        </div>
                        {total_possible.map(|_| view! {
                            <div class="w-full bg-gray-200 rounded-full h-2">
                                <div
                                    class={
                                        if completion_percentage >= 80.0 {
                                            "bg-green-500 h-2 rounded-full transition-all duration-300"
                                        } else if completion_percentage >= 60.0 {
                                            "bg-yellow-500 h-2 rounded-full transition-all duration-300"
                                        } else {
                                            "bg-red-500 h-2 rounded-full transition-all duration-300"
                                        }
                                    }
                                    style=format!("width: {}%", completion_percentage)
                                ></div>
                            </div>
                        })}
                        <p class="text-xs text-gray-500">
                            {format!("{:.1}% Complete", completion_percentage)}
                        </p>
                    </div>

                    // Performance Rating Section
                    <div class="space-y-3">
                        <h4 class="text-sm font-medium text-gray-700">Performance Level</h4>
                        <div class={format!("text-xl font-semibold {}", rating_color)}>
                            {assessment_rating.clone()}
                        </div>
                        <div class="flex items-center gap-2">
                            {
                                if assessment_rating.contains("Above") || assessment_rating.contains("High") {
                                    view! {
                                        <div class="flex items-center gap-1 text-green-600">
                                            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                                                <path fill-rule="evenodd" d="M3.293 9.707a1 1 0 010-1.414l6-6a1 1 0 011.414 0l6 6a1 1 0 01-1.414 1.414L11 5.414V17a1 1 0 11-2 0V5.414L4.707 9.707a1 1 0 01-1.414 0z" clip-rule="evenodd" />
                                            </svg>
                                            <span class="text-sm">Above Average</span>
                                        </div>
                                    }.into_any()
                                } else if assessment_rating.contains("Below") || assessment_rating.contains("Risk") {
                                    view! {
                                        <div class="flex items-center gap-1 text-red-600">
                                            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                                                <path fill-rule="evenodd" d="M16.707 10.293a1 1 0 010 1.414l-6 6a1 1 0 01-1.414 0l-6-6a1 1 0 111.414-1.414L9 14.586V3a1 1 0 012 0v11.586l4.293-4.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                                            </svg>
                                            <span class="text-sm">Needs Support</span>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="flex items-center gap-1 text-blue-600">
                                            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                                                <path fill-rule="evenodd" d="M3 10a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z" clip-rule="evenodd" />
                                            </svg>
                                            <span class="text-sm">On Track</span>
                                        </div>
                                    }.into_any()
                                }
                            }
                        </div>
                    </div>

                    // Test Count Section
                    <div class="space-y-3">
                        <h4 class="text-sm font-medium text-gray-700">Tests Completed</h4>
                        <div class="flex items-center gap-2">
                            <span class="text-2xl font-bold text-gray-900">
                                {assessment.test_details.len()}
                            </span>
                            <span class="text-sm text-gray-500">tests</span>
                        </div>
                        <div class="flex items-center gap-2 text-sm text-gray-600">
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                            <span>
                                {
                                    let completed_count = assessment.test_details.iter()
                                        .filter(|test| test.score > 0)
                                        .count();
                                    format!("{} completed", completed_count)
                                }
                            </span>
                        </div>
                    </div>
                </div>

                // Expanded Details Section
                {if is_expanded {
                    view! {
                        <div class="mt-6 pt-6 border-t border-gray-100">
                            <h4 class="text-sm font-medium text-gray-700 mb-4">Test Details</h4>
                            <div class="space-y-3">
                                {assessment.test_details.iter().map(|test| {
                                    let score_percentage = if test.total_possible > 0 {
                                        (test.score as f32 / test.total_possible as f32) * 100.0
                                    } else {
                                        0.0
                                    };

                                    let (status_color, status_bg) = if test.score > 0 {
                                        if score_percentage >= 80.0 {
                                            ("text-green-700", "bg-green-100")
                                        } else if score_percentage >= 60.0 {
                                            ("text-yellow-700", "bg-yellow-100")
                                        } else {
                                            ("text-red-700", "bg-red-100")
                                        }
                                    } else {
                                        ("text-gray-700", "bg-gray-100")
                                    };

                                    view! {
                                        <div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                                            <div class="flex-1">
                                                <div class="flex items-center gap-3">
                                                    <div class="flex-1">
                                                        <h5 class="font-medium text-gray-900">{test.test_name.clone()}</h5>
                                                        <p class="text-sm text-gray-500">{test.test_area.clone()}</p>
                                                    </div>
                                                    <div class="text-right">
                                                        <div class="text-lg font-semibold text-gray-900">
                                                            {test.score} / {test.total_possible}
                                                        </div>
                                                        <div class={format!("text-sm font-medium {}", status_color)}>
                                                            {if test.score > 0 {
                                                                format!("{:.0}%", score_percentage)
                                                            } else {
                                                                "Not taken".to_string()
                                                            }}
                                                        </div>
                                                    </div>
                                                    <span class={format!("px-2 py-1 rounded-full text-xs font-medium {} {}", status_color, status_bg)}>
                                                        {test.performance_class.clone()}
                                                    </span>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_any()
                                }).collect::<Vec<_>>()}
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
