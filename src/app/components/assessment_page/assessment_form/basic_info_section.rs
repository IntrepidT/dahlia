use crate::app::components::assessment_page::shared::types::AssessmentFormState;
use crate::app::models::assessment::{ScopeEnum, SubjectEnum};
use crate::app::models::student::GradeEnum;
use leptos::html;
use leptos::prelude::*;
use strum::IntoEnumIterator;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

#[component]
pub fn BasicInfoSection(
    state: ReadSignal<AssessmentFormState>,
    set_state: WriteSignal<AssessmentFormState>,
    courses_resource: Resource<Result<Vec<crate::app::models::course::Course>, ServerFnError>>,
) -> impl IntoView {
    // Create node refs for the select elements
    let subject_ref = NodeRef::<html::Select>::new();
    let grade_ref = NodeRef::<html::Select>::new();
    let scope_ref = NodeRef::<html::Select>::new();

    // Effect to update subject select when state changes
    Effect::new(move |_| {
        let current_state = state.get();
        if let Some(subject_element) = subject_ref.get() {
            match current_state.subject {
                Some(subject) => {
                    let formatted = format!("{}", subject);
                    log::info!("Setting subject select to: {}", formatted);
                    subject_element.set_value(&formatted);
                }
                None => {
                    log::info!("Setting subject select to empty");
                    subject_element.set_value("");
                }
            }
        }
    });

    // Effect to update grade select when state changes
    Effect::new(move |_| {
        let current_state = state.get();
        if let Some(grade_element) = grade_ref.get() {
            match current_state.grade {
                Some(grade) => {
                    let formatted = format!("{}", grade);
                    log::info!("Setting grade select to: {}", formatted);
                    grade_element.set_value(&formatted);
                }
                None => {
                    log::info!("Setting grade select to empty");
                    grade_element.set_value("");
                }
            }
        }
    });

    // Effect to update scope select when state changes
    Effect::new(move |_| {
        let current_state = state.get();
        if let Some(scope_element) = scope_ref.get() {
            match current_state.scope {
                Some(scope) => {
                    let formatted = format!("{}", scope);
                    log::info!("Setting scope select to: {}", formatted);
                    scope_element.set_value(&formatted);
                }
                None => {
                    log::info!("Setting scope select to empty");
                    scope_element.set_value("");
                }
            }
        }
    });

    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6 bg-white p-4 rounded-lg">
            <div>
                <label for="name" class="block text-sm font-medium mb-1 text-gray-700">"Name"</label>
                <input
                    type="text"
                    id="name"
                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-[#2E3A59] focus:border-[#2E3A59] text-gray-900 bg-white"
                    prop:value={move || state.get().name}
                    on:input=move |ev| set_state.update(|s| s.name = event_target_value(&ev))
                    required
                />
            </div>

            <div>
                <label for="subject" class="block text-sm font-medium mb-1 text-gray-700">"Subject"</label>
                <select
                    node_ref=subject_ref
                    id="subject"
                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-[#2E3A59] focus:border-[#2E3A59] bg-white text-gray-900"
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        log::info!("Subject select changed to: {}", value);
                        if value.is_empty() {
                            set_state.update(|s| s.subject = None);
                        } else {
                            match value.parse::<SubjectEnum>() {
                                Ok(subject_enum) => set_state.update(|s| s.subject = Some(subject_enum)),
                                Err(_) => log::info!("Failed to parse subject: {}", value)
                            }
                        }
                    }
                >
                    <option value="" class="text-gray-900 bg-white">"None"</option>
                    {SubjectEnum::iter().map(|option| {
                        let option_value = format!("{}", option);
                        view! {
                            <option value=option_value.clone() class="text-gray-900 bg-white">
                                {option_value.clone()}
                            </option>
                        }
                    }).collect::<Vec<_>>()}
                </select>
            </div>

            <div>
                <label for="grade" class="block text-sm font-medium mb-1 text-gray-700">"Grade"</label>
                <select
                    node_ref=grade_ref
                    id="grade"
                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-[#2E3A59] focus:border-[#2E3A59] bg-white text-gray-900"
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        log::info!("Grade select changed to: {}", value);
                        if value.is_empty() {
                            set_state.update(|s| s.grade = None);
                        } else {
                            match value.parse::<GradeEnum>() {
                                Ok(grade_enum) => set_state.update(|s| s.grade = Some(grade_enum)),
                                Err(_) => log::info!("Failed to parse grade: {}", value)
                            }
                        }
                    }
                >
                    <option value="" class="text-gray-900 bg-white">"None"</option>
                    {GradeEnum::iter().map(|grade| {
                        let option_value = format!("{}", grade);
                        view! {
                            <option value=option_value.clone() class="text-gray-900 bg-white">
                                {option_value.clone()}
                            </option>
                        }
                    }).collect::<Vec<_>>()}
                </select>
            </div>

            <div class="grid grid-cols-2 gap-4">
                <div>
                    <label for="frequency" class="block text-sm font-medium mb-1 text-gray-700">"Frequency (per year)"</label>
                    <input
                        type="number"
                        id="frequency"
                        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-[#2E3A59] focus:border-[#2E3A59] text-gray-900 bg-white"
                        prop:value={move || state.get().frequency.unwrap_or(0)}
                        on:input=move |ev| {
                            let value = event_target_value(&ev);
                            if value.is_empty() {
                                set_state.update(|s| s.frequency = None);
                            } else if let Ok(f) = value.parse::<i32>() {
                                set_state.update(|s| s.frequency = Some(f));
                            }
                        }
                    />
                </div>

                <div>
                    <label for="version" class="block text-sm font-medium mb-1 text-gray-700">"Version"</label>
                    <input
                        type="number"
                        id="version"
                        min="1"
                        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-[#2E3A59] focus:border-[#2E3A59] text-gray-900 bg-white"
                        prop:value={move || state.get().version}
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                set_state.update(|s| s.version = v);
                            }
                        }
                        required
                    />
                </div>
            </div>

            <div>
                <label for="scope" class="block text-sm font-medium mb-1 text-gray-700">"Scope"</label>
                <select
                    node_ref=scope_ref
                    id="scope"
                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-[#2E3A59] focus:border-[#2E3A59] bg-white text-gray-900"
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        log::info!("Scope select changed to: {}", value);
                        if value.is_empty() {
                            set_state.update(|s| s.scope = None);
                        } else {
                            match value.parse::<ScopeEnum>() {
                                Ok(scope_enum) => set_state.update(|s| s.scope = Some(scope_enum)),
                                Err(_) => log::info!("Failed to parse scope: {}", value)
                            }
                        }
                    }
                >
                    <option value="" class="text-gray-900 bg-white">"None"</option>
                    {ScopeEnum::iter().map(|option| {
                        let option_value = format!("{}", option);
                        view! {
                            <option value=option_value.clone() class="text-gray-900 bg-white">
                                {option_value.clone()}
                            </option>
                        }
                    }).collect::<Vec<_>>()}
                </select>
            </div>

            // Course selection (show when scope is Course)
            <Show when=move || state.get().scope == Some(ScopeEnum::Course)>
                <div>
                    <label for="course" class="block text-sm font-medium mb-1 text-gray-700">"Course"</label>
                    <select
                        id="course"
                        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-[#2E3A59] focus:border-[#2E3A59] bg-white text-gray-900"
                        prop:value={move || state.get().course_id.map(|id| id.to_string()).unwrap_or_default()}
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            if let Ok(course_id) = value.parse::<i32>() {
                                set_state.update(|s| s.course_id = Some(course_id));
                            } else {
                                set_state.update(|s| s.course_id = None);
                            }
                        }
                    >
                        <option value="">"Select a course"</option>
                        {move || {
                            courses_resource.get().map(|courses_result| {
                                match courses_result {
                                    Ok(courses) => {
                                        courses.into_iter().map(|course| {
                                            view! {
                                                <option value=course.id.to_string()>
                                                    {course.name}
                                                </option>
                                            }
                                        }).collect_view().into_any()
                                    },
                                    Err(_) => view! {}.into_any()
                                }
                            }).unwrap_or_else(|| view! {}.into_any())
                        }}
                    </select>
                </div>
            </Show>
        </div>
    }
}
