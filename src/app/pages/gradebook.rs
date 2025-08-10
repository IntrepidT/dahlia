use crate::app::components::auth::server_auth_components::ServerAuthGuard;
use crate::app::components::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::data_processing::{
    AssessmentSummary, Progress, StudentResultsSummary, TestDetail,
};
use crate::app::components::enhanced_login_form::{
    use_student_mapping_service, DeAnonymizedStudent, StudentMappingService,
};
use crate::app::components::gradebook_side_panel::{ScorePanelType, StudentScorePanel};
use crate::app::components::header::Header;
use crate::app::middleware::global_settings::use_settings;
use crate::app::models::assessment::Assessment;
use crate::app::models::student::Student;
use crate::app::server_functions::assessments::get_assessments;
use crate::app::server_functions::data_wrappers::get_student_results_batch;
use crate::app::server_functions::scores::get_scores_by_test;
use crate::app::server_functions::students::get_students;
use crate::app::server_functions::tests::get_tests_batch;
use chrono::Utc;
use icondata::{BiCheckboxCheckedRegular, BiCheckboxRegular, HiUserCircleOutlineLg};
use leptos::prelude::*;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::*;
use leptos_router::hooks::*;
use leptos_router::path;
use std::collections::HashMap;

#[component]
pub fn Gradebook() -> impl IntoView {
    view! {
        <ServerAuthGuard page_path="/gradebook">
            <GradebookContent />
        </ServerAuthGuard>
    }
}

