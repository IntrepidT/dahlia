use super::types::Role;
use leptos::prelude::*;

#[component]
pub fn NavigationControls(
    #[prop(into)] role: Signal<Role>,
    #[prop(into)] is_test_active: Signal<bool>,
    #[prop(into)] is_submitted: Signal<bool>,
    #[prop(into)] should_disable_inputs: Signal<bool>,
    #[prop(into)] current_card_index: Signal<usize>,
    #[prop(into)] total_questions: Signal<usize>,
    #[prop(into)] selected_student_id: Signal<Option<i32>>,
    #[prop(into)] on_previous: Callback<()>,
    #[prop(into)] on_next: Callback<()>,
    #[prop(into)] on_submit: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || is_test_active.get() || matches!(role.get(), Role::Teacher)>
            <div class="flex flex-wrap items-center justify-center gap-4 mt-8">
                <button
                    class="flex items-center justify-center px-5 py-2 bg-white border border-gray-200 rounded-lg shadow-sm text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                    disabled=move || (current_card_index.get() == 0 || should_disable_inputs())
                    on:click=move |_| on_previous.run(())
                >
                    <span class="mr-1">"←"</span>
                    "Previous"
                </button>

                {move || {
                    let is_last = current_card_index.get() == total_questions.get().saturating_sub(1);

                    if is_last && matches!(role.get(), Role::Teacher) && is_test_active.get() && !is_submitted.get() {
                        view! {
                            <button
                                class="flex items-center justify-center px-5 py-2 bg-gradient-to-r from-blue-600 to-purple-600 text-white rounded-lg shadow-sm hover:from-blue-700 hover:to-purple-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                on:click=move |_| on_submit.run(())
                                disabled=move || (selected_student_id.get().is_none() || should_disable_inputs())
                            >
                                "Submit Assessment"
                                <span class="ml-1">"✓"</span>
                            </button>
                        }.into_any()
                    } else if !is_last {
                        view! {
                            <button
                                class="flex items-center justify-center px-5 py-2 bg-gradient-to-r from-blue-600 to-purple-600 text-white rounded-lg shadow-sm hover:from-blue-700 hover:to-purple-700 transition-colors"
                                on:click=move |_| on_next.run(())
                                disabled=move || should_disable_inputs()
                            >
                                "Next"
                                <span class="ml-1">"→"</span>
                            </button>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
            </div>
        </Show>
    }
}
