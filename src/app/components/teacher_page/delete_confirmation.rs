use crate::app::models::employee::Employee;
use crate::app::models::teacher::DeleteTeacherRequest;
use crate::app::server_functions::employees::delete_employee;
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use log::{error, info, warn};
use std::sync::Arc;

#[component]
pub fn DeleteConfirmation(
    #[prop(into)] selected_employee: Signal<Option<Arc<Employee>>>,
    #[prop(into)] on_cancel: Callback<()>,
    #[prop(into)] on_delete: Callback<()>,
) -> impl IntoView {
    log::info!("DeleteConfirmation component initialized");

    // Input field state
    let (confirm_id, set_confirm_id) = signal(String::new());

    let handle_delete_employee = move |ev: SubmitEvent| {
        ev.prevent_default();
        log::info!("Delete form submitted");

        if let Some(employee_to_be_deleted) = selected_employee() {
            let validated_confirm_id = confirm_id()
                .parse::<i32>()
                .expect("Delete confirmation ID was processed correctly");

            if validated_confirm_id == employee_to_be_deleted.id {
                let delete_teacher_request = DeleteTeacherRequest::new(validated_confirm_id);

                spawn_local(async move {
                    let delete_result = delete_employee(delete_teacher_request).await;

                    match delete_result {
                        Ok(_deleted_employee) => on_delete.run(()),
                        Err(e) => {
                            println!("Error deleting = {:?}", e);
                            on_cancel.run(());
                        }
                    };
                });
            } else {
                on_cancel.run(());
                log::info!("Delete was cancelled");
            }
        }
    };

    view! {
        <Show
            when=move || {
                let is_some = selected_employee().is_some();
                log::info!("DeleteConfirmation <Show> condition: {}", is_some);
                is_some
            }
            fallback=move || {
                log::warn!("DeleteConfirmation rendering fallback (no employee)");
                view! { <div></div> }
            }
        >
            <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                <div class="bg-white p-6 rounded-lg shadow-xl max-w-md w-full">
                    <h3 class="text-xl font-bold mb-4">"Confirm Delete"</h3>

                    {move || {
                        log::info!("Rendering employee name section");
                        selected_employee().map(|emp| {
                            view! {
                                <p class="mb-4">
                                    "To confirm deletion, please enter the employee's full name: "
                                    <span class="font-semibold">
                                        {emp.id}
                                    </span>
                                </p>
                            }
                        })
                    }}

                    <form on:submit=handle_delete_employee>
                        <input
                            type="text"
                            class="w-full p-2 border rounded mb-4"
                            placeholder="Enter full name"
                            on:input=move |ev| set_confirm_id(event_target_value(&ev))
                            required
                        />
                        <div class="flex justify-end gap-2">
                            <button
                                type="button"
                                class="px-4 py-2 bg-gray-200 rounded hover:bg-gray-300"
                                on:click=move |_| on_cancel.run(())
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
