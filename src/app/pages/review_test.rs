use crate::app::components::data_processing::test_pie_chart::PieChart;
use crate::app::components::Header;
use crate::app::models::question::QuestionType;
use crate::app::models::score::Score;
use crate::app::models::student::{ESLEnum, GenderEnum, GradeEnum, Student};
use crate::app::models::test::Test;
use crate::app::server_functions::questions::get_questions;
use crate::app::server_functions::scores::get_score;
use crate::app::server_functions::students::get_student;
use crate::app::server_functions::tests::get_test;
use leptos::*;
use leptos_router::*;

#[derive(Clone, PartialEq)]
enum ReviewTab {
    Detailed,
    Grid,
}

#[component]
pub fn ReviewTest() -> impl IntoView {
    let params = use_params_map();
    let test_id = move || params().get("test_id").cloned().unwrap_or_default();
    let student_id = move || {
        params()
            .get("student_id")
            .cloned()
            .unwrap_or_default()
            .parse::<i32>()
            .unwrap()
    };
    let test_variant = move || {
        params()
            .get("test_variant")
            .cloned()
            .unwrap_or_default()
            .parse::<i32>()
            .unwrap()
    };
    let attempt = move || {
        params()
            .get("attempt")
            .cloned()
            .unwrap_or_default()
            .parse::<i32>()
            .unwrap()
    };

    // Active tab state
    let (active_tab, set_active_tab) = create_signal(ReviewTab::Detailed);

    //Create resources for fetching score, test, and questions
    let score = create_resource(
        move || (student_id(), test_id(), test_variant(), attempt()),
        |(student_id, test_id, test_variant, attempt)| async move {
            match get_score(student_id, test_id, test_variant, attempt).await {
                Ok(score) => Ok(score),
                Err(e) => {
                    log::error!("Failed to load score: {}", e);
                    Err(ServerFnError::new("Failed to load score"))
                }
            }
        },
    );
    let test = create_resource(
        move || test_id(),
        |test_id| async move {
            match get_test(test_id).await {
                Ok(test) => Ok(test),
                Err(e) => {
                    log::error!("Failed to load test: {}", e);
                    Err(ServerFnError::new("Failed to load test"))
                }
            }
        },
    );
    let questions = create_resource(
        move || test_id(),
        |test_id| async move {
            match get_questions(test_id).await {
                Ok(questions) => questions,
                Err(e) => {
                    log::error!("Failed to fetch questions: {}", e);
                    Vec::new()
                }
            }
        },
    );
    let student = create_resource(
        move || student_id(),
        |student_id| async move {
            match get_student(student_id).await {
                Ok(student) => student,
                Err(e) => {
                    log::error!("Failed to fetch student: {}", e);
                    Student::new(
                        Some(String::from("Unknown")),
                        Some(String::from("Student")),
                        String::new(),
                        GenderEnum::Male,
                        chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
                        0,
                        ESLEnum::NotApplicable,
                        crate::app::models::student::GradeEnum::First,
                        String::new(),
                        false,
                        false,
                        false,
                        false,
                        false,
                        None,
                        false,
                        String::new(),
                        Some(0),
                    )
                }
            }
        },
    );

    // Signal to determine if all questions are true/false
    let all_true_false = create_memo(move |_| {
        questions
            .get()
            .map(|qs| {
                !qs.is_empty()
                    && qs
                        .iter()
                        .all(|q| q.question_type == QuestionType::TrueFalse)
            })
            .unwrap_or(false)
    });

    view! {
        <Header />
        <div class="flex h-full">
            <main class="flex-1 px-8 py-6 bg-gray-50 min-h-screen">
                // Header section with test info
                <div class="mb-8">
                    {move || {
                        test.get().map(|test_result| {
                            match test_result {
                                Ok(test) => {
                                    view! {
                                        <div class="mt-2 flex justify-between items-center">
                                            <div>
                                                <h1 class="text-3xl font-bold text-gray-900">{&test.name}</h1>
                                                {move || {
                                                    student.get().map(|student_data| {
                                                        let firstname = match &student_data.firstname {
                                                            Some(name) => name.clone(),
                                                            None => "Unknown".to_string(),
                                                        };
                                                        let lastname = match &student_data.lastname {
                                                            Some(name) => name.clone(),
                                                            None => "Student".to_string(),
                                                        };
                                                        view! {
                                                            <p class="text-gray-600">
                                                                Student: {firstname}{" "}{lastname}
                                                            </p>
                                                        }
                                                    })
                                                }}
                                                <p class="text-gray-600">Test Type: {test.testarea.to_string()}</p>
                                                {match &test.school_year {
                                                    Some(year) => view! { <p class="text-gray-600">School Year: {year}</p> },
                                                    None => view! { <p class="text-gray-600">School Year: Not specified</p> }
                                                }}
                                            </div>
                                            <div class="bg-white shadow rounded-lg p-4 text-center">
                                                <p class="text-sm text-gray-500">Test Variant</p>
                                                <p class="text-2xl font-semibold">{test.test_variant}</p>
                                            </div>
                                        </div>
                                    }
                                },
                                Err(_) => {
                                    view! { <div class="bg-red-50 p-4 rounded-md">
                                        <p class="text-red-700">Failed to load test information.</p>
                                    </div> }
                                }
                            }
                        })
                    }}
                </div>

                // Score summary card
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
                    <div class="bg-white shadow rounded-lg p-6">
                        {move || {
                            score.get().map(|score_result| {
                                match score_result {
                                    Ok(score) => {
                                        // Calculate total correct answers
                                        let total_correct = create_memo(move |_| {
                                            if let Some(qs) = questions.get() {
                                                qs.iter().enumerate().filter(|(i, q)| {
                                                    *i < score.test_scores.len() && score.test_scores[*i] == q.point_value
                                                }).count() as i32
                                            } else {
                                                0
                                            }
                                        });

                                        let total_possible = move || {
                                            questions.get().map(|q| q.len()).unwrap_or(0) as i32
                                        };

                                        let percentage = move || {
                                            let total = total_possible();
                                            if total > 0 {
                                                (total_correct.get() as f32 / total as f32) * 100.0
                                            } else {
                                                0.0
                                            }
                                        };

                                        view! {
                                            <div class="grid grid-cols-3 gap-6">
                                                <div class="border-r pr-6">
                                                    <h3 class="text-sm font-medium text-gray-500">Student ID</h3>
                                                    <p class="text-2xl font-semibold">{score.student_id}</p>
                                                </div>
                                                <div class="border-r px-6">
                                                    <h3 class="text-sm font-medium text-gray-500">Date Administered</h3>
                                                    <p class="text-2xl font-semibold">{score.date_administered.format("%b %d, %Y").to_string()}</p>
                                                </div>
                                                <div class="pl-6">
                                                    <h3 class="text-sm font-medium text-gray-500">Evaluator</h3>
                                                    <p class="text-2xl font-semibold">{&score.evaluator}</p>
                                                </div>
                                            </div>
                                            <div class="mt-6 pt-6 border-t">
                                                <div class="flex justify-between items-end">
                                                    <div>
                                                        <h3 class="text-sm font-medium text-gray-500">Total Score</h3>
                                                        <div class="flex items-baseline">
                                                            <p class="text-3xl font-bold text-indigo-600">{move || total_correct.get()}</p>
                                                            <p class="ml-2 text-lg text-gray-500">/ {total_possible}</p>
                                                        </div>
                                                    </div>
                                                    <div class="bg-indigo-50 px-4 py-2 rounded-md">
                                                        <p class="text-indigo-700 text-xl font-semibold">{move || format!("{:.1}%", percentage())}</p>
                                                    </div>
                                                </div>
                                                <div class="mt-4 w-full bg-gray-200 rounded-full h-2.5">
                                                    <div class="bg-indigo-600 h-2.5 rounded-full" style={move || format!("width: {}%", percentage())}></div>
                                                </div>
                                            </div>
                                        }.into_view()
                                    },
                                    Err(_) => {
                                        view! {
                                            <div class="bg-red-50 p-4 rounded-md">
                                                <p class="text-red-700">Failed to load score information.</p>
                                            </div>
                                        }.into_view()
                                    }
                                }
                            })
                        }}
                    </div>

                    <div>
                        {move || {
                            match (score.get(), test.get()) {
                                (Some(Ok(score_data)), Some(Ok(test_data))) => {
                                    let total_possible = test_data.score;
                                    view! {
                                        <PieChart
                                            score=score_data.clone()
                                            test=test_data.clone()
                                        />
                                    }.into_view()
                                },
                                _ => {
                                    view! {
                                        <div class="bg-white shadow rounded-lg p-6 h-64 flex items-center justify-center">
                                            <p class="text-gray-500">Loading chart...</p>
                                        </div>
                                    }.into_view()
                                }
                            }
                        }}
                    </div>
                </div>

                // Tab selection - Only show Grid tab if all questions are true/false
                <div class="border-b border-gray-200 mb-6">
                    <nav class="-mb-px flex space-x-6">
                        <button
                            class="py-4 px-1 border-b-2 font-medium text-sm"
                            class:border-indigo-500=move || active_tab.get() == ReviewTab::Detailed
                            class:text-indigo-600=move || active_tab.get() == ReviewTab::Detailed
                            class:border-transparent=move || active_tab.get() != ReviewTab::Detailed
                            class:text-gray-500=move || active_tab.get() != ReviewTab::Detailed
                            class:hover:text-gray-700=move || active_tab.get() != ReviewTab::Detailed
                            class:hover:border-gray-300=move || active_tab.get() != ReviewTab::Detailed
                            on:click=move |_| set_active_tab.set(ReviewTab::Detailed)
                        >
                            "Detailed View"
                        </button>

                        // Only show Grid tab if all questions are true/false
                        {move || {
                            if all_true_false.get() {
                                view! {
                                    <button
                                        class="py-4 px-1 border-b-2 font-medium text-sm"
                                        class:border-indigo-500=move || active_tab.get() == ReviewTab::Grid
                                        class:text-indigo-600=move || active_tab.get() == ReviewTab::Grid
                                        class:border-transparent=move || active_tab.get() != ReviewTab::Grid
                                        class:text-gray-500=move || active_tab.get() != ReviewTab::Grid
                                        class:hover:text-gray-700=move || active_tab.get() != ReviewTab::Grid
                                        class:hover:border-gray-300=move || active_tab.get() != ReviewTab::Grid
                                        on:click=move |_| set_active_tab.set(ReviewTab::Grid)
                                    >
                                        "Grid View"
                                    </button>
                                }.into_view()
                            } else {
                                view! {}.into_view()
                            }
                        }}
                    </nav>
                </div>

                // Content based on selected tab
                {move || {
                    match active_tab.get() {
                        ReviewTab::Detailed => view! {
                            <DetailedView
                                questions=questions.clone()
                                score=score.clone()
                            />
                        }.into_view(),
                        ReviewTab::Grid => view! {
                            <GridView
                                questions=questions.clone()
                                score=score.clone()
                            />
                        }.into_view()
                    }
                }}
            </main>
        </div>
    }
}

