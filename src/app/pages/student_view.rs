use crate::app::components::header::Header;
use crate::app::components::student_page::student_search_filter::SearchFilter; // Import the new component
use crate::app::components::student_page::update_student_form::UpdateStudent;
use crate::app::components::student_page::{
    add_student_form::AddStudentForm, student_details::StudentDetails,
};
use crate::app::models::student::{DeleteStudentRequest, ESLEnum, Student};
use crate::app::server_functions::students::{delete_student, get_students};
use crate::app::server_functions::teachers::get_teachers;
use leptos::ev::SubmitEvent;
use leptos::*;
use log::{debug, error, info};
use std::rc::Rc;

// Styles
const TABLE_CONTAINER_STYLE: &str =
    "w-2/3 p-8 mt-20 mr-10 fixed ml-8 h-[calc(100vh-5rem)] flex flex-col";
const TABLE_SCROLL_CONTAINER_STYLE: &str = "flex-grow overflow-auto rounded-lg";
const TABLE_HEADER_STYLE: &str = "bg-[#00356b] text-white text-left p-4";
const TABLE_STYLE: &str = "w-full table-fixed bg-white shadow-lg rounded-lg";
const ROW_BASE_STYLE: &str = "border-b hover:bg-gray-50 cursor-pointer h-16";
const ROW_SELECTED_STYLE: &str = "border-b bg-[#FDFBD4] h-16";
const CELL_STYLE: &str = "text-left p-4 truncate";
const CHECKBOX_CONTAINER_STYLE: &str = "flex items-center gap-2 bg-white rounded-lg px-4 py-3";

// Side panel styles
const SIDE_PANEL_STYLE: &str = "w-1/3 h-[calc(100vh-5rem)] fixed right-0 top-0 mt-20 p-8";
const INFO_CONTAINER_STYLE: &str =
    "h-full p-6 border-t-8 border-[#00356B] shadow-lg rounded-lg flex flex-col";
const INFO_CONTENT_STYLE: &str = "flex-grow overflow-y-auto";
const INFO_TITLE_STYLE: &str = "text-stone-400 text-xs";
const INFO_VALUE_STYLE: &str = "mt-1";
const INFO_GROUP_STYLE: &str = "mb-2";
const BUTTON_CONTAINER_STYLE: &str =
    "mt-4 pt-4 flex border-t gap-2 justify-end sticky bottom-0 bg-white";

