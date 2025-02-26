use crate::app::components::header::Header;
use crate::app::components::student_page::student_search_filter::{FilterState, SearchFilter}; // Import the new component
use crate::app::models::student::{
    AddStudentRequest, DeleteStudentRequest, ELLEnum, GenderEnum, GradeEnum, Student,
};
use crate::app::models::EmployeeRole;
use crate::app::server_functions::get_teachers;
use crate::app::server_functions::students::{add_student, delete_student, get_students};
use chrono::NaiveDate;
use leptos::ev::SubmitEvent;
use leptos::*;
use std::rc::Rc;
use std::str::FromStr;
use strum::IntoEnumIterator;
use validator::Validate;

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
                Ok(teachers) => Some(teachers),
                Err(e) => {
                    log::error!("Failed to fetch teachers: {}", e);
                    Some(vec![])
                }
            }
        },
    );

    let (selected_student, set_selected_student) = create_signal(None::<Rc<Student>>);

    // Filter state signals
    let (search_term, set_search_term) = create_signal(String::new());
    let (grade_filter, set_grade_filter) = create_signal(String::from("all"));
    let (iep_filter, set_iep_filter) = create_signal(false);
    let (ell_filter, set_ell_filter) = create_signal(false);
    let (teacher_filter, set_teacher_filter) = create_signal(String::from("all"));

    //Signals for error messaging
    let (error_message, set_error_message) = create_signal(String::new());
    let (if_error, set_if_error) = create_signal(false);

    //Signals for getting a new student
    let (adding_student, set_adding_student) = create_signal(false);

    let (new_firstname, set_new_firstname) = create_signal(String::new());
    let (new_lastname, set_new_lastname) = create_signal(String::new());
    let (new_student_gender, set_student_gender) = create_signal(String::new());
    let (new_student_dob, set_student_dob) = create_signal(String::new());
    let (new_student_id, set_new_student_id) = create_signal(String::new());
    let (new_grade, set_new_grade) = create_signal(String::new());
    let (new_teacher, set_new_teacher) = create_signal(String::new());
    let (new_iep, set_new_iep) = create_signal(false);
    let (new_504, set_new_504) = create_signal(false);
    let (yes_no_ell, set_yes_no_ell) = create_signal(false);
    let (new_ell, set_new_ell) = create_signal(String::from("Not Applicable"));
    let (new_gt, set_new_gt) = create_signal(false);
    let (new_readplan, set_new_readplan) = create_signal(false);
    let (new_intervention, set_new_intervention) = create_signal(false);
    let (new_eye_glasses, set_new_eye_glasses) = create_signal(false);

    //Add edit mode signal
    let (show_form_modal, set_show_form_modal) = create_signal(false);
    let (is_editing, set_is_editing) = create_signal(false);

    //Delete Student Signal
    let (confirm_delete_one, set_confirm_delete_one) = create_signal(false);
    let (confirm_delete_two, set_confirm_delete_two) = create_signal(String::new());

    // Extract teacher names for the filter dropdown
    let teacher_names = create_memo(move |_| {
        teachers
            .get()
            .unwrap_or(Some(vec![]))
            .unwrap_or_default()
            .iter()
            .map(|teacher| teacher.lastname.clone())
            .collect::<Vec<_>>()
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

                        let matches_ell = !ell_filter() || student.ell != ELLEnum::NotApplicable;

                        let matches_teacher =
                            teacher_filter() == "all" || student.teacher == teacher_filter();

                        matches_search
                            && matches_grade
                            && matches_iep
                            && matches_ell
                            && matches_teacher
                    })
                    .collect::<Vec<_>>()
            })
        })
    };

    // Create a derived signal for filtered teachers based on selected grade
    let filtered_teachers = create_memo(move |_| {
        let grade_str = new_grade();
        if grade_str.is_empty() {
            return Vec::new(); // Return empty if no grade selected yet
        }

        // Convert the selected grade string to GradeEnum
        let selected_grade = match GradeEnum::from_str(&grade_str) {
            Ok(grade) => grade,
            Err(_) => return Vec::new(), // Return empty on error
        };

        // Filter teachers to only include those matching the selected grade
        teachers
            .get()
            .unwrap_or(Some(Vec::new()))
            .unwrap_or_default()
            .into_iter()
            .filter(|teacher| {
                // Check if teacher has EmployeeRole::Teacher{grade} matching selected_grade
                match &teacher.role {
                    EmployeeRole::Teacher { grade } => *grade == Some(selected_grade.clone()),
                    _ => false,
                }
            })
            .collect::<Vec<_>>()
    });

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

    let handle_submit_new_student = move |ev: SubmitEvent| {
        ev.prevent_default();

        let validated_student_id = new_student_id().parse::<i32>();
        let validated_dob: NaiveDate = NaiveDate::parse_from_str(&new_student_dob(), "%Y-%m-%d")
            .expect("Issue gathering date");

        let convert_gender_to_enum = match GenderEnum::from_str(&new_student_gender()) {
            Ok(gender_enum) => gender_enum,
            Err(_) => {
                log::error!("Invalid gender value submitted for new student");
                return;
            }
        };
        let convert_grade_to_enum = match GradeEnum::from_str(&new_grade()) {
            Ok(grade_enum) => grade_enum,
            Err(_) => {
                log::error!("Invalid grade value submitted for new student");
                return;
            }
        };
        let convert_ell_to_enum = ELLEnum::from_str(&new_ell()).clone().unwrap();

        let add_student_request = AddStudentRequest {
            firstname: new_firstname(),
            lastname: new_lastname(),
            gender: convert_gender_to_enum,
            date_of_birth: validated_dob,
            student_id: validated_student_id.expect("Student ID was not processed correctly"),
            ell: convert_ell_to_enum,
            grade: convert_grade_to_enum,
            teacher: new_teacher(),
            iep: new_iep(),
            student_504: new_504(),
            readplan: new_readplan(),
            gt: new_gt(),
            intervention: new_intervention(),
            eye_glasses: new_eye_glasses(),
        };

        let is_valid = add_student_request.validate();

        match is_valid {
            Ok(_) => {
                spawn_local(async move {
                    let add_result = add_student(add_student_request).await;

                    //we get the result back and do something with it
                    match add_result {
                        Ok(_added_result) => {
                            set_adding_student(false);
                            set_refresh_trigger.update(|count| *count += 1);
                            log::info!("Student added successfully");
                        }
                        Err(e) => println!("Error adding: {:?}", e),
                    };
                });
            }
            Err(_) => {
                set_if_error(true);
                set_error_message(String::from("All fields required"))
            }
        }
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
                    set_ell_filter=set_ell_filter
                    teachers=teacher_names()
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
                    when=move || selected_student().is_some() || adding_student()
                    fallback=|| view! {
                        <div class="flex items-center justify-center border-t-8 border-[#00356b] h-full text-gray-500 rounded-lg shadow-lg">
                            "Select a student to view details"
                        </div>
                    }
                >
                    {move || {
                        if adding_student() {
                            view! {
                                <div class=INFO_CONTAINER_STYLE>
                                    <Show when=move || {if_error()}>
                                        <p class="text-red-500 font-semibold">"There was an error with one or more of the entered fields"</p>
                                        <p class="text-red-500 rounded w-full h-12 px-5 -y-3">{error_message()}</p>
                                    </Show>
                                    <h2 class="text-xl font-bold mb-4">"Add New Student"</h2>
                                    <form on:submit=handle_submit_new_student class=INFO_CONTENT_STYLE>
                                        <div class="grid grid-cols-2 gap-4">
                                            // Basic Information Section
                                            <div class="col-span-2">
                                                <h3 class="text-sm font-semibold text-gray-600 mb-2">"Basic Information"</h3>
                                                <div class="grid grid-cols-2 gap-4 bg-gray-50 p-4 rounded-lg">
                                                    <div class=INFO_GROUP_STYLE>
                                                        <label class=INFO_TITLE_STYLE for="firstname">"First Name"</label>
                                                        <input
                                                            id="firstname"
                                                            type="text"
                                                            class="mt-1 w-full rounded-md border p-2"
                                                            on:input=move |ev| set_new_firstname(event_target_value(&ev))
                                                            required
                                                        />
                                                    </div>
                                                    <div class=INFO_GROUP_STYLE>
                                                        <label class=INFO_TITLE_STYLE for="lastname">"Last Name"</label>
                                                        <input
                                                            id="lastname"
                                                            type="text"
                                                            class="mt-1 w-full rounded-md border p-2"
                                                            on:input=move |ev| set_new_lastname(event_target_value(&ev))
                                                            required
                                                        />
                                                    </div>
                                                    <div class=INFO_GROUP_STYLE>
                                                        <label class=INFO_TITLE_STYLE for="student-id">"Student ID"</label>
                                                        <input
                                                            required
                                                            id="student-id"
                                                            type="text"
                                                            class="mt-1 w-full rounded-md border p-2"
                                                            on:input=move |ev| set_new_student_id(event_target_value(&ev))
                                                        />
                                                    </div>
                                                    <div class=INFO_GROUP_STYLE>
                                                        <label class=INFO_TITLE_STYLE for="gender">"Gender"</label>
                                                        <select
                                                            required
                                                            id="gender"
                                                            class="mt-1 w-full rounded-md border p-2"
                                                            on:change=move |ev| set_student_gender(event_target_value(&ev))
                                                        >
                                                            <option value="">"Please select a value"</option>
                                                            {GenderEnum::iter().map(|grade| view! {
                                                                <option value=format!("{}", grade)>
                                                                    {format!("{}", grade)}
                                                                </option>
                                                            }).collect::<Vec<_>>()}
                                                        </select>
                                                    </div>

                                                    <div class=INFO_GROUP_STYLE>
                                                        <label class=INFO_TITLE_STYLE for="grade">"Grade"</label>
                                                        <select
                                                            required
                                                            id="grade"
                                                            class="mt-1 w-full rounded-md border p-2"
                                                            on:change=move |ev| set_new_grade(event_target_value(&ev))
                                                        >
                                                            <option value="">"Please select a value"</option>
                                                            {GradeEnum::iter().map(|grade| view! {
                                                                <option value=format!("{}", grade)>
                                                                    {format!("{}", grade)}
                                                                </option>
                                                            }).collect::<Vec<_>>()}
                                                        </select>
                                                    </div>
                                                    <div class=INFO_GROUP_STYLE>
                                                        <label class=INFO_TITLE_STYLE for="birthdate">"Birthdate"</label>
                                                        <input
                                                            type="date"
                                                            required
                                                            id="birthdate"
                                                            class="mt-1 w-full rounded-md border p-2"
                                                            on:change=move |ev| set_student_dob(event_target_value(&ev))
                                                        />
                                                    </div>
                                                    <div class=INFO_GROUP_STYLE>
                                                        <label class=INFO_TITLE_STYLE for="teacher">"Teacher"</label>
                                                        <select
                                                            required
                                                            id="teacher"
                                                            class="mt-1 w-full rounded-md border p-2"
                                                            on:change=move |ev| set_new_teacher(event_target_value(&ev))
                                                        >
                                                            <option value="">"Please select a value"</option>
                                                            {move || {
                                                                if new_grade().is_empty() {
                                                                    vec![view! { <option disabled>"First select a grade"</option> }].into_iter().collect_view()
                                                                } else {
                                                                    let filtered = filtered_teachers();
                                                                    if filtered.is_empty() {
                                                                        vec![view! { <option disabled>"No teachers available for this grade"</option> }].into_iter().collect_view()
                                                                    } else {
                                                                        filtered.iter().map(|teacher| view! {
                                                                            <option value=teacher.lastname.clone()>{teacher.lastname.clone()}</option>
                                                                        }).collect_view()
                                                                    }
                                                                }
                                                            }}
                                                        </select>
                                                    </div>
                                                </div>
                                            </div>

                                            // Support Services Section
                                            <div class="col-span-2">
                                                <h3 class="text-sm font-semibold text-gray-600 mb-2">"Support Services"</h3>
                                                <div class="grid grid-cols-2 gap-4 bg-gray-50 p-4 rounded-lg">
                                                    <div class=INFO_GROUP_STYLE>
                                                        <label class="flex items-center gap-2">
                                                            <input
                                                                type="checkbox"
                                                                class="form-checkbox h-5 w-5"
                                                                on:change=move |ev| set_new_iep(event_target_checked(&ev))
                                                            />
                                                            <span class=INFO_TITLE_STYLE>"IEP"</span>
                                                        </label>
                                                    </div>
                                                    <div class=INFO_GROUP_STYLE>
                                                        <label class="flex items-center gap-2">
                                                            <input
                                                                type="checkbox"
                                                                class="form-checkbox h-5 w-5"
                                                                on:change=move |ev| set_new_504(event_target_checked(&ev))
                                                            />
                                                            <span class=INFO_TITLE_STYLE>"504"</span>
                                                        </label>
                                                    </div>
                                                    <div class=INFO_GROUP_STYLE>
                                                        <label class="flex items-center gap-2">
                                                            <input
                                                                type="checkbox"
                                                                class="form-checkbox h-5 w-5"
                                                                on:change=move |ev| set_yes_no_ell(event_target_checked(&ev))
                                                            />
                                                            <span class=INFO_TITLE_STYLE>"ELL"</span>
                                                        </label>
                                                        <Show when=move || {yes_no_ell()}>
                                                            <select class="p-3 rounded-lg"
                                                                required
                                                                value=new_ell
                                                                on:change=move |event| {
                                                                    set_new_ell(event_target_value(&event))
                                                                }
                                                            >
                                                                <option value="">"Please Select"</option>
                                                                {ELLEnum::iter().map(|lang| view! {
                                                                    <option vallue=format!("{}", lang)>
                                                                        {format!("{}", lang)}
                                                                    </option>
                                                                }).collect::<Vec<_>>()}
                                                            </select>
                                                        </Show>
                                                    </div>
                                                    <div class=INFO_GROUP_STYLE>
                                                        <label class="flex items-center gap-2">
                                                            <input
                                                                type="checkbox"
                                                                class="form-checkbox h-5 w-5"
                                                                on:change=move |ev| set_new_readplan(event_target_checked(&ev))
                                                            />
                                                            <span class=INFO_TITLE_STYLE>"Read Plan"</span>
                                                        </label>
                                                    </div>
                                                    <div class=INFO_GROUP_STYLE>
                                                        <label class="flex items-center gap-2">
                                                            <input
                                                                type="checkbox"
                                                                class="form-checkbox h-5 w-5"
                                                                on:change=move |ev| set_new_gt(event_target_checked(&ev))
                                                            />
                                                            <span class=INFO_TITLE_STYLE>"GT Status"</span>
                                                        </label>
                                                    </div>
                                                    <div class=INFO_GROUP_STYLE>
                                                        <label class="flex items-center gap-2">
                                                            <input
                                                                type="checkbox"
                                                                class="form-checkbox h-5 w-5"
                                                                on:change=move |ev| set_new_intervention(event_target_checked(&ev))
                                                            />
                                                            <span class=INFO_TITLE_STYLE>"Intervention"</span>
                                                        </label>
                                                    </div>
                                                </div>
                                            </div>
                                            //Additional Services
                                            <div class="col-span-2">
                                                <h3 class="text-sm font-semibold text-gray-600 mb-2">"Additional Services"</h3>
                                                <div class="grid grid-cols-2 gap-4 bg-gray-50 p-4 rounded-lg">
                                                    <div class=INFO_GROUP_STYLE>
                                                        <label class="flex items-center gap-2">
                                                            <input
                                                                type="checkbox"
                                                                class="form-checkbox h-5 w-5"
                                                                on:change=move |ev| set_new_eye_glasses(event_target_checked(&ev))
                                                            />
                                                            <span class=INFO_TITLE_STYLE>"Glasses"</span>
                                                        </label>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                        <div class=BUTTON_CONTAINER_STYLE>
                                            <button
                                                type="button"
                                                class="px-4 py-2 bg-gray-200 rounded-lg font-bold hover:bg-gray-300"
                                                on:click=move |_| set_adding_student(false)
                                            >
                                                "Cancel"
                                            </button>
                                            <button
                                                type="submit"
                                                class="px-4 py-2 bg-green-500 text-white font-bold rounded-lg hover:bg-[#A8DCAB]"
                                            >
                                                "Save Student"
                                            </button>
                                        </div>
                                    </form>
                                </div>
                            }
                        } else {
                            let student = selected_student().unwrap();
                            view! {
                                <div class=INFO_CONTAINER_STYLE>
                                    <h2 class="text-xl font-bold mb-4">
                                        {&student.firstname}
                                        {" "}
                                        {&student.lastname}
                                    </h2>

                                    <div class=INFO_CONTENT_STYLE>
                                        <div class="grid grid-cols-2 gap-4">
                                            // Basic Information Section
                                            <div class="col-span-2">
                                                <h3 class="text-sm font-semibold text-gray-600 mb-2">"Basic Information"</h3>
                                                <div class="grid grid-cols-2 gap-4 bg-gray-50 p-4 rounded-lg">
                                                    <div class=INFO_GROUP_STYLE>
                                                        <div class=INFO_TITLE_STYLE>"Student ID"</div>
                                                        <div class=INFO_VALUE_STYLE>{format!("{}", &student.student_id)}</div>
                                                    </div>
                                                    <div class=INFO_GROUP_STYLE>
                                                        <div class=INFO_TITLE_STYLE>"Grade"</div>
                                                        <div class=INFO_VALUE_STYLE>{&student.grade.to_string()}</div>
                                                    </div>
                                                    <div class=INFO_GROUP_STYLE>
                                                        <div class=INFO_TITLE_STYLE>"Teacher"</div>
                                                        <div class=INFO_VALUE_STYLE>{&student.teacher}</div>
                                                    </div>
                                                    <div class=INFO_GROUP_STYLE>
                                                        <div class=INFO_TITLE_STYLE>"Date of Birth"</div>
                                                        <div class=INFO_VALUE_STYLE>{format!("{:?}", &student.date_of_birth)}</div>
                                                    </div>
                                                </div>
                                            </div>

                                            // Support Services Section
                                            <div class="col-span-2">
                                                <h3 class="text-sm font-semibold text-gray-600 mb-2">"Support Services"</h3>
                                                <div class="grid grid-cols-2 gap-4 bg-gray-50 p-4 rounded-lg">
                                                    <div class=INFO_GROUP_STYLE>
                                                        <div class=INFO_TITLE_STYLE>"IEP Status"</div>
                                                        <div class=INFO_VALUE_STYLE>{&student.iep.to_string()}</div>
                                                    </div>
                                                    <div class=INFO_GROUP_STYLE>
                                                        <div class=INFO_TITLE_STYLE>"504 Status"</div>
                                                        <div class=INFO_VALUE_STYLE>{&student.student_504.to_string()}</div>
                                                    </div>
                                                    <div class=INFO_GROUP_STYLE>
                                                        <div class=INFO_TITLE_STYLE>"ELL Status"</div>
                                                        <div class=INFO_VALUE_STYLE>{&student.ell.to_string()}</div>
                                                    </div>
                                                    <div class=INFO_GROUP_STYLE>
                                                        <div class=INFO_TITLE_STYLE>"GT Status"</div>
                                                        <div class=INFO_VALUE_STYLE>{&student.gt.to_string()}</div>
                                                    </div>
                                                    <div class=INFO_GROUP_STYLE>
                                                        <div class=INFO_TITLE_STYLE>"Readplan"</div>
                                                        <div class=INFO_VALUE_STYLE>{&student.readplan.to_string()}</div>
                                                    </div>
                                                    <div class=INFO_GROUP_STYLE>
                                                        <div class=INFO_TITLE_STYLE>"Intervention"</div>
                                                        <div class=INFO_VALUE_STYLE>{&student.intervention.to_string()}</div>
                                                    </div>
                                                </div>
                                            </div>

                                            // Additional Information Section
                                            <div class="col-span-2">
                                                <h3 class="text-sm font-semibold text-gray-600 mb-2">"Additional Information"</h3>
                                                <div class="grid grid-cols-2 gap-4 bg-gray-50 p-4 rounded-lg">
                                                    <div class=INFO_GROUP_STYLE>
                                                        <div class=INFO_TITLE_STYLE>"Eye Glasses"</div>
                                                        <div class=INFO_VALUE_STYLE>{&student.eye_glasses.to_string()}</div>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    // Button container at the bottom
                                    <div class=BUTTON_CONTAINER_STYLE>
                                        <button class="px-4 py-2 bg-[#00356b] text-white rounded-lg font-bold hover:bg-[#7F9AB5]">
                                            "Test Results"
                                        </button>
                                        <button class="px-4 py-2 bg-[#FDF8D4] text-black rounded-lg border-2 border-gray-50 font-bold hover:bg-[#FCFDD4] hover:border-2 hover:border-gray-50">
                                            "Edit Student"
                                        </button>
                                        <button class="px-4 py-2 bg-gray-200 border rounded-lg font-bold hover:bg-gray-100">
                                            "Next Student"
                                        </button>
                                    </div>
                                </div>
                            }
                        }
                    }}
                </Show>
            </div>
        </div>
    }
}
