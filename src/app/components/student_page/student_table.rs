use crate::app::components::enhanced_login_form::{
    use_student_mapping_service, DeAnonymizedStudent,
};
use crate::app::middleware::global_settings::use_settings;
use crate::app::models::student::ESLEnum;
use crate::app::models::student::Student;
use leptos::prelude::*;
use leptos::prelude::*;
use std::sync::Arc;

// Base colors
const COLOR_PRIMARY: &str = "#2E3A59"; // Navy blue
const COLOR_NEAR_BLACK: &str = "#0D0D0D";
const COLOR_LIGHT_GRAY: &str = "#DADADA"; // Light gray
const COLOR_OFF_WHITE: &str = "#F9F9F8"; // Off-white

// Accent/Functional colors
const COLOR_ERROR: &str = "#D64045";
const COLOR_WARNING: &str = "#E9B872";
const COLOR_SUCCESS: &str = "#5B8C5A";
const COLOR_ACCENT_BLUE: &str = "#3E92CC";
const COLOR_ACCENT_TERRACOTTA: &str = "#D3A588";

// Table styles - Updated for better responsiveness
const TABLE_CONTAINER_STYLE: &str =
    "bg-[#F9F9F8] rounded-lg shadow-sm border border-[#DADADA] overflow-hidden flex flex-col h-full";
const TABLE_HEADER_STYLE: &str =
    "py-3 md:py-5 px-4 md:px-6 flex justify-between items-center bg-[#2E3A59] border-b border-[#2E3A59] flex-shrink-0";
const TABLE_WRAPPER_STYLE: &str = "overflow-auto flex-1 min-h-0 scroll-smooth";
const TABLE_STYLE: &str = "w-full min-w-[800px] divide-y divide-[#DADADA]";
const HEADER_CELL_STYLE: &str =
    "px-2 md:px-6 py-2 md:py-3 text-left text-sm font-medium text-[#2E3A59] uppercase tracking-wider whitespace-nowrap";
const CELL_STYLE: &str =
    "px-2 md:px-6 py-2 md:py-4 whitespace-nowrap text-sm md:text-md bg-[#F9F9F8]";
const SELECTED_ROW_STYLE: &str =
    "bg-[#DADADA] border-l-4 border-t-2 border-b-2 border-r-2 border-[#2E3A59]";

