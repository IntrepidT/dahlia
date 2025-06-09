use crate::app::models::student::{ESLEnum, GenderEnum, GradeEnum, InterventionEnum, Student};
use crate::app::models::UpdateStudentRequest;
use crate::app::server_functions::students::edit_student;
use leptos::*;
use std::rc::Rc;
use std::str::FromStr;
use strum::IntoEnumIterator;

// Styles - matching add form for consistency
const INFO_CONTAINER_STYLE: &str =
    "h-full p-6 border-t-8 border-[#00356B] shadow-lg rounded-lg flex flex-col";
const INFO_CONTENT_STYLE: &str = "flex-grow overflow-y-auto";
const INFO_TITLE_STYLE: &str = "text-stone-400 text-xs";
const INFO_GROUP_STYLE: &str = "mb-2";
const BUTTON_CONTAINER_STYLE: &str =
    "mt-4 pt-4 flex border-t gap-2 justify-end sticky bottom-0 bg-[#F9F9F8]";

#[component]
pub fn UpdateStudent(
    #[prop()] student: Rc<Student>,
    #[prop(optional)] on_cancel: Option<Callback<()>>,
    #[prop(optional)] on_update_success: Option<Callback<Student>>,
) -> impl IntoView {
    // Create signals for each field
    let (firstname, set_firstname) = create_signal(student.firstname.clone().unwrap());
    let (lastname, set_lastname) = create_signal(student.lastname.clone().unwrap());
    let (preferred, set_preferred) = create_signal(student.preferred.clone());
    let (gender, set_gender) = create_signal(student.gender.clone().to_string());
    let (date_of_birth, set_date_of_birth) = create_signal(student.date_of_birth);
    let (student_id, set_student_id) = create_signal(student.student_id.clone().to_string());
    let (current_grade_level, set_current_grade_level) =
        create_signal(student.current_grade_level.clone().to_string());
    let (teacher, set_teacher) = create_signal(student.teacher.clone());
    let (yes_no_esl, set_yes_no_esl) = if student.esl.to_string() == "Not Applicable" {
        create_signal(false)
    } else {
        create_signal(true)
    };

    let (esl, set_esl) = create_signal(student.esl.to_string());

    let (iep, set_iep) = create_signal(student.iep);
    let (bip, set_bip) = create_signal(student.bip);
    let (student_504, set_student_504) = create_signal(student.student_504);
    let (readplan, set_readplan) = create_signal(student.readplan);
    let (gt, set_gt) = create_signal(student.gt);
    let (intervention_selection, set_intervention_selection) =
        create_signal(match &student.intervention {
            Some(intervention) => intervention.to_string(),
            None => "None".to_string(),
        });

    // Additional information
    let (eye_glasses, set_eye_glasses) = create_signal(student.eye_glasses);
    let (notes, set_notes) = create_signal(student.notes.clone());
    let (pin, set_pin) = create_signal(student.pin.clone().unwrap().to_string());
    // For handling form submission
    let (is_submitting, set_is_submitting) = create_signal(false);
    let (error_message, set_error_message) = create_signal(String::new());
    let (if_error, set_if_error) = create_signal(false);

    // Create a resource to fetch teachers (similar to add form)
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
        let grade_str = current_grade_level();
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
                    crate::app::models::EmployeeRole::Teacher { grade } => {
                        *grade == Some(selected_grade.clone())
                    }
                    _ => false,
                }
            })
            .collect::<Vec<_>>()
    });

    // Handle form submission
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_is_submitting(true);
        set_error_message(String::new());
        set_if_error(false);

        // Parse and validate student ID
        let validated_student_id = match student_id().parse::<i32>() {
            Ok(id) => id,
            Err(_) => {
                set_if_error(true);
                set_error_message(String::from("Invalid student ID"));
                set_is_submitting(false);
                return;
            }
        };

        let validated_pin = match pin().parse::<i32>() {
            Ok(pin) => pin,
            Err(_) => {
                set_if_error(true);
                set_error_message(String::from("Invalid pin"));
                set_is_submitting(false);
                return;
            }
        };

        // Convert gender string to enum
        let convert_gender_to_enum = match GenderEnum::from_str(&gender()) {
            Ok(gender_enum) => gender_enum,
            Err(_) => {
                log::error!("Invalid gender value submitted for update");
                set_if_error(true);
                set_error_message(String::from("Invalid gender selection"));
                set_is_submitting(false);
                return;
            }
        };

        // Convert grade string to enum
        let convert_grade_to_enum = match GradeEnum::from_str(&current_grade_level()) {
            Ok(grade_enum) => grade_enum,
            Err(_) => {
                log::error!("Invalid grade value submitted for update");
                set_if_error(true);
                set_error_message(String::from("Invalid grade selection"));
                set_is_submitting(false);
                return;
            }
        };

        // Convert ELL string to enum
        let convert_esl_to_enum = match ESLEnum::from_str(&esl()) {
            Ok(esl_enum) => esl_enum,
            Err(_) => {
                log::error!("Invalid ELL value submitted for update");
                set_if_error(true);
                set_error_message(String::from("Invalid ELL selection"));
                set_is_submitting(false);
                return;
            }
        };

        // Convert selection_intervention into enum
        let convert_intervention = if intervention_selection() == "None" {
            None
        } else {
            match InterventionEnum::from_str(&intervention_selection()) {
                Ok(intervention_enum) => Some(intervention_enum),
                Err(_) => {
                    log::error!("Invalid intervention value submitted for update");
                    set_if_error(true);
                    set_error_message(String::from("Invalid intervention selection"));
                    set_is_submitting(false);
                    return;
                }
            }
        };

        let update_data = UpdateStudentRequest {
            firstname: firstname(),
            lastname: lastname(),
            preferred: preferred(),
            gender: convert_gender_to_enum,
            date_of_birth: date_of_birth(),
            student_id: validated_student_id,
            esl: convert_esl_to_enum,
            current_grade_level: convert_grade_to_enum,
            teacher: teacher(),
            iep: iep(),
            bip: bip(),
            student_504: student_504(),
            readplan: readplan(),
            gt: gt(),
            intervention: convert_intervention,
            eye_glasses: eye_glasses(),
            notes: notes(),
            pin: validated_pin,
        };

        spawn_local(async move {
            match edit_student(update_data).await {
                Ok(updated_student) => {
                    set_is_submitting(false);
                    if let Some(callback) = on_update_success {
                        callback.call(updated_student);
                    }
                }
                Err(e) => {
                    set_is_submitting(false);
                    set_if_error(true);
                    set_error_message(format!("Failed to update student: {}", e));
                }
            }
        });
    };

    let handle_cancel = move |_| {
        if let Some(callback) = on_cancel {
            callback.call(());
        }
    };

    view! {
        <div class=INFO_CONTAINER_STYLE>
            <Show when=move || if_error()>
                <p class="text-red-500 font-semibold">"There was an error with one or more of the entered fields"</p>
                <p class="text-red-500 rounded w-full h-12 px-5 -y-3">{error_message()}</p>
            </Show>
            <h2 class="text-xl font-bold mb-4">
                "Edit Student: " {move || firstname()} " " {move || lastname()}
            </h2>
            <form on:submit=on_submit class=INFO_CONTENT_STYLE>
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
                                    value={firstname}
                                    on:input=move |ev| set_firstname(event_target_value(&ev))
                                    required
                                />
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class=INFO_TITLE_STYLE for="lastname">"Last Name"</label>
                                <input
                                    id="lastname"
                                    type="text"
                                    class="mt-1 w-full rounded-md border p-2"
                                    value={lastname}
                                    on:input=move |ev| set_lastname(event_target_value(&ev))
                                    required
                                />
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class=INFO_TITLE_STYLE for="preferred">"Preferred Name"</label>
                                <input
                                    id="preferred"
                                    type="text"
                                    class="mt-1 w-full rounded-md border p-2"
                                    value={preferred}
                                    on:input=move |ev| set_preferred(event_target_value(&ev))
                                />
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class=INFO_TITLE_STYLE for="student-id">"Student ID"</label>
                                <input
                                    required
                                    id="student-id"
                                    type="text"
                                    class="mt-1 w-full rounded-md border p-2"
                                    value={student_id}
                                    on:input=move |ev| set_student_id(event_target_value(&ev))
                                    readonly
                                />
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class=INFO_TITLE_STYLE for="gender">"Gender"</label>
                                <select
                                    required
                                    id="gender"
                                    class="mt-1 w-full rounded-md border p-2"
                                    on:change=move |ev| set_gender(event_target_value(&ev))
                                >
                                    <option value="">"Please select a value"</option>
                                    {GenderEnum::iter().map(|g| view! {
                                        <option value=format!("{}", g) selected=g.to_string() == gender()>
                                            {format!("{}", g)}
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
                                    on:change=move |ev| set_current_grade_level(event_target_value(&ev))
                                >
                                    <option value="">"Please select a value"</option>
                                    {GradeEnum::iter().map(|g| view! {
                                        <option value=format!("{}", g) selected=g.to_string() == current_grade_level()>
                                            {format!("{}", g)}
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
                                    value={move || date_of_birth().format("%Y-%m-%d").to_string()}
                                    on:change=move |ev| {
                                        let date_str = event_target_value(&ev);
                                        match chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                                            Ok(parsed_date) => set_date_of_birth(parsed_date),
                                            Err(e) => {
                                                log::error!("Error parsing date: {}", e);
                                                // Keep the original date on error
                                            }
                                        }
                                    }
                                />
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class=INFO_TITLE_STYLE for="pin">"Pin"</label>
                                <input
                                    type="number"
                                    id="birthdate"
                                    class="mt-1 w-full rounded-md border p-2"
                                    value={pin}
                                    on:input=move |ev| set_pin(event_target_value(&ev))
                                />
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class=INFO_TITLE_STYLE for="teacher">"Teacher"</label>
                                <select
                                    required
                                    id="teacher"
                                    class="mt-1 w-full rounded-md border p-2"
                                    on:change=move |ev| set_teacher(event_target_value(&ev))
                                >
                                    <option value="">"Please select a value"</option>
                                    {move || {
                                        if current_grade_level().is_empty() {
                                            vec![view! { <option disabled>"First select a grade"</option> }].into_iter().collect_view()
                                        } else {
                                            let filtered = filtered_teachers();
                                            if filtered.is_empty() {
                                                vec![view! { <option disabled>"No teachers available for this grade"</option> }].into_iter().collect_view()
                                            } else {
                                                let current_teacher = teacher();
                                                filtered.iter().map(|t| view! {
                                                    <option value=t.lastname.clone() selected=t.lastname == current_teacher>
                                                        {t.lastname.clone()}
                                                    </option>
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
                                        checked={move || iep()}
                                        on:change=move |ev| set_iep(event_target_checked(&ev))
                                    />
                                    <span class=INFO_TITLE_STYLE>"IEP"</span>
                                </label>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class="flex items-center gap-2">
                                    <input
                                        type="checkbox"
                                        class="form-checkbox h-5 w-5"
                                        checked={move || bip()}
                                        on:change=move |ev| set_bip(event_target_checked(&ev))
                                    />
                                    <span class=INFO_TITLE_STYLE>"BIP"</span>
                                </label>
                            </div>

                            <div class=INFO_GROUP_STYLE>
                                <label class="flex items-center gap-2">
                                    <input
                                        type="checkbox"
                                        class="form-checkbox h-5 w-5"
                                        checked={move || student_504()}
                                        on:change=move |ev| set_student_504(event_target_checked(&ev))
                                    />
                                    <span class=INFO_TITLE_STYLE>"504"</span>
                                </label>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class="flex items-center gap-2">
                                    <input
                                        type="checkbox"
                                        class="form-checkbox h-5 w-5"
                                        checked={move || yes_no_esl()}
                                        on:change=move |ev| set_yes_no_esl(event_target_checked(&ev))
                                    />
                                    <span class=INFO_TITLE_STYLE>"ESL"</span>
                                </label>
                                <Show when=move || yes_no_esl()>
                                    <select class="p-3 rounded-lg mt-2 w-full"
                                        required
                                        on:change=move |event| {
                                            set_esl(event_target_value(&event))
                                        }
                                    >
                                        <option value="">"Please Select"</option>
                                        {ESLEnum::iter().map(|lang| view! {
                                            <option value=format!("{}", lang) selected=lang.to_string() == esl()>
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
                                        checked={move || readplan()}
                                        on:change=move |ev| set_readplan(event_target_checked(&ev))
                                    />
                                    <span class=INFO_TITLE_STYLE>"Read Plan"</span>
                                </label>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class="flex items-center gap-2">
                                    <input
                                        type="checkbox"
                                        class="form-checkbox h-5 w-5"
                                        checked={move || gt()}
                                        on:change=move |ev| set_gt(event_target_checked(&ev))
                                    />
                                    <span class=INFO_TITLE_STYLE>"GT Status"</span>
                                </label>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <label class="flex items-center gap-2">
                                    <select
                                        required
                                        id="intervention"
                                        class="mt-1 w-full rounded-md p-2"
                                        on:change=move |ev| set_intervention_selection(event_target_value(&ev))
                                    >
                                        <option value="">"Please select a value"</option>
                                        <option value="None" selected={intervention_selection() == "None"}>"None"</option>
                                        {InterventionEnum::iter().map(|intervention| view! {
                                            <option value=format!("{}", intervention) selected=intervention.to_string() == intervention_selection()>
                                                {format!("{}", intervention)}
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
                                        checked={move || eye_glasses()}
                                        on:change=move |ev| set_eye_glasses(event_target_checked(&ev))
                                    />
                                    <span class=INFO_TITLE_STYLE>"Glasses"</span>
                                </label>
                            </div>
                        </div>

                        <h3 class="text-sm font-semibold text-gray-600 mb-2">"Student Notes"</h3>
                        <div class="grid grid-cols-1 gap-4 bg-gray-50 p-4 rounded-lg">
                            <div class=INFO_GROUP_STYLE>
                                <label class=INFO_TITLE_STYLE for="notes">"Notes"</label>
                                <textarea
                                    id="notes"
                                    class="mt-1 w-full rounded-md border p-2 h-32"
                                    prop:value={move || notes()}
                                    on:input=move |ev| set_notes(event_target_value(&ev))
                                    placeholder="Enter any additional notes about the student..."
                                />
                            </div>
                        </div>
                    </div>
                </div>
                <div class=BUTTON_CONTAINER_STYLE>
                    <button
                        type="button"
                        class="px-4 py-2 bg-gray-200 rounded-lg font-md hover:bg-gray-300"
                        on:click=handle_cancel
                        disabled=move || is_submitting()
                    >
                        "Cancel"
                    </button>
                    <button
                        type="submit"
                        class="px-4 py-2 bg-green-500 text-white font-md rounded-lg hover:bg-[#A8DCAB]"
                        disabled=move || is_submitting()
                    >
                        {move || if is_submitting() { "Updating..." } else { "Update Student" }}
                    </button>
                </div>
            </form>
        </div>
    }
}
