use crate::app::models::assessment::{Assessment, RangeCategory, ScopeEnum, SubjectEnum};
use crate::app::models::assessment_sequences::{
    SequenceBehavior, TestSequenceItem, VariationLevel,
};
use crate::app::models::student::GradeEnum;
use crate::app::models::test::Test;
use leptos::prelude::*;
use leptos::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct AssessmentFormState {
    pub name: String,
    pub frequency: Option<i32>,
    pub grade: Option<GradeEnum>,
    pub version: i32,
    pub subject: Option<SubjectEnum>,
    pub scope: Option<ScopeEnum>,
    pub course_id: Option<i32>,
    pub selected_tests: Vec<Uuid>,
    pub test_sequence: Vec<TestSequenceItem>,
    pub use_sequences: bool,
    pub risk_benchmarks: Option<Vec<RangeCategory>>,
    pub national_benchmarks: Option<Vec<RangeCategory>>,
}

impl Default for AssessmentFormState {
    fn default() -> Self {
        Self {
            name: String::new(),
            frequency: None,
            grade: None,
            version: 1,
            subject: None,
            scope: None,
            course_id: None,
            selected_tests: vec![],
            test_sequence: vec![],
            use_sequences: false,
            risk_benchmarks: None,
            national_benchmarks: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SequenceBuilderState {
    pub sequence_counter: i32,
    pub dragging_item: Option<usize>,
    pub show_sequence_details: Option<usize>,
    pub selected_test_for_sequence: Option<Uuid>,
    pub sequence_behavior: SequenceBehavior,
    pub required_score: Option<i32>,
    pub variation_levels: Vec<VariationLevel>,
    pub show_variations_panel: bool,
    pub editing_variation_index: Option<usize>,
}

impl Default for SequenceBuilderState {
    fn default() -> Self {
        Self {
            sequence_counter: 1,
            dragging_item: None,
            show_sequence_details: None,
            selected_test_for_sequence: None,
            sequence_behavior: SequenceBehavior::Node,
            required_score: None,
            variation_levels: vec![],
            show_variations_panel: false,
            editing_variation_index: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BenchmarkFormState {
    pub min: i32,
    pub max: i32,
    pub label: String,
}

impl Default for BenchmarkFormState {
    fn default() -> Self {
        Self {
            min: 0,
            max: 0,
            label: String::new(),
        }
    }
}

// Common utility functions
pub fn is_variation_test(test: &Test) -> bool {
    test.name.contains(" - ")
        && (test.name.to_lowercase().contains("randomized")
            || test.name.to_lowercase().contains("distinct")
            || test.name.to_lowercase().contains("practice")
            || test.comments.to_lowercase().contains("variation:"))
}

pub fn get_behavior_display_props(
    behavior: &SequenceBehavior,
) -> (&'static str, &'static str, &'static str, &'static str) {
    match behavior {
        SequenceBehavior::Node => ("#3b82f6", "→", "#2563eb", "Node"),
        SequenceBehavior::Attainment => ("#10b981", "✓", "#059669", "Attainment"),
        SequenceBehavior::Optional => ("#6b7280", "?", "#4b5563", "Optional"),
        SequenceBehavior::Diagnostic => ("#8b5cf6", "📊", "#7c3aed", "Diagnostic"),
        SequenceBehavior::Remediation => ("#f59e0b", "🔧", "#d97706", "Remediation"),
        SequenceBehavior::Branching => ("#eab308", "⚡", "#ca8a04", "Branching"),
    }
}
