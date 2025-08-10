use crate::app::models::test::Test;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use std::rc::Rc;

#[component]
pub fn GenericTestModal(test_id: String, test_name: String, children: Children) -> impl IntoView {
    let (show_options_modal, set_show_options_modal) = signal(false);
    let name_clone = test_name.clone();

    // Handle wrapper click to show the modal
    let on_wrapper_click = move |_| {
        set_show_options_modal.set(true);
    };

    // Navigation functions for different test modes
    let test_id_clone = test_id.clone();
    let on_realtime_click = move |_| {
        let navigate = use_navigate();
        navigate(
            &format!("/test-session/{}", test_id_clone),
            Default::default(),
        );
        set_show_options_modal.set(false);
    };

    let test_id_clone = test_id.clone();
    let on_individual_click = move |_| {
        let navigate = use_navigate();
        navigate(
            &format!("/flashcardset/{}", test_id_clone),
            Default::default(),
        );
        set_show_options_modal.set(false);
    };

    let test_id_clone = test_id.clone();
    let on_grid_test_click = move |_| {
        let navigate = use_navigate();
        navigate(&format!("/gridtest/{}", test_id_clone), Default::default());
        set_show_options_modal.set(false);
    };

    let on_cancel_click = move |_| {
        set_show_options_modal.set(false);
    };

    view! {
        <div class="relative cursor-pointer" on:click=on_wrapper_click>
            {children()}

            // Test Options Modal
            {move || {
                if show_options_modal() {
                    view! {
                        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
                             on:click=move |e| {
                                 // Stop propagation to prevent modal closing when clicking inside it
                                 e.stop_propagation();
                             }
                        >
                            <div class="bg-white rounded-lg shadow-xl p-6 max-w-md w-full mx-4">
                                <h3 class="text-xl font-semibold text-gray-800 mb-4">"Test Options: " {name_clone.clone()}</h3>
                                <div class="space-y-4">
                                    <button
                                        class="w-full p-3 bg-blue-600 text-white rounded-lg flex items-center justify-between hover:bg-blue-700 transition-colors"
                                        on:click=on_realtime_click.clone()
                                    >
                                        <span class="text-lg">"Real-time Live Testing"</span>
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-12a1 1 0 10-2 0v4a1 1 0 00.293.707l2.828 2.829a1 1 0 101.415-1.415L11 9.586V6z" clip-rule="evenodd" />
                                        </svg>
                                    </button>

                                    <button
                                        class="w-full p-3 bg-green-600 text-white rounded-lg flex items-center justify-between hover:bg-green-700 transition-colors"
                                        on:click=on_individual_click.clone()
                                    >
                                        <span class="text-lg">"Individual Flashcard Test"</span>
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                            <path d="M9 2a1 1 0 000 2h2a1 1 0 100-2H9z" />
                                            <path fill-rule="evenodd" d="M4 5a2 2 0 012-2 3 3 0 003 3h2a3 3 0 003-3 2 2 0 012 2v11a2 2 0 01-2 2H6a2 2 0 01-2-2V5zm3 4a1 1 0 000 2h.01a1 1 0 100-2H7zm3 0a1 1 0 000 2h3a1 1 0 100-2h-3zm-3 4a1 1 0 100 2h.01a1 1 0 100-2H7zm3 0a1 1 0 100 2h3a1 1 0 100-2h-3z" clip-rule="evenodd" />
                                        </svg>
                                    </button>

                                    <button
                                        class="w-full p-3 bg-purple-600 text-white rounded-lg flex items-center justify-between hover:bg-purple-700 transition-colors"
                                        on:click=on_grid_test_click.clone()
                                    >
                                        <span class="text-lg">"Grid Test"</span>
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                            <path d="M5 3a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2V5a2 2 0 00-2-2H5zM5 11a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2v-2a2 2 0 00-2-2H5zM11 5a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V5zM11 13a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
                                        </svg>
                                    </button>
                                </div>

                                <div class="mt-6 flex justify-end">
                                    <button
                                        class="px-4 py-2 bg-gray-200 rounded-md hover:bg-gray-300 transition-colors"
                                        on:click=on_cancel_click
                                    >
                                        "Cancel"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>
    }
}

#[component]
pub fn TestItem(test: Test, test_id: String, test_name: String) -> impl IntoView {
    let (show_options_modal, set_show_options_modal) = signal(false);
    let name_clone = test_name.clone();

    // Handle test click to show the modal
    let on_test_click = move |_| {
        set_show_options_modal.set(true);
    };

    // Navigation functions for different test modes
    let test_id_clone = test_id.clone();
    let on_realtime_click = move |_| {
        let navigate = use_navigate();
        navigate(
            &format!("/test-session/{}", test_id_clone),
            Default::default(),
        );
        set_show_options_modal.set(false);
    };

    let test_id_clone = test_id.clone();
    let on_individual_click = move |_| {
        let navigate = use_navigate();
        navigate(
            &format!("/flashcardset/{}", test_id_clone),
            Default::default(),
        );
        set_show_options_modal.set(false);
    };

    let test_id_clone = test_id.clone();
    let on_grid_test_click = move |_| {
        let navigate = use_navigate();
        navigate(&format!("/gridtest/{}", test_id_clone), Default::default());
        set_show_options_modal.set(false);
    };

    let on_cancel_click = move |_| {
        set_show_options_modal.set(false);
    };

    view! {
        <div class="relative">
            <button
                class="flex items-center space-x-2 p-2 rounded bg-gray-50 hover:bg-gray-100 w-full text-left transition-colors"
                on:click=on_test_click
            >
                <span class="text-sm font-medium mr-2">{test_name}</span>
                <span class="text-xs text-gray-500 ml-auto">"Score: " {test.score}</span>
            </button>

            // Test Options Modal
            {move || {
                if show_options_modal() {
                    view! {
                        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                            <div class="bg-white rounded-lg shadow-xl p-6 max-w-md w-full mx-4">
                                <h3 class="text-xl font-semibold text-gray-800 mb-4">"Test Options: " {name_clone.clone()}</h3>
                                <div class="space-y-4">
                                    <button
                                        class="w-full p-3 bg-blue-600 text-white rounded-lg flex items-center justify-between hover:bg-blue-700 transition-colors"
                                        on:click=on_realtime_click.clone()
                                    >
                                        <span class="text-lg">"Real-time Live Testing"</span>
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-12a1 1 0 10-2 0v4a1 1 0 00.293.707l2.828 2.829a1 1 0 101.415-1.415L11 9.586V6z" clip-rule="evenodd" />
                                        </svg>
                                    </button>

                                    <button
                                        class="w-full p-3 bg-green-600 text-white rounded-lg flex items-center justify-between hover:bg-green-700 transition-colors"
                                        on:click=on_individual_click.clone()
                                    >
                                        <span class="text-lg">"Individual Flashcard Test"</span>
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                            <path d="M9 2a1 1 0 000 2h2a1 1 0 100-2H9z" />
                                            <path fill-rule="evenodd" d="M4 5a2 2 0 012-2 3 3 0 003 3h2a3 3 0 003-3 2 2 0 012 2v11a2 2 0 01-2 2H6a2 2 0 01-2-2V5zm3 4a1 1 0 000 2h.01a1 1 0 100-2H7zm3 0a1 1 0 000 2h3a1 1 0 100-2h-3zm-3 4a1 1 0 100 2h.01a1 1 0 100-2H7zm3 0a1 1 0 100 2h3a1 1 0 100-2h-3z" clip-rule="evenodd" />
                                        </svg>
                                    </button>

                                    <button
                                        class="w-full p-3 bg-purple-600 text-white rounded-lg flex items-center justify-between hover:bg-purple-700 transition-colors"
                                        on:click=on_grid_test_click.clone()
                                    >
                                        <span class="text-lg">"Grid Test"</span>
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                            <path d="M5 3a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2V5a2 2 0 00-2-2H5zM5 11a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2v-2a2 2 0 00-2-2H5zM11 5a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V5zM11 13a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
                                        </svg>
                                    </button>
                                </div>

                                <div class="mt-6 flex justify-end">
                                    <button
                                        class="px-4 py-2 bg-gray-200 rounded-md hover:bg-gray-300 transition-colors"
                                        on:click=on_cancel_click
                                    >
                                        "Cancel"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>
    }
}
