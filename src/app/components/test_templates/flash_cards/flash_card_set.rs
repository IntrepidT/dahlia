use super::*;
use crate::app::components::test_components::font_controls::{use_font_settings, FontControls};
use crate::app::components::test_components::test_instructions::TestInstructions;
use crate::app::models::question::{Question, QuestionType};
use crate::app::models::test::Test;
use crate::app::models::user::SessionUser;
use leptos::*;
use std::collections::HashMap;

#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

#[component]
pub fn FlashCardContainer(
    questions: Vec<Question>,
    test_details: Option<Test>,
    user: Option<SessionUser>,
    #[prop(into)] on_submit: Callback<(HashMap<i32, QuestionResponse>, Option<i32>)>,
) -> impl IntoView {
    let (font_settings, set_font_settings) = use_font_settings();
    let (shortcuts_expanded, set_shortcuts_expanded) = create_signal(false);
    let (instructions_expanded, set_instructions_expanded) = create_signal(false);

    let (state, set_responses, set_current_card_index, set_is_submitted, set_selected_student_id) =
        use_flash_card_state();

    // Clone questions for multiple uses
    let questions_for_percentage = questions.clone();
    let questions_for_navigation = questions.clone();
    let questions_for_memo = questions.clone();
    let questions_for_keyboard = questions.clone();

    // Clone test_details for multiple uses
    let test_details_for_title = test_details.clone();

    // Create a signal for instructions to handle lifetime issues
    let (instructions_signal, set_instructions_signal) = create_signal(None::<String>);

    // Update instructions when test details change
    create_effect({
        let test_details = test_details.clone();
        move |_| {
            if let Some(test) = test_details.get().flatten() {
                set_instructions_signal.set(test.instructions.clone());
            } else {
                set_instructions_signal.set(None);
            }
        }
    });

    // Calculate answered percentage
    let calculate_answered_percentage = create_memo(move |_| {
        let answered_count = state.responses.with(|r| {
            questions_for_percentage
                .iter()
                .filter(|question| {
                    r.get(&question.qnumber)
                        .map(|resp| !resp.answer.trim().is_empty())
                        .unwrap_or(false)
                })
                .count() as f32
        });

        let total = questions_for_percentage.len() as f32;
        if total > 0.0 {
            (answered_count / total) * 100.0
        } else {
            0.0
        }
    });

    // Create simple wrapper functions for keyboard handler
    let handle_answer_change_simple = {
        let set_responses = set_responses.clone();
        move |qnumber: i32, value: String| {
            set_responses.update(|r| {
                let response = r.entry(qnumber).or_insert_with(QuestionResponse::new);
                response.answer = value;
            });
        }
    };

    let handle_weighted_selection_simple = {
        let set_responses = set_responses.clone();
        move |qnumber: i32, selected_options: Vec<String>| {
            set_responses.update(|r| {
                let response = r.entry(qnumber).or_insert_with(QuestionResponse::new);
                response.selected_options = Some(selected_options.clone());
                response.answer = serde_json::to_string(&selected_options).unwrap_or_default();
            });
        }
    };

    let handle_submit_click_simple = {
        let state = state.clone();
        let on_submit = on_submit.clone();
        let set_is_submitted = set_is_submitted.clone();
        move || {
            let current_responses = state.responses.get();
            let student_id = state.selected_student_id.get();
            on_submit.call((current_responses, student_id));
            set_is_submitted.set(true);
        }
    };

    // Handler functions for callbacks
    let handle_answer_change = move |(qnumber, value): (i32, String)| {
        set_responses.update(|r| {
            let response = r.entry(qnumber).or_insert_with(QuestionResponse::new);
            response.answer = value;
        });
    };

    let handle_weighted_selection = move |(qnumber, selected_options): (i32, Vec<String>)| {
        set_responses.update(|r| {
            let response = r.entry(qnumber).or_insert_with(QuestionResponse::new);
            response.selected_options = Some(selected_options.clone());
            response.answer = serde_json::to_string(&selected_options).unwrap_or_default();
        });
    };

    let handle_comment_change = move |qnumber: i32, value: String| {
        set_responses.update(|r| {
            let response = r.entry(qnumber).or_insert_with(QuestionResponse::new);
            response.comment = value;
        });
    };

    // Navigation handlers
    let go_to_next_card = move |_| {
        set_current_card_index.update(|index| {
            *index = (*index + 1).min(questions_for_navigation.len() - 1);
        });
    };

    let go_to_previous_card = move |_| {
        set_current_card_index.update(|index| {
            *index = index.saturating_sub(1);
        });
    };

    let handle_submit_click = move |_| {
        let current_responses = state.responses.get();
        let student_id = state.selected_student_id.get();
        on_submit.call((current_responses, student_id));
        set_is_submitted.set(true);
    };

    // Get current question
    let current_question = create_memo(move |_| {
        questions_for_memo
            .get(state.current_card_index.get())
            .cloned()
            .unwrap_or_else(|| questions_for_memo.first().cloned().unwrap())
    });

    // Focus comments box
    let focus_comments = move || {
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(textarea) = document
                        .query_selector("textarea[placeholder*='notes']")
                        .ok()
                        .flatten()
                    {
                        let _ = textarea.unchecked_ref::<web_sys::HtmlElement>().focus();
                    }
                }
            }
        }
    };

    // Keyboard handling setup
    use_keyboard_handler(
        state.clone(),
        questions_for_keyboard,
        handle_answer_change_simple,
        handle_weighted_selection_simple,
        focus_comments,
        handle_submit_click_simple,
        set_current_card_index,
    );

    view! {
        <div class="min-h-screen bg-gray-50" tabindex="-1">
            {/* Header */}
            <div class="sticky top-0 z-10 bg-white/80 backdrop-blur-md border-b border-gray-100">
                <div class="max-w-5xl mx-auto px-6 py-3">
                    <div class="flex items-center justify-between">
                        {/* Left: Student Select */}
                        <div class="flex-shrink-0">
                            <StudentSelector set_selected_student_id=set_selected_student_id />
                        </div>

                        {/* Center: Test Title */}
                        <div class="flex-1 text-center px-8">
                            <h1 class="text-lg font-medium text-gray-900 truncate">
                                {move || match &test_details_for_title {
                                    Some(test) => test.name.clone(),
                                    None => "Flash Card Test".to_string()
                                }}
                            </h1>
                        </div>

                        {/* Right: Controls */}
                        <div class="flex items-center gap-3">
                            <FontControls
                                font_settings=font_settings
                                set_font_settings=set_font_settings
                            />
                            <div class="text-sm text-gray-500 font-medium hidden sm:block">
                                {move || match &user {
                                    Some(user_data) => format!("{} {}",
                                        user_data.first_name.as_ref().unwrap_or(&"".to_string()),
                                        user_data.last_name.as_ref().unwrap_or(&"".to_string())
                                    ).trim().to_string(),
                                    None => "Guest".to_string(),
                                }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            {/* Instructions - Collapsible */}
            <div class="max-w-5xl mx-auto px-6 pt-4">
                {move || match &test_details_for_instructions {
                    Some(test) => {
                        if test.instructions.as_ref().map_or(false, |inst| !inst.is_empty()) {
                                                        view! {
                                <div class="mb-2">
                                    <button
                                        class="flex items-center gap-2 text-sm text-blue-600 hover:text-blue-800 font-medium"
                                        on:click=move |_| set_instructions_expanded.update(|x| *x = !*x)
                                    >
                                        <svg class=move || format!("w-4 h-4 transition-transform {}",
                                            if instructions_expanded() { "rotate-90" } else { "" }
                                        ) fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path>
                                        </svg>
                                        "Test Instructions"
                                    </button>
                                    <Show when=move || instructions_expanded()>
                                        <div class="mt-2">
                                            <TestInstructions instructions=test.instructions.clone() />
                                        </div>
                                    </Show>
                                </div>
                            }.into_view()
                        } else {
                            view! { <div></div> }.into_view()
                        }
                    },
                    None => view! { <div></div> }.into_view()
                }}
            </div>

            {/* Keyboard Shortcuts Help - Collapsible */}
            <div class="max-w-5xl mx-auto px-6">
                <div class="mb-4">
                    <button
                        class="flex items-center gap-2 text-sm text-blue-600 hover:text-blue-800 font-medium"
                        on:click=move |_| set_shortcuts_expanded.update(|x| *x = !*x)
                    >
                        <svg class=move || format!("w-4 h-4 transition-transform {}",
                            if shortcuts_expanded() { "rotate-90" } else { "" }
                        ) fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path>
                        </svg>
                        "Keyboard Shortcuts"
                    </button>
                    <Show when=move || shortcuts_expanded()>
                        <div class="mt-2 bg-blue-50 border border-blue-200 rounded-lg p-3 text-sm text-blue-800">
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs">
                                <span>"← → or P/N: Navigate"</span>
                                <span>"1-9: Select answers"</span>
                                <span>"Ctrl+Enter: Next/Submit"</span>
                                <span>"C: Focus comments"</span>
                            </div>
                        </div>
                    </Show>
                </div>
            </div>

            {/* Main Content */}
            <div class="max-w-5xl mx-auto px-6 pb-8">
                <div class="space-y-6">
                    {/* Progress Section */}
                    <ProgressIndicator
                        current_index=state.current_card_index
                        total_questions=questions.len()
                        answered_percentage=Signal::derive(move || calculate_answered_percentage())
                        point_value=current_question().point_value
                    />

                    {/* Card Container */}
                    <div class="max-w-4xl mx-auto">
                        <div class="bg-white rounded-xl shadow-sm border border-gray-100 overflow-hidden">
                            <QuestionDisplay
                                question=current_question()
                                font_settings=font_settings
                                current_index=state.current_card_index
                                total_questions=questions.len()
                            />

                            <div class="p-6">
                                {/* Answer Section */}
                                <div class="space-y-4">
                                    <AnswerInput
                                        question=current_question()
                                        responses=state.responses
                                        font_settings=font_settings
                                        on_answer_change=Callback::new(handle_answer_change)
                                        on_weighted_selection=Callback::new(handle_weighted_selection)
                                        disabled=Signal::derive(move || false)
                                    />

                                    {/* Comments Section */}
                                    <div class="border-t border-gray-100 pt-4">
                                        <label class="block text-sm font-medium text-gray-700 mb-2">
                                            "Notes & Comments " <span class="text-xs text-gray-400">"(Press C to focus)"</span>
                                        </label>
                                        {move || {
                                            let qnumber = current_question().qnumber;
                                            let comment_value = create_memo(move |_| {
                                                state.responses.with(|r| {
                                                    r.get(&qnumber)
                                                     .map(|resp| resp.comment.clone())
                                                     .unwrap_or_default()
                                                })
                                            });

                                            view! {
                                                <textarea
                                                    class="w-full p-3 border border-gray-200 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 resize-none transition-all duration-200 text-sm"
                                                    prop:value=move || comment_value()
                                                    on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        handle_comment_change(qnumber, value);
                                                    }
                                                    placeholder="Add any notes or observations about this question..."
                                                    rows="2"
                                                ></textarea>
                                            }
                                        }}
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* Navigation */}
                    <NavigationControls
                        current_index=state.current_card_index
                        total_questions=questions.len()
                        is_submitted=state.is_submitted
                        selected_student_id=state.selected_student_id
                        on_previous=Callback::new(go_to_previous_card)
                        on_next=Callback::new(go_to_next_card)
                        on_submit=Callback::new(handle_submit_click)
                    />

                    {/* Success Message */}
                    <Show when=move || state.is_submitted.get()>
                        <div class="text-center pt-4">
                            <div class="inline-flex items-center gap-3 px-6 py-3 bg-green-50 border border-green-200 rounded-lg text-green-800">
                                <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"></path>
                                </svg>
                                "Assessment submitted successfully!"
                            </div>
                        </div>
                    </Show>
                </div>
            </div>
        </div>
    }
}
