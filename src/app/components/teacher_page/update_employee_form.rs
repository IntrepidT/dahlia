use crate::app::models::employee::{Employee, EmployeeRole, StatusEnum, UpdateEmployeeRequest};
use crate::app::models::student::GradeEnum;
use crate::app::server_functions::employees::edit_employee;
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use log::info;
use std::str::FromStr;
use std::sync::Arc;
use strum::IntoEnumIterator;
use validator::Validate;

// Define consistent styling constants to match employee_details.rs
const INFO_CONTAINER_STYLE: &str =
    "h-full p-6 border-t-8 border-[#00356B] shadow-lg rounded-lg flex flex-col";
const FORM_GROUP_STYLE: &str = "mb-4";
const FORM_LABEL_STYLE: &str = "block text-stone-400 text-xs mb-1";
const FORM_INPUT_STYLE: &str = "w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500";
const FORM_SELECT_STYLE: &str = "w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500";
const ERROR_MESSAGE_STYLE: &str = "text-red-500 text-sm mb-4";
const BUTTON_CONTAINER_STYLE: &str =
    "mt-4 pt-4 flex border-t gap-2 justify-end sticky bottom-0 bg-[#F9F9F8] w-full";
const BUTTON_PRIMARY_STYLE: &str = "px-4 py-2 bg-green-500 text-white rounded-lg font-bold";
const BUTTON_SECONDARY_STYLE: &str = "px-4 py-2 bg-gray-200 rounded-lg font-bold hover:bg-gray-300";

