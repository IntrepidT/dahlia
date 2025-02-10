use crate::app::components::{Toast, ToastMessage, ToastMessageType};
use crate::app::models::{DeleteStudentRequest, Student};
use crate::app::server_functions::students::delete_student;
use leptos::*;
use std::rc::Rc;

//styles for fields
const INFO_STYLE: &str = "w-full h-12 pr-4 py-4 mt-6 flex flex-col outline-none focus:outline-none focus:pl-7 transition-all duration-1000 ease-in-out";

const INFO_TITLE_STYLE: &str = "text-stone-400 text-xs";
const INFO_VALUE_STYLE: &str = "text-white";

const CLOSE_BUTTON_STYLE: &str = "mt-10 bg-[#555555] px-8 py-2 rounded text-white mr-3 transition-all duration-1000 ease-in-out hover:bg-[#666666]";

const DELETE_BUTTON_STYLE: &str = "mt-10 bg-red-800 px-8 py-2 rounded text-white transition-all duration-1000 ease-in-out hover:bg-red-600";

const MODAL_STYLE: &str = "flex flex-col ml-10 bg-[#222222] border-t-8 border-[#00356B] px-10 pt-5 h-[40rem] w-full max-w-[80rem] z-50 -mt-2 fixed px-10 top-20 z-50 rounded-xl";

#[component]
pub fn ShowStudentModal(
    student: Rc<Student>,
    set_if_show_modal: WriteSignal<bool>,
    set_if_show_deleted: WriteSignal<bool>,
    student_resource: Resource<(), Result<Vec<Student>, ServerFnError>>,
    set_toast_message: WriteSignal<ToastMessage>,
) -> impl IntoView {
    let this_student = student.clone();

    //to close the MODAL_STYLE
    let on_close = move |_| {
        set_if_show_modal(false);
    };

    //to perform the deletion
    let on_click_delete = move |_| {
        let delete_student_request = DeleteStudentRequest::new(
            this_student.firstname.clone(),
            this_student.lastname.clone(),
            this_student.student_id,
        );

        spawn_local(async move {
            let delete_result = delete_student(delete_student_request).await;

            match delete_result {
                Ok(_deleted_student) => {
                    student_resource.refetch();

                    set_toast_message(ToastMessage::create(ToastMessageType::StudentDeleted));

                    set_if_show_deleted(true);

                    set_if_show_modal(false);
                }
                Err(e) => println!("Error deleting = {:?}", e),
            };
        });
    };

    view! {
        <div class="flex flex-col w-full h-full z-49 bag-[#222222/[.06]] rounded-2xl">

            <div class="flex flex-col w-full h-full z-50 mx-auto items-center align-center">

                <div class=MODAL_STYLE>

                    <p class="text-white pt-5 text-4xl font-bold mb-2 mt-2">
                        {&student.firstname}
                        {" "}
                        {&student.lastname}
                    </p>

                    <div class="grid grid-cols-3 gap-x-4 gap-y-4">
                        <div class=INFO_STYLE>
                            <div class=INFO_TITLE_STYLE>"Gender"</div>
                            <div class=INFO_VALUE_STYLE>{&student.gender.to_string()}</div>
                        </div>

                        <div class=INFO_STYLE>
                            <div class=INFO_TITLE_STYLE>"Date of Birth"</div>
                            <div class=INFO_VALUE_STYLE>{format!("{:?}", &student.date_of_birth)}
                            </div>
                        </div>
                        <div class=INFO_STYLE>
                            <div class=INFO_TITLE_STYLE>"Student ID"</div>
                            <div class=INFO_VALUE_STYLE>{format!("#{:?}", &student.student_id)}
                            </div>
                        </div>
                        <div class=INFO_STYLE>
                            <div class=INFO_TITLE_STYLE>"ELL Status"</div>
                            <div class=INFO_VALUE_STYLE>{format!("{}", &student.ell.to_string())}
                            </div>
                        </div>
                        <div class=INFO_STYLE>
                            <div class=INFO_TITLE_STYLE>"Grade"</div>
                            <div class=INFO_VALUE_STYLE>{format!("{}", &student.grade.to_string())}
                            </div>
                        </div>
                        <div class=INFO_STYLE>
                            <div class=INFO_TITLE_STYLE>"Teacher"</div>
                            <div class=INFO_VALUE_STYLE>{format!("{}", &student.teacher)}
                            </div>
                        </div>
                        <div class=INFO_STYLE>
                            <div class=INFO_TITLE_STYLE>"IEP Status"</div>
                            <div class=INFO_VALUE_STYLE>{format!("{:?}", &student.iep)}
                            </div>
                        </div>
                        <div class=INFO_STYLE>
                            <div class=INFO_TITLE_STYLE>"Student 504 Status"</div>
                            <div class=INFO_VALUE_STYLE>{format!("{:?}", &student.student_504)}
                            </div>
                        </div>
                        <div class=INFO_STYLE>
                            <div class=INFO_TITLE_STYLE>"Student Readplan Status"</div>
                            <div class=INFO_VALUE_STYLE>{format!("{:?}", &student.readplan)}
                            </div>
                        </div>
                        <div class=INFO_STYLE>
                            <div class=INFO_TITLE_STYLE>"GT Status"</div>
                            <div class=INFO_VALUE_STYLE>{format!("{:?}", &student.gt)}
                            </div>
                        </div>
                        <div class=INFO_STYLE>
                            <div class=INFO_TITLE_STYLE>"Intervention Status"</div>
                            <div class=INFO_VALUE_STYLE>{format!("{:?}", &student.intervention)}
                            </div>
                        </div>
                        <div class=INFO_STYLE>
                            <div class=INFO_TITLE_STYLE>"Eye Glasses"</div>
                            <div class=INFO_VALUE_STYLE>{format!("{:?}", &student.eye_glasses)}
                            </div>
                        </div>
                    </div>

                    <div class="flex flex-row w-full items-right justify-right mt-3">
                        <button on:click=on_close class=CLOSE_BUTTON_STYLE>
                            "Close"
                        </button>
                        <button on:click=on_click_delete class=DELETE_BUTTON_STYLE>
                            "Delete"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
