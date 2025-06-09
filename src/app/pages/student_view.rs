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

// Side panel styles - Updated for responsiveness and toggle behavior
const SIDE_PANEL_STYLE: &str = "lg:w-[30%] w-full h-[calc(100vh-2rem)] fixed lg:right-0 right-0 top-0 mt-10 p-5 lg:p-10 z-20 lg:z-10 transform transition-transform duration-300 ease-in-out";
const SIDE_PANEL_STYLE_HIDDEN: &str = "w-0 h-[calc(100vh-2rem)] fixed lg:right-0 right-0 top-0 mt-10 overflow-hidden z-20 lg:z-10 transform transition-all duration-300 ease-in-out";

// Toggle button styles
const TOGGLE_BUTTON_STYLE: &str = "absolute left-0 top-1/2 -ml-8 bg-[#2E3A59] text-white p-2 rounded-l-md shadow-md hidden lg:flex items-center justify-center transition-all duration-300 transform";

// Table styles - Updated to be responsive to panel toggle
const TABLE_CONTAINER_STYLE_DEFAULT: &str = "w-full lg:w-[68%] fixed p-3 lg:p-5 h-[calc(100vh-2rem)] flex flex-col lg:ml-20 transition-all duration-300 ease-in-out";
const TABLE_CONTAINER_STYLE_EXPANDED: &str = "w-full lg:w-[92%] fixed p-3 lg:p-5 h-[calc(100vh-2rem)] flex flex-col lg:ml-20 transition-all duration-300 ease-in-out";

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

    // Additional filter signals
    let (student_504_filter, set_student_504_filter) = create_signal(false);
    let (readplan_filter, set_readplan_filter) = create_signal(false);
    let (gt_filter, set_gt_filter) = create_signal(false);
    let (bip_filter, set_bip_filter) = create_signal(false);

    // Adding student state
    let (adding_student, set_adding_student) = create_signal(false);

    // Delete student confirmation state
    let (confirm_delete_one, set_confirm_delete_one) = create_signal(false);
    let (confirm_delete_two, set_confirm_delete_two) = create_signal(String::new());

    // Signal for showing bulk upload modal
    let (show_bulk_upload_modal, set_show_bulk_upload_modal) = create_signal(false);

    // Panel visibility control
    let (show_side_panel, set_show_side_panel) = create_signal(false);

    // Panel toggle for desktop view
    let (panel_expanded, set_panel_expanded) = create_signal(false);

    // Watch for selected student changes to show panel on mobile
    create_effect(move |_| {
        if selected_student().is_some() || adding_student() || editing() {
            set_show_side_panel(true);
            set_panel_expanded(true);
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
                        student_to_be_deleted.firstname.clone().unwrap(),
                        student_to_be_deleted.lastname.clone().unwrap(),
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
        set_panel_expanded(true); // Ensure panel is expanded
    };

    // Handle clearing all filters
    let handle_clear_filters = move |_| {
        set_search_term(String::new());
        set_grade_filter(String::from("all"));
        set_teacher_filter(String::from("all"));
        set_iep_filter(false);
        set_esl_filter(false);
        set_intervention_filter(String::from("all"));
        set_student_504_filter(false);
        set_readplan_filter(false);
        set_gt_filter(false);
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

            // Main content area with dynamic width based on panel state
            <div class=move || {
                if panel_expanded() {
                    TABLE_CONTAINER_STYLE_DEFAULT.to_string()
                } else {
                    TABLE_CONTAINER_STYLE_EXPANDED.to_string()
                }
            }>
                // Search and filter component
                <SearchFilter
                    set_search_term=set_search_term
                    set_grade_filter=set_grade_filter
                    set_teacher_filter=set_teacher_filter
                    set_iep_filter=set_iep_filter
                    set_esl_filter=set_esl_filter
                    set_intervention_filter=set_intervention_filter
                    set_student_504_filter=set_student_504_filter
                    set_readplan_filter=set_readplan_filter
                    set_gt_filter=set_gt_filter
                    set_bip_filter=set_bip_filter
                    search_term=search_term
                    teachers=Signal::derive(move || teacher_names())
                    on_clear_filters=Callback::new(handle_clear_filters)
                    is_panel_expanded=Signal::derive(move || panel_expanded())
                />

                // Student table component
                <StudentTable
                    students=students
                    search_term=search_term
                    grade_filter=Signal::derive(move || transformed_grade_filter())
                    teacher_filter=teacher_filter
                    iep_filter=iep_filter
                    esl_filter=esl_filter
                    intervention_filter=Signal::derive(move || transformed_intervention_filter())
                    student_504_filter=student_504_filter
                    readplan_filter=readplan_filter
                    gt_filter=gt_filter
                    bip_filter=bip_filter
                    is_panel_expanded=Signal::derive(move || panel_expanded())
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

            // Student Detail Side Panel - modified for responsive behavior with toggle capability
            <div class=move || {
                if show_side_panel() && panel_expanded() {
                    format!("{} {}", SIDE_PANEL_STYLE, "translate-x-0")
                } else if !panel_expanded() {
                    SIDE_PANEL_STYLE_HIDDEN.to_string()
                } else {
                    format!("{} {}", SIDE_PANEL_STYLE, "translate-x-full lg:translate-x-0")
                }
            }>
                // Panel header with close button
                <div class="flex justify-between items-center mb-4">
                    <h2 class="text-lg font-bold text-[#2E3A59]">
                        {move || {
                            if adding_student() {
                                "Add New Student"
                            } else if editing() {
                                "Edit Student"
                            } else if selected_student().is_some() {
                                "Student Details"
                            } else {
                                "Details"
                            }
                        }}
                    </h2>
                    <div class="flex gap-2">
                        // Desktop toggle button
                        <button
                            class="hidden lg:block text-[#2E3A59] p-1 rounded hover:bg-[#DADADA] transition-colors"
                            on:click=move |_| {
                                set_panel_expanded(false);
                            }
                            title="Collapse panel"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7" />
                            </svg>
                        </button>

                        // Mobile close button
                        <button
                            class="lg:hidden rounded p-1 hover:bg-[#DADADA] text-[#2E3A59]"
                            on:click=move |_| set_show_side_panel(false)
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                        </button>
                    </div>
                </div>

                <Show
                    when=move || selected_student().is_some() || adding_student() || editing()
                    fallback=|| view! {
                        <div class="hidden lg:flex items-center justify-center border-t-8 border-[#2E3A59] h-[95%] text-gray-500 rounded-lg shadow-lg bg-[#F9F9F8]">
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

            // Global Panel toggle button for when panel is fully collapsed
            <Show when=move || !panel_expanded()>
                <button
                    class="fixed right-4 top-16 lg:right-8 lg:top-20 bg-[#2E3A59] text-white p-2 rounded-full shadow-lg z-20"
                    on:click=move |_| {
                        set_panel_expanded(true);
                        if selected_student().is_some() || adding_student() || editing() {
                            set_show_side_panel(true);
                        }
                    }
                    title="Expand panel"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7" />
                    </svg>
                </button>
            </Show>
        </div>
    }
}