#[component]
pub fn StudentTable(
    #[prop(into)] students: Resource<Option<Vec<Student>>>,
    #[prop(into)] search_term: Signal<String>,
    #[prop(into)] grade_filter: Signal<String>,
    #[prop(into)] teacher_filter: Signal<String>,
    #[prop(into)] iep_filter: Signal<bool>,
    #[prop(into)] esl_filter: Signal<bool>,
    #[prop(into)] intervention_filter: Signal<String>,
    #[prop(into)] student_504_filter: Signal<bool>,
    #[prop(into)] readplan_filter: Signal<bool>,
    #[prop(into)] gt_filter: Signal<bool>,
    #[prop(into)] bip_filter: Signal<bool>,
    #[prop(into)] is_panel_expanded: Signal<bool>,
    #[prop(into)] selected_student: Signal<Option<Arc<Student>>>,
    #[prop(into)] set_selected_student: WriteSignal<Option<Arc<Student>>>,
) -> impl IntoView {
    //get settings
    let (settings, _) = use_settings();
    let anonymization_enabled = move || settings.get().student_protections;

    // Get mapping service for de-anonymization
    let (mapping_service, _) = use_student_mapping_service();

    // Create enhanced student data with de-anonymization info
    let enhanced_students = Memo::new(move |_| {
        let students_data = students.get().unwrap_or(None).unwrap_or_default();

        if anonymization_enabled() {
            students_data
                .into_iter()
                .map(|student| {
                    let de_anon = DeAnonymizedStudent::from_student_with_mapping(
                        &student,
                        mapping_service.get().as_ref(),
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

    let filtered_students = Memo::new(move |_| {
        let search = search_term().trim().to_lowercase();
        let current_grade_level = grade_filter();
        let teacher = teacher_filter();
        let show_iep = iep_filter();
        let show_esl = esl_filter();
        let intervention = intervention_filter();
        let show_504 = student_504_filter();
        let show_readplan = readplan_filter();
        let show_gt = gt_filter();
        let show_bip = bip_filter();

        enhanced_students()
            .into_iter()
            .filter(|(student, de_anon_opt)| {
                // Use de-anonymized data for search if available
                let (search_firstname, search_lastname, search_id) = if let Some(de_anon) =
                    de_anon_opt
                {
                    // Extract first and last name from display_name
                    let name_parts: Vec<&str> = de_anon.display_name.split_whitespace().collect();
                    let first = name_parts.get(0).unwrap_or(&"").to_string();
                    let last = name_parts.get(1).unwrap_or(&"").to_string();
                    (first, last, de_anon.display_id.clone())
                } else {
                    (
                        student
                            .firstname
                            .as_ref()
                            .unwrap_or(&"Unknown".to_string())
                            .clone(),
                        student
                            .lastname
                            .as_ref()
                            .unwrap_or(&"Unknown".to_string())
                            .clone(),
                        student.student_id.to_string(),
                    )
                };

                // Filter by search term (now using de-anonymized data when available)
                let matches_search = search.is_empty()
                    || search_firstname.to_lowercase().contains(&search)
                    || search_lastname.to_lowercase().contains(&search)
                    || search_id.to_lowercase().contains(&search);

                // Filter by grade
                let matches_grade = current_grade_level.is_empty()
                    || student
                        .current_grade_level
                        .to_string()
                        .contains(&current_grade_level);

                // Filter by teacher
                let matches_teacher = teacher == "all" || student.teacher.to_string() == teacher;

                // Filter by IEP
                let matches_iep = !show_iep || student.iep;

                // Filter by ESL - fixed for Option<ESLEnum>
                let matches_esl = !show_esl || student.esl != ESLEnum::NotApplicable;

                // Filter by intervention
                let matches_intervention = intervention.is_empty()
                    || intervention == "all"
                    || (intervention == "None" && student.intervention.is_none())
                    || student
                        .intervention
                        .as_ref()
                        .map(|i| i.to_string().contains(&intervention))
                        .unwrap_or(false);

                // New filters - assuming these fields exist on Student
                let matches_504 = !show_504 || student.student_504;
                let matches_readplan = !show_readplan || student.readplan;
                let matches_gt = !show_gt || student.gt;
                let matches_bip = !show_bip || student.bip;

                matches_search
                    && matches_grade
                    && matches_teacher
                    && matches_iep
                    && matches_esl
                    && matches_intervention
                    && matches_504
                    && matches_readplan
                    && matches_gt
                    && matches_bip
            })
            .collect::<Vec<_>>()
    });

    view! {
        <div class=TABLE_CONTAINER_STYLE>
            <div class=TABLE_HEADER_STYLE>
                <h2 class="text-lg md:text-xl font-medium text-[#F9F9F8]">
                    "Students"
                </h2>
                <span class="text-xs md:text-sm text-[#F9F9F8]">
                    {move || {
                        let count = filtered_students().len();
                        format!("{} {}", count, if count == 1 { "student" } else { "students" })
                    }}
                </span>
            </div>
            <div class=TABLE_WRAPPER_STYLE>
                <table class=TABLE_STYLE>
                    <thead class="bg-[#DADADA] sticky top-0 z-10">
                        <tr>
                            <th class=HEADER_CELL_STYLE>"First"</th>
                            <th class=HEADER_CELL_STYLE>"Last"</th>
                            <th class=HEADER_CELL_STYLE>"ID"</th>
                            <th class=format!("{} {}", HEADER_CELL_STYLE, "whitespace-nowrap")>"Grade"</th>
                            <th class=format!("{} {}", HEADER_CELL_STYLE, "whitespace-nowrap")>"Teacher"</th>
                            <th class=format!("{} {}", HEADER_CELL_STYLE, "whitespace-nowrap")>"IEP"</th>
                            <th class=format!("{} {}", HEADER_CELL_STYLE, "whitespace-nowrap")>"ESL"</th>
                            <th class=format!("{} {}", HEADER_CELL_STYLE, "whitespace-nowrap")>"Intervention"</th>

                            // Additional columns that appear regardless of panel state
                            <th class=format!("{} {}", HEADER_CELL_STYLE, "whitespace-nowrap")>"504 Plan"</th>
                            <th class=format!("{} {}", HEADER_CELL_STYLE, "whitespace-nowrap")>"Read Plan"</th>
                            <th class=format!("{} {}", HEADER_CELL_STYLE, "whitespace-nowrap")>"GT"</th>
                            <th class=format!("{} {}", HEADER_CELL_STYLE, "whitespace-nowrap")>"BEH"</th>
                        </tr>
                    </thead>
                    <Suspense fallback=move || view! {
                        <tr>
                            <td colspan="12" class="text-center p-8">
                                <div class="inline-block h-6 w-6 animate-spin rounded-full border-2 border-[#DADADA] border-t-[#2E3A59]"></div>
                            </td>
                        </tr>
                    }>
                        <tbody class="bg-[#F9F9F8]">
                            {move || {
                                let students = filtered_students();
                                if students.is_empty() {
                                    view! {
                                        <tr>
                                            <td colspan="12" class="px-6 py-12 text-center text-sm text-[#2E3A59] text-opacity-70">
                                                "No students match your search criteria"
                                            </td>
                                        </tr>
                                    }.into_any()
                                } else {
                                    students.into_iter().map(|(student, de_anon_opt)| {
                                        let student_rc = Arc::new(student.clone());
                                        let student_cmp = Arc::new(student.clone());
                                        let is_selected = move || selected_student() == Some(student_cmp.clone());

                                        // Determine display values based on anonymization status
                                        let (display_first, display_last, display_id) = if let Some(de_anon) = &de_anon_opt {
                                            // Split the display_name for first and last name
                                            let name_parts: Vec<&str> = de_anon.display_name.split_whitespace().collect();
                                            let first = name_parts.get(0).unwrap_or(&"Unknown").to_string();
                                            let last = if name_parts.len() > 1 {
                                                name_parts[1..].join(" ")
                                            } else {
                                                "Unknown".to_string()
                                            };
                                            (first, last, de_anon.display_id.clone())
                                        } else {
                                            (
                                                student.firstname.as_ref().unwrap_or(&"Unknown".to_string()).clone(),
                                                student.lastname.as_ref().unwrap_or(&"Unknown".to_string()).clone(),
                                                student.student_id.to_string()
                                            )
                                        };

                                        view! {
                                            <tr
                                                class=move || if is_selected() {
                                                    format!("{} {}", SELECTED_ROW_STYLE, "cursor-pointer")
                                                } else {
                                                    "hover:bg-[#DADADA] hover:bg-opacity-20 cursor-pointer border-b border-[#DADADA]".to_string()
                                                }
                                                on:click=move |_| set_selected_student(Some(student_rc.clone()))
                                            >
                                                <td class=format!("{} {}", CELL_STYLE, "font-medium text-[#2E3A59]")>{display_first}</td>
                                                <td class=format!("{} {}", CELL_STYLE, "font-medium text-[#2E3A59]")>{display_last}</td>
                                                <td class=format!("{} {}", CELL_STYLE, "text-[#2E3A59] text-opacity-70")>{display_id}</td>
                                                <td class=format!("{} {}", CELL_STYLE, "text-[#2E3A59] text-opacity-70")>{student.current_grade_level.clone().to_string()}</td>
                                                <td class=format!("{} {}", CELL_STYLE, "text-[#2E3A59] text-opacity-70")>{student.teacher.clone().to_string()}</td>

                                                // IEP Column
                                                <td class=CELL_STYLE>
                                                    { if student.iep {
                                                        view! {
                                                            <span class="px-2 py-1 text-sm font-medium rounded-full bg-[#4CAF50] bg-opacity-40 text-[#2E3A59]">
                                                                "IEP"
                                                            </span>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <span class="px-2 py-1 text-sm font-medium rounded-full bg-opacity-40 text-[#2E3A59]">
                                                                "-"
                                                            </span>
                                                        }.into_any()
                                                    }}
                                                </td>

                                                // ESL Column
                                                <td class=CELL_STYLE>
                                                    { if student.esl != ESLEnum::NotApplicable {
                                                        view! {
                                                            <span class="px-2 py-1 text-sm font-medium rounded-full bg-[#4CAF50] bg-opacity-40 text-[#2E3A59]">
                                                                {student.esl.to_string()}
                                                            </span>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <span class="px-2 py-1 text-sm font-medium rounded-full  bg-opacity-40 text-[#2E3A59]">
                                                                "-"
                                                            </span>
                                                        }.into_any()
                                                    }}
                                                </td>

                                                // Intervention Column
                                                <td class=CELL_STYLE>
                                                    { if let Some(intervention) = &student.intervention {
                                                        view! {
                                                            <span class="px-2 py-1 text-sm font-medium rounded-full bg-[#4CAF50] bg-opacity-40 text-[#2E3A59]">
                                                                {intervention.to_string()}
                                                            </span>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <span class="px-2 py-1 text-sm font-medium rounded-full bg-opacity-40 text-[#2E3A59]">
                                                                "-"
                                                            </span>
                                                        }.into_any()
                                                    }}
                                                </td>

                                                // 504 Plan Column
                                                <td class=CELL_STYLE>
                                                    { if student.student_504 {
                                                        view! {
                                                            <span class="px-2 py-1 text-sm font-medium rounded-full bg-[#4CAF50] bg-opacity-40 text-[#2E3A59]">
                                                                "504"
                                                            </span>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <span class="px-2 py-1 text-sm font-medium rounded-full bg-opacity-40 text-[#2E3A59]">
                                                                "-"
                                                            </span>
                                                        }.into_any()
                                                    }}
                                                </td>

                                                // Read Plan Column
                                                <td class=CELL_STYLE>
                                                    { if student.readplan {
                                                        view! {
                                                            <span class="px-2 py-1 text-sm font-medium rounded-full bg-[#4CAF50] bg-opacity-40 text-[#2E3A59]">
                                                                "Read Plan"
                                                            </span>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <span class="px-2 py-1 text-sm font-medium rounded-full bg-opacity-40 text-[#2E3A59]">
                                                                "-"
                                                            </span>
                                                        }.into_any()
                                                    }}
                                                </td>

                                                // GT Column
                                                <td class=CELL_STYLE>
                                                    { if student.gt {
                                                        view! {
                                                            <span class="px-2 py-1 text-sm font-medium rounded-full bg-[#4CAF50] bg-opacity-40 text-[#2E3A59]">
                                                                "GT"
                                                            </span>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <span class="px-2 py-1 text-sm font-medium rounded-full bg-opacity-40 text-[#2E3A59]">
                                                                "-"
                                                            </span>
                                                        }.into_any()
                                                    }}
                                                </td>

                                                //BEH/BIP column
                                                <td class=CELL_STYLE>
                                                    { if student.bip {
                                                        view! {
                                                            <span class="px-2 py-1 text-sm font-medium rounded-full bg-[#4CAF50] bg-opacity-40 text-[#2E3A59]">
                                                                "BEH"
                                                            </span>
                                                      }.into_any()
                                                    } else {
                                                        view! {
                                                            <span class="px-2 py-1 text-sm font-medium rounded-full bg-opacity-40 text-[#2E3A59]">
                                                                "-"
                                                            </span>
                                                        }.into_any()
                                                    }}
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view().into_any()
                                }
                            }}
                        </tbody>
                    </Suspense>
                </table>
            </div>
        </div>
    }
}
