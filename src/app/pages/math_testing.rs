use crate::app::components::{AddTestModal, Header, MathTestDisplay, Toast, ToastMessage};
use crate::app::server_functions::get_tests;
use leptos::*;
use leptos_router::*;
use std::rc::Rc;

#[component]
pub fn MathTesting() -> impl IntoView {
    const ADD_BUTTON_STYLE: &str = "bg-[#00356b] px-8 py-2 rounded text-white transition-all duration-1000 ease-in-out hover:bg-black";
    const EDIT_BUTTON_STYLE: &str = "bg-[#00A86B] px-8 py-2 rounded text-white transition-all duration-1000 ease-in-out hover:bg-black ml-2";
    const EDIT_BUTTON_CLICKED_STYLE: &str =
        "bg-red-800 px-8 py-2 rounded text-white transition-all duration-1000 ease-in-out ml-2";

    let (if_show_modal, set_if_show_modal) = create_signal(false);
    let (if_show_edit, set_if_show_edit) = create_signal(false);

    let (if_show_toast, set_if_show_toast) = create_signal(false);
    let (toast_message, set_toast_message) = create_signal(ToastMessage::new());

    let get_tests_info = create_resource(|| (), |_| async move { get_tests().await });
    let on_click_add = move |_| {
        set_if_show_modal(!if_show_modal());
    };
    let on_click_edit = move |_| {
        set_if_show_edit(!if_show_edit());
    };

    view! {
        <div class="relative">
            <Header />
            <div class="mx-auto max-w-7xl mt-20">
                <Toast
                    toast_message
                    if_appear=if_show_toast
                    set_if_appear = set_if_show_toast
                />

                <Show when=move || {if_show_modal()}>
                    <AddTestModal
                        set_if_show_modal
                        set_if_show_added=set_if_show_toast
                        set_toast_message
                    />
                </Show>
                <div class="flex flex-row w-full">
                    <h1 class="text-2xl font-bold leading-7 text-[#00356b] mt-4">
                        Math Tests
                    </h1>
                    <hr class="w-full max-w-[58rem] inline justify-center items-center ml-3 pl-4 pr-4 pt-4 mt-8 mr-4 text-[#00356b]" />
                    <button on:click=on_click_add class=ADD_BUTTON_STYLE>
                        "Add"
                    </button>
                    <button on:click=on_click_edit class=EDIT_BUTTON_STYLE>
                        "Edit"
                    </button>
                </div>
            //this will define the list view for each of the math tests
                <Suspense fallback=move || {
                    view!{<p>"Loading..."</p>}
                }>
                    <ul role="list" class="grid grid-cols-4 gap-x-4 gap-y-8 mt-8 static">
                        {
                            move || {
                                get_tests_info.get().map(|data| {

                                    match data {
                                        Ok(tests_data) => {
                                            tests_data.iter().map(|each_test| view!{
                                                <MathTestDisplay
                                                    test=Rc::new(each_test.clone())
                                                    test_resource=get_tests_info
                                                    set_if_show_toast
                                                    set_toast_message
                                                    editing_mode=if_show_edit
                                                />
                                            }).collect_view()
                                        },
                                        Err(_) =>
                                            view! {<div>"An Error has occured"</div>}.into_view()
                                    }
                                })
                            }
                        }
                    </ul>
                </Suspense>
            </div>
        </div>
    }
}