#[component]
pub fn GradebookContent() -> impl IntoView {
    let (refresh_trigger, set_refresh_trigger) = signal(0);
    let (selected_view, set_selected_view) = signal(SidebarSelected::Dashboard);
    let (search_term, set_search_term) = signal(String::new());

    // Store assessment ID instead of the whole assessment
    let (selected_assessment_id, set_selected_assessment_id) = signal(Option::<String>::None);

    // Side panel state
    let (show_side_panel, set_show_side_panel) = signal(false);
    let (panel_type, set_panel_type) = signal(ScorePanelType::None);
    let (selected_student, set_selected_student) = signal(Option::<Student>::None);

    // Current assessment/test data for the side panel
    let (current_assessment_data, set_current_assessment_data) =
        signal(Option::<AssessmentSummary>::None);
    let (current_test_data, set_current_test_data) = signal(Option::<TestDetail>::None);
    let (next_test_id, set_next_test_id) = signal(Option::<String>::None);

    // Get global settings for anonymization
    let (settings, _) = use_settings();
    let anonymization_enabled = move || settings.get().student_protections;

    // Get the student mapping service
    let (student_mapping_service, _) = use_student_mapping_service();

    // Updated LocalResource syntax - single fetcher function
    let initial_data = LocalResource::new(move || {
        // Create the async block here that tracks refresh_trigger
        let trigger = refresh_trigger.get();
        async move {
            log::info!("Starting gradebook data fetch...");

            // Fetch students and assessments concurrently
            let (students_result, assessments_result) =
                futures::join!(get_students(), get_assessments());

            match (students_result, assessments_result) {
                (Ok(students), Ok(assessments)) => {
                    log::info!(
                        "Fetched {} students and {} assessments",
                        students.len(),
                        assessments.len()
                    );

                    if !students.is_empty() {
                        let student_ids: Vec<i32> =
                            students.iter().map(|student| student.student_id).collect();

                        log::info!("Starting batch fetch for {} students", student_ids.len());

                        // Use the optimized batch function
                        match get_student_results_batch(student_ids).await {
                            Ok(student_results) => {
                                log::info!(
                                    "Successfully fetched batch results for {} students",
                                    student_results.len()
                                );
                                Some((students, assessments, student_results))
                            }
                            Err(e) => {
                                log::error!("Failed to fetch batch results: {}", e);
                                // Return with empty results rather than failing completely
                                Some((students, assessments, HashMap::new()))
                            }
                        }
                    } else {
                        log::info!("No students found");
                        Some((students, assessments, HashMap::new()))
                    }
                }
                (Err(e), _) => {
                    log::error!("Failed to fetch students: {}", e);
                    None
                }
                (_, Err(e)) => {
                    log::error!("Failed to fetch assessments: {}", e);
                    None
                }
            }
        }
    });

    let students = Memo::new(move |_| {
        initial_data
            .get()
            .and_then(|data| data.as_ref().map(|(students, _, _)| students.clone()))
            .unwrap_or_default()
    });

    let assessment_list = Memo::new(move |_| {
        initial_data
            .get()
            .and_then(|data| data.as_ref().map(|(_, assessments, _)| assessments.clone()))
            .unwrap_or_default()
    });

    let all_student_results = Memo::new(move |_| {
        initial_data
            .get()
            .and_then(|data| data.as_ref().map(|(_, _, results)| results.clone()))
            .unwrap_or_default()
    });

    let selected_assessment = Memo::new(move |_| {
        if let Some(assessment_id) = selected_assessment_id.get() {
            if assessment_id.is_empty() {
                return None;
            }

            let assessments = assessment_list.get();
            assessments
                .iter()
                .find(|a| a.id.to_string() == assessment_id)
                .cloned()
        } else {
            None
        }
    });

    // Updated tests resource - single fetcher function that tracks dependencies
    let tests = LocalResource::new(move || {
        let selected_assessment = selected_assessment.get();
        let trigger = refresh_trigger.get();
        async move {
            if let Some(assessment) = selected_assessment {
                match get_tests_batch(assessment.tests).await {
                    Ok(tests) => Some(tests),
                    Err(e) => {
                        log::error!("Failed to fetch tests: {}", e);
                        Some(vec![])
                    }
                }
            } else {
                None
            }
        }
    });

    // Updated scores resource - single fetcher function that tracks dependencies
    let scores = LocalResource::new(move || {
        let selected_assessment = selected_assessment.get();
        let trigger = refresh_trigger.get();
        async move {
            if let Some(assessment) = selected_assessment {
                match get_scores_by_test(assessment.tests).await {
                    Ok(scores) => Some(scores),
                    Err(e) => {
                        log::error!("Failed to fetch scores: {}", e);
                        Some(vec![])
                    }
                }
            } else {
                None
            }
        }
    });

    // Helper function to get display name and ID
    let get_student_display = move |student: &Student| -> (String, String) {
        if anonymization_enabled() {
            if let Some(service) = student_mapping_service.get() {
                if let Some(mapping) = service.get_original_student_info(student.student_id) {
                    return (
                        format!("{} {}", mapping.firstname, mapping.lastname),
                        mapping.original_student_id.to_string(),
                    );
                }
            }
        }

        // Fallback to anonymized or regular display
        (
            format!(
                "{} {}",
                student.firstname.as_deref().unwrap_or("Student"),
                student
                    .lastname
                    .as_deref()
                    .unwrap_or(&format!("#{}", student.student_id))
            ),
            student.student_id.to_string(),
        )
    };

    // OPTIMIZATION 4: Memoized filtered students with debouncing
    let filtered_students = Memo::new(move |_| {
        let search = search_term().trim().to_lowercase();
        let students_list = students.get();

        if search.is_empty() {
            return students_list;
        }

        students_list
            .into_iter()
            .filter(|student| {
                let (display_name, display_id) = get_student_display(student);
                display_name.to_lowercase().contains(&search)
                    || display_id.to_lowercase().contains(&search)
            })
            .collect::<Vec<_>>()
    });

    // Helper function to find the next test ID
    fn find_next_test_id(assessment: &AssessmentSummary) -> Option<String> {
        if assessment.progress == Progress::Completed {
            return None;
        }

        assessment
            .test_details
            .iter()
            .find(|test| test.score < test.total_possible)
            .map(|test| test.test_id.clone())
    }

    // Handler for opening assessment side panel
    let open_assessment_panel = move |assessment_id: String, student_id: i32| {
        let students_list = students.get();
        if let Some(student) = students_list.iter().find(|s| s.student_id == student_id) {
            set_selected_student(Some(student.clone()));

            let results_map = all_student_results.get();
            if let Some(student_results) = results_map.get(&student_id) {
                if let Some(summary) = student_results
                    .assessment_summaries
                    .iter()
                    .find(|s| s.assessment_id == assessment_id)
                {
                    set_current_assessment_data(Some(summary.clone()));
                    set_next_test_id(find_next_test_id(summary));
                    set_panel_type(ScorePanelType::AssessmentScore(assessment_id));
                    set_show_side_panel(true);
                    return;
                }
            }
        }
        log::error!("Failed to load assessment data for side panel");
    };

    // Handler for opening test side panel
    let open_test_panel = move |test_id: String, student_id: i32, attempt: i32| {
        let students_list = students.get();
        if let Some(student) = students_list.iter().find(|s| s.student_id == student_id) {
            set_selected_student(Some(student.clone()));

            if let Some(Some(test_list)) = tests.get() {
                if let Some(test) = test_list.iter().find(|t| t.test_id == test_id) {
                    let mut test_detail = TestDetail {
                        test_id: test.test_id.clone(),
                        test_name: test.name.clone(),
                        test_area: test.testarea.clone().to_string(),
                        score: 0,
                        total_possible: test.score,
                        performance_class: "Not available".to_string(),
                        date_administered: Utc::now(),
                        attempt: 0,
                        test_variant: 0,
                    };

                    if let Some(Some(score_data)) = scores.get() {
                        if let Some(score) = score_data.iter().find(|s| {
                            s.student_id == student_id
                                && s.test_id == test_id
                                && s.attempt == attempt
                        }) {
                            test_detail.score = score.get_total();
                            test_detail.performance_class = if score.get_total() >= (test.score / 2)
                            {
                                "Satisfactory".to_string()
                            } else {
                                "Needs Improvement".to_string()
                            };
                            test_detail.attempt = score.attempt;
                            test_detail.test_variant = score.test_variant;
                        }
                    }

                    set_current_test_data(Some(test_detail));
                    set_panel_type(ScorePanelType::TestScore(test_id, student_id, attempt));
                    set_show_side_panel(true);
                }
            }
        }
    };

    view! {
        <div class="h-screen flex flex-col bg-[#F9F9F8]">
            <Header />
            <div class="flex flex-1 overflow-hidden">
                <DashboardSidebar
                    selected_item=selected_view
                    set_selected_item=set_selected_view
                />
                <main class="flex-1 flex flex-col mt-16 ml-20 px-10 pb-6">
                    <h1 class="text-2xl font-bold mb-2 text-[#2E3A59]">
                        "Gradebook"
                        {move || {
                            if anonymization_enabled() {
                                if student_mapping_service.get().is_some() {
                                    view! {
                                        <span class="ml-2 px-2 py-1 bg-green-100 text-green-800 text-xs rounded-full">
                                            "De-anonymized"
                                        </span>
                                    }.into_any()
                                } else {
                                    view! {
                                        <span class="ml-2 px-2 py-1 bg-yellow-100 text-yellow-800 text-xs rounded-full">
                                            "Anonymized"
                                        </span>
                                    }.into_any()
                                }
                            } else {
                                view! { <span></span> }.into_any()
                            }
                        }}
                    </h1>

                    <div class="flex justify-between items-center mb-2">
                        <div class="w-[40rem] mr-4">
                            <input
                                type="text"
                                placeholder={move || {
                                    if anonymization_enabled() && student_mapping_service.get().is_some() {
                                        "Search students by real name or ID..."
                                    } else {
                                        "Search students..."
                                    }
                                }}
                                prop:value={move || search_term.get()}
                                class="border border-gray-300 rounded px-3 py-1 w-full text-sm"
                                on:input=move |ev| set_search_term(event_target_value(&ev))
                            />
                        </div>
                        <div class="w-[20rem]">
                            <select
                                id="assessment-select"
                                class="block w-full px-2 py-1 bg-white border-gray-200 rounded-md border text-sm"
                                on:change=move |ev| {
                                    let value = event_target_value(&ev);
                                    if value.is_empty() || value == "none" {
                                        set_selected_assessment_id(None);
                                    } else {
                                        set_selected_assessment_id(Some(value));
                                    }
                                }
                                prop:value=move || selected_assessment_id.get().unwrap_or_default()
                            >
                               <option value="">"All Assessments"</option>
                               {move || {
                                   let assessments = assessment_list.get();
                                   assessments.into_iter().map(|assessment| {
                                       view! {
                                           <option value={assessment.id.to_string()}>{assessment.name}</option>
                                       }
                                   }).collect_view()
                               }}
                            </select>
                        </div>
                    </div>

                    // OPTIMIZATION 5: Show loading state
                    {move || {
                        match initial_data.get() {
                            None => view! {
                                <div class="flex-1 flex items-center justify-center">
                                    <div class="text-center">
                                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-[#2E3A59] mx-auto mb-4"></div>
                                        <p class="text-gray-600">"Loading gradebook data..."</p>
                                    </div>
                                </div>
                            }.into_any(),
                            Some(None) => view! {
                                <div class="flex-1 flex items-center justify-center">
                                    <p class="text-red-600">"Failed to load gradebook data"</p>
                                </div>
                            }.into_any(),
                            Some(Some(_)) => view! {
                                <div class="flex-1 flex flex-col overflow-hidden rounded-md">
                                    <div class="h-full overflow-auto">
                                        <table class="min-w-full bg-[#F9F9F8] border border-gray-200 table-fixed divide-y divide-[#DADADA]">
                                            <thead class="sticky top-0 bg-[#DADADA]">
                                                <tr>
                                                    <th class="px-2 py-4 border text-center font-medium text-[#2E3A59] text-md uppercase tracking-wider">"Student Name"</th>
                                                    <th class="px-2 py-4 border text-center font-medium text-[#2E3A59] text-md uppercase tracking-wider">
                                                        {move || {
                                                            if anonymization_enabled() && student_mapping_service.get().is_some() {
                                                                "Original ID"
                                                            } else {
                                                                "ID"
                                                            }
                                                        }}
                                                    </th>
                                                    {
                                                        move || {
                                                            if selected_assessment_id.get().is_none() {
                                                                // Show all assessments as columns
                                                                let assessments = assessment_list.get();
                                                                assessments.into_iter().map(|assessment| {
                                                                    view! {
                                                                        <th class="px-2 py-4 border text-center font-medium text-[#2E3A59] text-md whitespace-normal uppercase tracking-wider">{assessment.name.clone()}</th>
                                                                    }
                                                                }).collect_view().into_any()
                                                            } else {
                                                                // Show selected assessment's tests as columns
                                                                match tests.get() {
                                                                    Some(Some(test_list)) => {
                                                                        test_list.iter().map(|test| {
                                                                            view! {
                                                                                <th class="px-2 py-4 border text-center font-medium text-[#2E3A59] text-md whitespace-normal uppercase tracking-wider">
                                                                                    {format!("{}",&test.name)}
                                                                                    <br/>
                                                                                    {format!("(Out of {})", &test.score)}
                                                                                </th>
                                                                            }
                                                                        }).collect_view().into_any()
                                                                    },
                                                                    _ => view! {}.into_any()
                                                                }
                                                            }
                                                        }
                                                    }
                                                </tr>
                                            </thead>
                                            <tbody class="text-md">
                                                {move || {
                                                    let students_list = filtered_students();
                                                    if students_list.is_empty() {
                                                        view! {
                                                            <tr>
                                                                <td colspan="2" class="px-2 py-1 border-b">
                                                                    "No students match your search criteria."
                                                                </td>
                                                            </tr>
                                                        }.into_any()
                                                    } else {
                                                        // Clone the results map to avoid move issues
                                                        let results_map = all_student_results.get();
                                                        students_list.into_iter().map(|student| {
                                                            let student_id = student.student_id;
                                                            let (display_name, display_id) = get_student_display(&student);

                                                            // Get student results once for this row
                                                            let student_results = results_map.get(&student_id);

                                                            view! {
                                                                <tr>
                                                                    <td class="px-2 py-2 border whitespace-nowrap text-indigo-500 bg-white">
                                                                        <a href=format!("/studentview/{}/results", &student.student_id)>
                                                                            <Icon
                                                                                icon=HiUserCircleOutlineLg
                                                                                attr:class="w-4 h-4 text-[#2E3A59] inline-block mr-2"
                                                                            />
                                                                            {display_name}
                                                                        </a>
                                                                    </td>
                                                                    <td class="px-2 py-2 border whitespace-nowrap text-center bg-white">{display_id}</td>
                                                                    {
                                                                        if selected_assessment_id.get().is_none() {
                                                                            // Show all assessments for this student
                                                                            let assessments = assessment_list.get();
                                                                            assessments.into_iter().map(|assessment| {
                                                                                if let Some(results) = student_results {
                                                                                    if let Some(summary) = results.assessment_summaries.iter()
                                                                                        .find(|summary| summary.assessment_id == assessment.id.to_string()) {
                                                                                        let score = summary.current_score;
                                                                                        let total = summary.total_possible.unwrap_or(0);
                                                                                        let progression_color = if summary.progress == Progress::Completed {
                                                                                            "bg-green-100"
                                                                                        } else {
                                                                                            "bg-yellow-100"
                                                                                        };

                                                                                        let assessment_id = assessment.id.to_string();
                                                                                        let student_id = student.student_id;
                                                                                        let open_assessment = open_assessment_panel.clone();

                                                                                        view! {
                                                                                            <td
                                                                                                class=format!("{} px-2 py-2 border whitespace-nowrap text-center cursor-pointer hover:bg-gray-100", progression_color)
                                                                                                on:click=move |_| open_assessment(assessment_id.clone(), student_id)
                                                                                            >
                                                                                                {format!("{} / {}", score, total)}
                                                                                            </td>
                                                                                        }.into_any()
                                                                                    } else {
                                                                                        view! {
                                                                                            <td class="px-2 py-2 border whitespace-nowrap bg-blue-100 text-center">
                                                                                                "Not started"
                                                                                            </td>
                                                                                        }.into_any()
                                                                                    }
                                                                                } else {
                                                                                    view! {
                                                                                        <td class="px-2 py-2 border whitespace-nowrap text-center">
                                                                                            "-"
                                                                                        </td>
                                                                                    }.into_any()
                                                                                }
                                                                            }).collect_view().into_any()
                                                                        } else {
                                                                            // Show selected assessment's test scores
                                                                            match tests.get() {
                                                                                Some(Some(test_list)) => {
                                                                                    let score_data = scores.get().unwrap_or(None).unwrap_or_default();

                                                                                    test_list.iter().map(|test| {
                                                                                        let score = score_data
                                                                                            .iter()
                                                                                            .find(|s| s.student_id == student.student_id && s.test_id == test.test_id);

                                                                                        let test_id = test.test_id.clone();
                                                                                        let student_id = student.student_id;
                                                                                        let open_test = open_test_panel.clone();
                                                                                        let attempt_clone = match score {
                                                                                            Some(s) => s.attempt.clone(),
                                                                                            None => 0,
                                                                                        };

                                                                                        view! {
                                                                                            <td class="px-2 py-2 border whitespace-nowrap text-center">
                                                                                                {
                                                                                                    match score {
                                                                                                        Some(s) => view! {
                                                                                                            <span class="cursor-pointer hover:text-indigo-600" on:click=move |_| open_test(test_id.clone(), student_id, attempt_clone)>
                                                                                                                {s.get_total().to_string()}
                                                                                                            </span>
                                                                                                        }.into_any(),
                                                                                                        None => view!{"-"}.into_any(),
                                                                                                    }
                                                                                                }
                                                                                            </td>
                                                                                        }.into_any()
                                                                                    }).collect_view().into_any()
                                                                                },
                                                                                _ => view! {}.into_any()
                                                                            }
                                                                        }
                                                                    }
                                                                </tr>
                                                            }.into_any()
                                                        }).collect_view().into_any()
                                                    }
                                                }}
                                            </tbody>
                                        </table>
                                    </div>
                                </div>
                            }.into_any()
                        }
                    }}
                </main>

                <StudentScorePanel
                    show=show_side_panel
                    panel_type=panel_type
                    set_show=Callback::new(move |value: bool| set_show_side_panel.set(value))
                    student=selected_student
                    assessment_data=current_assessment_data
                    test_data=current_test_data
                    next_test=next_test_id
                />
            </div>
        </div>
    }
}
