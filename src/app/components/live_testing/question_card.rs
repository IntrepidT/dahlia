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
    #[prop(into)] on_weighted_selection: Callback<(i32, Vec<String>)>,
) -> impl IntoView {
    let question_for_answer = question.clone();
    let question_for_comment = question.clone();

    view! {
        <div class="bg-white rounded-xl shadow-lg overflow-hidden w-full max-w-4xl" style="min-height: 450px;">
            <div class="p-8 flex flex-col justify-start items-center w-full h-full overflow-y-auto">
                {/* Question Section */}
                <div class="text-center w-full overflow-auto mb-6">
                    <p class="text-4xl sm:text-3xl font-bold text-gray-800 break-words mb-8">
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
                            on_weighted_selection=on_weighted_selection
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
    #[prop(into)] on_weighted_selection: Callback<(i32, Vec<String>)>,
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
        QuestionType::WeightedMultipleChoice => {
            let weighted_options = question.get_weighted_options();
            let q_clone_for_calc = question.clone();

            view! {
                <div class="space-y-3">
                    <div class="bg-blue-50 border border-blue-200 rounded-lg p-3 text-sm text-blue-800">
                        <p><strong>"Multiple selections allowed."</strong> " Each answer has different point values."</p>
                    </div>

                    <div class="space-y-2">
                        {weighted_options.clone().into_iter().enumerate().map(|(index, option)| {
                            let option_clone = option.clone();
                            let option_text = option.text.clone();
                            let option_text_for_memo = option_text.clone();
                            let option_text_for_change = option_text.clone();
                            let choice_number = index + 1;

                            let is_selected = create_memo(move |_| {
                                responses.with(|r| {
                                    r.get(&qnumber)
                                        .and_then(|resp| resp.selected_options.as_ref())
                                        .map(|opts| opts.contains(&option_text_for_memo))
                                        .unwrap_or(false)
                                })
                            });

                            view! {
                                <div class=move || {
                                    let base_classes = "group flex items-center justify-between p-3 rounded-lg border transition-all duration-200";
                                    if option_clone.is_selectable {
                                        format!("{} border-gray-200 hover:border-blue-300 hover:bg-blue-50/50 cursor-pointer", base_classes)
                                    } else {
                                        format!("{} border-gray-200 bg-gray-50 cursor-not-allowed opacity-60", base_classes)
                                    }
                                }
                                on:click=move |_| {
                                    if option_clone.is_selectable && !should_disable_inputs.get() {
                                        let current_selected = responses.with(|r| {
                                            r.get(&qnumber)
                                                .and_then(|resp| resp.selected_options.as_ref())
                                                .cloned()
                                                .unwrap_or_default()
                                        });

                                        let mut new_selected = current_selected;
                                        if new_selected.contains(&option_text_for_change) {
                                            new_selected.retain(|x| x != &option_text_for_change);
                                        } else {
                                            new_selected.push(option_text_for_change.clone());
                                        }

                                        on_weighted_selection.call((qnumber, new_selected));
                                    }
                                }>
                                    <div class="flex items-center gap-3">
                                        <div class="relative flex-shrink-0">
                                            {if option_clone.is_selectable {
                                                view! {
                                                    <div class=move || {
                                                        if is_selected() {
                                                            "w-5 h-5 rounded border-2 border-blue-500 bg-blue-500 flex items-center justify-center"
                                                        } else {
                                                            "w-5 h-5 rounded border-2 border-gray-300 group-hover:border-blue-400 transition-colors"
                                                        }
                                                    }>
                                                        <Show when=move || is_selected()>
                                                            <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                                                                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                                                            </svg>
                                                        </Show>
                                                    </div>
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <div class="w-5 h-5 rounded border-2 border-gray-300 bg-gray-100"></div>
                                                }.into_view()
                                            }}
                                        </div>
                                        <div class="flex items-start gap-3">
                                            <span class="text-xs text-gray-500 font-medium mt-1 min-w-[1rem]">
                                                {choice_number}
                                            </span>
                                            <span class="leading-relaxed break-words">
                                                {option_clone.text.clone()}
                                            </span>
                                        </div>
                                    </div>
                                    <div class="flex items-center gap-2">
                                        <span class=move || {
                                            if option_clone.points >= 0 {
                                                "text-green-600 font-semibold text-sm"
                                            } else {
                                                "text-red-600 font-semibold text-sm"
                                            }
                                        }>
                                            {if option_clone.points >= 0 { "+" } else { "" }}
                                            {option_clone.points}
                                            " pts"
                                        </span>
                                        {if !option_clone.is_selectable {
                                            view! {
                                                <span class="text-xs text-gray-400 italic">"(info only)"</span>
                                            }.into_view()
                                        } else {
                                            view! { <span></span> }.into_view()
                                        }}
                                    </div>
                                </div>
                            }
                        }).collect_view()}
                    </div>

                    <div class="bg-gray-50 border border-gray-200 rounded-lg p-3">
                        <div class="text-sm text-gray-700">
                            "Current score: "
                            <span class="font-semibold text-indigo-600">
                                {move || {
                                    let selected = responses.with(|r| {
                                        r.get(&qnumber)
                                            .and_then(|resp| resp.selected_options.as_ref())
                                            .cloned()
                                            .unwrap_or_default()
                                    });
                                    q_clone_for_calc.calculate_weighted_score(&selected)
                                }}
                                " / " {question.point_value} " points"
                            </span>
                        </div>
                    </div>
                </div>
            }.into_view()
        },
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
                        "True"
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
                        "False"
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
