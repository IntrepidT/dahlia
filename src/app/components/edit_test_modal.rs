use crate::app::components::{Toast, ToastMessage, ToastMessageType};
use crate::app::models::{DeleteTestRequest, Test, TestType, UpdateTestRequest};
use crate::app::server_functions::tests::{delete_test, update_test};
use leptos::*;
use leptos_router::*;
use std::rc::Rc;
use std::str::FromStr;
use validator::Validate;

const INFO_STYLE: &str = "h-20 pr-4 py-4 mt-4 text-white outline-none focus:outline-none focus:pl-7 transition-all duration-1000 ease-in-out";

const INFO_TITLE_STYLE: &str = "text-stone-400 text-xs";

const INPUT_STYLE: &str = "w-full h-12 bg-[#333333] pr-4 pl-6 py-4 text-white mt-4 outline-none focus:outline-non focus:pl-7 transition-all duration-1000 ease-in-out";

const INPUT_STYLE_TITLE: &str = "text-2xl w-full h-12 bg-[#333333] pr-4 pl-6 py-4 text-white mt-4 outline-none focus:outline-non focus:pl-7 transition-all duration-1000 ease-in-out";

const UPDATE_TEST_QUESTIONS: &str = "mt-5 mb-4 bg-[#FFBF00] px-8 py-2 rounded text-white transition-all duration-1000 ease-in-out hover:bg-[#F8DE7E]";

const CANCEL_BUTTON_STYLE: &str = "mt-5 mb-4 bg-[#555555] px-8 py-2 rounded text-white transition-all duration-1000 ease-in-out hover:bg-[#666666]";

const UPDATE_BUTTON_STYLE: &str = "mt-5 mb-4 bg-[#00356b] px-8 py-2 rounded text-white transition-all duration-1000 ease-in-out hover:bg-[#6495ED]";

const DELETE_BUTTON_STYLE: &str = "mt-5 mb-4 bg-red-800 px-8 py-2 rounded text-white transition-all duration-1000 ease-in-out hover:bg-[#FFB6C1] ml-2";

const NO_ERROR_STYLE:&str = "flex flex-col bg-[#222222] border-t-8 border-[#00356b] rounded px-6 pt-5 h-[42rem] w-full max-w-[40rem] z-50 fixed -mt-2 top-20 z-50";

const ERROR_STYLE: &str = "flex flex-col bg-[#222222] border-t-8 border-[#00356b] px-6 pt-5 h-[42rem] rounded w-full max-w-[36rem] z-50 fixed -mt-2 top-20 z-50";

