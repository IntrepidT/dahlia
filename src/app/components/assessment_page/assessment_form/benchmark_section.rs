use crate::app::models::assessment::RangeCategory;
use leptos::*;

#[component]
pub fn BenchmarkSection(
    benchmarks: Signal<Option<Vec<RangeCategory>>>,
    set_benchmarks: impl Fn(Option<Vec<RangeCategory>>) + 'static + Copy,
    section_type: &'static str,
) -> impl IntoView {
    let (benchmark_min, set_benchmark_min) = create_signal(0);
    let (benchmark_max, set_benchmark_max) = create_signal(0);
    let (benchmark_label, set_benchmark_label) = create_signal(String::new());

    let add_benchmark = move |_| {
        let min = benchmark_min.get();
        let max = benchmark_max.get();
        let label = benchmark_label.get();

        if !label.is_empty() && min < max {
            let new_benchmark = RangeCategory::new(min, max, label);
            let mut current = benchmarks.get().unwrap_or_default();
            current.push(new_benchmark);
            set_benchmarks(Some(current));

            set_benchmark_min.set(0);
            set_benchmark_max.set(0);
            set_benchmark_label.set(String::new());
        }
    };

    view! {
        <div class="space-y-4">
            // Existing benchmarks display
            <div class="space-y-3">
                {move || {
                    let benchmarks_list = benchmarks.get().unwrap_or_default();
                    benchmarks_list.into_iter().map(|benchmark| {
                        let benchmark_label_clone = benchmark.label.clone();
                        view! {
                            <div class="flex items-center justify-between bg-gray-50 p-4 rounded-lg border border-gray-200 hover:bg-gray-100 transition-colors">
                                <div class="flex items-center space-x-3 text-gray-900">
                                    <span class="font-medium text-sm min-w-0 flex-shrink-0">{benchmark.label}</span>
                                    <span class="text-gray-400">"|"</span>
                                    <div class="flex items-center space-x-2 text-sm text-gray-700">
                                        <span class="bg-blue-100 text-blue-800 px-2 py-1 rounded text-xs font-medium">{benchmark.min}</span>
                                        <span class="text-gray-500">"to"</span>
                                        <span class="bg-blue-100 text-blue-800 px-2 py-1 rounded text-xs font-medium">{benchmark.max}</span>
                                    </div>
                                </div>
                                <button
                                    type="button"
                                    class="text-sm text-red-600 hover:text-red-800 hover:bg-red-50 px-3 py-1.5 rounded-md transition-colors flex-shrink-0 font-medium"
                                    on:click=move |_| {
                                        let mut current = benchmarks.get().unwrap_or_default();
                                        current.retain(|b| b.label != benchmark_label_clone);
                                        set_benchmarks(Some(current));
                                    }
                                >
                                    "Remove"
                                </button>
                            </div>
                        }
                    }).collect_view()
                }}
            </div>

            // Add new benchmark form
            <div class="bg-gray-50 p-4 rounded-lg border border-gray-200">
                <h6 class="text-sm font-medium text-gray-700 mb-3">"Add New "{section_type}" Benchmark"</h6>
                <div class="grid grid-cols-1 md:grid-cols-4 gap-4 items-end">
                    <div>
                        <label class="block text-xs font-medium text-gray-600 mb-1">"Min Score"</label>
                        <input
                            type="number"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 bg-white text-gray-900"
                            placeholder="0"
                            min="0"
                            prop:value={move || benchmark_min.get()}
                            on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                    set_benchmark_min.set(v);
                                }
                            }
                        />
                    </div>
                    <div>
                        <label class="block text-xs font-medium text-gray-600 mb-1">"Max Score"</label>
                        <input
                            type="number"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 bg-white text-gray-900"
                            placeholder="100"
                            min="0"
                            prop:value={move || benchmark_max.get()}
                            on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                    set_benchmark_max.set(v);
                                }
                            }
                        />
                    </div>
                    <div>
                        <label class="block text-xs font-medium text-gray-600 mb-1">"Category Label"</label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 bg-white text-gray-900"
                            placeholder="e.g., Mastery, Developing"
                            prop:value={move || benchmark_label.get()}
                            on:input=move |ev| set_benchmark_label.set(event_target_value(&ev))
                        />
                    </div>
                    <div>
                        <button
                            type="button"
                            class="w-full px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md border border-blue-600 hover:bg-blue-700 focus:ring-2 focus:ring-blue-500 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                            on:click=add_benchmark
                            disabled=move || {
                                let label_empty = benchmark_label.get().trim().is_empty();
                                let invalid_range = benchmark_min.get() >= benchmark_max.get();
                                label_empty || invalid_range
                            }
                        >
                            "Add Benchmark"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
