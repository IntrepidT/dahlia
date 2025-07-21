use super::flash_card_state::FlashCardState;
use crate::app::models::question::{Question, QuestionType};
use leptos::*;

#[cfg(feature = "hydrate")]
use leptos::ev::KeyboardEvent;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

pub fn use_keyboard_handler(
    state: FlashCardState,
    questions: Vec<Question>,
    handle_answer_change: impl Fn(i32, String) + Clone + 'static,
    handle_weighted_selection: impl Fn(i32, Vec<String>) + Clone + 'static,
    focus_comments: impl Fn() + Clone + 'static,
    handle_submit_click: impl Fn() + Clone + 'static,
    set_current_card_index: WriteSignal<usize>,
) {
    #[cfg(feature = "hydrate")]
    {
        create_effect(move |_| {
            let questions_clone = questions.clone();
            let handle_answer_change_clone = handle_answer_change.clone();
            let handle_weighted_selection_clone = handle_weighted_selection.clone();
            let focus_comments_clone = focus_comments.clone();
            let handle_submit_click_clone = handle_submit_click.clone();

            let handle_keydown = move |ev: KeyboardEvent| {
                let target = ev.target().unwrap();
                let tag_name = target
                    .unchecked_ref::<web_sys::Element>()
                    .tag_name()
                    .to_lowercase();

                // Handle Tab to blur from textarea/input
                if ev.key().as_str() == "Tab" && (tag_name == "textarea" || tag_name == "input") {
                    if let Some(html_element) = target.dyn_ref::<web_sys::HtmlElement>() {
                        let _ = html_element.blur();
                        ev.prevent_default();
                    }
                    return;
                }

                // Only handle navigation shortcuts when not typing in input fields
                if tag_name == "input" || tag_name == "textarea" || tag_name == "select" {
                    return;
                }

                match ev.key().as_str() {
                    "ArrowRight" | "n" | "N" => {
                        ev.prevent_default();
                        set_current_card_index.update(|index| {
                            *index = (*index + 1).min(questions_clone.len() - 1);
                        });
                    }
                    "ArrowLeft" | "p" | "P" => {
                        ev.prevent_default();
                        set_current_card_index.update(|index| {
                            *index = index.saturating_sub(1);
                        });
                    }
                    "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                        if !ev.ctrl_key() && !ev.alt_key() && !ev.meta_key() {
                            ev.prevent_default();
                            if let Ok(num) = ev.key().parse::<usize>() {
                                let current_index = state.current_card_index.get();
                                if let Some(current_question) = questions_clone.get(current_index) {
                                    match current_question.question_type {
                                        QuestionType::MultipleChoice => {
                                            if num <= current_question.options.len() {
                                                let option =
                                                    current_question.options[num - 1].clone();
                                                handle_answer_change_clone(
                                                    current_question.qnumber,
                                                    option,
                                                );
                                            }
                                        }
                                        QuestionType::WeightedMultipleChoice => {
                                            let weighted_options =
                                                current_question.get_weighted_options();
                                            if num <= weighted_options.len() {
                                                let option = &weighted_options[num - 1];
                                                if option.is_selectable {
                                                    let current_selected =
                                                        state.responses.with(|r| {
                                                            r.get(&current_question.qnumber)
                                                                .and_then(|resp| {
                                                                    resp.selected_options.as_ref()
                                                                })
                                                                .cloned()
                                                                .unwrap_or_default()
                                                        });

                                                    let mut new_selected = current_selected;
                                                    if new_selected.contains(&option.text) {
                                                        new_selected.retain(|x| x != &option.text);
                                                    } else {
                                                        new_selected.push(option.text.clone());
                                                    }

                                                    handle_weighted_selection_clone(
                                                        current_question.qnumber,
                                                        new_selected,
                                                    );
                                                }
                                            }
                                        }
                                        QuestionType::TrueFalse => {
                                            if num == 1 {
                                                handle_answer_change_clone(
                                                    current_question.qnumber,
                                                    "true".to_string(),
                                                );
                                            } else if num == 2 {
                                                handle_answer_change_clone(
                                                    current_question.qnumber,
                                                    "false".to_string(),
                                                );
                                            }
                                        }
                                        _ => {
                                            // For other question types, jump to question
                                            if num > 0 && num <= questions_clone.len() {
                                                set_current_card_index.set(num - 1);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    "c" | "C" => {
                        if !ev.ctrl_key() && !ev.alt_key() && !ev.meta_key() {
                            ev.prevent_default();
                            focus_comments_clone();
                        }
                    }
                    "Enter" => {
                        if ev.ctrl_key() || ev.meta_key() {
                            ev.prevent_default();
                            let current_index = state.current_card_index.get();
                            if current_index == questions_clone.len() - 1
                                && !state.is_submitted.get()
                            {
                                // Submit on last question
                                if state.selected_student_id.get().is_some() {
                                    handle_submit_click_clone();
                                }
                            } else {
                                set_current_card_index.update(|index| {
                                    *index = (*index + 1).min(questions_clone.len() - 1);
                                });
                            }
                        }
                    }
                    _ => {}
                }
            };

            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();

            let closure = wasm_bindgen::closure::Closure::wrap(
                Box::new(handle_keydown) as Box<dyn Fn(KeyboardEvent)>
            );

            document
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
                .unwrap();

            closure.forget();
        });
    }
}
