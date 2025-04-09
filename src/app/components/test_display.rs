use crate::app::components::{EditTestModal, ShowTestModal, ToastMessage};
use crate::app::models::Test;
use leptos::*;
use leptos_router::*;
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
    on_delete: Option<Callback<String>>,
    show_delete_mode: ReadSignal<bool>,
) -> impl IntoView {
    let edit_test = test.clone();
    let test_info = test.clone();

    let _page_source: Vec<String> = Vec::new();
    let (if_show_info_modal, set_if_show_info_modal) = create_signal(false);

    let on_show_info = move |_| {
        if editing_mode() {
            let navigate = leptos_router::use_navigate();
            navigate(
                &format!("/testbuilder/{}", edit_test.test_id),
                Default::default(),
            );
        } else {
            let navigate = leptos_router::use_navigate();
            navigate(
                &format!("/flashcardset/{}", edit_test.test_id),
                Default::default(),
            );
        }
    };

    let styling = move || {
        if editing_mode() {
            DISPLAY_TEST_EDIT_STYLE
        } else {
            DISPLAY_TEST_STYLE
        }
    };

    view! {
        <div class="z-auto relative">
            <button on:click=on_show_info>
                <div class=styling>
                    <img src=IMG_SRC />
                </div>
                <p class=CAPTION_STYLE>{&test.name}</p>
            </button>

            {move || {
                if show_delete_mode() && on_delete.is_some() {
                    let test_id = test.test_id.clone(); // Clone directly from the Rc<Test>
                    view! {
                        <div class="absolute top-1 right-1 z-10 mt-2">
                            <button
                                class="bg-red-800 text-white p-2 rounded hover:bg-red-900"
                                on:click=move |_| {
                                    if let Some(delete_fn) = on_delete.clone() {
                                        delete_fn(test_id.clone());
                                    }
                                }
                            >
                                "Delete"
                            </button>
                        </div>
                    }
                } else {
                    view! { <div></div> }
                }
            }}
        </div>
    }
}
