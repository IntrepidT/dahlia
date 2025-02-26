use crate::app::models::student::Student;
use leptos::*;
use std::rc::Rc;

// Styles
const TABLE_STYLE: &str = "w-full table-fixed bg-white shadow-lg rounded-lg";
const TABLE_HEADER_STYLE: &str = "bg-[#00356b] text-white text-left p-4";
const ROW_BASE_STYLE: &str = "border-b hover:bg-gray-50 cursor-pointer h-16";
const ROW_SELECTED_STYLE: &str = "border-b bg-[#FDFBD4] h-16";
const CELL_STYLE: &str = "text-left p-4 truncate";

#[component]
pub fn StudentTable(#[prop(into)] students: Resource<i32, Option<Vec<Student>>>) -> impl IntoView {
    view! {
        <div class="h-[calc(100vh-10rem)] overflow-auto rounded-lg border-b">
            <table class=TABLE_STYLE>
                <thead class="bg-[#00356b] text-white sticky top-0">
                    <tr>
                        <th class=TABLE_HEADER_STYLE style="width: 15%">"First Name"</th>
                        <th class=TABLE_HEADER_STYLE style="width: 15%">"Last Name"</th>
                        <th class=TABLE_HEADER_STYLE style="width: 15%">"ID"</th>
                        <th class=TABLE_HEADER_STYLE style="width: 15%">"Grade"</th>
                        <th class=TABLE_HEADER_STYLE style="width: 15%">"Teacher"</th>
                        <th class=TABLE_HEADER_STYLE style="width: 15%">"IEP"</th>
                    </tr>
                </thead>
                <Suspense fallback=move || view! { <tr><td colspan="6" class="text-center p-4">"Loading..."</td></tr> }>
                    <tbody>
                        {move || {
                            filtered_students().map(|students_opt| {
                                students_opt.map(|students| {
                                    students.into_iter().map(|student| {
                                        let student_rc = Rc::new(student.clone());
                                        let student_cmp = Rc::new(student.clone());
                                        let is_selected = move || selected_student() == Some(student_cmp.clone());

                                        view! {
                                            <tr
                                                class=move || if is_selected() { ROW_SELECTED_STYLE } else { ROW_BASE_STYLE }
                                                on:click=move |_| set_selected_student(Some(student_rc.clone()))
                                            >
                                                <td class=CELL_STYLE>{&student.firstname}</td>
                                                <td class=CELL_STYLE>{&student.lastname}</td>
                                                <td class=CELL_STYLE>{&student.student_id.to_string()}</td>
                                                <td class=CELL_STYLE>{&student.grade.to_string()}</td>
                                                <td class=CELL_STYLE>{&student.teacher.to_string()}</td>
                                                <td class=CELL_STYLE>{&student.iep.to_string()}</td>
                                            </tr>
                                        }
                                    }).collect_view()
                                })
                            })
                        }}
                    </tbody>
                </Suspense>
            </table>
        </div>
    }
}
