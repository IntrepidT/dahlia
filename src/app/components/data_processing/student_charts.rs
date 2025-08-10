use crate::app::components::data_processing::{AssessmentSummary, TestDetail, TestHistoryEntry};
use leptos::html;
use leptos::prelude::*;
use leptos::prelude::*;

// Helper function to wait for Chart.js to be available
#[cfg(feature = "hydrate")]
pub fn wait_for_chartjs<F>(callback: F)
where
    F: Fn() + 'static,
{
    use wasm_bindgen::prelude::*;

    let window = web_sys::window().unwrap();

    // Check if Chart is already available
    if js_sys::Reflect::has(&window, &"Chart".into()).unwrap_or(false) {
        callback();
        return;
    }

    // Listen for the chartjs-loaded event
    let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
        callback();
    }) as Box<dyn Fn(_)>);

    window
        .add_event_listener_with_callback("chartjs-loaded", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();
}

// Assessment Progress Overview Chart
#[component]
pub fn AssessmentProgressChart(
    assessments: Vec<AssessmentSummary>,
    chart_id: String,
) -> impl IntoView {
    let chart_ref = create_node_ref::<html::Canvas>();

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if let Some(canvas) = chart_ref.get() {
            let assessments = assessments.clone();

            wait_for_chartjs(move || {
                use gloo_utils::format::JsValueSerdeExt;
                use serde_json::json;
                use wasm_bindgen::prelude::*;

                let window = web_sys::window().unwrap();
                let chart_constructor = js_sys::Reflect::get(&window, &"Chart".into()).unwrap();

                let labels: Vec<String> = assessments
                    .iter()
                    .map(|a| a.assessment_name.clone())
                    .collect();

                let current_scores: Vec<i32> =
                    assessments.iter().map(|a| a.current_score).collect();

                let total_possibles: Vec<i32> = assessments
                    .iter()
                    .map(|a| a.total_possible.unwrap_or(0))
                    .collect();

                let config = json!({
                    "type": "bar",
                    "data": {
                        "labels": labels,
                        "datasets": [
                            {
                                "label": "Current Score",
                                "data": current_scores,
                                "backgroundColor": "rgba(59, 130, 246, 0.8)",
                                "borderColor": "rgba(59, 130, 246, 1)",
                                "borderWidth": 2,
                                "borderRadius": 6,
                            },
                            {
                                "label": "Total Possible",
                                "data": total_possibles,
                                "backgroundColor": "rgba(229, 231, 235, 0.8)",
                                "borderColor": "rgba(156, 163, 175, 1)",
                                "borderWidth": 1,
                                "borderRadius": 6,
                            }
                        ]
                    },
                    "options": {
                        "responsive": true,
                        "maintainAspectRatio": false,
                        "plugins": {
                            "title": {
                                "display": true,
                                "text": "Assessment Progress Overview",
                                "font": {
                                    "size": 16,
                                    "weight": "bold"
                                },
                                "color": "#1f2937"
                            },
                            "legend": {
                                "position": "top"
                            }
                        },
                        "scales": {
                            "y": {
                                "beginAtZero": true
                            }
                        }
                    }
                });

                let config_js = JsValue::from_serde(&config).unwrap();
                let canvas_element = canvas.unchecked_ref::<web_sys::HtmlCanvasElement>();
                let args = js_sys::Array::new();
                args.push(&JsValue::from(canvas_element));
                args.push(&config_js);

                let _ = js_sys::Reflect::construct(&chart_constructor.into(), &args);
            });
        }
    });

    view! {
        <div class="bg-white rounded-lg shadow-lg p-6 mb-6">
            <div class="h-80 relative">
                <canvas node_ref=chart_ref id=chart_id></canvas>
            </div>
        </div>
    }
}

