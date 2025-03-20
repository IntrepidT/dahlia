use crate::app::models::student::{ELLEnum, Student};
use leptos::*;
use std::rc::Rc;
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

                // Filter by ESL
                let matches_esl = !show_esl
                    || match student.esl {
                        Some(esl_status) => esl_status != ESLEnum::None,
                        None => false,
                    };

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
            // Search and filters header
            <div class="bg-[#00356b] rounded-lg p-6 mb-6">
                <div class="flex gap-4 flex-wrap">
                    // Search input
                    <div class="relative flex-grow max-w-[20rem]">
                        <input
                            type="text"
                            placeholder="Search students..."
                            class="w-full p-3 pl-4 rounded-lg"
                            on:input=move |ev| {
                                set_search_term(event_target_value(&ev));
                            }
                        />
                    </div>

                    // Filter dropdowns
                    <select
                        class="p-3 rounded-lg"
                        on:change=move |ev| set_grade_filter(event_target_value(&ev))
                    >
                        <option value="all">"All Grades"</option>
                        <option value="Kindergarten">"K"</option>
                        <option value="1st Grade">"1st"</option>
                        <option value="2nd Grade">"2nd"</option>
                        <option value="3rd Grade">"3rd"</option>
                        <option value="4th Grade">"4th"</option>
                        <option value="5th Grade">"5th"</option>
                        <option value="6th Grade">"6th"</option>
                        <option value="7th Grade">"7th"</option>
                        <option value="8th Grade">"8th"</option>
                        <option value="9th Grade">"9th"</option>
                        <option value="10th Grade">"10th"</option>
                        <option value="11th Grade">"11th"</option>
                        <option value="12th Grade">"12th"</option>
                    </select>

                    <select
                        class="p-3 rounded-lg"
                        on:change=move |ev| set_teacher_filter(event_target_value(&ev))
                    >
                        <option value="all">"Teacher"</option>
                    </select>

                   <div class=CHECKBOX_CONTAINER_STYLE>
                        <input
                            type="checkbox"
                            id="iep-filter"
                            class="form-checkbox h-5 w-5 text-[#00356b]"
                            on:change=move |ev| set_iep_filter(event_target_checked(&ev))
                        />
                        <label for="iep-filter">"Show IEP Students"</label>
                    </div>

                    <div class=CHECKBOX_CONTAINER_STYLE>
                        <input
                            type="checkbox"
                            id="ell-filter"
                            class="form-checkbox h-5 w-5 text-[#00356b]"
                            on:change=move |ev| set_ell_filter(event_target_checked(&ev))
                        />
                        <label for="ell-filter">"Show ELL Students"</label>
                    </div>
                </div>
            </div>

            <div class="h-[calc(100vh-16rem)] overflow-auto rounded-lg">
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
            <div class=BUTTON_CONTAINER_STYLE>
                    <button
                        class="px-4 py-2 bg-red-500 font-bold text-white rounded-lg hover:bg-[#FAA0A0]"
                        on:click=move |_| {
                            if selected_student().is_some() {
                                set_confirm_delete_one(true)
                            }
                        }
                    >
                        "Delete Student"
                    </button>
                    <button
                        class="px-4 py-2 bg-green-500 text-white font-bold rounded-lg hover:bg-[#A8DCAB]"
                        on:click=move |_| {
                            set_selected_student(None);
                            set_adding_student(true);
                        }
                    >
                        "Add Student"
                    </button>
                </div>
            </div>
        </div>
    }
}
