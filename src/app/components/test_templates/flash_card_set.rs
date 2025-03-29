use crate::app::models::question::QuestionType;
use crate::app::models::score::CreateScoreRequest;
use crate::app::models::student::Student;
use crate::app::models::user::User;
use crate::app::server_functions::students::get_students;
use crate::app::server_functions::{questions::get_questions, scores::add_score};
use chrono::Local;
use leptos::*;
use leptos_router::*;
use log::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct QuestionResponse {
    answer: String,
    comment: String,
}

#[component]
pub fn FlashCardSet() -> impl IntoView {
    // Get test_id from URL parameters
    let params = use_params_map();
    let test_id = move || params.with(|params| params.get("test_id").cloned().unwrap_or_default());
    let user = use_context::<ReadSignal<Option<User>>>().expect("AuthProvider not Found");

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

    // Store responses for each question
    let (responses, set_responses) = create_signal(HashMap::<i32, QuestionResponse>::new());
    let (selected_student_id, set_selected_student_id) = create_signal(None::<i32>);

    // Flashcard state
    let (current_card_index, set_current_card_index) = create_signal(0);
    let (is_submitted, set_is_submitted) = create_signal(false);

    // Handler for answer updates
    let handle_answer_change = move |qnumber: i32, value: String| {
        set_responses.update(|r| {
            let response = r.entry(qnumber).or_insert(QuestionResponse {
                answer: String::new(),
                comment: String::new(),
            });
            response.answer = value;
        });
    };

    // Handler for comment updates
    let handle_comment_change = move |qnumber: i32, value: String| {
        set_responses.update(|r| {
            let response = r.entry(qnumber).or_insert(QuestionResponse {
                answer: String::new(),
                comment: String::new(),
            });
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

        // TODO: Get actual student_id and evaluator from user session/context
        let student_id = selected_student_id.get().unwrap_or(0); // Placeholder
        let evaluator = match user.get() {
            Some(user_data) => user_data.id.to_string(),
            None => 0.to_string(),
        }; // Placeholder
        let test_variant = 1; // Placeholder

        // Collect all scores and comments
        let mut test_scores = Vec::new();
        let mut comments = Vec::new();

        if let Some(questions) = questions.get() {
            // Sort questions by qnumber to ensure correct order
            let mut sorted_questions = questions.clone();
            sorted_questions.sort_by_key(|q| q.qnumber);

            for question in sorted_questions {
                if let Some(response) = current_responses.get(&question.qnumber) {
                    // Calculate score for this question
                    let score = if response.answer == question.correct_answer {
                        question.point_value
                    } else {
                        0
                    };
                    test_scores.push(score);
                    comments.push(response.comment.clone());
                } else {
                    // If no response, push 0 score and empty comment
                    test_scores.push(0);
                    comments.push(String::new());
                }
            }
        }

        // Create score request
        let score_request = CreateScoreRequest {
            student_id,
            test_id: current_test_id,
            test_scores,
            comments,
            test_variant,
            evaluator,
        };

        // Submit score to server
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

    view! {
        <div class="p-4 max-w-2xl mx-auto">
            {/* Student Selection */}
            <StudentSelect set_selected_student_id=set_selected_student_id />

            {/* Questions View */}
            <Suspense
                fallback=move || view! { <div>"Loading questions..."</div> }
            >
                {move || match questions.get() {
                    None => view! { <div>"Loading..."</div> }.into_view(),
                    Some(questions) if questions.is_empty() => {
                        view! { <div class="text-red-500">"No questions found for this test ID."</div> }.into_view()
                    },
                    Some(questions) => {
                        let total_questions = questions.len();
                        // Create a memo to get the current question
                        let current_question = create_memo(move |_| {
                            questions[current_card_index.get()].clone()
                        });

                        view! {
                            <div class="w-full bg-white shadow-md rounded-lg p-6">
                                {/* Question Card */}
                                <div class="mb-4">
                                    <h3 class="text-lg font-semibold mb-2">
                                        {"Question "}
                                        {current_card_index.get() + 1}
                                        {" of "}
                                        {total_questions}
                                        {" ("} {current_question.get().point_value} {" points)"}
                                    </h3>
                                    <p class="mt-2">{current_question.get().word_problem.clone()}</p>
                                </div>

                                {/* Answer Section */}
                                <div class="mb-4">
                                    {move || match current_question.get().question_type {
                                        QuestionType::MultipleChoice => view! {
                                            <div class="space-y-2">
                                                <For
                                                    each=move || current_question.get().options.clone()
                                                    key=|option| option.clone()
                                                    children=move |option| {
                                                        let option_value = option.clone();
                                                        let qnumber = current_question.get().qnumber;
                                                        view! {
                                                            <label class="block">
                                                                <input
                                                                    type="radio"
                                                                    name=format!("q_{}", qnumber)
                                                                    value=option_value.clone()
                                                                    checked={
                                                                        responses.with(|r|
                                                                            r.get(&qnumber)
                                                                                .map(|resp| resp.answer == option_value)
                                                                                .unwrap_or(false)
                                                                        )
                                                                    }
                                                                    on:change=move |ev| {
                                                                        let value = event_target_value(&ev);
                                                                        handle_answer_change(qnumber, value);
                                                                    }
                                                                />
                                                                {" "} {option_value}
                                                            </label>
                                                        }
                                                    }
                                                />
                                            </div>
                                        }.into_any(),
                                        QuestionType::TrueFalse => view! {
                                            <div class="space-y-2">
                                                <label class="block">
                                                    <input
                                                        type="radio"
                                                        name=format!("q_{}", current_question.get().qnumber)
                                                        value="true"
                                                        checked={
                                                            responses.with(|r|
                                                                r.get(&current_question.get().qnumber)
                                                                    .map(|resp| resp.answer == "true")
                                                                    .unwrap_or(false)
                                                            )
                                                        }
                                                        on:change=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            handle_answer_change(current_question.get().qnumber, value);
                                                        }
                                                    />
                                                    {" True"}
                                                </label>
                                                <label class="block">
                                                    <input
                                                        type="radio"
                                                        name=format!("q_{}", current_question.get().qnumber)
                                                        value="false"
                                                        checked={
                                                            responses.with(|r|
                                                                r.get(&current_question.get().qnumber)
                                                                    .map(|resp| resp.answer == "false")
                                                                    .unwrap_or(false)
                                                            )
                                                        }
                                                        on:change=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            handle_answer_change(current_question.get().qnumber, value);
                                                        }
                                                    />
                                                    {" False"}
                                                </label>
                                            </div>
                                        }.into_any(),
                                        _ => view! {
                                            <textarea
                                                class="w-full p-2 border rounded"
                                                value={
                                                    responses.with(|r|
                                                        r.get(&current_question.get().qnumber)
                                                            .map(|resp| resp.answer.clone())
                                                            .unwrap_or_default()
                                                    )
                                                }
                                                on:input=move |ev| {
                                                    let value = event_target_value(&ev);
                                                    handle_answer_change(current_question.get().qnumber, value);
                                                }
                                                placeholder="Enter your answer here..."
                                            ></textarea>
                                        }.into_any()
                                    }}
                                </div>

                                {/* Comments Section */}
                                <div class="mt-4">
                                    <label class="block text-sm font-medium mb-2">
                                        "Comments:"
                                    </label>
                                    <textarea
                                        class="w-full p-2 border rounded"
                                        prop:value={move ||
                                            responses.with(|r|
                                                r.get(&current_question.get().qnumber)
                                                    .map(|resp| resp.comment.clone())
                                                    .unwrap_or_default()
                                            )
                                        }
                                        on:input=move |ev| {
                                            let value = event_target_value(&ev);
                                            let qnumber = current_question.get().qnumber;
                                            handle_comment_change(qnumber, value);
                                        }
                                        placeholder="Add any comments or notes here..."
                                    ></textarea>
                                </div>

                                {/* Navigation Buttons */}
                                <div class="flex justify-between mt-6">
                                    <button
                                        class="px-4 py-2 bg-gray-200 rounded disabled:opacity-50"
                                        disabled={current_card_index.get() == 0}
                                        on:click=go_to_previous_card
                                    >
                                        "Previous"
                                    </button>

                                    {move || if current_card_index.get() == total_questions - 1 {
                                        view! {
                                            <Show when=move ||{is_submitted.get() == false}>
                                                <button
                                                    class="px-4 py-2 bg-green-500 text-white rounded"
                                                    on:click=move |_| {
                                                        handle_submit.dispatch(());
                                                        set_is_submitted.set(true);
                                                    }
                                                >
                                                    "Submit Assessment"
                                                </button>
                                            </Show>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <button
                                                class="px-4 py-2 bg-blue-500 text-white rounded"
                                                on:click=go_to_next_card
                                            >
                                                "Next"
                                            </button>
                                        }.into_view()
                                    }}
                                </div>

                                {/* Submission Status */}
                                {move || if is_submitted.get() {
                                    view! {
                                        <div class="mt-4 text-green-600">
                                            "Assessment submitted successfully!"
                                        </div>
                                        <button
                                            class="px-4 py-2 bg-gray-700 text-white rounded hover:bg-gray-600"
                                            on:click=move |_| {
                                                let navigate=leptos_router::use_navigate();
                                                navigate("/dashboard", Default::default());
                                            }
                                        >
                                            "Return home"
                                        </button>
                                    }.into_view()
                                } else {
                                    view! {}.into_view()
                                }}
                            </div>
                        }.into_view()
                    }.into_view()
                }}
            </Suspense>
        </div>
    }
}

// StudentSelect component remains the same
#[component]
pub fn StudentSelect(set_selected_student_id: WriteSignal<Option<i32>>) -> impl IntoView {
    let (students, set_students) = create_signal(Vec::new());
    let get_students_action = create_action(|_: &()| async move {
        match get_students().await {
            Ok(fetched_students) => fetched_students,
            Err(e) => {
                log::error!("Failed to fetch students: {}", e);
                Vec::new()
            }
        }
    });

    create_effect(move |_| {
        get_students_action.dispatch(());
    });

    create_effect(move |_| {
        if let Some(result) = get_students_action.value().get() {
            set_students.set(result);
        }
    });

    view! {
        <div class="mb-4">
            <label class="block text-sm font-medium mb-2">"Select Student:"</label>
            <select
                class="w-full p-2 border rounded-md"
                on:change=move |ev| {
                    let value = event_target_value(&ev).parse().ok();
                    set_selected_student_id.set(value);
                }
            >
                <option value="">"Select a student..."</option>
                {move || students.get().into_iter().map(|student| {
                    view! {
                        <option value={student.student_id.to_string()}>
                            {format!("{} {} - {}", student.firstname, student.lastname, student.student_id)}
                        </option>
                    }
                }).collect_view()}
            </select>
        </div>
    }
}