#[component]
pub fn UpdateEmployeeForm(
    employee: Arc<Employee>,
    #[prop(into)] on_cancel: Callback<()>,
    #[prop(into)] on_save: Callback<()>,
) -> impl IntoView {
    let (id, set_new_id) = signal(employee.id.clone());
    let (new_firstname, set_new_firstname) = signal(employee.firstname.clone());
    let (new_lastname, set_new_lastname) = signal(employee.lastname.clone());
    let (new_status, set_new_status) = signal(employee.status.to_string().clone());
    let (new_role, set_new_role) = signal(employee.role.to_string());

    let (yes_no_grade, set_yes_no_grade) = if new_role() == "Teacher" {
        signal(true)
    } else {
        signal(false)
    };

    let (new_grade, set_new_grade) = match &employee.role {
        EmployeeRole::Teacher { grade } => {
            signal(grade.clone().expect("Some value received").to_string())
        }
        _ => signal(String::from("None")),
    };

    let (error_message, set_error_message) = signal(String::new());
    let (if_error, set_if_error) = signal(false);

    let handle_submit_update_employee = move |ev: SubmitEvent| {
        ev.prevent_default();

        let convert_status_to_enum = match StatusEnum::from_str(&new_status()) {
            Ok(employee_status) => employee_status,
            Err(_) => {
                set_if_error(true);
                set_error_message("Invalid employee status".to_string());
                return;
            }
        };

        let convert_role_to_enum = if new_role() == "Teacher" {
            let grade_enum = GradeEnum::from_str(&new_grade()).ok();
            EmployeeRole::Teacher { grade: grade_enum }
        } else {
            match EmployeeRole::from_str(&new_role()) {
                Ok(role) => role,
                Err(_) => {
                    set_if_error(true);
                    set_error_message("Invalid employee role".to_string());
                    return;
                }
            }
        };

        let update_employee_request = UpdateEmployeeRequest {
            id: id(),
            firstname: new_firstname(),
            lastname: new_lastname(),
            status: convert_status_to_enum,
            role: convert_role_to_enum,
            grade: GradeEnum::from_str(&new_grade()).ok(),
        };

        log::info!("Update Employee Request successful");
        match update_employee_request.validate() {
            Ok(_) => {
                log::info!("Validation passed");
            }
            Err(e) => {
                log::error!("Validatino failed: {:?}", e);
                set_if_error(true);
                set_error_message(format!("Validation error: {:?}", e));
                return;
            }
        }

        spawn_local(async move {
            match edit_employee(update_employee_request).await {
                Ok(_) => {
                    log::info!("Successfully updated employee");
                    on_save.run(());
                }
                Err(e) => {
                    set_if_error(true);
                    set_error_message(format!("Error updating employee: {:?}", e));
                }
            }
        });
    };

    view! {
        <div class=INFO_CONTAINER_STYLE>
            <Show when=move || if_error()>
                <p class=ERROR_MESSAGE_STYLE>{error_message()}</p>
            </Show>
            <h2 class="text-xl font-bold mb-4">"Update Employee"</h2>
            <form on:submit=handle_submit_update_employee class="flex-grow">
                <div class=FORM_GROUP_STYLE>
                    <label class=FORM_LABEL_STYLE>"First Name"</label>
                    <input
                        type="text"
                        class=FORM_INPUT_STYLE
                        placeholder="Enter first name"
                        value=new_firstname()
                        on:input=move |ev| set_new_firstname(event_target_value(&ev))
                        required
                    />
                </div>

                <div class=FORM_GROUP_STYLE>
                    <label class=FORM_LABEL_STYLE>"Last Name"</label>
                    <input
                        type="text"
                        class=FORM_INPUT_STYLE
                        placeholder="Enter last name"
                        value=new_lastname()
                        on:input=move |ev| set_new_lastname(event_target_value(&ev))
                        required
                    />
                </div>
                <div class=FORM_GROUP_STYLE>
                    <label class=FORM_LABEL_STYLE>"System ID"</label>
                    <input
                        type="text"
                        class=FORM_INPUT_STYLE
                        value=id()
                        readonly
                    />
                </div>

                <div class=FORM_GROUP_STYLE>
                    <label class=FORM_LABEL_STYLE>"Status"</label>
                    <select
                        class=FORM_SELECT_STYLE
                        on:change=move |ev| set_new_status(event_target_value(&ev))
                    >
                        {StatusEnum::iter().map(|status| view! {
                            <option value=format!("{}", status) selected=(status.to_string() == new_status())>
                                {format!("{}", status)}
                            </option>
                        }).collect::<Vec<_>>()}
                    </select>
                </div>

                <div class=FORM_GROUP_STYLE>
                    <label class=FORM_LABEL_STYLE>"Role"</label>
                    <select
                        class=FORM_SELECT_STYLE
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            set_new_role(value.clone());
                            set_yes_no_grade(value == "Teacher");
                        }
                    >
                        <option value="" disabled selected>"Select a role"</option>
                        {EmployeeRole::iter().map(|role| view! {
                            <option value=format!("{}", role) selected=(role.to_string() == new_role())>
                                {format!("{}", role)}
                            </option>
                        }).collect::<Vec<_>>()}
                    </select>
                </div>

                <Show when=move || yes_no_grade()>
                    <div class=FORM_GROUP_STYLE>
                        <label class=FORM_LABEL_STYLE>"Assigned Grade"</label>
                        <select
                            class=FORM_SELECT_STYLE
                            on:change=move |ev| set_new_grade(event_target_value(&ev))
                        >
                            {GradeEnum::iter().map(|grade| view! {
                                <option value=format!("{}", grade) selected=(grade.to_string() == new_grade())>
                                    {format!("{}", grade)}
                                </option>
                            }).collect::<Vec<_>>()}
                        </select>
                    </div>
                </Show>

                <div class=BUTTON_CONTAINER_STYLE>
                    <button
                        type="button"
                        class=BUTTON_SECONDARY_STYLE
                        on:click=move |_| on_cancel.run(())
                    >
                        "Cancel"
                    </button>
                    <button
                        type="submit"
                        class=BUTTON_PRIMARY_STYLE
                    >
                        "Save Changes"
                    </button>
                </div>
            </form>
        </div>
    }
}
