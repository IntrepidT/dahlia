use crate::app::models::employee::Employee;
use leptos::*;
use std::rc::Rc;

const TABLE_CONTAINER_STYLE: &str = "mb-4 border-b-2";
const TABLE_WRAPPER_STYLE: &str = "overflow-x-auto rounded-lg shadow h-[33rem]";
const SELECTED_EMPLOYEE_STYLE: &str = "border-b bg-[#FDFBD4] cursor-pointer";

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
            <div class="min-w-0 flex-1 mb-6 flex justify-between items-center">
                <h2 class="text-2xl font-bold leading-7 text-[#00356b] sm:truncate sm:text-3xl sm:tracking-tight">
                    "All Employees"
                </h2>
            </div>
            <div class=TABLE_WRAPPER_STYLE>
                <div class="overflow-y-auto max-h-full">
                    <table class="min-w-full divide-y divide-gray-200">
                        <thead class="bg-[#00356b] sticky top-0 z-10">
                            <tr>
                                <th class="px-6 py-3 text-left text-xs font-medium text-white uppercase tracking-wider">"ID"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-white uppercase tracking-wider">"First Name"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-white uppercase tracking-wider">"Last Name"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-white uppercase tracking-wider">"Status"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-white uppercase tracking-wider">"Role"</th>
                            </tr>
                        </thead>
                        <Suspense fallback=move || view! { <tr><td colspan="6" class="text-center p-4">"Loading"</td></tr>}>
                            <tbody class="bg-white divide-y divide-gray-200">
                                {move || {
                                    let employees = filtered_employees();
                                    if employees.is_empty() {
                                        view! { <tr><td colspan="6" class="px-6 py-4 text-center text-sm text-gray-500">"No employees match your search criteria"</td></tr> }.into_view()
                                    } else {
                                        employees.into_iter().map(|employee| {
                                            let employee_rc = Rc::new(employee.clone());
                                            let employee_cmp = Rc::new(employee.clone());
                                            let is_selected = move || selected_employee() == Some(employee_cmp.clone());
                                            view! {
                                                <tr
                                                    class=move || if is_selected() {SELECTED_EMPLOYEE_STYLE} else {"hover:bg-gray-50 cursor-pointer"}
                                                    on:click=move |_| set_selected_employee(Some(employee_rc.clone()))
                                                >
                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{employee.id}</td>
                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{&employee.firstname}</td>
                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{&employee.lastname}</td>
                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{employee.status.to_string()}</td>
                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{employee.role.to_string()}</td>
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