// Performance Distribution Pie Chart
#[component]
pub fn PerformanceDistributionChart(
    distribution_data: Vec<(String, i32)>,
    chart_id: String,
    title: String,
) -> impl IntoView {
    let chart_ref = create_node_ref::<html::Canvas>();

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if let Some(canvas) = chart_ref.get() {
            let distribution_data = distribution_data.clone();
            let title = title.clone();

            wait_for_chartjs(move || {
                use gloo_utils::format::JsValueSerdeExt;
                use serde_json::json;
                use wasm_bindgen::prelude::*;

                let window = web_sys::window().unwrap();
                let chart_constructor = js_sys::Reflect::get(&window, &"Chart".into()).unwrap();

                let labels: Vec<String> = distribution_data
                    .iter()
                    .map(|(label, _)| label.clone())
                    .collect();

                let data: Vec<i32> = distribution_data.iter().map(|(_, count)| *count).collect();

                let colors: Vec<String> = labels
                    .iter()
                    .map(|label| {
                        if label.contains("Above")
                            || label.contains("High")
                            || label.contains("Excellent")
                        {
                            "rgba(34, 197, 94, 0.8)".to_string()
                        } else if label.contains("Average")
                            || label.contains("On Track")
                            || label.contains("Satisfactory")
                        {
                            "rgba(59, 130, 246, 0.8)".to_string()
                        } else if label.contains("Below")
                            || label.contains("Risk")
                            || label.contains("Needs")
                        {
                            "rgba(239, 68, 68, 0.8)".to_string()
                        } else {
                            "rgba(156, 163, 175, 0.8)".to_string()
                        }
                    })
                    .collect();

                let config = json!({
                    "type": "doughnut",
                    "data": {
                        "labels": labels,
                        "datasets": [{
                            "data": data,
                            "backgroundColor": colors,
                            "borderWidth": 2,
                            "hoverOffset": 8
                        }]
                    },
                    "options": {
                        "responsive": true,
                        "maintainAspectRatio": false,
                        "plugins": {
                            "title": {
                                "display": true,
                                "text": title,
                                "font": {
                                    "size": 14,
                                    "weight": "bold"
                                }
                            },
                            "legend": {
                                "position": "bottom"
                            }
                        },
                        "cutout": "60%"
                    }
                });

                let config_js = JsValue::from_serde(&config).unwrap();
                let canvas_element = canvas.unchecked_ref::<web_sys::HtmlCanvasElement>();
                let args = js_sys::Array::new();
                args.push(&JsValue::from(canvas_element));
                args.push(&config_js);

                let _ = js_sys::Reflect::construct(&chart_constructor.into(), &args);
            });
        }
    });

    view! {
        <div class="bg-white rounded-lg shadow-lg p-4">
            <div class="h-64 relative">
                <canvas node_ref=chart_ref id=chart_id></canvas>
            </div>
        </div>
    }
}

// Test Scores Timeline Chart
#[component]
pub fn TestScoresTimelineChart(
    test_history: Vec<TestHistoryEntry>,
    chart_id: String,
) -> impl IntoView {
    let chart_ref = create_node_ref::<html::Canvas>();

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if let Some(canvas) = chart_ref.get() {
            let test_history = test_history.clone();

            wait_for_chartjs(move || {
                use gloo_utils::format::JsValueSerdeExt;
                use serde_json::json;
                use wasm_bindgen::prelude::*;

                let window = web_sys::window().unwrap();
                let chart_constructor = js_sys::Reflect::get(&window, &"Chart".into()).unwrap();

                let mut sorted_history = test_history.clone();
                sorted_history.sort_by(|a, b| a.date_administered.cmp(&b.date_administered));

                let labels: Vec<String> = sorted_history
                    .iter()
                    .map(|entry| entry.date_administered.format("%Y-%m-%d").to_string())
                    .collect();

                let scores: Vec<f32> = sorted_history
                    .iter()
                    .map(|entry| (entry.score as f32 / entry.total_possible as f32) * 100.0)
                    .collect();

                let config = json!({
                    "type": "line",
                    "data": {
                        "labels": labels,
                        "datasets": [{
                            "label": "Score Percentage",
                            "data": scores,
                            "borderColor": "rgba(59, 130, 246, 1)",
                            "backgroundColor": "rgba(59, 130, 246, 0.1)",
                            "borderWidth": 3,
                            "pointRadius": 6,
                            "fill": true,
                            "tension": 0.4
                        }]
                    },
                    "options": {
                        "responsive": true,
                        "maintainAspectRatio": false,
                        "plugins": {
                            "title": {
                                "display": true,
                                "text": "Test Performance Timeline",
                                "font": {
                                    "size": 16,
                                    "weight": "bold"
                                }
                            }
                        },
                        "scales": {
                            "y": {
                                "beginAtZero": true,
                                "max": 100,
                                "title": {
                                    "display": true,
                                    "text": "Score Percentage"
                                }
                            },
                            "x": {
                                "title": {
                                    "display": true,
                                    "text": "Date"
                                }
                            }
                        }
                    }
                });

                let config_js = JsValue::from_serde(&config).unwrap();
                let canvas_element = canvas.unchecked_ref::<web_sys::HtmlCanvasElement>();
                let args = js_sys::Array::new();
                args.push(&JsValue::from(canvas_element));
                args.push(&config_js);

                let _ = js_sys::Reflect::construct(&chart_constructor.into(), &args);
            });
        }
    });

    view! {
        <div class="bg-white rounded-lg shadow-lg p-6 mb-6">
            <div class="h-80 relative">
                <canvas node_ref=chart_ref id=chart_id></canvas>
            </div>
        </div>
    }
}

