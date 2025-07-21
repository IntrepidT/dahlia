use leptos::*;

#[component]
pub fn ProgressIndicator(
    #[prop(into)] current_index: Signal<usize>,
    total_questions: usize,
    #[prop(into)] answered_percentage: Signal<f32>,
    point_value: i32,
) -> impl IntoView {
    view! {
        <div class="text-center space-y-3">
            {/* Minimalist Progress Bar */}
            <div class="w-full max-w-md mx-auto">
                <div class="bg-gray-100 rounded-full h-1">
                    <div
                        class="bg-gradient-to-r from-blue-500 to-indigo-600 h-1 rounded-full transition-all duration-700 ease-out"
                        style=move || format!("width: {}%", answered_percentage())
                    ></div>
                </div>
            </div>

            {/* Question Counter */}
            <div class="flex items-center justify-center gap-6 text-sm">
                <span class="text-gray-500">
                    "Question " {move || current_index.get() + 1} " of " {total_questions}
                </span>
                <span class="px-3 py-1 bg-indigo-50 text-indigo-700 rounded-full font-medium">
                    {point_value} " points"
                </span>
            </div>
        </div>
    }
}
