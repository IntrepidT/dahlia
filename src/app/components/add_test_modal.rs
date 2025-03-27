use crate::app::components::{Toast, ToastMessage, ToastMessageType};
use crate::app::models::test::{CreateNewTestRequest, TestType};
use crate::app::server_functions::tests::add_test;
use leptos::*;
use leptos_router::*;
use std::str::FromStr;
use validator::Validate;

//these are purely for testing purposes
use crate::app::models::question::{CreateNewQuestionRequest, QuestionType};
use crate::app::server_functions::questions::add_question;

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
    let (test_score, set_test_score) = create_signal("0".to_string());
    let (test_comments, set_test_comments) = create_signal(String::new());
    let (test_area, set_test_area) = create_signal(String::new());
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
        let converting_to_test_type = match TestType::from_str(&test_area()) {
            Ok(test_type) => test_type,
            Err(_) => {
                set_if_error(true);
                set_error_message(String::from("Invalid test area selected"));
                return;
            }
        };
        let navigate = leptos_router::use_navigate();

        let score = match test_score().parse::<i32>() {
            Ok(num) => num,
            Err(_) => {
                set_if_error(true);
                set_error_message(String::from("Score must be a valid number"));
                return;
            }
        };

        let add_test_request =
            CreateNewTestRequest::new(test_name(), score, test_comments(), converting_to_test_type);

        match add_test_request.validate() {
            Ok(_) => {
                spawn_local(async move {
                    let add_result = add_test(add_test_request).await;

                    //we get the result back and do something with it
                    match add_result {
                        Ok(added_test) => {
                            set_if_show_modal(false);

                            set_toast_message(ToastMessage::create(ToastMessageType::NewTestAdded));

                            //setting this to true will make the Toast
                            //"new member added" appear
                            set_if_show_added(true);
                            let test_question = CreateNewQuestionRequest::new(
                                "What Letter is this: A".to_string(),
                                1,
                                QuestionType::TrueFalse,
                                vec!["true".to_string(), "false".to_string()],
                                "true".to_string(),
                                1,
                                added_test.test_id.clone(),
                            );
                            match add_question(added_test.test_id.clone(), test_question).await {
                                Ok(_) => {}
                                Err(e) => {}
                            }
                            navigate(
                                &format!("/testbuilder/{}", added_test.test_id),
                                Default::default(),
                            );
                        }
                        Err(e) => {
                            set_if_error(true);
                            set_error_message(format!("Failed to add test: {}", e));
                            log::error!("Error adding test: {:?}", e);
                        }
                    };
                });
            }
            Err(e) => {
                set_if_error(true);
                set_error_message(format!("Validation error: {}", e));
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
                //OVERRIDE OCCURS ON:SUBMIT BUTTON TEST_BUILDER
                /*<input type="text" placeholder="Score"
                    class=INPUT_STYLE
                    value=test_score
                    on:input=move |event| {
                        set_test_score(event_target_value(&event));
                    }
                />*/
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
