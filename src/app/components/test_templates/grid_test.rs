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
}

#[component]
pub fn GridTest() -> impl IntoView {
    // Get test_id from URL parameters
    let params = use_params_map();
    let test_id = move || params.with(|params| params.get("test_id").cloned().unwrap_or_default());
    let user = use_context::<ReadSignal<Option<SessionUser>>>().expect("AuthProvider not Found");
    let user_resource = create_resource(
        move || user.get().map(|u| u.id),
        move |id| async move {
            match id {
                Some(user_id) => match get_user(user_id).await {
                    Ok(user) => Some(user),
                    Err(e) => {
                        log::error!("Failed to fetch user from database: {}", e);
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

    // Create resource for questions
    let questions = create_resource(test_id, move |tid| async move {
        if tid.is_empty() {
            log::warn!("No test ID provided in URL");
            return Vec::new();
        }
        match get_questions(tid).await {
            Ok(questions) => {
                // Validate all questions are true/false type
                for q in &questions {
                    if q.question_type != QuestionType::TrueFalse {
                        log::error!("GridTest requires all questions to be TrueFalse type. Found question {} with type {:?}", 
                            q.qnumber, q.question_type);
                    }
                }
                questions
            }
            Err(e) => {
                log::error!("Failed to fetch questions: {}", e);
                Vec::new()
            }
        }
    });

    // Store responses for each question
    let (responses, set_responses) = create_signal(HashMap::<i32, QuestionResponse>::new());
    let (selected_student_id, set_selected_student_id) = create_signal(None::<i32>);

    // Track if test is submitted
    let (is_submitted, set_is_submitted) = create_signal(false);

    // Currently selected question for commenting
    let (selected_question, set_selected_question) = create_signal(None::<i32>);

    // Get evaluator ID
    let evaluator_id = create_memo(move |_| match user.get() {
        Some(user_data) => user_data.id.to_string(),
        None => "0".to_string(),
    });

    // Handler for toggling answer
    let toggle_answer = move |qnumber: i32| {
        set_responses.update(|r| {
            let response = r.entry(qnumber).or_insert(QuestionResponse {
                answer: "true".to_string(), // Default to "true" since all cells start as "correct"
                comment: String::new(),
            });
            // Toggle between true and false
            response.answer = if response.answer == "true" {
                "false".to_string()
            } else {
                "true".to_string()
            };
        });

        // Set this question as selected for comments
        set_selected_question.set(Some(qnumber));
    };

    // Handler for updating comment
    let handle_comment_change = move |value: String| {
        if let Some(qnumber) = selected_question.get() {
            set_responses.update(|r| {
                let response = r.entry(qnumber).or_insert(QuestionResponse {
                    answer: "true".to_string(), // Default to "true"
                    comment: String::new(),
                });
                response.comment = value;
            });
        }
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
            // Sort questions by qnumber to ensure correct order
            let mut sorted_questions = questions.clone();
            sorted_questions.sort_by_key(|q| q.qnumber);

            for question in sorted_questions {
                let response = current_responses
                    .get(&question.qnumber)
                    .cloned()
                    .unwrap_or_else(|| {
                        // Default response is "true" (correct) if not explicitly marked wrong
                        QuestionResponse {
                            answer: "true".to_string(),
                            comment: String::new(),
                        }
                    });

                // Calculate score - "true" means correct (full points), "false" means incorrect (0 points)
                let score = if response.answer == "true" {
                    question.point_value
                } else {
                    0
                };

                test_scores.push(score);
                comments.push(response.comment);
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

    // Calculate square grid dimensions
    let grid_dimensions = create_memo(move |_| {
        if let Some(questions_list) = questions.get() {
            let count = questions_list.len();
            if count == 0 {
                return (0, 0);
            }

            // Calculate dimensions for a perfect square grid
            let sqrt = (count as f64).sqrt().ceil() as usize;

            // Make sure grid is square - use same value for rows and columns
            (sqrt, sqrt)
        } else {
            (0, 0)
        }
    });

    // Calculate cell size based on question count
    let cell_size_class = create_memo(move |_| {
        if let Some(questions_list) = questions.get() {
            let count = questions_list.len();
            // Determine appropriate cell size based on question count
            if count > 100 {
                "text-xs" // Extremely small text for very many questions
            } else if count > 64 {
                "text-sm" // Very small text for many questions
            } else if count > 36 {
                "text-base" // Small text for lots of questions
            } else if count > 16 {
                "text-lg" // Normal text for medium count
            } else if count > 9 {
                "text-xl" // Larger text for few questions
            } else {
                "text-2xl" // Very large text for very few questions
            }
        } else {
            "text-base" // Default size
        }
    });

    // Create a memo for the currently selected question's comment
    let selected_comment = create_memo(move |_| {
        if let Some(qnumber) = selected_question.get() {
            responses.with(|r| {
                r.get(&qnumber)
                    .map(|resp| resp.comment.clone())
                    .unwrap_or_default()
            })
        } else {
            String::new()
        }
    });

    view! {
        <div class="flex flex-col h-screen bg-gray-50 overflow-hidden">
            {/* Header with Student Selection and Evaluator - Fixed at top */}
            <div class="p-4 bg-white shadow-sm">
                <div class="flex flex-wrap items-center justify-between mb-2 max-w-4xl mx-auto w-full">
                    <div class="w-full md:w-1/2 mb-2 md:mb-0">
                        <StudentSelect set_selected_student_id=set_selected_student_id />
                    </div>
                    <div class="text-sm text-gray-600 font-medium">
                        {"Evaluator: "}
                        {move || match user_resource.get() {
                            Some(Some(user)) => format!("{} {}", user.first_name.unwrap_or("None".to_string()), user.last_name.unwrap_or("None".to_string())),
                            Some(None) => evaluator_id(),
                            None => "Loading...".to_string(),
                        }}
                    </div>
                </div>

                {/* Test Title */}
                <div class="text-center">
                    <h2 class="text-xl font-medium text-gray-700 break-words">
                        {move || match &test_details.get() {
                            Some(Some(test)) => test.name.clone(),
                            _ => test_id()
                        }}
                    </h2>
                </div>
            </div>

            {/* Main container with grid and comments - scrollable with proper height constraints */}
            <div class="flex-1 flex flex-col overflow-auto p-4">
                <Suspense
                    fallback=move || view! { <div class="flex justify-center items-center h-full">
                        <div class="animate-pulse bg-white rounded w-full h-full flex items-center justify-center">
                            <p class="text-gray-400">"Loading..."</p>
                        </div>
                    </div> }
                >
                    {move || match (questions.get(), test_details.get()) {
                        (None, _) => view! { <div class="text-center py-4">"Loading..."</div> }.into_view(),
                        (Some(questions), _) if questions.is_empty() => {
                            view! { <div class="text-center py-4 text-red-500">"No questions found for this test ID."</div> }.into_view()
                        },
                        (Some(questions), _) => {
                            // Check if all questions are TrueFalse type
                            let has_invalid_questions = questions.iter().any(|q| q.question_type != QuestionType::TrueFalse);

                            if has_invalid_questions {
                                view! {
                                    <div class="text-center py-4 text-red-500">
                                        "Error: This test contains questions that are not True/False type. GridTest requires all questions to be True/False."
                                    </div>
                                }.into_view()
                            } else {
                                let (rows, cols) = grid_dimensions();
                                let sorted_questions = create_memo(move |_| {
                                    let mut sorted = questions.clone();
                                    sorted.sort_by_key(|q| q.qnumber);
                                    sorted
                                });

                                let current_cell_size = cell_size_class();

                                view! {
                                    <div class="flex flex-col h-full gap-4">
                                        {/* Grid container with fixed max height instead of aspect-square */}
                                        <div class="bg-white rounded shadow-md p-2 max-h-[55vh] overflow-auto">
                                            <div
                                                class="grid w-full"
                                                style=move || format!(
                                                    "grid-template-columns: repeat({}, minmax(0, 1fr)); grid-template-rows: repeat({}, minmax(0, 1fr)); gap: 1px; background-color: #e5e7eb;",
                                                    cols, rows
                                                )
                                            >
                                                {move || {
                                                    sorted_questions().into_iter().map(|question| {
                                                        let qnumber = question.qnumber;
                                                        let display_text = question.word_problem.clone();

                                                        let is_correct = create_memo(move |_| {
                                                            responses.with(|r| {
                                                                r.get(&qnumber)
                                                                 .map(|resp| resp.answer == "true")
                                                                 .unwrap_or(true) // Default to true if not explicitly marked
                                                            })
                                                        });

                                                        let is_selected = create_memo(move |_| {
                                                            selected_question.get() == Some(qnumber)
                                                        });

                                                        view! {
                                                            <div
                                                                class="flex items-center justify-center cursor-pointer transition-all relative p-2"
                                                                class:bg-green-100=move || is_correct()
                                                                class:bg-red-100=move || !is_correct()
                                                                class:ring-2=move || is_selected()
                                                                class:ring-blue-500=move || is_selected()
                                                                on:click=move |_| toggle_answer(qnumber)
                                                            >
                                                                <span class=format!("select-none font-medium {} px-1 py-1 text-center text-7xl", current_cell_size)>{display_text}</span>
                                                                {move || if !is_correct() {
                                                                    view! {
                                                                        <span class="absolute top-0 right-0 text-xs bg-red-500 text-white rounded-full w-3 h-3 flex items-center justify-center">
                                                                            "×"
                                                                        </span>
                                                                    }.into_view()
                                                                } else {
                                                                    view! { <span></span> }.into_view()
                                                                }}
                                                            </div>
                                                        }
                                                    }).collect_view()
                                                }}
                                            </div>
                                        </div>

                                        {/* Comments section - will now always be visible */}
                                        <div class="bg-white rounded-lg shadow-sm p-4">
                                            {move || match selected_question.get() {
                                                Some(qnumber) => {
                                                    let question_text = sorted_questions().iter()
                                                        .find(|q| q.qnumber == qnumber)
                                                        .map(|q| q.word_problem.clone())
                                                        .unwrap_or_default();

                                                    view! {
                                                        <div>
                                                            <label class="block text-sm font-medium text-gray-700 mb-2">
                                                                {format!("Comment for '{}':", question_text)}
                                                            </label>
                                                            <textarea
                                                                class="w-full p-3 border border-gray-200 rounded-lg focus:ring-blue-500 focus:border-blue-500"
                                                                prop:value=move || selected_comment()
                                                                on:input=move |ev| {
                                                                    let value = event_target_value(&ev);
                                                                    handle_comment_change(value);
                                                                }
                                                                placeholder="Add any comments or notes here..."
                                                                rows="2"
                                                            ></textarea>
                                                        </div>
                                                    }.into_view()
                                                },
                                                None => view! {
                                                    <div class="text-sm text-gray-500 italic py-3 text-center">
                                                        "Click any grid cell to select it and add comments"
                                                    </div>
                                                }.into_view()
                                            }}
                                        </div>

                                        {/* Submit Button section - always visible */}
                                        <div class="flex flex-wrap items-center justify-center gap-4 mb-2">
                                            {move || if !is_submitted.get() {
                                                view! {
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
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <div class="text-center">
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
                                                }.into_view()
                                            }}
                                        </div>
                                    </div>
                                }.into_view()
                            }
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}

// StudentSelect component with improved styling from FlashCardSet
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

    // Dispatch action only once on component mount
    create_effect(move |_| {
        get_students_action.dispatch(());
    });

    // Update students when data is received
    create_effect(move |_| {
        if let Some(result) = get_students_action.value().get() {
            set_students.set(result);
        }
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
                {move || students.get().into_iter().map(|student| {
                    view! {
                        <option value={student.student_id.to_string()}>
                            {format!("{} {} - {}", student.firstname.unwrap(), student.lastname.unwrap(), student.student_id)}
                        </option>
                    }
                }).collect_view()}
            </select>
        </div>
    }
}
