use crate::app::components::teacher_page::update_employee_form::UpdateEmployeeForm;
use crate::app::models::employee::{Employee, EmployeeRole};
use leptos::*;
use std::rc::Rc;

// Updated consistent color scheme and styling
const THEME_PRIMARY: &str = "#003366";
const THEME_PRIMARY_LIGHT: &str = "#5D7A9E";
const THEME_GRAY_BG: &str = "#F0F2F5";

// Improved consistent styling with better naming
const CARD_CONTAINER: &str =
    "h-[95%] bg-[#F9F9F8] p-3 sm:mb-3 mb-2 pb-2 border-t-8 border-[#2E3A59] shadow-md rounded-lg flex flex-col";
const SECTION_CONTAINER: &str = "bg-white p-5 rounded-lg border border-[#DADADA] shadow-sm";
const SECTION_TITLE: &str =
    "text-sm font-semibold text-[#2E3A59] mb-3 pb-2 border-b border-[#DADADA]";
const INFO_TITLE: &str = "text-xs text-[#2E3A59] text-opacity-70 font-medium";
const INFO_VALUE: &str = "text-[#2E3A59] mt-1";
const INFO_GROUP: &str = "mb-4";
const BUTTON_CONTAINER: &str =
    "mt-6 pt-4 flex gap-3 justify-end sticky bottom-0 bg-[#F9F9F8] border-t border-[#DADADA]";
const BUTTON_PRIMARY: &str =
    "px-4 py-2 bg-[#2E3A59] rounded-md font-medium text-[#F9F9F8] hover:bg-opacity-80 transition-colors";
const BUTTON_SECONDARY: &str = "px-4 py-2 bg-[#F9F9F8] rounded-md font-medium text-gray-600 bg-white hover:bg-gray-50 transition-colors border  shadow-sm border-[#DADADA]";
const BUTTON_ACCENT: &str = "px-4 py-2 bg-[#F9F9F8] rounded-md font-medium text-[#2E3A59] hover:bg-opacity-30 hover:bg-[#DADADA] transition-colors border border-[#DADADA]";

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
                    <h2 class="text-xl font-bold text-[#2E3A59]">
                        {move || format!("{} {}", employee_memo().firstname, employee_memo().lastname)}
                    </h2>
                    <div class="px-3 py-1 rounded-full bg-[#2E3A59] text-white text-xs font-medium">
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
