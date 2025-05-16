use crate::app::components::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::data_processing::{
    AssessmentSummary, Progress, StudentResultsSummary, TestDetail,
};
use crate::app::components::header::Header;
use crate::app::models::assessment::Assessment;
use crate::app::server_functions::assessments::get_assessments;
use crate::app::server_functions::data_wrappers::get_student_results_batch;
use crate::app::server_functions::scores::get_scores_by_test;
use crate::app::server_functions::students::get_students;
use crate::app::server_functions::tests::get_tests_batch;
use icondata::{BiCheckboxCheckedRegular, BiCheckboxRegular, HiUserCircleOutlineLg};
use leptos::*;
use leptos_icons::Icon;
use leptos_router::*;
use std::collections::HashMap;

#[component]
pub fn Gradebook() -> impl IntoView {
    let (refresh_trigger, set_refresh_trigger) = create_signal(0);
    let (selected_view, set_selected_view) = create_signal(SidebarSelected::Dashboard);
    let (search_term, set_search_term) = create_signal(String::new());

    // Store assessment ID instead of the whole assessment
    let (selected_assessment_id, set_selected_assessment_id) =
        create_signal(Option::<String>::None);

    // Fetch students
    let students = create_resource(
        move || refresh_trigger(),
        |_| async move {
            match get_students().await {
                Ok(students) => Some(students),
                Err(e) => {
                    log::error!("Failed to fetch students: {}", e);
                    None
                }
            }
        },
    );

    // Fetch assessment list
    let assessment_list = create_resource(move || (), |_| async move { get_assessments().await });

    // Create a derived resource to get the selected assessment
    let selected_assessment = create_memo(move |_| {
        if let Some(assessment_id) = selected_assessment_id.get() {
            if assessment_id.is_empty() {
                return None;
            }

            match assessment_list.get() {
                Some(Ok(assessments)) => assessments
                    .iter()
                    .find(|a| a.id.to_string() == assessment_id)
                    .cloned(),
                _ => None,
            }
        } else {
            None
        }
    });

    // Create a derived resource that reacts to changes in selected assessment and loads up tests in batches
    let tests = create_resource(
        move || (selected_assessment.get(), refresh_trigger()),
        |(selected_assessment_opt, _)| async move {
            if let Some(assessment) = selected_assessment_opt {
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
        },
    );

    // Create resource for scores by test
    let scores = create_resource(
        move || (selected_assessment.get(), refresh_trigger()),
        |(selected_assessment_opt, _)| async move {
            if let Some(assessment) = selected_assessment_opt {
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
        },
    );

    // Filter students based on search term
    let filtered_students = create_memo(move |_| {
        let search = search_term().trim().to_lowercase();

        students
            .get()
            .unwrap_or(None)
            .unwrap_or_default()
            .into_iter()
            .filter(|student| {
                search.is_empty()
                    || student.firstname.to_lowercase().contains(&search)
                    || student.lastname.to_lowercase().contains(&search)
            })
            .collect::<Vec<_>>()
    });

    // Create a resource for all student results to avoid repeated API calls
    let all_student_results = create_resource(
        move || (students.get(), refresh_trigger()),
        move |(students_opt, _)| async move {
            let mut results_map = HashMap::new();

            if let Some(Some(students_list)) = students_opt {
                if !students_list.is_empty() {
                    let student_ids: Vec<i32> = students_list
                        .iter()
                        .map(|student| student.student_id)
                        .collect();

                    match get_student_results_batch(student_ids).await {
                        Ok(batch_results) => {
                            results_map = batch_results;
                        }
                        Err(e) => {
                            log::error!("Failed to fetch batch results: {}", e);
                        }
                    }
                }
            }
            results_map
        },
    );

    view! {
        <div class="h-screen flex flex-col bg-[#F9F9F8]">
            <Header />
            <div class="flex  flex-1 overflow-hidden">
                <DashboardSidebar
                    selected_item=selected_view
                    set_selected_item=set_selected_view
                />
                <main class="flex-1 flex flex-col mt-16 ml-20 px-10 pb-6">
                    <h1 class="text-2xl font-bold mb-2 text-[#2E3A59]">"Gradebook"</h1>

                    <div class="flex justify-between items-center mb-2">
                        <div class="w-[40rem] mr-4">
                            <input
                                type="text"
                                placeholder="Search students..."
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
                               {move || match assessment_list.get(){
                                    None => view!{<option>"Loading..."</option>}.into_view(),
                                    Some(Ok(list)) => list.into_iter().map(|assessment| {
                                        view! {
                                            <option value={assessment.id.to_string()}>{assessment.name}</option>
                                        }
                                    }).collect_view(),
                                    Some(Err(e)) => view! {<option>"Error: " {e.to_string()}</option>}.into_view(),
                                }}
                            </select>
                        </div>
                    </div>
                    <div class="flex-1 flex flex-col overflow-hidden rounded-md">
                        <div class="h-full overflow-auto">
                            <table class="min-w-full bg-[#F9F9F8] border border-gray-200 table-fixed divide-y divide-[#DADADA]">
                                <thead class="sticky top-0 bg-[#DADADA]">
                                    <tr>
                                        <th class="px-2 py-4 border text-center font-medium text-[#2E3A59] text-md uppercase tracking-wider">"Student Name"</th>
                                        <th class="px-2 py-4 border text-center font-medium text-[#2E3A59] text-md uppercase tracking-wider">"ID"</th>
                                        {
                                            move || {
                                                if selected_assessment_id.get().is_none() {
                                                    // Show all assessments as columns
                                                    match assessment_list.get() {
                                                        None => view!{}.into_view(),
                                                        Some(Ok(list)) => list.into_iter().map(|assessment| {
                                                            view! {
                                                                <th class="px-2 py-4 border text-center font-medium text-[#2E3A59] text-md whitespace-normal uppercase tracking-wider">{&assessment.name}</th>
                                                            }
                                                        }).collect_view(),
                                                        Some(Err(_)) => view! {}.into_view(),
                                                    }
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
                                                            }).collect_view()
                                                        },
                                                        _ => view! {}.into_view()
                                                    }
                                                }
                                            }
                                        }
                                    </tr>
                                </thead>
                                <tbody class="text-md">
                                    {move || {
                                        let students = filtered_students();
                                        if students.is_empty() {
                                            view! {
                                                <tr>
                                                    <td colspan="2" class="px-2 py-1 border-b">
                                                        "No students match your search criteria."
                                                    </td>
                                                </tr>
                                            }.into_view()
                                        } else {
                                            let results_map = all_student_results.get().unwrap_or_default();
                                            students.into_iter().map(|student| {
                                                let student_id = student.student_id;
                                                let student_results = results_map.get(&student_id);
                                                view! {
                                                    <tr>
                                                        <td class="px-2 py-2 border whitespace-nowrap text-indigo-500">
                                                            <a href=format!("/studentview/{}/results", &student.student_id)>
                                                                <Icon
                                                                    icon=HiUserCircleOutlineLg
                                                                    class="w-4 h-4 text-[#2E3A59] inline-block mr-2"
                                                                />
                                                                {format!("{} {}", &student.firstname, &student.lastname)}
                                                            </a>
                                                        </td>
                                                        <td class="px-2 py-2 border whitespace-nowrap text-center">{&student.student_id.to_string()}</td>
                                                        {
                                                            move || {
                                                                let student_results_map = all_student_results.get().unwrap_or_default();
                                                                let student_results = student_results_map.get(&student.student_id);

                                                                if selected_assessment_id.get().is_none() {
                                                                    // Show all assessments for this student
                                                                    match assessment_list.get() {
                                                                        None => view! {}.into_view(),
                                                                        Some(Ok(list)) => list.into_iter().map(|assessment| {
                                                                            // Try to find assessment summary for this assessment
                                                                            if let Some(results) = student_results {
                                                                                if let Some(summary) = results.assessment_summaries.iter()
                                                                                    .find(|summary| summary.assessment_id == assessment.id.to_string()) {
                                                                                    // Found an assessment summary
                                                                                    let score = summary.current_score;
                                                                                    let total = summary.total_possible.unwrap_or(0);
                                                                                    let progression_color = if summary.progress == Progress::Completed {
                                                                                        "bg-green-100"
                                                                                    } else {
                                                                                        "bg-yellow-100"
                                                                                    };
                                                                                    let icon_val = if summary.progress == Progress::Completed {
                                                                                        BiCheckboxCheckedRegular
                                                                                    } else {
                                                                                        BiCheckboxRegular
                                                                                    };
                                                                                    view! {
                                                                                        <td class="px-2 py-2 border whitespace-nowrap text-center">
                                                                                            {format!("{} / {}", score, total)}
                                                                                            <Icon
                                                                                                icon=icon_val
                                                                                                class=format!("w-6 h-6 text-[#2E3A59] inline-block ml-2 {}", progression_color)
                                                                                            />
                                                                                        </td>
                                                                                    }.into_view()
                                                                                } else {
                                                                                    // No summary for this assessment
                                                                                    view! {
                                                                                        <td class="px-2 py-2 border whitespace-nowrap bg-blue-100 text-center">
                                                                                            "Not started"
                                                                                        </td>
                                                                                    }.into_view()
                                                                                }
                                                                            } else {
                                                                                // No results for this student
                                                                                view! {
                                                                                    <td class="px-2 py-2 border whitespace-nowrap text-center">
                                                                                        "-"
                                                                                    </td>
                                                                                }.into_view()
                                                                            }
                                                                        }).collect_view(),
                                                                        Some(Err(_)) => view! {}.into_view(),
                                                                    }
                                                                } else {
                                                                    // Show selected assessment's test scores
                                                                    match tests.get() {
                                                                        Some(Some(test_list)) => {
                                                                            let score_data = scores.get().unwrap_or(None).unwrap_or_default();

                                                                            test_list.iter().map(|test| {
                                                                                let score = score_data
                                                                                    .iter()
                                                                                    .find(|s| s.student_id == student.student_id && s.test_id == test.test_id);

                                                                                view! {
                                                                                    <td class="px-2 py-2 border whitespace-nowrap text-center">
                                                                                        {
                                                                                            match score {
                                                                                                Some(s) => view! {
                                                                                                    {s.get_total().to_string()}

                                                                                                    <a href=format!("/reviewtest/{}/{}/{}/{}", test.test_id, student.student_id, test.test_variant, s.attempt) class="inline-block ml-2">
                                                                                                        <button class="bg-blue-500 text-white px-3 py-1 hover:bg-blue-600 rounded-lg">
                                                                                                            "Review"
                                                                                                        </button>
                                                                                                    </a>
                                                                                                }.into_view(),
                                                                                                None => view!{"-"}.into_view(),
                                                                                            }
                                                                                        }
                                                                                    </td>
                                                                                }
                                                                            }).collect_view()
                                                                        },
                                                                        _ => view! {}.into_view()
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    </tr>
                                                }
                                            }).collect_view()
                                        }
                                    }}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </main>
            </div>
        </div>
    }
}
