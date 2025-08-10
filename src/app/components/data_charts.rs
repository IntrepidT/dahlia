use crate::app::models::score::Score;
use crate::app::models::test::Test;
use leptos::html;
use leptos::prelude::*;
use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use {gloo_utils::format::JsValueSerdeExt, wasm_bindgen::prelude::*, wasm_bindgen::JsValue};

// Import needed for Plotly
#[cfg(feature = "hydrate")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Plotly)]
    pub fn newPlot(div_id: &str, data: &JsValue, layout: &JsValue, config: &JsValue);
}

// Define color scheme
fn color_primary() -> &'static str {
    "rgb(255, 122, 89)"
} // Warm orange/coral
fn color_secondary() -> &'static str {
    "rgb(247, 148, 89)"
} // Light orange
fn color_tertiary() -> &'static str {
    "rgb(255, 172, 130)"
} // Pale orange
fn color_text() -> &'static str {
    "rgb(66, 71, 76)"
} // Dark slate for text
fn color_low() -> &'static str {
    "rgb(247, 114, 89)"
} // Red-orange for low scores
fn color_medium() -> &'static str {
    "rgb(255, 166, 77)"
} // Medium orange for mid scores
fn color_high() -> &'static str {
    "rgb(255, 145, 115)"
} // Orange for high scores

// Common chart configuration
#[cfg(feature = "hydrate")]
fn get_common_config() -> JsValue {
    let config = js_sys::Object::new();
    js_sys::Reflect::set(
        &config,
        &JsValue::from_str("displayModeBar"),
        &JsValue::from_bool(false),
    )
    .unwrap();
    js_sys::Reflect::set(
        &config,
        &JsValue::from_str("responsive"),
        &JsValue::from_bool(true),
    )
    .unwrap();
    config.into()
}

// Common chart layout settings
#[cfg(feature = "hydrate")]
fn apply_common_layout_settings(layout: &js_sys::Object) {
    // Set font family for the entire plot
    js_sys::Reflect::set(&layout, &JsValue::from_str("font"), &{
        let font = js_sys::Object::new();
        js_sys::Reflect::set(
            &font,
            &JsValue::from_str("family"),
            &JsValue::from_str("Inter, system-ui, sans-serif"),
        )
        .unwrap();
        js_sys::Reflect::set(
            &font,
            &JsValue::from_str("color"),
            &JsValue::from_str(color_text()),
        )
        .unwrap();
        font.into()
    })
    .unwrap();

    // Set plot background color
    js_sys::Reflect::set(
        &layout,
        &JsValue::from_str("paper_bgcolor"),
        &JsValue::from_str("rgba(0,0,0,0)"),
    )
    .unwrap();

    js_sys::Reflect::set(
        &layout,
        &JsValue::from_str("plot_bgcolor"),
        &JsValue::from_str("rgba(0,0,0,0)"),
    )
    .unwrap();

    // Add subtle grid lines
    js_sys::Reflect::set(&layout, &JsValue::from_str("xaxis"), &{
        let axis = js_sys::Object::new();
        js_sys::Reflect::set(
            &axis,
            &JsValue::from_str("showgrid"),
            &JsValue::from_bool(false),
        )
        .unwrap();
        js_sys::Reflect::set(
            &axis,
            &JsValue::from_str("zeroline"),
            &JsValue::from_bool(false),
        )
        .unwrap();
        js_sys::Reflect::set(
            &axis,
            &JsValue::from_str("showline"),
            &JsValue::from_bool(true),
        )
        .unwrap();
        js_sys::Reflect::set(
            &axis,
            &JsValue::from_str("linecolor"),
            &JsValue::from_str("rgba(0,0,0,0.1)"),
        )
        .unwrap();
        js_sys::Reflect::set(
            &axis,
            &JsValue::from_str("linewidth"),
            &JsValue::from_f64(1.0),
        )
        .unwrap();
        js_sys::Reflect::set(
            &axis,
            &JsValue::from_str("ticks"),
            &JsValue::from_str("outside"),
        )
        .unwrap();
        js_sys::Reflect::set(
            &axis,
            &JsValue::from_str("tickcolor"),
            &JsValue::from_str("rgba(0,0,0,0.1)"),
        )
        .unwrap();
        axis.into()
    })
    .unwrap();

    js_sys::Reflect::set(&layout, &JsValue::from_str("yaxis"), &{
        let axis = js_sys::Object::new();
        js_sys::Reflect::set(
            &axis,
            &JsValue::from_str("showgrid"),
            &JsValue::from_bool(true),
        )
        .unwrap();
        js_sys::Reflect::set(
            &axis,
            &JsValue::from_str("gridcolor"),
            &JsValue::from_str("rgba(0,0,0,0.05)"),
        )
        .unwrap();
        js_sys::Reflect::set(
            &axis,
            &JsValue::from_str("zeroline"),
            &JsValue::from_bool(false),
        )
        .unwrap();
        js_sys::Reflect::set(
            &axis,
            &JsValue::from_str("showline"),
            &JsValue::from_bool(false),
        )
        .unwrap();
        js_sys::Reflect::set(&axis, &JsValue::from_str("ticks"), &JsValue::from_str("")).unwrap();
        axis.into()
    })
    .unwrap();

    // Add padding
    js_sys::Reflect::set(&layout, &JsValue::from_str("margin"), &{
        let margin = js_sys::Object::new();
        js_sys::Reflect::set(&margin, &JsValue::from_str("l"), &JsValue::from_f64(50.0)).unwrap();
        js_sys::Reflect::set(&margin, &JsValue::from_str("r"), &JsValue::from_f64(30.0)).unwrap();
        js_sys::Reflect::set(&margin, &JsValue::from_str("t"), &JsValue::from_f64(40.0)).unwrap();
        js_sys::Reflect::set(&margin, &JsValue::from_str("b"), &JsValue::from_f64(50.0)).unwrap();
        js_sys::Reflect::set(&margin, &JsValue::from_str("pad"), &JsValue::from_f64(0.0)).unwrap();
        margin.into()
    })
    .unwrap();
}

