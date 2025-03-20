
use crate::app::components::{Toast, ToastMessage, ToastMessageType};
use leptos::*;
use leptos_router::*;
use std::rc::Rc;

const MODAL_STYLE: &str = 
    "flex flex-col bg-white px-6 py-5 w-96 rounded-lg shadow-lg border border-gray-100";

const CARD_STYLE: &str = 
    "flex flex-col items-center justify-center p-4 rounded-lg hover:bg-gray-50 border border-gray-200 transition-all duration-200 hover:shadow-md";

const ICON_STYLE: &str = 
    "h-10 w-10 mb-3 p-2 rounded-full bg-gray-100";

const BUTTON_TEXT_STYLE: &str = 
    "text-gray-800 font-medium text-sm mt-1";

#[component]
pub fn ShowAdministerTestModal(set_if_show_modal: WriteSignal<bool>) -> impl IntoView {
    view! {
       <div class=MODAL_STYLE>
           <div class="mb-4">
               <h2 class="text-xl font-semibold text-gray-800">Select Assessment Type</h2>
               <p class="text-sm text-gray-500">"Choose which assessment you'd like to administer"</p>
           </div>
           
           <div class="grid grid-cols-3 gap-4">
               <A href="/mathtesting" class=CARD_STYLE>
                   <div class=ICON_STYLE>
                       <img src="/assets/calculator.png" class="h-6 w-6"/>
                   </div>
                   <span class=BUTTON_TEXT_STYLE>Math</span>
               </A>
               
               <A href="/readingtesting" class=CARD_STYLE>
                   <div class=ICON_STYLE>
                       <img src="/assets/reading.png" class="h-6 w-6" />
                   </div>
                   <span class=BUTTON_TEXT_STYLE>Reading</span>
               </A>
               
               <A href="https://dibels.amplify.com" class=CARD_STYLE>
                   <div class=ICON_STYLE>
                       <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                           <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                       </svg>
                   </div>
                   <span class=BUTTON_TEXT_STYLE>Dibels</span>
               </A>
           </div>
           
           <div class="mt-4 flex justify-end">
               <button 
                   class="text-sm text-gray-500 hover:text-gray-700"
                   on:click=move |_| set_if_show_modal.update(|value| *value = false)
               >
                   Cancel
               </button>
           </div>
       </div>
    }
}

