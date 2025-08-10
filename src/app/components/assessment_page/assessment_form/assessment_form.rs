use crate::app::components::assessment_page::assessment_form::{
    basic_info_section::BasicInfoSection, benchmark_section::BenchmarkSection,
    test_selection_section::TestSelectionSection,
};
use crate::app::components::assessment_page::sequence_builder::SequenceBuilder;
use crate::app::components::assessment_page::shared::{
    hooks::UseAssessmentForm, types::AssessmentFormState,
};
use crate::app::models::assessment::{
    CreateNewAssessmentRequest, RangeCategory, UpdateAssessmentRequest,
};
use crate::app::models::test::Test;
use crate::app::server_functions::assessments::{add_assessment, update_assessment};
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn AssessmentForm(
    show_modal: ReadSignal<bool>,
    set_show_modal: WriteSignal<bool>,
    form_hook: UseAssessmentForm,
    // Fix: Updated Resource type signatures to match Leptos 0.8
    tests_resource: Resource<Result<Vec<Test>, ServerFnError>>,
    courses_resource: Resource<Result<Vec<crate::app::models::course::Course>, ServerFnError>>,
    on_success: impl Fn() + 'static + Copy,
) -> impl IntoView {
    // Form submission logic
    let submit_form = Action::new(move |_: &()| {
        let state = form_hook.state.get();
        let editing = form_hook.editing.get();
        let selected_id = form_hook.selected_assessment_id.get();

        async move {
            let composite = calculate_composite_score(&state, &tests_resource).await;

            if editing && selected_id.is_some() {
                if state.use_sequences {
                    let request = UpdateAssessmentRequest::new_with_sequence(
                        state.name,
                        state.frequency,
                        state.grade,
                        state.version,
                        selected_id.unwrap(),
                        composite,
                        state.risk_benchmarks,
                        state.national_benchmarks,
                        state.subject,
                        state.scope,
                        state.course_id,
                        state.test_sequence,
                    );
                    update_assessment(request).await
                } else {
                    let request = UpdateAssessmentRequest::new(
                        state.name,
                        state.frequency,
                        state.grade,
                        state.version,
                        selected_id.unwrap(),
                        state.selected_tests,
                        composite,
                        state.risk_benchmarks,
                        state.national_benchmarks,
                        state.subject,
                        state.scope,
                        state.course_id,
                    );
                    update_assessment(request).await
                }
            } else {
                if state.use_sequences {
                    let request = CreateNewAssessmentRequest::new_with_sequence(
                        state.name,
                        state.frequency,
                        state.grade,
                        state.version,
                        composite,
                        state.risk_benchmarks,
                        state.national_benchmarks,
                        state.subject,
                        state.scope,
                        state.course_id,
                        state.test_sequence,
                    );
                    add_assessment(request).await
                } else {
                    let request = CreateNewAssessmentRequest::new(
                        state.name,
                        state.frequency,
                        state.grade,
                        state.version,
                        state.selected_tests,
                        composite,
                        state.risk_benchmarks,
                        state.national_benchmarks,
                        state.subject,
                        state.scope,
                        state.course_id,
                    );
                    add_assessment(request).await
                }
            }
        }
    });

    // Handle form submission success
    Effect::new(move |_| {
        if let Some(Ok(_)) = submit_form.value().get() {
            form_hook.reset_form.run(()); // Fixed: Added ()
            set_show_modal.set(false);
            on_success();
        }
    });

    view! {
        <Show when=move || show_modal.get()>
            <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50 modal-overlay">
                <div class="bg-white rounded-lg shadow-xl max-w-6xl w-full max-h-[90vh] flex flex-col modal-content">
                    <FormHeader
                        editing=form_hook.editing
                        on_close=move || {
                            set_show_modal.set(false);
                            form_hook.reset_form.run(());  // Fixed: Added ()
                        }
                    />

                    <div class="flex-1 overflow-y-auto bg-white">
                        <div class="p-6 bg-white">
                            <form on:submit=move |ev| {
                                ev.prevent_default();
                                submit_form.dispatch(());
                            }>
                                <div class="space-y-6">
                                    <BasicInfoSection
                                        state=form_hook.state
                                        set_state=form_hook.set_state
                                        courses_resource=courses_resource
                                    />

                                    <TestManagementSection
                                        state=form_hook.state
                                        set_state=form_hook.set_state
                                        tests_resource=tests_resource
                                    />

                                    <BenchmarkSections
                                        state=form_hook.state
                                        set_state=form_hook.set_state
                                    />
                                </div>
                            </form>
                        </div>
                    </div>

                    <FormFooter
                        editing=form_hook.editing
                        on_cancel=move || {
                            set_show_modal.set(false);
                            form_hook.reset_form.run(());  // Fixed: Added ()
                        }
                        on_submit=move || {
                            let _ = submit_form.dispatch(());
                        }
                    />
                </div>
            </div>
        </Show>
    }
}

#[component]
fn FormHeader(editing: ReadSignal<bool>, on_close: impl Fn() + 'static + Copy) -> impl IntoView {
    view! {
        <div class="sticky top-0 bg-white border-b border-gray-200 px-6 py-4 flex justify-between items-center z-10 rounded-t-lg">
            <h2 class="text-xl font-medium text-[#2E3A59]">
                {move || if editing.get() { "Edit Assessment" } else { "Create New Assessment" }}
            </h2>
            <button
                class="text-gray-400 hover:text-gray-600 text-2xl leading-none p-1 hover:bg-gray-100 rounded transition-colors"
                on:click=move |_| on_close()
            >
                "×"
            </button>
        </div>
    }
}