// Function to render test plot
#[cfg(feature = "hydrate")]
pub fn render_test_plot(
    test_id: String,
    test_name: String,
    score_data: Vec<(Score, Test)>,
) -> impl IntoView {
    // Create a unique div id for this plot
    let plot_div_id = format!("plot-{}", test_id.replace("-", ""));
    let plot_div_id_clone = plot_div_id.clone();
    let plot_div_ref = create_node_ref::<html::Div>();

    let test_name_clone = test_name.clone();

    // Function to create the plot after component is mounted
    Effect::new(move |_| {
        // Extract score data for plotting
        let scores: Vec<i32> = score_data.iter().map(|(s, _)| s.get_total()).collect();
        let dates: Vec<String> = score_data
            .iter()
            .map(|(s, _)| format!("{:?}", s.date_administered))
            .collect();

        // Create JS data for Plotly
        let plot_data = js_sys::Array::new();
        let trace = js_sys::Object::new();

        // Set the trace properties
        js_sys::Reflect::set(
            &trace,
            &JsValue::from_str("x"),
            &JsValue::from_serde(&dates).unwrap(),
        )
        .unwrap();

        js_sys::Reflect::set(
            &trace,
            &JsValue::from_str("y"),
            &JsValue::from_serde(&scores).unwrap(),
        )
        .unwrap();

        js_sys::Reflect::set(
            &trace,
            &JsValue::from_str("type"),
            &JsValue::from_str("scatter"),
        )
        .unwrap();

        js_sys::Reflect::set(
            &trace,
            &JsValue::from_str("mode"),
            &JsValue::from_str("lines+markers"),
        )
        .unwrap();

        js_sys::Reflect::set(&trace, &JsValue::from_str("marker"), &{
            let marker = js_sys::Object::new();
            js_sys::Reflect::set(
                &marker,
                &JsValue::from_str("color"),
                &JsValue::from_str(color_primary()),
            )
            .unwrap();
            js_sys::Reflect::set(&marker, &JsValue::from_str("size"), &JsValue::from_f64(8.0))
                .unwrap();
            marker.into()
        })
        .unwrap();

        js_sys::Reflect::set(&trace, &JsValue::from_str("line"), &{
            let line = js_sys::Object::new();
            js_sys::Reflect::set(
                &line,
                &JsValue::from_str("color"),
                &JsValue::from_str(color_primary()),
            )
            .unwrap();
            js_sys::Reflect::set(&line, &JsValue::from_str("width"), &JsValue::from_f64(2.0))
                .unwrap();
            js_sys::Reflect::set(
                &line,
                &JsValue::from_str("shape"),
                &JsValue::from_str("spline"),
            )
            .unwrap();
            line.into()
        })
        .unwrap();

        plot_data.push(&trace);

        // Create layout
        let layout = js_sys::Object::new();

        // Add title with subtle styling
        js_sys::Reflect::set(&layout, &JsValue::from_str("title"), &{
            let title = js_sys::Object::new();
            js_sys::Reflect::set(
                &title,
                &JsValue::from_str("text"),
                &JsValue::from_str(&format!("{} Progress", test_name_clone.clone())),
            )
            .unwrap();
            js_sys::Reflect::set(&title, &JsValue::from_str("font"), &{
                let font = js_sys::Object::new();
                js_sys::Reflect::set(&font, &JsValue::from_str("size"), &JsValue::from_f64(16.0))
                    .unwrap();
                js_sys::Reflect::set(
                    &font,
                    &JsValue::from_str("color"),
                    &JsValue::from_str(color_text()),
                )
                .unwrap();
                font.into()
            })
            .unwrap();
            title.into()
        })
        .unwrap();

        // Apply common layout settings
        apply_common_layout_settings(&layout);

        // Specific settings for this chart
        js_sys::Reflect::set(&layout, &JsValue::from_str("xaxis"), &{
            let axis = js_sys::Object::new();
            js_sys::Reflect::set(&axis, &JsValue::from_str("title"), &{
                let title = js_sys::Object::new();
                js_sys::Reflect::set(
                    &title,
                    &JsValue::from_str("text"),
                    &JsValue::from_str("Date"),
                )
                .unwrap();
                js_sys::Reflect::set(
                    &title,
                    &JsValue::from_str("standoff"),
                    &JsValue::from_f64(10.0),
                )
                .unwrap();
                title.into()
            })
            .unwrap();
            js_sys::Reflect::set(
                &axis,
                &JsValue::from_str("showgrid"),
                &JsValue::from_bool(false),
            )
            .unwrap();
            axis.into()
        })
        .unwrap();

        js_sys::Reflect::set(&layout, &JsValue::from_str("yaxis"), &{
            let axis = js_sys::Object::new();
            js_sys::Reflect::set(&axis, &JsValue::from_str("title"), &{
                let title = js_sys::Object::new();
                js_sys::Reflect::set(
                    &title,
                    &JsValue::from_str("text"),
                    &JsValue::from_str("Score"),
                )
                .unwrap();
                title.into()
            })
            .unwrap();
            axis.into()
        })
        .unwrap();

        js_sys::Reflect::set(
            &layout,
            &JsValue::from_str("height"),
            &JsValue::from_f64(300.0),
        )
        .unwrap();

        // Get common config
        let config = get_common_config();

        // Call Plotly to create the chart
        newPlot(&plot_div_id_clone, &plot_data, &layout, &config);
    });

    // Create div for the plot and attach event listener to render after mount
    view! {
        <div class="my-6 rounded-xl shadow-sm bg-white overflow-hidden">
            <div class="px-6 py-4">
                <h4 class="text-lg font-medium text-gray-800">{test_name.clone()} " Progress"</h4>
                <div id={plot_div_id.clone()} class="h-80 w-full mt-2" node_ref={plot_div_ref}></div>
            </div>
        </div>
    }
}

