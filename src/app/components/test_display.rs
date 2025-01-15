use crate::app::components::{EditTestModal, ShowTestModal, ToastMessage};
use crate::app::models::Test;
use leptos::*;
use std::cell::RefCell;
use std::rc::Rc;

const DISPLAY_TEST_STYLE: &str = "group aspect-w-16 aspect-h-9 block w-full overflow-hidden rounded-lg bg-[#00356b] hover:ease-in-out";
const DISPLAY_TEST_EDIT_STYLE: &str = "group aspect-w-16 aspect-h-9 block w-full overflow-hidden rounded-lg bg-red-800 hover:bg-[#00A86B] hover:scale-110 hover:-translate-y-1";

const IMG_SRC: &str = "/assets/math.png";

const CAPTION_STYLE: &str = "mt-2 text-sm font-medium text-[#00356b]";
#[component]
pub fn MathTestDisplay(
    test: Rc<Test>,
    test_resource: Resource<(), Result<Vec<Test>, ServerFnError>>,
    set_if_show_toast: WriteSignal<bool>,
    set_toast_message: WriteSignal<ToastMessage>,
    editing_mode: ReadSignal<bool>,
) -> impl IntoView {
    let page_source: Vec<String> = Vec::new();
    let (if_show_info_modal, set_if_show_info_modal) = create_signal(false);

    let on_show_info = move |_| {
        if editing_mode() {
            set_if_show_info_modal(!if_show_info_modal());
        }
    };

    let styling = move || {
        if editing_mode() {
            DISPLAY_TEST_EDIT_STYLE
        } else {
            DISPLAY_TEST_STYLE
        }
    };

    let edit_test = test.clone();
    let test_info = test.clone();

    view! {
        //<Show when=move || {if_show_info_modal()}>
        //    <ShowTestModal
        //        test=test_info.clone()
        //        set_if_show_modal=set_if_show_info_modal
        //        set_if_show_deleted=set_if_show_toast
        //        test_resource
        //        set_toast_message
        //    />
        //</Show>
        <Show when=move || {if_show_info_modal()}>
            <EditTestModal
                test=test_info.clone()
                set_if_show_modal=set_if_show_info_modal
                set_if_show_deleted=set_if_show_toast
                set_if_show_toast=set_if_show_toast
                test_resource
                set_toast_message
            />
        </Show>
        <li class="z-auto">
            <button on:click=on_show_info>
                <div class=styling>
                    <img src=IMG_SRC />
                </div>
                <p class=CAPTION_STYLE>{&test.name}</p>
            </button>
        </li>
    }
}
