use crate::app::components::teacher_page::update_employee_form::UpdateEmployeeForm;
use crate::app::models::employee::{Employee, EmployeeRole};
use leptos::*;
use std::rc::Rc;

// Updated consistent color scheme and styling
const THEME_PRIMARY: &str = "#003366";
const THEME_PRIMARY_LIGHT: &str = "#5D7A9E";
const THEME_GRAY_BG: &str = "#F0F2F5";

// Improved consistent styling with better naming
const CARD_CONTAINER: &str = "h-full bg-white p-6 border-t-4 border-l border-r border-b border-gray-200 shadow-md rounded-lg flex flex-col";
const SECTION_CONTAINER: &str = "bg-gray-50 p-5 rounded-lg border border-gray-100 shadow-sm";
const SECTION_TITLE: &str =
    "text-sm font-semibold text-gray-700 mb-3 pb-2 border-b border-gray-200";
const INFO_TITLE: &str = "text-xs text-gray-600 font-medium";
const INFO_VALUE: &str = "text-gray-800 mt-1";
const INFO_GROUP: &str = "mb-4";
const BUTTON_CONTAINER: &str =
    "mt-6 pt-4 flex gap-3 justify-end sticky bottom-0 bg-white border-t border-gray-200";
const BUTTON_PRIMARY: &str =
    "px-4 py-2 bg-[#00356B] rounded-md font-medium text-white hover:bg-blue-700 transition-colors";
const BUTTON_SECONDARY: &str = "px-4 py-2 bg-gray-200 rounded-md font-medium text-gray-500 hover:text-gray-900 transition-colors border border-gray-300";
const BUTTON_ACCENT: &str = "px-4 py-2 bg-[#FCEDA0] rounded-md font-medium text-gray-900 hover:bg-[#F5E080] transition-colors border border-gray-300";

#[component]
pub fn EmployeeDetails(
    #[prop()] employee: Rc<Employee>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] call_refresh: Callback<()>,
) -> impl IntoView {
    let (updating_employee, set_updating_employee) = create_signal(false);

    // Create a memo for the employee to ensure stable references
    let employee_memo = create_memo(move |_| employee.clone());

    // Create explicit callbacks for the UpdateEmployeeForm
    let on_cancel = Callback::new(move |()| {
        set_updating_employee(false);
    });

    let on_save = Callback::new(move |()| {
        set_updating_employee(false);
        call_refresh.call(());
    });

    view! {
        <Show when=move || updating_employee()>
            <UpdateEmployeeForm
                employee=employee_memo()
                on_cancel=on_cancel
                on_save=on_save
            />
        </Show>
        <Show when=move || !updating_employee()>
            <div class=CARD_CONTAINER>
                <div class="flex items-center justify-between mb-6">
                    <h2 class="text-xl font-bold text-gray-800">
                        {move || format!("{} {}", employee_memo().firstname, employee_memo().lastname)}
                    </h2>
                    <div class="px-3 py-1 rounded-full bg-blue-100 text-blue-800 text-xs font-medium">
                        {move || employee_memo().status.to_string()}
                    </div>
                </div>

                <div class="flex-grow overflow-y-auto space-y-6">
                    // Basic Information Section
                    <div>
                        <h3 class=SECTION_TITLE>"Employee Information"</h3>
                        <div class=SECTION_CONTAINER>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                <div class=INFO_GROUP>
                                    <div class=INFO_TITLE>"Employee ID"</div>
                                    <div class=INFO_VALUE>{move || format!("#{}", employee_memo().id)}</div>
                                </div>
                                <div class=INFO_GROUP>
                                    <div class=INFO_TITLE>"Role"</div>
                                    <div class=INFO_VALUE>{move || employee_memo().role.to_string()}</div>
                                </div>
                                {move || {
                                    let employee = employee_memo();
                                    match &employee.role {
                                        EmployeeRole::Teacher { grade } => {
                                            view! {
                                                <div class=INFO_GROUP>
                                                    <div class=INFO_TITLE>"Assigned Grade"</div>
                                                    <div class=INFO_VALUE>
                                                        {grade.as_ref().map_or(
                                                            view! { <span class="text-gray-400">"Not Assigned"</span> },
                                                            |g| view! { <span class="font-medium">{g.to_string()}</span> }
                                                        )}
                                                    </div>
                                                </div>
                                            }.into_view()
                                        }
                                        _ => view! {}.into_view()
                                    }
                                }}
                            </div>
                        </div>
                    </div>
                </div>
                <div class=BUTTON_CONTAINER>
                    <button
                        type="button"
                        class=BUTTON_SECONDARY
                        on:click=move |_| on_close.call(())
                    >
                        "Close"
                    </button>
                    <button
                        type="button"
                        class=BUTTON_PRIMARY
                        on:click=move |_| set_updating_employee(true)
                    >
                        "Edit Employee"
                    </button>
                </div>
            </div>
        </Show>
    }
}
