use crate::app::components::test_components::font_controls::FontSettings;
use crate::app::models::question::Question;
use leptos::*;

#[component]
pub fn QuestionDisplay(
    question: Question,
    #[prop(into)] font_settings: Signal<FontSettings>,
    #[prop(into)] current_index: Signal<usize>,
    total_questions: usize,
) -> impl IntoView {
    // Clone the word_problem to avoid move issues
    let word_problem = question.word_problem.clone();
    let word_problem_for_closure = word_problem.clone(); // Create a second clone for the closure
    let point_value = question.point_value;

    view! {
        <div class="bg-gradient-to-r from-blue-50 to-indigo-50 border-b border-gray-100 px-6 py-3">
            <div class="flex items-center justify-between">
                <h2 class="text-lg font-semibold text-gray-900">
                    "Question " {move || current_index.get() + 1}
                </h2>
                <span class="text-sm text-gray-600 font-medium">
                    {point_value} " points"
                </span>
            </div>
        </div>

        <div class="p-6">
            <div class="mb-6">
                <div class=move || {
                    let question_text = word_problem_for_closure.clone();
                    let is_long = question_text.len() > 100;
                    let alignment = if is_long { "text-left" } else { "text-center" };
                    format!("leading-relaxed {} {}", font_settings.get().get_question_classes(), alignment)
                }>
                    {word_problem}
                </div>
            </div>
        </div>
    }
}