#[component]
pub fn EditTestModal(
    test: Rc<Test>,
    set_if_show_modal: WriteSignal<bool>,
    set_if_show_deleted: WriteSignal<bool>,
    set_if_show_toast: WriteSignal<bool>,
    test_resource: Resource<(), Result<Vec<Test>, ServerFnError>>,
    set_toast_message: WriteSignal<ToastMessage>,
) -> impl IntoView {
    let _mimic_test = test.clone();

    let (test_name, set_test_name) = create_signal(test.name.clone());
    let (test_score, set_test_score) = create_signal(format!("{}", test.score.clone()));
    let (test_comments, set_test_comments) = create_signal(test.comments.clone());
    let (test_area, set_test_area) = create_signal(test.testarea.to_string());
    let (test_identifier, set_test_identifier) = create_signal(test.test_id.to_string());
    // for errors
    let (error_message, set_error_message) = create_signal(String::new());
    let (if_error, set_if_error) = create_signal(false);
    //to close the modal if needed
    let on_close = move |_| {
        set_if_show_modal(false);
    };

    //to perform deletion
    let on_click_delete = move |_| {
        let delete_test_request = DeleteTestRequest::new(test.test_id.clone());

        spawn_local(async move {
            let delete_result = delete_test(delete_test_request).await;

            match delete_result {
                Ok(deleted_test) => {
                    test_resource.refetch();

                    set_toast_message(ToastMessage::create(ToastMessageType::TestDeleted));

                    set_if_show_deleted(true);

                    set_if_show_modal(false);
                }
                Err(e) => println!("Error deleting = {:?}", e),
            };
        });
    };
    //to perform an edit/modification
    let on_click_update = move |_| {
        //let test_id = test.test_id.clone();
        let test_score_validated = match test_score().parse::<i32>() {
            Ok(score) => score,
            Err(_) => {
                set_if_error(true);
                set_error_message(String::from("Score must be a valid number"));
                return;
            }
        };
        let convert_test_area_to_enum = match TestType::from_str(&test_area()) {
            Ok(test_type) => test_type,
            Err(_) => {
                set_if_error(true);
                set_error_message(String::from("Invalid test area selected"));
                return;
            }
        };

        let edit_test_request = UpdateTestRequest::new(
            test_name(),
            test_score_validated,
            test_comments(),
            convert_test_area_to_enum,
            test_identifier(),
        );

        let is_valid = edit_test_request.validate();

        match is_valid {
            Ok(_) => {
                spawn_local(async move {
                    let edit_result = update_test(edit_test_request).await;

                    match edit_result {
                        Ok(edited_test) => {
                            test_resource.refetch();

                            set_if_show_modal(false);

                            set_toast_message(ToastMessage::create(ToastMessageType::TestUpdated));

                            set_if_show_toast(true);
                        }
                        Err(e) => {
                            set_if_error(true);
                            set_error_message(format!(
                                "Error Updating Test. Please try again later: {}",
                                e
                            ))
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
        <div class="flex flex-col w-full h-full max-w-7xl z-50 mx-auto items-center align-center rounded-2xl absolute">

            <div class={move || {
                if if_error() {ERROR_STYLE}
                else {NO_ERROR_STYLE}
            }}>
                <Show when=move || {if_error()}>
                    <p class="text-white bg-red-500 rounded w-full h-12 px-5 py-3 transition-all duration-750 ease-in-out">
                        {error_message()}
                    </p>
                </Show>

                <p class="text-white font-bold text-3xl mt-2">Modify Existing Test</p>


                <div class=INFO_STYLE>
                    <div class=INFO_TITLE_STYLE>"Name"</div>
                    <input type="text" placeholder="Name" class=INPUT_STYLE
                        value=test_name
                        on:input=move |event| {
                            set_test_name(event_target_value(&event));
                        }
                    />
                </div>

                <div class=INFO_STYLE>
                    <div class=INFO_TITLE_STYLE>"Maximum Score"</div>
                    <input type="text" placeholder="Maximum Score" class=INPUT_STYLE
                        value=test_score
                        on:input=move |event| {
                            set_test_score(event_target_value(&event));
                        }
                    />
                </div>

                <div class=INFO_STYLE>
                    <div class=INFO_TITLE_STYLE>"Comments"</div>
                    <input type="text" placeholder="Comments" class=INPUT_STYLE
                        value=test_comments
                        on:input=move |event| {
                            set_test_comments(event_target_value(&event));
                        }
                    />
                </div>
                <div class=INFO_STYLE>
                    <div class=INFO_TITLE_STYLE>"Test Subject"</div>
                    <select class=INPUT_STYLE
                        value=test_area
                        on:change=move |event| {
                            set_test_area(event_target_value(&event));
                        }
                    >
                        <option value="Reading">"Reading"</option>
                        <option value="Math">"Math"</option>
                        <option value="" disabled selected>"Please Select a Value"</option>
                    </select>
                </div>

                <div class=INFO_STYLE>
                    <div class=INFO_TITLE_STYLE>"Test Identifier"</div>
                    <input type="text" placeholder="Test Identifier" class=INPUT_STYLE
                        value=test_identifier
                        on:input=move |event| {
                            set_test_identifier(test_identifier());
                        }
                        readonly
                    />
                </div>

                <div class="flex flex-row w-full items-right justify-right mt-6 space-x-4">
                    <button on:click=on_close class=CANCEL_BUTTON_STYLE>
                        "Cancel"
                    </button>
                    <button on:click=on_click_update class=UPDATE_BUTTON_STYLE>
                        "Update"
                    </button>
                    <button on:click=on_click_delete class=DELETE_BUTTON_STYLE>
                        "Delete"
                    </button>
                    <A href="/testbuilder" class=UPDATE_TEST_QUESTIONS>
                        "Update Questions"
                    </A>
                </div>
            </div>
        </div>
    }
}
