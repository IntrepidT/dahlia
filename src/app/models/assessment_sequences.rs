use crate::app::models::test::Test;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use strum_macros::EnumIter;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum TestStatus {
    NotAttempted,
    Completed(i32),
    Passed(i32),
    Failed(i32),
}

#[derive(Debug, Clone)]
pub struct VariationStatus {
    pub level: i32,
    pub test_name: String,
    pub test_id: Uuid,
    pub status: TestStatus,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct LearningPathItem {
    pub test_name: String,
    pub test_id: Uuid,
    pub behavior: SequenceBehavior,
    pub status: TestStatus,
    pub required_score: Option<i32>,
    pub variations: Vec<VariationStatus>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct VariationLevel {
    pub level: i32, // 1, 2, or 3
    pub test_id: Uuid,
    pub required_score: Option<i32>, // Score needed to pass this variation
    pub max_attempts: Option<i32>,   // Attempts allowed at this level
    pub description: String,         // "Practice A", "Remedial", "Guided", etc.
}

impl VariationLevel {
    pub fn new(level: i32, test_id: Uuid, description: String) -> Self {
        VariationLevel {
            level,
            test_id,
            required_score: Some(60), // Default lower threshold for variations
            max_attempts: Some(2),
            description,
        }
    }

    pub fn new_with_score(
        level: i32,
        test_id: Uuid,
        description: String,
        required_score: i32,
    ) -> Self {
        VariationLevel {
            level,
            test_id,
            required_score: Some(required_score),
            max_attempts: Some(2),
            description,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, EnumIter)]
pub enum SequenceBehavior {
    #[strum(to_string = "attainment")]
    Attainment,
    #[strum(to_string = "node")]
    Node,
    #[strum(to_string = "optional")]
    Optional,
    #[strum(to_string = "diagnostic")]
    Diagnostic,
    #[strum(to_string = "remediation")]
    Remediation,
    #[strum(to_string = "branching")]
    Branching,
}

impl fmt::Display for SequenceBehavior {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SequenceBehavior::Attainment => "attainment",
                SequenceBehavior::Node => "node",
                SequenceBehavior::Optional => "optional",
                SequenceBehavior::Diagnostic => "diagnostic",
                SequenceBehavior::Remediation => "remediation",
                SequenceBehavior::Branching => "branching",
            }
        )
    }
}

