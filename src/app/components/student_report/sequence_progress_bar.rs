use crate::app::components::data_processing::{AssessmentSummary, Progress, TestDetail};
use leptos::prelude::*;
use leptos::prelude::*;

#[component]
pub fn StripeProgressBar(
    assessment: AssessmentSummary,
    test_details: Vec<TestDetail>,
) -> impl IntoView {
    let total_tests = test_details.len();
    let completed_tests = test_details.iter().filter(|t| t.score > 0).count();
    let progress_percentage = if total_tests > 0 {
        (completed_tests as f32 / total_tests as f32) * 100.0
    } else {
        0.0
    };

    // Calculate average score for color determination
    let avg_score = if !test_details.is_empty() {
        let total_score: i32 = test_details.iter().map(|t| t.score).sum();
        let total_possible: i32 = test_details.iter().map(|t| t.total_possible).sum();
        if total_possible > 0 {
            (total_score as f32 / total_possible as f32) * 100.0
        } else {
            0.0
        }
    } else {
        0.0
    };

    view! {
        <div class="relative">
            // Main progress container with Stripe-inspired design
            <div class="bg-gradient-to-r from-slate-50 to-slate-100 rounded-2xl p-6 border border-slate-200 shadow-lg">
                // Header with assessment info
                <div class="flex items-center justify-between mb-6">
                    <div class="flex items-center space-x-3">
                        <div class=move || {
                            match assessment.progress {
                                Progress::Completed => "w-3 h-3 bg-emerald-500 rounded-full animate-pulse",
                                Progress::Ongoing => "w-3 h-3 bg-amber-500 rounded-full animate-pulse",
                                Progress::NotStarted => "w-3 h-3 bg-slate-400 rounded-full",
                            }
                        }></div>
                        <h3 class="text-xl font-semibold text-slate-800">{assessment.assessment_name.clone()}</h3>
                    </div>
                    <div class="flex items-center space-x-2">
                        <span class="text-sm text-slate-600">
                            {completed_tests} "/" {total_tests} " completed"
                        </span>
                        <div class=move || {
                            if avg_score >= 80.0 {
                                "px-3 py-1 bg-emerald-100 text-emerald-800 rounded-full text-sm font-medium"
                            } else if avg_score >= 60.0 {
                                "px-3 py-1 bg-amber-100 text-amber-800 rounded-full text-sm font-medium"
                            } else {
                                "px-3 py-1 bg-rose-100 text-rose-800 rounded-full text-sm font-medium"
                            }
                        }>
                            {format!("{:.1}%", avg_score)}
                        </div>
                    </div>
                </div>

                // Stripe-style progress track
                <div class="relative mb-6">
                    // Background track
                    <div class="h-2 bg-slate-200 rounded-full overflow-hidden">
                        // Animated gradient progress bar
                        <div
                            class=move || {
                                if avg_score >= 80.0 {
                                    "h-full bg-gradient-to-r from-emerald-400 to-emerald-600 rounded-full transition-all duration-1000 ease-out relative overflow-hidden"
                                } else if avg_score >= 60.0 {
                                    "h-full bg-gradient-to-r from-amber-400 to-amber-600 rounded-full transition-all duration-1000 ease-out relative overflow-hidden"
                                } else {
                                    "h-full bg-gradient-to-r from-rose-400 to-rose-600 rounded-full transition-all duration-1000 ease-out relative overflow-hidden"
                                }
                            }
                            style=format!("width: {}%", progress_percentage)
                        >
                            // Stripe-style shine effect
                            <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/30 to-transparent skew-x-12 animate-shine"></div>
                        </div>
                    </div>

                    // Progress percentage indicator
                    <div
                        class="absolute top-0 transform -translate-y-8 transition-all duration-500"
                        style=format!("left: {}%", progress_percentage.min(95.0))
                    >
                        <div class="relative">
                            <div class=move || {
                                if avg_score >= 80.0 {
                                    "bg-emerald-600 text-white px-2 py-1 rounded-md text-xs font-medium shadow-lg"
                                } else if avg_score >= 60.0 {
                                    "bg-amber-600 text-white px-2 py-1 rounded-md text-xs font-medium shadow-lg"
                                } else {
                                    "bg-rose-600 text-white px-2 py-1 rounded-md text-xs font-medium shadow-lg"
                                }
                            }>
                                {format!("{:.0}%", progress_percentage)}
                            </div>
                            // Arrow pointing down
                            <div class=move || {
                                if avg_score >= 80.0 {
                                    "absolute left-1/2 transform -translate-x-1/2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-l-transparent border-r-transparent border-t-emerald-600"
                                } else if avg_score >= 60.0 {
                                    "absolute left-1/2 transform -translate-x-1/2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-l-transparent border-r-transparent border-t-amber-600"
                                } else {
                                    "absolute left-1/2 transform -translate-x-1/2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-l-transparent border-r-transparent border-t-rose-600"
                                }
                            }></div>
                        </div>
                    </div>
                </div>

                // Test sequence visualization
                <div class="space-y-3">
                    <h4 class="text-sm font-medium text-slate-700 mb-3">"Test Sequence Progress"</h4>
                    <div class="flex flex-wrap gap-2">
                        {test_details.iter().enumerate().map(|(index, test)| {
                            let test_score_percent = (test.score as f32 / test.total_possible as f32) * 100.0;
                            let is_completed = test.score > 0;
                            let test_name = test.test_name.clone();
                            let test_area = test.test_area.clone();
                            let performance_class = test.performance_class.clone();

                            view! {
                                <div class="group relative">
                                    // Test node
                                    <div class=move || {
                                        let base_classes = "w-10 h-10 rounded-lg flex items-center justify-center text-sm font-bold transition-all duration-300 cursor-pointer border-2";
                                        if is_completed {
                                            if test_score_percent >= 80.0 {
                                                format!("{} bg-emerald-500 border-emerald-600 text-white shadow-lg group-hover:shadow-emerald-500/50 group-hover:scale-110", base_classes)
                                            } else if test_score_percent >= 60.0 {
                                                format!("{} bg-amber-500 border-amber-600 text-white shadow-lg group-hover:shadow-amber-500/50 group-hover:scale-110", base_classes)
                                            } else {
                                                format!("{} bg-rose-500 border-rose-600 text-white shadow-lg group-hover:shadow-rose-500/50 group-hover:scale-110", base_classes)
                                            }
                                        } else {
                                            format!("{} bg-slate-200 border-slate-300 text-slate-500 group-hover:bg-slate-300", base_classes)
                                        }
                                    }>
                                        {index + 1}
                                    </div>

                                    // Connecting line to next test
                                    {if index < test_details.len() - 1 {
                                        view! {
                                            <div class="absolute top-1/2 left-10 w-6 h-0.5 bg-slate-300 transform -translate-y-1/2 z-0"></div>
                                        }.into_any()
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }}

                                    // Tooltip on hover
                                    <div class="absolute bottom-12 left-1/2 transform -translate-x-1/2 bg-slate-900 text-white px-3 py-2 rounded-lg text-xs font-medium opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none whitespace-nowrap z-10">
                                        <div class="text-center">
                                            <div class="font-semibold">{test_name.clone()}</div>
                                            <div class="text-slate-300">{test_area.clone()}</div>
                                            {if is_completed {
                                                view! {
                                                    <div class="mt-1">
                                                        <div>{test.score} "/" {test.total_possible}</div>
                                                        <div class="text-xs">{performance_class.clone()}</div>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! { <div class="text-slate-300">"Not started"</div> }.into_any()
                                            }}
                                        </div>
                                        // Tooltip arrow
                                        <div class="absolute top-full left-1/2 transform -translate-x-1/2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-l-transparent border-r-transparent border-t-slate-900"></div>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                // Performance stats grid
                <div class="grid grid-cols-3 gap-4 mt-6 pt-6 border-t border-slate-200">
                    <div class="text-center">
                        <div class="text-2xl font-bold text-slate-800">{assessment.current_score}</div>
                        <div class="text-sm text-slate-600">"Current Score"</div>
                    </div>
                    <div class="text-center">
                        <div class="text-2xl font-bold text-slate-800">
                            {assessment.total_possible.map(|t| t.to_string()).unwrap_or_else(|| "N/A".to_string())}
                        </div>
                        <div class="text-sm text-slate-600">"Total Possible"</div>
                    </div>
                    <div class="text-center">
                        <div class=move || {
                            if avg_score >= 80.0 {
                                "text-2xl font-bold text-emerald-600"
                            } else if avg_score >= 60.0 {
                                "text-2xl font-bold text-amber-600"
                            } else {
                                "text-2xl font-bold text-rose-600"
                            }
                        }>
                            {assessment.assessment_rating.clone()}
                        </div>
                        <div class="text-sm text-slate-600">"Overall Rating"</div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn CompactStripeProgress(
    assessment_name: String,
    current_score: i32,
    total_possible: Option<i32>,
    test_details: Vec<TestDetail>,
) -> impl IntoView {
    let completed_tests = test_details.iter().filter(|t| t.score > 0).count();
    let total_tests = test_details.len();
    let progress_percentage = if total_tests > 0 {
        (completed_tests as f32 / total_tests as f32) * 100.0
    } else {
        0.0
    };

    view! {
        <div class="bg-white rounded-lg border border-slate-200 p-4 shadow-sm">
            <div class="flex items-center justify-between mb-3">
                <h4 class="font-semibold text-slate-800">{assessment_name}</h4>
                <div class="flex items-center space-x-2">
                    <span class="text-sm text-slate-600">
                        {completed_tests} "/" {total_tests}
                    </span>
                    <div class="text-sm font-medium text-slate-800">
                        {current_score}
                        {total_possible.map(|t| format!("/{}", t)).unwrap_or_else(|| String::new())}
                    </div>
                </div>
            </div>

            <div class="h-2 bg-slate-200 rounded-full overflow-hidden mb-2">
                <div
                    class="h-full bg-gradient-to-r from-blue-400 to-blue-600 rounded-full transition-all duration-1000 ease-out relative overflow-hidden"
                    style=format!("width: {}%", progress_percentage)
                >
                    <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/30 to-transparent skew-x-12 animate-shine"></div>
                </div>
            </div>

            <div class="flex space-x-1">
                {test_details.iter().enumerate().map(|(index, test)| {
                    let is_completed = test.score > 0;
                    let test_score_percent = if test.total_possible > 0 {
                        (test.score as f32 / test.total_possible as f32) * 100.0
                    } else {
                        0.0
                    };

                    view! {
                        <div class=move || {
                            let base_classes = "w-3 h-3 rounded-sm transition-all duration-300";
                            if is_completed {
                                if test_score_percent >= 80.0 {
                                    format!("{} bg-emerald-500", base_classes)
                                } else if test_score_percent >= 60.0 {
                                    format!("{} bg-amber-500", base_classes)
                                } else {
                                    format!("{} bg-rose-500", base_classes)
                                }
                            } else {
                                format!("{} bg-slate-200", base_classes)
                            }
                        }
                        title=format!("{}: {}/{}", test.test_name, test.score, test.total_possible)
                        ></div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
