use crate::app::components::auth::server_auth_components::ServerAuthGuard;
use crate::app::components::header::Header;
use crate::app::components::question_builder::BuildingQuestion;
use crate::app::models::assessment::ScopeEnum;
use crate::app::models::student::GradeEnum;
use crate::app::models::test::BenchmarkCategory;
use crate::app::models::test::{CreateNewTestRequest, Test, TestType, UpdateTestRequest};
use crate::app::models::{CreateNewQuestionRequest, Question, QuestionType, WeightedOption};
use crate::app::server_functions::assessments::update_assessment_score;
use crate::app::server_functions::courses::get_courses;
use crate::app::server_functions::questions::{add_question, delete_questions, get_questions};
use crate::app::server_functions::tests::get_tests;
use crate::app::server_functions::tests::{add_test, get_test, score_overrider, update_test};
use crate::app::utils::BenchmarkUtils;
use leptos::prelude::*;
use leptos::*;
use leptos_router::*;
use std::str::FromStr;
use strum::IntoEnumIterator;

#[cfg(feature = "hydrate")]
use leptos::wasm_bindgen::JsCast;

#[component]
pub fn TestBuilder() -> impl IntoView {
    view! {
        <ServerAuthGuard page_path="/testbuilder">
            <TestBuilderContent />
        </ServerAuthGuard>
    }
}

async fn get_next_variant_number_for_new_test(
    test_name: &str,
) -> Result<i32, leptos::ServerFnError> {
    let all_tests = get_tests().await?;

    // Find all tests with the same name (exact match or base name match)
    let related_tests: Vec<&Test> = all_tests
        .iter()
        .filter(|test| {
            let test_base_name = if test.name.contains(" - ") {
                test.name.split(" - ").next().unwrap_or(&test.name)
            } else {
                &test.name
            };
            test_base_name == test_name || test.name == test_name
        })
        .collect();

    // Find the highest variant number
    let max_variant = related_tests
        .iter()
        .map(|test| test.test_variant)
        .max()
        .unwrap_or(-1); // Start from -1 so first test gets variant 0

    Ok(max_variant + 1)
}

