use crate::app::components::{AddStudentModal, Header, StudentRow, Toast, ToastMessage};
use crate::app::server_functions::get_students;
use leptos::*;
use std::rc::Rc;

#[component]
pub fn DataView() -> impl IntoView {
    const ADD_BUTTON_STYLE: &str = "bg-[#00356b] px-8 py-2 rounded text-white transition-all duration-1000 ease-in-out hover:bg-black";

    let (if_show_modal, set_if_show_modal) = create_signal(false);

    let (if_show_toast, set_if_show_toast) = create_signal(false);
    let (toast_message, set_toast_message) = create_signal(ToastMessage::new());

    let refresh_trigger = create_rw_signal(());

    let students = create_resource(
        move || refresh_trigger.get(),
        |_| async move { get_students().await },
    );

    let on_click = move |_| {
        set_if_show_modal(!if_show_modal());
    };

    view! {
         <Header />
        <div class="text-white w-full max-w-[64rem] mx-auto items-center justify-center align-center">
            <Toast
                toast_message
                if_appear=if_show_toast
                set_if_appear = set_if_show_toast
            />

            <div class="mt-20 rounded">
                <div class="text-[#00356B] flex flex-col w-full mx-auto items-center justify-center z-25">
                    <Show when=move || {if_show_modal()}>
                        <AddStudentModal
                            set_if_show_modal
                            set_if_show_added=set_if_show_toast
                            set_toast_message
                            refresh_students=refresh_trigger
                        />
                    </Show>

                    <div class="flex flex-row w-full max-w-[52rem]">

                        <div class="pr-4 mt-4 text-2xl font-bold">"Class"</div>
                        <hr class="w-full max-w-[48rem] pl-4 pr-4 pt-4 mt-8 mr-4 text-[#00356b]" />
                        <button on:click=on_click class=ADD_BUTTON_STYLE>
                            "Add"
                        </button>
                    </div>

                    <Suspense fallback=move || {
                        view!{<p>"Loading..."</p>}
                    }>
                        <div class="flex flex-col w-full max-w-[52rem] mt-6">
                            {
                                move || {
                                    students.get().map(|result| {

                                        match result {
                                            Ok(students_data) => {
                                                students_data.iter().map(|each_student| view!{
                                                    <StudentRow
                                                        student=Rc::new(each_student.clone())
                                                        student_resource=students
                                                        set_if_show_toast
                                                        set_toast_message
                                                        refresh_students=refresh_trigger
                                                    />
                                                }).collect_view()
                                            },
                                            Err(_) => {
                                                view!{<div>"Error has occurred"</div>}.into_view()
                                            }
                                        }
                                    })
                                }
                            }
                        </div>
                    </Suspense>
                </div>
            </div>
        </div>
    }
}
