use crate::app::models::student::ESLEnum;
use crate::app::models::student::Student;
use leptos::*;
use std::rc::Rc;

const TABLE_CONTAINER_STYLE: &str =
    "bg-white rounded-lg shadow-sm border border-gray-100 overflow-hidden";
const TABLE_HEADER_STYLE: &str =
    "py-5 px-6 flex justify-between items-center bg-[#00356b] border-b border-gray-100";
const TABLE_WRAPPER_STYLE: &str = "overflow-x-auto h-[33rem]";
const TABLE_STYLE: &str = "min-w-full divide-y divide-gray-100";
const HEADER_CELL_STYLE: &str =
    "px-6 py-3 text-left text-xs font-medium text-gray-600 uppercase tracking-wider";
const CELL_STYLE: &str = "px-6 py-4 whitespace-nowrap text-sm";
const SELECTED_ROW_STYLE: &str = "bg-blue-100 border-l-4 border-blue-600";

#[component]
pub fn StudentTable(
    #[prop(into)] students: Resource<i32, Option<Vec<Student>>>,
    #[prop(into)] search_term: Signal<String>,
    #[prop(into)] grade_filter: Signal<String>,
    #[prop(into)] teacher_filter: Signal<String>,
    #[prop(into)] iep_filter: Signal<bool>,
    #[prop(into)] esl_filter: Signal<bool>,
    #[prop(into)] bip_filter: Signal<bool>,
    #[prop(into)] selected_student: Signal<Option<Rc<Student>>>,
    #[prop(into)] set_selected_student: WriteSignal<Option<Rc<Student>>>,
) -> impl IntoView {
    let filtered_students = create_memo(move |_| {
        let search = search_term().trim().to_lowercase();
        let grade = grade_filter();
        let teacher = teacher_filter();
        let show_iep = iep_filter();
        let show_esl = esl_filter();
        let show_bip = bip_filter();

        students
            .get()
            .unwrap_or(None)
            .unwrap_or_default()
            .into_iter()
            .filter(|student| {
                // Filter by search term
                let matches_search = search.is_empty()
                    || student.firstname.to_lowercase().contains(&search)
                    || student.lastname.to_lowercase().contains(&search);

                // Filter by grade
                let matches_grade = grade.is_empty() || student.grade.to_string().contains(&grade);

                // Filter by teacher
                let matches_teacher = teacher == "all" || student.teacher.to_string() == teacher;

                // Filter by IEP
                let matches_iep = !show_iep || student.iep;

                // Filter by ESL - fixed for Option<ESLEnum>
                let matches_esl = !show_esl || student.esl != ESLEnum::NotApplicable;

                // Filter by BIP
                let matches_bip = !show_bip || student.bip;

                matches_search
                    && matches_grade
                    && matches_teacher
                    && matches_iep
                    && matches_esl
                    && matches_bip
            })
            .collect::<Vec<_>>()
    });
    view! {
        <div class=TABLE_CONTAINER_STYLE>
            <div class=TABLE_HEADER_STYLE>
                <h2 class="text-xl font-medium text-white">
                    "Students"
                </h2>
                <span class="text-sm text-white">
                    {move || {
                        let count = filtered_students().len();
                        format!("{} {}", count, if count == 1 { "student" } else { "students" })
                    }}
                </span>
            </div>
            <div class=TABLE_WRAPPER_STYLE>
                <div class="overflow-y-auto max-h-full">
                    <table class=TABLE_STYLE>
                        <thead class="bg-gray-50 sticky top-0 z-10">
                            <tr>
                                <th class=HEADER_CELL_STYLE>"First Name"</th>
                                <th class=HEADER_CELL_STYLE>"Last Name"</th>
                                <th class=HEADER_CELL_STYLE>"ID"</th>
                                <th class=HEADER_CELL_STYLE>"Grade"</th>
                                <th class=HEADER_CELL_STYLE>"Teacher"</th>
                                <th class=HEADER_CELL_STYLE>"IEP"</th>
                            </tr>
                        </thead>
                        <Suspense fallback=move || view! {
                            <tr>
                                <td colspan="6" class="text-center p-8">
                                    <div class="inline-block h-6 w-6 animate-spin rounded-full border-2 border-gray-300 border-t-gray-600"></div>
                                </td>
                            </tr>
                        }>
                            <tbody>
                                {move || {
                                    let students = filtered_students();
                                    if students.is_empty() {
                                        view! {
                                            <tr>
                                                <td colspan="6" class="px-6 py-12 text-center text-sm text-gray-500">
                                                    "No students match your search criteria"
                                                </td>
                                            </tr>
                                        }.into_view()
                                    } else {
                                        students.into_iter().map(|student| {
                                            let student_rc = Rc::new(student.clone());
                                            let student_cmp = Rc::new(student.clone());
                                            let is_selected = move || selected_student() == Some(student_cmp.clone());
                                            view! {
                                                <tr
                                                    class=move || if is_selected() {
                                                        format!("{} {}", SELECTED_ROW_STYLE, "cursor-pointer")
                                                    } else {
                                                        "hover:bg-gray-100 cursor-pointer border-b border-gray-200".to_string()
                                                    }
                                                    on:click=move |_| set_selected_student(Some(student_rc.clone()))
                                                >
                                                    <td class=format!("{} {}", CELL_STYLE, "font-medium text-gray-900")>{&student.firstname}</td>
                                                    <td class=format!("{} {}", CELL_STYLE, "font-medium text-gray-900")>{&student.lastname}</td>
                                                    <td class=format!("{} {}", CELL_STYLE, "text-gray-600")>{&student.student_id.to_string()}</td>
                                                    <td class=format!("{} {}", CELL_STYLE, "text-gray-600")>{&student.grade.to_string()}</td>
                                                    <td class=format!("{} {}", CELL_STYLE, "text-gray-600")>{&student.teacher.to_string()}</td>
                                                    <td class=CELL_STYLE>
                                                        { if student.iep {
                                                            view! {
                                                                <span class="px-2 py-1 text-xs font-medium rounded-full bg-green-200 text-green-800">
                                                                    "Yes"
                                                                </span>
                                                            }
                                                        } else {
                                                            view! {
                                                                <span class="px-2 py-1 text-xs font-medium rounded-full bg-gray-200 text-gray-700">
                                                                    "No"
                                                                </span>
                                                            }
                                                        }}
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()
                                    }
                                }}
                            </tbody>
                        </Suspense>
                    </table>
                </div>
            </div>
        </div>
    }
}
