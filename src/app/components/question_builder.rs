use crate::app::models::{Question, QuestionType};
use leptos::*;
use std::rc::Rc;
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
            <h1 class=FIELD_TITLE>Question: </h1>
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
    // Generate initial IDs
    let next_id = std::cell::Cell::new(0);
    let get_next_id = move || {
        let id = next_id.get();
        next_id.set(id + 1);
        id
    };

    // Create initial options with IDs
    let initial_options = options
        .into_iter()
        .map(|value| (get_next_id(), value))
        .collect::<Vec<_>>();

    // Create signals
    let (option_items, set_option_items) = create_signal(initial_options);
    let (correct_answer, set_correct_answer) = create_signal(
        option_items.with(|items| items.first().map(|(_, v)| v.clone()).unwrap_or_default()),
    );

    // Create a debounced update callback to reduce renders
    let debounced_update = store_value(move || {
        // Only call this when we want to notify the parent
        let values =
            option_items.with(|items| items.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>());
        on_change((values, correct_answer()));
    });

    // Add a new option
    let add_option = move |_| {
        set_option_items.update(|items| {
            items.push((get_next_id(), String::new()));
        });
        // Update callback after adding
        debounced_update.with_value(|update| update());
    };

    // Create a stored callback for updating options
    let update_option = move |id: usize, new_value: String| {
        set_option_items.update(|items| {
            if let Some(item) = items.iter_mut().find(|(item_id, _)| *item_id == id) {
                item.1 = new_value.clone();
            }
        });
        // Don't update the parent on every keystroke
        // We'll update on blur or other significant events
    };

    // Handle option blur - this is when we'll notify the parent
    let on_option_blur = move |_| {
        debounced_update.with_value(|update| update());
    };

    // Handle removing options
    let remove_option = move |id: usize| {
        set_option_items.update(|items| {
            // Check if we're removing the correct answer
            let removing_correct = items
                .iter()
                .find(|(item_id, _)| *item_id == id)
                .map(|(_, value)| value == &correct_answer())
                .unwrap_or(false);

            // Remove the item
            items.retain(|(item_id, _)| *item_id != id);

            // Update correct answer if needed
            if removing_correct && !items.is_empty() {
                let new_correct = items[0].1.clone();
                set_correct_answer.set(new_correct);
            }
        });
        // Update callback after removing
        debounced_update.with_value(|update| update());
    };

    // Set the correct answer
    let set_as_correct = move |id: usize| {
        if let Some(value) = option_items.with(|items| {
            items
                .iter()
                .find(|(item_id, _)| *item_id == id)
                .map(|(_, v)| v.clone())
        }) {
            set_correct_answer.set(value.clone());
            // Update callback after changing correct answer
            debounced_update.with_value(|update| update());
        }
    };

    view! {
       <div class="mt-4 space-y-4">
           <h3 class="text-[#00356b] font-semibold">"Multiple Choice Options"</h3>
           <For
               each=move || option_items.get()
               key=|(id, _)| *id
               children=move |(id, value)| {
                   let option_id = id;
                   let option_value = value.clone();

                   view! {
                       <div class="flex items-center gap-2">
                           <input
                               type="text"
                               placeholder=format!("Option {}", option_id + 1)
                               class="flex-grow p-2 border rounded"
                               prop:value=option_value.clone()
                               on:input=move |event| {
                                   update_option(option_id, event_target_value(&event))
                               }
                               on:blur=move |_| on_option_blur(option_id)
                           />
                           <button
                               class=move || {
                                   if correct_answer() == option_value {
                                       "bg-green-500 text-white p-2 rounded"
                                   } else {
                                       "bg-gray-200 text-gray-700 p-2 rounded"
                                   }
                               }
                               on:click=move |_| set_as_correct(option_id)
                           >
                               "Correct"
                           </button>
                           <button
                               class="bg-red-500 text-white p-2 rounded"
                               on:click=move |_| remove_option(option_id)
                           >
                               "Remove"
                           </button>
                       </div>
                   }
               }
           />

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
