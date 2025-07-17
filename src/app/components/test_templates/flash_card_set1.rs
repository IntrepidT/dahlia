use crate::app::components::auth::enhanced_login_form::{
    use_student_mapping_service, DeAnonymizedStudent,
};
use crate::app::middleware::global_settings::use_settings;
use crate::app::models::question::QuestionType;
use crate::app::models::score::CreateScoreRequest;
use crate::app::models::student::Student;
use crate::app::models::test::Test;
use crate::app::models::user::SessionUser;
use crate::app::server_functions::students::get_students;
use crate::app::server_functions::{
    questions::get_questions, scores::add_score, tests::get_tests, users::get_user,
};
use leptos::*;
use leptos_router::*;
use std::collections::HashMap;

#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

#[derive(Debug, Clone)]
struct QuestionResponse {
    answer: String,
    comment: String,
    selected_options: Option<Vec<String>>,
}
impl QuestionResponse {
    fn new() -> Self {
        Self {
            answer: String::new(),
            comment: String::new(),
            selected_options: None,
        }
    }
}

#[component]
pub fn FlashCardSet() -> impl IntoView {
    // Get test_id from URL parameters
    let params = use_params_map();
    let test_id = move || params.with(|params| params.get("test_id").cloned().unwrap_or_default());
    let user = use_context::<ReadSignal<Option<SessionUser>>>().expect("AuthProvider not Found");
    let user_data = create_resource(
        move || user.get().map(|u| u.id),
        move |id| async move {
            match id {
                Some(user_id) => match get_user(user_id).await {
                    Ok(user) => Some(user),
                    Err(e) => {
                        log::error!("Failed to fetch user data: {}", e);
                        None
                    }
                },
                None => None,
            }
        },
    );

    // Create resource to fetch test details
    let test_details = create_resource(test_id.clone(), move |tid| async move {
        if tid.is_empty() {
            log::warn!("No test ID provided in URL");
            return None;
        }
        match get_tests().await {
            Ok(tests) => tests.into_iter().find(|test| test.test_id == tid),
            Err(e) => {
                log::error!("Failed to fetch test details: {}", e);
                None
            }
        }
    });

    // Create resource that depends on the test_id from URL
    let questions = create_resource(test_id, move |tid| async move {
        if tid.is_empty() {
            log::warn!("No test ID provided in URL");
            return Vec::new();
        }
        match get_questions(tid).await {
            Ok(questions) => questions,
            Err(e) => {
                log::error!("Failed to fetch questions: {}", e);
                Vec::new()
            }
        }
    });

    // Store responses for each question with memo to prevent unnecessary re-renders
    let (responses, set_responses) = create_signal(HashMap::<i32, QuestionResponse>::new());
    let (selected_student_id, set_selected_student_id) = create_signal(None::<i32>);

    // Flashcard state
    let (current_card_index, set_current_card_index) = create_signal(0);
    let (is_submitted, set_is_submitted) = create_signal(false);

    // Get evaluator ID
    let evaluator_id = create_memo(move |_| match user.get() {
        Some(user_data) => user_data.id.to_string(),
        None => "0".to_string(),
    });

    // Handler for answer updates - using a local memo to prevent full re-renders
    let handle_answer_change = move |qnumber: i32, value: String| {
        set_responses.update(|r| {
            let response = r.entry(qnumber).or_insert_with(QuestionResponse::new);
            response.answer = value;
        });
    };

    //Handler for weighted multiple choice selection
    let handle_weighted_selection = move |qnumber: i32, selected_options: Vec<String>| {
        set_responses.update(|r| {
            let response = r.entry(qnumber).or_insert_with(QuestionResponse::new);
            response.selected_options = Some(selected_options.clone()); // Use correct field name
                                                                        // Also update answer field with JSON for compatibility
            response.answer = serde_json::to_string(&selected_options).unwrap_or_default();
        });
    };

    // Handler for comment updates
    let handle_comment_change = move |qnumber: i32, value: String| {
        set_responses.update(|r| {
            let response = r.entry(qnumber).or_insert_with(QuestionResponse::new);
            response.comment = value;
        });
    };

    // Navigation handlers
    let go_to_next_card = move |_| {
        set_current_card_index.update(|index| {
            if let Some(questions_vec) = questions.get() {
                *index = (*index + 1).min(questions_vec.len() - 1);
            }
        });
    };

    let go_to_previous_card = move |_| {
        set_current_card_index.update(|index| {
            *index = index.saturating_sub(1);
        });
    };

    // Submit handler
    let handle_submit = create_action(move |_: &()| async move {
        let current_responses = responses.get();
        let current_test_id = test_id();

        let student_id = selected_student_id.get().unwrap_or(0);
        let evaluator = evaluator_id();
        let test_variant = 1;

        let mut test_scores = Vec::new();
        let mut comments = Vec::new();

        if let Some(questions) = questions.get() {
            let mut sorted_questions = questions.clone();
            sorted_questions.sort_by_key(|q| q.qnumber);

            for question in sorted_questions {
                if let Some(response) = current_responses.get(&question.qnumber) {
                    let score = match question.question_type {
                        QuestionType::WeightedMultipleChoice => {
                            // Calculate weighted score
                            if let Some(ref selected_opts) = response.selected_options {
                                // Use correct field name
                                question.calculate_weighted_score(selected_opts)
                            } else {
                                0
                            }
                        }
                        _ => {
                            // Regular scoring logic
                            if response.answer == question.correct_answer {
                                question.point_value
                            } else {
                                0
                            }
                        }
                    };

                    test_scores.push(score);
                    comments.push(response.comment.clone());
                } else {
                    test_scores.push(0);
                    comments.push(String::new());
                }
            }
        }

        let score_request = CreateScoreRequest {
            student_id,
            test_id: current_test_id,
            test_scores,
            comments,
            test_variant,
            evaluator,
        };

        match add_score(score_request).await {
            Ok(score) => {
                log::info!(
                    "Successfully submitted score for student {}",
                    score.student_id
                );
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to submit score: {}", e);
                Err(e)
            }
        }
    });

    // Memoize the percentage calculation to avoid recalculating on every render
    let calculate_answered_percentage = create_memo(move |_| {
        let answered_count = responses.with(|r| {
            questions
                .get()
                .map(|q| {
                    q.iter()
                        .filter(|question| {
                            r.get(&question.qnumber)
                                .map(|resp| !resp.answer.trim().is_empty())
                                .unwrap_or(false)
                        })
                        .count() as f32
                })
                .unwrap_or(0.0)
        });

        let total = questions.get().map(|q| q.len() as f32).unwrap_or(1.0);
        (answered_count / total) * 100.0
    });

    view! {
        <div class="p-4 max-w-screen h-screen overflow-y-auto bg-gray-50 mx-auto">
            {/* Header with Student Selection and Evaluator */}
            <div class="flex flex-wrap items-center justify-between mb-8 max-w-4xl mx-auto">
                <div class="w-full md:w-1/2 mb-4 md:mb-0">
                    <StudentSelect set_selected_student_id=set_selected_student_id />
                </div>
                <div class="text-sm text-gray-600 font-medium">
                    {"Evaluator: "}
                    {move || match user_data.get() {
                        Some(Some(user)) => format!("{} {}", user.first_name.unwrap_or("None".to_string()), user.last_name.unwrap_or("None".to_string())),
                        Some(None) => evaluator_id(),
                        None => "Loading...".to_string(),
                    }}
                </div>
            </div>

            {/* Test Title */}
            <div class="text-center mb-8">
                <h2 class="text-xl font-medium text-gray-700 break-words">
                    {move || match &test_details.get() {
                        Some(Some(test)) => test.name.clone(),
                        _ => test_id()
                    }}
                </h2>
            </div>

            {/* Questions View */}
            <Suspense
                fallback=move || view! { <div class="flex justify-center items-center h-64">
                    <div class="animate-pulse bg-white rounded-lg shadow-md w-full max-w-2xl h-64 flex items-center justify-center">
                        <p class="text-gray-400">"Loading questions..."</p>
                    </div>
                </div> }
            >
                {move || match (questions.get(), test_details.get()) {
                    (None, _) => view! { <div class="text-center py-8">"Loading..."</div> }.into_view(),
                    (Some(questions), _) if questions.is_empty() => {
                        view! { <div class="text-center py-8 text-red-500">"No questions found for this test ID."</div> }.into_view()
                    },
                    (Some(questions), _) => {
                        let total_questions = questions.len();

                        // Create a memo to get the current question
                        let current_question = create_memo(move |_| {
                            questions.get(current_card_index.get()).cloned().unwrap_or_else(|| {
                                log::warn!("Question index out of bounds");
                                questions.first().cloned().unwrap_or_else(|| panic!("No questions available"))
                            })
                        });

                        view! {
                            <div class="flex flex-col items-center justify-center">
                                {/* Flash Card */}
                                <div class="relative w-full max-w-2xl transition-all duration-300 my-2">
                                    {/* Progress Bar */}
                                    <div class="mb-4 w-full bg-gray-200 rounded-full h-2.5">
                                        <div
                                            class="bg-gradient-to-r from-blue-500 to-purple-600 h-2.5 rounded-full transition-all duration-1500 ease-in"
                                            style=move || format!("width: {}%", calculate_answered_percentage())
                                        ></div>
                                    </div>

                                    {/* Card Counter */}
                                    <div class="text-center mb-4">
                                        <span class="inline-flex items-center justify-center bg-white text-sm font-medium text-gray-700 px-3 py-1 rounded-full shadow-sm border border-gray-200">
                                            {move || current_card_index.get() + 1}
                                            {" / "}
                                            {total_questions}
                                            <span class="ml-2 text-purple-600 font-semibold">
                                                {move || current_question().point_value}
                                                {" pts"}
                                            </span>
                                        </span>
                                    </div>

                                    {/* Unified Flash Card - No flipping */}
                                    <div
                                        class="bg-white rounded-xl shadow-lg overflow-hidden"
                                        style="min-height: 450px;"
                                    >
                                        <div class="p-8 flex flex-col justify-start items-center w-full h-full overflow-y-auto">
                                            {/* Question Section */}
                                            <div class="text-center w-full overflow-auto mb-6">
                                                //to fix the sizing for another user, we need to
                                                //reduce the font size to text-4xl or smth for sm:
                                                <p class="text-sm sm:text-9xl text-gray-800 break-words mb-8 font-custom font-bold">
                                                    {move || current_question().word_problem.clone()}
                                                </p>
                                            </div>

                                            {/* Answer Section - Using a local view! to isolate renders */}
                                            <div class="w-full mt-2">
                                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                                    "Your Answer:"
                                                </label>
                                                {move || {
                                                    let q = current_question();
                                                    let q_clone_for_calc = q.clone(); // Clone for the calculation closure
                                                    let q_point_value = q.point_value;
                                                    match q.question_type {
                                                        QuestionType::MultipleChoice => view! {
                                                            <div class="space-y-2 max-h-48 overflow-y-auto">
                                                                <For
                                                                    each=move || q.options.clone()
                                                                    key=|option| option.clone()
                                                                    children=move |option| {
                                                                        let option_value = option.clone();
                                                                        let option_value_clone = option_value.clone();
                                                                        let qnumber = q.qnumber;
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
                                                                                    on:change=move |ev| {
                                                                                        let value = event_target_value(&ev);
                                                                                        handle_answer_change(qnumber, value);
                                                                                    }
                                                                                />
                                                                                <span class="ml-2 break-words">{option_value}</span>
                                                                            </label>
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                        },
                                                        QuestionType::WeightedMultipleChoice => {
                                                            let qnumber = q.qnumber;
                                                            let weighted_options = q.get_weighted_options();

                                                            view! {
                                                                <div class="space-y-3">
                                                                    <div class="bg-blue-50 border border-blue-200 rounded p-3 text-sm text-blue-800">
                                                                        <p><strong>"Instructions:"</strong> " You can select multiple answers. Each answer has different point values."</p>
                                                                    </div>

                                                                    <div class="space-y-2 max-h-64 overflow-y-auto">
                                                                        <For
                                                                            each=move || weighted_options.clone()
                                                                            key=|option| option.text.clone()
                                                                            children=move |option| {
                                                                                let option_clone = option.clone();
                                                                                let option_text = option.text.clone();
                                                                                let option_text_for_memo = option_text.clone();
                                                                                let option_text_for_change = option_text.clone();
                                                                                let qnumber = q.qnumber;

                                                                                let is_selected = create_memo(move |_| {
                                                                                    responses.with(|r| {
                                                                                        r.get(&qnumber)
                                                                                            .and_then(|resp| resp.selected_options.as_ref())
                                                                                            .map(|opts| opts.contains(&option_text_for_memo))
                                                                                            .unwrap_or(false)
                                                                                    })
                                                                                });

                                                                                // Show ALL options, but only make selectable ones clickable
                                                                                view! {
                                                                                    <div class=move || {
                                                                                        if option_clone.is_selectable {
                                                                                            "flex items-center justify-between p-3 rounded-lg border border-gray-200 hover:border-blue-400 hover:bg-blue-50 transition-colors cursor-pointer"
                                                                                        } else {
                                                                                            "flex items-center justify-between p-3 rounded-lg border border-gray-300 bg-gray-100 transition-colors cursor-not-allowed opacity-60"
                                                                                        }
                                                                                    }
                                                                                    on:click=move |_| {
                                                                                        if option_clone.is_selectable {
                                                                                            // Toggle this option in the selected list
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

                                                                                            handle_weighted_selection(qnumber, new_selected);
                                                                                        }
                                                                                    }>
                                                                                        <div class="flex items-center">
                                                                                            {if option_clone.is_selectable {
                                                                                                view! {
                                                                                                    <input
                                                                                                        type="checkbox"
                                                                                                        class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded pointer-events-none"
                                                                                                        prop:checked=move || is_selected()
                                                                                                        tabindex="-1"
                                                                                                    />
                                                                                                }.into_view()
                                                                                            } else {
                                                                                                view! {
                                                                                                    <div class="h-4 w-4 border border-gray-400 rounded bg-gray-200"></div>
                                                                                                }.into_view()
                                                                                            }}
                                                                                            <span class="ml-2 break-words">{option_clone.text.clone()}</span>
                                                                                        </div>
                                                                                        <div class="flex items-center space-x-2">
                                                                                            <span class=move || {
                                                                                                if option_clone.points >= 0 {
                                                                                                    "text-green-600 font-semibold"
                                                                                                } else {
                                                                                                    "text-red-600 font-semibold"
                                                                                                }
                                                                                            }>
                                                                                                {if option_clone.points >= 0 { "+" } else { "" }}
                                                                                                {option_clone.points}
                                                                                                " pts"
                                                                                            </span>
                                                                                            {if !option_clone.is_selectable {
                                                                                                view! {
                                                                                                    <span class="text-xs text-gray-500 italic">"(info only)"</span>
                                                                                                }.into_view()
                                                                                            } else {
                                                                                                view! { <span></span> }.into_view()
                                                                                            }}
                                                                                        </div>
                                                                                    </div>
                                                                                }
                                                                            }
                                                                        />
                                                                    </div>

                                                                    // Show current score calculation
                                                                    <div class="bg-gray-50 border border-gray-200 rounded p-3">
                                                                        <div class="text-sm text-gray-700">
                                                                            "Current selection score: "
                                                                            <span class="font-semibold text-blue-600">
                                                                                {move || {
                                                                                    let selected = responses.with(|r| {
                                                                                        r.get(&qnumber)
                                                                                            .and_then(|resp| resp.selected_options.as_ref())
                                                                                            .cloned()
                                                                                            .unwrap_or_default()
                                                                                    });
                                                                                    q_clone_for_calc.calculate_weighted_score(&selected)
                                                                                }}
                                                                                " / " {q_point_value} " points"
                                                                            </span>
                                                                        </div>
                                                                    </div>
                                                                </div>
                                                            }
                                                        }
                                                        QuestionType::TrueFalse => {
                                                            let qnumber = q.qnumber;
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
                                                                        on:click=move |_| {
                                                                            handle_answer_change(qnumber, "true".to_string());
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
                                                                        on:click=move |_| {
                                                                            handle_answer_change(qnumber, "false".to_string());
                                                                        }
                                                                    >
                                                                        "No"
                                                                    </button>
                                                                </div>
                                                            }
                                                        },
                                                        _ => {
                                                            let qnumber = q.qnumber;
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
                                                                        on:input=move |ev| {
                                                                            let value = event_target_value(&ev);
                                                                            handle_answer_change(qnumber, value);
                                                                        }
                                                                        placeholder="Enter your answer here..."
                                                                        rows="3"
                                                                    ></textarea>
                                                                </div>
                                                            }
                                                        }
                                                    }
                                                }}
                                            </div>

                                            {/* Comments Section - THIS NEEDS SPECIAL ATTENTION TO FIX THE FOCUS ISSUE */}
                                            <div class="w-full mt-4">
                                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                                    "Comments:"
                                                </label>
                                                {move || {
                                                    let qnumber = current_question().qnumber;

                                                    // Create a memo for the comment value to prevent unnecessary re-renders
                                                    let comment_value = create_memo(move |_| {
                                                        responses.with(|r| {
                                                            r.get(&qnumber)
                                                             .map(|resp| resp.comment.clone())
                                                             .unwrap_or_default()
                                                        })
                                                    });

                                                    view! {
                                                        <textarea
                                                            class="w-full p-3 border border-gray-200 rounded-lg focus:ring-blue-500 focus:border-blue-500"
                                                            prop:value=move || comment_value()
                                                            on:input=move |ev| {
                                                                let value = event_target_value(&ev);
                                                                handle_comment_change(qnumber, value);
                                                            }
                                                            placeholder="Add any comments or notes here..."
                                                            rows="2"
                                                        ></textarea>
                                                    }
                                                }}
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                {/* Navigation Buttons */}
                                <div class="flex flex-wrap items-center justify-center gap-4 mt-8">
                                    <button
                                        class="flex items-center justify-center px-5 py-2 bg-white border border-gray-200 rounded-lg shadow-sm text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                                        disabled=move || current_card_index.get() == 0
                                        on:click=go_to_previous_card
                                    >
                                        <span class="mr-1">"←"</span>
                                        "Previous"
                                    </button>

                                    {move || {
                                        if current_card_index.get() == total_questions - 1 {
                                            view! {
                                                <Show when=move || !is_submitted.get()>
                                                    <button
                                                        class="flex items-center justify-center px-5 py-2 bg-gradient-to-r from-blue-600 to-purple-600 text-white rounded-lg shadow-sm hover:from-blue-700 hover:to-purple-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                                        on:click=move |_| {
                                                            handle_submit.dispatch(());
                                                            set_is_submitted.set(true);
                                                        }
                                                        disabled=move || selected_student_id.get().is_none()
                                                    >
                                                        "Submit Assessment"
                                                        <span class="ml-1">"✓"</span>
                                                    </button>
                                                </Show>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <div>
                                                    <button
                                                        class="flex items-center justify-center px-5 py-2 bg-gradient-to-r from-blue-600 to-purple-600 text-white rounded-lg shadow-sm hover:from-blue-700 hover:to-purple-700 transition-colors"
                                                        on:click=go_to_next_card
                                                    >
                                                        "Next"
                                                        <span class="ml-1">"→"</span>
                                                    </button>
                                                </div>
                                            }.into_view()
                                        }
                                    }}
                                </div>

                                {/* Submission Status */}
                                {move || {
                                    if is_submitted.get() {
                                        view! {
                                            <div class="mt-8 text-center">
                                                <div class="inline-flex items-center px-4 py-2 rounded-full bg-green-100 text-green-800 mb-4">
                                                    <span class="mr-2">"✓"</span>
                                                    "Assessment submitted successfully!"
                                                </div>
                                                <div>
                                                    <button
                                                        class="px-5 py-2 mt-2 bg-gray-800 text-white rounded-lg hover:bg-gray-700 transition-colors"
                                                        on:click=move |_| {
                                                            let navigate=leptos_router::use_navigate();
                                                            navigate("/dashboard", Default::default());
                                                        }
                                                    >
                                                        "Return to Dashboard"
                                                    </button>
                                                </div>
                                            </div>
                                        }
                                    } else {
                                        view! {<div></div>}
                                    }
                                }}
                            </div>
                        }.into_view()
                    }
                }}
            </Suspense>
        </div>
    }
}

