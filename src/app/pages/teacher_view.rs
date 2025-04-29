use crate::app::components::dashboard::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::header::Header;
use crate::app::components::teacher_page::{
    AddEmployeeForm, DeleteConfirmation, EmployeeDetails, EmployeeTable, SearchFilter, TeacherTable,
};
use crate::app::models::employee::{AddNewEmployeeRequest, Employee, EmployeeRole};
use crate::app::models::student::GradeEnum;
use crate::app::models::StatusEnum;
use crate::app::server_functions::employees::{add_employee, get_employees};
use crate::app::server_functions::teachers::get_teachers;
use leptos::ev::SubmitEvent;
use leptos::*;
use std::rc::Rc;
use std::str::FromStr;
use strum::IntoEnumIterator;
use validator::Validate;

// Side panel and button styles
const SIDE_PANEL_STYLE: &str = "w-[30%] h-[calc(100vh-5rem)] fixed right-0 top-0 mt-20 p-8";
const BUTTON_CONTAINER_STYLE_FLOAT: &str =
    "mt-4 pt-2 flex gap-2 justify-end sticky bottom-0 bg-[#F9F9F8] w-full";
const TAB_BUTTON_ACTIVE: &str =
    "px-4 py-2 font-medium rounded-t-lg bg-[#2E3A59] text-white border-none";
const TAB_BUTTON_INACTIVE: &str =
    "px-4 py-2 font-medium rounded-t-lg bg-[#DADADA] text-[#2E3A59] hover:bg-gray-200";
const ADD_BUTTON_STYLE: &str = "inline-flex items-center justify-center px-4 py-2 bg-[#4CAF50] text-white rounded-md font-semibold hover:bg-[#388E3C] focus:outline-none focus:ring-2 focus:ring-[#388E3C]/50 transition-colors duration-200 shadow-sm hover:shadow-md";
const DELETE_BUTTON_STYLE: &str = "inline-flex items-center justify-center px-4 py-2 bg-[#F44336] text-white rounded-md font-semibold hover:bg-[#D32F2F] focus:outline-none focus:ring-2 focus:ring-[#D32F2F]/50 transition-colors duration-200 shadow-sm hover:shadow-md";

// Side panel styles
const INFO_CONTAINER_STYLE: &str =
    "h-full p-6 border-t-8 border-[#2E3A59] shadow-lg rounded-lg flex flex-col";
const INFO_CONTENT_STYLE: &str = "flex-grow overflow-y-auto";
const INFO_TITLE_STYLE: &str = "text-stone-400 text-xs";
const INFO_VALUE_STYLE: &str = "mt-1";
const INFO_GROUP_STYLE: &str = "mb-2";
const BUTTON_CONTAINER_STYLE: &str =
    "mt-4 pt-4 flex border-t gap-2 justify-end sticky bottom-0 bg-white w-full";

