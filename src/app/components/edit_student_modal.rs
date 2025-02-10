use crate::app::components::{Toast, ToastMessage, ToastMessageType};
use crate::app::models::student::{ELLEnum, GenderEnum, GradeEnum};
use crate::app::models::{Student, UpdateStudentRequest};
use crate::app::server_functions::students::edit_student;
use chrono::NaiveDate;
use leptos::*;
use std::rc::Rc;
use std::str::FromStr;
use strum::IntoEnumIterator;
use validator::Validate;

const INPUT_STYLE: &str = "w-full h-12 bg-[#333333] pr-4 pl-6 py-4 text-white mt-6 outline-none focus:outline-non focus:pl-7 transition-all duration-1000 ease-in-out";

const INFO_STYLE: &str = "h-12 pr-4 py-4 mt-4 text-white outline-none focus:outline-none focus:pl-7 transition-all duration-1000 ease-in-out";

const INFO_TITLE_STYLE: &str = "text-stone-400 text-xs";

const CANCEL_BUTTON_STYLE: &str = "mt-10 bg-[#555555] px-8 py-2 rounded text-white mr-4 transition-all duration-1000 ease-in-out hover:bg-[#666666]";

const UPDATE_BUTTON_STYLE: &str = "mt-10 bg-[#00356b] px-8 py-2 rounded text-white transition-all duration-1000 ease-in-out hover:bg-[#8448e9]";

const NO_ERROR_STYLE:&str = "flex flex-col bg-[#222222] border-t-8 border-[#00356b] px-6 pt-5 h-[40rem] w-full max-w-[80rem] z-50 -mt-2 fixed top-20 z-50";

const ERROR_STYLE: &str = "flex flex-col bg-[#222222] border-t-8 border-[#00356b] px-6 pt-5 h-[40rem] w-full max-w-[80rem] z-50 -mt-2 fixed top-20 z-50";

