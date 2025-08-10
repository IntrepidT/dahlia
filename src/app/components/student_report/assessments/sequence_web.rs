use crate::app::components::data_processing::{AssessmentSummary, Progress, TestDetail};
use leptos::prelude::*;
use leptos::prelude::*;
use std::collections::HashMap;

#[component]
pub fn AssessmentProgressChart(
    assessment: AssessmentSummary,
    test_details: Vec<TestDetail>,
    #[prop(default = 200)] height: u32,
    #[prop(default = false)] show_legend: bool,
) -> impl IntoView {
    // Group tests by test name to handle multiple attempts
    let test_groups = Memo::new(move |_| {
        let mut groups: HashMap<String, Vec<TestDetail>> = HashMap::new();

        for test in test_details.iter() {
            groups
                .entry(test.test_name.clone())
                .or_insert_with(Vec::new)
                .push(test.clone());
        }

        // Sort groups by date and attempts
        let mut sorted_groups: Vec<(String, Vec<TestDetail>)> = groups.into_iter().collect();
        sorted_groups.sort_by_key(|(_, tests)| {
            tests
                .iter()
                .map(|t| t.date_administered)
                .min()
                .unwrap_or_default()
        });

        for (_, tests) in sorted_groups.iter_mut() {
            tests.sort_by_key(|t| (t.date_administered, t.attempt));
        }

        sorted_groups
    });

    // Calculate chart data points
    let chart_data = Memo::new(move |_| {
        let groups = test_groups.get();
        let mut data_points = Vec::new();
        let mut cumulative_score = 0;
        let mut cumulative_possible = 0;
        let mut x_position = 0;

        for (test_name, attempts) in groups.iter() {
            for (attempt_index, attempt) in attempts.iter().enumerate() {
                if attempt.score > 0 {
                    cumulative_score += attempt.score;
                    cumulative_possible += attempt.total_possible;

                    let percentage = if cumulative_possible > 0 {
                        (cumulative_score as f32 / cumulative_possible as f32) * 100.0
                    } else {
                        0.0
                    };

                    let attempt_percentage = if attempt.total_possible > 0 {
                        (attempt.score as f32 / attempt.total_possible as f32) * 100.0
                    } else {
                        0.0
                    };

                    data_points.push(ChartPoint {
                        x: x_position,
                        y: percentage,
                        test_name: test_name.clone(),
                        attempt_number: attempt.attempt,
                        score: attempt.score,
                        total_possible: attempt.total_possible,
                        attempt_percentage,
                        date: attempt.date_administered.date_naive(),
                        is_retry: attempt_index > 0,
                    });

                    x_position += 1;
                }
            }
        }

        data_points
    });

    // Calculate chart dimensions and scales
    let chart_bounds = Memo::new(move |_| {
        let data = chart_data.get();
        let max_x = data.len().max(1);
        let max_y = 100.0; // Always 0-100%

        ChartBounds {
            width: 400.0,
            height: height as f32,
            padding: 40.0,
            max_x: max_x as f32,
            max_y,
        }
    });

    view! {
        <div class="assessment-progress-chart bg-white rounded-lg border border-gray-200 p-4">
            {if show_legend {
                view! {
                    <div class="mb-4">
                        <h4 class="text-sm font-medium text-gray-700 mb-2">Assessment Progress Over Time</h4>
                        <div class="flex items-center gap-4 text-xs text-gray-600">
                            <div class="flex items-center gap-1">
                                <div class="w-3 h-0.5 bg-blue-500"></div>
                                <span>Cumulative Score</span>
                            </div>
                            <div class="flex items-center gap-1">
                                <div class="w-2 h-2 bg-blue-500 rounded-full"></div>
                                <span>Test Attempt</span>
                            </div>
                            <div class="flex items-center gap-1">
                                <div class="w-2 h-2 bg-amber-500 rounded-full"></div>
                                <span>Retry</span>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            <div class="relative">
                <svg
                    width={move || chart_bounds.get().width + chart_bounds.get().padding * 2.0}
                    height={move || chart_bounds.get().height + chart_bounds.get().padding * 2.0}
                    class="overflow-visible"
                >
                    // Background grid
                    <g class="grid" stroke="#f3f4f6" stroke-width="1" opacity="0.5">
                        // Horizontal grid lines
                        {move || {
                            let bounds = chart_bounds.get();
                            (0..=4).map(|i| {
                                let y = bounds.padding + (i as f32 * bounds.height / 4.0);
                                view! {
                                    <line
                                        x1={bounds.padding}
                                        y1={y}
                                        x2={bounds.padding + bounds.width}
                                        y2={y}
                                    />
                                }
                            }).collect::<Vec<_>>()
                        }}

                        // Vertical grid lines
                        {move || {
                            let bounds = chart_bounds.get();
                            let data = chart_data.get();
                            if data.is_empty() { return vec![]; }

                            (0..=data.len()).map(|i| {
                                let x = bounds.padding + (i as f32 * bounds.width / data.len().max(1) as f32);
                                view! {
                                    <line
                                        x1={x}
                                        y1={bounds.padding}
                                        x2={x}
                                        y2={bounds.padding + bounds.height}
                                    />
                                }
                            }).collect::<Vec<_>>()
                        }}
                    </g>

                    // Y-axis labels
                    <g class="y-axis-labels" fill="#6b7280" font-size="10" text-anchor="end">
                        {move || {
                            let bounds = chart_bounds.get();
                            (0..=4).map(|i| {
                                let y = bounds.padding + (i as f32 * bounds.height / 4.0) + 3.0;
                                let value = 100.0 - (i as f32 * 25.0);
                                view! {
                                    <text x={bounds.padding - 5.0} y={y}>
                                        {format!("{}%", value as i32)}
                                    </text>
                                }
                            }).collect::<Vec<_>>()
                        }}
                    </g>

                    // Progress line
                    {move || {
                        let data = chart_data.get();
                        let bounds = chart_bounds.get();

                        if data.len() < 2 {
                            return view! { <g></g> }.into_any();
                        }

                        let path_data = data.iter().enumerate().map(|(i, point)| {
                            let x = bounds.padding + (i as f32 * bounds.width / (data.len() - 1).max(1) as f32);
                            let y = bounds.padding + bounds.height - (point.y / 100.0 * bounds.height);

                            if i == 0 {
                                format!("M {} {}", x, y)
                            } else {
                                format!(" L {} {}", x, y)
                            }
                        }).collect::<String>();

                        view! {
                            <g>
                                <path
                                    d={path_data}
                                    stroke="#3b82f6"
                                    stroke-width="2"
                                    fill="none"
                                    class="transition-all duration-300"
                                />
                            </g>
                        }.into_any()
                    }}

                    // Data points
                    {move || {
                        let data = chart_data.get();
                        let bounds = chart_bounds.get();

                        data.iter().enumerate().map(|(i, point)| {
                            let x = bounds.padding + (i as f32 * bounds.width / data.len().max(1) as f32);
                            let y = bounds.padding + bounds.height - (point.y / 100.0 * bounds.height);

                            let (fill_color, stroke_color) = if point.is_retry {
                                ("#f59e0b", "#d97706") // Amber for retries
                            } else {
                                ("#3b82f6", "#2563eb") // Blue for first attempts
                            };

                            view! {
                                <g class="data-point group cursor-pointer">
                                    <circle
                                        cx={x}
                                        cy={y}
                                        r="4"
                                        fill={fill_color}
                                        stroke={stroke_color}
                                        stroke-width="2"
                                        class="transition-all duration-200 group-hover:r-6 group-hover:stroke-width-3"
                                    />

                                    // Tooltip (hidden by default, shown on hover)
                                    <g class="tooltip opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none">
                                        <rect
                                            x={x - 40.0}
                                            y={y - 35.0}
                                            width="80"
                                            height="25"
                                            fill="#1f2937"
                                            rx="4"
                                            class="drop-shadow-lg"
                                        />
                                        <text
                                            x={x}
                                            y={y - 22.0}
                                            text-anchor="middle"
                                            fill="white"
                                            font-size="10"
                                            font-weight="600"
                                        >
                                            {point.test_name.chars().take(8).collect::<String>()}
                                            {if point.test_name.len() > 8 { "..." } else { "" }}
                                        </text>
                                        <text
                                            x={x}
                                            y={y - 12.0}
                                            text-anchor="middle"
                                            fill="white"
                                            font-size="9"
                                        >
                                            {format!("{}% ({}/{})", point.attempt_percentage as i32, point.score, point.total_possible)}
                                        </text>
                                    </g>
                                </g>
                            }
                        }).collect::<Vec<_>>()
                    }}

                    // X-axis
                    <line
                        x1={move || chart_bounds.get().padding}
                        y1={move || chart_bounds.get().padding + chart_bounds.get().height}
                        x2={move || chart_bounds.get().padding + chart_bounds.get().width}
                        y2={move || chart_bounds.get().padding + chart_bounds.get().height}
                        stroke="#6b7280"
                        stroke-width="1"
                    />

                    // Y-axis
                    <line
                        x1={move || chart_bounds.get().padding}
                        y1={move || chart_bounds.get().padding}
                        x2={move || chart_bounds.get().padding}
                        y2={move || chart_bounds.get().padding + chart_bounds.get().height}
                        stroke="#6b7280"
                        stroke-width="1"
                    />
                </svg>

                // Progress statistics overlay
                <div class="absolute top-2 right-2 bg-white/90 backdrop-blur-sm rounded-lg px-3 py-2 text-xs">
                    <div class="space-y-1">
                        <div class="flex justify-between gap-4">
                            <span class="text-gray-600">Tests:</span>
                            <span class="font-medium">{move || test_groups.get().len()}</span>
                        </div>
                        <div class="flex justify-between gap-4">
                            <span class="text-gray-600">Attempts:</span>
                            <span class="font-medium">{move || chart_data.get().len()}</span>
                        </div>
                        <div class="flex justify-between gap-4">
                            <span class="text-gray-600">Avg Score:</span>
                            <span class="font-medium text-blue-600">
                                {move || {
                                    let data = chart_data.get();
                                    if data.is_empty() {
                                        "0%".to_string()
                                    } else {
                                        let avg = data.iter().map(|p| p.attempt_percentage).sum::<f32>() / data.len() as f32;
                                        format!("{}%", avg as i32)
                                    }
                                }}
                            </span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ChartPoint {
    x: usize,
    y: f32, // Cumulative percentage
    test_name: String,
    attempt_number: i32,
    score: i32,
    total_possible: i32,
    attempt_percentage: f32,
    date: chrono::NaiveDate,
    is_retry: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct ChartBounds {
    width: f32,
    height: f32,
    padding: f32,
    max_x: f32,
    max_y: f32,
}

// Compact version for embedding in assessment cards
#[component]
pub fn CompactProgressChart(
    assessment: AssessmentSummary,
    test_details: Vec<TestDetail>,
) -> impl IntoView {
    view! {
        <AssessmentProgressChart
            assessment={assessment}
            test_details={test_details}
            height={120}
            show_legend={false}
        />
    }
}

// Enhanced version with more details
#[component]
pub fn DetailedProgressChart(
    assessment: AssessmentSummary,
    test_details: Vec<TestDetail>,
) -> impl IntoView {
    view! {
        <AssessmentProgressChart
            assessment={assessment}
            test_details={test_details}
            height={300}
            show_legend={true}
        />
    }
}

// Legacy component aliases for backward compatibility with existing code
#[component]
pub fn SequenceWeb(assessment: AssessmentSummary, test_details: Vec<TestDetail>) -> impl IntoView {
    view! {
        <DetailedProgressChart
            assessment={assessment}
            test_details={test_details}
        />
    }
}

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
        <DetailedProgressChart
            assessment={assessment}
            test_details={test_details}
        />
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
        <CompactProgressChart
            assessment={assessment}
            test_details={test_details}
        />
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
        <CompactProgressChart
            assessment={assessment}
            test_details={test_details}
        />
    }
}