// Separate component for the detailed view (original content)
#[component]
fn DetailedView(
    questions: Resource<String, Vec<crate::app::models::question::Question>>,
    score: Resource<(i32, String, i32, i32), Result<Score, ServerFnError>>,
) -> impl IntoView {
    view! {
        <div class="bg-white shadow rounded-lg p-6">
            <h2 class="text-xl font-semibold mb-4">Question Breakdown</h2>

            {move || {
                score.get().map(|score_result| {
                    questions.get().map(|questions_data| {
                        match score_result {
                            Ok(score) => {
                                view! {
                                    <div class="space-y-6">
                                        {questions_data.iter().enumerate().map(|(i, question)| {
                                            let student_answer = if i < score.test_scores.len() {
                                                score.test_scores[i]
                                            } else {
                                                0
                                            };

                                            let student_comment = if i < score.comments.len() {
                                                score.comments[i].clone()
                                            } else {
                                                String::new()
                                            };

                                            // Fixed: A student answer is correct if it equals the point value
                                            let is_correct = student_answer == question.point_value;

                                            view! {
                                                <div class="border rounded-lg overflow-hidden">
                                                    <div class={"flex items-center justify-between p-4 border-b ".to_string() + if is_correct { "bg-green-50" } else { "bg-red-50" }}>
                                                        <div class="flex items-center">
                                                            <span class={"flex-shrink-0 h-8 w-8 rounded-full flex items-center justify-center ".to_string() +
                                                                if is_correct { "bg-green-100 text-green-600" } else { "bg-red-100 text-red-600" }}>
                                                                {if is_correct { "✓" } else { "✗" }}
                                                            </span>
                                                            <span class="ml-3 font-medium">Question {question.qnumber}: {&question.word_problem}</span>
                                                        </div>
                                                        <div class="flex items-baseline">
                                                            <span class={"font-semibold ".to_string() + if is_correct { "text-green-600" } else { "text-red-600" }}>
                                                                {student_answer}
                                                            </span>
                                                            <span class="text-gray-500 ml-1">/ {question.point_value}</span>
                                                        </div>
                                                    </div>

                                                    <div class="p-4 bg-white">
                                                        <div class="grid grid-cols-2 gap-4">
                                                            <div>
                                                                <h4 class="text-sm font-medium text-gray-500 mb-2">Question Type</h4>
                                                                <p>{question.question_type.to_string()}</p>
                                                            </div>
                                                            <div>
                                                                <h4 class="text-sm font-medium text-gray-500 mb-2">Correct Answer</h4>
                                                                <p>{&question.correct_answer}</p>
                                                            </div>
                                                        </div>

                                                        {if !question.options.is_empty() {
                                                            view! {
                                                                <div class="mt-4">
                                                                    <h4 class="text-sm font-medium text-gray-500 mb-2">Options</h4>
                                                                    <div class="grid grid-cols-2 gap-2">
                                                                        {question.options.iter().enumerate().map(|(j, option)| {
                                                                            let is_correct_option = *option == question.correct_answer;
                                                                            view! {
                                                                                <div class={"p-2 rounded ".to_string() +
                                                                                    if is_correct_option { "bg-green-100 border border-green-200" }
                                                                                    else { "bg-gray-50 border border-gray-200" }}>
                                                                                    {option}
                                                                                </div>
                                                                            }
                                                                        }).collect::<Vec<_>>()}
                                                                    </div>
                                                                </div>
                                                            }
                                                        } else {
                                                            view! { <div></div> }
                                                        }}

                                                        {if !student_comment.is_empty() {
                                                            view! {
                                                                <div class="mt-4">
                                                                    <h4 class="text-sm font-medium text-gray-500 mb-2">Evaluator Comment</h4>
                                                                    <p class="bg-gray-50 p-3 rounded border border-gray-200">{student_comment}</p>
                                                                </div>
                                                            }
                                                        } else {
                                                            view! { <div></div> }
                                                        }}
                                                    </div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }
                            },
                            Err(_) => {
                                view! { <div class="bg-red-50 p-4 rounded-md">
                                    <p class="text-red-700">Failed to load score data.</p>
                                </div> }
                            }
                        }
                    })
                })
            }}
        </div>
    }
}

// New component for grid view
#[component]
fn GridView(
    questions: Resource<String, Vec<crate::app::models::question::Question>>,
    score: Resource<(i32, String, i32, i32), Result<Score, ServerFnError>>,
) -> impl IntoView {
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
                "text-xl" // Extremely small text for very many questions
            } else if count > 64 {
                "text-2xl" // Very small text for many questions
            } else if count > 36 {
                "text-2xl" // Small text for lots of questions
            } else if count > 16 {
                "text-3xl" // Normal text for medium count
            } else if count > 9 {
                "text-3xl" // Larger text for few questions
            } else {
                "text-4xl" // Very large text for very few questions
            }
        } else {
            "text-lg" // Default size
        }
    });

    // Currently selected question for details
    let (selected_question, set_selected_question) = create_signal(None::<i32>);

    view! {
        <div class="bg-white shadow rounded-lg p-6">
            <h2 class="text-xl font-semibold mb-4">Grid View</h2>

            <div class="flex flex-col h-full">
                <div class="flex-grow flex items-center justify-center p-4 overflow-hidden">
                    <div class="aspect-square w-full max-w-4xl bg-white rounded shadow-inner">
                        {move || {
                            let (rows, cols) = grid_dimensions();

                            view! {
                                <div
                                    class="grid h-full w-full"
                                    style=move || format!(
                                        "grid-template-columns: repeat({}, minmax(0, 1fr)); grid-template-rows: repeat({}, minmax(0, 1fr)); gap: 1px; background-color: #e5e7eb;",
                                        cols, rows
                                    )
                                >
                                    {move || {
                                        match (questions.get(), score.get()) {
                                            (Some(questions_data), Some(Ok(score_data))) => {
                                                let mut sorted_questions = questions_data.clone();
                                                sorted_questions.sort_by_key(|q| q.qnumber);

                                                sorted_questions.iter().enumerate().map(|(i, question)| {
                                                    let qnumber = question.qnumber;
                                                    let display_text = question.word_problem.clone();

                                                    let student_answer = if i < score_data.test_scores.len() {
                                                        score_data.test_scores[i]
                                                    } else {
                                                        0
                                                    };

                                                    // Fixed: A student answer is correct if it equals the point value
                                                    let is_correct = student_answer == question.point_value;
                                                    let is_selected = create_memo(move |_| {
                                                        selected_question.get() == Some(qnumber)
                                                    });

                                                    let current_cell_size = cell_size_class();

                                                    view! {
                                                        <div
                                                            class="flex items-center justify-center cursor-pointer transition-all relative"
                                                            class:bg-green-100=move || is_correct
                                                            class:bg-red-100=move || !is_correct
                                                            class:ring-2=move || is_selected()
                                                            class:ring-blue-500=move || is_selected()
                                                            on:click=move |_| {
                                                                if selected_question.get() == Some(qnumber) {
                                                                    set_selected_question.set(None);
                                                                } else {
                                                                    set_selected_question.set(Some(qnumber));
                                                                }
                                                            }
                                                        >
                                                            <span class=format!("select-none font-bold {} px-0.5 py-0.5 text-center", current_cell_size)>{display_text}</span>
                                                            {move || if !is_correct {
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
                                            },
                                            _ => view! { <div>Loading...</div> }.into_view()
                                        }
                                    }}
                                </div>
                            }
                        }}
                    </div>
                </div>

                // Selected question details
                <div class="mt-6 border-t pt-6">
                    {move || {
                        match (questions.get(), score.get(), selected_question.get()) {
                            (Some(questions_data), Some(Ok(score_data)), Some(qnumber)) => {
                                // Find the relevant question and score data
                                let question = questions_data.iter().find(|q| q.qnumber == qnumber);

                                match question {
                                    Some(q) => {
                                        // Find the index of this question to map to score
                                        let index = questions_data.iter().position(|quest| quest.qnumber == qnumber).unwrap_or(0);

                                        let student_answer = if index < score_data.test_scores.len() {
                                            score_data.test_scores[index]
                                        } else {
                                            0
                                        };

                                        let student_comment = if index < score_data.comments.len() {
                                            score_data.comments[index].clone()
                                        } else {
                                            String::new()
                                        };

                                        // Fixed: A student answer is correct if it equals the point value
                                        let is_correct = student_answer == q.point_value;

                                        view! {
                                            <div class="bg-gray-100 rounded-lg p-4">
                                                <h3 class="text-lg font-semibold mb-2">Question {q.qnumber}: {&q.word_problem}</h3>
                                                <div class="grid grid-cols-2 gap-4 mb-3">
                                                    <div>
                                                        <h4 class="text-sm font-medium text-gray-500 mb-1">Correct Answer</h4>
                                                        <p>{&q.correct_answer}</p>
                                                    </div>
                                                    <div>
                                                        <h4 class="text-sm font-medium text-gray-500 mb-1">"Student's Result"</h4>
                                                        <p class=if is_correct { "text-green-600 font-semibold" } else { "text-red-600 font-semibold" }>
                                                            {if is_correct { "Correct" } else { "Incorrect" }}
                                                        </p>
                                                    </div>
                                                </div>

                                                {if !student_comment.is_empty() {
                                                    view! {
                                                        <div>
                                                            <h4 class="text-sm font-medium text-gray-500 mb-1">Evaluator Comment</h4>
                                                            <p class="bg-white p-3 rounded border">{student_comment}</p>
                                                        </div>
                                                    }.into_view()
                                                } else {
                                                    view! {}.into_view()
                                                }}
                                            </div>
                                        }.into_view()
                                    },
                                    None => view! {
                                        <div class="text-gray-500 italic">Question not found.</div>
                                    }.into_view()
                                }
                            },
                            (_, _, Some(_)) => view! {
                                <div class="text-gray-500 italic">Loading question details...</div>
                            }.into_view(),
                            _ => view! {
                                <div class="text-gray-500 italic">Click on any grid cell to view question details.</div>
                            }.into_view()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
