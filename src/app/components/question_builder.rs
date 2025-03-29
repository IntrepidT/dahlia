use crate::app::models::{Question, QuestionType};
use leptos::*;
use std::str::FromStr;

const FIELD_TITLE: &str = "mt-5 font-base text-[#00356b] text-xl";
const INPUT: &str = "w-40 h-12 border-[#00356b] border pr-4 pl-6 py-4 text-[#00356b] rounded transition-all duration-1000 ease-in-out";
const INPUT_QUESTION: &str = "w-full h-12 border-[#00356b] border pr-4 pl-6 py-4 text-[#00356b] rounded transition-all duration-1000 ease-in-out";
const UNSELECTED_BUTTON: &str = "w-40 h-12 bg-gray-300 border-[#00356b] border pr-4 pl-6 py-4 text-[#00356b] rounded transition-all duration-1000 ease-in-out";
const SELECTED_BUTTON: &str = "w-40 h-12 bg-[#00356b] pr-4 pl-6 py-4 text-white rounded transition-all duration-1000 ease-in-out";
const INPUT_SELECTOR: &str ="w-45 h-12 border-[#00356b] border pr-4 pl-6 py-2 text-[#00356b] rounded transition-all duration-1000 ease-in-out";

#[component]
pub fn BuildingQuestion(
    initial_question: Question,
    on_update: Callback<Question>,
    on_remove: Callback<()>,
) -> impl IntoView {
    let (question_data, set_question_data) = create_signal(initial_question.clone());

    let update_field = move |field: &'static str, value: String| {
        set_question_data.update(|q| match field {
            "word_problem" => q.word_problem = value,
            "point_value" => q.point_value = value.parse().unwrap_or(0),
            "question_type" => {
                if !value.is_empty() {
                    let new_type = QuestionType::from_str(match value.as_str() {
                        "MultipleChoice" => "Multiple choice",
                        "Written" => "Written",
                        "Selection" => "Selection",
                        "TrueFalse" => "True False",
                        _ => "",
                    })
                    .unwrap_or(QuestionType::Selection);
                    q.question_type = new_type.clone();

                    match new_type {
                        QuestionType::TrueFalse => {
                            q.options = vec!["true".to_string(), "false".to_string()];
                            q.correct_answer = "true".to_string();
                        }
                        _ => {
                            q.options = Vec::new();
                            q.correct_answer = String::new();
                        }
                    }
                }
            }
            _ => {}
        });
        on_update(question_data());
    };

    let handle_options_update = move |(options, correct_answer): (Vec<String>, String)| {
        set_question_data.update(|q| {
            q.options = options;
            q.correct_answer = correct_answer;
        });
        on_update(question_data());
    };

    view! {
        <div class="question-builder p-4 border rounded mb-4">
            <h1 class=FIELD_TITLE>Question # {initial_question.qnumber}</h1>
            <input type="text" placeholder="Question" class=INPUT_QUESTION
                prop:value=move || question_data.with(|q| q.word_problem.clone())
                on:input=move |event| update_field("word_problem", event_target_value(&event))
            />
            <h1 class=FIELD_TITLE>Point Value</h1>
            <input type="number" placeholder="Points" class=INPUT
                prop:value=move || question_data.with(|q| q.point_value.to_string())
                on:input=move |event| update_field("point_value", event_target_value(&event))
            />
            <h1 class=FIELD_TITLE>Question Type</h1>
            <select class=INPUT_SELECTOR
                on:change=move |event| update_field("question_type", event_target_value(&event))
            >
                <option value="">"Please Select a Value"</option>
                <option value="MultipleChoice">"Multiple Choice"</option>
                <option value="Written">"Written"</option>
                <option value="Selection">"Selection"</option>
                <option value="TrueFalse">"True-False"</option>
            </select>
            {move || match question_data.with(|q| q.question_type.clone()) {
                QuestionType::MultipleChoice => {
                    let options = question_data.with(|q| q.options.clone());
                    view! {
                        <MultipleChoice
                        options=options
                        on_change=Callback::new(handle_options_update)
                        />
                    }
                },
                QuestionType::TrueFalse => {
                    let designated_answer = question_data.with(|q| q.correct_answer.clone());
                    view! {
                        <TrueFalse
                            designated_answer=designated_answer
                            on_change=Callback::new(handle_options_update)
                        />
                    }
                },
                _ => view! {<p>"Please select an option"</p>}.into_view(),
            }}
            <hr class="w-full mt-10" />
            <button
                class="bg-red-500 text-white px-2 py-1 rounded"
                on:click=move |_| on_remove(())
            >
                Remove Question
            </button>
        </div>
    }
}