#[component]
pub fn EditStudentModal(
    student: Rc<Student>,
    set_if_show_modal: WriteSignal<bool>,
    set_if_show_toast: WriteSignal<bool>,
    set_toast_message: WriteSignal<ToastMessage>,
    student_resource: Resource<(), Result<Vec<Student>, ServerFnError>>,
) -> impl IntoView {
    //define the signals for each of the fields that will need to be collected
    let (student_firstname, set_student_firstname) = create_signal(student.firstname.clone());
    let (student_lastname, set_student_lastname) = create_signal(student.lastname.clone());
    let (student_gender, set_student_gender) = create_signal(student.gender.to_string());
    let (student_dob, set_student_dob) = create_signal(student.date_of_birth.to_string());
    let (student_id, set_student_id) = create_signal(format!("{}", student.student_id));
    let (student_ell, set_student_ell) = create_signal(student.ell.to_string());
    let (student_grade, set_student_grade) = create_signal(student.grade.to_string());
    let (student_teacher, set_student_teacher) = create_signal(student.teacher.clone());
    let (student_iep, set_student_iep) = create_signal(student.iep.to_string());
    let (student_504, set_student_504) = create_signal(student.student_504.to_string());
    let (student_readplan, set_student_readplan) = create_signal(student.readplan.to_string());
    let (student_gt, set_student_gt) = create_signal(student.gt.to_string());
    let (student_intervention, set_student_intervention) =
        create_signal(student.intervention.to_string());
    let (student_eye_glasses, set_student_eye_glasses) =
        create_signal(student.eye_glasses.to_string());

    // for errors
    let (error_message, set_error_message) = create_signal(String::new());
    let (if_error, set_if_error) = create_signal(false);
    //to close the modal if needed
    let on_close = move |_| {
        set_if_show_modal(false);
    };

    let on_click = move |_| {
        let validated_student_id = student_id().parse::<i32>();
        let validated_dob: NaiveDate =
            NaiveDate::parse_from_str(&student_dob(), "%Y-%m-%d").expect("Issue gathering date");
        let convert_gender_to_enum = GenderEnum::from_str(&student_gender()).clone().unwrap();
        let convert_grade_to_enum = GradeEnum::from_str(&student_grade()).clone().unwrap();
        let convert_ell_to_enum = ELLEnum::from_str(&student_ell()).clone().unwrap();

        if let Ok(ok_student_id) = validated_student_id {
            let edit_student_request = UpdateStudentRequest::new(
                student_firstname(),
                student_lastname(),
                convert_gender_to_enum,
                validated_dob,
                ok_student_id,
                convert_ell_to_enum,
                convert_grade_to_enum,
                student_teacher(),
                student_iep().parse().unwrap(),
                student_504().parse().unwrap(),
                student_readplan().parse().unwrap(),
                student_gt().parse().unwrap(),
                student_intervention().parse().unwrap(),
                student_eye_glasses().parse().unwrap(),
            );

            let is_valid = edit_student_request.validate();

            match is_valid {
                Ok(_) => {
                    let _ = spawn_local(async move {
                        let edit_result = edit_student(edit_student_request).await;

                        match edit_result {
                            Ok(_edited_student) => {
                                student_resource.refetch();

                                set_if_show_modal(false);

                                set_toast_message(ToastMessage::create(
                                    ToastMessageType::StudentUpdated,
                                ));

                                set_if_show_toast(true);
                            }
                            Err(_e) => {
                                set_if_error(true);
                                set_error_message(String::from(
                                    "Error Updating Student. Please try again later",
                                ))
                            }
                        };
                    });
                }
                Err(_e) => {
                    set_if_error(true);
                    set_error_message(String::from("All fields are required"))
                }
            }
        } else {
            set_if_error(true);
            set_error_message(String::from("student_id should be numeric"))
        }
    };
    view! {
        <div class="flex flex-col w-full h-full z-50 mx-auto items-center align-center">

            <div class={move || {
                if if_error() {ERROR_STYLE}
                else {NO_ERROR_STYLE}
            }}>
                <Show when=move || {if_error()}>
                    <p class="text-white bg-red-500 rounded w-full h-12 px-5 py-3 transition-all duration-750 ease-in-out">
                        {error_message()}
                    </p>
                </Show>
                <p class="text-white font-bold text-3xl mt-2">Modify Existing Test</p>
                <p class="text-white pt-5 text-4xl mb-10">{student_firstname}</p>
                <div class="grid grid-cols-3 gap-x-4 gap-y-4">
                    <input type="text" placeholder="First Name" class=INPUT_STYLE
                        value=student_firstname
                        on:input=move |event| {
                            set_student_firstname(event_target_value(&event));
                        }
                    />
                    <input type="text" placeholder="Last Name" class=INPUT_STYLE
                        value=student_lastname
                        on:input=move |event| {
                            set_student_lastname(event_target_value(&event));
                        }
                    />
                    <select class=INPUT_STYLE
                        value=student_gender
                        on:change=move |event| {
                            set_student_gender(event_target_value(&event));
                        }
                    >
                        {GenderEnum::iter().map(|gender| view! {
                            <option value=format!("{:?}",gender)>
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
                    <select class=INPUT_STYLE
                        value=student_ell
                        on:change=move |event| {
                            set_student_ell(event_target_value(&event));
                        }
                    >
                        {ELLEnum::iter().map(|lang| view! {
                            <option value=format!("{:?}", lang)>
                                {format!("{}", lang)}
                            </option>
                        }).collect::<Vec<_>>()}
                    </select>
                    <select class=INPUT_STYLE
                        value=student_grade
                        on:input=move |event| {
                            set_student_grade(event_target_value(&event));
                        }
                    >
                        {GradeEnum::iter().map(|grade| view! {
                            <option value=format!("{:?}", grade)>
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
                    <select class=INPUT_STYLE
                        value=student_iep
                        on:change=move |event| {
                            set_student_iep(event_target_value(&event));
                        }
                    >
                        <option value="true">"Yes"</option>
                        <option value="false">"No"</option>
                    </select>
                    <select class=INPUT_STYLE
                        value=student_504
                        on:input=move |event| {
                            set_student_504(event_target_value(&event));
                        }
                    >
                        <option value="true">"Yes"</option>
                        <option value="false">"No"</option>
                    </select>
                    <select class=INPUT_STYLE
                        value=student_readplan
                        on:input=move |event| {
                            set_student_readplan(event_target_value(&event));
                        }
                    >
                        <option value="true">"Yes"</option>
                        <option value="false">"No"</option>
                    </select>
                    <select class=INPUT_STYLE
                        value=student_gt
                        on:input=move |event| {
                            set_student_gt(event_target_value(&event));
                        }
                    >
                        <option value="true">"Yes"</option>
                        <option value="false">"No"</option>
                    </select>
                    <select class=INPUT_STYLE
                        value=student_intervention
                        on:input=move |event| {
                            set_student_intervention(event_target_value(&event));
                        }
                    >
                        <option value="true">"Yes"</option>
                        <option value="false">"No"</option>
                    </select>
                    <select class=INPUT_STYLE
                        value=student_eye_glasses
                        on:input=move |event| {
                            set_student_eye_glasses(event_target_value(&event));
                        }
                    >
                        <option value="true">"Yes"</option>
                        <option value="false">"No"</option>
                    </select>
                </div>
                <div class="flex flex-row w-full items-right justify-right mt-3">
                    <button on:click=on_close class=CANCEL_BUTTON_STYLE>
                        "Cancel"
                    </button>
                    <button on:click=on_click class=UPDATE_BUTTON_STYLE>
                        "Update"
                    </button>
                </div>
            </div>
        </div>
    }
}
