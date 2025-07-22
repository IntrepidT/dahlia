use crate::app::components::auth::enhanced_login_form::{
    use_student_mapping_service, DeAnonymizedStudent,
};
use crate::app::components::test_components::font_controls::{
    use_font_settings, FontControls, FontSettings,
};
use crate::app::components::test_components::test_instructions::TestInstructions;
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
    let (font_settings, set_font_settings) = use_font_settings();

    // Add state for collapsible sections
    let (shortcuts_expanded, set_shortcuts_expanded) = create_signal(false);
    let (instructions_expanded, set_instructions_expanded) = create_signal(false);

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
            Ok(mut questions) => {
                // Sort questions by qnumber to ensure consistent ordering
                questions.sort_by_key(|q| q.qnumber);
                questions
            }
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
    let go_to_next_card = move |_ev| {
        set_current_card_index.update(|index| {
            if let Some(questions_vec) = questions.get() {
                *index = (*index + 1).min(questions_vec.len() - 1);
            }
        });
    };

    let go_to_previous_card = move |_ev| {
        set_current_card_index.update(|index| {
            *index = index.saturating_sub(1);
        });
    };

    // Jump to specific question (for keyboard shortcuts)
    let jump_to_question = move |question_number: usize| {
        questions.with(|questions_opt| {
            if let Some(questions_vec) = questions_opt {
                if question_number > 0 && question_number <= questions_vec.len() {
                    set_current_card_index.set(question_number - 1);
                }
            }
        });
    };

    // Focus comments box
    let focus_comments = move || {
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(textarea) = document
                        .query_selector("textarea[placeholder*='notes']")
                        .ok()
                        .flatten()
                    {
                        let _ = textarea.unchecked_ref::<web_sys::HtmlElement>().focus();
                    }
                }
            }
        }
    };

    // De-focus/blur current active element
    let blur_active_element = move || {
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    // Get the currently active element and blur it directly
                    if let Some(active_element) = document.active_element() {
                        log::info!("Blurring element: {}", active_element.tag_name());

                        // Try multiple approaches to ensure blur works
                        if let Some(html_element) = active_element.dyn_ref::<web_sys::HtmlElement>()
                        {
                            let _ = html_element.blur();
                        }

                        // Additional approaches for stubborn elements
                        if let Some(input_element) =
                            active_element.dyn_ref::<web_sys::HtmlInputElement>()
                        {
                            let _ = input_element.blur();
                        }

                        if let Some(textarea_element) =
                            active_element.dyn_ref::<web_sys::HtmlTextAreaElement>()
                        {
                            let _ = textarea_element.blur();
                        }
                    }

                    // Force focus to main container to ensure nothing has focus
                    if let Some(main_container) =
                        document.query_selector(".min-h-screen").ok().flatten()
                    {
                        if let Some(html_element) = main_container.dyn_ref::<web_sys::HtmlElement>()
                        {
                            let _ = html_element.focus();

                            // Immediately blur it so nothing has focus
                            let _ = html_element.blur();
                        }
                    }
                }
            }
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

    // Keyboard event handler
    #[cfg(feature = "hydrate")]
    {
        use leptos::ev::KeyboardEvent;

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
                        if let Some(questions_vec) = questions.get() {
                            *index = (*index + 1).min(questions_vec.len() - 1);
                        }
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
                            // Check if we're on a multiple choice question
                            if let Some(questions_vec) = questions.get() {
                                let current_question = &questions_vec[current_card_index.get()];
                                match current_question.question_type {
                                    QuestionType::MultipleChoice => {
                                        if num <= current_question.options.len() {
                                            let option = current_question.options[num - 1].clone();
                                            handle_answer_change(current_question.qnumber, option);
                                        }
                                    }
                                    QuestionType::WeightedMultipleChoice => {
                                        let weighted_options =
                                            current_question.get_weighted_options();
                                        if num <= weighted_options.len() {
                                            let option = &weighted_options[num - 1];
                                            if option.is_selectable {
                                                let current_selected = responses.with(|r| {
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

                                                handle_weighted_selection(
                                                    current_question.qnumber,
                                                    new_selected,
                                                );
                                            }
                                        }
                                    }
                                    QuestionType::TrueFalse => {
                                        if num == 1 {
                                            handle_answer_change(
                                                current_question.qnumber,
                                                "true".to_string(),
                                            );
                                        } else if num == 2 {
                                            handle_answer_change(
                                                current_question.qnumber,
                                                "false".to_string(),
                                            );
                                        }
                                    }
                                    _ => {
                                        // For other question types, jump to question
                                        jump_to_question(num);
                                    }
                                }
                            }
                        }
                    }
                }
                "c" | "C" => {
                    if !ev.ctrl_key() && !ev.alt_key() && !ev.meta_key() {
                        ev.prevent_default();
                        focus_comments();
                    }
                }
                "Enter" => {
                    if ev.ctrl_key() || ev.meta_key() {
                        ev.prevent_default();
                        if let Some(questions_vec) = questions.get() {
                            if current_card_index.get() == questions_vec.len() - 1
                                && !is_submitted.get()
                            {
                                // Submit on last question
                                if selected_student_id.get().is_some() {
                                    handle_submit.dispatch(());
                                    set_is_submitted.set(true);
                                }
                            } else {
                                set_current_card_index.update(|index| {
                                    *index = (*index + 1).min(questions_vec.len() - 1);
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        };

        create_effect(move |_| {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();

            let closure = wasm_bindgen::closure::Closure::wrap(
                Box::new(handle_keydown) as Box<dyn Fn(KeyboardEvent)>
            );

            document
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
                .unwrap();

            closure.forget(); // Keep the closure alive
        });
    }

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
        <div class="min-h-screen bg-gray-50" tabindex="-1">
            {/* Minimal Top Bar */}
            <div class="sticky top-0 z-10 bg-white/80 backdrop-blur-md border-b border-gray-100">
                <div class="max-w-5xl mx-auto px-6 py-3">
                    <div class="flex items-center justify-between">
                        {/* Left: Student Select */}
                        <div class="flex-shrink-0">
                            <StudentSelect set_selected_student_id=set_selected_student_id />
                        </div>

                        {/* Center: Test Title */}
                        <div class="flex-1 text-center px-8">
                            <h1 class="text-lg font-medium text-gray-900 truncate">
                                {move || match &test_details.get() {
                                    Some(Some(test)) => test.name.clone(),
                                    _ => test_id()
                                }}
                            </h1>
                        </div>

                        {/* Right: Controls */}
                        <div class="flex items-center gap-3">
                            <FontControls
                                font_settings=font_settings
                                set_font_settings=set_font_settings
                            />
                            <div class="text-sm text-gray-500 font-medium hidden sm:block">
                                {move || match user_data.get() {
                                    Some(Some(user)) => format!("{} {}",
                                        user.first_name.unwrap_or("".to_string()),
                                        user.last_name.unwrap_or("".to_string())
                                    ).trim().to_string(),
                                    Some(None) => evaluator_id(),
                                    None => "Loading...".to_string(),
                                }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            {/* Instructions - Collapsible */}
            <div class="max-w-5xl mx-auto px-6 pt-4">
                <Suspense fallback=move || view! { <div></div> }>
                    {move || match test_details.get() {
                        Some(Some(test)) => {
                            if test.instructions.as_ref().map_or(false, |inst| !inst.is_empty()) {
                                view! {
                                    <div class="mb-2">
                                        <button
                                            class="flex items-center gap-2 text-sm text-blue-600 hover:text-blue-800 font-medium"
                                            on:click=move |_| set_instructions_expanded.update(|x| *x = !*x)
                                        >
                                            <svg class=move || format!("w-4 h-4 transition-transform {}",
                                                if instructions_expanded() { "rotate-90" } else { "" }
                                            ) fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path>
                                            </svg>
                                            "Test Instructions"
                                        </button>
                                        <Show when=move || instructions_expanded()>
                                            <div class="mt-2">
                                                <TestInstructions instructions=test.instructions.clone() />
                                            </div>
                                        </Show>
                                    </div>
                                }.into_view()
                            } else {
                                view! { <div></div> }.into_view()
                            }
                        },
                        _ => view! { <div></div> }.into_view()
                    }}
                </Suspense>
            </div>

            {/* Keyboard Shortcuts Help - Collapsible */}
            <div class="max-w-5xl mx-auto px-6">
                <div class="mb-4">
                    <button
                        class="flex items-center gap-2 text-sm text-blue-600 hover:text-blue-800 font-medium"
                        on:click=move |_| set_shortcuts_expanded.update(|x| *x = !*x)
                    >
                        <svg class=move || format!("w-4 h-4 transition-transform {}",
                            if shortcuts_expanded() { "rotate-90" } else { "" }
                        ) fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path>
                        </svg>
                        "Keyboard Shortcuts"
                    </button>
                    <Show when=move || shortcuts_expanded()>
                        <div class="mt-2 bg-blue-50 border border-blue-200 rounded-lg p-3 text-sm text-blue-800">
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs">
                                <span>"← → or P/N: Navigate (works in textarea)"</span>
                                <span>"1-9: Select answers (Ctrl+1-9 in textarea)"</span>
                                <span>"Ctrl+Enter: Next/Submit (works everywhere)"</span>
                                <span>"C or Ctrl+C: Focus comments"</span>
                            </div>
                        </div>
                    </Show>
                </div>
            </div>

            {/* Main Content */}
            <div class="max-w-5xl mx-auto px-6 pb-8">
                <Suspense
                    fallback=move || view! {
                        <div class="flex items-center justify-center h-96">
                            <div class="flex flex-col items-center gap-4">
                                <div class="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
                                <p class="text-gray-500 text-sm">"Loading questions..."</p>
                            </div>
                        </div>
                    }
                >
                    {move || match (questions.get(), test_details.get()) {
                        (None, _) => view! {
                            <div class="flex items-center justify-center h-96">
                                <div class="text-center">
                                    <div class="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
                                    <p class="text-gray-500">"Loading..."</p>
                                </div>
                            </div>
                        }.into_view(),
                        (Some(questions), _) if questions.is_empty() => {
                            view! {
                                <div class="flex items-center justify-center h-96">
                                    <div class="text-center">
                                        <div class="w-16 h-16 bg-red-50 rounded-full flex items-center justify-center mx-auto mb-4">
                                            <svg class="w-8 h-8 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.732-.833-2.5 0L4.268 18.5c-.77.833.192 2.5 1.732 2.5z"></path>
                                            </svg>
                                        </div>
                                        <p class="text-gray-500">"No questions found for this test."</p>
                                    </div>
                                </div>
                            }.into_view()
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
                                <div class="space-y-6">
                                    {/* Progress Section */}
                                    <div class="text-center space-y-3">
                                        {/* Minimalist Progress Bar */}
                                        <div class="w-full max-w-md mx-auto">
                                            <div class="bg-gray-100 rounded-full h-1">
                                                <div
                                                    class="bg-gradient-to-r from-blue-500 to-indigo-600 h-1 rounded-full transition-all duration-700 ease-out"
                                                    style=move || format!("width: {}%", calculate_answered_percentage())
                                                ></div>
                                            </div>
                                        </div>

                                        {/* Question Counter */}
                                        <div class="flex items-center justify-center gap-6 text-sm">
                                            <span class="text-gray-500">
                                                "Question " {move || current_card_index.get() + 1} " of " {total_questions}
                                            </span>
                                            <span class="px-3 py-1 bg-indigo-50 text-indigo-700 rounded-full font-medium">
                                                {move || current_question().point_value} " points"
                                            </span>
                                        </div>
                                    </div>

                                    {/* Card Container - Compact */}
                                    <div class="max-w-4xl mx-auto">
                                        <div class="bg-white rounded-xl shadow-sm border border-gray-100 overflow-hidden">
                                            {/* Question Header - Compact */}
                                            <div class="bg-gradient-to-r from-blue-50 to-indigo-50 border-b border-gray-100 px-6 py-3">
                                                <div class="flex items-center justify-between">
                                                    <h2 class="text-lg font-semibold text-gray-900">
                                                        "Question " {move || current_card_index.get() + 1}
                                                    </h2>
                                                    <span class="text-sm text-gray-600 font-medium">
                                                        {move || current_question().point_value} " points"
                                                    </span>
                                                </div>
                                            </div>

                                            <div class="p-6">
                                                {/* Question - Compact */}
                                                <div class="mb-6">
                                                    <div class=move || {
                                                        let question_text = current_question().word_problem.clone();
                                                        let is_long = question_text.len() > 10;
                                                        let alignment = if is_long { "text-left" } else { "text-center" };
                                                        format!("leading-relaxed {} {}", font_settings.get().get_question_classes(), alignment)
                                                    }>
                                                        {move || current_question().word_problem.clone()}
                                                    </div>
                                                </div>

                                                {/* Answer Section - Compact */}
                                                <div class="space-y-4">
                                                    {move || {
                                                        let q = current_question();
                                                        let q_clone_for_calc = q.clone();
                                                        let q_point_value = q.point_value;
                                                        match q.question_type {
                                                            QuestionType::MultipleChoice => view! {
                                                                <div class="space-y-2">
                                                                    {q.options.clone().into_iter().enumerate().map(|(index, option)| {
                                                                        let option_value = option.clone();
                                                                        let option_value_clone = option_value.clone();
                                                                        let qnumber = q.qnumber;
                                                                        let choice_number = index + 1;
                                                                        let is_checked = create_memo(move |_| {
                                                                            responses.with(|r| {
                                                                                r.get(&qnumber)
                                                                                 .map(|resp| resp.answer == option_value_clone.clone())
                                                                                 .unwrap_or(false)
                                                                            })
                                                                        });

                                                                        view! {
                                                                            <label class="group flex items-start gap-3 p-3 rounded-lg border border-gray-200 hover:border-blue-300 hover:bg-blue-50/50 transition-all duration-200 cursor-pointer">
                                                                                <div class="relative flex-shrink-0 mt-0.5">
                                                                                    <input
                                                                                        type="radio"
                                                                                        name=format!("q_{}", qnumber)
                                                                                        value=option_value.clone()
                                                                                        class="sr-only"
                                                                                        prop:checked=move || is_checked()
                                                                                        on:change=move |ev| {
                                                                                            let value = event_target_value(&ev);
                                                                                            handle_answer_change(qnumber, value);
                                                                                        }
                                                                                    />
                                                                                    <div class=move || {
                                                                                        if is_checked() {
                                                                                            "w-5 h-5 rounded-full border-2 border-blue-500 bg-blue-500 flex items-center justify-center"
                                                                                        } else {
                                                                                            "w-5 h-5 rounded-full border-2 border-gray-300 group-hover:border-blue-400 transition-colors"
                                                                                        }
                                                                                    }>
                                                                                        <Show when=move || is_checked()>
                                                                                            <div class="w-2 h-2 bg-white rounded-full"></div>
                                                                                        </Show>
                                                                                    </div>
                                                                                </div>
                                                                                <div class="flex-1 flex items-start gap-3">
                                                                                    <span class="text-xs text-gray-500 font-medium mt-1 min-w-[1rem]">
                                                                                        {choice_number}
                                                                                    </span>
                                                                                    <span class=move || format!("leading-relaxed {}", font_settings.get().get_answer_classes())>
                                                                                        {option_value}
                                                                                    </span>
                                                                                </div>
                                                                            </label>
                                                                        }
                                                                    }).collect_view()}
                                                                </div>
                                                            },
                                                            QuestionType::WeightedMultipleChoice => {
                                                                let qnumber = q.qnumber;
                                                                let weighted_options = q.get_weighted_options();

                                                                view! {
                                                                    <div class="space-y-3">
                                                                        <div class="bg-blue-50 border border-blue-200 rounded-lg p-3 text-sm text-blue-800">
                                                                            <p><strong>"Multiple selections allowed."</strong> " Each answer has different point values."</p>
                                                                        </div>

                                                                        <div class="space-y-2">
                                                                            {weighted_options.clone().into_iter().enumerate().map(|(index, option)| {
                                                                                let option_clone = option.clone();
                                                                                let option_text = option.text.clone();
                                                                                let option_text_for_memo = option_text.clone();
                                                                                let option_text_for_change = option_text.clone();
                                                                                let choice_number = index + 1;
                                                                                let qnumber = q.qnumber;

                                                                                let is_selected = create_memo(move |_| {
                                                                                    responses.with(|r| {
                                                                                        r.get(&qnumber)
                                                                                            .and_then(|resp| resp.selected_options.as_ref())
                                                                                            .map(|opts| opts.contains(&option_text_for_memo))
                                                                                            .unwrap_or(false)
                                                                                    })
                                                                                });

                                                                                view! {
                                                                                    <div class=move || {
                                                                                        let base_classes = "group flex items-center justify-between p-3 rounded-lg border transition-all duration-200";
                                                                                        if option_clone.is_selectable {
                                                                                            format!("{} border-gray-200 hover:border-blue-300 hover:bg-blue-50/50 cursor-pointer", base_classes)
                                                                                        } else {
                                                                                            format!("{} border-gray-200 bg-gray-50 cursor-not-allowed opacity-60", base_classes)
                                                                                        }
                                                                                    }
                                                                                    on:click=move |_| {
                                                                                        if option_clone.is_selectable {
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
                                                                                        <div class="flex items-center gap-3">
                                                                                            <div class="relative flex-shrink-0">
                                                                                                {if option_clone.is_selectable {
                                                                                                    view! {
                                                                                                        <div class=move || {
                                                                                                            if is_selected() {
                                                                                                                "w-5 h-5 rounded border-2 border-blue-500 bg-blue-500 flex items-center justify-center"
                                                                                                            } else {
                                                                                                                "w-5 h-5 rounded border-2 border-gray-300 group-hover:border-blue-400 transition-colors"
                                                                                                            }
                                                                                                        }>
                                                                                                            <Show when=move || is_selected()>
                                                                                                                <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                                                                                                                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                                                                                                                </svg>
                                                                                                            </Show>
                                                                                                        </div>
                                                                                                    }.into_view()
                                                                                                } else {
                                                                                                    view! {
                                                                                                        <div class="w-5 h-5 rounded border-2 border-gray-300 bg-gray-100"></div>
                                                                                                    }.into_view()
                                                                                                }}
                                                                                            </div>
                                                                                            <div class="flex items-start gap-3">
                                                                                                <span class="text-xs text-gray-500 font-medium mt-1 min-w-[1rem]">
                                                                                                    {choice_number}
                                                                                                </span>
                                                                                                <span class=move || format!("leading-relaxed {}", font_settings.get().get_answer_classes())>
                                                                                                    {option_clone.text.clone()}
                                                                                                </span>
                                                                                            </div>
                                                                                        </div>
                                                                                        <div class="flex items-center gap-2">
                                                                                            <span class=move || {
                                                                                                if option_clone.points >= 0 {
                                                                                                    "text-green-600 font-semibold text-sm"
                                                                                                } else {
                                                                                                    "text-red-600 font-semibold text-sm"
                                                                                                }
                                                                                            }>
                                                                                                {if option_clone.points >= 0 { "+" } else { "" }}
                                                                                                {option_clone.points}
                                                                                                " pts"
                                                                                            </span>
                                                                                            {if !option_clone.is_selectable {
                                                                                                view! {
                                                                                                    <span class="text-xs text-gray-400 italic">"(info only)"</span>
                                                                                                }.into_view()
                                                                                            } else {
                                                                                                view! { <span></span> }.into_view()
                                                                                            }}
                                                                                        </div>
                                                                                    </div>
                                                                                }
                                                                            }).collect_view()}
                                                                        </div>

                                                                        <div class="bg-gray-50 border border-gray-200 rounded-lg p-3">
                                                                            <div class="text-sm text-gray-700">
                                                                                "Current score: "
                                                                                <span class="font-semibold text-indigo-600">
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
                                                                    <div class="flex gap-4">
                                                                        <button
                                                                            type="button"
                                                                            class=move || format!("flex-1 py-3 px-4 rounded-lg font-medium transition-all duration-200 flex items-center justify-center gap-2 {}",
                                                                                if is_true() {
                                                                                    "bg-green-500 text-white shadow-lg transform scale-105"
                                                                                } else {
                                                                                    "bg-white text-gray-700 border-2 border-gray-200 hover:border-green-400 hover:bg-green-50"
                                                                                }
                                                                            )
                                                                            on:click=move |_| {
                                                                                handle_answer_change(qnumber, "true".to_string());
                                                                            }
                                                                        >
                                                                            <span class="text-xs text-gray-500 font-medium">1</span>
                                                                            <span class=move || font_settings.get().get_answer_classes()>
                                                                                "Yes" //manually
                                                                            //changed to "Yes" for consistency
                                                                            </span>
                                                                        </button>
                                                                        <button
                                                                            type="button"
                                                                            class=move || format!("flex-1 py-3 px-4 rounded-lg font-medium transition-all duration-200 flex items-center justify-center gap-2 {}",
                                                                                if is_false() {
                                                                                    "bg-red-500 text-white shadow-lg transform scale-105"
                                                                                } else {
                                                                                    "bg-white text-gray-700 border-2 border-gray-200 hover:border-red-400 hover:bg-red-50"
                                                                                }
                                                                            )
                                                                            on:click=move |_| {
                                                                                handle_answer_change(qnumber, "false".to_string());
                                                                            }
                                                                        >
                                                                            <span class="text-xs text-gray-500 font-medium">2</span>
                                                                            <span class=move || font_settings.get().get_answer_classes()>
                                                                                "No" //manually
                                                                            //changed to "No" for consistency
                                                                            </span>
                                                                        </button>
                                                                    </div>
                                                                }
                                                            }
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
                                                                            class=move || format!("w-full p-3 border border-gray-200 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 resize-none transition-all duration-200 {}",
                                                                                font_settings.get().get_answer_classes())
                                                                            prop:value=move || answer_value()
                                                                            on:input=move |ev| {
                                                                                let value = event_target_value(&ev);
                                                                                handle_answer_change(qnumber, value);
                                                                            }
                                                                            placeholder="Type your answer here..."
                                                                            rows="3"
                                                                        ></textarea>
                                                                    </div>
                                                                }
                                                            }
                                                        }
                                                    }}

                                                    {/* Comments Section - Compact */}
                                                    <div class="border-t border-gray-100 pt-4">
                                                        <label class="block text-sm font-medium text-gray-700 mb-2">
                                                            "Notes & Comments " <span class="text-xs text-gray-400">"(Press C to focus)"</span>
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
                                                                    class="w-full p-3 border border-gray-200 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 resize-none transition-all duration-200 text-sm"
                                                                    prop:value=move || comment_value()
                                                                    on:input=move |ev| {
                                                                        let value = event_target_value(&ev);
                                                                        handle_comment_change(qnumber, value);
                                                                    }
                                                                    placeholder="Add any notes or observations about this question..."
                                                                    rows="2"
                                                                ></textarea>
                                                            }
                                                        }}
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    {/* Navigation - Compact */}
                                    <div class="flex items-center justify-center gap-4 pt-4">
                                        <button
                                            class="flex items-center gap-2 px-4 py-2 bg-white border border-gray-200 rounded-lg text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200 shadow-sm"
                                            disabled=move || current_card_index.get() == 0
                                            on:click=go_to_previous_card
                                        >
                                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"></path>
                                            </svg>
                                            "Previous"
                                        </button>

                                        {move || {
                                            if current_card_index.get() == total_questions - 1 {
                                                view! {
                                                    <Show when=move || !is_submitted.get() fallback=move || view! {
                                                        <button
                                                            class="flex items-center gap-2 px-6 py-2 bg-gray-900 text-white rounded-lg hover:bg-gray-800 transition-all duration-200 shadow-lg hover:shadow-xl transform hover:scale-105"
                                                            on:click=move |_| {
                                                                let navigate=leptos_router::use_navigate();
                                                                navigate("/dashboard", Default::default());
                                                            }
                                                        >
                                                            "Return to Dashboard"
                                                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18"></path>
                                                            </svg>
                                                        </button>
                                                    }>
                                                        <button
                                                            class="flex items-center gap-2 px-6 py-2 bg-gradient-to-r from-blue-600 to-indigo-600 text-white rounded-lg hover:from-blue-700 hover:to-indigo-700 transition-all duration-200 shadow-lg hover:shadow-xl transform hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none"
                                                            on:click=move |_| {
                                                                handle_submit.dispatch(());
                                                                set_is_submitted.set(true);
                                                            }
                                                            disabled=move || selected_student_id.get().is_none()
                                                        >
                                                            "Submit Assessment"
                                                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
                                                            </svg>
                                                        </button>
                                                    </Show>
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <button
                                                        class="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-blue-600 to-indigo-600 text-white rounded-lg hover:from-blue-700 hover:to-indigo-700 transition-all duration-200 shadow-lg hover:shadow-xl transform hover:scale-105"
                                                        on:click=go_to_next_card
                                                    >
                                                        "Next"
                                                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path>
                                                        </svg>
                                                    </button>
                                                }.into_view()
                                            }
                                        }}
                                    </div>

                                    {/* Success Message */}
                                    <Show when=move || is_submitted.get()>
                                        <div class="text-center pt-4">
                                            <div class="inline-flex items-center gap-3 px-6 py-3 bg-green-50 border border-green-200 rounded-lg text-green-800">
                                                <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                                                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"></path>
                                                </svg>
                                                "Assessment submitted successfully!"
                                            </div>
                                        </div>
                                    </Show>
                                </div>
                            }.into_view()
                        }
                    }}
                </Suspense>
            </div>
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
        <div class="min-w-[200px]">
            <select
                class="w-full px-3 py-2 text-sm border border-gray-200 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 bg-white transition-all duration-200"
                on:change=move |ev| {
                    let value = event_target_value(&ev).parse().ok();
                    set_selected_student_id.set(value);
                }
            >
                <option value="">"Select student..."</option>
                <Suspense fallback=move || view! {
                    <option>"Loading..."</option>
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
