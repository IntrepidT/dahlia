use crate::app::components::{Toast, ToastMessage, ToastMessageType};
use crate::app::models::{DeleteTestRequest, Test};
use crate::app::server_functions::tests::delete_test;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::rc::Rc;

const INFO_STYLE: &str = "w-full h-12 pr-4 py-4 mt-6 flex flex-col outline-none focus:outline-none focus:pl-7 transition-all duration-1000 ease-in-out";

const INFO_TITLE_STYLE: &str = "text-stone-400 text-xs";
const INFO_VALUE_STYLE: &str = "text-white";

const CLOSE_BUTTON_STYLE: &str = "mt-10 bg-[#555555] px-8 py-2 rounded text-white mr-3 transition-all duration-1000 ease-in-out hover:bg-[#666666]";

const DELETE_BUTTON_STYLE: &str = "mt-10 bg-red-800 px-8 py-2 rounded text-white transition-all duration-1000 ease-in-out hover:bg-red-600";

const MODAL_STYLE: &str = "flex flex-col bg-[#222222] border-t-8 border-[#00356b] px-6 pt-5 h-[32rem] w-full max-w-[36rem] rounded-2xl z-50 -mt-2 fixed top-20 z-50";

#[component]
pub fn ShowTestModal(
    test: Rc<Test>,
    set_if_show_modal: WriteSignal<bool>,
    set_if_show_deleted: WriteSignal<bool>,
    test_resource: Resource<Result<Vec<Test>, ServerFnError>>,
    set_toast_message: WriteSignal<ToastMessage>,
) -> impl IntoView {
    let mimic_test = test.clone();
    //to close the MODAL_STYLE
    let on_close = move |_| {
        set_if_show_modal(false);
    };

    //to perform deletion
    let on_click_delete = move |_| {
        let delete_test_request = DeleteTestRequest::new(mimic_test.test_id.clone());

        let _ = spawn_local(async move {
            let delete_result = delete_test(delete_test_request).await;

            match delete_result {
                Ok(_deleted_test) => {
                    test_resource.refetch();

                    set_toast_message(ToastMessage::create(ToastMessageType::TestDeleted));

                    set_if_show_deleted(true);

                    set_if_show_modal(false);
                }
                Err(e) => println!("Error deleting = {:?}", e),
            };
        });
    };

    view! {
        <div class="flex flex-col w-full h-full z-49 bg-[#222222/[.06]] rounded-2xl absolute">

            <div class="flex flex-col w-full h-full z-50 mx-auto items-center align-center">
                <div class=MODAL_STYLE>

                    <p class="text-white pt-2 text-4xl mt-2">
                        {test.name.clone()}
                    </p>

                    <div class=INFO_STYLE>
                        <div class=INFO_TITLE_STYLE>"Maximum Score"</div>
                        <div class=INFO_VALUE_STYLE>{format!("{:?}", &test.score)}</div>
                    </div>

                    <div class=INFO_STYLE>
                        <div class=INFO_TITLE_STYLE>"Comments"</div>
                        <div class=INFO_VALUE_STYLE>{test.comments.clone()}</div>
                    </div>

                    <div class=INFO_STYLE>
                        <div class=INFO_TITLE_STYLE>"Test Identifier"</div>
                        <div class=INFO_VALUE_STYLE>{format!("#{:?}", test.test_id.clone())}</div>
                    </div>

                    <div class=INFO_STYLE>
                        <div class=INFO_TITLE_STYLE>"Test Type"</div>
                        <div class=INFO_VALUE_STYLE>{format!("{:?}", test.testarea.clone())}</div>
                    </div>

                    <div class="flex flex-row w-full items-right justify-right mt-1">
                        <button on:click=on_close class=CLOSE_BUTTON_STYLE>
                            "Close"
                        </button>
                        <button on:click=on_click_delete class=DELETE_BUTTON_STYLE>
                            "Delete"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