// Function to render test distribution chart
#[cfg(feature = "hydrate")]
pub fn render_test_distribution(
    assessment_id: String,
    scores: Vec<(Score, Test)>,
) -> impl IntoView {
    let plot_div_id = format!("distribution-{}", assessment_id.replace("-", ""));
    let plot_div_id_clone = plot_div_id.clone();
    let plot_div_ref = create_node_ref::<html::Div>();

    Effect::new(move |_| {
        // Group scores by test name
        let mut test_scores = std::collections::HashMap::new();
        for (score, test) in &scores {
            let entry = test_scores
                .entry(test.name.clone())
                .or_insert_with(Vec::new);
            entry.push(score.get_total());
        }

        // Create data for the bar chart
        let plot_data = js_sys::Array::new();
        let trace = js_sys::Object::new();

        let test_names: Vec<String> = test_scores.keys().cloned().collect();
        let averages: Vec<f64> = test_scores
            .values()
            .map(|scores| {
                let sum: i32 = scores.iter().sum();
                sum as f64 / scores.len() as f64
            })
            .collect();

        js_sys::Reflect::set(
            &trace,
            &JsValue::from_str("x"),
            &JsValue::from_serde(&test_names).unwrap(),
        )
        .unwrap();

        js_sys::Reflect::set(
            &trace,
            &JsValue::from_str("y"),
            &JsValue::from_serde(&averages).unwrap(),
        )
        .unwrap();

        js_sys::Reflect::set(
            &trace,
            &JsValue::from_str("type"),
            &JsValue::from_str("bar"),
        )
        .unwrap();

        js_sys::Reflect::set(&trace, &JsValue::from_str("marker"), &{
            let marker = js_sys::Object::new();

            // Create gradient colors for bars
            let colors = js_sys::Array::new();
            for (i, _) in test_names.iter().enumerate() {
                let color_index = i % 3;
                let color = match color_index {
                    0 => color_primary(),
                    1 => color_secondary(),
                    _ => color_tertiary(),
                };
                colors.push(&JsValue::from_str(color));
            }

            js_sys::Reflect::set(&marker, &JsValue::from_str("color"), &colors).unwrap();

            js_sys::Reflect::set(
                &marker,
                &JsValue::from_str("opacity"),
                &JsValue::from_f64(0.9),
            )
            .unwrap();

            marker.into()
        })
        .unwrap();

        plot_data.push(&trace);

        // Create layout
        let layout = js_sys::Object::new();

        // Add title with subtle styling
        js_sys::Reflect::set(&layout, &JsValue::from_str("title"), &{
            let title = js_sys::Object::new();
            js_sys::Reflect::set(
                &title,
                &JsValue::from_str("text"),
                &JsValue::from_str("Test Score Distribution"),
            )
            .unwrap();
            js_sys::Reflect::set(&title, &JsValue::from_str("font"), &{
                let font = js_sys::Object::new();
                js_sys::Reflect::set(&font, &JsValue::from_str("size"), &JsValue::from_f64(16.0))
                    .unwrap();
                js_sys::Reflect::set(
                    &font,
                    &JsValue::from_str("color"),
                    &JsValue::from_str(color_text()),
                )
                .unwrap();
                font.into()
            })
            .unwrap();
            title.into()
        })
        .unwrap();

        // Apply common layout settings
        apply_common_layout_settings(&layout);

        // Specific settings for this chart
        js_sys::Reflect::set(&layout, &JsValue::from_str("xaxis"), &{
            let axis = js_sys::Object::new();
            js_sys::Reflect::set(&axis, &JsValue::from_str("title"), &{
                let title = js_sys::Object::new();
                js_sys::Reflect::set(
                    &title,
                    &JsValue::from_str("text"),
                    &JsValue::from_str("Subject"),
                )
                .unwrap();
                js_sys::Reflect::set(
                    &title,
                    &JsValue::from_str("standoff"),
                    &JsValue::from_f64(10.0),
                )
                .unwrap();
                title.into()
            })
            .unwrap();
            axis.into()
        })
        .unwrap();

        js_sys::Reflect::set(&layout, &JsValue::from_str("yaxis"), &{
            let axis = js_sys::Object::new();
            js_sys::Reflect::set(&axis, &JsValue::from_str("title"), &{
                let title = js_sys::Object::new();
                js_sys::Reflect::set(
                    &title,
                    &JsValue::from_str("text"),
                    &JsValue::from_str("Average Score"),
                )
                .unwrap();
                title.into()
            })
            .unwrap();
            axis.into()
        })
        .unwrap();

        js_sys::Reflect::set(
            &layout,
            &JsValue::from_str("height"),
            &JsValue::from_f64(300.0),
        )
        .unwrap();

        js_sys::Reflect::set(
            &layout,
            &JsValue::from_str("bargap"),
            &JsValue::from_f64(0.3),
        )
        .unwrap();

        // Get common config
        let config = get_common_config();

        // Call Plotly to create the chart
        newPlot(&plot_div_id_clone, &plot_data, &layout, &config);
    });

    view! {
        <div class="my-6 rounded-xl shadow-sm bg-white overflow-hidden">
            <div class="px-6 py-4">
                <h4 class="text-lg font-medium text-gray-800">"Test Score Distribution"</h4>
                <div id={plot_div_id.clone()} class="h-80 w-full mt-2" node_ref={plot_div_ref}></div>
            </div>
        </div>
    }
}

