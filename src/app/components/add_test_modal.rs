use crate::app::components::{Toast, ToastMessage, ToastMessageType};
use crate::app::models::test::{test_type, CreateNewTestRequest};
use crate::app::server_functions::tests::add_test;
use leptos::*;
use leptos_router::*;
use std::str::FromStr;
use validator::Validate;

#[component]
pub fn AddTestModal(
    set_if_show_modal: WriteSignal<bool>,
    set_if_show_added: WriteSignal<bool>,
    set_toast_message: WriteSignal<ToastMessage>,
) -> impl IntoView {
    const INPUT_STYLE: &str = "w-full h-12 bg-[#333333] pr-4 pl-6 py-4 text-white mt-6 outline-none focus:outline-none focus:pl-7 transition-all duration-1000 ease-in-out rounded";

    const CANCEL_BUTTON_STYLE: &str = "mt-10 bg-[#555555] px-8 py-2 rounded text-white mr-3 transition-all duration-1000 ease-in-out hover:bg-[#666666]";

    const ADD_BUTTON_STYLE: &str = "mt-10 bg-[#00356B] px-8 py-2 rounded text-white transition-all duration-1000 ease-in-out hover:bg-[#FFB6C1]";

    const NO_ERROR_STYLE: &str = "flex flex-col bg-[#222222] border-t-8 border-[#00356B] px-6 pt-5 h-[34rem] w-full max-w-[36rem] z-50 -mt-2 fixed z-50 rounded";

    const ERROR_STYLE: &str = "flex flex-col bg-[#222222] border-t-8 border-[#00356B] px-6 pt-5 h-[34rem] w-full max-w-[36rem] z-50 -mt-2 fixed z-50 rounded";
    //create and send signals for various data
    let (test_name, set_test_name) = create_signal(String::new());
    let (test_score, set_test_score) = create_signal(String::new());
    let (test_comments, set_test_comments) = create_signal(String::new());
    let (test_area, set_test_area) = create_signal(String::new());
    let (test_identifier, set_test_identifier) = create_signal(String::new());

    //
    //create and send signals for error messages
    let (error_message, set_error_message) = create_signal(String::new());
    let (if_error, set_if_error) = create_signal(false);
    //close the modal
    let on_close = move |_| {
        set_if_show_modal(false);
    };
    //add a new person to the modal
    let on_click = move |_| {
        let converting_to_test_type = test_type::from_str(&test_area()).clone().unwrap();
        let navigate = leptos_router::use_navigate();

        let add_test_request = CreateNewTestRequest::new(
            test_name(),
            test_score().parse::<i32>().expect("Numbers only"),
            test_comments(),
            converting_to_test_type,
            test_identifier().parse::<i64>().expect("Numbers only"),
        );

        let is_valid = add_test_request.validate();

        match is_valid {
            Ok(_) => {
                spawn_local(async move {
                    let add_result = add_test(add_test_request).await;

                    //we get the result back and do something with it
                    match add_result {
                        Ok(_added_test) => {
                            set_if_show_modal(false);

                            set_toast_message(ToastMessage::create(ToastMessageType::NewTestAdded));

                            //setting this to true will make the Toast
                            //"new member added" appear
                            set_if_show_added(true);
                            navigate("/testbuilder", Default::default());
                        }
                        Err(e) => println!("Error adding: {:?}", e),
                    };
                });
            }
            Err(_) => {
                set_if_error(true);
                set_error_message(String::from("All fields are required"))
            }
        }
    };

    view! {
        <div class="flex flex-col w-full h-full z-50 mx-auto items-center align-center rounded-2xl">
            <div class={move || {
                if if_error() {ERROR_STYLE}
                else {NO_ERROR_STYLE}
            }}>
                <Show when=move || {if_error() }>
                    <p class="text-white bg-red-500 rounded w-full h-12 px-5 py-3 transition-all duration-750 ease-in-out">
                    {error_message()}
                    </p>
                </Show>
                <p class="text-white pt-2 font-bold text-2xl">"Add New Test"</p>
                <input type="text" placeholder="Name of Test"
                    class=INPUT_STYLE
                    value=test_name
                    on:input=move |event| {
                        set_test_name(event_target_value(&event));
                    }
                />
                <input type="text" placeholder="Score"
                    class=INPUT_STYLE
                    value=test_score
                    on:input=move |event| {
                        set_test_score(event_target_value(&event));
                    }
                />
                <input type="text" placeholder="Comments"
                    class=INPUT_STYLE
                    value=test_comments
                    on:input=move |event| {
                        set_test_comments(event_target_value(&event));
                    }
                />
                <select class=INPUT_STYLE
                    value=test_area
                    on:change=move |event| {
                        set_test_area(event_target_value(&event));
                    }
                >
                    <option value="">"Please Select a Value"</option>
                    <option value="Reading">"Reading"</option>
                    <option value="Math">"Math"</option>
                </select>
               <input type="text" placeholder="Test ID"
                    class=INPUT_STYLE
                    value=test_identifier
                    on:input=move |event| {
                        set_test_identifier(event_target_value(&event));
                    }
               />
               <div class="flex flex-row w-full items-right justify-right">
                    <button on:click=on_close class=CANCEL_BUTTON_STYLE>
                        "Cancel"
                    </button>
                    <button on:click=on_click class=ADD_BUTTON_STYLE>
                        "Add"
                    </button>
               </div>
            </div>
        </div>
    }
}
