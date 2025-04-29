use crate::app::components::header::Header;
use crate::app::components::question_builder::BuildingQuestion;
use crate::app::models::student::GradeEnum;
use crate::app::models::test::BenchmarkCategory;
use crate::app::models::test::{CreateNewTestRequest, TestType, UpdateTestRequest};
use crate::app::models::{CreateNewQuestionRequest, Question, QuestionType};
use crate::app::server_functions::questions::{add_question, delete_questions, get_questions};
use crate::app::server_functions::tests::{add_test, get_test, score_overrider, update_test};
use leptos::prelude::*;
use leptos::*;
use leptos_router::*;
use std::str::FromStr;
use strum::IntoEnumIterator;

#[component]
pub fn TestBuilder() -> impl IntoView {
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

    let (selected_tab, set_selected_tab) = create_signal(0);
    let (test_title, set_test_title) = create_signal(String::new());
    let (test_instructions, set_test_instructions) = create_signal(String::new());
    let (test_area, set_test_area) = create_signal(String::new());
    let (school_year, set_school_year) = create_signal(String::new());
    let (grade_level, set_grade_level) = create_signal(String::new());
    let (benchmark_categories, set_benchmark_categories) =
        create_signal::<Vec<(i32, i32, i32, String)>>(Vec::new());
    let (test_variant, set_test_variant) = create_signal(0);
    let (test_comments, set_test_comments) = create_signal(String::new());
    let (test_id, set_test_id) = create_signal(String::new());
    let (error_message, set_error_message) = create_signal(String::new());
    let (show_error, set_show_error) = create_signal(false);
    let (is_submitting, set_is_submitting) = create_signal(false);

    let (questions, set_questions) = create_signal(Vec::<Question>::new());

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
        set_questions.update(|qs| {
            let new_question = Question::new(
                String::new(),
                0,
                QuestionType::Selection,
                Vec::new(),
                String::new(),
                (qs.len() + 1) as i32, // Use 1-based indexing for question numbers
                test_id(),
            );
            qs.push(new_question);
        });
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

    // Effect to load test data when a test_id is available
    create_effect(move |_| {
        if let Some(Some(test)) = test_resource.get() {
            set_is_edit_mode(true);
            set_test_id(test.test_id.clone());
            set_test_title(test.name.clone());
            set_test_area(test.testarea.clone().to_string());
            set_school_year(test.school_year.clone().unwrap_or_default());
            set_test_comments(test.comments.clone());
            set_test_variant(test.test_variant);

            // Convert BenchmarkCategory to our internal tuple representation
            let categories = test.benchmark_categories.clone().unwrap_or_default();
            let tuple_categories = categories
                .iter()
                .enumerate()
                .map(|(idx, cat)| (idx as i32, cat.min, cat.max, cat.label.clone()))
                .collect::<Vec<_>>();
            set_benchmark_categories(tuple_categories);

            set_grade_level(
                test.grade_level
                    .as_ref()
                    .map_or("default".to_string(), |grade| grade.to_string()),
            );
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
                CreateNewQuestionRequest::new(
                    q.word_problem.clone(),
                    q.point_value,
                    q.question_type.clone(),
                    q.options.clone(),
                    q.correct_answer.clone(),
                    q.qnumber,
                    q.testlinker.clone(), // Use existing test_id for edit mode
                )
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

        let convert_grade_to_enum = match GradeEnum::from_str(&grade_level()) {
            Ok(grade_enum) => grade_enum,
            Err(_) => {
                set_error_message(format!(
                    "Grade Enum was not converted correctly: {}",
                    grade_level()
                ));
                set_show_error(true);
                set_is_submitting(false);
                return;
            }
        };

        // Convert our tuple representation back to BenchmarkCategory
        let converted_cats = if benchmark_categories().is_empty() {
            None
        } else {
            Some(
                benchmark_categories()
                    .iter()
                    .map(|(_, min, max, label)| BenchmarkCategory {
                        min: *min,
                        max: *max,
                        label: label.clone(),
                    })
                    .collect::<Vec<BenchmarkCategory>>(),
            )
        };

        // Create the test request, whether for new or update
        let add_test_request = CreateNewTestRequest::new(
            test_title(),
            total_points,
            test_comments(),
            test_type.clone(),
            Some(school_year()),
            converted_cats.clone(),
            test_variant(),
            Some(convert_grade_to_enum.clone()),
        );

        let converted_clone = converted_cats.clone();
        spawn_local(async move {
            let current_test_id = test_id();
            let is_editing = is_edit_mode();

            // If we're editing, update the test rather than creating a new one
            let new_test_id = if is_editing && !current_test_id.is_empty() {
                let update_test_request = UpdateTestRequest::new(
                    test_title(),
                    total_points,
                    test_comments(),
                    test_type,
                    Some(school_year()),
                    converted_clone,
                    test_variant(),
                    Some(convert_grade_to_enum),
                    test_id(),
                );
                // For now, we'll assume we're just keeping the same test_id
                log::info!("Updating test with ID: {}", current_test_id);
                // Implement update_test function as needed
                match update_test(update_test_request).await {
                    Ok(updated) => {
                        let new_id = updated.test_id.clone();
                    }
                    Err(e) => {
                        log::error!("Error updating test: {:?}", e);
                        set_show_error(true);
                        set_error_message(format!("Failed to update test: {}", e));
                        return;
                    }
                }
                current_test_id
            } else {
                // Create a new test
                match add_test(add_test_request).await {
                    Ok(added_test) => {
                        let new_id = added_test.test_id.clone();
                        set_test_id(new_id.clone());
                        log::info!("Created test with ID: {}", new_id);
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

            // Navigate to the test list page only after all questions are processed
            let navigate = leptos_router::use_navigate();
            navigate("/dashboard", Default::default());
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
                                        class="w-full px-4 py-3 rounded-md border border-gray-300 shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
                                        prop:value=grade_level
                                        on:input=move |event| {
                                            set_grade_level(event_target_value(&event));
                                        }
                                    >
                                        <option value="">Select Grade</option>
                                        {GradeEnum::iter().map(|grade| view! {
                                            <option value=format!("{}", grade)>
                                                {format!("{}", grade)}
                                            </option>
                                        }).collect::<Vec<_>>()}
                                    </select>
                                </div>

                                <div class="form-group">
                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                        "Variant of Test"
                                    </label>
                                    <input
                                        type="number"
                                        class="w-full px-4 py-3 rounded-md border border-gray-300 shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
                                        value=move || test_variant().to_string()
                                        on:input=move |event| {
                                            if let Ok(value) = event_target_value(&event).parse::<i32>() {
                                                set_test_variant(value);
                                            }
                                        }
                                    />
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
                                            view! {
                                                <div class="flex items-center space-x-3">
                                                    <div class="flex-1 grid grid-cols-3 gap-3">
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
                                                        <div class="flex items-center">
                                                            <input
                                                                type="number"
                                                                placeholder="Min"
                                                                class="w-full px-3 py-2 rounded-l border border-gray-300 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                                                value=min_score.to_string()
                                                                on:input=move |ev| {
                                                                    if let Ok(new_min) = event_target_value(&ev).parse::<i32>() {
                                                                        set_benchmark_categories.update(|cats| {
                                                                            if let Some(cat) = cats.iter_mut().find(|(cid, _, _, _)| *cid == id) {
                                                                                cat.1 = new_min;
                                                                            }
                                                                        });
                                                                    }
                                                                }
                                                            />
                                                            <span class="px-2 py-2 bg-gray-200 border-t border-b border-gray-300">-</span>
                                                            <input
                                                                type="number"
                                                                placeholder="Max"
                                                                class="w-full px-3 py-2 rounded-r border border-gray-300 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                                                value=max_score.to_string()
                                                                on:input=move |ev| {
                                                                    if let Ok(new_max) = event_target_value(&ev).parse::<i32>() {
                                                                        set_benchmark_categories.update(|cats| {
                                                                            if let Some(cat) = cats.iter_mut().find(|(cid, _, _, _)| *cid == id) {
                                                                                cat.2 = new_max;
                                                                            }
                                                                        });
                                                                    }
                                                                }
                                                            />
                                                        </div>
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
                                    <button
                                        type="button"
                                        class="flex items-center px-4 py-2 text-sm font-medium text-blue-700 bg-blue-50 border border-blue-300 rounded-md hover:bg-blue-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                                        on:click=move |_| {
                                            set_benchmark_categories.update(|cats| {
                                                // Generate a unique ID for the new category
                                                let new_id = cats.iter().map(|(id, _, _, _)| *id).max().unwrap_or(0) + 1;
                                                cats.push((new_id, 0, 0, String::new()));
                                            });
                                        }
                                    >
                                        <svg class="w-4 h-4 mr-2" viewBox="0 0 20 20" fill="currentColor">
                                            <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
                                        </svg>
                                        "Add Benchmark Category"
                                    </button>
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
                            <div class="flex justify-end">
                                <button
                                    class="flex items-center px-5 py-2 bg-[#00356b] text-white rounded-md font-medium shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-all"
                                    on:click=add_new_question
                                >
                                    <svg class="w-5 h-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                                        <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
                                    </svg>
                                    "Add New Question"
                                </button>
                            </div>

                            <div class="space-y-8">
                                <For
                                    each=move || questions.get()
                                    key=|question| question.qnumber
                                    children=move |question: Question| {
                                        let index= questions.get().iter().position(|q| q.qnumber == question.qnumber).unwrap_or(0);
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
                                                    />
                                                </div>
                                            </div>
                                        }
                                    }
                                />
                            </div>

                            <div class="flex justify-between pt-6">
                                <button
                                    class="flex items-center px-5 py-2 bg-[#00356b] text-white rounded-md font-medium shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-all"
                                    on:click=add_new_question
                                >
                                    <svg class="w-5 h-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                                        <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
                                    </svg>
                                    "Add New Question"
                                </button>
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
#[component]
fn TinyMCEEditor() -> impl IntoView {
    view! {
        <html lang="en">
        <head>
            <script
                type="text/javascript"
                src="/static/tinymce/tinymce.min.js"
                referrerpolicy="origin">
            </script>
            <script type="text/javascript">
            tinymce.init({
                selector: "#myTextarea",
                width: 1200,
                height: 500,
                plugins: [
                    "advlist", "autosave", "autolink", "link", "image", "lists", "charmap", "preview", "anchor", "pagebreak",
                    "wordcount", "visualblocks", "visualchars", "insertdatetime",
                    "media", "table", "save"
                ],
                toolbar: "undo redo | styles | bold italic | alignleft aligncenter alignright alignjustify | " +
                    "bullist numlist outdent indent | link image | " +
                    "forecolor backcolor save",
                autosave_ask_before_unload: true,
                menubar: "file edit view insert format tools table",
                autosave_interval: "10s",
                autosave_restore_when_empty: true,
                license_key: "gpl",
            });
            </script>
        </head>

        <body>
            <textarea id="myTextarea" placeholder="Please Write Your Instructions Here" class="border-[#00356b] border"></textarea>
        </body>
        </html>
    }
}
