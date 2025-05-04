use crate::app::models::employee::{AddNewEmployeeRequest, EmployeeRole, StatusEnum};
use crate::app::models::student::GradeEnum;
use crate::app::server_functions::employees::add_employee;
use leptos::ev::SubmitEvent;
use leptos::*;
use std::str::FromStr;
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
pub fn AddEmployeeForm(
    #[prop(into)] on_cancel: Callback<()>,
    #[prop(into)] on_save: Callback<()>,
) -> impl IntoView {
    let (new_firstname, set_new_firstname) = create_signal(String::new());
    let (new_lastname, set_new_lastname) = create_signal(String::new());
    let (new_status, set_new_status) = create_signal(String::from("Not Applicable"));
    let (new_role, set_new_role) = create_signal(String::new());
    let (yes_no_grade, set_yes_no_grade) = create_signal(false);
    let (new_grade, set_new_grade) = create_signal(String::from("None"));
    let (error_message, set_error_message) = create_signal(String::new());
    let (if_error, set_if_error) = create_signal(false);

    let handle_submit_new_employee = move |ev: SubmitEvent| {
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

        let add_employee_request = AddNewEmployeeRequest {
            firstname: new_firstname(),
            lastname: new_lastname(),
            status: convert_status_to_enum,
            role: convert_role_to_enum,
            grade: GradeEnum::from_str(&new_grade()).ok(),
        };

        if let Err(_) = add_employee_request.validate() {
            set_if_error(true);
            set_error_message("All fields required".to_string());
            return;
        }

        spawn_local(async move {
            match add_employee(add_employee_request).await {
                Ok(_) => on_save(()),
                Err(e) => {
                    set_if_error(true);
                    set_error_message(format!("Error adding employee: {:?}", e));
                }
            }
        });
    };

    view! {
        <div class=INFO_CONTAINER_STYLE>
            <Show when=move || if_error()>
                <p class=ERROR_MESSAGE_STYLE>{error_message()}</p>
            </Show>
            <h2 class="text-xl font-bold mb-4">"Add New Employee"</h2>
            <form on:submit=handle_submit_new_employee class="flex-grow">
                <div class=FORM_GROUP_STYLE>
                    <label class=FORM_LABEL_STYLE>"First Name"</label>
                    <input
                        type="text"
                        class=FORM_INPUT_STYLE
                        placeholder="Enter first name"
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
                        on:input=move |ev| set_new_lastname(event_target_value(&ev))
                        required
                    />
                </div>

                <div class=FORM_GROUP_STYLE>
                    <label class=FORM_LABEL_STYLE>"Status"</label>
                    <select
                        class=FORM_SELECT_STYLE
                        on:change=move |ev| set_new_status(event_target_value(&ev))
                    >
                        <option value="" disabled selected>"Select Employee Status"</option>
                        {StatusEnum::iter().map(|status| view! {
                            <option>{format!("{}", status)}</option>
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
                            <option>{format!("{}", role)}</option>
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
                            <option value="" disabled selected>"Select Grade"</option>
                            {GradeEnum::iter().map(|grade| view! {
                                <option>{format!("{}", grade)}</option>
                            }).collect::<Vec<_>>()}
                        </select>
                    </div>
                </Show>

                <div class=BUTTON_CONTAINER_STYLE>
                    <button
                        type="button"
                        class=BUTTON_SECONDARY_STYLE
                        on:click=move |_| on_cancel(())
                    >
                        "Cancel"
                    </button>
                    <button
                        type="submit"
                        class=BUTTON_PRIMARY_STYLE
                    >
                        "Save"
                    </button>
                </div>
            </form>
        </div>
    }
}
