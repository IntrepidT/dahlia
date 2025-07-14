use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use strum_macros::EnumIter;
use uuid::Uuid;
//This file primarily defines data structures to be called and used within the assessment structs.
//Theses data structures allow for complex sequencing and branching logic in assessments.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, EnumIter)]
pub enum SequenceBehavior {
    #[strum(to_string = "attainment")]
    Attainment,
    #[strum(to_string = "node")]
    Node,
    #[strum(to_string = "optional")]
    Optional, // Can be skipped, doesn't affect progression
    #[strum(to_string = "diagnostic")]
    Diagnostic, // For assessment only, doesn't block progression
    #[strum(to_string = "remediation")]
    Remediation, // Only shown if previous test failed
    #[strum(to_string = "branching")]
    Branching, // Multiple paths based on score ranges
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
            "attainment" => Ok(SequenceBehavior::Attainment),
            "node" => Ok(SequenceBehavior::Node),
            "optional" => Ok(SequenceBehavior::Optional),
            "diagnostic" => Ok(SequenceBehavior::Diagnostic),
            "remediation" => Ok(SequenceBehavior::Remediation),
            "branching" => Ok(SequenceBehavior::Branching),
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TestSequenceItem {
    pub test_id: Uuid,
    pub sequence_behavior: SequenceBehavior,
    pub sequence_order: i32,
    pub required_score: Option<i32>, // Only used for attainment tests
    pub next_on_pass: Option<Uuid>,  // Next test if this test is passed
    pub next_on_fail: Option<Uuid>,  // Next test if this test is failed

    // Advanced sequencing options
    pub score_ranges: Option<Vec<ScoreRange>>, // For branching behavior
    pub max_attempts: Option<i32>,             // How many times can this test be retaken
    pub time_limit_minutes: Option<i32>,       // Time limit for this test
    pub prerequisite_tests: Option<Vec<Uuid>>, // Tests that must be completed first
    pub skip_conditions: Option<Vec<Uuid>>,    // If these tests are passed, skip this one
    pub show_feedback: bool,                   // Whether to show immediate feedback
    pub allow_review: bool,                    // Whether student can review answers
    pub randomize_questions: bool,             // Whether to randomize question order
    pub adaptive_difficulty: bool,             // Whether test adapts based on performance
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
            score_ranges: None,
            max_attempts: Some(3), // Allow retakes for attainment tests
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
            score_ranges: None,
            max_attempts: Some(1),
            time_limit_minutes: None,
            prerequisite_tests: None,
            skip_conditions: None,
            show_feedback: false, // Often diagnostic tests don't show immediate feedback
            allow_review: false,
            randomize_questions: true, // Good for diagnostic assessments
            adaptive_difficulty: true, // Diagnostics often adapt
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
            score_ranges: None,
            max_attempts: Some(5), // More attempts for remediation
            time_limit_minutes: None,
            prerequisite_tests: Some(prerequisite_tests),
            skip_conditions: None,
            show_feedback: true,
            allow_review: true,
            randomize_questions: false,
            adaptive_difficulty: false,
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use sqlx::{Postgres, Encode, Decode, Type, postgres::{PgTypeInfo, PgValueRef, PgArgumentBuffer}, encode::IsNull};
        use sqlx::types::Json;

        // Add SequenceBehavior SQL support
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

        // Add TestSequenceItem SQL support
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