// StudentSelect component with improved performance
#[component]
pub fn StudentSelect(set_selected_student_id: WriteSignal<Option<i32>>) -> impl IntoView {
    // Extract information in the event student is anonymized
    let (settings, _) = use_settings();
    let anonymization_enabled = move || settings.get().student_protections;

    // Get mapping service for de-anonymization
    let (student_mapping_service, _) = use_student_mapping_service();

    // Fetch students from server
    let get_students_action = create_action(|_: &()| async move {
        match get_students().await {
            Ok(fetched_students) => fetched_students,
            Err(e) => {
                log::error!("Failed to fetch students: {}", e);
                Vec::new()
            }
        }
    });

    // Create enhanced student data with de-anonymization info
    let enhanced_students = create_memo(move |_| {
        let students_data = get_students_action
            .value()
            .get()
            .as_ref()
            .cloned()
            .unwrap_or_default();

        if anonymization_enabled() {
            let mapping_service = student_mapping_service.get();
            students_data
                .into_iter()
                .map(|student| {
                    let de_anon = DeAnonymizedStudent::from_student_with_mapping(
                        &student,
                        mapping_service.as_ref(),
                    );
                    (student, Some(de_anon))
                })
                .collect::<Vec<_>>()
        } else {
            students_data
                .into_iter()
                .map(|student| (student, None))
                .collect::<Vec<_>>()
        }
    });

    // Dispatch action only once on component mount
    create_effect(move |_| {
        get_students_action.dispatch(());
    });

    view! {
        <div class="mb-2 max-w-[20rem]">
            <label class="block text-sm font-medium mb-1">"Select Student:"</label>
            <select
                class="w-full p-2 border rounded-md"
                on:change=move |ev| {
                    let value = event_target_value(&ev).parse().ok();
                    set_selected_student_id.set(value);
                }
            >
                <option value="">"Select a student..."</option>
                <Suspense fallback=move || view! {
                    <option>"Loading students..."</option>
                }>
                    {move || {
                        enhanced_students().into_iter().map(|(student, de_anon_opt)| {
                            // Determine display values based on anonymization status
                            let display_text = if let Some(de_anon) = &de_anon_opt {
                                // Use de-anonymized display name and ID
                                format!("{} - {}", de_anon.display_name, de_anon.display_id)
                            } else {
                                // Use original student data
                                format!(
                                    "{} {} - {}",
                                    student.firstname.as_ref().unwrap_or(&"Unknown".to_string()),
                                    student.lastname.as_ref().unwrap_or(&"Unknown".to_string()),
                                    student.student_id
                                )
                            };

                            view! {
                                <option value={student.student_id.to_string()}>
                                    {display_text}
                                </option>
                            }
                        }).collect_view()
                    }}
                </Suspense>
            </select>
        </div>
    }
}
