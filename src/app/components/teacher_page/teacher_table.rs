use crate::app::models::employee::{Employee, EmployeeRole};
use leptos::*;
use std::rc::Rc;

const TABLE_CONTAINER_STYLE: &str =
    "bg-[#F9F9F8] rounded-lg shadow-sm border border-[#DADADA] overflow-hidden";
const TABLE_HEADER_STYLE: &str =
    "py-5 px-6 flex justify-between items-center border-b border-[#2E3A59] bg-[#2E3A59]";
const TABLE_WRAPPER_STYLE: &str = "overflow-x-auto h-[34rem]";
const TABLE_STYLE: &str = "min-w-full divide-y divide-[#DADADA]";
const HEADER_CELL_STYLE: &str =
    "px-6 py-3 text-left text-xs font-medium text-[#2E3A59] uppercase tracking-wider";
const CELL_STYLE: &str = "px-6 py-4 whitespace-nowrap text-sm bg-[#F9F9F8]";
const SELECTED_ROW_STYLE: &str =
    "bg-[#DADADA] border-l-4 border-r-2 border-t-2 border-b-2 border-[#2E3A59]";

#[component]
pub fn TeacherTable(
    #[prop(into)] teachers: Resource<i32, Option<Vec<Employee>>>,
    #[prop(into)] search_term: Signal<String>,
    #[prop(into)] role_filter: Signal<String>,
    #[prop(into)] selected_employee: Signal<Option<Rc<Employee>>>,
    #[prop(into)] set_selected_employee: WriteSignal<Option<Rc<Employee>>>,
    #[prop(into)] is_panel_expanded: Signal<bool>,
) -> impl IntoView {
    let filtered_teachers = create_memo(move |_| {
        let search = search_term().trim().to_lowercase();
        let role = role_filter();

        teachers
            .get()
            .unwrap_or(None)
            .unwrap_or_default()
            .into_iter()
            .filter(|teacher| {
                // For teachers, only filter by search term since they're already filtered by role
                let matches_search = search.is_empty()
                    || teacher.firstname.to_lowercase().contains(&search)
                    || teacher.lastname.to_lowercase().contains(&search);

                // If role filter is set to something other than Teacher, don't show any teachers
                let matches_role = role.is_empty() || role == "Teacher";

                matches_search && matches_role
            })
            .collect::<Vec<_>>()
    });

    // Create a derived class for the container based on panel expansion state
    let container_class = create_memo(move |_| {
        if is_panel_expanded() {
            // Less width when panel is expanded
            format!("{} transition-all duration-300 ease-in-out", TABLE_CONTAINER_STYLE)
        } else {
            // Full width when panel is collapsed
            format!("{} transition-all duration-300 ease-in-out", TABLE_CONTAINER_STYLE)
        }
    });

    view! {
        <div class=move || container_class()>
            <div class=TABLE_HEADER_STYLE>
                <h2 class="text-xl font-medium text-white">
                    "Teachers"
                </h2>
                <span class="text-sm text-white">
                    {move || {
                        let count = filtered_teachers().len();
                        format!("{} {}", count, if count == 1 { "teacher" } else { "teachers" })
                    }}
                </span>
            </div>
            <div class=TABLE_WRAPPER_STYLE>
                <div class="overflow-y-auto max-h-full">
                    <table class=TABLE_STYLE>
                        <thead class="bg-[#DADADA] sticky top-0 z-10">
                            <tr>
                                <th class=HEADER_CELL_STYLE>"ID"</th>
                                <th class=HEADER_CELL_STYLE>"First Name"</th>
                                <th class=HEADER_CELL_STYLE>"Last Name"</th>
                                <th class=HEADER_CELL_STYLE>"Status"</th>
                                <th class=HEADER_CELL_STYLE>"Grade"</th>
                            </tr>
                        </thead>
                        <Suspense fallback=move || view! {
                            <tr>
                                <td colspan="6" class="text-center p-8">
                                    <div class="inline-block h-6 w-6 animate-spin rounded-full border-2 border-[#DADADA] border-t-[#2E3A59]"></div>
                                </td>
                            </tr>
                        }>
                            <tbody>
                                {move || {
                                    let teachers = filtered_teachers();
                                    if teachers.is_empty() {
                                        view! {
                                            <tr>
                                                <td colspan="6" class="px-6 py-12 text-center text-sm text-gray-500">
                                                    "No teachers match your search criteria"
                                                </td>
                                            </tr>
                                        }.into_view()
                                    } else {
                                        teachers.into_iter().map(|teacher| {
                                            let teacher_rc = Rc::new(teacher.clone());
                                            let teacher_cmp = Rc::new(teacher.clone());
                                            let is_selected = move || selected_employee() == Some(teacher_cmp.clone());
                                            view! {
                                                <tr
                                                    class=move || if is_selected() {
                                                        format!("{} {}", SELECTED_ROW_STYLE, "cursor-pointer")
                                                    } else {
                                                        "hover:bg-opacity-20 cursor-pointer border-b border-[#DADADA]".to_string()
                                                    }
                                                    on:click=move |_| set_selected_employee(Some(teacher_rc.clone()))
                                                >
                                                    <td class=format!("{} {}", CELL_STYLE, "text-[#2E3A59]")>{teacher.id}</td>
                                                    <td class=format!("{} {}", CELL_STYLE, "font-medium text-[#2E3A59]")>{&teacher.firstname}</td>
                                                    <td class=format!("{} {}", CELL_STYLE, "text-[#2E3A59] font-medium")>{&teacher.lastname}</td>
                                                    <td class=CELL_STYLE>
                                                        <span class=format!("px-2 py-1 text-xs font-medium rounded-full {}",
                                                            if teacher.status.to_string() == "Active" {
                                                                "bg-[#4CAF50] bg-opacity-40 text-[#2E3A59]"
                                                            } else {
                                                                "bg-[#F44336] bg-opacity-40 text-[#2E3A59]"
                                                            })>
                                                            {teacher.status.to_string()}
                                                        </span>
                                                    </td>
                                                    <td class=format!("{} {}", CELL_STYLE, "text-[#2E3A59] text-opacity-70")>
                                                        {match &teacher.role {
                                                            EmployeeRole::Teacher { grade } =>
                                                                grade.as_ref().map_or("Not Assigned".to_string(), |g| g.to_string()),
                                                            _ => "N/A".to_string()
                                                        }}
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()
                                    }
                                }}
                            </tbody>
                        </Suspense>
                    </table>
                </div>
            </div>
        </div>
    }
}