// Function to render overall student progress chart
#[cfg(feature = "hydrate")]
pub fn render_overall_progress(scores: Vec<Score>) -> impl IntoView {
    let plot_div_id = "overall-progress-plot";
    let plot_div_ref = create_node_ref::<html::Div>();

    Effect::new(move |_| {
        if scores.is_empty() {
            return;
        }

        // Group scores by date
        let mut score_by_date = std::collections::HashMap::new();
        for score in &scores {
            let date = format!("{:?}", score.date_administered);
            let entry = score_by_date.entry(date).or_insert_with(Vec::new);
            entry.push(score.get_total());
        }

        // Calculate average for each date
        let mut dates: Vec<String> = score_by_date.keys().cloned().collect();
        dates.sort(); // Sort dates chronologically

        let averages: Vec<f64> = dates
            .iter()
            .map(|date| {
                let scores = score_by_date.get(date).unwrap();
                let sum: i32 = scores.iter().sum();
                sum as f64 / scores.len() as f64
            })
            .collect();

        // Create JS data for Plotly
        let plot_data = js_sys::Array::new();
        let trace = js_sys::Object::new();

        js_sys::Reflect::set(
            &trace,
            &JsValue::from_str("x"),
            &JsValue::from_serde(&dates).unwrap(),
        )
        .unwrap();

        js_sys::Reflect::set(
            &trace,
            &JsValue::from_str("y"),
            &JsValue::from_serde(&averages).unwrap(),
        )
        .unwrap();

        js_sys::Reflect::set(
            &trace,
            &JsValue::from_str("type"),
            &JsValue::from_str("scatter"),
        )
        .unwrap();

        js_sys::Reflect::set(
            &trace,
            &JsValue::from_str("mode"),
            &JsValue::from_str("lines+markers"),
        )
        .unwrap();

        js_sys::Reflect::set(
            &trace,
            &JsValue::from_str("fill"),
            &JsValue::from_str("tozeroy"),
        )
        .unwrap();

        js_sys::Reflect::set(
            &trace,
            &JsValue::from_str("fillcolor"),
            &JsValue::from_str("rgba(255, 122, 89, 0.1)"),
        )
        .unwrap();

        js_sys::Reflect::set(&trace, &JsValue::from_str("marker"), &{
            let marker = js_sys::Object::new();
            js_sys::Reflect::set(
                &marker,
                &JsValue::from_str("color"),
                &JsValue::from_str(color_primary()),
            )
            .unwrap();
            js_sys::Reflect::set(&marker, &JsValue::from_str("size"), &JsValue::from_f64(8.0))
                .unwrap();
            marker.into()
        })
        .unwrap();

        js_sys::Reflect::set(&trace, &JsValue::from_str("line"), &{
            let line = js_sys::Object::new();
            js_sys::Reflect::set(
                &line,
                &JsValue::from_str("color"),
                &JsValue::from_str(color_primary()),
            )
            .unwrap();
            js_sys::Reflect::set(&line, &JsValue::from_str("width"), &JsValue::from_f64(2.5))
                .unwrap();
            js_sys::Reflect::set(
                &line,
                &JsValue::from_str("shape"),
                &JsValue::from_str("spline"),
            )
            .unwrap();
            line.into()
        })
        .unwrap();

        plot_data.push(&trace);

        // Create layout
        let layout = js_sys::Object::new();

        // Add title with subtle styling
        js_sys::Reflect::set(&layout, &JsValue::from_str("title"), &{
            let title = js_sys::Object::new();
            js_sys::Reflect::set(
                &title,
                &JsValue::from_str("text"),
                &JsValue::from_str("Performance Trend"),
            )
            .unwrap();
            js_sys::Reflect::set(&title, &JsValue::from_str("font"), &{
                let font = js_sys::Object::new();
                js_sys::Reflect::set(&font, &JsValue::from_str("size"), &JsValue::from_f64(18.0))
                    .unwrap();
                js_sys::Reflect::set(
                    &font,
                    &JsValue::from_str("color"),
                    &JsValue::from_str(color_text()),
                )
                .unwrap();
                font.into()
            })
            .unwrap();
            title.into()
        })
        .unwrap();

        // Apply common layout settings
        apply_common_layout_settings(&layout);

        // Specific settings for this chart
        js_sys::Reflect::set(&layout, &JsValue::from_str("xaxis"), &{
            let axis = js_sys::Object::new();
            js_sys::Reflect::set(&axis, &JsValue::from_str("title"), &{
                let title = js_sys::Object::new();
                js_sys::Reflect::set(
                    &title,
                    &JsValue::from_str("text"),
                    &JsValue::from_str("Date"),
                )
                .unwrap();
                js_sys::Reflect::set(
                    &title,
                    &JsValue::from_str("standoff"),
                    &JsValue::from_f64(10.0),
                )
                .unwrap();
                title.into()
            })
            .unwrap();
            axis.into()
        })
        .unwrap();

        js_sys::Reflect::set(&layout, &JsValue::from_str("yaxis"), &{
            let axis = js_sys::Object::new();
            js_sys::Reflect::set(&axis, &JsValue::from_str("title"), &{
                let title = js_sys::Object::new();
                js_sys::Reflect::set(
                    &title,
                    &JsValue::from_str("text"),
                    &JsValue::from_str("Average Score"),
                )
                .unwrap();
                title.into()
            })
            .unwrap();
            axis.into()
        })
        .unwrap();

        js_sys::Reflect::set(
            &layout,
            &JsValue::from_str("height"),
            &JsValue::from_f64(350.0),
        )
        .unwrap();

        // Get common config
        let config = get_common_config();

        // Call Plotly to create the chart
        newPlot(plot_div_id, &plot_data, &layout, &config);
    });

    view! {
        <div class="mt-6 rounded-xl shadow-sm bg-white overflow-hidden">
            <div class="px-6 py-4">
                <h3 class="text-xl font-medium text-gray-800">"Performance Trend"</h3>
                <div id={plot_div_id} class="h-96 w-full mt-2" node_ref={plot_div_ref}></div>
            </div>
        </div>
    }
}

