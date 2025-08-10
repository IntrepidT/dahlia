use crate::app::components::dashboard::color_utils::ColorUtils; // Import your new module
use crate::app::components::enhanced_login_form::{
    use_student_mapping_service, DeAnonymizedStudent, StudentMappingService,
};
use crate::app::middleware::global_settings::use_settings;
use crate::app::models::score::{DeleteScoreRequest, Score};
use crate::app::models::student::Student;
use crate::app::models::test::{BenchmarkCategory, Test};
use crate::app::server_functions::scores::{delete_score, get_scores};
use crate::app::server_functions::students::get_students;
use crate::app::server_functions::tests::get_tests;
use chrono::DateTime;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use std::rc::Rc;

#[component]
pub fn ScoresLedger() -> impl IntoView {
    let navigate = use_navigate();

    // Get global settings for anonymization
    let (settings, _) = use_settings();
    let anonymization_enabled = move || settings.get().student_protections;

    // Get the student mapping service
    let (student_mapping_service, _) = use_student_mapping_service();

    // Create resource for fetching scores from the database
    let scores_resource = LocalResource::new(|| async {
        match get_scores().await {
            Ok(mut scores) => {
                if scores.len() > 4 {
                    scores.truncate(4);
                    scores.reverse();
                }
                Ok(scores)
            }
            Err(e) => {
                log::error!("Failed to load scores: {}", e);
                Err(ServerFnError::new("Failed to load scores"))
            }
        }
    });

    // Create students resource
    let students_resource = LocalResource::new(|| async {
        match get_students().await {
            Ok(students) => Some(students),
            Err(e) => {
                log::error!("Failed to load students: {}", e);
                None
            }
        }
    });

    // Create enhanced student data with de-anonymization info
    let enhanced_students = Memo::new(move |_| {
        let students_data = students_resource.get().unwrap_or(None).unwrap_or_default();

        if anonymization_enabled() {
            students_data
                .into_iter()
                .map(|student| {
                    let de_anon = DeAnonymizedStudent::from_student_with_mapping(
                        &student,
                        student_mapping_service.get().as_ref(),
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

    let tests_resource = LocalResource::new(|| async {
        match get_tests().await {
            Ok(mut tests) => Ok(tests),
            Err(e) => {
                log::error!("Failed to load tests: {}", e);
                Err(ServerFnError::new("Failed to load tests"))
            }
        }
    });

    let (expanded_view, set_expanded_view) = signal(false);

    let toggle_expanded_view = move |_| {
        set_expanded_view.update(|val| *val = !*val);
    };

    view! {
        <div class={move || {
            if expanded_view() {
                "fixed inset-0 z-50 bg-[#F9F9F8] flex flex-col p-5"
            } else {
                "w-full"
            }
        }}>
            <div class="flex items-center justify-between mb-2 p-2">
                <h2 class="text-xl font-bold">Recent Scores</h2>
                <div>
                    <button
                        class="text-indigo-600 hover:text-indigo-800 text-sm font-medium"
                        on:click=toggle_expanded_view
                    >
                        {move || if expanded_view() {"Collapse"} else {"View all"}}
                    </button>
                </div>
            </div>

            <div class={move ||{
                let base_classes = "bg-[#F9F9F8] overflow-hidden shadow-lg sm:rounded-lg border border-gray-200";
                if expanded_view() {
                    format!("{} flex-grow overflow-hidden", base_classes)
                } else {
                    base_classes.to_string()
                }
            }}>
                <div class={move || {
                    let base_classes = "overflow-x-auto overflow-y-auto";
                    if expanded_view() {
                        format!("{} h-full", base_classes)
                    } else {
                        format!("{} max-h-80", base_classes)
                    }
                }}>
                    <table class="min-w-full divide-y divide-[#DADADA]">
                        <thead class="bg-[#DADADA]">
                            <tr>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-[#2E3A59] uppercase tracking-wider">
                                    Student ID
                                </th>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-[#2E3A59] uppercase tracking-wider">
                                    Student Name
                                </th>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-[#2E3A59] uppercase tracking-wider">
                                    Test
                                </th>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-[#2E3A59] uppercase tracking-wider">
                                    Date
                                </th>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-[#2E3A59] uppercase tracking-wider">
                                    Percentage
                                </th>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-[#2E3A59] uppercase tracking-wider">
                                    Benchmark
                                </th>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-[#2E3A59] uppercase tracking-wider">
                                    Evaluator
                                </th>
                                <th scope="col" class="px-6 py-3 text-right text-xs font-medium text-[#2E3A59] uppercase tracking-wider">
                                    Actions
                                </th>
                            </tr>
                        </thead>
                        <tbody class="bg-[#F9F9F8] divide-y divide-[#DADADA]">
                            {move || {
                                scores_resource.get().map(|result| {
                                    match result {
                                        Ok(scores) => {
                                            if scores.is_empty() {
                                                view! {
                                                    <tr>
                                                        <td colspan="7" class="px-6 py-4 text-center text-sm text-[#2E3A59]">
                                                            No scores found.
                                                        </td>
                                                    </tr>
                                                }
                                                .into_any()
                                            } else {
                                                scores.iter().rev().map(|score| {
                                                    let student_id = score.student_id;
                                                    let test_id = score.test_id.clone();
                                                    let test_variant = score.test_variant;
                                                    let attempt = score.attempt;

                                                    // Get test data for this score
                                                    let test_data = tests_resource.get()
                                                        .and_then(|result| result.ok())
                                                        .and_then(|tests| tests.iter().find(|t| t.test_id == test_id).cloned());

                                                    let max_score = test_data.as_ref().map(|t| t.score).unwrap_or(0);
                                                    let test_name = test_data.as_ref().map(|t| t.name.clone()).unwrap_or_else(|| "Unknown Test".to_string());
                                                    let benchmark_categories = test_data.as_ref().and_then(|t| t.benchmark_categories.as_ref());

                                                    // Calculate derived values
                                                    let total_score: i32 = score.test_scores.iter().sum();
                                                    let percentage = if max_score > 0 {
                                                        format!("{:.1}%", (total_score as f64 / max_score as f64 * 100.0))
                                                    } else {
                                                        "N/A".to_string()
                                                    };

                                                    let benchmark_label = ScoreUtils::get_benchmark_label(total_score, max_score, benchmark_categories);
                                                    let badge_classes = ColorUtils::get_badge_classes_for_score(total_score, max_score, benchmark_categories);

                                                    // Get student display info
                                                    let student_display = ScoreUtils::get_student_display_info(student_id, &enhanced_students());

                                                    view! {
                                                        <tr class="hover:bg-gray-50">
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-[#2E3A59]">
                                                                {student_display.id}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-[#2E3A59]">
                                                                {student_display.name}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-[#2E3A59]">
                                                                <div class="flex flex-col">
                                                                    <span>{test_name}</span>
                                                                    <span class="text-xs text-[#2E3A59]">{"Variant: "}{score.test_variant}</span>
                                                                </div>
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-[#2E3A59]">
                                                                <div class="flex flex-col">
                                                                    <span>{score.date_administered.format("%b %d, %Y").to_string()}</span>
                                                                    <span class="text-xs text-[#2E3A59]">{score.date_administered.format("%H:%M").to_string()}</span>
                                                                </div>
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap">
                                                                <span class={format!("px-2 inline-flex text-xs leading-5 font-semibold rounded-full {}", badge_classes)}>
                                                                    {percentage}
                                                                </span>
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap">
                                                                <span class={format!("px-2 inline-flex text-xs leading-5 font-semibold rounded-full {}", badge_classes)}>
                                                                    {benchmark_label}
                                                                </span>
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-[#2E3A59]">
                                                                {score.evaluator.clone()}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                                                {
                                                                    let nav = navigate.clone();
                                                                    let test_id = score.test_id.clone();
                                                                    let student_id = score.student_id;
                                                                    let test_variant = score.test_variant.clone();
                                                                    let attempt = score.attempt.clone();

                                                                    view! {
                                                                        <button
                                                                            class="text-indigo-600 hover:text-indigo-900 mr-3"
                                                                            on:click=move |_| {
                                                                                nav(&format!("/reviewtest/{}/{}/{}/{}", test_id, student_id, test_variant, attempt), Default::default());
                                                                            }
                                                                        >
                                                                            View
                                                                        </button>
                                                                    }
                                                                }
                                                            </td>
                                                        </tr>
                                                    }.into_any()
                                                })
                                                .collect_view().into_any()
                                            }
                                        }
                                        Err(_) => {
                                            view! {
                                                <tr>
                                                    <td colspan="7" class="px-6 py-4 text-center text-sm text-[#2E3A59]">
                                                        Failed to load scores. Please try again later.
                                                    </td>
                                                </tr>
                                            }
                                            .into_any()
                                        }
                                    }
                                }).unwrap_or_else(|| view! {
                                    <tr>
                                        <td colspan="7" class="px-6 py-4 text-center text-sm text-[#2E3A59]">
                                            <div class="flex justify-center items-center">
                                                <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-indigo-500" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                                </svg>
                                                Loading scores...
                                            </div>
                                        </td>
                                    </tr>
                                }.into_any())
                            }}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}

// Helper struct for student display info
#[derive(Clone)]
pub struct StudentDisplayInfo {
    pub id: String,
    pub name: String,
}

// Additional utility struct for score-related operations
pub struct ScoreUtils;

impl ScoreUtils {
    pub fn get_student_display_info(
        student_id: i32,
        enhanced_students: &[(Student, Option<DeAnonymizedStudent>)],
    ) -> StudentDisplayInfo {
        if let Some((student, de_anon_opt)) = enhanced_students
            .iter()
            .find(|(s, _)| s.student_id == student_id)
        {
            if let Some(de_anon) = de_anon_opt {
                StudentDisplayInfo {
                    id: de_anon.display_id.clone(),
                    name: de_anon.display_name.clone(),
                }
            } else {
                StudentDisplayInfo {
                    id: student.student_id.to_string(),
                    name: format!(
                        "{} {}",
                        student.firstname.as_ref().unwrap_or(&"Unknown".to_string()),
                        student.lastname.as_ref().unwrap_or(&"Student".to_string())
                    ),
                }
            }
        } else {
            StudentDisplayInfo {
                id: student_id.to_string(),
                name: "Unknown Student".to_string(),
            }
        }
    }

    pub fn get_benchmark_label(
        score: i32,
        max_score: i32,
        benchmark_categories: Option<&Vec<BenchmarkCategory>>,
    ) -> String {
        if max_score <= 0 {
            return "N/A".to_string();
        }

        let percentage = (score as f64 / max_score as f64) * 100.0;

        if let Some(categories) = benchmark_categories {
            for category in categories {
                let min_percent = category.min as f64;
                let max_percent = category.max as f64;
                if percentage >= min_percent && percentage <= max_percent {
                    return category.label.clone();
                }
            }
        }

        // Default labels if no custom categories
        if percentage >= 90.0 {
            "Excellent".to_string()
        } else if percentage >= 80.0 {
            "Good".to_string()
        } else if percentage >= 70.0 {
            "Satisfactory".to_string()
        } else {
            "Needs Improvement".to_string()
        }
    }
}
