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

// Side panel styles - Updated for responsiveness and toggle behavior
const SIDE_PANEL_STYLE: &str = "lg:w-[30%] w-full h-[calc(100vh-2rem)] fixed lg:right-0 right-0 top-0 mt-10 p-5 lg:p-10 z-20 lg:z-10 transform transition-transform duration-300 ease-in-out";
const SIDE_PANEL_STYLE_HIDDEN: &str = "w-0 h-[calc(100vh-2rem)] fixed lg:right-0 right-0 top-0 mt-10 overflow-hidden z-20 lg:z-10 transform transition-all duration-300 ease-in-out";

// Toggle button styles
const TOGGLE_BUTTON_STYLE: &str = "absolute left-0 top-1/2 -ml-8 bg-[#2E3A59] text-white p-2 rounded-l-md shadow-md hidden lg:flex items-center justify-center transition-all duration-300 transform";

// Table styles - Updated to be responsive to panel toggle
const TABLE_CONTAINER_STYLE_DEFAULT: &str = "w-full lg:w-[68%] fixed p-3 lg:p-5 h-[calc(100vh-2rem)] flex flex-col lg:ml-20 transition-all duration-300 ease-in-out";
const TABLE_CONTAINER_STYLE_EXPANDED: &str = "w-full lg:w-[92%] fixed p-3 lg:p-5 h-[calc(100vh-2rem)] flex flex-col lg:ml-20 transition-all duration-300 ease-in-out";

const TAB_BUTTON_ACTIVE: &str =
    "px-4 py-2 font-medium rounded-t-lg bg-[#2E3A59] text-white border-none";
const TAB_BUTTON_INACTIVE: &str =
    "px-4 py-2 font-medium rounded-t-lg bg-[#DADADA] text-[#2E3A59] hover:bg-gray-200";
const ADD_BUTTON_STYLE: &str = "inline-flex items-center justify-center px-4 py-2 bg-[#4CAF50] text-white rounded-md font-semibold hover:bg-[#388E3C] focus:outline-none focus:ring-2 focus:ring-[#388E3C]/50 transition-colors duration-200 shadow-sm hover:shadow-md";
const DELETE_BUTTON_STYLE: &str = "inline-flex items-center justify-center px-4 py-2 bg-[#F44336] text-white rounded-md font-semibold hover:bg-[#D32F2F] focus:outline-none focus:ring-2 focus:ring-[#D32F2F]/50 transition-colors duration-200 shadow-sm hover:shadow-md";

// Side panel styles for details
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
    
    // Panel visibility control
    let (show_side_panel, set_show_side_panel) = create_signal(false);
    
    // Panel toggle for desktop view
    let (panel_expanded, set_panel_expanded) = create_signal(false);

    // Watch for selected employee changes to show panel on mobile
    create_effect(move |_| {
        if selected_employee().is_some() || adding_employee() {
            set_show_side_panel(true);
            set_panel_expanded(true);
        }
    });

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

            // Main content area with dynamic width based on panel state
            <div class=move || {
                if panel_expanded() {
                    TABLE_CONTAINER_STYLE_DEFAULT.to_string()
                } else {
                    TABLE_CONTAINER_STYLE_EXPANDED.to_string()
                }
            }>
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
                        is_panel_expanded=Signal::derive(move || panel_expanded())
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
                        is_panel_expanded=Signal::derive(move || panel_expanded())
                    />
                </Show>

                // Action Buttons
                <div class="mt-4 pt-2 flex flex-wrap gap-2 justify-end sticky bottom-0 bg-[#F9F9F8]">
                    <button
                        class=DELETE_BUTTON_STYLE
                        class:opacity-50=move || selected_employee().is_none()
                        class:cursor-not-allowed=move || selected_employee().is_none()
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
                            set_show_side_panel(true); // Show panel on mobile when adding
                            set_panel_expanded(true); // Ensure panel is expanded
                        }
                    >
                        "Add Employee"
                    </button>
                </div>
            </div>

            // Employee Detail Side Panel - modified for responsive behavior with toggle capability
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
                            if adding_employee() {
                                "Add New Employee"
                            } else if selected_employee().is_some() {
                                "Employee Details"
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
                    when=move || selected_employee().is_some() || adding_employee()
                    fallback=|| view! {
                        <div class="hidden lg:flex items-center justify-center border-t-8 border-[#2E3A59] h-[95%] text-gray-500 rounded-lg shadow-lg bg-[#F9F9F8]">
                            "Select an employee to view details"
                        </div>
                    }
                >
                    {move || {
                        if adding_employee() {
                            view! {
                                <div class="h-full">
                                    <AddEmployeeForm
                                        on_cancel=move |_| {
                                            set_adding_employee(false);
                                            set_show_side_panel(false); // Close panel on mobile
                                        }
                                        on_save=move |_| {
                                            set_adding_employee(false);
                                            set_refresh_trigger.update(|count| *count += 1);
                                            set_show_side_panel(false); // Close panel on mobile
                                        }
                                    />
                                </div>
                            }
                        } else if let Some(employee) = selected_employee() {
                            view! {
                                <div class="h-full">
                                    <EmployeeDetails
                                        employee=employee
                                        on_close=move |_| {
                                            set_selected_employee(None);
                                            set_show_side_panel(false); // Close panel on mobile
                                        }
                                        call_refresh=move |_| {
                                            set_selected_employee(None);
                                            set_refresh_trigger.update(|count| *count +=1);
                                            set_show_side_panel(false); // Close panel on mobile
                                        }
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
                        if selected_employee().is_some() || adding_employee() {
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