// Function to render score distribution chart
#[cfg(feature = "hydrate")]
pub fn render_score_distribution(scores: Vec<Score>) -> impl IntoView {
    let plot_div_id = "score-distribution-plot";
    let plot_div_ref = create_node_ref::<html::Div>();

    Effect::new(move |_| {
        if scores.is_empty() {
            return;
        }

        // Group scores by range
        let mut score_ranges = vec![0, 0, 0, 0, 0]; // 0-20, 21-40, 41-60, 61-80, 81-100

        for score in &scores {
            let total = score.get_total();
            if total <= 20 {
                score_ranges[0] += 1;
            } else if total <= 40 {
                score_ranges[1] += 1;
            } else if total <= 60 {
                score_ranges[2] += 1;
            } else if total <= 80 {
                score_ranges[3] += 1;
            } else {
                score_ranges[4] += 1;
            }
        }

        let range_labels = vec!["0-20", "21-40", "41-60", "61-80", "81-100"];

        // Create JS data for Plotly
        let plot_data = js_sys::Array::new();
        let trace = js_sys::Object::new();

        js_sys::Reflect::set(
            &trace,
            &JsValue::from_str("x"),
            &JsValue::from_serde(&range_labels).unwrap(),
        )
        .unwrap();

        js_sys::Reflect::set(
            &trace,
            &JsValue::from_str("y"),
            &JsValue::from_serde(&score_ranges).unwrap(),
        )
        .unwrap();

        js_sys::Reflect::set(
            &trace,
            &JsValue::from_str("type"),
            &JsValue::from_str("bar"),
        )
        .unwrap();

        js_sys::Reflect::set(&trace, &JsValue::from_str("marker"), &{
            let marker = js_sys::Object::new();
            let colors = js_sys::Array::new();
            colors.push(&JsValue::from_str(color_low())); // Red-orange for 0-20
            colors.push(&JsValue::from_str(color_medium())); // Medium orange for 21-40
            colors.push(&JsValue::from_str(color_medium())); // Medium orange for 41-60
            colors.push(&JsValue::from_str(color_medium())); // Medium orange for 61-80
            colors.push(&JsValue::from_str(color_high())); // Orange for 81-100

            js_sys::Reflect::set(&marker, &JsValue::from_str("color"), &colors).unwrap();
            js_sys::Reflect::set(
                &marker,
                &JsValue::from_str("opacity"),
                &JsValue::from_f64(0.9),
            )
            .unwrap();

            marker.into()
        })
        .unwrap();

        plot_data.push(&trace);

        // Create layout
        let layout = js_sys::Object::new();

        // Add title with subtle styling
        js_sys::Reflect::set(&layout, &JsValue::from_str("title"), &{
            let title = js_sys::Object::new();
            js_sys::Reflect::set(
                &title,
                &JsValue::from_str("text"),
                &JsValue::from_str("Score Distribution"),
            )
            .unwrap();
            js_sys::Reflect::set(&title, &JsValue::from_str("font"), &{
                let font = js_sys::Object::new();
                js_sys::Reflect::set(&font, &JsValue::from_str("size"), &JsValue::from_f64(16.0))
                    .unwrap();
                js_sys::Reflect::set(
                    &font,
                    &JsValue::from_str("color"),
                    &JsValue::from_str(color_text()),
                )
                .unwrap();
                font.into()
            })
            .unwrap();
            title.into()
        })
        .unwrap();

        // Apply common layout settings
        apply_common_layout_settings(&layout);

        // Specific settings for this chart
        js_sys::Reflect::set(&layout, &JsValue::from_str("xaxis"), &{
            let axis = js_sys::Object::new();
            js_sys::Reflect::set(&axis, &JsValue::from_str("title"), &{
                let title = js_sys::Object::new();
                js_sys::Reflect::set(
                    &title,
                    &JsValue::from_str("text"),
                    &JsValue::from_str("Score Range"),
                )
                .unwrap();
                js_sys::Reflect::set(
                    &title,
                    &JsValue::from_str("standoff"),
                    &JsValue::from_f64(10.0),
                )
                .unwrap();
                title.into()
            })
            .unwrap();
            axis.into()
        })
        .unwrap();

        js_sys::Reflect::set(&layout, &JsValue::from_str("yaxis"), &{
            let axis = js_sys::Object::new();
            js_sys::Reflect::set(&axis, &JsValue::from_str("title"), &{
                let title = js_sys::Object::new();
                js_sys::Reflect::set(
                    &title,
                    &JsValue::from_str("text"),
                    &JsValue::from_str("Number of Tests"),
                )
                .unwrap();
                title.into()
            })
            .unwrap();
            axis.into()
        })
        .unwrap();

        js_sys::Reflect::set(
            &layout,
            &JsValue::from_str("height"),
            &JsValue::from_f64(350.0),
        )
        .unwrap();

        js_sys::Reflect::set(
            &layout,
            &JsValue::from_str("bargap"),
            &JsValue::from_f64(0.3),
        )
        .unwrap();

        // Get common config
        let config = get_common_config();

        // Call Plotly to create the chart
        newPlot(plot_div_id, &plot_data, &layout, &config);
    });

    view! {
        <div class="mb-6">
            <h3 class="font-semibold mb-2">"Score Distribution"</h3>
            <div id={plot_div_id} class="h-80 w-full border rounded p-4 bg-white" node_ref={plot_div_ref}></div>
        </div>
    }
}
