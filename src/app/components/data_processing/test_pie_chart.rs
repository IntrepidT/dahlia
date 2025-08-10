#[cfg(feature = "hydrate")]
use crate::app::components::data_processing::student_charts::wait_for_chartjs;
use crate::app::models::{
    score::Score,
    test::{BenchmarkCategory, Test},
};
use leptos::html;
use leptos::prelude::*;
use leptos::prelude::*;
use std::rc::Rc;

#[component]
pub fn PieChart(
    score: Score,
    test: Test,
    #[prop(default = "pie-chart".to_string())] chart_id: String,
    #[prop(default = "Test Score Breakdown".to_string())] title: String,
) -> impl IntoView {
    let chart_ref = create_node_ref::<html::Canvas>();

    // Wrap in Rc for shared ownership
    let score_rc = Rc::new(score);
    let test_rc = Rc::new(test);
    let title_rc = Rc::new(title);

    // Clone Rc references for use in different closures
    let score_for_effect = Rc::clone(&score_rc);
    let test_for_effect = Rc::clone(&test_rc);
    let title_for_effect = Rc::clone(&title_rc);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if let Some(canvas) = chart_ref.get() {
            let current_score = Rc::clone(&score_for_effect);
            let current_test = Rc::clone(&test_for_effect);
            let title_clone = Rc::clone(&title_for_effect);

            wait_for_chartjs(move || {
                use gloo_utils::format::JsValueSerdeExt;
                use serde_json::json;
                use wasm_bindgen::prelude::*;

                let window = web_sys::window().unwrap();
                let chart_constructor = js_sys::Reflect::get(&window, &"Chart".into()).unwrap();

                let total_score: i32 = current_score.test_scores.iter().sum();

                // Get benchmark categories from the test
                let benchmark_categories = current_test
                    .benchmark_categories
                    .as_ref()
                    .cloned()
                    .unwrap_or_default();

                // Find which benchmark category this score falls into
                let current_benchmark =
                    Score::find_benchmark_category(total_score, &benchmark_categories);

                let max_possible = current_test.score;
                let remaining = (max_possible - total_score).max(0);

                // Get the color from the benchmark category the score falls into
                let earned_color = if let Some(ref current_cat) = current_benchmark {
                    current_cat.get_color()
                } else {
                    // Fallback color if no benchmark found
                    "#6b7280".to_string()
                };

                let (labels, data, colors) = if remaining > 0 {
                    (
                        vec!["Earned Points".to_string(), "Remaining Points".to_string()],
                        vec![total_score, remaining],
                        vec![
                            earned_color,
                            "rgba(229, 231, 235, 0.8)".to_string(), // Light gray for remaining
                        ],
                    )
                } else {
                    // Perfect score case
                    (
                        vec!["Perfect Score".to_string()],
                        vec![total_score],
                        vec![earned_color],
                    )
                };

                let config = json!({
                    "type": "pie",
                    "data": {
                        "labels": labels,
                        "datasets": [{
                            "data": data,
                            "backgroundColor": colors,
                            "borderWidth": 2,
                            "borderColor": "#ffffff",
                            "hoverOffset": 8
                        }]
                    },
                    "options": {
                        "responsive": true,
                        "maintainAspectRatio": false,
                        "plugins": {
                            "title": {
                                "display": true,
                                "text": title_clone.as_ref(),
                                "font": {
                                    "size": 16,
                                    "weight": "bold"
                                },
                                "color": "#1f2937"
                            },
                            "legend": {
                                "position": "bottom",
                                "labels": {
                                    "padding": 20,
                                    "usePointStyle": true
                                }
                            },
                            "tooltip": {
                                "callbacks": {
                                    "label": "(context) => {
                                        const label = context.label || '';
                                        const value = context.raw;
                                        const total = context.dataset.data.reduce((a, b) => a + b, 0);
                                        const percentage = ((value / total) * 100).toFixed(1);
                                        return `${label}: ${value} (${percentage}%)`;
                                    }"
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

    // Clone Rc references for use in the view
    let score_for_view = Rc::clone(&score_rc);
    let test_for_view = Rc::clone(&test_rc);

    view! {
        <div class="bg-white rounded-lg shadow-lg p-6">
            // Collapsible benchmark dropdown
            {
                let total_score: i32 = score_for_view.test_scores.iter().sum();

                if let Some(benchmark_categories) = &test_for_view.benchmark_categories {
                    if !benchmark_categories.is_empty() {
                        let current_benchmark = Score::find_benchmark_category(total_score, benchmark_categories);
                        let (is_open, set_is_open) = signal(false);

                        view! {
                            <div class="mb-4">
                                // Dropdown header (always visible)
                                <button
                                    class="w-full flex items-center justify-between p-3 bg-gray-50 hover:bg-gray-100 border border-gray-200 rounded-lg transition-colors duration-150 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    on:click=move |_| set_is_open.update(|open| *open = !*open)
                                >
                                    <div class="flex items-center space-x-3">
                                        {
                                            if let Some(current_benchmark) = &current_benchmark {
                                                view! {
                                                    <div
                                                        class="w-4 h-4 rounded-full border-2 border-white shadow-sm"
                                                        style=format!("background-color: {}", current_benchmark.get_color())
                                                    ></div>
                                                    <div class="text-left">
                                                        <div class="font-medium text-gray-900">
                                                            {current_benchmark.label.clone()}
                                                        </div>
                                                        <div class="text-sm text-gray-600">
                                                            "Score: " {total_score} " / " {test_for_view.score} " (" {current_benchmark.range_display()} " range)"
                                                        </div>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div class="text-left">
                                                        <div class="font-medium text-gray-900">No Benchmark Match</div>
                                                        <div class="text-sm text-gray-600">
                                                            "Score: " {total_score} " / " {test_for_view.score}
                                                        </div>
                                                    </div>
                                                }.into_any()
                                            }
                                        }
                                    </div>

                                    // Dropdown arrow
                                    <svg
                                        class=move || format!("w-5 h-5 text-gray-500 transition-transform duration-200 {}",
                                            if is_open.get() { "rotate-180" } else { "" })
                                        fill="none"
                                        stroke="currentColor"
                                        viewBox="0 0 24 24"
                                    >
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                                    </svg>
                                </button>

                                // Expandable content
                                <div class=move || format!("overflow-hidden transition-all duration-300 ease-in-out {}",
                                    if is_open.get() { "max-h-96 opacity-100" } else { "max-h-0 opacity-0" })
                                >
                                    <div class="mt-2 border border-gray-200 rounded-lg bg-white shadow-sm">
                                        <div class="p-3 border-b border-gray-200 bg-gray-50">
                                            <h4 class="font-medium text-gray-900 text-sm">All Benchmark Categories</h4>
                                        </div>
                                        <div class="divide-y divide-gray-100">
                                            {benchmark_categories.iter().map(|category| {
                                                let is_current = current_benchmark
                                                    .as_ref()
                                                    .map_or(false, |current_cat| current_cat.label == category.label);

                                                view! {
                                                    <div class=format!("p-3 flex items-center justify-between hover:bg-gray-50 {}",
                                                        if is_current { "bg-blue-50 border-l-4 border-blue-500" } else { "" })
                                                    >
                                                        <div class="flex items-center space-x-3">
                                                            <div
                                                                class="w-4 h-4 rounded-full border-2 border-white shadow-sm"
                                                                style=format!("background-color: {}", category.get_color())
                                                            ></div>
                                                            <div>
                                                                <div class=format!("font-medium {}",
                                                                    if is_current { "text-blue-900" } else { "text-gray-900" })
                                                                >
                                                                    {category.label.clone()}
                                                                </div>
                                                                <div class=format!("text-sm {}",
                                                                    if is_current { "text-blue-700" } else { "text-gray-600" })
                                                                >
                                                                    {category.range_display()} " points"
                                                                </div>
                                                            </div>
                                                        </div>
                                                        {
                                                            if is_current {
                                                                view! {
                                                                    <span class="px-2 py-1 bg-blue-100 text-blue-800 text-xs font-medium rounded-full">
                                                                        Current
                                                                    </span>
                                                                }.into_any()
                                                            } else {
                                                                view! { <div></div> }.into_any()
                                                            }
                                                        }
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                } else {
                    view! { <div></div> }.into_any()
                }
            }

            <div class="h-64 relative">
                <canvas node_ref=chart_ref id=chart_id></canvas>
            </div>
        </div>
    }
}
