use crate::app::models::employee::Employee;
use crate::app::models::teacher::DeleteTeacherRequest;
use crate::app::server_functions::employees::delete_employee;
use leptos::*;
use std::rc::Rc;

#[component]
pub fn DeleteConfirmation(
    #[prop(into)] selected_employee: Signal<Option<Rc<Employee>>>,
    #[prop(into)] on_cancel: Callback<()>,
    #[prop(into)] on_delete: Callback<()>,
) -> impl IntoView {
    // Input field state
    let (confirm_delete_text, set_confirm_delete_text) = create_signal(String::new());

    let handle_delete_student = move |ev| {
        let employee_to_be_deleted = selected_employee().unwrap();
        let binding = confirm_delete_text().clone();
        let mut validated_delete_two = binding.split_whitespace();
        let firstname: String = validated_delete_two.next().unwrap_or_default().to_string();

        let lastname: String = validated_delete_two.next().unwrap_or_default().to_string();

        if firstname.to_lowercase() == employee_to_be_deleted.firstname.to_lowercase()
            && lastname.to_lowercase() == employee_to_be_deleted.lastname.to_lowercase()
        {
            let delete_employee_request = DeleteTeacherRequest::new(firstname, lastname);

            spawn_local(async move {
                let delete_result = delete_employee(delete_employee_request).await;

                match delete_result {
                    Ok(_deleted_student) => {
                        on_delete(());
                    }
                    Err(e) => {
                        println!("Error deleting = {:?}", e);
                        on_cancel(());
                    }
                };
            });
        } else {
            log::info!("Delete was cancelled");
        }
    };

    view! {
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div class="bg-white p-6 rounded-lg shadow-xl max-w-md w-full">
                <h3 class="text-xl font-bold mb-4">"Confirm Delete"</h3>
                <p class="mb-4">
                    "To confirm deletion, please enter the employee's full name: "
                    {selected_employee().unwrap().firstname.clone()}
                    {" "}
                    {selected_employee().unwrap().lastname.clone()}
                </p>
                <input
                    type="text"
                    class="w-full p-2 border rounded mb-4"
                    placeholder="Enter full name"
                    prop:value=confirm_delete_text
                    on:input=move |ev| set_confirm_delete_text(event_target_value(&ev))
                    required
                />
                <div class="flex justify-end gap-2">
                    <button
                        type="button"
                        class="px-4 py-2 bg-gray-200 rounded hover:bg-gray-300"
                        on:click=move |_| on_cancel(())
                    >
                        "Cancel"
                    </button>
                    <button
                        type="submit"
                        class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600"
                        on:click=move |ev| handle_delete_student(ev)
                    >
                        "Delete"
                    </button>
                </div>
            </div>
        </div>
    }
}
