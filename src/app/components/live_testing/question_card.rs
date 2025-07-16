use super::types::{QuestionResponse, Role};
use crate::app::models::question::{Question, QuestionType};
use leptos::*;
use std::collections::HashMap;

#[component]
pub fn QuestionCard(
    question: Question,
    #[prop(into)] role: Signal<Role>,
    #[prop(into)] responses: Signal<HashMap<i32, QuestionResponse>>,
    #[prop(into)] should_disable_inputs: Signal<bool>,
    #[prop(into)] on_answer_change: Callback<(i32, String)>,
    #[prop(into)] on_comment_change: Callback<(i32, String)>,
) -> impl IntoView {
    // Clone question for use in different closures
    let question_for_answer = question.clone();
    let question_for_comment = question.clone();

    view! {
        <div class="bg-white rounded-xl shadow-lg overflow-hidden w-full max-w-2xl" style="min-height: 450px;">
            <div class="p-8 flex flex-col justify-start items-center w-full h-full overflow-y-auto">
                {/* Question Section */}
                <div class="text-center w-full overflow-auto mb-6">
                    <p class="text-4xl sm:text-3xl font-bold text-gray-800 break-words mb-8 font-custom">
                        {question.word_problem.clone()}
                    </p>
                </div>

                {/* Answer Section */}
                <Show when=move || matches!(role.get(), Role::Teacher)>
                    <div class="w-full mt-2">
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            "Your Answer:"
                        </label>
                        <AnswerInput
                            question=question_for_answer.clone()
                            responses=responses
                            should_disable_inputs=should_disable_inputs
                            on_answer_change=on_answer_change
                        />
                    </div>
                </Show>

                {/* Teacher Comments Section */}
                <Show when=move || matches!(role.get(), Role::Teacher)>
                    <div class="w-full mt-4">
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            "Teacher Comments:"
                        </label>
                        <CommentInput
                            question=question_for_comment.clone()
                            responses=responses
                            on_comment_change=on_comment_change
                        />
                    </div>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn AnswerInput(
    question: Question,
    #[prop(into)] responses: Signal<HashMap<i32, QuestionResponse>>,
    #[prop(into)] should_disable_inputs: Signal<bool>,
    #[prop(into)] on_answer_change: Callback<(i32, String)>,
) -> impl IntoView {
    let qnumber = question.qnumber;

    match question.question_type {
        QuestionType::MultipleChoice => view! {
            <div class="space-y-2 max-h-48 overflow-y-auto">
                <For
                    each=move || question.options.clone()
                    key=|option| option.clone()
                    children=move |option| {
                        let option_value = option.clone();
                        let option_value_clone = option_value.clone();
                        let is_checked = create_memo(move |_| {
                            responses.with(|r| {
                                r.get(&qnumber)
                                 .map(|resp| resp.answer == option_value_clone.clone())
                                 .unwrap_or(false)
                            })
                        });

                        view! {
                            <label class="flex items-center p-3 rounded-lg border border-gray-200 hover:border-blue-400 hover:bg-blue-50 transition-colors cursor-pointer">
                                <input
                                    type="radio"
                                    name=format!("q_{}", qnumber)
                                    value=option_value.clone()
                                    class="h-4 w-4 text-blue-600 focus:ring-blue-500"
                                    prop:checked=move || is_checked()
                                    prop:disabled=should_disable_inputs.get()
                                    on:change=move |ev| {
                                        if !should_disable_inputs.get() {
                                            let value = event_target_value(&ev);
                                            on_answer_change.call((qnumber, value));
                                        }
                                    }
                                />
                                <span class="ml-2 break-words">{option_value}</span>
                            </label>
                        }
                    }
                />
            </div>
        }.into_view(),
        QuestionType::TrueFalse => {
            let is_true = create_memo(move |_| {
                responses.with(|r| {
                    r.get(&qnumber)
                     .map(|resp| resp.answer == "true")
                     .unwrap_or(false)
                })
            });
            let is_false = create_memo(move |_| {
                responses.with(|r| {
                    r.get(&qnumber)
                     .map(|resp| resp.answer == "false")
                     .unwrap_or(false)
                })
            });

            view! {
                <div class="w-full flex flex-col sm:flex-row gap-4 items-center justify-center">
                    <button
                        type="button"
                        class="px-6 py-3 w-full rounded-lg font-medium text-center transition-colors"
                        class:bg-white={move || !is_true()}
                        class:text-gray-800={move || !is_true()}
                        class:border-gray-200={move || !is_true()}
                        class:border={move || !is_true()}
                        class:bg-green-500={move || is_true()}
                        class:text-white={move || is_true()}
                        class:border-transparent={move || is_true()}
                        class:cursor-not-allowed={should_disable_inputs()}
                        on:click=move |_| {
                            if !should_disable_inputs.get() {
                                on_answer_change.call((qnumber, "true".to_string()));
                            }
                        }
                    >
                        "Yes"
                    </button>
                    <button
                        type="button"
                        class="px-6 py-3 w-full rounded-lg font-medium text-center transition-colors"
                        class:bg-white={move || !is_false()}
                        class:text-gray-800={move || !is_false()}
                        class:border-gray-200={move || !is_false()}
                        class:border={move || !is_false()}
                        class:bg-red-500={move || is_false()}
                        class:text-white={move || is_false()}
                        class:border-transparent={move || is_false()}
                        class:cursor-not-allowed={should_disable_inputs()}
                        on:click=move |_| {
                            if !should_disable_inputs.get() {
                                on_answer_change.call((qnumber, "false".to_string()));
                            }
                        }
                    >
                        "No"
                    </button>
                </div>
            }.into_view()
        },
        _ => {
            let answer_value = create_memo(move |_| {
                responses.with(|r| {
                    r.get(&qnumber)
                     .map(|resp| resp.answer.clone())
                     .unwrap_or_default()
                })
            });

            view! {
                <div>
                    <textarea
                        class="w-full p-3 border border-gray-200 rounded-lg focus:ring-blue-500 focus:border-blue-500"
                        prop:value=move || answer_value()
                        prop:disabled={should_disable_inputs()}
                        on:input=move |ev| {
                            if !should_disable_inputs.get() {
                                let value = event_target_value(&ev);
                                on_answer_change.call((qnumber, value));
                            }
                        }
                        placeholder="Enter your answer here..."
                        rows="3"
                    ></textarea>
                </div>
            }.into_view()
        }
    }
}

#[component]
fn CommentInput(
    question: Question,
    #[prop(into)] responses: Signal<HashMap<i32, QuestionResponse>>,
    #[prop(into)] on_comment_change: Callback<(i32, String)>,
) -> impl IntoView {
    let qnumber = question.qnumber;
    let comment_value = create_memo(move |_| {
        responses.with(|r| {
            r.get(&qnumber)
                .map(|resp| resp.comment.clone())
                .unwrap_or_default()
        })
    });

    view! {
        <div>
            <textarea
                class="w-full p-3 border border-gray-200 rounded-lg focus:ring-blue-500 focus:border-blue-500"
                prop:value=move || comment_value()
                on:input=move |ev| {
                    let value = event_target_value(&ev);
                    on_comment_change.call((qnumber, value));
                }
                placeholder="Add teacher comments or notes here..."
                rows="2"
            ></textarea>
        </div>
    }
}