#[component]
fn TestManagementSection(
    state: ReadSignal<AssessmentFormState>,
    set_state: WriteSignal<AssessmentFormState>,
    // Fix: Updated Resource type signature
    tests_resource: Resource<Result<Vec<Test>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <div class="border-t border-gray-200 pt-6 bg-white">
            <div class="mb-4 bg-white p-4 rounded-lg">
                <h3 class="text-lg font-medium mb-3 text-gray-900">"Test Management"</h3>
                <div class="flex items-center space-x-4">
                    <label class="inline-flex items-center">
                        <input
                            type="radio"
                            name="test_mode"
                            class="form-radio h-4 w-4 text-[#2E3A59] focus:ring-[#2E3A59] border-gray-300"
                            prop:checked={move || !state.get().use_sequences}
                            on:change=move |_| set_state.update(|s| s.use_sequences = false)
                        />
                        <span class="ml-2 text-sm text-gray-700">"Simple Test List"</span>
                    </label>
                    <label class="inline-flex items-center">
                        <input
                            type="radio"
                            name="test_mode"
                            class="form-radio h-4 w-4 text-[#2E3A59] focus:ring-[#2E3A59] border-gray-300"
                            prop:checked={move || state.get().use_sequences}
                            on:change=move |_| set_state.update(|s| s.use_sequences = true)
                        />
                        <span class="ml-2 text-sm text-gray-700">"Advanced Sequencing"</span>
                    </label>
                </div>
                <p class="text-xs text-gray-500 mt-1">
                    "Advanced sequencing allows you to control test flow with requirements and branching logic."
                </p>
            </div>

            <div class="bg-white">
                <Show
                    when=move || !state.get().use_sequences
                    fallback=move || view! {
                        <div class="bg-white p-4 rounded-lg">
                            <SequenceBuilder
                                state=state
                                set_state=set_state
                                tests_resource=tests_resource
                            />
                        </div>
                    }
                >
                    <div class="bg-white p-4 rounded-lg">
                        <TestSelectionSection
                            state=state
                            set_state=set_state
                            tests_resource=tests_resource
                        />
                    </div>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn BenchmarkSections(
    state: ReadSignal<AssessmentFormState>,
    set_state: WriteSignal<AssessmentFormState>,
) -> impl IntoView {
    let handle_risk_benchmarks_change = move |benchmarks: Option<Vec<RangeCategory>>| {
        set_state.update(|s| s.risk_benchmarks = benchmarks);
    };

    let handle_national_benchmarks_change = move |benchmarks: Option<Vec<RangeCategory>>| {
        set_state.update(|s| s.national_benchmarks = benchmarks);
    };

    view! {
        <div class="space-y-6">
            <div class="border-t border-gray-200 pt-6 bg-white p-4 rounded-lg">
                <h3 class="text-lg font-medium mb-4 text-gray-900">"Risk Benchmarks"</h3>
                <BenchmarkSection
                    benchmarks=Signal::derive(move || state.get().risk_benchmarks)
                    set_benchmarks=handle_risk_benchmarks_change
                    section_type="risk"
                />
            </div>

            <div class="border-t border-gray-200 pt-6 bg-white p-4 rounded-lg">
                <h3 class="text-lg font-medium mb-4 text-gray-900">"National Benchmarks"</h3>
                <BenchmarkSection
                    benchmarks=Signal::derive(move || state.get().national_benchmarks)
                    set_benchmarks=handle_national_benchmarks_change
                    section_type="national"
                />
            </div>
        </div>
    }
}

#[component]
fn FormFooter(
    editing: ReadSignal<bool>,
    on_cancel: impl Fn() + 'static + Copy,
    on_submit: impl Fn() + 'static + Copy,
) -> impl IntoView {
    view! {
        <div class="sticky bottom-0 bg-white border-t border-gray-200 px-6 py-4 flex justify-end space-x-3 rounded-b-lg">
            <button
                type="button"
                class="px-4 py-2 bg-gray-100 text-gray-700 rounded border border-gray-300 hover:bg-gray-200 transition-colors text-sm font-medium"
                on:click=move |_| on_cancel()
            >
                "Cancel"
            </button>
            <button
                type="submit"
                class="px-4 py-2 bg-[#2E3A59] text-white rounded shadow-sm hover:bg-[#1e293b] transition-colors text-sm font-medium"
                on:click=move |ev| {
                    ev.prevent_default();
                    on_submit();
                }
            >
                {move || if editing.get() { "Update Assessment" } else { "Create Assessment" }}
            </button>
        </div>
    }
}

// Fix: Updated function signature and resource access pattern for Leptos 0.8
async fn calculate_composite_score(
    state: &AssessmentFormState,
    tests_resource: &Resource<Result<Vec<Test>, ServerFnError>>,
) -> Option<i32> {
    let tests_to_sum = if state.use_sequences {
        if state.test_sequence.is_empty() {
            return None;
        }
        state
            .test_sequence
            .iter()
            .map(|seq_item| seq_item.test_id)
            .collect()
    } else {
        if state.selected_tests.is_empty() {
            return None;
        }
        state.selected_tests.clone()
    };

    // Fix: Updated resource access pattern for Leptos 0.8
    tests_resource
        .get()
        .and_then(|result| result.ok())
        .map(|tests| {
            tests
                .iter()
                .filter(|test| {
                    let test_uuid = Uuid::parse_str(&test.test_id).unwrap_or_default();
                    tests_to_sum.contains(&test_uuid)
                })
                .map(|test| test.score)
                .sum()
        })
}
