use crate::app::components::assessment_page::sequence_builder::{
    test_add_panel::TestAddPanel,
    sequence_flow::SequenceFlow,
};
use crate::app::components::assessment_page::shared::{
    types::AssessmentFormState,
    hooks::{use_sequence_builder, UseSequenceBuilder},
};
use crate::app::models::test::Test;
use leptos::*;

#[component]
pub fn SequenceBuilder(
    state: ReadSignal<AssessmentFormState>,
    set_state: WriteSignal<AssessmentFormState>,
    tests_resource: Resource<(), Result<Vec<Test>, ServerFnError>>,
) -> impl IntoView {
    let sequence_builder = use_sequence_builder();

    // Sync sequence changes to main state
    let update_sequence = move |new_sequence| {
        set_state.update(|s| s.test_sequence = new_sequence);
    };

    view! {
        <div class="sequence-builder bg-white p-6 rounded-lg border border-gray-200">
            <h4 class="text-gray-900 text-lg font-medium mb-4">"Advanced Visual Sequence Builder"</h4>

            <TestAddPanel
                tests_resource=tests_resource
                sequence_builder=sequence_builder.clone()
                current_sequence=Signal::derive(move || state.get().test_sequence)
                on_sequence_change=update_sequence
            />

            <SequenceFlow
                sequence=Signal::derive(move || state.get().test_sequence)
                tests_resource=tests_resource
                sequence_builder=sequence_builder
                on_sequence_change=update_sequence
            />
        </div>
    }
}
