use crate::app::models::student::InterventionEnum;
use crate::app::models::student::{AddStudentRequest, ESLEnum, GenderEnum, GradeEnum};
use crate::app::models::EmployeeRole;
use chrono::NaiveDate;
use leptos::ev::SubmitEvent;
use leptos::*;
use std::str::FromStr;
use strum::IntoEnumIterator;
use validator::Validate;

// Styles
const INFO_CONTAINER_STYLE: &str =
    "h-full p-6 border-t-8 border-[#00356B] shadow-lg rounded-lg flex flex-col";
const INFO_CONTENT_STYLE: &str = "flex-grow overflow-y-auto";
const INFO_TITLE_STYLE: &str = "text-stone-400 text-xs";
const INFO_GROUP_STYLE: &str = "mb-2";
const BUTTON_CONTAINER_STYLE: &str =
    "mt-4 pt-4 flex border-t gap-2 justify-end sticky bottom-0 bg-[#F9F9F8]";

#[component]
pub fn AddStudentForm(
    #[prop(into)] set_adding_student: Callback<bool>,
    #[prop(into)] set_refresh_trigger: WriteSignal<i32>,
) -> impl IntoView {
    //Signals for error messaging
    let (error_message, set_error_message) = create_signal(String::new());
    let (if_error, set_if_error) = create_signal(false);

    //Signals for getting a new student
    let (new_firstname, set_new_firstname) = create_signal(String::new());
    let (new_lastname, set_new_lastname) = create_signal(String::new());
    let (new_preferred, set_new_preferred) = create_signal(String::new());
    let (new_student_gender, set_student_gender) = create_signal(String::new());
    let (new_student_dob, set_student_dob) = create_signal(String::new());
    let (new_student_id, set_new_student_id) = create_signal(String::new());
    let (new_grade, set_new_grade) = create_signal(String::new());
    let (new_teacher, set_new_teacher) = create_signal(String::new());
    let (new_iep, set_new_iep) = create_signal(false);
    let (new_bip, set_new_bip) = create_signal(false);
    let (new_504, set_new_504) = create_signal(false);
    let (yes_no_esl, set_yes_no_esl) = create_signal(false);
    let (new_esl, set_new_esl) = create_signal(String::from("Not Applicable"));
    let (new_gt, set_new_gt) = create_signal(false);
    let (new_readplan, set_new_readplan) = create_signal(false);
    let (new_intervention, set_new_intervention) = create_signal(String::new());
    let (new_eye_glasses, set_new_eye_glasses) = create_signal(false);
    let (new_notes, set_new_notes) = create_signal(String::new());
    let (new_pin, set_new_pin) = create_signal(String::new());

    // Create a resource to fetch teachers
    let teachers = create_resource(
        || (),
        |_| async move {
            match crate::app::server_functions::get_teachers().await {
                Ok(teachers) => Some(teachers),
                Err(e) => {
                    log::error!("Failed to fetch teachers: {}", e);
                    Some(vec![])
                }
            }
        },
    );

    // Create a derived signal for filtered teachers based on selected grade
    let filtered_teachers = create_memo(move |_| {
        let grade_str = new_grade();
        if grade_str.is_empty() {
            return Vec::new(); // Return empty if no grade selected yet
        }

        // Convert the selected grade string to GradeEnum
        let selected_grade = match GradeEnum::from_str(&grade_str) {
            Ok(grade) => grade,
            Err(_) => return Vec::new(), // Return empty on error
        };

        // Filter teachers to only include those matching the selected grade
        teachers
            .get()
            .unwrap_or(Some(Vec::new()))
            .unwrap_or_default()
            .into_iter()
            .filter(|teacher| {
                // Check if teacher has EmployeeRole::Teacher{grade} matching selected_grade
                match &teacher.role {
                    EmployeeRole::Teacher { grade } => *grade == Some(selected_grade.clone()),
                    _ => false,
                }
            })
            .collect::<Vec<_>>()
    });

    let handle_submit_new_student = move |ev: SubmitEvent| {
        ev.prevent_default();

        let validated_student_id = new_student_id().parse::<i32>();
        let validated_dob = match NaiveDate::parse_from_str(&new_student_dob(), "%Y-%m-%d") {
            Ok(date) => date,
            Err(e) => {
                log::error!("Error parsing date: {}", e);
                set_if_error(true);
                set_error_message(String::from("Invalid date format"));
                return;
            }
        };

        let validated_pin = match new_pin().parse::<i32>() {
            Ok(pin) => pin,
            Err(_) => {
                set_if_error(true);
                set_error_message(String::from("Invalid pin"));
                return;
            }
        };

        let convert_gender_to_enum = match GenderEnum::from_str(&new_student_gender()) {
            Ok(gender_enum) => gender_enum,
            Err(_) => {
                log::error!("Invalid gender value submitted for new student");
                set_if_error(true);
                set_error_message(String::from("Invalid gender selection"));
                return;
            }
        };

        let convert_grade_to_enum = match GradeEnum::from_str(&new_grade()) {
            Ok(grade_enum) => grade_enum,
            Err(_) => {
                log::error!("Invalid grade value submitted for new student");
                set_if_error(true);
                set_error_message(String::from("Invalid grade selection"));
                return;
            }
        };

        let convert_esl_to_enum = match ESLEnum::from_str(&new_esl()) {
            Ok(esl_enum) => esl_enum,
            Err(_) => {
                log::error!("Invalid ESL value submitted for new student");
                set_if_error(true);
                set_error_message(String::from("Invalid ESL selection"));
                return;
            }
        };

        let convert_intervention_to_enum = if new_intervention() == "None" {
            None
        } else {
            match InterventionEnum::from_str(&new_intervention()) {
                Ok(intervention_enum) => Some(intervention_enum),
                Err(_) => {
                    log::error!("Invalid intervention value for new student");
                    set_if_error(true);
                    set_error_message(String::from("Invalid intervention selection"));
                    return;
                }
            }
        };

        let add_student_request = AddStudentRequest {
            firstname: new_firstname(),
            lastname: new_lastname(),
            preferred: new_preferred(),
            gender: convert_gender_to_enum,
            date_of_birth: validated_dob,
            student_id: match validated_student_id {
                Ok(id) => id,
                Err(_) => {
                    set_if_error(true);
                    set_error_message(String::from("Invalid student ID"));
                    return;
                }
            },
            esl: convert_esl_to_enum,
            current_grade_level: convert_grade_to_enum,
            teacher: new_teacher(),
            iep: new_iep(),
            bip: new_bip(),
            student_504: new_504(),
            readplan: new_readplan(),
            gt: new_gt(),
            intervention: convert_intervention_to_enum,
            eye_glasses: new_eye_glasses(),
            notes: new_notes(),
            pin: validated_pin,
        };

        let is_valid = add_student_request.validate();

        match is_valid {
            Ok(_) => {
                spawn_local(async move {
                    let add_result =
                        crate::app::server_functions::students::add_student(add_student_request)
                            .await;

                    //we get the result back and do something with it
                    match add_result {
                        Ok(_added_result) => {
                            set_adding_student(false);
                            set_refresh_trigger.update(|count| *count += 1);
                            log::info!("Student added successfully");
                        }
                        Err(e) => {
                            log::error!("Error adding student: {:?}", e);
                            set_if_error(true);
                            set_error_message(format!("Error adding student: {}", e));
                        }
                    };
                });
            }
            Err(e) => {
                set_if_error(true);
                set_error_message(format!("Validation error: {:?}", e));
            }
        }
    };

    view! {
        <div class=INFO_CONTAINER_STYLE>
            <Show when=move || if_error()>
                <p class="text-red-500 font-semibold">"There was an error with one or more of the entered fields"</p>
                <p class="text-red-500 rounded w-full h-12 px-5 -y-3">{error_message()}</p>
            </Show>
            <h2 class="text-xl font-bold mb-4">"Add New Student"</h2>
            <form on:submit=handle_submit_new_student class=INFO_CONTENT_STYLE>
                <div class="grid grid-cols-2 gap-4">
                    // Basic Information Section
                    <div class="col-span-2">
                        <h3 class="text-sm font-semibold text-gray-600 mb-2">"Basic Information"</h3>
                        <div class="grid grid-cols-2 gap-4 bg-gray-50 p-4 rounded-lg">
                            <div class=INFO_GROUP_STYLE>
                                <label class=INFO_TITLE_STYLE for="firstname">"First Name"</label>
                                <input
                                    id="firstname"
                                    type="text"
                                    class="mt-1 w-full rounded-md border p-2"
                                    on:input=move |ev| set_new_firstname(event_target_value(&ev))
                                    required
                                />
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class=INFO_TITLE_STYLE for="lastname">"Last Name"</label>
                                <input
                                    id="lastname"
                                    type="text"
                                    class="mt-1 w-full rounded-md border p-2"
                                    on:input=move |ev| set_new_lastname(event_target_value(&ev))
                                    required
                                />
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class=INFO_TITLE_STYLE for="preferred">"Preferred Name"</label>
                                <input
                                    id="preferred"
                                    type="text"
                                    class="mt-1 w-full rounded-md border p-2"
                                    on:input=move |ev| set_new_preferred(event_target_value(&ev))
                                />
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class=INFO_TITLE_STYLE for="student-id">"Student ID"</label>
                                <input
                                    required
                                    id="student-id"
                                    type="text"
                                    class="mt-1 w-full rounded-md border p-2"
                                    on:input=move |ev| set_new_student_id(event_target_value(&ev))
                                />
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class=INFO_TITLE_STYLE for="gender">"Gender"</label>
                                <select
                                    required
                                    id="gender"
                                    class="mt-1 w-full rounded-md border p-2"
                                    on:change=move |ev| set_student_gender(event_target_value(&ev))
                                >
                                    <option value="">"Please select a value"</option>
                                    {GenderEnum::iter().map(|gender| view! {
                                        <option value=format!("{}", gender)>
                                            {format!("{}", gender)}
                                        </option>
                                    }).collect::<Vec<_>>()}
                                </select>
                            </div>

                            <div class=INFO_GROUP_STYLE>
                                <label class=INFO_TITLE_STYLE for="grade">"Grade"</label>
                                <select
                                    required
                                    id="grade"
                                    class="mt-1 w-full rounded-md border p-2"
                                    on:change=move |ev| set_new_grade(event_target_value(&ev))
                                >
                                    <option value="">"Please select a value"</option>
                                    {GradeEnum::iter().map(|grade| view! {
                                        <option value=format!("{}", grade)>
                                            {format!("{}", grade)}
                                        </option>
                                    }).collect::<Vec<_>>()}
                                </select>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class=INFO_TITLE_STYLE for="birthdate">"Birthdate"</label>
                                <input
                                    type="date"
                                    required
                                    id="birthdate"
                                    class="mt-1 w-full rounded-md border p-2"
                                    on:change=move |ev| set_student_dob(event_target_value(&ev))
                                />
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class=INFO_TITLE_STYLE for="pin">"Pin"</label>
                                <input
                                    type="number"
                                    id="pin"
                                    min="0"
                                    class="mt-1 w-full rounded-md border p-2"
                                    on:input=move |ev| set_new_pin(event_target_value(&ev))
                                />
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class=INFO_TITLE_STYLE for="teacher">"Teacher"</label>
                                <select
                                    required
                                    id="teacher"
                                    class="mt-1 w-full rounded-md border p-2"
                                    on:change=move |ev| set_new_teacher(event_target_value(&ev))
                                >
                                    <option value="">"Please select a value"</option>
                                    {move || {
                                        if new_grade().is_empty() {
                                            vec![view! { <option disabled>"First select a grade"</option> }].into_iter().collect_view()
                                        } else {
                                            let filtered = filtered_teachers();
                                            if filtered.is_empty() {
                                                vec![view! { <option disabled>"No teachers available for this grade"</option> }].into_iter().collect_view()
                                            } else {
                                                filtered.iter().map(|teacher| view! {
                                                    <option value=teacher.lastname.clone()>{teacher.lastname.clone()}</option>
                                                }).collect_view()
                                            }
                                        }
                                    }}
                                </select>
                            </div>
                        </div>
                    </div>

                    // Support Services Section
                    <div class="col-span-2">
                        <h3 class="text-sm font-semibold text-gray-600 mb-2">"Support Services"</h3>
                        <div class="grid grid-cols-2 gap-4 bg-gray-50 p-4 rounded-lg">
                            <div class=INFO_GROUP_STYLE>
                                <label class="flex items-center gap-2">
                                    <input
                                        type="checkbox"
                                        class="form-checkbox h-5 w-5"
                                        on:change=move |ev| set_new_iep(event_target_checked(&ev))
                                    />
                                    <span class=INFO_TITLE_STYLE>"IEP"</span>
                                </label>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class="flex items-center gap-2">
                                    <input
                                        type="checkbox"
                                        class="form-checkbox h-5 w-5"
                                        on:change=move |ev| set_new_bip(event_target_checked(&ev))
                                    />
                                    <span class=INFO_TITLE_STYLE>"BIP"</span>
                                </label>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class="flex items-center gap-2">
                                    <input
                                        type="checkbox"
                                        class="form-checkbox h-5 w-5"
                                        on:change=move |ev| set_new_504(event_target_checked(&ev))
                                    />
                                    <span class=INFO_TITLE_STYLE>"504"</span>
                                </label>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class="flex items-center gap-2">
                                    <input
                                        type="checkbox"
                                        class="form-checkbox h-5 w-5"
                                        on:change=move |ev| set_yes_no_esl(event_target_checked(&ev))
                                    />
                                    <span class=INFO_TITLE_STYLE>"ESL"</span>
                                </label>
                                <Show when=move || yes_no_esl()>
                                    <select class="p-3 rounded-lg mt-2 w-full"
                                        required
                                        value=new_esl
                                        on:change=move |event| {
                                            set_new_esl(event_target_value(&event))
                                        }
                                    >
                                        <option value="">"Please Select"</option>
                                        {ESLEnum::iter().map(|lang| view! {
                                            <option value=format!("{}", lang)>
                                                {format!("{}", lang)}
                                            </option>
                                        }).collect::<Vec<_>>()}
                                    </select>
                                </Show>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class="flex items-center gap-2">
                                    <input
                                        type="checkbox"
                                        class="form-checkbox h-5 w-5"
                                        on:change=move |ev| set_new_readplan(event_target_checked(&ev))
                                    />
                                    <span class=INFO_TITLE_STYLE>"Read Plan"</span>
                                </label>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class="flex items-center gap-2">
                                    <input
                                        type="checkbox"
                                        class="form-checkbox h-5 w-5"
                                        on:change=move |ev| set_new_gt(event_target_checked(&ev))
                                    />
                                    <span class=INFO_TITLE_STYLE>"GT Status"</span>
                                </label>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class="flex items-center gap-2">
                                    <select
                                        class="mt-1 w-full rounded-md border p-2"
                                        required
                                        id="intervention"
                                        on:change=move |event| set_new_intervention(event_target_value(&event))
                                    >
                                        <option value="">"Please Select"</option>
                                        <option value="None">"None"</option>
                                        {InterventionEnum::iter().map(|int| view! {
                                            <option value=format!("{}", int)>
                                                {format!("{}", int)}
                                            </option>
                                        }).collect::<Vec<_>>()}
                                    </select>
                                    <span class=INFO_TITLE_STYLE>"Intervention"</span>
                                </label>
                            </div>
                        </div>
                    </div>
                    //Additional Services
                    <div class="col-span-2">
                        <h3 class="text-sm font-semibold text-gray-600 mb-2">"Additional Services"</h3>
                        <div class="grid grid-cols-2 gap-4 bg-gray-50 p-4 rounded-lg">
                            <div class=INFO_GROUP_STYLE>
                                <label class="flex items-center gap-2">
                                    <input
                                        type="checkbox"
                                        class="form-checkbox h-5 w-5"
                                        on:change=move |ev| set_new_eye_glasses(event_target_checked(&ev))
                                    />
                                    <span class=INFO_TITLE_STYLE>"Glasses"</span>
                                </label>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class=INFO_TITLE_STYLE for="notes">"Notes"</label>
                                <textarea
                                    id="notes"
                                    class="mt-1 w-full rounded-md border p-2 h-32"
                                    on:input=move |ev| set_new_notes(event_target_value(&ev))
                                    placeholder="Enter any additional notes about the student..."
                                />
                            </div>
                        </div>
                    </div>
                </div>
                <div class=BUTTON_CONTAINER_STYLE>
                    <button
                        type="button"
                        class="px-4 py-2 bg-gray-200 rounded-lg font-bold hover:bg-gray-300"
                        on:click=move |_| set_adding_student(false)
                    >
                        "Cancel"
                    </button>
                    <button
                        type="submit"
                        class="px-4 py-2 bg-green-500 text-white font-bold rounded-lg hover:bg-[#A8DCAB]"
                    >
                        "Save Student"
                    </button>
                </div>
            </form>
        </div>
    }
}
