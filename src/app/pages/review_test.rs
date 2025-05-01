use crate::app::components::Header;
use crate::app::models::score::Score;
use crate::app::models::student::{ESLEnum, GenderEnum, GradeEnum, Student};
use crate::app::models::test::Test;
use crate::app::server_functions::questions::get_questions;
use crate::app::server_functions::scores::get_score;
use crate::app::server_functions::students::get_student;
use crate::app::server_functions::tests::get_test;
use leptos::*;
use leptos_router::*;

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

    //Create resources for fetching score, test, and questions
    let score = create_resource(
        move || (student_id(), test_id(), test_variant()),
        |(student_id, test_id, test_variant)| async move {
            match get_score(student_id, test_id, test_variant).await {
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
                        String::from("Unknown"),
                        String::from("Student"),
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
                        0,
                    )
                }
            }
        },
    );

    view! {
        <Header />
        <div class="flex h-full">
            <main class="flex-1 mt-16 ml-20 px-8 py-6 bg-gray-50 min-h-screen">
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
                                                        view! {
                                                            <p class="text-gray-600">
                                                                Student: {&student_data.firstname}{" "}{&student_data.lastname}
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
                <div class="bg-white shadow rounded-lg p-6 mb-8">
                    {move || {
                        score.get().map(|score_result| {
                            match score_result {
                                Ok(score) => {
                                    let total_score: i32 = score.test_scores.iter().sum();
                                    let total_possible = move || {
                                        questions.get().map(|q| {
                                            q.iter().map(|question| question.point_value).sum::<i32>()
                                        }).unwrap_or(0)
                                    };

                                    let percentage = move || {
                                        let total = total_possible();
                                        if total > 0 {
                                            (total_score as f32 / total as f32) * 100.0
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
                                                        <p class="text-3xl font-bold text-indigo-600">{total_score}</p>
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

                // Question-by-question breakdown
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
            </main>
        </div>
    }
}