#[component]
pub fn Teachers() -> impl IntoView {
    // Create resource for refreshing data
    let (refresh_trigger, set_refresh_trigger) = create_signal(0);
    let (selected_view, set_selected_view) = create_signal(SidebarSelected::TeacherView);
    //
    // Create resources for employees and teachers
    let employees = create_resource(
        move || refresh_trigger(),
        |_| async move {
            match get_employees().await {
                Ok(employees) => Some(employees),
                Err(e) => {
                    log::error!("Failed to fetch employees: {}", e);
                    Some(vec![])
                }
            }
        },
    );

    let teachers = create_resource(
        move || refresh_trigger(),
        |_| async move {
            match get_teachers().await {
                Ok(teachers) => Some(teachers),
                Err(e) => {
                    log::error!("Failed to fetch teachers: {}", e);
                    Some(vec![])
                }
            }
        },
    );

    // Main UI state signals
    let (active_view, set_active_view) = create_signal(String::from("employees"));
    let (selected_employee, set_selected_employee) = create_signal(None::<Rc<Employee>>);
    let (search_term, set_search_term) = create_signal(String::new());
    let (role_filter, set_role_filter) = create_signal(String::new());
    let (adding_employee, set_adding_employee) = create_signal(false);
    let (confirm_delete, set_confirm_delete) = create_signal(false);

    // Tab view selection handlers
    let select_teachers_view = move |_| set_active_view(String::from("teachers"));
    let select_employees_view = move |_| set_active_view(String::from("employees"));

    // Clear filters function
    let clear_filters = move |_| {
        set_search_term(String::new());
        set_role_filter(String::new());
    };

    view! {
        <div class="min-h-screen flex flex-col bg-[#F9F9F8]">
            <Header/>
            <DashboardSidebar
                selected_item=selected_view
                set_selected_item=set_selected_view
            />
            // Delete confirmation modal
            <Show when=move || confirm_delete() && selected_employee().is_some()>
                <DeleteConfirmation
                    selected_employee=selected_employee
                    on_cancel=Callback::new(move |_| set_confirm_delete(false))
                    on_delete=Callback::new(move |_| {
                        set_selected_employee(None::<Rc::<Employee>>);
                        set_refresh_trigger.update(|count| *count += 1);
                        set_confirm_delete(false);
                    })
                />
            </Show>

            <div class="w-[68%] px-6 ml-20">
                // Search and Filters
                <SearchFilter
                    search_term=search_term
                    set_search_term=set_search_term
                    role_filter=role_filter
                    set_role_filter=set_role_filter
                    on_clear_filters=clear_filters
                />

                // View selection tabs
                <div class="flex border-b mb-4">
                    <button
                        class=move || if active_view() == "employees" { TAB_BUTTON_ACTIVE } else { TAB_BUTTON_INACTIVE }
                        on:click=select_employees_view
                    >
                        "All Employees"
                    </button>
                    <button
                        class=move || if active_view() == "teachers" { TAB_BUTTON_ACTIVE } else { TAB_BUTTON_INACTIVE }
                        on:click=select_teachers_view
                    >
                        "Teachers"
                    </button>
                </div>

                // Teachers Table
                <Show when=move || active_view() == "teachers">
                    <TeacherTable
                        teachers=teachers
                        search_term=search_term
                        role_filter=role_filter
                        selected_employee=selected_employee
                        set_selected_employee=set_selected_employee
                    />
                </Show>

                // All Employees Table
                <Show when=move || active_view() == "employees">
                    <EmployeeTable
                        employees=employees
                        search_term=search_term
                        role_filter=role_filter
                        selected_employee=selected_employee
                        set_selected_employee=set_selected_employee
                    />
                </Show>

                // Action Buttons
                <div class=BUTTON_CONTAINER_STYLE_FLOAT>
                    <button
                        class=DELETE_BUTTON_STYLE
                        on:click=move |_| {
                            if selected_employee().is_some() {
                                set_confirm_delete(true)
                            }
                        }
                    >
                        "Delete"
                    </button>
                    <button
                        class=ADD_BUTTON_STYLE
                        on:click=move |_| {
                            set_adding_employee(true);
                            set_selected_employee(None);
                        }
                    >
                        "Add Employee"
                    </button>
                </div>


                // Side Panel
                <div class=SIDE_PANEL_STYLE>
                    {move || {
                        if adding_employee() {
                            view! {
                                <div class="h-full">
                                    <AddEmployeeForm
                                        on_cancel=move |_| set_adding_employee(false)
                                        on_save=move |_| {
                                            set_adding_employee(false);
                                            set_refresh_trigger.update(|count| *count += 1);
                                        }
                                    />
                                </div>
                            }
                        } else if let Some(employee) = selected_employee() {
                            view! {
                                <div class="h-full">
                                    <EmployeeDetails
                                        employee=employee
                                        on_close=move |_| set_selected_employee(None)
                                        call_refresh=move |_| {
                                            set_selected_employee(None);
                                            set_refresh_trigger.update(|count| *count +=1);
                                        }
                                    />
                                </div>
                            }
                        } else {
                            view! {
                                <div class=INFO_CONTAINER_STYLE>
                                    <div class="flex items-center justify-center h-full text-gray-500">
                                        "Select an employee to view their details"
                                    </div>
                                </div>
                            }
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
