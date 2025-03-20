use crate::app::models::employee::Employee;
use leptos::*;
use std::rc::Rc;

const TABLE_CONTAINER_STYLE: &str =
    "bg-white rounded-lg shadow-sm border border-gray-100 overflow-hidden";
const TABLE_HEADER_STYLE: &str =
    "py-5 px-6 flex justify-between items-center border-b border-gray-100 bg-[#00356b]";
const TABLE_WRAPPER_STYLE: &str = "overflow-x-auto h-[33rem]";
const TABLE_STYLE: &str = "min-w-full divide-y divide-gray-100";
const HEADER_CELL_STYLE: &str =
    "px-6 py-3 text-left text-xs font-medium text-gray-600 uppercase tracking-wider";
const CELL_STYLE: &str = "px-6 py-4 whitespace-nowrap text-sm";
const SELECTED_ROW_STYLE: &str = "bg-blue-50 border-l-4 border-blue-500";

#[component]
pub fn EmployeeTable(
    #[prop(into)] employees: Resource<i32, Option<Vec<Employee>>>,
    #[prop(into)] search_term: Signal<String>,
    #[prop(into)] role_filter: Signal<String>,
    #[prop(into)] selected_employee: Signal<Option<Rc<Employee>>>,
    #[prop(into)] set_selected_employee: WriteSignal<Option<Rc<Employee>>>,
) -> impl IntoView {
    let filtered_employees = create_memo(move |_| {
        let search = search_term().trim().to_lowercase();
        let role = role_filter();

        employees
            .get()
            .unwrap_or(None)
            .unwrap_or_default()
            .into_iter()
            .filter(|employee| {
                // Filter by search term
                let matches_search = search.is_empty()
                    || employee.firstname.to_lowercase().contains(&search)
                    || employee.lastname.to_lowercase().contains(&search);

                // Filter by role
                let matches_role = role.is_empty() || employee.role.to_string().contains(&role);

                matches_search && matches_role
            })
            .collect::<Vec<_>>()
    });

    view! {
        <div class=TABLE_CONTAINER_STYLE>
            <div class=TABLE_HEADER_STYLE>
                <h2 class="text-xl font-medium text-white">
                    "Employees"
                </h2>
                <span class="text-sm text-white">
                    {move || {
                        let count = filtered_employees().len();
                        format!("{} {}", count, if count == 1 { "employee" } else { "employees" })
                    }}
                </span>
            </div>
            <div class=TABLE_WRAPPER_STYLE>
                <div class="overflow-y-auto max-h-full">
                    <table class=TABLE_STYLE>
                        <thead class="bg-gray-50 sticky top-0 z-10">
                            <tr>
                                <th class=HEADER_CELL_STYLE>"ID"</th>
                                <th class=HEADER_CELL_STYLE>"First Name"</th>
                                <th class=HEADER_CELL_STYLE>"Last Name"</th>
                                <th class=HEADER_CELL_STYLE>"Status"</th>
                                <th class=HEADER_CELL_STYLE>"Role"</th>
                            </tr>
                        </thead>
                        <Suspense fallback=move || view! {
                            <tr>
                                <td colspan="6" class="text-center p-8">
                                    <div class="inline-block h-6 w-6 animate-spin rounded-full border-2 border-gray-300 border-t-gray-600"></div>
                                </td>
                            </tr>
                        }>
                            <tbody>
                                {move || {
                                    let employees = filtered_employees();
                                    if employees.is_empty() {
                                        view! {
                                            <tr>
                                                <td colspan="6" class="px-6 py-12 text-center text-sm text-gray-500">
                                                    "No employees match your search criteria"
                                                </td>
                                            </tr>
                                        }.into_view()
                                    } else {
                                        employees.into_iter().map(|employee| {
                                            let employee_rc = Rc::new(employee.clone());
                                            let employee_cmp = Rc::new(employee.clone());
                                            let is_selected = move || selected_employee() == Some(employee_cmp.clone());
                                            view! {
                                                <tr
                                                    class=move || if is_selected() {
                                                        format!("{} {}", SELECTED_ROW_STYLE, "cursor-pointer")
                                                    } else {
                                                        "hover:bg-gray-50 cursor-pointer border-b border-gray-100".to_string()
                                                    }
                                                    on:click=move |_| set_selected_employee(Some(employee_rc.clone()))
                                                >
                                                    <td class=format!("{} {}", CELL_STYLE, "text-gray-600")>{employee.id}</td>
                                                    <td class=format!("{} {}", CELL_STYLE, "font-medium text-gray-600")>{&employee.firstname}</td>
                                                    <td class=format!("{} {}", CELL_STYLE, "font-medium text-gray-600")>{&employee.lastname}</td>
                                                    <td class=CELL_STYLE>
                                                        <span class=format!("px-2 py-1 text-xs font-medium rounded-full {}",
                                                            if employee.status.to_string() == "Active" {
                                                                "bg-green-200 text-green-800 font-medium"
                                                            } else {
                                                                "bg-gray-200 font-mediumtext-gray-700"
                                                            })>
                                                            {employee.status.to_string()}
                                                        </span>
                                                    </td>
                                                    <td class=format!("{} {}", CELL_STYLE, "text-gray-500 font-medium")>{employee.role.to_string()}</td>
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
