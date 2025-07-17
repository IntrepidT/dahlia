use crate::app::models::{Question, QuestionType, WeightedOption};
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
                        "WeightedMultipleChoice" => "Weighted Multiple Choice",
                        _ => "",
                    })
                    .unwrap_or(QuestionType::TrueFalse);
                    q.question_type = new_type.clone();

                    match new_type {
                        QuestionType::TrueFalse => {
                            q.options = vec!["true".to_string(), "false".to_string()];
                            q.correct_answer = "true".to_string();
                            q.weighted_options = None;
                        }
                        QuestionType::WeightedMultipleChoice => {
                            q.options = Vec::new();
                            q.correct_answer = String::new();
                            // Initialize with some default weighted options if none exist
                            if q.weighted_options.is_none() || q.get_weighted_options().is_empty() {
                                let default_options = vec![
                                    WeightedOption::new("Option 1".to_string(), 1, true),
                                    WeightedOption::new("Option 2".to_string(), 1, true),
                                ];
                                q.set_weighted_options(default_options);
                            }
                        }
                        _ => {
                            q.options = Vec::new();
                            q.correct_answer = String::new();
                            q.weighted_options = None;
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

    let handle_weighted_options_update = move |weighted_options: Vec<WeightedOption>| {
        set_question_data.update(|q| {
            q.set_weighted_options(weighted_options.clone());
            // Update regular options and correct_answer for compatibility
            q.options = weighted_options
                .iter()
                .map(|opt| opt.text.clone())
                .collect();
            // For weighted questions, correct_answer can be a JSON array of selectable options
            let selectable_options: Vec<String> = weighted_options
                .iter()
                .filter(|opt| opt.is_selectable)
                .map(|opt| opt.text.clone())
                .collect();
            q.correct_answer = serde_json::to_string(&selectable_options).unwrap_or_default();
        });
        on_update(question_data());
    };

    // Function to convert QuestionType to dropdown value - Fixed to match dropdown options
    let question_type_to_value = move |question_type: &QuestionType| -> String {
        match question_type {
            QuestionType::MultipleChoice => "MultipleChoice".to_string(),
            QuestionType::Written => "Written".to_string(),
            QuestionType::Selection => "Selection".to_string(),
            QuestionType::TrueFalse => "TrueFalse".to_string(),
            QuestionType::WeightedMultipleChoice => "WeightedMultipleChoice".to_string(),
        }
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
                prop:value=move || question_data.with(|q| question_type_to_value(&q.question_type))
                on:change=move |event| update_field("question_type", event_target_value(&event))
            >
                <option value="">Please Select a Value</option>
                <option value="MultipleChoice">Multiple Choice</option>
                /*<option value="Written">Written</option>
                <option value="Selection">Selection</option>*/
                <option value="TrueFalse">True-False</option>
                <option value="WeightedMultipleChoice">Weighted Multiple Choice</option>
            </select>
            {move || match question_data.with(|q| q.question_type.clone()) {
                QuestionType::MultipleChoice => {
                    let options = question_data.with(|q| q.options.clone());
                    let correct_answer = question_data.with(|q| q.correct_answer.clone());
                    view! {
                        <MultipleChoice
                        options=options
                        designated_answer=correct_answer
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
                QuestionType::WeightedMultipleChoice => {
                    let weighted_options = question_data.with(|q| q.get_weighted_options());
                    let max_points = question_data.with(|q| q.point_value);
                    view! {
                        <WeightedMultipleChoice
                            weighted_options=weighted_options
                            max_points=max_points
                            on_change=Callback::new(handle_weighted_options_update)
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
pub fn WeightedMultipleChoice(
    weighted_options: Vec<WeightedOption>,
    max_points: i32,
    on_change: Callback<Vec<WeightedOption>>,
) -> impl IntoView {
    // Generate initial IDs for tracking
    let next_id = std::cell::Cell::new(0);
    let get_next_id = move || {
        let id = next_id.get();
        next_id.set(id + 1);
        id
    };

    // Create initial options with IDs
    let initial_options = weighted_options
        .into_iter()
        .map(|opt| (get_next_id(), opt))
        .collect::<Vec<_>>();

    let (option_items, set_option_items) = create_signal(initial_options);

    // Calculate total assigned points
    let total_assigned_points = create_memo(move |_| {
        option_items.with(|items| {
            items
                .iter()
                .filter(|(_, opt)| opt.is_selectable)
                .map(|(_, opt)| opt.points)
                .sum::<i32>()
        })
    });

    // Create a debounced update callback
    let debounced_update = store_value(move || {
        let options =
            option_items.with(|items| items.iter().map(|(_, opt)| opt.clone()).collect::<Vec<_>>());
        on_change(options);
    });

    // Add a new option
    let add_option = move |_| {
        set_option_items.update(|items| {
            items.push((get_next_id(), WeightedOption::new(String::new(), 0, false)));
        });
        debounced_update.with_value(|update| update());
    };

    // Update option text
    let update_option_text = move |id: usize, new_text: String| {
        set_option_items.update(|items| {
            if let Some((_, option)) = items.iter_mut().find(|(item_id, _)| *item_id == id) {
                option.text = new_text;
            }
        });
    };

    // Update option points
    let update_option_points = move |id: usize, new_points: i32| {
        set_option_items.update(|items| {
            if let Some((_, option)) = items.iter_mut().find(|(item_id, _)| *item_id == id) {
                option.points = new_points;
            }
        });
        debounced_update.with_value(|update| update());
    };

    // Toggle option selectability
    let toggle_selectable = move |id: usize| {
        set_option_items.update(|items| {
            if let Some((_, option)) = items.iter_mut().find(|(item_id, _)| *item_id == id) {
                option.is_selectable = !option.is_selectable;
            }
        });
        debounced_update.with_value(|update| update());
    };

    // Remove option
    let remove_option = move |id: usize| {
        set_option_items.update(|items| {
            items.retain(|(item_id, _)| *item_id != id);
        });
        debounced_update.with_value(|update| update());
    };

    // Handle blur events
    let on_text_blur = move |_| {
        debounced_update.with_value(|update| update());
    };

    view! {
        <div class="mt-4 space-y-4">
            <div class="flex justify-between items-center">
                <h3 class="text-[#00356b] font-semibold">"Weighted Multiple Choice Options"</h3>
                <div class="text-sm text-gray-600">
                    "Total Points Assigned: "
                    <span class=move || {
                        if total_assigned_points() > max_points {
                            "font-bold text-red-600"
                        } else {
                            "font-bold text-green-600"
                        }
                    }>
                        {total_assigned_points}
                    </span>
                    " / " {max_points}
                </div>
            </div>

            <div class="bg-blue-50 border border-blue-200 rounded p-3 text-sm text-blue-800">
                <p><strong>"Instructions:"</strong></p>
                <ul class="list-disc list-inside mt-1 space-y-1">
                    <li>"Set point values for each option (can be positive or negative)"</li>
                    <li>"Check 'Selectable' for options students can choose"</li>
                    <li>"Students can select multiple options simultaneously"</li>
                    <li>"Final score = sum of selected option points (capped at question max)"</li>
                </ul>
            </div>

            <For
                each=move || option_items.get()
                key=|(id, _)| *id
                children=move |(id, option)| {
                    let option_id = id;
                    let option_clone = option.clone();

                    view! {
                        <div class="bg-gray-50 border border-gray-200 rounded p-4 space-y-3">
                            <div class="flex items-center gap-3">
                                <div class="flex-grow">
                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                        "Option Text"
                                    </label>
                                    <input
                                        type="text"
                                        placeholder=format!("Option {}", option_id + 1)
                                        class="w-full p-2 border border-gray-300 rounded focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                        prop:value=option_clone.text.clone()
                                        on:input=move |event| {
                                            update_option_text(option_id, event_target_value(&event))
                                        }
                                        on:blur=move |_| on_text_blur(option_id)
                                    />
                                </div>
                                <div class="w-24">
                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                        "Points"
                                    </label>
                                    <input
                                        type="number"
                                        class="w-full p-2 border border-gray-300 rounded focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                        prop:value=option_clone.points.to_string()
                                        on:input=move |event| {
                                            let value = event_target_value(&event).parse().unwrap_or(0);
                                            update_option_points(option_id, value);
                                        }
                                    />
                                </div>
                            </div>

                            <div class="flex items-center justify-between">
                                <label class="flex items-center cursor-pointer">
                                    <input
                                        type="checkbox"
                                        class="mr-2 h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                                        prop:checked=option_clone.is_selectable
                                        on:change=move |_| toggle_selectable(option_id)
                                    />
                                    <span class="text-sm text-gray-700">"Selectable by students"</span>
                                </label>

                                <button
                                    class="bg-red-500 text-white px-3 py-1 rounded text-sm hover:bg-red-600 transition-colors"
                                    on:click=move |_| remove_option(option_id)
                                >
                                    "Remove"
                                </button>
                            </div>
                        </div>
                    }
                }
            />

            <button
                class="bg-[#00356b] text-white px-4 py-2 rounded hover:bg-blue-700 transition-colors"
                on:click=add_option
            >
                "Add Option"
            </button>

            // Warning if points exceed maximum
            {move || {
                if total_assigned_points() > max_points {
                    view! {
                        <div class="bg-yellow-50 border border-yellow-200 rounded p-3 text-sm text-yellow-800">
                            <p><strong>"Warning:"</strong> " Total selectable points exceed the question's maximum value. Student scores will be capped at " {max_points} " points."</p>
                        </div>
                    }.into_view()
                } else {
                    view! { <div></div> }.into_view()
                }
            }}
        </div>
    }
}

#[component]
pub fn MultipleChoice(
    options: Vec<String>,
    designated_answer: String,
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
    let (correct_answer, set_correct_answer) = create_signal(if designated_answer.is_empty() {
        option_items.with(|items| items.first().map(|(_, v)| v.clone()).unwrap_or_default())
    } else {
        designated_answer
    });

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

                // If this was the correct answer, update the correct_answer signal
                if correct_answer() == item.1 {
                    set_correct_answer.set(new_value);
                }
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
            set_correct_answer.set(value);
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
                   // Clone the value for use in closures
                   let option_value = value.clone();
                   let option_value_cloned = option_value.clone();

                   // Create a derived signal that checks if this option is the correct answer
                   let is_correct = create_memo(move |_| correct_answer() == option_value_cloned);

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
                                   if is_correct() {
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