impl FromStr for SequenceBehavior {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "attainment" | "Attainment" => Ok(SequenceBehavior::Attainment),
            "node" | "Node" => Ok(SequenceBehavior::Node),
            "optional" | "Optional" => Ok(SequenceBehavior::Optional),
            "diagnostic" | "Diagnostic" => Ok(SequenceBehavior::Diagnostic),
            "remediation" | "Remediation" => Ok(SequenceBehavior::Remediation),
            "branching" | "Branching" => Ok(SequenceBehavior::Branching),
            _ => Err(format!("Invalid sequence behavior value: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ScoreRange {
    pub min: i32,
    pub max: i32,
    pub next_test: Option<Uuid>,
}

impl ScoreRange {
    pub fn new(min: i32, max: i32, next_test: Option<Uuid>) -> Self {
        ScoreRange {
            min,
            max,
            next_test,
        }
    }
}

// Updated TestSequenceItem with multi-level variation support
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TestSequenceItem {
    pub test_id: Uuid,
    pub sequence_behavior: SequenceBehavior,
    pub sequence_order: i32,
    pub required_score: Option<i32>,
    pub next_on_pass: Option<Uuid>,
    pub next_on_fail: Option<Uuid>,
    pub variation_levels: Option<Vec<VariationLevel>>,
    pub score_ranges: Option<Vec<ScoreRange>>,
    pub max_attempts: Option<i32>,
    pub time_limit_minutes: Option<i32>,
    pub prerequisite_tests: Option<Vec<Uuid>>,
    pub skip_conditions: Option<Vec<Uuid>>,
    pub show_feedback: bool,
    pub allow_review: bool,
    pub randomize_questions: bool,
    pub adaptive_difficulty: bool,
}

impl TestSequenceItem {
    pub fn new_node(test_id: Uuid, sequence_order: i32) -> Self {
        TestSequenceItem {
            test_id,
            sequence_behavior: SequenceBehavior::Node,
            sequence_order,
            required_score: None,
            next_on_pass: None,
            next_on_fail: None,
            variation_levels: None,
            score_ranges: None,
            max_attempts: Some(1),
            time_limit_minutes: None,
            prerequisite_tests: None,
            skip_conditions: None,
            show_feedback: true,
            allow_review: false,
            randomize_questions: false,
            adaptive_difficulty: false,
        }
    }

    pub fn new_attainment(
        test_id: Uuid,
        sequence_order: i32,
        required_score: i32,
        next_on_pass: Option<Uuid>,
        next_on_fail: Option<Uuid>,
    ) -> Self {
        TestSequenceItem {
            test_id,
            sequence_behavior: SequenceBehavior::Attainment,
            sequence_order,
            required_score: Some(required_score),
            next_on_pass,
            next_on_fail,
            variation_levels: None,
            score_ranges: None,
            max_attempts: Some(3),
            time_limit_minutes: None,
            prerequisite_tests: None,
            skip_conditions: None,
            show_feedback: true,
            allow_review: true,
            randomize_questions: false,
            adaptive_difficulty: false,
        }
    }

    // NEW: Create attainment with multi-level variations
    pub fn new_attainment_with_variations(
        test_id: Uuid,
        sequence_order: i32,
        required_score: i32,
        variation_levels: Vec<VariationLevel>,
    ) -> Self {
        TestSequenceItem {
            test_id,
            sequence_behavior: SequenceBehavior::Attainment,
            sequence_order,
            required_score: Some(required_score),
            next_on_pass: None,
            next_on_fail: None,
            variation_levels: Some(variation_levels),
            score_ranges: None,
            max_attempts: Some(3),
            time_limit_minutes: None,
            prerequisite_tests: None,
            skip_conditions: None,
            show_feedback: true,
            allow_review: true,
            randomize_questions: false,
            adaptive_difficulty: false,
        }
    }

    pub fn new_branching(
        test_id: Uuid,
        sequence_order: i32,
        score_ranges: Vec<ScoreRange>,
    ) -> Self {
        TestSequenceItem {
            test_id,
            sequence_behavior: SequenceBehavior::Branching,
            sequence_order,
            required_score: None,
            next_on_pass: None,
            next_on_fail: None,
            variation_levels: None,
            score_ranges: Some(score_ranges),
            max_attempts: Some(1),
            time_limit_minutes: None,
            prerequisite_tests: None,
            skip_conditions: None,
            show_feedback: true,
            allow_review: false,
            randomize_questions: false,
            adaptive_difficulty: false,
        }
    }

    pub fn new_optional(test_id: Uuid, sequence_order: i32) -> Self {
        TestSequenceItem {
            test_id,
            sequence_behavior: SequenceBehavior::Optional,
            sequence_order,
            required_score: None,
            next_on_pass: None,
            next_on_fail: None,
            variation_levels: None,
            score_ranges: None,
            max_attempts: Some(1),
            time_limit_minutes: None,
            prerequisite_tests: None,
            skip_conditions: None,
            show_feedback: true,
            allow_review: true,
            randomize_questions: false,
            adaptive_difficulty: false,
        }
    }

    pub fn new_diagnostic(test_id: Uuid, sequence_order: i32) -> Self {
        TestSequenceItem {
            test_id,
            sequence_behavior: SequenceBehavior::Diagnostic,
            sequence_order,
            required_score: None,
            next_on_pass: None,
            next_on_fail: None,
            variation_levels: None,
            score_ranges: None,
            max_attempts: Some(1),
            time_limit_minutes: None,
            prerequisite_tests: None,
            skip_conditions: None,
            show_feedback: false,
            allow_review: false,
            randomize_questions: true,
            adaptive_difficulty: true,
        }
    }

    pub fn new_remediation(
        test_id: Uuid,
        sequence_order: i32,
        prerequisite_tests: Vec<Uuid>,
    ) -> Self {
        TestSequenceItem {
            test_id,
            sequence_behavior: SequenceBehavior::Remediation,
            sequence_order,
            required_score: None,
            next_on_pass: None,
            next_on_fail: None,
            variation_levels: None,
            score_ranges: None,
            max_attempts: Some(5),
            time_limit_minutes: None,
            prerequisite_tests: Some(prerequisite_tests),
            skip_conditions: None,
            show_feedback: true,
            allow_review: true,
            randomize_questions: false,
            adaptive_difficulty: false,
        }
    }

    // NEW: Helper methods for variation support
    pub fn has_variations(&self) -> bool {
        self.variation_levels
            .as_ref()
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }

    pub fn get_variation_count(&self) -> usize {
        self.variation_levels.as_ref().map(|v| v.len()).unwrap_or(0)
    }

    pub fn get_variation_by_level(&self, level: i32) -> Option<&VariationLevel> {
        self.variation_levels
            .as_ref()?
            .iter()
            .find(|v| v.level == level)
    }

    pub fn add_variation_level(&mut self, variation: VariationLevel) {
        if let Some(ref mut variations) = self.variation_levels {
            variations.push(variation);
            variations.sort_by_key(|v| v.level);
        } else {
            self.variation_levels = Some(vec![variation]);
        }
    }

    pub fn remove_variation_level(&mut self, level: i32) {
        if let Some(ref mut variations) = self.variation_levels {
            variations.retain(|v| v.level != level);
            if variations.is_empty() {
                self.variation_levels = None;
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use sqlx::{Postgres, Encode, Decode, Type, postgres::{PgTypeInfo, PgValueRef, PgArgumentBuffer}, encode::IsNull};
        use sqlx::types::Json;

        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for SequenceBehavior {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                let s = self.to_string();
                <&str as Encode<Postgres>>::encode(&s.as_str(), buf)
            }
        }
        impl<'r> sqlx::decode::Decode<'r, sqlx::Postgres> for SequenceBehavior {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let s: &str = Decode::<sqlx::Postgres>::decode(value)?;
                SequenceBehavior::from_str(s).map_err(|_| format!("Invalid SequenceBehavior: {}", s).into())
            }
        }
        impl Type<Postgres> for SequenceBehavior {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("sequence_behavior_enum")
            }
        }

        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for VariationLevel {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                Json(self).encode_by_ref(buf)
            }
        }
        impl sqlx::Type<sqlx::Postgres> for VariationLevel {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("jsonb")
            }
        }
        impl<'r> sqlx::decode::Decode<'r, sqlx::Postgres> for VariationLevel {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let json: Json<VariationLevel> = sqlx::decode::Decode::decode(value)?;
                Ok(json.0)
            }
        }

        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for TestSequenceItem {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                Json(self).encode_by_ref(buf)
            }
        }
        impl sqlx::Type<sqlx::Postgres> for TestSequenceItem {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("jsonb")
            }
        }
        impl<'r> sqlx::decode::Decode<'r, sqlx::Postgres> for TestSequenceItem {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let json: Json<TestSequenceItem> = sqlx::decode::Decode::decode(value)?;
                Ok(json.0)
            }
        }
    }
}
