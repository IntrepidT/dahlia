use leptos::prelude::*;
use log::*;
use crate::app::components::{Toast, ToastMessage, ToastMessageType};
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::hooks::*;
use std::rc::Rc;

const MODAL_STYLE: &str = 
    "flex flex-col bg-[#F9F9F8] px-6 py-5 w-96 rounded-lg shadow-lg border border-[#DADADA]";

const CARD_STYLE: &str = 
    "flex flex-col items-center justify-center p-4 rounded-lg hover:bg-[#DADADA] border border-[#DADADA] transition-all duration-200 hover:shadow-md cursor-pointer";

const ICON_STYLE: &str = 
    "h-10 w-10 mb-3 p-2 rounded-full bg-[#DADADA] flex items-center justify-center";

const BUTTON_TEXT_STYLE: &str = 
    "text-[#2E3A59] font-medium text-sm mt-1";

#[component]
pub fn ShowAdministerTestModal(set_if_show_modal: WriteSignal<bool>) -> impl IntoView {
    let close_modal = move |_| set_if_show_modal.update(|value| *value = false);

    // Prevent clicks inside the modal from closing the overlay
    let prevent_propagation = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
    };

    view! {
        <div 
            class=format!("{} animate-slide-in-right", MODAL_STYLE)
            on:click=prevent_propagation
        >
            <div class="mb-4">
                <div class="flex justify-between items-center mb-2">
                    <h2 class="text-xl font-semibold text-[#2E3A59]">Select Assessment Type</h2>
                    /*<button 
                        class="text-gray-400 hover:text-gray-600 transition-colors"
                        on:click=close_modal
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                            <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                        </svg>
                    </button>*/
                </div>
                <p class="text-sm text-gray-500">"Choose which assessment you'd like to administer"</p>
            </div>
            
            <div class="grid grid-cols-2 gap-4">
                <A href="/mathtesting" attr:class=CARD_STYLE on:click=close_modal>
                    <div class=ICON_STYLE>
                        <img src="/assets/calculator.png" class="h-6 w-6" />
                    </div>
                    <span class=BUTTON_TEXT_STYLE>Math</span>
                </A>
                
                <A href="/readingtesting" attr:class=CARD_STYLE on:click=close_modal>
                    <div class=ICON_STYLE>
                        <img src="/assets/reading.png" class="h-6 w-6" />
                    </div>
                    <span class=BUTTON_TEXT_STYLE>Reading</span>
                </A>

            </div>
        </div>
    }
}