#[component]
pub fn StudentView() -> impl IntoView {
    //Signals for gathering data from existing students
    let (refresh_trigger, set_refresh_trigger) = create_signal(0);
    let students = create_resource(
        move || refresh_trigger(),
        |_| async move { get_students().await },
    );
    //gather all teachers for use when updating or selecting a teacher field
    let teachers = create_resource(
        move || refresh_trigger(),
        |_| async move {
            match get_teachers().await {
                Ok(teachers) => {
                    log::info!("Teachers fetched successfully: {} teachers", teachers.len());
                    Some(teachers)
                }
                Err(e) => {
                    log::error!("Failed to fetch teachers: {}", e);
                    Some(vec![])
                }
            }
        },
    );

    let (selected_student, set_selected_student) = create_signal(None::<Rc<Student>>);

    //editing signals
    let (editing, set_editing) = create_signal(false);

    // Filter state signals
    let (search_term, set_search_term) = create_signal(String::new());
    let (grade_filter, set_grade_filter) = create_signal(String::from("all"));
    let (iep_filter, set_iep_filter) = create_signal(false);
    let (esl_filter, set_esl_filter) = create_signal(false);
    let (teacher_filter, set_teacher_filter) = create_signal(String::from("all"));

    //Signals for getting a new student
    let (adding_student, set_adding_student) = create_signal(false);

    //Delete Student Signal
    let (confirm_delete_one, set_confirm_delete_one) = create_signal(false);
    let (confirm_delete_two, set_confirm_delete_two) = create_signal(String::new());

    // Extract teacher names for the filter dropdown
    let teacher_names = create_memo(move |_| {
        if let Some(Some(teacher_list)) = teachers.get() {
            teacher_list
                .iter()
                .map(|teacher| teacher.lastname.clone())
                .collect::<Vec<_>>()
        } else {
            vec![]
        }
    });

    let filtered_students = move || {
        students.get().map(|result| {
            result.ok().map(|students_data| {
                students_data
                    .into_iter()
                    .filter(|student| {
                        let search = search_term().to_lowercase();
                        let matches_search = student.firstname.to_lowercase().contains(&search)
                            || student.lastname.to_lowercase().contains(&search)
                            || student.student_id.to_string().contains(&search);

                        let matches_grade =
                            grade_filter() == "all" || student.grade.to_string() == grade_filter();

                        let matches_iep = !iep_filter() || student.iep;

                        let matches_esl = !esl_filter() || student.esl != ESLEnum::NotApplicable;

                        let matches_teacher =
                            teacher_filter() == "all" || student.teacher == teacher_filter();

                        matches_search
                            && matches_grade
                            && matches_iep
                            && matches_esl
                            && matches_teacher
                    })
                    .collect::<Vec<_>>()
            })
        })
    };

    //to perform the deletion
    let handle_delete_student = move |ev: SubmitEvent| {
        ev.prevent_default();
        let student_to_be_deleted = selected_student().unwrap();
        let validated_delete_two = confirm_delete_two()
            .parse::<i32>()
            .expect("Delete two was processed correctly");
        if validated_delete_two == student_to_be_deleted.student_id {
            let delete_student_request = DeleteStudentRequest::new(
                student_to_be_deleted.firstname.clone(),
                student_to_be_deleted.lastname.clone(),
                validated_delete_two,
            );

            spawn_local(async move {
                let delete_result = delete_student(delete_student_request).await;

                match delete_result {
                    Ok(_deleted_student) => {
                        set_refresh_trigger.update(|count| *count += 1);
                        set_confirm_delete_one(false);
                    }
                    Err(e) => {
                        println!("Error deleting = {:?}", e);
                        set_confirm_delete_one(false);
                    }
                };
            });
        } else {
            set_confirm_delete_one(false);
            log::info!("Delete was cancelled");
        }
    };

    let handle_add_student = move |_| {
        set_selected_student(None);
        set_adding_student(true);
    };

    view! {
        <div class="min-h-screen flex">
            <Header />
            <Show when=move || confirm_delete_one() && selected_student().is_some()>
                <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                    <div class="bg-white p-6 rounded-lg shadow-xl max-w-md w-full">
                        <h3 class="text-xl font-bold mb-4">"Confirm Delete"</h3>
                        <p class="mb-4">
                            "To confirm deletion, please enter the student ID number: "
                                {selected_student().unwrap().student_id}
                        </p>
                        <form on:submit=handle_delete_student>
                            <input
                                type="text"
                                class="w-full p-2 border rounded mb-4"
                                placeholder="Enter student ID"
                                on:input=move |ev| set_confirm_delete_two(event_target_value(&ev))
                                required
                            />
                            <div class="flex justify-end gap-2">
                                <button
                                    type="button"
                                    class="px-4 py-2 bg-gray-200 rounded hover:bg-gray-300"
                                    on:click=move |_| set_confirm_delete_one(false)
                                >
                                    "Cancel"
                                </button>
                                <button
                                    type="submit"
                                    class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600"
                                >
                                    "Delete"
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            </Show>
            // Main content area (2/3 width)
            <div class=TABLE_CONTAINER_STYLE>
                // Using the new SearchFilter component
                <SearchFilter
                    set_search_term=set_search_term
                    set_grade_filter=set_grade_filter
                    set_teacher_filter=set_teacher_filter
                    set_iep_filter=set_iep_filter
                    set_esl_filter=set_esl_filter
                    teachers=Signal::derive(move || {
                        if let Some(data) = teachers.get() {
                            if let Some(teacher_list) = data {
                                return teacher_list.iter()
                                    .map(|teacher| teacher.lastname.clone())
                                    .collect::<Vec<_>>();
                            }
                        }
                        vec![]
                    })
                />

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
                <div class="mt-4 pt-2 flex gap-2 justify-end sticky bottom-0 bg-white">
                    <button class="px-4 py-2 bg-red-500 font-bold text-white rounded-lg hover:bg-[#FAA0A0]"
                        on:click=move |_| {
                            if selected_student().is_some() {
                                set_confirm_delete_one(true)
                            }
                        }
                    >
                        "Delete Student"
                    </button>
                    <button class="px-4 py-2 bg-green-500 text-white font-bold rounded-lg hover:bg-[#A8DCAB]"
                        on:click=handle_add_student
                    >
                        "Add Student"
                    </button>
                </div>
            </div>

            // Rest of the component remains unchanged...
            // Student Detail Side Panel
            <div class=SIDE_PANEL_STYLE>
                <Show
                    when=move || selected_student().is_some() || adding_student() || editing()
                    fallback=|| view! {
                        <div class="flex items-center justify-center border-t-8 border-[#00356b] h-full text-gray-500 rounded-lg shadow-lg">
                            "Select a student to view details"
                        </div>
                    }
                >
                    {move || {
                        if adding_student() {
                            view! {
                                <div class="h-full">
                                    <AddStudentForm
                                        set_adding_student=set_adding_student
                                        set_refresh_trigger=set_refresh_trigger
                                    />
                                </div>
                            }
                        } else if editing() {
                            view!{
                                <div class="h-full">
                                    <UpdateStudent
                                        student=selected_student().expect("A student struct")
                                        on_cancel=Callback::new(move |_| set_editing(false))
                                        on_update_success=Callback::new(move |updated| {
                                            set_selected_student(Some(Rc::new(updated)));
                                            set_editing(false);
                                            set_refresh_trigger.update(|count| *count +=1);
                                        })
                                    />
                                </div>
                            }
                        } else if let Some(student) = selected_student() {
                            view! {
                                <div class="h-full">
                                    <StudentDetails
                                        student=student
                                        on_edit_student=Callback::new(move |_| {
                                            set_adding_student(false);
                                            set_editing(true);
                                        })
                                    />
                                </div>
                            }
                        } else {
                            view! {
                                <div>"An error has occurred"</div>
                            }
                        }
                    }}
                </Show>
            </div>
        </div>
    }
}
