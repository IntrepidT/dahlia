use crate::app::models::student::Student;
use leptos::*;
use std::rc::Rc;

// Styles
const INFO_CONTAINER_STYLE: &str =
    "h-full p-6 border-t-8 border-[#00356B] shadow-lg rounded-lg flex flex-col";
const INFO_CONTENT_STYLE: &str = "flex-grow overflow-y-auto";
const INFO_TITLE_STYLE: &str = "text-stone-400 text-xs";
const INFO_VALUE_STYLE: &str = "mt-1";
const INFO_GROUP_STYLE: &str = "mb-2";
const BUTTON_CONTAINER_STYLE: &str =
    "mt-4 pt-4 flex border-t gap-2 justify-end sticky bottom-0 bg-white";

#[component]
pub fn StudentDetails(
    #[prop()] student: Rc<Student>,
    #[prop(optional)] on_edit_student: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class=INFO_CONTAINER_STYLE>
            <h2 class="text-xl font-bold mb-4">
                {&student.firstname}
                {" "}
                {&student.lastname}
            </h2>

            <div class=INFO_CONTENT_STYLE>
                <div class="grid grid-cols-2 gap-4">
                    // Basic Information Section
                    <div class="col-span-2">
                        <h3 class="text-sm font-semibold text-gray-600 mb-2">"Basic Information"</h3>
                        <div class="grid grid-cols-2 gap-4 bg-gray-50 p-4 rounded-lg">
                            <div class=INFO_GROUP_STYLE>
                                <div class=INFO_TITLE_STYLE>"Student ID"</div>
                                <div class=INFO_VALUE_STYLE>{format!("{}", &student.student_id)}</div>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <div class=INFO_TITLE_STYLE>"Grade"</div>
                                <div class=INFO_VALUE_STYLE>{&student.grade.to_string()}</div>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <div class=INFO_TITLE_STYLE>"Teacher"</div>
                                <div class=INFO_VALUE_STYLE>{&student.teacher}</div>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <div class=INFO_TITLE_STYLE>"Date of Birth"</div>
                                <div class=INFO_VALUE_STYLE>{format!("{:?}", &student.date_of_birth)}</div>
                            </div>
                        </div>
                    </div>

                    // Support Services Section
                    <div class="col-span-2">
                        <h3 class="text-sm font-semibold text-gray-600 mb-2">"Support Services"</h3>
                        <div class="grid grid-cols-2 gap-4 bg-gray-50 p-4 rounded-lg">
                            <div class=INFO_GROUP_STYLE>
                                <div class=INFO_TITLE_STYLE>"IEP Status"</div>
                                <div class=INFO_VALUE_STYLE>{&student.iep.to_string()}</div>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <div class=INFO_TITLE_STYLE>"504 Status"</div>
                                <div class=INFO_VALUE_STYLE>{&student.student_504.to_string()}</div>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <div class=INFO_TITLE_STYLE>"ELL Status"</div>
                                <div class=INFO_VALUE_STYLE>{&student.ell.to_string()}</div>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <div class=INFO_TITLE_STYLE>"GT Status"</div>
                                <div class=INFO_VALUE_STYLE>{&student.gt.to_string()}</div>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <div class=INFO_TITLE_STYLE>"Readplan"</div>
                                <div class=INFO_VALUE_STYLE>{&student.readplan.to_string()}</div>
                            </div>
                            <div class=INFO_GROUP_STYLE>
                                <div class=INFO_TITLE_STYLE>"Intervention"</div>
                                <div class=INFO_VALUE_STYLE>{&student.intervention.to_string()}</div>
                            </div>
                        </div>
                    </div>

                    // Additional Information Section
                    <div class="col-span-2">
                        <h3 class="text-sm font-semibold text-gray-600 mb-2">"Additional Information"</h3>
                        <div class="grid grid-cols-2 gap-4 bg-gray-50 p-4 rounded-lg">
                            <div class=INFO_GROUP_STYLE>
                                <div class=INFO_TITLE_STYLE>"Eye Glasses"</div>
                                <div class=INFO_VALUE_STYLE>{&student.eye_glasses.to_string()}</div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            // Button container at the bottom
            <div class=BUTTON_CONTAINER_STYLE>
                <button class="px-4 py-2 bg-[#00356b] text-white rounded-lg font-bold hover:bg-[#7F9AB5]">
                    "Test Results"
                </button>
                <button
                    class="px-4 py-2 bg-[#FDF8D4] text-black rounded-lg border-2 border-gray-50 font-bold hover:bg-[#FCFDD4] hover:border-2 hover:border-gray-50"
                    on:click=move |_| {
                        if let Some(callback) = on_edit_student {
                            callback.call(());
                        }
                    }
                >
                    "Edit Student"
                </button>
                <button class="px-4 py-2 bg-gray-200 border rounded-lg font-bold hover:bg-gray-100">
                    "Next Student"
                </button>
            </div>
        </div>
    }
}
