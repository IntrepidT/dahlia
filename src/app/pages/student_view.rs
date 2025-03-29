use crate::app::components::header::Header;
use crate::app::components::student_page::bulk_upload_modal::BulkUploadModal;
use crate::app::components::student_page::student_search_filter::SearchFilter;
use crate::app::components::student_page::student_table::StudentTable;
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

// Side panel styles
const SIDE_PANEL_STYLE: &str = "w-1/3 h-[calc(100vh-5rem)] fixed right-0 top-0 mt-20 p-8";
const TABLE_CONTAINER_STYLE: &str = "w-2/3 mt-20 p-5 fixed h-[calc(100vh-5rem)] flex flex-col ml-5";

#[component]
pub fn StudentView() -> impl IntoView {
    // Signals for gathering data from existing students
    let (refresh_trigger, set_refresh_trigger) = create_signal(0);

    // Resource for fetching students
    let students = create_resource(
        move || refresh_trigger(),
        |_| async move {
            match get_students().await {
                Ok(students) => Some(students),
                Err(e) => {
                    log::error!("Failed to fetch students: {}", e);
                    None
                }
            }
        },
    );

    // Resource for fetching teachers
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

    // Selected student state
    let (selected_student, set_selected_student) = create_signal(None::<Rc<Student>>);

    // Editing state
    let (editing, set_editing) = create_signal(false);

    // Filter state signals
    let (search_term, set_search_term) = create_signal(String::new());
    let (grade_filter, set_grade_filter) = create_signal(String::from("all"));
    let (iep_filter, set_iep_filter) = create_signal(false);
    let (esl_filter, set_esl_filter) = create_signal(false);
    let (bip_filter, set_bip_filter) = create_signal(false);
    let (teacher_filter, set_teacher_filter) = create_signal(String::from("all"));

    // Adding student state
    let (adding_student, set_adding_student) = create_signal(false);

    // Delete student confirmation state
    let (confirm_delete_one, set_confirm_delete_one) = create_signal(false);
    let (confirm_delete_two, set_confirm_delete_two) = create_signal(String::new());

    //Signal for showing bulk upload modal
    let (show_bulk_upload_modal, set_show_bulk_upload_modal) = create_signal(false);

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

    // Handle student deletion
    let handle_delete_student = move |ev: SubmitEvent| {
        ev.prevent_default();
        let student_to_be_deleted = selected_student().unwrap();
        match confirm_delete_two().parse::<i32>() {
            Ok(validated_delete_two) => {
                if validated_delete_two == student_to_be_deleted.student_id {
                    let delete_student_request = DeleteStudentRequest::new(
                        student_to_be_deleted.firstname.clone(),
                        student_to_be_deleted.lastname.clone(),
                        validated_delete_two,
                    );

                    spawn_local(async move {
                        match delete_student(delete_student_request).await {
                            Ok(_) => {
                                set_refresh_trigger.update(|count| *count += 1);
                                set_selected_student(None);
                                set_confirm_delete_one(false);
                            }
                            Err(e) => {
                                log::error!("Error deleting student: {:?}", e);
                                set_confirm_delete_one(false);
                            }
                        };
                    });
                } else {
                    set_confirm_delete_one(false);
                    log::info!("Delete was cancelled - ID mismatch");
                }
            }
            Err(e) => {
                log::error!("Invalid student ID entered: {:?}", e);
                set_confirm_delete_one(false);
            }
        }
    };

    // Handle adding a new student
    let handle_add_student = move |_| {
        set_selected_student(None);
        set_adding_student(true);
        set_editing(false);
    };

    // Handle clearing all filters
    let handle_clear_filters = move |_| {
        set_search_term(String::new());
        set_grade_filter(String::from("all"));
        set_teacher_filter(String::from("all"));
        set_iep_filter(false);
        set_esl_filter(false);
        set_bip_filter(false);
    };

    // Grade filter transformer (converts "all" to empty string for matching logic)
    let transformed_grade_filter = create_memo(move |_| {
        if grade_filter() == "all" {
            String::new()
        } else {
            grade_filter()
        }
    });

    view! {
        <div class="min-h-screen flex flex-col">
            <Header />

            // Delete confirmation modal
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

            {/* Bulk Upload Modal */}
            <Show when=move || show_bulk_upload_modal()>
                <BulkUploadModal
                    set_show_modal=set_show_bulk_upload_modal
                    set_refresh_trigger=set_refresh_trigger
                />
            </Show>

            // Main content area (2/3 width)
            <div class=TABLE_CONTAINER_STYLE>
                // Search and filter component
                <SearchFilter
                    set_search_term=set_search_term
                    set_grade_filter=set_grade_filter
                    set_teacher_filter=set_teacher_filter
                    set_iep_filter=set_iep_filter
                    set_esl_filter=set_esl_filter
                    set_bip_filter=set_bip_filter
                    search_term=search_term
                    teachers=Signal::derive(move || teacher_names())
                    on_clear_filters=Callback::new(handle_clear_filters)
                />

                // Student table component
                <StudentTable
                    students=students
                    search_term=search_term
                    grade_filter=Signal::derive(move || transformed_grade_filter())
                    teacher_filter=teacher_filter
                    iep_filter=iep_filter
                    esl_filter=esl_filter
                    bip_filter=bip_filter
                    selected_student=selected_student
                    set_selected_student=set_selected_student
                />

                // Bottom action buttons
                <div class="mt-4 pt-2 flex gap-2 justify-end sticky bottom-0 bg-white">
                    <button
                        class="px-4 py-2 bg-gray-300 font-bold text-white rounded-lg"
                        on:click=move |_| set_show_bulk_upload_modal(true)
                    >
                        "Bulk Student Upload"
                    </button>
                    <button
                        class="inline-flex items-center justify-center px-4 py-2 bg-red-500 text-white rounded-md font-semibold hover:bg-red-600 focus:outline-none focus:ring-2 focus:ring-red-500/50 transition-colors duration-200 shadow-sm hover:shadow-md"
                        class:opacity-50=move || selected_student().is_none()
                        class:cursor-not-allowed=move || selected_student().is_none()
                        on:click=move |_| {
                            if selected_student().is_some() {
                                set_confirm_delete_one(true)
                            }
                        }
                    >
                        "Delete Student"
                    </button>
                    <button
                        class="inline-flex items-center justify-center px-4 py-2 bg-[#50C878] text-white rounded-md font-semibold hover:bg-[#41C35C] focus:outline-none focus:ring-2 focus:ring-[#50C878]/50 transition-colors duration-200 shadow-sm hover:shadow-md"
                        on:click=handle_add_student
                    >
                        "Add Student"
                    </button>
                </div>
            </div>

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
