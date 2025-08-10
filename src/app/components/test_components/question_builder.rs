use crate::app::models::{Question, QuestionType, WeightedOption};
use leptos::html;
use leptos::prelude::*;
use std::rc::Rc;
use std::str::FromStr;

#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

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
    on_duplicate: Option<Callback<Question>>,
    should_auto_focus: Memo<bool>,   // Changed from ReadSignal to Memo
    on_focus_complete: Callback<()>, // New callback to clear auto-focus
) -> impl IntoView {
    let (question_data, set_question_data) = signal(initial_question.clone());
    let question_input_ref = NodeRef::<html::Textarea>::new();

    // Store initial_question for use in different closures
    let initial_question_clone = initial_question.clone();
    let initial_question_for_header = initial_question.clone();

    // Effect to update question_data when initial_question changes (for edit mode)
    Effect::new(move |_| {
        let current_initial = initial_question_clone.clone();
        set_question_data.update(|q| {
            // Only update if the question actually changed to avoid infinite loops
            if q.testlinker != current_initial.testlinker
                || q.qnumber != current_initial.qnumber
                || (q.word_problem.is_empty() && !current_initial.word_problem.is_empty())
            {
                *q = current_initial;
            }
        });
    });

    // Improved auto-focus mechanism with cleanup
    Effect::new(move |_| {
        if should_auto_focus() {
            if let Some(input) = question_input_ref.get() {
                #[cfg(feature = "hydrate")]
                {
                    request_animation_frame(move || {
                        let focus_result = input.focus();
                        if focus_result.is_ok() {
                            // Call the completion callback after successful focus
                            on_focus_complete.run(());
                        }
                    });
                }
                #[cfg(not(feature = "hydrate"))]
                {
                    // On server side, just call the completion callback immediately
                    on_focus_complete.run(());
                }
            }
        }
    });

    let update_field = move |field: &'static str, value: String| {
        set_question_data.update(|q| match field {
            "word_problem" => q.word_problem = value,
            "point_value" => q.point_value = value.parse().unwrap_or(1), // Default to 1 instead of 0
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
                    .unwrap_or(QuestionType::MultipleChoice);

                    // Only update if the type is actually different to avoid unnecessary resets
                    if q.question_type != new_type {
                        q.question_type = new_type.clone();

                        match new_type {
                            QuestionType::TrueFalse => {
                                q.options = vec!["true".to_string(), "false".to_string()];
                                q.correct_answer = "true".to_string();
                                q.weighted_options = None;
                            }
                            QuestionType::MultipleChoice => {
                                // Preserve existing options if they exist and are valid
                                if q.options.is_empty() || q.weighted_options.is_some() {
                                    q.options = vec!["".to_string(), "".to_string()];
                                }
                                // Set first option as correct if no correct answer or if coming from weighted
                                if q.correct_answer.is_empty() || q.weighted_options.is_some() {
                                    q.correct_answer =
                                        q.options.first().cloned().unwrap_or_default();
                                }
                                q.weighted_options = None;
                            }
                            QuestionType::WeightedMultipleChoice => {
                                // Clear regular options when switching to weighted
                                q.options = Vec::new();
                                q.correct_answer = String::new();
                                if q.weighted_options.is_none()
                                    || q.get_weighted_options().is_empty()
                                {
                                    let default_options = vec![
                                        WeightedOption::new("".to_string(), 1, true),
                                        WeightedOption::new("".to_string(), 1, true),
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
            }
            _ => {}
        });
        on_update.run(question_data());
    };

    let handle_options_update = move |(options, correct_answer): (Vec<String>, String)| {
        set_question_data.update(|q| {
            q.options = options;
            q.correct_answer = correct_answer;
        });
        on_update.run(question_data());
    };

    let handle_weighted_options_update = move |weighted_options: Vec<WeightedOption>| {
        set_question_data.update(|q| {
            q.set_weighted_options(weighted_options.clone());
            q.options = weighted_options
                .iter()
                .map(|opt| opt.text.clone())
                .collect();
            let selectable_options: Vec<String> = weighted_options
                .iter()
                .filter(|opt| opt.is_selectable)
                .map(|opt| opt.text.clone())
                .collect();
            q.correct_answer = serde_json::to_string(&selectable_options).unwrap_or_default();
        });
        on_update.run(question_data());
    };

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
        <div class="question-builder p-6 border rounded-lg mb-6 bg-white shadow-sm hover:shadow-md transition-shadow">
            // Question header with actions
            <div class="flex justify-between items-center mb-4">
                <h3 class="text-lg font-semibold text-[#00356b]">
                    "Question " {initial_question.qnumber}
                </h3>
                <div class="flex space-x-2">
                    {move || {
                        if let Some(duplicate_callback) = on_duplicate {
                            view! {
                                <button
                                    type="button"
                                    class="px-3 py-1 text-sm bg-blue-100 text-blue-700 rounded hover:bg-blue-200 transition-colors"
                                    on:click=move |_| duplicate_callback.run(question_data.get())
                                    title="Duplicate this question"
                                >
                                    "Duplicate"
                                </button>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }}
                    <button
                        type="button"
                        class="px-3 py-1 text-sm bg-red-100 text-red-700 rounded hover:bg-red-200 transition-colors"
                        on:click=move |_| on_remove.run(())
                        title="Remove this question"
                    >
                        "Remove"
                    </button>
                </div>
            </div>

            // Question text with enhanced input
            <div class="mb-4">
                <label class="block text-sm font-medium text-gray-700 mb-2">
                    "Question Text"
                    <span class="text-red-500">"*"</span>
                </label>
                <textarea
                    node_ref=question_input_ref
                    placeholder="Enter your question here..."
                    class="w-full h-20 px-4 py-3 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all resize-none"
                    prop:value=move || question_data.with(|q| q.word_problem.clone())
                    on:input=move |event| update_field("word_problem", event_target_value(&event))
                ></textarea>
            </div>

            // Point value and question type in a row
            <div class="grid grid-cols-2 gap-4 mb-4">
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                        "Point Value" <span class="text-red-500">"*"</span>
                    </label>
                    <input
                        type="number"
                        min="1"
                        placeholder="Points"
                        class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                        prop:value=move || question_data.with(|q| q.point_value.to_string())
                        on:input=move |event| update_field("point_value", event_target_value(&event))
                        on:focus=move |event| {
                            // Select all text when focused - only on client side
                            #[cfg(feature = "hydrate")]
                            {
                                if let Some(target) = event.target() {
                                    if let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() {
                                        let _ = input.select();
                                    }
                                }
                            }
                        }
                    />
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                        "Question Type" <span class="text-red-500">"*"</span>
                    </label>
                    <select
                        class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                        prop:value=move || question_data.with(|q| question_type_to_value(&q.question_type))
                        on:change=move |event| update_field("question_type", event_target_value(&event))
                    >
                        // Most common types first
                        <option value="MultipleChoice">"Multiple Choice"</option>
                        <option value="TrueFalse">"True/False"</option>
                        <option value="WeightedMultipleChoice">"Weighted Multiple Choice"</option>
                    </select>
                </div>
            </div>

            // Question type specific content
            <div class="border-t pt-4">
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
                        }.into_any()
                    },
                    QuestionType::TrueFalse => {
                        let designated_answer = question_data.with(|q| q.correct_answer.clone());
                        view! {
                            <TrueFalse
                                designated_answer=designated_answer
                                on_change=Callback::new(handle_options_update)
                            />
                        }.into_any()
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
                        }.into_any()
                    },
                    _ => view! {
                        <div class="bg-gray-50 border border-gray-200 rounded p-4 text-center text-gray-500">
                            "Please select a question type to continue"
                        </div>
                    }.into_any(),
                }}
            </div>
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

    let (option_items, set_option_items) = signal(initial_options);

    // Calculate total assigned points
    let total_assigned_points = Memo::new(move |_| {
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
        on_change.run(options);
    });

    // Add a new option
    let add_option = move |_| {
        let new_id = get_next_id();
        set_option_items.update(|items| {
            items.push((new_id, WeightedOption::new(String::new(), 0, false)));
        });
        debounced_update.with_value(|update| update());

        // Focus the newly added input with better timing
        #[cfg(feature = "hydrate")]
        {
            request_animation_frame(move || {
                request_animation_frame(move || {
                    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                        // Use data attribute for more reliable selection
                        let selector = &format!("input[data-weighted-option-id='{}']", new_id);
                        if let Ok(Some(element)) = document.query_selector(selector) {
                            if let Ok(input) = element.dyn_into::<web_sys::HtmlInputElement>() {
                                let _ = input.focus();
                            }
                        }
                    }
                });
            });
        }
    };

    // Update option text
    let update_option_text = move |id: usize, new_text: String| {
        set_option_items.update(|items| {
            if let Some((_, option)) = items.iter_mut().find(|(item_id, _)| *item_id == id) {
                option.text = new_text;
            }
        });
        // No immediate parent notification
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
                each=move || {
                    option_items.get()
                        .into_iter()
                        .enumerate()
                        .collect::<Vec<_>>()
                }
                key=|(index, (id, _))| (*index, *id)
                children=move |(index, (id, option)): (usize, (usize, WeightedOption))| {
                    let option_id = id;
                    let option_index = index;
                    let option_text = option.text.clone();
                    let option_points = option.points;
                    let option_selectable = option.is_selectable;

                    view! {
                        <div class="flex items-center gap-3 p-3 bg-gray-50 rounded-lg">
                            // Text input for option - natural tab order
                            <input
                                type="text"
                                placeholder=format!("Option {}", option_index + 1)
                                class="flex-grow px-3 py-2 border border-gray-300 rounded focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                prop:value=option_text
                                on:input=move |event| update_option_text(option_id, event_target_value(&event))
                                // No parent notifications, no blur handlers, no debouncing
                                // Use data attribute for more reliable selection
                                attr:data-weighted-option-id=option_id.to_string()
                                // NO tabindex - let browser use natural DOM order
                            />

                            // Points input - natural tab order
                            <div class="flex items-center space-x-2">
                                <label class="text-sm text-gray-600">"Points:"</label>
                                <input
                                    type="number"
                                    class="w-20 px-2 py-1 border border-gray-300 rounded text-center focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                    prop:value=option_points.to_string()
                                    on:input=move |event| {
                                        if let Ok(points) = event_target_value(&event).parse::<i32>() {
                                            update_option_points(option_id, points);
                                        }
                                    }
                                    // NO tabindex - let browser use natural DOM order
                                />
                            </div>

                            // Selectable checkbox - natural tab order
                            <div class="flex items-center space-x-2">
                                <label class="text-sm text-gray-600">"Selectable:"</label>
                                <input
                                    type="checkbox"
                                    class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                    prop:checked=option_selectable
                                    on:change=move |_| toggle_selectable(option_id)
                                    // NO tabindex - let browser use natural DOM order
                                />
                            </div>

                            // Remove button - excluded from tab order
                            <button
                                type="button"
                                class="flex-shrink-0 p-2 text-red-600 hover:bg-red-100 rounded-full transition-colors"
                                on:click=move |_| {
                                    remove_option(option_id);
                                    debounced_update.with_value(|update| update()); // Only notify on remove
                                }
                                title="Remove option"
                                // Remove buttons from tab order
                                prop:tabindex=-1
                            >
                                <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd"/>
                                </svg>
                            </button>
                        </div>
                    }
                }
            />

            <button
                type="button"
                class="bg-[#00356b] text-white px-4 py-2 rounded hover:bg-blue-700 transition-colors"
                on:click=add_option
                // Ensure add button doesn't interfere with tab order
                prop:tabindex=-1
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
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
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
    let next_id = std::cell::Cell::new(0);
    let get_next_id = move || {
        let id = next_id.get();
        next_id.set(id + 1);
        id
    };

    let initial_options = options
        .into_iter()
        .map(|value| (get_next_id(), value))
        .collect::<Vec<_>>();

    let (option_items, set_option_items) = signal(initial_options);
    let (correct_answer, set_correct_answer) = signal(if designated_answer.is_empty() {
        option_items.with(|items| items.first().map(|(_, v)| v.clone()).unwrap_or_default())
    } else {
        designated_answer
    });

    let notify_parent = move || {
        let values =
            option_items.with(|items| items.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>());
        on_change.run((values, correct_answer()));
    };

    let add_option = move |_| {
        let new_id = get_next_id();
        set_option_items.update(|items| {
            items.push((new_id, String::new()));
        });
        notify_parent();

        // Focus the newly added input with better timing
        #[cfg(feature = "hydrate")]
        {
            request_animation_frame(move || {
                request_animation_frame(move || {
                    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                        // Use a more specific selector that targets the actual input
                        let selector = &format!("input[data-option-id='{}']", new_id);
                        if let Ok(Some(element)) = document.query_selector(selector) {
                            if let Ok(input) = element.dyn_into::<web_sys::HtmlInputElement>() {
                                let _ = input.focus();
                            }
                        }
                    }
                });
            });
        }
    };

    let update_option = move |id: usize, new_value: String| {
        set_option_items.update(|items| {
            if let Some(item) = items.iter_mut().find(|(item_id, _)| *item_id == id) {
                let old_value = item.1.clone();
                item.1 = new_value.clone();

                // If this was the correct answer, update it
                if correct_answer() == old_value {
                    set_correct_answer.set(new_value);
                }
            }
        });
        // No immediate parent notification - let the parent component handle this
    };

    let remove_option = move |id: usize| {
        set_option_items.update(|items| {
            let removing_correct = items
                .iter()
                .find(|(item_id, _)| *item_id == id)
                .map(|(_, value)| value == &correct_answer())
                .unwrap_or(false);

            items.retain(|(item_id, _)| *item_id != id);

            if removing_correct && !items.is_empty() {
                let new_correct = items[0].1.clone();
                set_correct_answer.set(new_correct);
            }
        });
        notify_parent();
    };

    let set_as_correct = move |id: usize| {
        if let Some(value) = option_items.with(|items| {
            items
                .iter()
                .find(|(item_id, _)| *item_id == id)
                .map(|(_, v)| v.clone())
        }) {
            set_correct_answer.set(value);
            notify_parent();
        }
    };

    view! {
        <div class="space-y-3">
            <div class="flex justify-between items-center">
                <h4 class="font-medium text-gray-700">"Answer Options"</h4>
                <span class="text-sm text-gray-500">"Select the correct answer"</span>
            </div>

            <For
                each=move || {
                    option_items.get()
                        .into_iter()
                        .enumerate()
                        .collect::<Vec<_>>()
                }
                key=|(index, (id, _))| (*index, *id)
                children=move |(index, (id, value)): (usize, (usize, String))| {
                    let option_id = id;
                    let option_index = index;
                    let option_value_for_memo = value.clone();
                    let option_value_for_input = value.clone();

                    let is_correct = Memo::new(move |_| correct_answer() == option_value_for_memo);

                    view! {
                        <div class=move || {
                            if is_correct() {
                                "flex items-center gap-3 p-3 bg-green-50 border border-green-200 rounded-lg"
                            } else {
                                "flex items-center gap-3 p-3 bg-gray-50 rounded-lg"
                            }
                        }>
                            <div class="flex-shrink-0">
                                <input
                                    type="radio"
                                    name="correct_answer"
                                    class="h-4 w-4 text-blue-600 focus:ring-blue-500"
                                    prop:checked=is_correct.get()
                                    on:change=move |_| {
                                        set_as_correct(option_id);
                                        notify_parent(); // Only notify on radio button changes
                                    }
                                    // Remove radio buttons from tab order completely
                                    prop:tabindex=-1
                                />
                            </div>
                            <input
                                type="text"
                                placeholder=format!("Option {}", option_index + 1)
                                class="flex-grow px-3 py-2 border border-gray-300 rounded focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                prop:value=option_value_for_input
                                on:input=move |event| update_option(option_id, event_target_value(&event))
                                // No parent notifications, no blur handlers, no debouncing
                                // Use data attribute for more reliable selection
                                attr:data-option-id=option_id.to_string()
                                // NO tabindex - let browser use natural DOM order
                            />
                            <button
                                type="button"
                                class="flex-shrink-0 p-2 text-red-600 hover:bg-red-100 rounded-full transition-colors"
                                on:click=move |_| {
                                    remove_option(option_id);
                                    notify_parent(); // Only notify on remove
                                }
                                title="Remove option"
                                // Remove buttons from tab order
                                prop:tabindex=-1
                            >
                                <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd"/>
                                </svg>
                            </button>
                        </div>
                    }
                }
            />

            <button
                type="button"
                class="w-full py-2 px-4 border-2 border-dashed border-gray-300 rounded-lg text-gray-600 hover:border-blue-500 hover:text-blue-600 transition-colors"
                on:click=add_option
                // Ensure add button doesn't interfere with tab order
                prop:tabindex=-1
            >
                "+ Add Option"
            </button>
        </div>
    }
}

