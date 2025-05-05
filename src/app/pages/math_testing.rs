use crate::app::components::dashboard::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::{Header, MathTestDisplay, Toast, ToastMessage, ToastMessageType};
use crate::app::models::{DeleteTestRequest, Test, TestType};
use crate::app::server_functions::{get_tests, tests::delete_test};
use leptos::callback::*;
use leptos::*;
use std::rc::Rc;

#[component]
pub fn MathTesting() -> impl IntoView {
    let (selected_view, set_selected_view) = create_signal(SidebarSelected::AdministerTest);
    // Button styles updated to match Stripe's design language
    const ADD_BUTTON_STYLE: &str = "bg-[#2E3A59] px-3 py-2 sm:px-4 rounded-md text-white font-medium text-xs sm:text-sm shadow-sm transition-all focus:outline-none focus:ring-2 focus:ring-[#2E3A59] focus:ring-offset-2";
    const EDIT_BUTTON_STYLE: &str = "bg-white px-3 py-2 sm:px-4 rounded-md text-gray-700 font-medium text-xs sm:text-sm border border-gray-300 shadow-sm transition-all hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 ml-2 sm:ml-3";
    const EDIT_BUTTON_CLICKED_STYLE: &str = "bg-indigo-100 px-3 py-2 sm:px-4 rounded-md text-indigo-700 font-medium text-xs sm:text-sm border border-[#2E3A59] shadow-sm transition-all hover:bg-indigo-50 focus:outline-none focus:ring-2 focus:ring-[#2E3A59] focus:ring-offset-2 ml-2 sm:ml-3";
    const DELETE_BUTTON_STYLE: &str = "bg-white px-3 py-2 sm:px-4 rounded-md text-red-600 font-medium text-xs sm:text-sm border border-gray-300 shadow-sm transition-all hover:bg-red-50 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2 ml-2 sm:ml-3";

    let (if_show_modal, set_if_show_modal) = create_signal(false);
    let (if_show_edit, set_if_show_edit) = create_signal(false);
    let (if_show_delete, set_if_show_delete) = create_signal(false);
    let (selected_test_id, set_selected_test_id) = create_signal(String::new());

    let (if_show_toast, set_if_show_toast) = create_signal(false);
    let (toast_message, set_toast_message) = create_signal(ToastMessage::new());

    let (search_term, set_search_term) = create_signal(String::new());

    let get_tests_info = create_resource(|| (), |_| async move { get_tests().await });

    let on_click_add = move |_| {
        let navigate = leptos_router::use_navigate();
        navigate("/testbuilder", Default::default());
    };

    let on_click_edit = move |_| {
        set_if_show_edit(!if_show_edit());
        // Disable delete mode when toggling edit mode
        set_if_show_delete(false);
    };

    let on_click_delete_mode = move |_| {
        set_if_show_delete(!if_show_delete());
        // Disable edit mode when toggling delete mode
        set_if_show_edit(false);
    };

    let on_delete_test = Callback::new(move |test_id: String| {
        let test_id_clone = test_id.clone();

        spawn_local(async move {
            let delete_test_request = DeleteTestRequest::new(test_id);

            let delete_result = delete_test(delete_test_request).await;

            match delete_result {
                Ok(_) => {
                    get_tests_info.refetch();
                    set_toast_message(ToastMessage::create(ToastMessageType::TestDeleted));
                    set_if_show_toast(true);
                }
                Err(e) => {
                    log::error!("Error deleting test: {:?}", e);
                    set_if_show_toast(true);
                }
            };
        });
    });

    view! {
        <div class="min-h-screen bg-[#F9F9F8]">
            <Header />
            <DashboardSidebar
                selected_item=selected_view
                set_selected_item=set_selected_view
            />
            <div class="ml-0 sm:ml-10 md:ml-30 transition-all duration-300 ease-in-out">
                <div class="max-w-7xl mx-auto px-3 sm:px-6 lg:px-8 py-6 sm:py-12">
                    <Toast
                        toast_message
                        if_appear=if_show_toast
                        set_if_appear=set_if_show_toast
                    />

                    {/* Page header */}
                    <div class="pb-4 sm:pb-5 border-b border-gray-200 mb-6 sm:mb-8">
                        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 sm:gap-0">
                            <div>
                                <h1 class="text-2xl sm:text-3xl font-bold text-[#2E3A59]">Math Tests</h1>
                                <p class="mt-1 sm:mt-2 text-xs sm:text-sm text-gray-500">
                                    "Manage your math test collection"
                                </p>
                            </div>

                            {/* Search bar - repositioned for mobile */}
                            <div class="relative w-full sm:w-auto order-3 sm:order-2 mt-4 sm:mt-0">
                                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                    <span class="text-gray-500 sm:text-sm">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                                            <path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd" />
                                        </svg>
                                    </span>
                                </div>
                                <input
                                    type="text"
                                    name="search"
                                    id="search"
                                    class="focus:ring-indigo-500 focus:border-indigo-500 block w-full pl-10 pr-3 text-xs sm:text-sm border-gray-300 rounded-md h-8 sm:h-10 border"
                                    placeholder="Search tests..."
                                    prop:value={move || search_term.get()}
                                    on:input=move |ev| set_search_term(event_target_value(&ev))
                                />
                            </div>

                            {/* Action buttons */}
                            <div class="flex space-x-2 sm:space-x-3 order-2 sm:order-3">
                                <button on:click=on_click_add class=ADD_BUTTON_STYLE>
                                    <div class="flex items-center">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 sm:h-4 sm:w-4 mr-1" viewBox="0 0 20 20" fill="currentColor">
                                            <path fill-rule="evenodd" d="M10 5a1 1 0 011 1v3h3a1 1 0 110 2h-3v3a1 1 0 11-2 0v-3H6a1 1 0 110-2h3V6a1 1 0 011-1z" clip-rule="evenodd" />
                                        </svg>
                                        <span class="hidden xs:inline">Add Test</span>
                                        <span class="inline xs:hidden">Add</span>
                                    </div>
                                </button>
                                <button
                                    on:click=on_click_edit
                                    class=move || if if_show_edit() {EDIT_BUTTON_CLICKED_STYLE} else {EDIT_BUTTON_STYLE}
                                >
                                    <div class="flex items-center">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 sm:h-4 sm:w-4 mr-1" viewBox="0 0 20 20" fill="currentColor">
                                            <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
                                        </svg>
                                        <span class="xs:inline">Edit</span>
                                    </div>
                                </button>
                                <button
                                    on:click=on_click_delete_mode
                                    class=move || if if_show_delete() {EDIT_BUTTON_CLICKED_STYLE} else {DELETE_BUTTON_STYLE}
                                >
                                    <div class="flex items-center">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 sm:h-4 sm:w-4 mr-1" viewBox="0 0 20 20" fill="currentColor">
                                            <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
                                        </svg>
                                        <span class="xs:inline">Delete</span>
                                    </div>
                                </button>
                            </div>
                        </div>
                    </div>

                    {/* Tests Grid */}
                    <Suspense fallback=move || {
                        view!{
                            <div class="flex justify-center items-center py-8 sm:py-12">
                                <div class="animate-pulse flex space-x-4">
                                    <div class="rounded-full bg-gray-200 h-10 w-10 sm:h-12 sm:w-12"></div>
                                    <div class="flex-1 space-y-3 sm:space-y-4 py-1">
                                        <div class="h-3 sm:h-4 bg-gray-200 rounded w-3/4"></div>
                                        <div class="space-y-2">
                                            <div class="h-3 sm:h-4 bg-gray-200 rounded"></div>
                                            <div class="h-3 sm:h-4 bg-gray-200 rounded w-5/6"></div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }
                    }>
                        <div class="grid grid-cols-1 gap-4 sm:gap-6">
                            {
                                move || {
                                    get_tests_info.get().map(|data| {
                                        match data {
                                            Ok(tests_data) => {
                                                let filtered_tests = tests_data.iter()
                                                    .filter(|test| test.testarea == TestType::Math &&
                                                        (search_term().is_empty() ||
                                                         test.name.to_lowercase().contains(&search_term().to_lowercase())))
                                                    .collect::<Vec<_>>();
                                                if filtered_tests.iter().filter(|test| test.testarea == TestType::Math).count() == 0 {
                                                    view! {
                                                        <div class="col-span-full text-center py-8 sm:py-12">
                                                            <svg class="mx-auto h-10 w-10 sm:h-12 sm:w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                                                            </svg>
                                                            <h3 class="mt-2 text-base sm:text-lg font-medium text-gray-900">No tests found</h3>
                                                            <p class="mt-1 text-xs sm:text-sm text-gray-500">Get started by creating a new math test.</p>
                                                            <div class="mt-4 sm:mt-6">
                                                                <button on:click=on_click_add class=ADD_BUTTON_STYLE>
                                                                    <div class="flex items-center justify-center">
                                                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 sm:h-4 sm:w-4 mr-1" viewBox="0 0 20 20" fill="currentColor">
                                                                            <path fill-rule="evenodd" d="M10 5a1 1 0 011 1v3h3a1 1 0 110 2h-3v3a1 1 0 11-2 0v-3H6a1 1 0 110-2h3V6a1 1 0 011-1z" clip-rule="evenodd" />
                                                                        </svg>
                                                                        "Create New Test"
                                                                    </div>
                                                                </button>
                                                            </div>
                                                        </div>
                                                    }.into_view()
                                                } else {
                                                    filtered_tests.into_iter()
                                                        .map(|each_test| {
                                                            let test_id = each_test.test_id.clone();

                                                            view!{
                                                                <div class="group relative bg-[#F9F9F8] rounded-lg overflow-hidden">
                                                                    <MathTestDisplay
                                                                        test=Rc::new(each_test.clone())
                                                                        test_resource=get_tests_info
                                                                        set_if_show_toast
                                                                        set_toast_message
                                                                        editing_mode=if_show_edit
                                                                        on_delete=Some(on_delete_test.clone())
                                                                        show_delete_mode=if_show_delete
                                                                    />
                                                                </div>
                                                            }
                                                        }).collect_view()
                                                }
                                            },
                                            Err(_) =>
                                                view! {
                                                    <div class="col-span-full rounded-md bg-red-50 p-3 sm:p-4">
                                                        <div class="flex">
                                                            <div class="flex-shrink-0">
                                                                <svg class="h-4 w-4 sm:h-5 sm:w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                                                                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                                                                </svg>
                                                            </div>
                                                            <div class="ml-3">
                                                                <h3 class="text-xs sm:text-sm font-medium text-red-800">Error loading tests</h3>
                                                                <div class="mt-2 text-xs sm:text-sm text-red-700">
                                                                    <p>Please try refreshing the page or contact support if the problem persists.</p>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    </div>
                                                }.into_view()
                                        }
                                    })
                                }
                            }
                        </div>
                    </Suspense>
                </div>
            </div>
        </div>
    }
}
