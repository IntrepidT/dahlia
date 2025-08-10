use crate::app::components::auth::enhanced_login_form::{
    use_student_mapping_service, DeAnonymizedStudent,
};
use crate::app::middleware::global_settings::use_settings;
use crate::app::server_functions::students::get_students;
use leptos::prelude::*;
use leptos::prelude::*;
use log;

#[component]
pub fn StudentSelect(set_selected_student_id: WriteSignal<Option<i32>>) -> impl IntoView {
    let (settings, _) = use_settings();
    let anonymization_enabled = move || settings.get().student_protections;
    let (student_mapping_service, _) = use_student_mapping_service();

    let get_students_action = Action::new(|_: &()| async move {
        match get_students().await {
            Ok(fetched_students) => fetched_students,
            Err(e) => {
                log::error!("Failed to fetch students: {}", e);
                Vec::new()
            }
        }
    });

    let enhanced_students = Memo::new(move |_| {
        let students_data = get_students_action
            .value()
            .get()
            .as_ref()
            .cloned()
            .unwrap_or_default();

        if anonymization_enabled() {
            let mapping_service = student_mapping_service.get();
            students_data
                .into_iter()
                .map(|student| {
                    let de_anon = DeAnonymizedStudent::from_student_with_mapping(
                        &student,
                        mapping_service.as_ref(),
                    );
                    (student, Some(de_anon))
                })
                .collect::<Vec<_>>()
        } else {
            students_data
                .into_iter()
                .map(|student| (student, None))
                .collect::<Vec<_>>()
        }
    });

    Effect::new(move |_| {
        get_students_action.dispatch(());
    });

    view! {
        <div class="mb-2 max-w-[20rem]">
            <label class="block text-sm font-medium mb-1">"Select Student:"</label>
            <select
                class="w-full p-2 border rounded-md"
                on:change=move |ev| {
                    let value = event_target_value(&ev).parse().ok();
                    set_selected_student_id.set(value);
                }
            >
                <option value="">"Select a student..."</option>
                <Suspense fallback=move || view! {
                    <option>"Loading students..."</option>
                }>
                    {move || {
                        enhanced_students().into_iter().map(|(student, de_anon_opt)| {
                            let display_text = if let Some(de_anon) = &de_anon_opt {
                                format!("{} - {}", de_anon.display_name, de_anon.display_id)
                            } else {
                                format!(
                                    "{} {} - {}",
                                    student.firstname.as_ref().unwrap_or(&"Unknown".to_string()),
                                    student.lastname.as_ref().unwrap_or(&"Unknown".to_string()),
                                    student.student_id
                                )
                            };

                            view! {
                                <option value={student.student_id.to_string()}>
                                    {display_text}
                                </option>
                            }
                        }).collect_view()
                    }}
                </Suspense>
            </select>
        </div>
    }
}
