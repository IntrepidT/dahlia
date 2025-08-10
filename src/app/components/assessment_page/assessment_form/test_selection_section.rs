use crate::app::components::assessment_page::shared::types::{
    is_variation_test, AssessmentFormState,
};
use crate::app::models::test::Test;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn TestSelectionSection(
    state: ReadSignal<AssessmentFormState>,
    set_state: WriteSignal<AssessmentFormState>,
    tests_resource: Resource<Result<Vec<Test>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <div class="bg-white p-4 rounded-lg">
            <label class="block text-sm font-medium mb-3 text-gray-700">"Tests"</label>
            {move || tests_resource.get().map(|tests_result| {
                match tests_result {
                    Ok(tests) => {
                        // Filter out variation tests for simple selection
                        let main_tests: Vec<Test> = tests.into_iter()
                            .filter(|test| !is_variation_test(test))
                            .collect();

                        view! {
                            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2 max-h-64 overflow-y-auto p-3 bg-gray-50 rounded-md border border-gray-200">
                                {main_tests.into_iter().map(|test| {
                                    let test_id = Uuid::parse_str(&test.test_id).expect("Did not convert uuid to string");
                                    let test_name = test.name.clone();

                                    view! {
                                        <TestCheckbox
                                            test_id=test_id
                                            test_name=test_name
                                            selected_tests=Signal::derive(move || state.get().selected_tests)
                                            on_toggle=move |test_id: Uuid, checked: bool| {
                                                set_state.update(|s| {
                                                    if checked && !s.selected_tests.contains(&test_id) {
                                                        s.selected_tests.push(test_id);
                                                    } else if !checked {
                                                        s.selected_tests.retain(|&id| id != test_id);
                                                    }
                                                });
                                            }
                                        />
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    },
                    Err(e) => view! {
                        <div class="p-4 bg-red-50 text-red-700 rounded border border-red-200">
                            "Error loading tests: " {e.to_string()}
                        </div>
                    }.into_any()
                }
            })}
        </div>
    }
}

#[component]
fn TestCheckbox(
    test_id: Uuid,
    test_name: String,
    selected_tests: Signal<Vec<Uuid>>,
    on_toggle: impl Fn(Uuid, bool) + 'static + Copy,
) -> impl IntoView {
    view! {
        <div class="flex items-center space-x-2 p-2 rounded hover:bg-white transition-colors">
            <input
                type="checkbox"
                id={format!("test-{}", test_id)}
                class="h-4 w-4 text-[#2E3A59] rounded border-gray-300 focus:ring-[#2E3A59]"
                prop:checked={move || selected_tests.get().contains(&test_id)}
                on:change=move |ev| {
                    let checked = event_target_checked(&ev);
                    on_toggle(test_id, checked);
                }
            />
            <label for={format!("test-{}", test_id)} class="text-sm text-gray-700 cursor-pointer">
                {test_name}
            </label>
        </div>
    }
}
