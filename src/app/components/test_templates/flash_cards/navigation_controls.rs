use leptos::*;

#[component]
pub fn NavigationControls(
    #[prop(into)] current_index: Signal<usize>,
    total_questions: usize,
    #[prop(into)] is_submitted: Signal<bool>,
    #[prop(into)] selected_student_id: Signal<Option<i32>>,
    #[prop(into)] on_previous: Callback<()>,
    #[prop(into)] on_next: Callback<()>,
    #[prop(into)] on_submit: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-center gap-4 pt-4">
            <button
                class="flex items-center gap-2 px-4 py-2 bg-white border border-gray-200 rounded-lg text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200 shadow-sm"
                disabled=move || current_index.get() == 0
                on:click=move |_| on_previous.call(())
            >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"></path>
                </svg>
                "Previous"
            </button>

            {move || {
                if current_index.get() == total_questions - 1 {
                    view! {
                        <Show when=move || !is_submitted.get() fallback=move || view! {
                            <button
                                class="flex items-center gap-2 px-6 py-2 bg-gray-900 text-white rounded-lg hover:bg-gray-800 transition-all duration-200 shadow-lg hover:shadow-xl transform hover:scale-105"
                                on:click=move |_| {
                                    let navigate = leptos_router::use_navigate();
                                    navigate("/dashboard", Default::default());
                                }
                            >
                                "Return to Dashboard"
                                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18"></path>
                                </svg>
                            </button>
                        }>
                            <button
                                class="flex items-center gap-2 px-6 py-2 bg-gradient-to-r from-blue-600 to-indigo-600 text-white rounded-lg hover:from-blue-700 hover:to-indigo-700 transition-all duration-200 shadow-lg hover:shadow-xl transform hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none"
                                on:click=move |_| on_submit.call(())
                                disabled=move || selected_student_id.get().is_none()
                            >
                                "Submit Assessment"
                                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
                                </svg>
                            </button>
                        </Show>
                    }.into_view()
                } else {
                    view! {
                        <button
                            class="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-blue-600 to-indigo-600 text-white rounded-lg hover:from-blue-700 hover:to-indigo-700 transition-all duration-200 shadow-lg hover:shadow-xl transform hover:scale-105"
                            on:click=move |_| on_next.call(())
                        >
                            "Next"
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path>
                            </svg>
                        </button>
                    }.into_view()
                }
            }}
        </div>
    }
}