#[component]
pub fn MultipleChoice(
    options: Vec<String>,
    on_change: Callback<(Vec<String>, String)>,
) -> impl IntoView {
    let (current_options, set_current_options) = create_signal(options.clone());
    let (correct_answer, set_correct_answer) =
        create_signal(options.first().cloned().unwrap_or_default());

    let add_option = move |_| {
        set_current_options.update(|opts| {
            opts.push(String::new());
            on_change((opts.clone(), correct_answer()));
        });
    };

    let create_update_option = move || {
        move |index: usize, new_value: String| {
            set_current_options.update(|opts| {
                if index < opts.len() {
                    opts[index] = new_value.clone();
                    if opts[index] == correct_answer() {
                        set_correct_answer.set(new_value);
                    }
                    on_change((opts.clone(), correct_answer()));
                }
            });
        }
    };
    let create_remove_option = move || {
        move |index: usize| {
            set_current_options.update(|opts| {
                if index < opts.len() {
                    if opts[index] == correct_answer() {
                        let new_correct = opts.first().cloned().unwrap_or_default();
                        set_correct_answer.set(new_correct);
                    }
                    opts.remove(index);
                    on_change((opts.clone(), correct_answer()));
                }
            });
        }
    };

    let set_correct_option = move |option: String| {
        set_correct_answer.set(option);
        on_change((current_options(), correct_answer()));
    };

    view! {
       <div class="mt-4 space-y-4">
           <h3 class="text-[#00356b] font-semibold">"Multiple Choice Options"</h3>
           <For
            each=move || current_options.get()
            key=|option| option.clone()
            children=move |option: String| {

                let index = current_options.with(|opts| opts.iter().position(|opt| opt == &option).unwrap_or(0));
                let update_option = create_update_option();
                let remove_option = create_remove_option();
                let option_for_correct = option.clone();

                view! {
                    <div class="flex items-center gap-2">
                        <input
                            type="text"
                            placeholder=format!("Option {}", index + 1)
                            class="flex-grow p-2 border rounded"
                            prop:value=option.clone()
                            on:input=move |event| {
                                update_option(index, event_target_value(&event))
                            }
                        />
                        <button
                            class=move || {
                                if correct_answer() == option_for_correct {
                                    "bg-green-500 text-white p-2 rounded"
                                } else {
                                    "bg-gray-200 text-gray-700 p-2 rounded"
                                }
                            }
                            on:click=move |_| set_correct_option(option.clone())
                        >
                            "Correct"
                        </button>
                        <button
                            class="bg-red-500 text-white p-2 rounded"
                            on:click=move |_| remove_option(index)
                        >
                            "Remove"
                        </button>
                    </div>
                }
            }/>

            <button
                class="bg-[#00356b] text-white p-2 rounded mt-2"
                on:click=add_option
            >
                "Add Option"
            </button>
        </div>
    }
}

#[component]
pub fn SelectionQuestion() -> impl IntoView {
    let (options_number, set_options_number) = create_signal(2);
    let on_click_add = move |_| {
        set_options_number(options_number() + 1);
    };
    view! {
        <button on:click=on_click_add class=INPUT_SELECTOR>
            "Add Selection Option"
        </button>
        <p>"This is the Selction Question Builder"</p>
        <p>"This is the value of the option counter " {options_number}</p>
    }
}

#[component]
pub fn TrueFalse(
    designated_answer: String,
    on_change: Callback<(Vec<String>, String)>,
) -> impl IntoView {
    let options = store_value(vec!["true".to_string(), "false".to_string()]);
    let (selected_answer, set_selected_answer) = create_signal(designated_answer);
    let stored_callback = store_value(on_change);

    let true_click = move |_| {
        set_selected_answer.set("true".to_string());
        stored_callback.with_value(|cb| cb.call((options.get_value(), "true".to_string())))
    };

    let false_click = move |_| {
        set_selected_answer.set("false".to_string());
        stored_callback.with_value(|cb| cb.call((options.get_value(), "false".to_string())))
    };

    view! {
        <div class="flex gap=4 items-center mt-4">
            <div class="flex-col flex gap-y-4">
                <h3 class="text-[#00356b] font-semibold">True/False Answer</h3>
                <div class="space-x-4">
                    <button
                        class=move || if selected_answer() == "true" {SELECTED_BUTTON } else {UNSELECTED_BUTTON }
                        on:click=true_click
                    >
                        "True"
                    </button>
                    <button
                        class=move || if selected_answer() == "false" {SELECTED_BUTTON} else {UNSELECTED_BUTTON }
                        on:click=false_click
                    >
                        "False"
                    </button>
                </div>
            </div>
        </div>
    }
}
