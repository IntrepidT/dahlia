use crate::app::components::teacher_page::update_employee_form::UpdateEmployeeForm;
use crate::app::models::employee::{Employee, EmployeeRole};
use leptos::*;
use std::rc::Rc;

const INFO_CONTAINER_STYLE: &str =
    "h-full p-6 border-t-8 border-[#00356B] shadow-lg rounded-lg flex flex-col";
const INFO_CONTENT_STYLE: &str = "flex-grow overflow-y-auto";
const INFO_TITLE_STYLE: &str = "text-stone-400 text-xs";
const INFO_VALUE_STYLE: &str = "mt-1";
const INFO_GROUP_STYLE: &str = "mb-2";
const BUTTON_CONTAINER_STYLE: &str =
    "mt-4 pt-4 flex border-t gap-2 justify-end sticky bottom-0 bg-white w-full";

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
            <div class=INFO_CONTAINER_STYLE>
                <h2 class="text-xl font-bold mb-4">
                    {move || format!("{} {}", employee_memo().firstname, employee_memo().lastname)}
                </h2>

                <div class=INFO_CONTENT_STYLE>
                    <div class="grid grid-cols-2 gap-4">
                        // Basic Information Section
                        <div class="col-span-2">
                            <h3 class="text-sm font-semibold text-gray-600 mb-2">"Basic Information"</h3>
                            <div class="grid grid-cols-2 gap-4 bg-gray-50 p-4 rounded-lg">
                                <div class=INFO_GROUP_STYLE>
                                    <div class=INFO_TITLE_STYLE>"Employee ID"</div>
                                    <div class=INFO_VALUE_STYLE>{move || format!("{}", employee_memo().id)}</div>
                                </div>
                                <div class=INFO_GROUP_STYLE>
                                    <div class=INFO_TITLE_STYLE>"Status"</div>
                                    <div class=INFO_VALUE_STYLE>{move || employee_memo().status.to_string()}</div>
                                </div>
                                <div class=INFO_GROUP_STYLE>
                                    <div class=INFO_TITLE_STYLE>"Role"</div>
                                    <div class=INFO_VALUE_STYLE>{move || employee_memo().role.to_string()}</div>
                                </div>
                                {move || {
                                    let employee = employee_memo();
                                    match &employee.role {
                                        EmployeeRole::Teacher { grade } => {
                                            view! {
                                                <div class=INFO_GROUP_STYLE>
                                                    <div class=INFO_TITLE_STYLE>"Assigned Grade"</div>
                                                    <div class=INFO_VALUE_STYLE>
                                                        {grade.as_ref().map_or("Not Assigned".to_string(), |g| g.to_string())}
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
                <div class=BUTTON_CONTAINER_STYLE>
                    <button
                        type="button"
                        class="px-4 py-2 bg-gray-200 rounded-lg font-bold hover:bg-gray-300"
                        on:click=move |_| on_close.call(())
                    >
                        "Close"
                    </button>
                    <button
                        type="button"
                        class="px-4 py-2 bg-[#00356b] rounded-lg font-bold text-white hover:bg-[#7F9AB5]"
                        on:click=move |_| set_updating_employee(true)
                    >
                        "Edit Employee"
                    </button>
                </div>
            </div>
        </Show>
    }
}