#[component]
pub fn SelectionQuestion() -> impl IntoView {
    let (options_number, set_options_number) = signal(2);
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
    let (selected_answer, set_selected_answer) = signal(if designated_answer.is_empty() {
        "true".to_string()
    } else {
        designated_answer
    });

    let update_answer = move |answer: String| {
        set_selected_answer.set(answer.clone());
        on_change.run((options.get_value(), answer));
    };

    view! {
        <div class="space-y-4">
            <h4 class="font-medium text-gray-700">"Select the correct answer:"</h4>
            <div class="flex gap-4">
                <button
                    type="button"
                    class=move || {
                        if selected_answer() == "true" {
                            "px-8 py-3 bg-green-500 text-white rounded-lg font-medium transition-all"
                        } else {
                            "px-8 py-3 bg-gray-200 text-gray-700 rounded-lg font-medium hover:bg-gray-300 transition-all"
                        }
                    }
                    on:click=move |_| update_answer("true".to_string())
                >
                    "True"
                </button>
                <button
                    type="button"
                    class=move || {
                        if selected_answer() == "false" {
                            "px-8 py-3 bg-green-500 text-white rounded-lg font-medium transition-all"
                        } else {
                            "px-8 py-3 bg-gray-200 text-gray-700 rounded-lg font-medium hover:bg-gray-300 transition-all"
                        }
                    }
                    on:click=move |_| update_answer("false".to_string())
                >
                    "False"
                </button>
            </div>
        </div>
    }
}
