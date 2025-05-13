use crate::app::components::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::header::Header;
use crate::app::server_functions::assessments::get_assessments;
use crate::app::server_functions::students::get_students;
use leptos::*;
use leptos_router::*;

#[component]
pub fn Gradebook() -> impl IntoView {
    let (refresh_trigger, set_refresh_trigger) = create_signal(0);
    let (selected_view, set_selected_view) = create_signal(SidebarSelected::Dashboard);
    let (search_term, set_search_term) = create_signal(String::new());
    let (selected_assessment, set_selected_assessment) = create_signal(String::new());

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
    let assessment_list = create_resource(move || (), |_| async move { get_assessments().await });

    let filtered_students = create_memo(move |_| {
        let search = search_term().trim().to_lowercase();

        students
            .get()
            .unwrap_or(None)
            .unwrap_or_default()
            .into_iter()
            .filter(|student| {
                search.is_empty()
                    || student.firstname.to_lowercase().contains(&search)
                    || student.lastname.to_lowercase().contains(&search)
            })
            .collect::<Vec<_>>()
    });

    view! {
        <div class="h-screen flex flex-col bg-[#F9F9F8] overflow-hidden">
            <Header />
            <DashboardSidebar
                selected_item=selected_view
                set_selected_item=set_selected_view
            />
            <div class="w-full flex-1 mt-16 ml-20 px-5 mr-20 flex flex-col">
                <h1 class="text-xl font-bold my-2">"Gradebook"</h1>

                <div class="flex justify-between items-center mb-2">
                    <div class="w-2/3 mr-4">
                        <input
                            type="text"
                            placeholder="Search students..."
                            prop:value={move || search_term.get()}
                            class="border border-gray-300 rounded px-3 py-1 w-full text-sm"
                            on:input=move |ev| set_search_term(event_target_value(&ev))
                        />
                    </div>
                    <div class="w-1/3">
                        <select
                            id="assessment-select"
                            class="block w-full px-2 py-1 bg-white border-gray-200 rounded-md border text-sm"
                            on:change=move |ev| set_selected_assessment(event_target_value(&ev))
                        >
                           <option value="all">All Assessments</option>
                           {move || match assessment_list.get(){
                                None => view!{<option>"Loading..."</option>}.into_view(),
                                Some(Ok(list)) => list.into_iter().map(|assessment| {
                                    view! {
                                        <option value={assessment.name.clone()}>{assessment.name}</option>
                                    }
                                }).collect_view(),
                                Some(Err(e)) => view! {<option>"Error: " {e.to_string()}</option>}.into_view(),
                            }}
                        </select>
                    </div>
                </div>
                <div class="flex-1 overflow-hidden">
                    <div class="h-full overflow-auto">
                        <table class="w-full bg-white border border-gray-200 table-fixed">
                            <thead class="sticky top-0 bg-white">
                                <tr>
                                    <th class="px-2 py-1 border-b text-left font-medium text-[#2E3A59] text-xs">"Student Name"</th>
                                    <th class="px-2 py-1 border-b text-left font-medium text-[#2E3A59] text-xs">"Id"</th>
                                </tr>
                            </thead>
                            <tbody class="text-xs">
                                {move || {
                                    let students = filtered_students();
                                    if students.is_empty() {
                                        view! {
                                            <tr>
                                                <td colspan="2" class="px-2 py-1 border-b">
                                                    "No students match your search criteria."
                                                </td>
                                            </tr>
                                        }.into_view()
                                    } else {
                                        students.into_iter().map(|student| {
                                            view! {
                                                <tr>
                                                    <td class="px-2 py-1 border-b whitespace-nowrap">{format!("{} {}", &student.firstname, &student.lastname)}</td>
                                                    <td class="px-2 py-1 border-b whitespace-nowrap">{&student.student_id.to_string()}</td>
                                                </tr>
                                            }
                                        }).collect_view()
                                    }
                                }}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}