// Radar Chart for Assessment Comparison
#[component]
pub fn AssessmentRadarChart(
    assessment_data: Vec<AssessmentSummary>,
    radar_chart_id: String,
) -> impl IntoView {
    let chart_ref = create_node_ref::<html::Canvas>();

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if let Some(canvas) = chart_ref.get() {
            let assessment_data = assessment_data.clone();

            wait_for_chartjs(move || {
                use gloo_utils::format::JsValueSerdeExt;
                use serde_json::json;
                use wasm_bindgen::prelude::*;

                let window = web_sys::window().unwrap();
                let chart_constructor = js_sys::Reflect::get(&window, &"Chart".into()).unwrap();

                let labels: Vec<String> = assessment_data
                    .iter()
                    .map(|a| a.assessment_name.clone())
                    .collect();

                let percentages: Vec<f32> = assessment_data
                    .iter()
                    .map(|a| {
                        if let Some(total) = a.total_possible {
                            if total > 0 {
                                (a.current_score as f32 / total as f32) * 100.0
                            } else {
                                0.0
                            }
                        } else {
                            0.0
                        }
                    })
                    .collect();

                let config = json!({
                    "type": "radar",
                    "data": {
                        "labels": labels,
                        "datasets": [{
                            "label": "Performance",
                            "data": percentages,
                            "borderColor": "rgba(59, 130, 246, 1)",
                            "backgroundColor": "rgba(59, 130, 246, 0.2)",
                            "borderWidth": 2,
                            "pointRadius": 5
                        }]
                    },
                    "options": {
                        "responsive": true,
                        "maintainAspectRatio": false,
                        "plugins": {
                            "title": {
                                "display": true,
                                "text": "Assessment Performance Comparison",
                                "font": {
                                    "size": 16,
                                    "weight": "bold"
                                }
                            }
                        },
                        "scales": {
                            "r": {
                                "beginAtZero": true,
                                "max": 100
                            }
                        }
                    }
                });

                let config_js = JsValue::from_serde(&config).unwrap();
                let canvas_element = canvas.unchecked_ref::<web_sys::HtmlCanvasElement>();
                let args = js_sys::Array::new();
                args.push(&JsValue::from(canvas_element));
                args.push(&config_js);

                let _ = js_sys::Reflect::construct(&chart_constructor.into(), &args);
            });
        }
    });

    view! {
        <div class="bg-white rounded-lg shadow-lg p-6 mb-6">
            <div class="h-80 relative">
                <canvas node_ref=chart_ref id=radar_chart_id></canvas>
            </div>
        </div>
    }
}

// Test Area Performance Horizontal Bar Chart
#[component]
pub fn TestAreaPerformanceChart(
    test_area_data: Vec<TestDetail>,
    area_chart_id: String,
) -> impl IntoView {
    let chart_ref = create_node_ref::<html::Canvas>();

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if let Some(canvas) = chart_ref.get() {
            let test_area_data = test_area_data.clone();

            wait_for_chartjs(move || {
                use gloo_utils::format::JsValueSerdeExt;
                use serde_json::json;
                use wasm_bindgen::prelude::*;

                let window = web_sys::window().unwrap();
                let chart_constructor = js_sys::Reflect::get(&window, &"Chart".into()).unwrap();

                // Group by test area and calculate averages
                let mut area_scores: std::collections::HashMap<String, Vec<f32>> =
                    std::collections::HashMap::new();

                for test in &test_area_data {
                    let percentage = (test.score as f32 / test.total_possible as f32) * 100.0;
                    area_scores
                        .entry(test.test_area.clone())
                        .or_insert_with(Vec::new)
                        .push(percentage);
                }

                let mut labels = Vec::new();
                let mut averages = Vec::new();
                let mut colors = Vec::new();

                for (area, scores) in area_scores {
                    let avg = scores.iter().sum::<f32>() / scores.len() as f32;
                    labels.push(area);
                    averages.push(avg);

                    if avg >= 80.0 {
                        colors.push("rgba(34, 197, 94, 0.8)".to_string());
                    } else if avg >= 60.0 {
                        colors.push("rgba(59, 130, 246, 0.8)".to_string());
                    } else {
                        colors.push("rgba(239, 68, 68, 0.8)".to_string());
                    }
                }

                let config = json!({
                    "type": "bar",
                    "data": {
                        "labels": labels,
                        "datasets": [{
                            "label": "Average Performance",
                            "data": averages,
                            "backgroundColor": colors,
                            "borderWidth": 2,
                            "borderRadius": 6
                        }]
                    },
                    "options": {
                        "indexAxis": "y",
                        "responsive": true,
                        "maintainAspectRatio": false,
                        "plugins": {
                            "title": {
                                "display": true,
                                "text": "Performance by Test Area",
                                "font": {
                                    "size": 16,
                                    "weight": "bold"
                                }
                            }
                        },
                        "scales": {
                            "x": {
                                "beginAtZero": true,
                                "max": 100
                            }
                        }
                    }
                });

                let config_js = JsValue::from_serde(&config).unwrap();
                let canvas_element = canvas.unchecked_ref::<web_sys::HtmlCanvasElement>();
                let args = js_sys::Array::new();
                args.push(&JsValue::from(canvas_element));
                args.push(&config_js);

                let _ = js_sys::Reflect::construct(&chart_constructor.into(), &args);
            });
        }
    });

    view! {
        <div class="bg-white rounded-lg shadow-lg p-6 mb-6">
            <div class="h-80 relative">
                <canvas node_ref=chart_ref id=area_chart_id></canvas>
            </div>
        </div>
    }
}
