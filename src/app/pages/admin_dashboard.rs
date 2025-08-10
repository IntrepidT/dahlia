use crate::app::components::auth::server_auth_components::ServerAuthGuard;
use crate::app::components::dashboard::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::header::Header;
use crate::app::models::{
    course::{Course, CreateCourseRequest, UpdateCourseRequest},
    enrollment::{AcademicYear, CreateEnrollmentRequest, Enrollment, EnrollmentStatus},
    student::GradeEnum,
};
use crate::app::server_functions::{courses::*, enrollments::*};
use chrono::{Local, NaiveDate};
use leptos::prelude::*;
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::hooks::*;
use leptos_router::path;
use rust_decimal::Decimal;
use std::str::FromStr;
use strum::IntoEnumIterator;

#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

#[derive(Clone, Debug, PartialEq)]
enum EnrollmentAction {
    QuickEnroll,
    AddNew,
}

#[derive(Clone, Debug)]
struct EnrollmentFormData {
    student_id: String,
    course_id: String,
    academic_year: AcademicYear,
    grade_level: GradeEnum,
    teacher_id: String,
    status: EnrollmentStatus,
    enrollment_date: NaiveDate,
    status_change_date: Option<NaiveDate>,
    notes: Option<String>,
}
impl Default for EnrollmentFormData {
    fn default() -> Self {
        Self {
            student_id: String::new(),
            course_id: String::new(),
            academic_year: AcademicYear::Year2024_2025,
            grade_level: GradeEnum::Kindergarten,
            teacher_id: String::new(),
            status: EnrollmentStatus::Active,
            enrollment_date: Local::now().date_naive(),
            status_change_date: None,
            notes: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum DashboardView {
    Courses,
    Enrollments,
}

#[derive(Clone, Debug)]
struct CourseFormData {
    name: String,
    subject: String,
    course_code: String,
    course_level: GradeEnum,
    teacher_id: String,
    academic_year: AcademicYear,
    semester_period: String,
    credits: String,
    description: String,
    max_students: String,
    room_number: String,
}

impl Default for CourseFormData {
    fn default() -> Self {
        Self {
            name: String::new(),
            subject: String::new(),
            course_code: String::new(),
            course_level: GradeEnum::Kindergarten,
            teacher_id: String::new(),
            academic_year: AcademicYear::Year2024_2025,
            semester_period: String::new(),
            credits: String::new(),
            description: String::new(),
            max_students: String::new(),
            room_number: String::new(),
        }
    }
}

#[component]
pub fn AdminDashboard() -> impl IntoView {
    view! {
        <ServerAuthGuard page_path="/admindashboard">
            <AdminDashboardContent />
        </ServerAuthGuard>
    }
}

#[component]
pub fn AdminDashboardContent() -> impl IntoView {
    let (selected_view, set_selected_view) = signal(SidebarSelected::AdminDashboard);
    let (current_tab, set_current_tab) = signal(DashboardView::Courses);

    // Course management state
    let (courses, set_courses) = create_signal::<Vec<Course>>(vec![]);
    let (course_search, set_course_search) = signal(String::new());
    let (show_course_form, set_show_course_form) = signal(false);
    let (editing_course, set_editing_course) = create_signal::<Option<Course>>(None);

    // Enrollment management state
    let (enrollments, set_enrollments) = create_signal::<Vec<Enrollment>>(vec![]);
    let (enrollment_search, set_enrollment_search) = signal(String::new());
    let (show_enrollment_form, set_show_enrollment_form) = signal(false);
    let (enrollment_action, set_enrollment_action) = signal(EnrollmentAction::AddNew);
    let (selected_course_for_enrollment, set_selected_course_for_enrollment) =
        create_signal::<Option<Course>>(None);

    // Load courses
    let load_courses = Action::new(|_: &()| async move {
        match get_courses().await {
            Ok(courses) => courses,
            Err(_) => vec![],
        }
    });

    // Load enrollments
    let load_enrollments = Action::new(|_: &()| async move {
        match get_enrollments().await {
            Ok(enrollments) => enrollments,
            Err(_) => vec![],
        }
    });

    // Delete course action
    let delete_course_action = Action::new(|course_id: &i32| {
        let course_id = *course_id;
        async move {
            match delete_course(course_id).await {
                Ok(_) => Some(course_id),
                Err(_) => None,
            }
        }
    });

    // Create/Update course action
    let save_course_action =
        Action::new(|(form_data, editing_id): &(CourseFormData, Option<i32>)| {
            let form_data = form_data.clone();
            let editing_id = *editing_id;
            async move {
                let teacher_id = form_data.teacher_id.parse::<i32>().unwrap_or(0);
                let credits = Decimal::from_str(&form_data.credits).unwrap_or_default();
                let max_students = form_data.max_students.parse::<i32>().unwrap_or(0);
                let room_number = if form_data.room_number.is_empty() {
                    None
                } else {
                    Some(form_data.room_number)
                };

                if let Some(course_id) = editing_id {
                    // Update existing course
                    let update_request = UpdateCourseRequest::new(
                        Some(form_data.name),
                        Some(form_data.subject),
                        Some(form_data.course_code),
                        Some(form_data.course_level),
                        Some(teacher_id),
                        Some(form_data.academic_year),
                        Some(form_data.semester_period),
                        Some(credits),
                        Some(form_data.description),
                        Some(max_students),
                        Some(room_number),
                    );
                    update_course(course_id, update_request).await.is_ok()
                } else {
                    // Create new course
                    let create_request = CreateCourseRequest::new(
                        form_data.name,
                        form_data.subject,
                        form_data.course_code,
                        form_data.course_level,
                        teacher_id,
                        form_data.academic_year,
                        form_data.semester_period,
                        credits,
                        form_data.description,
                        max_students,
                        room_number,
                    );
                    add_course(create_request).await.is_ok()
                }
            }
        });

    // Update enrollment status action
    let update_enrollment_status_action = Action::new(
        |(student_id, academic_year, status): &(i32, AcademicYear, EnrollmentStatus)| {
            let student_id = *student_id;
            let academic_year = academic_year.clone();
            let status = status.clone();
            async move {
                update_enrollment_status(student_id, academic_year, status)
                    .await
                    .is_ok()
            }
        },
    );

    // Create enrollment action
    let create_enrollment_action = Action::new(|form_data: &EnrollmentFormData| {
        let form_data = form_data.clone();
        async move {
            let student_id = form_data.student_id.parse::<i32>().unwrap_or(0);
            let course_id = form_data.course_id.parse::<i32>().unwrap_or(0);
            let teacher_id = form_data.teacher_id.parse::<i32>().unwrap_or(0);

            if student_id > 0 && course_id > 0 && teacher_id > 0 {
                let create_request = CreateEnrollmentRequest::new(
                    student_id,
                    form_data.course_id.parse::<i32>().unwrap_or(0),
                    form_data.academic_year,
                    form_data.grade_level,
                    teacher_id,
                    form_data.status,
                    Local::now().date_naive(),
                    Some(Local::now().date_naive()),
                    form_data.notes.clone(),
                );
                create_enrollment(create_request).await.is_ok()
            } else {
                false
            }
        }
    });

    // Delete enrollment action
    let delete_enrollment_action =
        Action::new(|(student_id, academic_year): &(i32, AcademicYear)| {
            let student_id = *student_id;
            let academic_year = academic_year.clone(); // Clone here
            async move {
                match delete_enrollment(student_id, academic_year.clone()).await {
                    // Clone again for the function call
                    Ok(_) => Some((student_id, academic_year)), // Now we can use the cloned value
                    Err(_) => None,
                }
            }
        });

    // Effects to reload data when actions complete
    Effect::new(move |_| {
        if load_courses.value().get().is_some() {
            if let Some(loaded_courses) = load_courses.value().get() {
                set_courses.set(loaded_courses);
            }
        }
    });

    Effect::new(move |_| {
        if load_enrollments.value().get().is_some() {
            if let Some(loaded_enrollments) = load_enrollments.value().get() {
                set_enrollments.set(loaded_enrollments);
            }
        }
    });

    Effect::new(move |_| {
        if let Some(Some(_)) = delete_course_action.value().get() {
            load_courses.dispatch(());
        }
    });

    Effect::new(move |_| {
        if save_course_action.value().get() == Some(true) {
            set_show_course_form.set(false);
            set_editing_course.set(None);
            load_courses.dispatch(());
        }
    });

    Effect::new(move |_| {
        if update_enrollment_status_action.value().get() == Some(true) {
            load_enrollments.dispatch(());
        }
    });

    // Initial data load
    Effect::new(move |_| {
        load_courses.dispatch(());
        load_enrollments.dispatch(());
    });

    Effect::new(move |_| {
        if create_enrollment_action.value().get() == Some(true) {
            set_show_enrollment_form.set(false);
            set_selected_course_for_enrollment.set(None);
            load_enrollments.dispatch(());
        }
    });

    Effect::new(move |_| {
        if let Some(Some(_)) = delete_enrollment_action.value().get() {
            load_enrollments.dispatch(());
        }
    });

    // Filtered courses based on search
    let filtered_courses = Memo::new(move |_| {
        let search = course_search.get().to_lowercase();
        if search.is_empty() {
            courses.get()
        } else {
            courses
                .get()
                .into_iter()
                .filter(|course| {
                    course.name.to_lowercase().contains(&search)
                        || course.course_code.to_lowercase().contains(&search)
                        || course.subject.to_lowercase().contains(&search)
                })
                .collect()
        }
    });

    // Filtered enrollments based on search
    let filtered_enrollments = Memo::new(move |_| {
        let search = enrollment_search.get();
        if search.is_empty() {
            enrollments.get()
        } else {
            if let Ok(student_id) = search.parse::<i32>() {
                enrollments
                    .get()
                    .into_iter()
                    .filter(|enrollment| enrollment.student_id == student_id)
                    .collect()
            } else {
                enrollments.get()
            }
        }
    });

    view! {
        <div class="min-h-screen bg-[#F9F9F8]">
            <Header />
            <DashboardSidebar
                selected_item=selected_view
                set_selected_item=set_selected_view
            />
            <div class="flex flex-1 ml-20 mt-20 p-6 max-h-screen overflow-y-auto">
                <div class="w-full max-w-7xl mx-auto max-h-full">
                    // Tab Navigation
                    <div class="mb-6">
                        <div class="border-b border-gray-200">
                            <nav class="-mb-px flex space-x-8">
                                <button
                                    class=move || if current_tab.get() == DashboardView::Courses {
                                        "border-indigo-500 text-indigo-600 whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm"
                                    } else {
                                        "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm"
                                    }
                                    on:click=move |_| set_current_tab.set(DashboardView::Courses)
                                >
                                    "Courses"
                                </button>
                                <button
                                    class=move || if current_tab.get() == DashboardView::Enrollments {
                                        "border-indigo-500 text-indigo-600 whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm"
                                    } else {
                                        "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm"
                                    }
                                    on:click=move |_| set_current_tab.set(DashboardView::Enrollments)
                                >
                                    "Enrollments"
                                </button>
                            </nav>
                        </div>
                    </div>

                    // Content based on selected tab
                    <div class="overflow-y-auto">
                        {move || match current_tab.get() {
                            DashboardView::Courses => view! {
                                <div class="space-y-6">
                                    // Course Management Header
                                    <div class="flex justify-between items-center">
                                        <h1 class="text-2xl font-bold text-gray-900">"Course Management"</h1>
                                        <div class="space-x-3">
                                            <button
                                                class="bg-green-600 text-white px-4 py-2 rounded-md hover:bg-green-700 transition-colors"
                                                on:click=move |_| {
                                                    set_enrollment_action.set(EnrollmentAction::QuickEnroll);
                                                    set_show_enrollment_form.set(true);
                                                    set_selected_course_for_enrollment.set(None);
                                                }
                                            >
                                                "Quick Enroll Student"
                                            </button>
                                            <button
                                                class="bg-indigo-600 text-white px-4 py-2 rounded-md hover:bg-indigo-700 transition-colors"
                                                on:click=move |_| {
                                                    set_show_course_form.set(true);
                                                    set_editing_course.set(None);
                                                }
                                            >
                                                "Add New Course"
                                            </button>
                                        </div>
                                    </div>

                                    // Search Bar
                                    <div class="relative">
                                        <input
                                            type="text"
                                            placeholder="Search courses by name, code, or subject..."
                                            class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                                            prop:value=course_search
                                            on:input=move |ev| set_course_search.set(event_target_value(&ev))
                                        />
                                    </div>

                                    // Courses Table
                                    <div class="bg-white shadow rounded-lg overflow-hidden border border-gray-300">
                                        <table class="min-w-full divide-y divide-gray-200">
                                            <thead class="bg-gray-50">
                                                <tr>
                                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Course Code"</th>
                                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Name"</th>
                                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Subject"</th>
                                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Grade Level"</th>
                                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Academic Year"</th>
                                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Max Students"</th>
                                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Actions"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="bg-white divide-y divide-gray-200">
                                                <For
                                                    each=filtered_courses
                                                    key=|course| course.id
                                                    children=move |course| {
                                                        let course_clone_edit = course.clone();
                                                        let course_clone_enroll = course.clone();
                                                        let delete_course_id = course.id;

                                                        view! {
                                                            <tr>
                                                                <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">{course.course_code.clone()}</td>
                                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{course.name.clone()}</td>
                                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{course.subject.clone()}</td>
                                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{format!("{:?}", course.course_level)}</td>
                                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{format!("{}", course.academic_year)}</td>
                                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{course.max_students}</td>
                                                                <td class="px-6 py-4 whitespace-nowrap text-sm font-medium space-x-2">
                                                                    <button
                                                                        class="text-indigo-600 hover:text-indigo-900"
                                                                        on:click=move |_| {
                                                                            set_editing_course.set(Some(course_clone_edit.clone()));
                                                                            set_show_course_form.set(true);
                                                                        }
                                                                    >
                                                                        "Edit"
                                                                    </button>
                                                                    <button
                                                                        class="text-green-600 hover:text-green-900 ml-2"
                                                                        on:click=move |_| {
                                                                            set_selected_course_for_enrollment.set(Some(course_clone_enroll.clone()));
                                                                            set_enrollment_action.set(EnrollmentAction::QuickEnroll);
                                                                            set_show_enrollment_form.set(true);
                                                                        }
                                                                    >
                                                                        "Enroll"
                                                                    </button>
                                                                    <button
                                                                        class="text-red-600 hover:text-red-900"
                                                                        on:click=move |_| {
                                                                            #[cfg(feature = "hydrate")]
                                                                            {
                                                                                if window().confirm_with_message("Are you sure you want to delete this course?").unwrap_or(false) {
                                                                                    delete_course_action.dispatch(delete_course_id);
                                                                                }
                                                                            }
                                                                        }
                                                                    >
                                                                        "Delete"
                                                                    </button>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }
                                                />
                                            </tbody>
                                        </table>
                                    </div>
                                </div>
                            }.into_any(),

                            DashboardView::Enrollments => view! {
                                <div class="space-y-6">
                                    // Enrollment Management Header
                                    <div class="flex justify-between items-center">
                                        <h1 class="text-2xl font-bold text-gray-900">"Enrollment Management"</h1>
                                        <button
                                            class="bg-indigo-600 text-white px-4 py-2 rounded-md hover:bg-indigo-700 transition-colors"
                                            on:click=move |_| {
                                                set_enrollment_action.set(EnrollmentAction::AddNew);
                                                set_show_enrollment_form.set(true);
                                                set_selected_course_for_enrollment.set(None);
                                            }
                                        >
                                            "Add New Enrollment"
                                        </button>
                                    </div>

                                    // Search Bar
                                    <div class="relative">
                                        <input
                                            type="text"
                                            placeholder="Search by student ID..."
                                            class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                                            prop:value=enrollment_search
                                            on:input=move |ev| set_enrollment_search.set(event_target_value(&ev))
                                        />
                                    </div>

                                    // Enrollments Table
                                    <div class="bg-white shadow rounded-lg overflow-hidden">
                                        <table class="min-w-full divide-y divide-gray-200">
                                            <thead class="bg-gray-50">
                                                <tr>
                                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Student ID"</th>
                                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Academic Year"</th>
                                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Grade Level"</th>
                                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Teacher ID"</th>
                                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Status"</th>
                                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Enrollment Date"</th>
                                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Actions"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="bg-white divide-y divide-gray-200">
                                                <For
                                                    each=filtered_enrollments
                                                    key=|enrollment| (enrollment.student_id, enrollment.academic_year.clone())
                                                    children=move |enrollment| {
                                                        let enrollment_clone_status = enrollment.clone();
                                                        let enrollment_clone_delete = enrollment.clone();

                                                        view! {
                                                            <tr>
                                                                <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">{enrollment.student_id}</td>
                                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{format!("{}", enrollment.academic_year)}</td>
                                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{format!("{:?}", enrollment.grade_level)}</td>
                                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{enrollment.teacher_id}</td>
                                                                <td class="px-6 py-4 whitespace-nowrap">
                                                                    <select
                                                                        class="text-sm border border-gray-300 rounded px-2 py-1"
                                                                        on:change=move |ev| {
                                                                            let status_str = event_target_value(&ev);
                                                                            if let Some(new_status) = EnrollmentStatus::iter()
                                                                                .find(|s| s.to_string() == status_str) {
                                                                                update_enrollment_status_action.dispatch((
                                                                                    enrollment_clone_status.student_id,
                                                                                    enrollment_clone_status.academic_year.clone(),
                                                                                    new_status
                                                                                ));
                                                                            }
                                                                        }
                                                                    >
                                                                        {
                                                                            EnrollmentStatus::iter().map(|status| {
                                                                                let is_selected = enrollment.status == status;
                                                                                view! {
                                                                                    <option value=status.to_string() selected=is_selected>{status.to_string()}</option>
                                                                                }
                                                                            }).collect::<Vec<_>>()
                                                                        }
                                                                    </select>
                                                                </td>
                                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{enrollment.enrollment_date.format("%Y-%m-%d").to_string()}</td>
                                                                <td class="px-6 py-4 whitespace-nowrap text-sm font-medium space-x-2">
                                                                    <button
                                                                        class="text-red-600 hover:text-red-900"
                                                                        on:click=move |_| {
                                                                            #[cfg(feature = "hydrate")]
                                                                            {
                                                                                if window().confirm_with_message("Are you sure you want to delete this enrollment?").unwrap_or(false) {
                                                                                    delete_enrollment_action.dispatch((enrollment_clone_delete.student_id, enrollment_clone_delete.academic_year.clone()));
                                                                                }
                                                                            }
                                                                        }
                                                                    >
                                                                        "Delete"
                                                                    </button>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }
                                                />
                                            </tbody>
                                        </table>
                                    </div>
                                </div>
                            }.into_any(),
                        }}
                    </div>

                    // Course Form Modal
                    <Show when=show_course_form>
                        <div class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
                            <div class="relative top-20 mx-auto p-5 border w-11/12 max-w-2xl shadow-lg rounded-md bg-white">
                                <div class="mt-3">
                                    <h3 class="text-lg font-medium text-gray-900 mb-4">
                                        {move || if editing_course.get().is_some() { "Edit Course" } else { "Add New Course" }}
                                    </h3>
                                    <form on:submit=move |ev| {
                                        ev.prevent_default();
                                        #[cfg(feature = "hydrate")]
                                        {
                                            let form = ev.target().unwrap().dyn_into::<web_sys::HtmlFormElement>().unwrap();
                                            let form_data = web_sys::FormData::new_with_form(&form).unwrap();

                                            let course_data = CourseFormData {
                                                name: form_data.get("name").as_string().unwrap_or_default(),
                                                subject: form_data.get("subject").as_string().unwrap_or_default(),
                                                course_code: form_data.get("course_code").as_string().unwrap_or_default(),
                                                course_level: form_data.get("course_level").as_string()
                                                    .and_then(|s| GradeEnum::iter().find(|g| g.to_string() == s))
                                                    .unwrap_or(GradeEnum::Kindergarten),
                                                teacher_id: form_data.get("teacher_id").as_string().unwrap_or_default(),
                                                academic_year: form_data.get("academic_year").as_string()
                                                    .and_then(|s| AcademicYear::iter().find(|y| y.to_string() == s))
                                                    .unwrap_or(AcademicYear::Year2024_2025),
                                                semester_period: form_data.get("semester_period").as_string().unwrap_or_default(),
                                                credits: form_data.get("credits").as_string().unwrap_or_default(),
                                                description: form_data.get("description").as_string().unwrap_or_default(),
                                                max_students: form_data.get("max_students").as_string().unwrap_or_default(),
                                                room_number: form_data.get("room_number").as_string().unwrap_or_default(),
                                            };

                                            let editing_id = editing_course.get().map(|c| c.id);
                                            save_course_action.dispatch((course_data, editing_id));
                                        }

                                    }>
                                        <div class="grid grid-cols-2 gap-4">
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700">"Course Name"</label>
                                                <input
                                                    type="text"
                                                    required
                                                    class="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                                                    name="name"
                                                    value=move || editing_course.get().map(|c| c.name.clone()).unwrap_or_default()
                                                />
                                            </div>
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700">"Subject"</label>
                                                <input
                                                    type="text"
                                                    required
                                                    class="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                                                    name="subject"
                                                    value=move || editing_course.get().map(|c| c.subject.clone()).unwrap_or_default()
                                                />
                                            </div>
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700">"Course Code"</label>
                                                <input
                                                    type="text"
                                                    required
                                                    class="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                                                    name="course_code"
                                                    value=move || editing_course.get().map(|c| c.course_code.clone()).unwrap_or_default()
                                                />
                                            </div>
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700">"Teacher ID"</label>
                                                <input
                                                    type="number"
                                                    required
                                                    class="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                                                    name="teacher_id"
                                                    value=move || editing_course.get().map(|c| c.teacher_id.to_string()).unwrap_or_default()
                                                />
                                            </div>
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700">"Semester Period"</label>
                                                <input
                                                    type="text"
                                                    required
                                                    class="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                                                    name="semester_period"
                                                    value=move || editing_course.get().map(|c| c.semester_period.clone()).unwrap_or_default()
                                                />
                                            </div>
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700">"Credits"</label>
                                                <input
                                                    type="number"
                                                    step="0.1"
                                                    required
                                                    class="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                                                    name="credits"
                                                    value=move || editing_course.get().map(|c| c.credits.to_string()).unwrap_or_default()
                                                />
                                            </div>
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700">"Max Students"</label>
                                                <input
                                                    type="number"
                                                    required
                                                    class="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                                                    name="max_students"
                                                    value=move || editing_course.get().map(|c| c.max_students.to_string()).unwrap_or_default()
                                                />
                                            </div>
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700">"Room Number"</label>
                                                <input
                                                    type="text"
                                                    class="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                                                    name="room_number"
                                                    value=move || editing_course.get().and_then(|c| c.room_number.clone()).unwrap_or_default()
                                                />
                                            </div>
                                        </div>
                                        <div class="col-span-2 mt-4">
                                            <label class="block text-sm font-medium text-gray-700">"Description"</label>
                                            <textarea
                                                rows="3"
                                                class="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                                                name="description"
                                            >{move || editing_course.get().map(|c| c.description.clone()).unwrap_or_default()}</textarea>
                                        </div>

                                        // Grade Level and Academic Year selects
                                        <div class="grid grid-cols-2 gap-4 mt-4">
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700">"Grade Level"</label>
                                                <select
                                                    required
                                                    class="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                                                    name="course_level"
                                                >
                                                    {
                                                        GradeEnum::iter().map(|level| {
                                                            let is_selected = editing_course.get()
                                                                .map(|c| c.course_level == level)
                                                                .unwrap_or(level == GradeEnum::Kindergarten);
                                                            view! {
                                                                <option value=level.to_string() selected=is_selected>{level.to_string()}</option>
                                                            }
                                                        }).collect::<Vec<_>>()
                                                    }
                                                </select>
                                            </div>
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700">"Academic Year"</label>
                                                <select
                                                    required
                                                    class="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                                                    name="academic_year"
                                                >
                                                    {
                                                        AcademicYear::iter().map(|year| {
                                                            let is_selected = editing_course.get()
                                                                .map(|c| c.academic_year == year)
                                                                .unwrap_or(year == AcademicYear::Year2024_2025);
                                                            view! {
                                                                <option value=year.to_string() selected=is_selected>{year.to_string()}</option>
                                                            }
                                                        }).collect::<Vec<_>>()
                                                    }
                                                </select>
                                            </div>
                                        </div>

                                        // Form Actions
                                        <div class="flex justify-end space-x-3 mt-6">
                                            <button
                                                type="button"
                                                class="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                                                on:click=move |_| {
                                                    set_show_course_form.set(false);
                                                    set_editing_course.set(None);
                                                }
                                            >
                                                "Cancel"
                                            </button>
                                            <button
                                                type="submit"
                                                class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                                                disabled=move || save_course_action.pending().get()
                                            >
                                                {move || if save_course_action.pending().get() {
                                                    if editing_course.get().is_some() { "Updating..." } else { "Creating..." }
                                                } else {
                                                    if editing_course.get().is_some() { "Update Course" } else { "Create Course" }
                                                }}
                                            </button>
                                        </div>
                                    </form>
                                </div>
                            </div>
                        </div>
                    </Show>

                    // Enrollment Form Modal
                    <Show when=show_enrollment_form>
                        <div class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
                            <div class="relative top-20 mx-auto p-5 border w-11/12 max-w-2xl shadow-lg rounded-md bg-white">
                                <div class="mt-3">
                                    <h3 class="text-lg font-medium text-gray-900 mb-4">
                                        {move || match enrollment_action.get() {
                                            EnrollmentAction::QuickEnroll => "Quick Enroll Student",
                                            EnrollmentAction::AddNew => "Add New Enrollment",
                                        }}
                                    </h3>

                                    // Show selected course info for quick enroll
                                    <Show when=move || enrollment_action.get() == EnrollmentAction::QuickEnroll && selected_course_for_enrollment.get().is_some()>
                                        <div class="mb-4 p-3 bg-blue-50 border border-blue-200 rounded-md">
                                            <h4 class="font-medium text-blue-900">"Selected Course:"</h4>
                                            <p class="text-blue-800">
                                                {move || selected_course_for_enrollment.get().map(|c| format!("{} - {} ({})", c.course_code, c.name, c.subject)).unwrap_or_default()}
                                            </p>
                                        </div>
                                    </Show>

                                    <form on:submit=move |ev| {
                                        ev.prevent_default();
                                        #[cfg(feature = "hydrate")]
                                        {
                                            let form = ev.target().unwrap().dyn_into::<web_sys::HtmlFormElement>().unwrap();
                                            let form_data = web_sys::FormData::new_with_form(&form).unwrap();

                                            let enrollment_data = EnrollmentFormData {
                                                student_id: form_data.get("student_id").as_string().unwrap_or_default(),
                                                course_id: if let Some(course) = selected_course_for_enrollment.get() {
                                                    course.id.to_string()
                                                } else {
                                                    form_data.get("course_id").as_string().unwrap_or_default()
                                                },
                                                academic_year: form_data.get("academic_year").as_string()
                                                    .and_then(|s| AcademicYear::iter().find(|y| y.to_string() == s))
                                                    .unwrap_or(AcademicYear::Year2024_2025),
                                                grade_level: form_data.get("grade_level").as_string()
                                                    .and_then(|s| GradeEnum::iter().find(|g| g.to_string() == s))
                                                    .unwrap_or(GradeEnum::Kindergarten),
                                                teacher_id: form_data.get("teacher_id").as_string().unwrap_or_default(),
                                                status: form_data.get("status").as_string()
                                                    .and_then(|s| EnrollmentStatus::iter().find(|st| st.to_string() == s))
                                                    .unwrap_or(EnrollmentStatus::Active),
                                                enrollment_date: Local::now().date_naive(),
                                                status_change_date: Some(Local::now().date_naive()),
                                                notes: Some(form_data.get("notes").as_string().unwrap_or_default()),
                                            };

                                            create_enrollment_action.dispatch(enrollment_data);
                                        }

                                    }>
                                        <div class="grid grid-cols-2 gap-4">
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700">"Student ID"</label>
                                                <input
                                                    type="number"
                                                    required
                                                    class="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                                                    name="student_id"
                                                    placeholder="Enter student ID"
                                                />
                                            </div>

                                            // Show course selection only for new enrollment
                                            <Show when=move || enrollment_action.get() == EnrollmentAction::AddNew>
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700">"Course"</label>
                                                    <select
                                                        required
                                                        class="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                                                        name="course_id"
                                                    >
                                                        <option value="">"Select a course"</option>
                                                        {move || courses.get().into_iter().map(|course| {
                                                            view! {
                                                                <option value=course.id.to_string()>
                                                                    {format!("{} - {} ({})", course.course_code, course.name, course.subject)}
                                                                </option>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </select>
                                                </div>
                                            </Show>

                                            <div>
                                                <label class="block text-sm font-medium text-gray-700">"Teacher ID"</label>
                                                <input
                                                    type="number"
                                                    required
                                                    class="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                                                    name="teacher_id"
                                                    placeholder="Enter teacher ID"
                                                />
                                            </div>

                                            <div>
                                                <label class="block text-sm font-medium text-gray-700">"Academic Year"</label>
                                                <select
                                                    required
                                                    class="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                                                    name="academic_year"
                                                >
                                                    {
                                                        AcademicYear::iter().map(|year| {
                                                            let is_selected = year == AcademicYear::Year2024_2025;
                                                            view! {
                                                                <option value=year.to_string() selected=is_selected>{year.to_string()}</option>
                                                            }
                                                        }).collect::<Vec<_>>()
                                                    }
                                                </select>
                                            </div>

                                            <div>
                                                <label class="block text-sm font-medium text-gray-700">"Grade Level"</label>
                                                <select
                                                    required
                                                    class="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                                                    name="grade_level"
                                                >
                                                    {
                                                        GradeEnum::iter().map(|level| {
                                                            let is_selected = level == GradeEnum::Kindergarten;
                                                            view! {
                                                                <option value=level.to_string() selected=is_selected>{level.to_string()}</option>
                                                            }
                                                        }).collect::<Vec<_>>()
                                                    }
                                                </select>
                                            </div>

                                            <div>
                                                <label class="block text-sm font-medium text-gray-700">"Status"</label>
                                                <select
                                                    required
                                                    class="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                                                    name="status"
                                                >
                                                    {
                                                        EnrollmentStatus::iter().map(|status| {
                                                            let is_selected = status == EnrollmentStatus::Active;
                                                            view! {
                                                                <option value=status.to_string() selected=is_selected>{status.to_string()}</option>
                                                            }
                                                        }).collect::<Vec<_>>()
                                                    }
                                                </select>
                                            </div>
                                        </div>

                                        // Form Actions
                                        <div class="flex justify-end space-x-3 mt-6">
                                            <button
                                                type="button"
                                                class="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                                                on:click=move |_| {
                                                    set_show_enrollment_form.set(false);
                                                    set_selected_course_for_enrollment.set(None);
                                                }
                                            >
                                                "Cancel"
                                            </button>
                                            <button
                                                type="submit"
                                                class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                                                disabled=move || create_enrollment_action.pending().get()
                                            >
                                                {move || if create_enrollment_action.pending().get() {
                                                    "Creating..."
                                                } else {
                                                    match enrollment_action.get() {
                                                        EnrollmentAction::QuickEnroll => "Enroll Student",
                                                        EnrollmentAction::AddNew => "Create Enrollment",
                                                    }
                                                }}
                                            </button>
                                        </div>
                                    </form>
                                </div>
                            </div>
                        </div>
                    </Show>

                    // Loading states
                    <Show when=move || load_courses.pending().get() || load_enrollments.pending().get()>
                        <div class="fixed inset-0 bg-gray-600 bg-opacity-50 flex items-center justify-center z-40">
                            <div class="bg-white p-6 rounded-lg shadow-lg">
                                <div class="flex items-center space-x-2">
                                    <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-indigo-600"></div>
                                    <span class="text-gray-700">"Loading..."</span>
                                </div>
                            </div>
                        </div>
                    </Show>

                    // Error handling and notifications could be added here
                    <Show when=move || delete_course_action.value().get().flatten().is_none() && !delete_course_action.pending().get() && delete_course_action.value().get().is_some()>
                        <div class="fixed top-4 right-4 bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded z-50">
                            <span class="font-medium">"Error: "</span>
                            "Failed to delete course. Please try again."
                        </div>
                    </Show>

                    <Show when=move || save_course_action.value().get() == Some(false)>
                        <div class="fixed top-4 right-4 bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded z-50">
                            <span class="font-medium">"Error: "</span>
                            "Failed to save course. Please check all fields and try again."
                        </div>
                    </Show>

                    <Show when=move || update_enrollment_status_action.value().get() == Some(false)>
                        <div class="fixed top-4 right-4 bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded z-50">
                            <span class="font-medium">"Error: "</span>
                            "Failed to update enrollment status. Please try again."
                        </div>
                    </Show>

                    <Show when=move || delete_enrollment_action.value().get().flatten().is_none() && !delete_enrollment_action.pending().get() && delete_enrollment_action.value().get().is_some()>
                        <div class="fixed top-4 right-4 bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded z-50">
                            <span class="font-medium">"Error: "</span>
                            "Failed to delete enrollment. Please try again."
                        </div>
                    </Show>

                    <Show when=move || create_enrollment_action.value().get() == Some(false)>
                        <div class="fixed top-4 right-4 bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded z-50">
                            <span class="font-medium">"Error: "</span>
                            "Failed to create enrollment. Please check all fields and try again."
                        </div>
                    </Show>

                    // Success notification for enrollment creation
                    <Show when=move || create_enrollment_action.value().get() == Some(true) && !create_enrollment_action.pending().get()>
                        <div class="fixed top-4 right-4 bg-green-100 border border-green-400 text-green-700 px-4 py-3 rounded z-50">
                            <span class="font-medium">"Success: "</span>
                            "Student enrolled successfully!"
                        </div>
                    </Show>
                </div>
            </div>
        </div>
    }
}
