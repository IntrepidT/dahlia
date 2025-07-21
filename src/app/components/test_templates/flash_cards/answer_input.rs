use super::flash_card_state::QuestionResponse;
use crate::app::components::test_components::font_controls::FontSettings;
use crate::app::models::question::{Question, QuestionType};
use leptos::*;
use std::collections::HashMap;

#[component]
pub fn AnswerInput(
    question: Question,
    #[prop(into)] responses: Signal<HashMap<i32, QuestionResponse>>,
    #[prop(into)] font_settings: Signal<FontSettings>,
    #[prop(into)] on_answer_change: Callback<(i32, String)>,
    #[prop(into)] on_weighted_selection: Callback<(i32, Vec<String>)>,
    #[prop(into)] disabled: Signal<bool>,
) -> impl IntoView {
    let qnumber = question.qnumber;

    view! {
        <div class="space-y-4">
            {move || {
                let q = question.clone();
                match q.question_type {
                    QuestionType::MultipleChoice => render_multiple_choice(
                        q, responses, font_settings, on_answer_change, disabled
                    ).into_view(),
                    QuestionType::WeightedMultipleChoice => render_weighted_multiple_choice(
                        q, responses, font_settings, on_weighted_selection, disabled
                    ).into_view(),
                    QuestionType::TrueFalse => render_true_false(
                        q, responses, font_settings, on_answer_change, disabled
                    ).into_view(),
                    _ => render_text_input(
                        q, responses, font_settings, on_answer_change, disabled
                    ).into_view(),
                }
            }}
        </div>
    }
}

fn render_multiple_choice(
    question: Question,
    responses: Signal<HashMap<i32, QuestionResponse>>,
    font_settings: Signal<FontSettings>,
    on_answer_change: Callback<(i32, String)>,
    disabled: Signal<bool>,
) -> impl IntoView {
    let qnumber = question.qnumber;

    view! {
        <div class="space-y-2">
            {question.options.clone().into_iter().enumerate().map(|(index, option)| {
                let option_value = option.clone();
                let option_value_clone = option_value.clone();
                let choice_number = index + 1;
                let is_checked = create_memo(move |_| {
                    responses.with(|r| {
                        r.get(&qnumber)
                         .map(|resp| resp.answer == option_value_clone.clone())
                         .unwrap_or(false)
                    })
                });

                view! {
                    <label class="group flex items-start gap-3 p-3 rounded-lg border border-gray-200 hover:border-blue-300 hover:bg-blue-50/50 transition-all duration-200 cursor-pointer">
                        <div class="relative flex-shrink-0 mt-0.5">
                            <input
                                type="radio"
                                name=format!("q_{}", qnumber)
                                value=option_value.clone()
                                class="sr-only"
                                prop:checked=move || is_checked()
                                prop:disabled=move || disabled.get()
                                on:change=move |ev| {
                                    if !disabled.get() {
                                        let value = event_target_value(&ev);
                                        on_answer_change.call((qnumber, value));
                                    }
                                }
                            />
                            <div class=move || {
                                if is_checked() {
                                    "w-5 h-5 rounded-full border-2 border-blue-500 bg-blue-500 flex items-center justify-center"
                                } else {
                                    "w-5 h-5 rounded-full border-2 border-gray-300 group-hover:border-blue-400 transition-colors"
                                }
                            }>
                                <Show when=move || is_checked()>
                                    <div class="w-2 h-2 bg-white rounded-full"></div>
                                </Show>
                            </div>
                        </div>
                        <div class="flex-1 flex items-start gap-3">
                            <span class="text-xs text-gray-500 font-medium mt-1 min-w-[1rem]">
                                {choice_number}
                            </span>
                            <span class=move || format!("leading-relaxed {}", font_settings.get().get_answer_classes())>
                                {option_value}
                            </span>
                        </div>
                    </label>
                }
            }).collect_view()}
        </div>
    }
}

fn render_weighted_multiple_choice(
    question: Question,
    responses: Signal<HashMap<i32, QuestionResponse>>,
    font_settings: Signal<FontSettings>,
    on_weighted_selection: Callback<(i32, Vec<String>)>,
    disabled: Signal<bool>,
) -> impl IntoView {
    let qnumber = question.qnumber;
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
                            if option_clone.is_selectable && !disabled.get() {
                                format!("{} border-gray-200 hover:border-blue-300 hover:bg-blue-50/50 cursor-pointer", base_classes)
                            } else {
                                format!("{} border-gray-200 bg-gray-50 cursor-not-allowed opacity-60", base_classes)
                            }
                        }
                        on:click=move |_| {
                            if option_clone.is_selectable && !disabled.get() {
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
                                    <span class=move || format!("leading-relaxed {}", font_settings.get().get_answer_classes())>
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
    }
}

fn render_true_false(
    question: Question,
    responses: Signal<HashMap<i32, QuestionResponse>>,
    font_settings: Signal<FontSettings>,
    on_answer_change: Callback<(i32, String)>,
    disabled: Signal<bool>,
) -> impl IntoView {
    let qnumber = question.qnumber;
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
        <div class="flex gap-4">
            <button
                type="button"
                class=move || format!("flex-1 py-3 px-4 rounded-lg font-medium transition-all duration-200 flex items-center justify-center gap-2 {}",
                    if is_true() {
                        "bg-green-500 text-white shadow-lg transform scale-105"
                    } else {
                        "bg-white text-gray-700 border-2 border-gray-200 hover:border-green-400 hover:bg-green-50"
                    }
                )
                prop:disabled=move || disabled.get()
                on:click=move |_| {
                    if !disabled.get() {
                        on_answer_change.call((qnumber, "true".to_string()));
                    }
                }
            >
                <span class="text-xs text-gray-500 font-medium">1</span>
                <span class=move || font_settings.get().get_answer_classes()>
                    "True"
                </span>
            </button>
            <button
                type="button"
                class=move || format!("flex-1 py-3 px-4 rounded-lg font-medium transition-all duration-200 flex items-center justify-center gap-2 {}",
                    if is_false() {
                        "bg-red-500 text-white shadow-lg transform scale-105"
                    } else {
                        "bg-white text-gray-700 border-2 border-gray-200 hover:border-red-400 hover:bg-red-50"
                    }
                )
                prop:disabled=move || disabled.get()
                on:click=move |_| {
                    if !disabled.get() {
                        on_answer_change.call((qnumber, "false".to_string()));
                    }
                }
            >
                <span class="text-xs text-gray-500 font-medium">2</span>
                <span class=move || font_settings.get().get_answer_classes()>
                    "False"
                </span>
            </button>
        </div>
    }
}

fn render_text_input(
    question: Question,
    responses: Signal<HashMap<i32, QuestionResponse>>,
    font_settings: Signal<FontSettings>,
    on_answer_change: Callback<(i32, String)>,
    disabled: Signal<bool>,
) -> impl IntoView {
    let qnumber = question.qnumber;
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
                class=move || format!("w-full p-3 border border-gray-200 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 resize-none transition-all duration-200 {}",
                    font_settings.get().get_answer_classes())
                prop:value=move || answer_value()
                prop:disabled=move || disabled.get()
                on:input=move |ev| {
                    if !disabled.get() {
                        let value = event_target_value(&ev);
                        on_answer_change.call((qnumber, value));
                    }
                }
                placeholder="Type your answer here..."
                rows="3"
            ></textarea>
        </div>
    }
}
