use leptos::prelude::*;
use super::types::{ConnectedStudent, Role};
use leptos::prelude::*;

#[component]
pub fn ParticipantsList(
    #[prop(into)] connected_students: Signal<Vec<ConnectedStudent>>,
    #[prop(into)] role: Signal<Role>,
) -> impl IntoView {
    view! {
        <div class="mb-8 max-w-4xl mx-auto">
            <h3 class="text-lg font-medium mb-2">
                {move || match role.get() {
                    Role::Teacher => "Connected Students",
                    Role::Student => "Connected Participants",
                    Role::Unknown => "Session Participants"
                }}
            </h3>
            <div class="bg-white shadow-sm rounded-lg p-4">
                <Show when=move || !connected_students.get().is_empty() fallback=|| view! {
                    <p class="text-gray-500 text-center py-2">"No participants connected"</p>
                }>
                    <ul class="divide-y divide-gray-200">
                        <For
                            each=move || connected_students.get()
                            key=|student| student.student_id.clone()
                            children=move |student| {
                                let status_class = if student.status == "Connected" {
                                    "bg-green-100 text-green-800"
                                } else {
                                    "bg-red-100 text-red-800"
                                };

                                view! {
                                    <li class="py-2 flex justify-between items-center">
                                        <span>{student.name.clone()}</span>
                                        <span class=format!("text-sm px-2 py-1 rounded-full {}", status_class)>
                                            {student.status.clone()}
                                        </span>
                                    </li>
                                }
                            }
                        />
                    </ul>
                </Show>
            </div>
        </div>
    }
}
