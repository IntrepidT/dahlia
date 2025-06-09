use crate::app::models::student::{DeleteStudentRequest, Student};
use crate::app::server_functions::students::delete_student;
use leptos::ev::SubmitEvent;
use leptos::*;
use std::rc::Rc;

#[component]
pub fn DeleteStudentConfirmation(
    #[prop(into)] student: Signal<Option<Rc<Student>>>,
    #[prop(into)] show: Signal<bool>,
    #[prop(into)] set_show: Callback<bool>,
    #[prop(into)] on_delete_success: Callback<()>,
) -> impl IntoView {
    let (confirm_id, set_confirm_id) = create_signal(String::new());

    let handle_delete_student = move |ev: SubmitEvent| {
        ev.prevent_default();

        if let Some(student_to_be_deleted) = student() {
            let validated_confirm_id = confirm_id()
                .parse::<i32>()
                .expect("Delete confirmation ID was processed correctly");

            if validated_confirm_id == student_to_be_deleted.student_id {
                let delete_student_request = DeleteStudentRequest::new(
                    student_to_be_deleted.firstname.clone().unwrap(),
                    student_to_be_deleted.lastname.clone().unwrap(),
                    validated_confirm_id,
                );

                spawn_local(async move {
                    let delete_result = delete_student(delete_student_request).await;

                    match delete_result {
                        Ok(_deleted_student) => {
                            set_show.call(false);
                            on_delete_success.call(());
                        }
                        Err(e) => {
                            println!("Error deleting = {:?}", e);
                            set_show.call(false);
                        }
                    };
                });
            } else {
                set_show.call(false);
                log::info!("Delete was cancelled");
            }
        }
    };

    view! {
        <Show when=move || show() && student().is_some()>
            <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                <div class="bg-white p-6 rounded-lg shadow-xl max-w-md w-full">
                    <h3 class="text-xl font-bold mb-4">"Confirm Delete"</h3>
                    <p class="mb-4">
                        "To confirm deletion, please enter the student ID number: "
                        {move || student().map(|s| s.student_id.to_string())}
                    </p>
                    <form on:submit=handle_delete_student>
                        <input
                            type="text"
                            class="w-full p-2 border rounded mb-4"
                            placeholder="Enter student ID"
                            on:input=move |ev| set_confirm_id(event_target_value(&ev))
                            required
                        />
                        <div class="flex justify-end gap-2">
                            <button
                                type="button"
                                class="px-4 py-2 bg-gray-200 rounded hover:bg-gray-300"
                                on:click=move |_| set_show.call(false)
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
    }
}
