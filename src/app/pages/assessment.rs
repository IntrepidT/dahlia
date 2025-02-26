use crate::app::models::question::{Question, QuestionType};
use crate::app::models::score::{CreateScoreRequest, Score};
use crate::app::models::student::Student;
use crate::app::server_functions::students::get_students;
use crate::app::server_functions::{questions::get_questions, scores::add_score};
use chrono::Local;
use leptos::*;
use leptos_router::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct QuestionResponse {
    answer: String,
    comment: String,
}

#[component]
pub fn Assessment() -> impl IntoView {
    // Get test_id from URL parameters
    let params = use_params_map();
    let test_id = move || params.with(|params| params.get("test_id").cloned().unwrap_or_default());

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

    // Submit handler
    let handle_submit = create_action(move |_: &()| async move {
        let current_responses = responses.get();
        let current_test_id = test_id();

        // TODO: Get actual student_id and evaluator from user session/context
        let student_id = selected_student_id.get().unwrap_or(0); // Placeholder
        let evaluator = "temp_evaluator".to_string(); // Placeholder
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

    // Submission status message
    let submit_status = create_memo(move |_| match handle_submit.value().get() {
        Some(Ok(_)) => Some("Assessment submitted successfully!"),
        Some(Err(_)) => Some("Error Submitting Assessment"),
        None => None,
    });

    view! {
        <div class="p-4">
            <h2 class="text-2xl font-bold mb-4">
                "Assessment "
                {test_id}
            </h2>
            <StudentSelect set_selected_student_id=set_selected_student_id />
            
            // Show loading state
            <Suspense
                fallback=move || view! { <div>"Loading questions..."</div> }
            >
                {move || match questions.get() {
                    None => view! { <div>"Loading..."</div> }.into_view(),
                    Some(questions) if questions.is_empty() => {
                        view! { <div class="text-red-500">"No questions found for this test ID."</div> }.into_view()
                    },
                    Some(questions) => view! {
                        <div class="space-y-8">
                            <For
                                each=move || questions.clone()
                                key=|question| question.qnumber
                                children=move |question| {
                                    let qnumber = question.qnumber;

                                    view! {
                                        <div class="border p-4 rounded-lg shadow">
                                            <div class="mb-4">
                                                <h3 class="text-lg font-semibold">
                                                    {"Question "} {question.qnumber} {" ("} {question.point_value} {" points)"}
                                                </h3>
                                                <p class="mt-2">{question.word_problem.clone()}</p>
                                            </div>

                                            // Answer section
                                            <div class="mb-4">
                                                {match question.question_type {
                                                    QuestionType::MultipleChoice => view! {
                                                        <div class="space-y-2">
                                                            <For
                                                                each=move || question.options.clone()
                                                                key=|option| option.clone()
                                                                children=move |option| {
                                                                    let option_value = option.clone();
                                                                    view! {
                                                                        <label class="block">
                                                                            <input
                                                                                type="radio"
                                                                                name=format!("q_{}", qnumber)
                                                                                value=option_value.clone()
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
                                                                    name=format!("q_{}", qnumber)
                                                                    value="True"
                                                                    on:change=move |ev| {
                                                                        let value = event_target_value(&ev);
                                                                        handle_answer_change(qnumber, value);
                                                                    }
                                                                />
                                                                {" True"}
                                                            </label>
                                                            <label class="block">
                                                                <input
                                                                    type="radio"
                                                                    name=format!("q_{}", qnumber)
                                                                    value="False"
                                                                    on:change=move |ev| {
                                                                        let value = event_target_value(&ev);
                                                                        handle_answer_change(qnumber, value);
                                                                    }
                                                                />
                                                                {" False"}
                                                            </label>
                                                        </div>
                                                    }.into_any(),
                                                    _ => view! {
                                                        <textarea
                                                            class="w-full p-2 border rounded"
                                                            on:input=move |ev| {
                                                                let value = event_target_value(&ev);
                                                                handle_answer_change(qnumber, value);
                                                            }
                                                            placeholder="Enter your answer here..."
                                                        ></textarea>
                                                    }.into_any()
                                                }}
                                            </div>

                                            // Comments section
                                            <div class="mt-4">
                                                <label class="block text-sm font-medium mb-2">
                                                    "Comments:"
                                                </label>
                                                <textarea
                                                    class="w-full p-2 border rounded"
                                                    on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        handle_comment_change(qnumber, value);
                                                    }
                                                    placeholder="Add any comments or notes here..."
                                                ></textarea>
                                            </div>
                                        </div>
                                    }
                                }
                            />

                            <div class="mt-8">
                                // Show submission status
                                {move || submit_status.get().map(|status| {
                                    view! {
                                        <div class="mb-4" class:text-green-500=status.starts_with("Assessment submitted") class:text-red-500=status.starts_with("Error")>
                                            {status}
                                        </div>
                                    }
                                })}

                                <button
                                    class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
                                    on:click=move |_| handle_submit.dispatch(())
                                    disabled=move || handle_submit.pending().get()
                                >
                                    {move || if handle_submit.pending().get() {
                                        "Submitting..."
                                    } else {
                                        "Submit Assessment"
                                    }}
                                </button>
                            </div>
                        </div>
                    }.into_view()
                }}
            </Suspense>
        </div>
    }
}

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
