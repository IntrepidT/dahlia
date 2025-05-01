use crate::app::components::dashboard::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
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

// Side panel styles - Updated for responsiveness
const SIDE_PANEL_STYLE: &str = "lg:w-[30%] w-full h-[calc(100vh-2rem)] fixed lg:right-0 right-0 top-0 mt-10 p-5 lg:p-10 z-20 lg:z-10 transform transition-transform duration-300 ease-in-out";
const TABLE_CONTAINER_STYLE: &str =
    "w-full lg:w-[68%] fixed p-3 lg:p-5 h-[calc(100vh-2rem)] flex flex-col lg:ml-20";

#[component]
pub fn StudentView() -> impl IntoView {
    // Signals for gathering data from existing students
    let (refresh_trigger, set_refresh_trigger) = create_signal(0);
    let (selected_view, set_selected_view) = create_signal(SidebarSelected::StudentView);

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
    let (intervention_filter, set_intervention_filter) = create_signal(String::from("all"));
    let (teacher_filter, set_teacher_filter) = create_signal(String::from("all"));

    // Adding student state
    let (adding_student, set_adding_student) = create_signal(false);

    // Delete student confirmation state
    let (confirm_delete_one, set_confirm_delete_one) = create_signal(false);
    let (confirm_delete_two, set_confirm_delete_two) = create_signal(String::new());

    // Signal for showing bulk upload modal
    let (show_bulk_upload_modal, set_show_bulk_upload_modal) = create_signal(false);

    // NEW: Panel visibility control for mobile
    let (show_side_panel, set_show_side_panel) = create_signal(false);

    // Watch for selected student changes to show panel on mobile
    create_effect(move |_| {
        if selected_student().is_some() || adding_student() || editing() {
            set_show_side_panel(true);
        }
    });

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
                                set_show_side_panel(false); // Close panel on mobile after deletion
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
        set_show_side_panel(true); // Show panel on mobile when adding
    };

    // Handle clearing all filters
    let handle_clear_filters = move |_| {
        set_search_term(String::new());
        set_grade_filter(String::from("all"));
        set_teacher_filter(String::from("all"));
        set_iep_filter(false);
        set_esl_filter(false);
        set_intervention_filter(String::from("all"));
    };

    // Grade filter transformer (converts "all" to empty string for matching logic)
    let transformed_grade_filter = create_memo(move |_| {
        if grade_filter() == "all" {
            String::new()
        } else {
            grade_filter()
        }
    });

    // Intervention filter transformer (converts "all" to empty string for matching logic)
    let transformed_intervention_filter = create_memo(move |_| {
        if intervention_filter() == "all" {
            String::new()
        } else {
            intervention_filter()
        }
    });

    view! {
        <div class="min-h-screen flex flex-col bg-[#F9F9F8]">
            <Header />
            <DashboardSidebar
                selected_item=selected_view
                set_selected_item=set_selected_view
            />

            // Delete confirmation modal
            <Show when=move || confirm_delete_one() && selected_student().is_some()>
                <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                    <div class="bg-white p-4 md:p-6 rounded-lg shadow-xl max-w-md w-full mx-4">
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

            // Main content area
            <div class=TABLE_CONTAINER_STYLE>
                // Search and filter component
                <SearchFilter
                    set_search_term=set_search_term
                    set_grade_filter=set_grade_filter
                    set_teacher_filter=set_teacher_filter
                    set_iep_filter=set_iep_filter
                    set_esl_filter=set_esl_filter
                    set_intervention_filter=set_intervention_filter
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
                    intervention_filter=intervention_filter
                    selected_student=selected_student
                    set_selected_student=set_selected_student
                />

                // Bottom action buttons
                <div class="mt-4 pt-2 flex flex-wrap gap-2 justify-end sticky bottom-0 bg-[#F9F9F8]">
                    <button
                        class="px-3 md:px-4 py-2 bg-[#F9F9F8] hover:bg-[#DADADA] hover:bg-opacity-30 font-bold text-[#2E3A59] border-[#DADADA] rounded-md border transition-colors text-sm md:text-base"
                        on:click=move |_| set_show_bulk_upload_modal(true)
                    >
                        "Bulk Upload"
                    </button>
                    <button
                        class="inline-flex items-center justify-center px-3 md:px-4 py-2 bg-[#F44336] text-white rounded-md font-semibold hover:bg-[#D32F2F] focus:outline-none focus:ring-2 focus:ring-[#F44336]/50 transition-colors duration-200 shadow-sm hover:shadow-md text-sm md:text-base"
                        class:opacity-50=move || selected_student().is_none()
                        class:cursor-not-allowed=move || selected_student().is_none()
                        on:click=move |_| {
                            if selected_student().is_some() {
                                set_confirm_delete_one(true)
                            }
                        }
                    >
                        "Delete"
                    </button>
                    <button
                        class="inline-flex items-center justify-center px-3 md:px-4 py-2 bg-[#4CAF50] text-white rounded-md font-semibold hover:bg-[#388E3C] focus:outline-none focus:ring-2 focus:ring-[#4CAF50]/50 transition-colors duration-200 shadow-sm hover:shadow-md text-sm md:text-base"
                        on:click=handle_add_student
                    >
                        "Add Student"
                    </button>
                </div>
            </div>

            // Student Detail Side Panel - modified for responsive behavior
            <div class=format!("{} {}",
                SIDE_PANEL_STYLE,
                if show_side_panel() {
                    "translate-x-0"
                } else {
                    "translate-x-full lg:translate-x-0"
                }
            )>
                // Mobile close button
                <Show when=move ||
                    (selected_student().is_some() || adding_student() || editing()) &&
                    show_side_panel()
                >
                    <button
                        class="lg:hidden absolute top-3 left-3 rounded-full p-2 bg-[#2E3A59] text-white"
                        on:click=move |_| set_show_side_panel(false)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </Show>

                <Show
                    when=move || selected_student().is_some() || adding_student() || editing()
                    fallback=|| view! {
                        <div class="hidden lg:flex items-center justify-center border-t-8 border-[#2E3A59] h-full text-gray-500 rounded-lg shadow-lg bg-[#F9F9F8]">
                            "Select a student to view details"
                        </div>
                    }
                >
                    {move || {
                        if adding_student() {
                            view! {
                                <div class="h-full mt-6 lg:mt-0">
                                    <AddStudentForm
                                        set_adding_student=set_adding_student
                                        set_refresh_trigger=set_refresh_trigger
                                    />
                                </div>
                            }
                        } else if editing() {
                            view!{
                                <div class="h-full mt-6 lg:mt-0">
                                    <UpdateStudent
                                        student=selected_student().expect("A student struct")
                                        on_cancel=Callback::new(move |_| {
                                            set_editing(false);
                                            set_show_side_panel(false); // Close panel on mobile
                                        })
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
                                <div class="h-full mt-6 lg:mt-0">
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
