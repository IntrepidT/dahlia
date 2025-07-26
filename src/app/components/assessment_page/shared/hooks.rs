use super::types::*;
use crate::app::models::assessment::Assessment;
use leptos::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct UseAssessmentForm {
    pub state: ReadSignal<AssessmentFormState>,
    pub set_state: WriteSignal<AssessmentFormState>,
    pub editing: ReadSignal<bool>,
    pub set_editing: WriteSignal<bool>,
    pub selected_assessment_id: ReadSignal<Option<Uuid>>,
    pub set_selected_assessment_id: WriteSignal<Option<Uuid>>,
    pub reset_form: Callback<()>,
    pub load_assessment: Callback<Assessment>,
}

pub fn use_assessment_form() -> UseAssessmentForm {
    let (state, set_state) = create_signal(AssessmentFormState::default());
    let (editing, set_editing) = create_signal(false);
    let (selected_assessment_id, set_selected_assessment_id) = create_signal::<Option<Uuid>>(None);

    let reset_form = Callback::new(move |_: ()| {
        set_state.set(AssessmentFormState::default());
        set_editing.set(false);
        set_selected_assessment_id.set(None);
    });

    let load_assessment = Callback::new(move |assessment: Assessment| {
        logging::log!("Loading assessment for editing: {:?}", assessment.name);

        // Start with default state to ensure all fields are properly initialized
        let mut new_state = AssessmentFormState {
            // Populate all the basic fields
            name: assessment.name.clone(),
            frequency: assessment.frequency,
            grade: assessment.grade,
            version: assessment.version,
            subject: assessment.subject,
            scope: assessment.scope,
            course_id: assessment.course_id,
            risk_benchmarks: assessment.risk_benchmarks.clone(),
            national_benchmarks: assessment.national_benchmarks.clone(),

            ..AssessmentFormState::default()
        };

        // Handle test sequences vs simple test selection
        if let Some(sequence) = assessment.test_sequence.clone() {
            if !sequence.is_empty() {
                // Assessment uses sequences
                logging::log!(
                    "Loading assessment with sequences: {} items",
                    sequence.len()
                );
                new_state.use_sequences = true;
                new_state.test_sequence = sequence;
                new_state.selected_tests = vec![]; // Clear simple test selection
            } else {
                // Assessment has empty sequence, use simple test selection
                logging::log!(
                    "Loading assessment with empty sequence, using simple tests: {} items",
                    assessment.tests.len()
                );
                new_state.use_sequences = false;
                new_state.test_sequence = vec![];
                new_state.selected_tests = assessment.tests.clone();
            }
        } else {
            // No sequence data, definitely using simple test selection
            logging::log!(
                "Loading assessment with simple tests: {} items",
                assessment.tests.len()
            );
            new_state.use_sequences = false;
            new_state.test_sequence = vec![];
            new_state.selected_tests = assessment.tests.clone();
        }

        logging::log!(
            "Final state - use_sequences: {}, selected_tests: {:?}, test_sequence: {:?}",
            new_state.use_sequences,
            new_state.selected_tests,
            new_state.test_sequence
        );

        // Apply the state
        set_state.set(new_state);
        set_editing.set(true);
        set_selected_assessment_id.set(Some(assessment.id));
    });

    UseAssessmentForm {
        state,
        set_state,
        editing,
        set_editing,
        selected_assessment_id,
        set_selected_assessment_id,
        reset_form,
        load_assessment,
    }
}

#[derive(Clone)]
pub struct UseSequenceBuilder {
    pub state: ReadSignal<SequenceBuilderState>,
    pub set_state: WriteSignal<SequenceBuilderState>,
    pub add_test_to_sequence: Callback<
        (
            Uuid,
            Vec<crate::app::models::assessment_sequences::TestSequenceItem>,
        ),
        Vec<crate::app::models::assessment_sequences::TestSequenceItem>,
    >,
    pub remove_from_sequence: Callback<
        (
            Uuid,
            Vec<crate::app::models::assessment_sequences::TestSequenceItem>,
        ),
        Vec<crate::app::models::assessment_sequences::TestSequenceItem>,
    >,
    pub reorder_sequence: Callback<
        (
            usize,
            usize,
            Vec<crate::app::models::assessment_sequences::TestSequenceItem>,
        ),
        Vec<crate::app::models::assessment_sequences::TestSequenceItem>,
    >,
}

