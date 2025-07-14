use crate::app::components::enhanced_login_form::{
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
}

#[component]
pub fn GridTest() -> impl IntoView {
    // Get test_id from URL parameters
    let params = use_params_map();
    let test_id = move || params.with(|params| params.get("test_id").cloned().unwrap_or_default());
    let user = use_context::<ReadSignal<Option<SessionUser>>>().expect("AuthProvider not Found");
    let user_resource = create_local_resource(
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
        <div class="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50">
            {/* Header with Student Selection and Evaluator - Enhanced styling */}
            <div class="bg-white/80 backdrop-blur-sm border-b border-gray-200/50 shadow-lg sticky top-0 z-10">
                <div class="max-w-7xl mx-auto px-6 py-5">
                    <div class="flex flex-col lg:flex-row items-center justify-between gap-4">
                        <div class="flex-1 w-full lg:w-auto">
                            <StudentSelect set_selected_student_id=set_selected_student_id />
                        </div>

                        <div class="flex items-center gap-3 bg-gradient-to-r from-indigo-50 to-purple-50 px-4 py-2 rounded-full border border-indigo-200/50">
                            <span class="text-sm font-medium text-gray-700">
                                {"Evaluator: "}
                                <span class="text-indigo-600 font-semibold">
                                    {move || match user_resource.get() {
                                        Some(Some(user)) => format!("{} {}", user.first_name.unwrap_or("None".to_string()), user.last_name.unwrap_or("None".to_string())),
                                        Some(None) => evaluator_id(),
                                        None => "Loading...".to_string(),
                                    }}
                                </span>
                            </span>
                        </div>
                    </div>

                    {/* Test Title - Enhanced */}
                    <div class="mt-4 text-center">
                        <h1 class="text-3xl font-bold bg-gradient-to-r from-indigo-600 to-purple-600 bg-clip-text text-transparent">
                            {move || match &test_details.get() {
                                Some(Some(test)) => test.name.clone(),
                                _ => test_id()
                            }}
                        </h1>
                        <div class="mt-2 h-1 w-20 bg-gradient-to-r from-indigo-400 to-purple-400 rounded-full mx-auto"></div>
                    </div>
                </div>
            </div>

            {/* Main container - Enhanced layout */}
            <div class="max-w-7xl mx-auto px-6 py-8">
                <Suspense
                    fallback=move || view! {
                        <div class="flex justify-center items-center min-h-[60vh]">
                            <div class="bg-white/80 backdrop-blur-sm rounded-2xl shadow-xl p-12 border border-gray-200/50">
                                <div class="flex flex-col items-center gap-4">
                                    <div class="w-12 h-12 border-4 border-indigo-200 border-t-indigo-600 rounded-full animate-spin"></div>
                                    <p class="text-gray-600 font-medium">"Loading assessment..."</p>
                                </div>
                            </div>
                        </div>
                    }
                >
                    {move || match (questions.get(), test_details.get()) {
                        (None, _) => view! {
                            <div class="flex justify-center items-center min-h-[60vh]">
                                <div class="bg-white/80 backdrop-blur-sm rounded-2xl shadow-xl p-12 border border-gray-200/50">
                                    <div class="flex flex-col items-center gap-4">
                                        <div class="w-12 h-12 border-4 border-indigo-200 border-t-indigo-600 rounded-full animate-spin"></div>
                                        <p class="text-gray-600 font-medium">"Loading assessment..."</p>
                                    </div>
                                </div>
                            </div>
                        }.into_view(),
                        (Some(questions), _) if questions.is_empty() => {
                            view! {
                                <div class="flex justify-center items-center min-h-[60vh]">
                                    <div class="bg-white/80 backdrop-blur-sm rounded-2xl shadow-xl p-12 border border-red-200/50">
                                        <div class="flex flex-col items-center gap-4">
                                            <div class="w-16 h-16 bg-red-100 rounded-full flex items-center justify-center">
                                                <span class="text-red-600 text-2xl">"!"</span>
                                            </div>
                                            <p class="text-red-600 font-medium text-center">"No questions found for this test ID."</p>
                                        </div>
                                    </div>
                                </div>
                            }.into_view()
                        },
                        (Some(questions), _) => {
                            // Check if all questions are TrueFalse type
                            let has_invalid_questions = questions.iter().any(|q| q.question_type != QuestionType::TrueFalse);

                            if has_invalid_questions {
                                view! {
                                    <div class="flex justify-center items-center min-h-[60vh]">
                                        <div class="bg-white/80 backdrop-blur-sm rounded-2xl shadow-xl p-12 border border-red-200/50">
                                            <div class="flex flex-col items-center gap-4">
                                                <div class="w-16 h-16 bg-red-100 rounded-full flex items-center justify-center">
                                                    <span class="text-red-600 text-2xl">"⚠"</span>
                                                </div>
                                                <p class="text-red-600 font-medium text-center max-w-md">
                                                    "Error: This test contains questions that are not True/False type. GridTest requires all questions to be True/False."
                                                </p>
                                            </div>
                                        </div>
                                    </div>
                                }.into_view()
                            } else {
                                let (rows, cols) = grid_dimensions();
                                let questions_clone = questions.clone();
                                let questions_count = questions.len();
                                let sorted_questions = create_memo(move |_| {
                                    let mut sorted = questions_clone.clone();
                                    sorted.sort_by_key(|q| q.qnumber);
                                    sorted
                                });

                                let current_cell_size = cell_size_class();

                                view! {
                                    <div class="grid grid-cols-1 xl:grid-cols-3 gap-8 items-start">
                                        {/* Grid container - Enhanced design */}
                                        <div class="xl:col-span-2 h-full">
                                            <div class="bg-white/80 backdrop-blur-sm rounded-2xl shadow-xl border border-gray-200/50 p-6">
                                                <div class="flex items-center gap-3 mb-6">
                                                    <h2 class="text-xl font-semibold text-gray-800">"Assessment Grid"</h2>
                                                    <div class="flex-1 h-px bg-gradient-to-r from-gray-200 to-transparent"></div>
                                                    <span class="text-sm text-gray-500 bg-gray-100 px-3 py-1 rounded-full">
                                                        {format!("{} questions", questions_count)}
                                                    </span>
                                                </div>

                                                <div class="relative">
                                                    <div
                                                        class="grid w-full"
                                                        style=move || format!(
                                                            "grid-template-columns: repeat({}, minmax(0, 1fr)); grid-template-rows: repeat({}, minmax(0, 1fr)); gap: 3px; background-color: #f8fafc;",
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

                                                                let has_comment = create_memo(move |_| {
                                                                    responses.with(|r| {
                                                                        r.get(&qnumber)
                                                                         .map(|resp| !resp.comment.is_empty())
                                                                         .unwrap_or(false)
                                                                    })
                                                                });

                                                                view! {
                                                                    <div
                                                                        class="relative flex items-center justify-center cursor-pointer transition-all duration-200 rounded-lg min-h-[50px] group hover:shadow-md border-2"
                                                                        class:bg-gradient-to-br=move || is_correct()
                                                                        class:from-emerald-50=move || is_correct()
                                                                        class:to-green-100=move || is_correct()
                                                                        class:ring-3=move || is_selected()
                                                                        class:ring-indigo-400=move || is_selected()
                                                                        class:shadow-lg=move || is_selected()
                                                                        class:scale-105=move || is_selected()
                                                                        class:border-green-300=move || is_correct()
                                                                        class:border-red-400=move || !is_correct()
                                                                        class:bg-red-300=move || !is_correct()
                                                                        on:click=move |_| toggle_answer(qnumber)
                                                                    >
                                                                        <span class=format!("select-none font-bold text-gray-700 px-2 py-2 text-center {} group-hover:scale-110 transition-transform", current_cell_size)
                                                                    //class:text-white=move || !is_correct()
                                                                    class:text-gray-700=move || is_correct()
                                                                    >
                                                                            {display_text}
                                                                        </span>

                                                                        {/* Comment indicator */}
                                                                        {move || if has_comment() {
                                                                            view! {
                                                                                <div class="absolute -bottom-1 -right-1 w-4 h-4 bg-gradient-to-br from-blue-500 to-indigo-600 text-white rounded-full flex items-center justify-center shadow-sm">
                                                                                    <span class="text-xs">"💬"</span>
                                                                                </div>
                                                                            }.into_view()
                                                                        } else {
                                                                            view! { <div></div> }.into_view()
                                                                        }}
                                                                    </div>
                                                                }
                                                            }).collect_view()
                                                        }}
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        {/* Comments and Submit section - Enhanced sidebar */}
                                        <div class="space-y-6">
                                            {/* Comments section */}
                                            <div class="bg-white/80 backdrop-blur-sm rounded-2xl shadow-xl border border-gray-200/50 p-6">
                                                <div class="flex items-center gap-3 mb-4">
                                                    <div class="w-3 h-3 bg-gradient-to-r from-purple-400 to-pink-400 rounded-full"></div>
                                                    <h3 class="text-lg font-semibold text-gray-800">"Comments"</h3>
                                                </div>

                                                {move || match selected_question.get() {
                                                    Some(qnumber) => {
                                                        let question_text = sorted_questions().iter()
                                                            .find(|q| q.qnumber == qnumber)
                                                            .map(|q| q.word_problem.clone())
                                                            .unwrap_or_default();

                                                        view! {
                                                            <div class="space-y-4">
                                                                <div class="bg-gradient-to-r from-indigo-50 to-purple-50 p-4 rounded-xl border border-indigo-200/50">
                                                                    <div class="flex items-center gap-2 mb-2">
                                                                        <span class="text-lg font-bold text-indigo-600">
                                                                            {question_text.clone()}
                                                                        </span>
                                                                    </div>
                                                                    <p class="text-sm text-gray-600">"Selected for commenting"</p>
                                                                </div>

                                                                <div>
                                                                    <label class="block text-sm font-medium text-gray-700 mb-2">
                                                                        "Add your comments:"
                                                                    </label>
                                                                    <textarea
                                                                        class="w-full p-4 border border-gray-200 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all duration-200 resize-none bg-white/80 backdrop-blur-sm"
                                                                        prop:value=move || selected_comment()
                                                                        on:input=move |ev| {
                                                                            let value = event_target_value(&ev);
                                                                            handle_comment_change(value);
                                                                        }
                                                                        placeholder="Add any comments, notes, or feedback here..."
                                                                        rows="4"
                                                                    ></textarea>
                                                                </div>
                                                            </div>
                                                        }.into_view()
                                                    },
                                                    None => view! {
                                                        <div class="text-center py-8">
                                                            <div class="w-16 h-16 bg-gradient-to-br from-gray-100 to-gray-200 rounded-full flex items-center justify-center mx-auto mb-4">
                                                                <span class="text-gray-400 text-2xl">"💭"</span>
                                                            </div>
                                                            <p class="text-gray-500 text-sm">
                                                                "Click any grid cell to select it and add comments"
                                                            </p>
                                                        </div>
                                                    }.into_view()
                                                }}
                                            </div>

                                            {/* Submit section */}
                                            <div class="bg-white/80 backdrop-blur-sm rounded-2xl shadow-xl border border-gray-200/50 p-6">
                                                <div class="flex items-center gap-3 mb-4">
                                                    <h3 class="text-lg font-semibold text-gray-800">"Submit Assessment"</h3>
                                                </div>

                                                {move || if !is_submitted.get() {
                                                    view! {
                                                        <div class="space-y-4">

                                                            <button
                                                                class="w-full flex items-center justify-center gap-3 px-6 py-4 bg-gradient-to-r from-indigo-600 via-purple-600 to-indigo-700 text-white rounded-xl shadow-lg hover:shadow-xl transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed disabled:shadow-none transform hover:scale-105 active:scale-95"
                                                                on:click=move |_| {
                                                                    handle_submit.dispatch(());
                                                                    set_is_submitted.set(true);
                                                                }
                                                                disabled=move || selected_student_id.get().is_none()
                                                            >
                                                                <span class="font-semibold">"Submit Assessment"</span>
                                                                <span class="text-lg">"✓"</span>
                                                            </button>

                                                            {move || if selected_student_id.get().is_none() {
                                                                view! {
                                                                    <p class="text-xs text-amber-600 text-center bg-amber-50 px-3 py-2 rounded-lg border border-amber-200">
                                                                        "Please select a student before submitting"
                                                                    </p>
                                                                }.into_view()
                                                            } else {
                                                                view! { <div></div> }.into_view()
                                                            }}
                                                        </div>
                                                    }.into_view()
                                                } else {
                                                    view! {
                                                        <div class="text-center space-y-4">
                                                            <div class="w-20 h-20 bg-gradient-to-br from-green-400 to-emerald-500 rounded-full flex items-center justify-center mx-auto shadow-lg">
                                                                <span class="text-white text-3xl">"✓"</span>
                                                            </div>

                                                            <div class="space-y-2">
                                                                <h4 class="text-lg font-semibold text-gray-800">
                                                                    "Assessment Submitted!"
                                                                </h4>
                                                                <p class="text-sm text-gray-600">
                                                                    "Your assessment has been successfully submitted and saved."
                                                                </p>
                                                            </div>

                                                            <button
                                                                class="w-full flex items-center justify-center gap-3 px-6 py-4 bg-gradient-to-r from-gray-700 to-gray-800 text-white rounded-xl shadow-lg hover:shadow-xl transition-all duration-200 transform hover:scale-105 active:scale-95"
                                                                on:click=move |_| {
                                                                    let navigate=leptos_router::use_navigate();
                                                                    navigate("/dashboard", Default::default());
                                                                }
                                                            >
                                                                <span class="text-lg">"🏠"</span>
                                                                <span class="font-semibold">"Return to Dashboard"</span>
                                                            </button>
                                                        </div>
                                                    }.into_view()
                                                }}
                                            </div>
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