#[component]
pub fn TestBuilderContent() -> impl IntoView {
    const TAB_BUTTON_STYLE: &str =
        "bg-[#00356b] px-8 py-2 rounded text-white transition-all duration-1000 ease-in-out ml-2";
    const INPUT_STYLE: &str = "w-[20rem] h-12 border-[#00356b] border pr-4 pl-6 py-4 text-[#00356b] rounded transition-all duration-1000 ease-in-out";
    const INPUT_STYLE_BOX: &str = "mt-5 h-[30rem] w-full max-w-7xl rounded border-[#00356b] border text-wrap pl-4 align-text-top text-start text-[#00356b] align-top transition-all duration-1000 ease-in-out";
    const DROPDOWN_STYLE: &str = "w-40 h-12 border-[#00356b] border pr-4 pl-6 py-2 text-[#00356b] rounded transition-all duration-1000 ease-in-out";

    let params = use_params_map();
    let maybe_id = params.with(|params| params.get("test_id").cloned());

    // Signal to track if we're in edit mode
    let (is_edit_mode, set_is_edit_mode) = create_signal(false);

    let test_resource = create_resource(
        move || maybe_id.clone(),
        |id| async move {
            match id {
                Some(test_id) => get_test(test_id).await.ok(),
                None => None,
            }
        },
    );

    let courses_resource = create_resource(
        || (),
        |_| async move {
            match get_courses().await {
                Ok(courses) => courses,
                Err(e) => {
                    log::error!("Failed to load courses: {:?}", e);
                    Vec::new()
                }
            }
        },
    );

    let (selected_tab, set_selected_tab) = create_signal(0);
    let (test_title, set_test_title) = create_signal(String::new());
    let (test_instructions, set_test_instructions) = create_signal(String::new());
    let (test_area, set_test_area) = create_signal(String::new());
    let (school_year, set_school_year) = create_signal(String::new());
    let (grade_level, set_grade_level) = create_signal::<Option<GradeEnum>>(None);
    let (benchmark_categories, set_benchmark_categories) =
        create_signal::<Vec<(i32, i32, i32, String)>>(Vec::new());
    let (test_variant, set_test_variant) = create_signal(0);
    let (test_comments, set_test_comments) = create_signal(String::new());
    let (test_id, set_test_id) = create_signal(String::new());
    let (scope, set_scope) = create_signal::<Option<ScopeEnum>>(None);
    let (course_id, set_course_id) = create_signal::<Option<i32>>(None);

    //Signal to track which question to autofocus
    let (auto_focus_question, set_auto_focus_question) = create_signal::<Option<i32>>(None);
    let (default_point_value, set_default_point_value) = create_signal::<Option<i32>>(None);
    let (default_question_type, set_default_question_type) =
        create_signal::<Option<QuestionType>>(None);

    //Signals for TestVariation Management
    let (is_variation, set_is_variation) = create_signal(false);
    let (base_test_name, set_base_test_name) = create_signal(String::new());
    let (variation_type_display, set_variation_type_display) = create_signal(String::new());
    let (related_variations, set_related_variations) = create_signal(Vec::<Test>::new());

    let (error_message, set_error_message) = create_signal(String::new());
    let (show_error, set_show_error) = create_signal(false);
    let (is_submitting, set_is_submitting) = create_signal(false);

    let (questions, set_questions) = create_signal(Vec::<Question>::new());

    let (force_update_key, set_force_update_key) = create_signal(0);

    // Resource to load questions when in edit mode
    let questions_resource = create_resource(
        move || test_id.get(),
        |tid| async move {
            if tid.is_empty() {
                return Vec::new();
            }
            match get_questions(tid).await {
                Ok(qs) => qs,
                Err(e) => {
                    log::error!("Failed to load questions: {:?}", e);
                    Vec::new()
                }
            }
        },
    );

    let add_new_question = move |_| {
        let new_question_number = questions().len() + 1;
        set_questions.update(|qs| {
            // Use the default question type if set, otherwise fall back to MultipleChoice
            let question_type = default_question_type
                .get()
                .unwrap_or(QuestionType::MultipleChoice);

            let mut new_question = Question::new(
                String::new(),
                default_point_value.get().unwrap_or(1),
                question_type.clone(), // Use the variable directly
                vec![],
                String::new(),
                new_question_number as i32,
                test_id(),
            );

            // Set up initial options based on the ACTUAL question type being used
            match question_type {
                QuestionType::MultipleChoice => {
                    new_question.options = vec!["".to_string(), "".to_string()];
                    new_question.correct_answer = "".to_string();
                    new_question.weighted_options = None;
                }
                QuestionType::TrueFalse => {
                    new_question.options = vec!["true".to_string(), "false".to_string()];
                    new_question.correct_answer = "true".to_string();
                    new_question.weighted_options = None;
                }
                QuestionType::WeightedMultipleChoice => {
                    new_question.options = Vec::new();
                    new_question.correct_answer = String::new();
                    let default_weighted_options = vec![
                        WeightedOption::new("".to_string(), 1, true),
                        WeightedOption::new("".to_string(), 1, true),
                    ];
                    new_question.set_weighted_options(default_weighted_options);
                }
                _ => {
                    // Fallback for any other types
                    new_question.options = vec!["".to_string(), "".to_string()];
                    new_question.correct_answer = "".to_string();
                    new_question.weighted_options = None;
                }
            }

            qs.push(new_question);
        });

        // Focus logic remains the same
        #[cfg(feature = "hydrate")]
        {
            request_animation_frame(move || {
                set_auto_focus_question(Some(new_question_number as i32));
            });
        }
        #[cfg(not(feature = "hydrate"))]
        {
            set_auto_focus_question(Some(new_question_number as i32));
        }
    };

    let clear_auto_focus = move |question_number: i32| {
        // Only clear if this is the currently focused question
        if auto_focus_question() == Some(question_number) {
            set_auto_focus_question(None);
        }
    };

    let update_question = move |index: usize, updated_question: Question| {
        set_questions.update(|qs| {
            if index < qs.len() {
                qs[index] = updated_question;
            }
        });
    };

    let remove_question = move |index: usize| {
        set_questions.update(|qs| {
            qs.remove(index);

            // Renumber all questions to ensure sequential numbering
            for (i, q) in qs.iter_mut().enumerate() {
                q.qnumber = (i + 1) as i32;
            }
        });
    };

    //Load related variations in edit mode
    let related_variations_resource = create_resource(
        move || (test_id.get(), is_edit_mode()),
        |(current_test_id, is_editing)| async move {
            if !is_editing || current_test_id.is_empty() {
                return Vec::new();
            }

            match get_tests().await {
                Ok(all_tests) => {
                    // Find the current test to determine the base name
                    let current_test = all_tests.iter().find(|t| t.test_id == current_test_id);

                    if let Some(current) = current_test {
                        let base_name = if current.name.contains(" - ") {
                            current
                                .name
                                .split(" - ")
                                .next()
                                .unwrap_or(&current.name)
                                .to_string()
                        } else {
                            current.name.clone()
                        };

                        // Find all related tests (base + variations)
                        all_tests
                            .into_iter()
                            .filter(|test| {
                                let test_base_name = if test.name.contains(" - ") {
                                    test.name
                                        .split(" - ")
                                        .next()
                                        .unwrap_or(&test.name)
                                        .to_string()
                                } else {
                                    test.name.clone()
                                };
                                test_base_name == base_name && test.test_id != current_test_id
                            })
                            .collect()
                    } else {
                        Vec::new()
                    }
                }
                Err(_) => Vec::new(),
            }
        },
    );

    //Helper to display benchmark categories in test summary
    let benchmark_summary = move || {
        if benchmark_categories().is_empty() {
            "No benchmark categories defined".to_string()
        } else {
            let temp_categories = BenchmarkUtils::from_tuples(benchmark_categories());
            BenchmarkUtils::format_summary(&temp_categories)
        }
    };

    // Effect to load test data when a test_id is available
    create_effect(move |_| {
        if let Some(Some(test)) = test_resource.get() {
            set_is_edit_mode(true);
            set_test_id(test.test_id.clone());
            set_test_title(test.name.clone());
            set_grade_level(test.grade_level.clone());
            set_test_area(test.testarea.clone().to_string());
            set_school_year(test.school_year.clone().unwrap_or_default());
            set_test_comments(test.comments.clone());
            set_test_variant(test.test_variant);
            set_scope(test.scope.clone());
            set_course_id(test.course_id.clone());
            set_test_instructions(test.instructions.clone().unwrap_or_default());

            // Convert BenchmarkCategory to our internal tuple representation using utilities
            let categories = test.benchmark_categories.clone().unwrap_or_default();
            let tuple_categories = BenchmarkUtils::to_tuples(categories);
            set_benchmark_categories(tuple_categories);
        }
    });

    // Effect to load questions when questions_resource updates
    create_effect(move |_| {
        if let Some(loaded_questions) = questions_resource.get() {
            if !loaded_questions.is_empty() {
                // Sort questions by question number
                let mut sorted_questions = loaded_questions.clone();
                sorted_questions.sort_by_key(|q| q.qnumber);

                // Renumber questions sequentially starting from 1
                for (i, q) in sorted_questions.iter_mut().enumerate() {
                    q.qnumber = (i + 1) as i32;
                }

                set_questions(sorted_questions);
            }
        }
    });

    //Loading related variations when in edit mode
    create_effect(move |_| {
        if let Some(variations) = related_variations_resource.get() {
            set_related_variations(variations);
        }
    });

    // For editing a variation
    create_effect(move |_| {
        if let Some(Some(test)) = test_resource.get() {
            let is_var = test.name.contains(" - ")
                && (test.name.to_lowercase().contains("randomized")
                    || test.name.to_lowercase().contains("distinct")
                    || test.name.to_lowercase().contains("practice")
                    || test.comments.to_lowercase().contains("variation:"));

            set_is_variation(is_var);

            if is_var {
                if let Some(base_name) = test.name.split(" - ").next() {
                    set_base_test_name(base_name.to_string());
                }
                if let Some(variation_part) = test.name.split(" - ").nth(1) {
                    set_variation_type_display(variation_part.to_string());
                }
            }
        }
    });

    create_effect(move |_| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;
            use wasm_bindgen::JsCast;
            use web_sys::KeyboardEvent;

            let handle_keydown = Closure::wrap(Box::new(move |event: KeyboardEvent| {
                // Check if we're on the questions tab (tab 1)
                if selected_tab() == 1 {
                    // Check for Ctrl++ (Ctrl + Plus/Equal key)
                    if event.ctrl_key() && (event.key() == "+" || event.key() == "=") {
                        event.prevent_default();
                        add_new_question(());
                    }
                }
            }) as Box<dyn FnMut(KeyboardEvent)>);

            let window = web_sys::window().unwrap();

            // Convert the closure to a Function - use into() instead of unchecked_into on reference
            let function = handle_keydown.as_ref().unchecked_ref::<js_sys::Function>();
            window
                .add_event_listener_with_callback("keydown", function)
                .unwrap();

            // Store the closure
            let stored_closure = store_value(handle_keydown);

            // Cleanup function
            on_cleanup(move || {
                let window = web_sys::window().unwrap();
                stored_closure.with_value(|closure| {
                    let function = closure.as_ref().unchecked_ref::<js_sys::Function>();
                    window
                        .remove_event_listener_with_callback("keydown", function)
                        .unwrap();
                });
            });
        }
    });

    let validate_test_form = move || -> Result<(), String> {
        // Input validation
        if test_title().is_empty() {
            return Err("Test title cannot be empty".to_string());
        }

        if test_area().is_empty() {
            return Err("Test area must be selected".to_string());
        }

        match TestType::from_str(&test_area()) {
            Ok(_) => Ok(()),
            Err(_) => Err("Invalid test area selected".to_string()),
        }
    };

    let validate_questions = move || -> Result<Vec<CreateNewQuestionRequest>, String> {
        let current_questions = questions.get();

        if current_questions.is_empty() {
            return Err("Please add at least one question".to_string());
        }

        let valid_questions: Vec<_> = current_questions
            .into_iter()
            .filter(|q| {
                let is_valid = match &q.question_type {
                    QuestionType::MultipleChoice => {
                        !q.word_problem.is_empty()
                            && q.point_value > 0
                            && !q.options.is_empty()
                            && !q.correct_answer.is_empty()
                            && q.options.contains(&q.correct_answer)
                    }
                    QuestionType::TrueFalse => {
                        !q.word_problem.is_empty()
                            && q.point_value > 0
                            && (q.correct_answer == "true" || q.correct_answer == "false")
                            && !q.correct_answer.is_empty()
                    }
                    QuestionType::WeightedMultipleChoice => {
                        let weighted_options = q.get_weighted_options();
                        !q.word_problem.is_empty()
                            && q.point_value > 0
                            && !weighted_options.is_empty()
                            && weighted_options.iter().any(|opt| opt.is_selectable)
                            && weighted_options
                                .iter()
                                .all(|opt| !opt.text.trim().is_empty())
                    }
                    _ => false,
                };
                if !is_valid {
                    log::warn!("Invalid question found: {:?}", q);
                }
                is_valid
            })
            .collect();

        if valid_questions.is_empty() {
            return Err("No valid questions to submit".to_string());
        }

        let question_requests: Vec<CreateNewQuestionRequest> = valid_questions
            .into_iter()
            .map(|q| {
                let mut request = CreateNewQuestionRequest::from_question(&q);
                request.testlinker = q.testlinker.clone();
                request
            })
            .collect();

        Ok(question_requests)
    };

    let on_submit_test_and_questions = move |_| {
        if is_submitting() {
            return;
        }

        set_is_submitting(true);
        set_show_error(false);

        // First validate the test form
        if let Err(e) = validate_test_form() {
            set_show_error(true);
            set_error_message(e);
            set_is_submitting(false);
            return;
        }

        // Create the test
        let test_type = TestType::from_str(&test_area()).unwrap(); // Safe because we validated earlier

        // Calculate total points from current questions for initial test creation
        let total_points: i32 = questions.get().iter().map(|q| q.point_value).sum();

        // Convert our tuple representation back to BenchmarkCategory
        let converted_cats = if benchmark_categories().is_empty() {
            None
        } else {
            // Convert tuples to BenchmarkCategory objects
            let temp_categories = BenchmarkUtils::from_tuples(benchmark_categories());

            // Validate all categories
            match BenchmarkUtils::validate_all(&temp_categories) {
                Ok(_) => Some(temp_categories),
                Err(validation_error) => {
                    set_show_error(true);
                    set_error_message(format!(
                        "Benchmark category validation failed: {}",
                        validation_error
                    ));
                    set_is_submitting(false);
                    return;
                }
            }
        };

        // Convert scope back to Enum and course_id to i32
        let scope_value = scope();
        if scope_value != Some(ScopeEnum::Course) && course_id().is_some() {
            set_course_id(None);
        };
        let course_id_value = course_id();

        let converted_clone = converted_cats.clone();
        let test_title_clone = test_title();
        let test_comments_clone = test_comments();
        let school_year_clone = school_year();
        let grade_level_clone = grade_level();
        let scope_value_clone = scope_value.clone();
        let course_id_value_clone = course_id_value.clone();
        let test_instructions_clone = test_instructions();

        spawn_local(async move {
            let current_test_id = test_id();
            let is_editing = is_edit_mode();

            // Auto-assign variant number for new tests, keep existing for edits
            let final_test_variant = if !is_editing {
                // For new tests, get the next available variant number
                match get_next_variant_number_for_new_test(&test_title_clone).await {
                    Ok(num) => {
                        log::info!("Auto-assigned variant number: {}", num);
                        set_test_variant(num);
                        num
                    }
                    Err(e) => {
                        log::error!("Failed to get next variant number: {:?}", e);
                        set_show_error(true);
                        set_error_message(format!("Failed to determine variant number: {}", e));
                        set_is_submitting(false);
                        return;
                    }
                }
            } else {
                test_variant() // Keep existing variant for edits
            };

            // If we're editing, update the test rather than creating a new one
            let new_test_id = if is_editing && !current_test_id.is_empty() {
                let update_test_request = UpdateTestRequest::new(
                    test_title_clone.clone(),
                    total_points,
                    Some(test_instructions_clone.clone()),
                    test_comments_clone.clone(),
                    test_type.clone(),
                    Some(school_year_clone.clone()),
                    converted_clone.clone(),
                    final_test_variant,
                    grade_level_clone.clone(),
                    current_test_id.clone(),
                    scope_value_clone.clone(),
                    course_id_value_clone.clone(),
                );

                log::info!("Updating test with ID: {}", current_test_id);
                match update_test(update_test_request).await {
                    Ok(_) => {
                        log::info!("Successfully updated test");
                    }
                    Err(e) => {
                        log::error!("Error updating test: {:?}", e);
                        set_show_error(true);
                        set_error_message(format!("Failed to update test: {}", e));
                        set_is_submitting(false);
                        return;
                    }
                }
                current_test_id
            } else {
                // Create a new test with auto-assigned variant
                let add_test_request = CreateNewTestRequest::new(
                    test_title_clone.clone(),
                    total_points,
                    Some(test_instructions_clone.clone()),
                    test_comments_clone.clone(),
                    test_type.clone(),
                    Some(school_year_clone.clone()),
                    converted_clone.clone(),
                    final_test_variant,
                    grade_level_clone.clone(),
                    scope_value_clone.clone(),
                    course_id_value_clone.clone(),
                );

                match add_test(add_test_request).await {
                    Ok(added_test) => {
                        let new_id = added_test.test_id.clone();
                        set_test_id(new_id.clone());
                        log::info!(
                            "Created test with ID: {} and variant: {}",
                            new_id,
                            final_test_variant
                        );
                        new_id
                    }
                    Err(e) => {
                        log::error!("Error creating test: {:?}", e);
                        set_show_error(true);
                        set_error_message(format!("Failed to create test: {}", e));
                        set_is_submitting(false);
                        return;
                    }
                }
            };

            // Now validate questions
            let question_requests_result = validate_questions();

            if let Err(e) = question_requests_result {
                set_show_error(true);
                set_error_message(e.clone());
                set_is_submitting(false);
                log::warn!("Failed to validate questions: {}", e);
                return;
            }

            let mut question_requests = question_requests_result.unwrap();

            // Update question requests with the test ID
            for q in &mut question_requests {
                q.testlinker = new_test_id.clone();
            }

            if is_editing {
                match delete_questions(new_test_id.clone()).await {
                    Ok(_) => {
                        log::info!(
                            "Successfully deleted existing questions for test: {}",
                            new_test_id
                        );
                    }
                    Err(e) => {
                        log::error!("Failed to delete existing questions:{:?}", e);
                        set_show_error(true);
                        set_error_message(format!("Failed to delete existing questions: {}", e));
                        set_is_submitting(false);
                        return;
                    }
                }
            }

            // Add questions one by one
            let mut success_count = 0;
            for question in &question_requests {
                match add_question(new_test_id.clone(), question.clone()).await {
                    Ok(_) => {
                        log::info!("Added question {}", question.qnumber);
                        success_count += 1;
                    }
                    Err(e) => {
                        log::error!("Failed to add question {}: {:?}", question.qnumber, e)
                    }
                }
            }

            log::info!(
                "Added {}/{} questions successfully",
                success_count,
                &question_requests.len()
            );

            // Always update assessment scores, whether this is a new test or an edited one
            match update_assessment_score(new_test_id.clone()).await {
                Ok(_) => {
                    log::info!(
                        "Successfully updated assessment scores for test: {}",
                        new_test_id
                    );
                }
                Err(e) => {
                    log::error!("Failed to update assessment scores: {:?}", e);
                    // We continue even if updating assessment scores fails
                    // This allows the user to still complete their task
                }
            }

            set_is_submitting(false);

            // Navigate to the test list page only after all questions are processed
            let navigate = leptos_router::use_navigate();
            navigate("/test-manager", Default::default());
        });
    };

    view! {
        <Header />
        <main class="w-full max-w-5xl mx-auto px-6 py-12">
            <div class="flex flex-col mb-8">
                <h1 class="text-3xl font-semibold text-gray-800">
                    {move || if is_edit_mode() { "Edit Test" } else { "Test Builder" }}
                </h1>
                <div class="h-0.5 w-full bg-gray-300 mt-3"></div>
            </div>

            {move || {
                if is_variation() && is_edit_mode() {
                    let variation_info = variation_type_display();
                    let (info_class, icon_color, info_text, warning_text) = if variation_info.to_lowercase().contains("randomized") {
                        (
                            "bg-blue-50 border-blue-400",
                            "text-blue-400",
                            "This is a randomized variation with shuffled questions and answer choices.",
                            Some("Questions are automatically generated from the base test. You can edit them but they may be overwritten if regenerated.")
                        )
                    } else if variation_info.to_lowercase().contains("distinct") {
                        (
                            "bg-green-50 border-green-400",
                            "text-green-400",
                            "This is a distinct variation with entirely different questions.",
                            None
                        )
                    } else if variation_info.to_lowercase().contains("practice") {
                        (
                            "bg-purple-50 border-purple-400",
                            "text-purple-400",
                            "This is a practice variation for student preparation.",
                            None
                        )
                    } else {
                        (
                            "bg-blue-50 border-blue-400",
                            "text-blue-400",
                            "This is a test variation.",
                            None
                        )
                    };

                    view! {
                        <div class=format!("border-l-4 p-4 mb-6 {}", info_class)>
                            <div class="flex">
                                <div class="flex-shrink-0">
                                    <svg class=format!("h-5 w-5 {}", icon_color) viewBox="0 0 20 20" fill="currentColor">
                                        <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
                                    </svg>
                                </div>
                                <div class="ml-3 flex-1">
                                    <h3 class="text-sm font-medium text-blue-800">
                                        "Editing Test Variation"
                                    </h3>
                                    <div class="mt-2 text-sm text-blue-700">
                                        <p>
                                            "This is a " <strong>{variation_type_display}</strong> " variation of "
                                            <strong>{base_test_name}</strong>
                                        </p>
                                        <p class="mt-1 text-xs">{info_text}</p>
                                        {if let Some(warning) = warning_text {
                                            view! {
                                                <p class="mt-2 text-xs bg-yellow-100 text-yellow-800 p-2 rounded border border-yellow-200">
                                                    {warning}
                                                </p>
                                            }.into_view()
                                        } else {
                                            view! { <div></div> }.into_view()
                                        }}
                                        <div class="mt-3">
                                            <div class="flex items-center space-x-4">
                                                <button
                                                    class="text-sm bg-blue-100 hover:bg-blue-200 text-blue-800 px-3 py-1 rounded-md transition-colors"
                                                    on:click=move |_| {
                                                        let navigate = leptos_router::use_navigate();
                                                        navigate("/test-variations", Default::default());
                                                    }
                                                >
                                                    "Manage All Variations"
                                                </button>
                                                <span class="text-blue-600">
                                                    {move || {
                                                        let count = related_variations().len();
                                                        if count > 0 {
                                                            format!("{} related test(s)", count)
                                                        } else {
                                                            "No other variations".to_string()
                                                        }
                                                    }}
                                                </span>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                } else {
                    view! { <div></div> }
                }
            }}

            // Error message display
            <Show when=move || show_error()>
                <div class="bg-red-50 border-l-4 border-red-500 text-red-700 p-4 rounded mb-6">
                    <div class="flex">
                        <div class="flex-shrink-0">
                            <svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                            </svg>
                        </div>
                        <div class="ml-3">
                            {error_message}
                        </div>
                    </div>
                </div>
            </Show>

            <div class="flex space-x-2 mb-8 border-b">
                <button
                    class=move || {
                        let base_class = "px-6 py-3 text-sm font-medium transition-all duration-200 focus:outline-none";
                        let active_class = "text-[#00356b] border-b-2 border-[#00356b]";
                        let inactive_class = "text-gray-500 hover:text-[#00356b]";
                        format!("{} {}", base_class, if selected_tab() == 0 { active_class } else { inactive_class })
                    }
                    on:click=move |_| set_selected_tab.set(0)
                >
                    "Test Details"
                </button>
                <button
                    class=move || {
                        let base_class = "px-6 py-3 text-sm font-medium transition-all duration-200 focus:outline-none";
                        let active_class = "text-[#00356b] border-b-2 border-[#00356b]";
                        let inactive_class = "text-gray-500 hover:text-[#00356b]";
                        format!("{} {}", base_class, if selected_tab() == 1 { active_class } else { inactive_class })
                    }
                    on:click=move |_| set_selected_tab.set(1)
                >
                    "Questions"
                </button>
                <button
                    class=move || {
                        let base_class = "px-6 py-3 text-sm font-medium transition-all duration-200 focus:outline-none";
                        let active_class = "text-[#00356b] border-b-2 border-[#00356b]";
                        let inactive_class = "text-gray-500 hover:text-[#00356b]";
                        format!("{} {}", base_class, if selected_tab() == 2 { active_class } else { inactive_class })
                    }
                    on:click=move |_| set_selected_tab.set(2)
                >
                    "Review"
                </button>
            </div>

            <div class="tab-content">
                {move || match selected_tab() {
                    0 => view!{
                        <div class="space-y-6">
                            <div class="form-group">
                                <label class="block text-sm font-medium text-gray-700 mb-1">
                                    "Test Title"
                                </label>
                                <input
                                    type="text"
                                    placeholder="Enter test title"
                                    class="w-full px-4 py-3 rounded-md border border-gray-300 shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
                                    value=test_title
                                    on:input=move |event| {
                                        set_test_title(event_target_value(&event));
                                    }
                                />
                            </div>

                            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                <div class="form-group">
                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                        "Test Area"
                                    </label>
                                    <select
                                        class="w-full px-4 py-3 rounded-md border border-gray-300 shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
                                        prop:value=test_area
                                        on:change=move |event| {
                                            set_test_area(event_target_value(&event));
                                        }
                                    >
                                        <option value="">"Please Select a Value"</option>
                                        <option value="Reading">"Reading"</option>
                                        <option value="Math">"Math"</option>
                                    </select>
                                </div>

                                <div class="form-group">
                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                        "School Year"
                                    </label>
                                    <select
                                        class="w-full px-4 py-3 rounded-md border border-gray-300 shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
                                        prop:value=school_year
                                        on:input=move |event| {
                                            set_school_year(event_target_value(&event));
                                        }
                                    >
                                        <option value="">"Please Select a Year"</option>
                                        <option value="2023-2024">"2023-2024"</option>
                                        <option value="2024-2025">"2024-2025"</option>
                                        <option value="2025-2026">"2025-2026"</option>
                                    </select>
                                </div>
                            </div>

                            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                <div class="form-group">
                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                        "Test Grade Level"
                                    </label>
                                    <select
                                        required
                                        class="w-full px-4 py-3 rounded-md border border-gray-300 shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
                                        prop:value={move || grade_level.get().map(|g| g.to_string()).unwrap_or_else(|| "None".to_string())}
                                        on:change=move |event| {
                                            let value = event_target_value(&event);
                                            match value.parse::<GradeEnum>() {
                                                Ok(grade_enum) => set_grade_level(Some(grade_enum)),
                                                Err(_) => ()
                                            }
                                        }
                                    >
                                        <option value="">Select Grade</option>
                                        <option value="None">"None"</option>
                                        {GradeEnum::iter().map(|grade| view! {
                                            <option value=format!("{}", grade)>
                                                {format!("{}", grade)}
                                            </option>
                                        }).collect::<Vec<_>>()}
                                    </select>
                                </div>

                                <div class="form-group">
                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                        "Variant Number"
                                    </label>
                                    {move || {
                                        if is_edit_mode() {
                                            // In edit mode, show the current variant number as read-only
                                            view! {
                                                <div class="w-full px-4 py-3 rounded-md border border-gray-300 bg-gray-100 text-gray-600">
                                                    {test_variant().to_string()}
                                                    <span class="text-sm text-gray-500 ml-2">"(Auto-assigned)"</span>
                                                </div>
                                            }.into_view()
                                        } else {
                                            // In create mode, show that it will be auto-assigned
                                            view! {
                                                <div class="w-full px-4 py-3 rounded-md border border-gray-300 bg-gray-100 text-gray-600">
                                                    "Will be auto-assigned"
                                                    <span class="text-sm text-gray-500 ml-2">"(Next available number)"</span>
                                                </div>
                                            }.into_view()
                                        }
                                    }}
                                </div>

                                <div class="form-group">
                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                        "Scope"
                                    </label>
                                    <select
                                        class="w-full px-4 py-3 rounded-md border border-gray-300 shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
                                        prop:value={move ||  match scope() {
                                            Some(s) => s.to_string(),
                                            None => "none".to_string()
                                        }}
                                        on:change=move |event| {
                                            let value = event_target_value(&event);
                                            if value == "none" {
                                                set_scope(None);
                                            } else {
                                                match ScopeEnum::from_str(&value) {
                                                    Ok(scope_enum) => set_scope(Some(scope_enum)),
                                                    Err(_) => set_scope(None),
                                                }
                                            }
                                        }
                                    >
                                        <option value="none">"None"</option>
                                        {
                                            ScopeEnum::iter().map(|scope_enum| {
                                                view! {
                                                    <option value=scope_enum.to_string()>
                                                        {scope_enum.to_string()}
                                                    </option>
                                                }
                                            }).collect::<Vec<_>>()
                                        }
                                    </select>
                                    <Show when=move || matches!(scope(),Some(ScopeEnum::Course))>
                                        <div class="mt-2">
                                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                                "Course"
                                            </label>
                                            <Suspense fallback=move || view! {
                                                <div class="w-full px-4 py-3 rounded-md border border-gray-300 bg-gray-100 text-gray-500">
                                                    "Loading courses..."
                                                </div>
                                            }>
                                                <select
                                                    class="w-full px-4 py-3 rounded-md border border-gray-300 shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
                                                    prop:value=move || course_id().map(|id| id.to_string()).unwrap_or_default()
                                                    on:change=move |event| {
                                                        let value = event_target_value(&event);
                                                        if value.is_empty() {
                                                            set_course_id(None);
                                                        } else if let Ok(id) = value.parse::<i32>() {
                                                            set_course_id(Some(id));
                                                        }
                                                    }
                                                >
                                                    <option value="">"Select a Course"</option>
                                                    {move || {
                                                        courses_resource.get().unwrap_or_default().into_iter().map(|course| {
                                                            view! {
                                                                <option value=course.id.to_string()>
                                                                    {course.name.clone()}
                                                                </option>
                                                            }
                                                        }).collect::<Vec<_>>()
                                                    }}
                                                </select>
                                            </Suspense>
                                        </div>
                                    </Show>
                                </div>
                            </div>

                            <div class="form-group">
                                <label class="block text-sm font-medium text-gray-700 mb-1">
                                    "Grading/Benchmark Categories"
                                </label>
                                <div class="space-y-4 p-4 bg-gray-50 rounded-md border border-gray-200">
                                    // Display existing benchmark categories
                                    <For
                                        each=move || benchmark_categories.get()
                                        key=|(id, _, _, _)| *id
                                        children=move |(id, min_score, max_score, label): (i32, i32, i32, String)| {
                                            let id_clone = id;
                                            let (is_single_value, set_is_single_value) = create_signal(min_score == max_score);

                                            view! {
                                                <div class="flex items-center space-x-3">
                                                    <div class="flex-1 grid grid-cols-4 gap-3">
                                                        <input
                                                            type="text"
                                                            placeholder="Category (e.g. A+)"
                                                            class="px-3 py-2 rounded border border-gray-300 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                                            value=label.clone()
                                                            on:input=move |ev| {
                                                                let new_label = event_target_value(&ev);
                                                                set_benchmark_categories.update(|cats| {
                                                                    if let Some(cat) = cats.iter_mut().find(|(cid, _, _, _)| *cid == id) {
                                                                        cat.3 = new_label;
                                                                    }
                                                                });
                                                            }
                                                        />

                                                        // Toggle between single value and range
                                                        <div class="flex items-center space-x-2">
                                                            <label class="flex items-center space-x-1 text-sm">
                                                                <input
                                                                    type="checkbox"
                                                                    class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                                                    checked=is_single_value.get()
                                                                    on:change=move |ev| {
                                                                        let checked = event_target_checked(&ev);
                                                                        set_is_single_value.set(checked);

                                                                        set_benchmark_categories.update(|cats| {
                                                                            if let Some(cat) = cats.iter_mut().find(|(cid, _, _, _)| *cid == id) {
                                                                                if checked {
                                                                                    // Convert to single value - use min as the single value
                                                                                    cat.2 = cat.1; // max = min
                                                                                } else {
                                                                                    // Convert to range - ensure max >= min
                                                                                    if cat.2 <= cat.1 {
                                                                                        cat.2 = cat.1 + 10; // Set a reasonable default range
                                                                                    }
                                                                                }
                                                                            }
                                                                        });
                                                                    }
                                                                />
                                                                <span>"Single Value"</span>
                                                            </label>
                                                        </div>

                                                        // Input fields that change based on single value vs range
                                                        {move || {
                                                            if is_single_value.get() {
                                                                // Single value input
                                                                view! {
                                                                    <div class="col-span-2">
                                                                        <input
                                                                            type="number"
                                                                            placeholder="Value"
                                                                            min="0"
                                                                            class="w-full px-3 py-2 rounded border border-gray-300 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                                                            value=min_score.to_string()
                                                                            on:input=move |ev| {
                                                                                if let Ok(new_value) = event_target_value(&ev).parse::<i32>() {
                                                                                    set_benchmark_categories.update(|cats| {
                                                                                        if let Some(cat) = cats.iter_mut().find(|(cid, _, _, _)| *cid == id) {
                                                                                            cat.1 = new_value; // min
                                                                                            cat.2 = new_value; // max = min for single value
                                                                                        }
                                                                                    });
                                                                                }
                                                                            }
                                                                        />
                                                                    </div>
                                                                }.into_view()
                                                            } else {
                                                                // Range inputs
                                                                view! {
                                                                    <div class="col-span-2 flex items-center">
                                                                        <input
                                                                            type="number"
                                                                            placeholder="Min"
                                                                            min="0"
                                                                            class="w-full px-3 py-2 rounded-l border border-gray-300 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                                                            value=min_score.to_string()
                                                                            on:input=move |ev| {
                                                                                if let Ok(new_min) = event_target_value(&ev).parse::<i32>() {
                                                                                    set_benchmark_categories.update(|cats| {
                                                                                        if let Some(cat) = cats.iter_mut().find(|(cid, _, _, _)| *cid == id) {
                                                                                            cat.1 = new_min;
                                                                                            // Ensure max is at least equal to min
                                                                                            if cat.2 < new_min {
                                                                                                cat.2 = new_min;
                                                                                            }
                                                                                        }
                                                                                    });
                                                                                }
                                                                            }
                                                                        />
                                                                        <span class="px-2 py-2 bg-gray-200 border-t border-b border-gray-300">-</span>
                                                                        <input
                                                                            type="number"
                                                                            placeholder="Max"
                                                                            min=min_score.to_string()
                                                                            class="w-full px-3 py-2 rounded-r border border-gray-300 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                                                            value=max_score.to_string()
                                                                            on:input=move |ev| {
                                                                                if let Ok(new_max) = event_target_value(&ev).parse::<i32>() {
                                                                                    set_benchmark_categories.update(|cats| {
                                                                                        if let Some(cat) = cats.iter_mut().find(|(cid, _, _, _)| *cid == id) {
                                                                                            // Ensure max is at least equal to min
                                                                                            cat.2 = if new_max >= cat.1 { new_max } else { cat.1 };
                                                                                        }
                                                                                    });
                                                                                }
                                                                            }
                                                                        />
                                                                    </div>
                                                                }.into_view()
                                                            }
                                                        }}
                                                    </div>
                                                    <button
                                                        type="button"
                                                        class="p-1 text-red-600 hover:bg-red-100 rounded-full focus:outline-none"
                                                        on:click=move |_| {
                                                            set_benchmark_categories.update(|cats| {
                                                                cats.retain(|(cid, _, _, _)| *cid != id_clone);
                                                            });
                                                        }
                                                    >
                                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                                            <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
                                                        </svg>
                                                    </button>
                                                </div>
                                            }
                                        }
                                    />

                                    // Add new category button
                                    <div class="flex space-x-2">
                                        <button
                                            type="button"
                                            class="flex items-center px-4 py-2 text-sm font-medium text-blue-700 bg-blue-50 border border-blue-300 rounded-md hover:bg-blue-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                                            on:click=move |_| {
                                                set_benchmark_categories.update(|cats| {
                                                    // Generate a unique ID for the new category
                                                    let new_id = cats.iter().map(|(id, _, _, _)| *id).max().unwrap_or(0) + 1;
                                                    cats.push((new_id, 0, 10, String::new())); // Default range
                                                });
                                            }
                                        >
                                            <svg class="w-4 h-4 mr-2" viewBox="0 0 20 20" fill="currentColor">
                                                <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
                                            </svg>
                                            "Add Range Category"
                                        </button>

                                        <button
                                            type="button"
                                            class="flex items-center px-4 py-2 text-sm font-medium text-green-700 bg-green-50 border border-green-300 rounded-md hover:bg-green-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
                                            on:click=move |_| {
                                                set_benchmark_categories.update(|cats| {
                                                    // Generate a unique ID for the new category
                                                    let new_id = cats.iter().map(|(id, _, _, _)| *id).max().unwrap_or(0) + 1;
                                                    cats.push((new_id, 0, 0, String::new())); // Single value (min = max)
                                                });
                                            }
                                        >
                                            <svg class="w-4 h-4 mr-2" viewBox="0 0 20 20" fill="currentColor">
                                                <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
                                            </svg>
                                            "Add Single Value"
                                        </button>
                                    </div>

                                    // Help text
                                    <div class="text-xs text-gray-600 bg-blue-50 p-3 rounded border-l-4 border-blue-400">
                                        <p class="font-medium mb-1">Usage Examples:</p>
                                        <ul class="space-y-1">
                                            <li>"•" <strong>Range:</strong> "B" with 70-79 (students scoring 70-79 get a B)</li>
                                            <li>"•" <strong>Single Value:</strong> "Perfect" with 100 (only students scoring exactly 100 get "Perfect")</li>
                                            <li>"•" <strong>Mixed:</strong> You can have both ranges and single values in the same test</li>
                                        </ul>
                                    </div>
                                </div>
                            </div>

                            <div class="form-group">
                                <label class="block text-sm font-medium text-gray-700 mb-1">
                                    "Comments (Optional)"
                                </label>
                                <input
                                    type="text"
                                    placeholder="Add any additional comments"
                                    class="w-full px-4 py-3 rounded-md border border-gray-300 shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
                                    value=test_comments
                                    on:input=move |event| {
                                        set_test_comments(event_target_value(&event));
                                    }
                                />
                            </div>

                            <div class="form-group">
                                <label class="block text-sm font-medium text-gray-700 mb-1">
                                    "Test Instructions"
                                </label>
                                <textarea
                                    placeholder="Instructions for students taking the test"
                                    class="w-full h-64 px-4 py-3 rounded-md border border-gray-300 shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
                                    on:input=move |event| {
                                        set_test_instructions(event_target_value(&event));
                                    }
                                ></textarea>
                            </div>

                            {move || {
                                if !is_edit_mode() {
                                    view! {
                                        <div class="form-group">
                                            <div class="bg-gray-50 rounded-lg p-4 border border-gray-200">
                                                <h3 class="text-sm font-medium text-gray-700 mb-3">Test Variations</h3>
                                                <p class="text-sm text-gray-600 mb-4">
                                                    "After creating this test, you can create variations (easier, harder, practice versions) from the Variation Manager."
                                                </p>
                                                <button
                                                    type="button"
                                                    class="text-sm bg-blue-100 hover:bg-blue-200 text-blue-800 px-3 py-2 rounded-md transition-colors"
                                                    on:click=move |_| {
                                                        let navigate = leptos_router::use_navigate();
                                                        navigate("/test-variations", Default::default());
                                                    }
                                                >
                                                    "Go to Variation Manager"
                                                </button>
                                            </div>
                                        </div>
                                    }
                                } else {
                                    view! { <div></div> }
                                }
                            }}

                            <div class="pt-6">
                                <button
                                    class="px-6 py-3 bg-[#00356b] text-white rounded-md font-medium shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-all"
                                    on:click=move |_| set_selected_tab.set(1)
                                >
                                    "Continue to Questions"
                                </button>
                            </div>
                        </div>
                    }.into_any(),
                    1 => view!{
                        <div class="space-y-8">
                            <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-4">
                                <div class="flex justify-between items-center mb-4">
                                    <h3 class="text-lg font-medium text-gray-800">"Question Management"</h3>
                                    <div class="flex items-center space-x-4">
                                        <div class="text-sm text-gray-600">
                                            <span class="font-medium">
                                                {move || {
                                                    let count = questions().len();
                                                    let total = questions().iter().map(|q| q.point_value).sum::<i32>();
                                                    format!("{} questions • {} total points", count, total)
                                                }}
                                            </span>
                                        </div>
                                        <button
                                            class="flex items-center px-4 py-2 bg-[#00356b] text-white rounded-md font-medium shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-all"
                                            on:click=move |_| add_new_question(())
                                            title="Add New Question (Ctrl + +)"
                                        >
                                            <svg class="w-5 h-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                                                <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
                                            </svg>
                                            "Add Question"
                                            <span class="ml-2 text-xs opacity-75 bg-blue-600 px-2 py-1 rounded">
                                                "Ctrl + +"
                                            </span>
                                        </button>
                                    </div>
                                </div>

                                // Preset controls section
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4 p-4 bg-gray-50 rounded-lg border border-gray-200">
                                    // Point value preset
                                    <div class="flex items-center space-x-2">
                                        <label class="text-sm text-gray-600 whitespace-nowrap">"Set all points to:"</label>
                                        <select
                                            class="flex-1 px-3 py-1 border border-gray-300 rounded text-sm focus:ring-2 focus:ring-blue-500"
                                            prop:value=move || default_point_value().map(|p| p.to_string()).unwrap_or_default()
                                            on:change=move |event| {
                                                let value = event_target_value(&event);
                                                if !value.is_empty() {
                                                    if let Ok(points) = value.parse::<i32>() {
                                                        // Set the default for new questions
                                                        set_default_point_value(Some(points));

                                                        // Update existing questions
                                                        set_questions.update(|qs| {
                                                            for q in qs.iter_mut() {
                                                                q.point_value = points;
                                                            }
                                                        });
                                                        // Force re-render by updating the key
                                                        set_force_update_key.update(|k| *k += 1);
                                                    }
                                                } else {
                                                    // Clear default when "Select..." is chosen
                                                    set_default_point_value(None);
                                                }
                                            }
                                        >
                                            <option value="">"Select..."</option>
                                            <option value="1">"1 point"</option>
                                            <option value="2">"2 points"</option>
                                            <option value="3">"3 points"</option>
                                            <option value="5">"5 points"</option>
                                            <option value="10">"10 points"</option>
                                        </select>
                                        {move || {
                                            if let Some(points) = default_point_value() {
                                                view! {
                                                    <span class="text-xs text-blue-600 bg-blue-50 px-2 py-1 rounded whitespace-nowrap">
                                                        "New: " {points} " pts"
                                                    </span>
                                                }.into_view()
                                            } else {
                                                view! { <div></div> }.into_view()
                                            }
                                        }}
                                    </div>

                                    // Question type preset
                                    <div class="flex items-center space-x-2">
                                        <label class="text-sm text-gray-600 whitespace-nowrap">"Default question type:"</label>
                                        <select
                                            class="flex-1 px-3 py-1 border border-gray-300 rounded text-sm focus:ring-2 focus:ring-blue-500"
                                            prop:value=move || {
                                                default_question_type().map(|qt| {
                                                    match qt {
                                                        QuestionType::MultipleChoice => "MultipleChoice",
                                                        QuestionType::TrueFalse => "TrueFalse",
                                                        QuestionType::WeightedMultipleChoice => "WeightedMultipleChoice",
                                                        _ => "MultipleChoice"
                                                    }
                                                }).unwrap_or("")
                                            }
                                            on:change=move |event| {
                                                let value = event_target_value(&event);
                                                if !value.is_empty() {
                                                    let question_type = match value.as_str() {
                                                        "MultipleChoice" => QuestionType::MultipleChoice,
                                                        "TrueFalse" => QuestionType::TrueFalse,
                                                        "WeightedMultipleChoice" => QuestionType::WeightedMultipleChoice,
                                                        _ => QuestionType::MultipleChoice,
                                                    };
                                                    set_default_question_type(Some(question_type));
                                                } else {
                                                    set_default_question_type(None);
                                                }
                                            }
                                        >
                                            <option value="">"Select..."</option>
                                            <option value="MultipleChoice">"Multiple Choice"</option>
                                            <option value="TrueFalse">"True/False"</option>
                                            <option value="WeightedMultipleChoice">"Weighted Multiple Choice"</option>
                                        </select>
                                        {move || {
                                            if let Some(qt) = default_question_type() {
                                                let display_name = match qt {
                                                    QuestionType::MultipleChoice => "MC",
                                                    QuestionType::TrueFalse => "T/F",
                                                    QuestionType::WeightedMultipleChoice => "WMC",
                                                    _ => "MC"
                                                };
                                                view! {
                                                    <span class="text-xs text-green-600 bg-green-50 px-2 py-1 rounded whitespace-nowrap">
                                                        "New: " {display_name}
                                                    </span>
                                                }.into_view()
                                            } else {
                                                view! { <div></div> }.into_view()
                                            }
                                        }}
                                    </div>
                                </div>

                                // Help text for the preset controls
                                <div class="mt-3 text-xs text-gray-600 bg-blue-50 p-3 rounded border-l-4 border-blue-400">
                                    <p class="font-medium mb-1">Quick Setup Tips:</p>
                                    <ul class="space-y-1">
                                        <li>"•" <strong>Points:</strong> " Sets point value for all questions (existing + new)"</li>
                                        <li>"•" <strong>Default Type:</strong> " Sets question type for newly added questions"</li>
                                    </ul>
                                </div>
                            </div>

                            // Show message if no questions exist
                            <Show when=move || questions().is_empty()>
                                <div class="text-center py-12 bg-gray-50 rounded-lg border-2 border-dashed border-gray-300">
                                    <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                    </svg>
                                    <h3 class="mt-2 text-sm font-medium text-gray-900">"No questions added yet"</h3>
                                    <p class="mt-1 text-sm text-gray-500">
                                        {move || {
                                            if let Some(qt) = default_question_type() {
                                                let type_name = match qt {
                                                    QuestionType::MultipleChoice => "Multiple Choice",
                                                    QuestionType::TrueFalse => "True/False",
                                                    QuestionType::WeightedMultipleChoice => "Weighted Multiple Choice",
                                                    _ => "Multiple Choice"
                                                };
                                                format!("New questions will be {} type.", type_name)
                                            } else {
                                                "Get started by adding your first question.".to_string()
                                            }
                                        }}
                                    </p>
                                    <div class="mt-6">
                                        <button
                                            class="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-[#00356b] hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                                            on:click=move |_| add_new_question(())
                                            title="Add Question (Ctrl + +)"
                                        >
                                            <svg class="-ml-1 mr-2 h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                                <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
                                            </svg>
                                            "Add Question"
                                            <span class="ml-2 text-xs opacity-75 bg-blue-600 px-2 py-1 rounded">
                                                "Ctrl + +"
                                            </span>
                                        </button>
                                    </div>
                                </div>
                            </Show>

                            // Existing questions list
                            <div class="space-y-6">
                                <For
                                    each={move || questions.get().into_iter().enumerate().collect::<Vec<_>>()}
                                    key={move |(index, question)| (*index, question.qnumber, force_update_key.get())}
                                    children={move |(index, question): (usize, Question)| {
                                        // Create duplicate callback
                                        let duplicate_question = move |q: Question| {
                                            set_questions.update(|qs| {
                                                let mut new_q = q.clone();
                                                new_q.qnumber = (qs.len() + 1) as i32;
                                                new_q.word_problem = if new_q.word_problem.is_empty() {
                                                    "Copy of question".to_string()
                                                } else {
                                                    format!("{} (Copy)", new_q.word_problem)
                                                };
                                                // Reset the test linker to current test
                                                new_q.testlinker = test_id();
                                                qs.push(new_q);
                                            });
                                        };

                                        view! {
                                            <div class="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden">
                                                <div class="bg-gray-50 px-4 py-2 border-b border-gray-200">
                                                    <h3 class="font-medium text-gray-700">Question {question.qnumber}</h3>
                                                </div>
                                                <div class="p-4">
                                                    <BuildingQuestion
                                                        initial_question=question.clone()
                                                        on_update=Callback::new(move |updated_q| update_question(index, updated_q))
                                                        on_remove=Callback::new(move |_| remove_question(index))
                                                        on_duplicate=Some(Callback::new(duplicate_question))
                                                        should_auto_focus={
                                                            let current_question_number = question.qnumber;
                                                            create_memo(move |_| auto_focus_question() == Some(current_question_number))
                                                        }
                                                        on_focus_complete=Callback::new(move |_| clear_auto_focus(question.qnumber))
                                                    />
                                                </div>
                                            </div>
                                        }
                                    }}
                                />
                            </div>

                            // Bottom actions
                            <div class="flex justify-between items-center pt-6 border-t">
                                <button
                                    class="flex items-center px-5 py-2 bg-[#00356b] text-white rounded-md font-medium shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-all"
                                    on:click=move |_| add_new_question(())
                                    title="Add Another Question (Ctrl + +)"
                                >
                                    <svg class="w-5 h-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                                        <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
                                    </svg>
                                    "Add Another Question"
                                    <span class="ml-2 text-xs opacity-75 bg-blue-600 px-2 py-1 rounded">
                                        "Ctrl + +"
                                    </span>
                                </button>

                                <div class="flex space-x-3">
                                    <button
                                        class="px-5 py-2 bg-gray-100 text-gray-700 rounded-md font-medium hover:bg-gray-200 border border-gray-300 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500 transition-all"
                                        on:click=move |_| set_selected_tab.set(0)
                                    >
                                        "Back to Test Details"
                                    </button>
                                    <button
                                        class="px-5 py-2 bg-[#00356b] text-white rounded-md font-medium shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-all"
                                        on:click=move |_| set_selected_tab.set(2)
                                    >
                                        "Continue to Review"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                    2 => view!{
                        <div class="space-y-8">
                            <div class="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden">
                                <div class="bg-gray-50 px-4 py-3 border-b border-gray-200">
                                    <h2 class="text-lg font-medium text-gray-800">Test Summary</h2>
                                </div>
                                <div class="p-6 space-y-4">
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                        <div>
                                            <h3 class="text-sm font-medium text-gray-500">Title</h3>
                                            <p class="mt-1 text-lg text-gray-800">{test_title}</p>
                                        </div>
                                        <div>
                                            <h3 class="text-sm font-medium text-gray-500">Area</h3>
                                            <p class="mt-1 text-lg text-gray-800">{test_area}</p>
                                        </div>
                                        <div>
                                            <h3 class="text-sm font-medium text-gray-500">Questions</h3>
                                            <p class="mt-1 text-lg text-gray-800">{move || questions().len()}</p>
                                        </div>
                                        <div>
                                            <h3 class="text-sm font-medium text-gray-500">Total Points</h3>
                                            <p class="mt-1 text-lg text-gray-800">{move || {
                                                questions().iter().map(|q| q.point_value).sum::<i32>()
                                            }}</p>
                                        </div>
                                        <div>
                                            <h3 class="text-sm font-medium text-gray-500">Benchmark Categories</h3>
                                            <p class="mt-1 text-lg text-gray-800">{benchmark_summary}</p>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <div class="pt-6 flex justify-between">
                                <button
                                    class="px-5 py-2 bg-gray-100 text-gray-700 rounded-md font-medium hover:bg-gray-200 border border-gray-300 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500 transition-all"
                                    on:click=move |_| set_selected_tab.set(1)
                                >
                                    "Back to Questions"
                                </button>
                                <button
                                    class="flex items-center px-5 py-2 bg-[#00356b] text-white rounded-md font-medium shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-all disabled:bg-gray-400 disabled:cursor-not-allowed"
                                    on:click=on_submit_test_and_questions
                                    prop:disabled=is_submitting
                                >
                                    {move || if is_submitting() {
                                        view! {
                                            <>
                                                <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                                </svg>
                                                "Submitting..."
                                            </>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <>
                                                <svg class="w-5 h-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                                                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                                                </svg>
                                                "Submit Test & Questions"
                                            </>
                                        }.into_view()
                                    }}
                                </button>
                            </div>
                        </div>
                    }.into_any(),
                    _ => view!{<p>This is the backup tab</p>}.into_any(),
                }}
            </div>
        </main>
    }
}