pub fn use_sequence_builder() -> UseSequenceBuilder {
    let (state, set_state) = create_signal(SequenceBuilderState::default());

    let add_test_to_sequence = Callback::new(
        move |(test_id, current_sequence): (
            Uuid,
            Vec<crate::app::models::assessment_sequences::TestSequenceItem>,
        )|
              -> Vec<crate::app::models::assessment_sequences::TestSequenceItem> {
            let current_state = state.get();
            let order = current_state.sequence_counter;

            let new_item = match current_state.sequence_behavior {
                crate::app::models::assessment_sequences::SequenceBehavior::Attainment => {
                    let mut item =
                        crate::app::models::assessment_sequences::TestSequenceItem::new_attainment(
                            test_id,
                            order,
                            current_state.required_score.unwrap_or(70),
                            None,
                            None,
                        );
                    if !current_state.variation_levels.is_empty() {
                        item.variation_levels = Some(current_state.variation_levels.clone());
                    }
                    item
                }
                crate::app::models::assessment_sequences::SequenceBehavior::Node => {
                    crate::app::models::assessment_sequences::TestSequenceItem::new_node(
                        test_id, order,
                    )
                }
                crate::app::models::assessment_sequences::SequenceBehavior::Optional => {
                    crate::app::models::assessment_sequences::TestSequenceItem::new_optional(
                        test_id, order,
                    )
                }
                crate::app::models::assessment_sequences::SequenceBehavior::Diagnostic => {
                    crate::app::models::assessment_sequences::TestSequenceItem::new_diagnostic(
                        test_id, order,
                    )
                }
                crate::app::models::assessment_sequences::SequenceBehavior::Remediation => {
                    crate::app::models::assessment_sequences::TestSequenceItem::new_remediation(
                        test_id,
                        order,
                        vec![],
                    )
                }
                crate::app::models::assessment_sequences::SequenceBehavior::Branching => {
                    crate::app::models::assessment_sequences::TestSequenceItem::new_branching(
                        test_id,
                        order,
                        vec![],
                    )
                }
            };

            let mut updated_sequence = current_sequence;
            updated_sequence.push(new_item);
            updated_sequence.sort_by_key(|item| item.sequence_order);

            // Update counter
            set_state.update(|s| s.sequence_counter = order + 1);

            updated_sequence
        },
    );

    let remove_from_sequence = Callback::new(
        move |(test_id, current_sequence): (
            Uuid,
            Vec<crate::app::models::assessment_sequences::TestSequenceItem>,
        )|
              -> Vec<crate::app::models::assessment_sequences::TestSequenceItem> {
            let mut updated = current_sequence;
            updated.retain(|item| item.test_id != test_id);

            // Reorder
            for (index, item) in updated.iter_mut().enumerate() {
                item.sequence_order = (index + 1) as i32;
            }

            updated
        },
    );

    let reorder_sequence = Callback::new(
        move |(source_index, target_index, current_sequence): (
            usize,
            usize,
            Vec<crate::app::models::assessment_sequences::TestSequenceItem>,
        )|
              -> Vec<crate::app::models::assessment_sequences::TestSequenceItem> {
            if source_index == target_index {
                return current_sequence;
            }

            let mut updated = current_sequence;
            if source_index < updated.len() && target_index < updated.len() {
                let item = updated.remove(source_index);
                updated.insert(target_index, item);

                // Reorder sequence numbers
                for (index, item) in updated.iter_mut().enumerate() {
                    item.sequence_order = (index + 1) as i32;
                }
            }

            updated
        },
    );

    UseSequenceBuilder {
        state,
        set_state,
        add_test_to_sequence,
        remove_from_sequence,
        reorder_sequence,
    }
}
