use crate::app::components::{Toast, ToastMessage, ToastMessageType};
use crate::app::models::student::{AddStudentRequest, ESLEnum, GenderEnum, GradeEnum};
use crate::app::server_functions::students::add_student;
use chrono::NaiveDate;
use leptos::*;
use std::str::FromStr;
use strum::IntoEnumIterator;
use validator::Validate;

#[component]
pub fn AddStudentModal(
    set_if_show_modal: WriteSignal<bool>,
    set_if_show_added: WriteSignal<bool>,
    set_toast_message: WriteSignal<ToastMessage>,
    refresh_students: RwSignal<()>,
) -> impl IntoView {
    const INPUT_STYLE: &str = "w-full h-12 bg-[#333333] pr-4 pl-6 py-4 text-white mt-6 outline-none focus:outline-none focus:pl-7 transition-all duration-1000 ease-in-out rounded";

    const CANCEL_BUTTON_STYLE: &str = "mt-10 bg-[#555555] px-8 py-2 rounded text-white mr-3 transition-all duration-1000 ease-in-out hover:bg-[#666666]";

    const ADD_BUTTON_STYLE: &str = "mt-10 bg-[#00356B] px-8 py-2 rounded text-white transition-all duration-1000 ease-in-out hover:bg-[#FFB6C1]";

    const NO_ERROR_STYLE: &str = "flex flex-col bg-[#222222] border-t-8 border-[#00356B] px-6 pt-5 h-[40rem] w-full max-w-[80rem] z-50 -mt-2 fixed z-50 rounded";

    const ERROR_STYLE: &str = "flex flex-col bg-[#222222] border-t-8 border-[#00356B] px-6 pt-5 h-[40rem] w-full max-w-[80rem] z-50 -mt-2 fixed z-50 rounded-2xl";

    //create and send signals for various data
    let (student_firstname, set_student_firstname) = create_signal(String::new());
    let (student_lastname, set_student_lastname) = create_signal(String::new());
    let (student_gender, set_student_gender) = create_signal(String::new());
    let (student_dob, set_student_dob) = create_signal(String::new());
    let (student_id, set_student_id) = create_signal(String::new());
    let (student_esl, set_student_esl) = create_signal(String::new());
    let (student_grade, set_student_grade) = create_signal(String::new());
    let (student_teacher, set_student_teacher) = create_signal(String::new());
    let (student_iep, set_student_iep) = create_signal(String::new());
    let (student_504, set_student_504) = create_signal(String::new());
    let (student_readplan, set_student_readplan) = create_signal(String::new());
    let (student_gt, set_student_gt) = create_signal(String::new());
    let (student_intervention, set_student_intervention) = create_signal(String::new());
    let (student_eye_glasses, set_student_eye_glasses) = create_signal(String::new());
    //create and send signals for error messages
    let (error_message, set_error_message) = create_signal(String::new());
    let (if_error, set_if_error) = create_signal(false);
    //close the modal
    let on_close = move |_| {
        set_if_show_modal(false);
    };
    //add a new person to the modal
    let on_click = move |_| {
        let validated_student_id = student_id().parse::<i32>();
        let validated_dob: NaiveDate =
            NaiveDate::parse_from_str(&student_dob(), "%Y-%m-%d").expect("Issue gathering date");

        let convert_gender_to_enum = match GenderEnum::from_str(&student_gender()) {
            Ok(gender_enum) => gender_enum,
            Err(_) => {
                set_if_error(true);
                set_error_message(format!("Invalid gender selected: {:?}", student_gender()));
                return;
            }
        };
        let convert_grade_to_enum = match GradeEnum::from_str(&student_grade()) {
            Ok(grade_enum) => grade_enum,
            Err(_) => {
                set_if_error(true);
                set_error_message(format!("Invalid grade selected: {:?}", student_grade()));
                return;
            }
        };
        let convert_esl_to_enum = ESLEnum::from_str(&student_esl()).clone().unwrap();

        let add_student_request = AddStudentRequest::new(
            student_firstname(),
            student_lastname(),
            convert_gender_to_enum,
            validated_dob,
            validated_student_id.expect("Student ID was not processed correctly"),
            convert_esl_to_enum,
            convert_grade_to_enum,
            student_teacher(),
            student_iep().parse().unwrap(),
            student_504().parse().unwrap(),
            student_readplan().parse().unwrap(),
            student_gt().parse().unwrap(),
            student_intervention().parse().unwrap(),
            student_eye_glasses().parse().unwrap(),
        );

        let is_valid = add_student_request.validate();

        match is_valid {
            Ok(_) => {
                spawn_local(async move {
                    let add_result = add_student(add_student_request).await;

                    //we get the result back and do something with it
                    match add_result {
                        Ok(_added_student) => {
                            set_if_show_modal(false);

                            set_toast_message(ToastMessage::create(
                                ToastMessageType::NewStudentAdded,
                            ));

                            //setting this to true will make the Toast
                            //"new member added" appear
                            set_if_show_added(true);
                            refresh_students.set(());
                        }
                        Err(e) => println!("Error adding: {:?}", e),
                    };
                });
            }
            Err(_) => {
                set_if_error(true);
                set_error_message(String::from("All fields are required"))
            }
        }
    };

    view! {
        <div class="flex flex-col w-full h-full z-50 mx-auto items-center align-center rounded-2xl">
            <div class={move || {
                if if_error() {ERROR_STYLE}
                else {NO_ERROR_STYLE}
            }}>
                <Show when=move || {if_error() }>
                    <p class="text-white bg-red-500 rounded w-full h-12 px-5 py-3 transition-all duration-750 ease-in-out">
                    {error_message()}
                    </p>
                </Show>
                <p class="text-white text-xl font-bold pt-5">"Add New Student"</p>
                <div class="grid grid-cols-3 gap-x-4 gap-y-4">
                    <input type="text" placeholder="First Name" required class=INPUT_STYLE
                        value=student_firstname
                        on:input=move |event| {
                            set_student_firstname(event_target_value(&event));
                        }
                    />
                    <input type="text" placeholder="Last Name" required class=INPUT_STYLE
                        value=student_lastname
                        on:input=move |event| {
                            set_student_lastname(event_target_value(&event));
                        }
                    />
                    <select required class=INPUT_STYLE
                        value=student_gender
                        on:change=move |event| {
                            set_student_gender(event_target_value(&event));
                        }
                    >
                        <option value="">"Select Gender"</option>
                        {GenderEnum::iter().map(|gender| view! {
                            <option value=format!("{}", gender)>
                                {format!("{}", gender)}
                            </option>
                        }).collect::<Vec<_>>()}
                    </select>
                    <input type="date" class=INPUT_STYLE
                        value=student_dob
                        on:input=move |event| {
                            set_student_dob(event_target_value(&event))
                        }
                    />
                    <input type="text" placeholder="Student ID" class=INPUT_STYLE
                        value=student_id
                        on:input=move |event| {
                            set_student_id(event_target_value(&event));
                        }
                    />
                    <select required class=INPUT_STYLE
                        value=student_esl
                        on:change=move |event| {
                            set_student_esl(event_target_value(&event));
                        }
                    >
                        <option value="">"Select A Value for ESL"</option>
                        {ESLEnum::iter().map(|lang| view! {
                            <option value=format!("{}", lang)>
                                {format!("{}", lang)}
                            </option>
                        }).collect::<Vec<_>>()}
                    </select>
                    <select required class=INPUT_STYLE
                        value=student_grade
                        on:input=move |event| {
                            set_student_grade(event_target_value(&event));
                        }
                    >
                        <option value="">"Select a Grade for Student"</option>
                        {GradeEnum::iter().map(|grade| view! {
                            <option value=format!("{}", grade)>
                                {format!("{}", grade)}
                            </option>
                        }).collect::<Vec<_>>()}
                    </select>
                    <input type="text" placeholder="Teacher" class=INPUT_STYLE
                        value=student_teacher
                        on:input=move |event| {
                            set_student_teacher(event_target_value(&event));
                        }
                    />
                    <select required class=INPUT_STYLE
                        value=student_iep
                        on:change=move |event| {
                            set_student_iep(event_target_value(&event));
                        }
                    >
                        <option value="">"Student IEP?"</option>
                        <option value="true">"Yes"</option>
                        <option value="false">"No"</option>
                    </select>
                    <select required class=INPUT_STYLE
                        value=student_504
                        on:input=move |event| {
                            set_student_504(event_target_value(&event));
                        }
                    >
                        <option value="">"Student 504?"</option>
                        <option value="true">"Yes"</option>
                        <option value="false">"No"</option>
                    </select>
                    <select required class=INPUT_STYLE
                        value=student_readplan
                        on:input=move |event| {
                            set_student_readplan(event_target_value(&event));
                        }
                    >
                        <option value="">"Student Readplan?"</option>
                        <option value="true">"Yes"</option>
                        <option value="false">"No"</option>
                    </select>
                    <select required class=INPUT_STYLE
                        value=student_gt
                        on:input=move |event| {
                            set_student_gt(event_target_value(&event));
                        }
                    >
                        <option value="">"GT status"</option>
                        <option value="true">"Yes"</option>
                        <option value="false">"No"</option>
                    </select>
                    <select required class=INPUT_STYLE
                        value=student_intervention
                        on:input=move |event| {
                            set_student_intervention(event_target_value(&event));
                        }
                    >
                        <option value="">"Intervention for Student?"</option>
                        <option value="true">"Yes"</option>
                        <option value="false">"No"</option>
                    </select>
                    <select required class=INPUT_STYLE
                        value=student_eye_glasses
                        on:input=move |event| {
                            set_student_eye_glasses(event_target_value(&event));
                        }
                    >
                        <option value="">"Glasses?"</option>
                        <option value="true">"Yes"</option>
                        <option value="false">"No"</option>
                    </select>
                </div>
               <div class="flex flex-row w-full items-right justify-right">
                    <button on:click=on_close class=CANCEL_BUTTON_STYLE>
                        "Cancel"
                    </button>
                    <button on:click=on_click class=ADD_BUTTON_STYLE>
                        "Add"
                    </button>
               </div>
            </div>
        </div>
    }
}
