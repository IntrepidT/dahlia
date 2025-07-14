use crate::app::models::assessment_sequences::{SequenceBehavior, TestSequenceItem};
use crate::app::models::student::GradeEnum;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};
use std::str::FromStr;
use strum_macros::{EnumIter, EnumString};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, EnumIter)]
pub enum ScopeEnum {
    #[strum(to_string = "course")]
    Course,
    #[strum(to_string = "grade_level")]
    GradeLevel,
    #[strum(to_string = "all-required")]
    AllRequired,
}
impl fmt::Display for ScopeEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ScopeEnum::Course => "course".to_string(),
                ScopeEnum::GradeLevel => "grade_level".to_string(),
                ScopeEnum::AllRequired => "all-required".to_string(),
            }
        )
    }
}
impl FromStr for ScopeEnum {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "course" => Ok(ScopeEnum::Course),
            "grade_level" => Ok(ScopeEnum::GradeLevel),
            "all-required" => Ok(ScopeEnum::AllRequired),
            _ => Err(format!("Invalid scope value: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct RangeCategory {
    pub min: i32,
    pub max: i32,
    pub label: String,
}
impl RangeCategory {
    pub fn new(min: i32, max: i32, label: String) -> RangeCategory {
        RangeCategory { min, max, label }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, EnumIter)]
pub enum SubjectEnum {
    #[strum(to_string = "Reading")]
    Reading,
    #[strum(to_string = "Math")]
    Math,
    #[strum(to_string = "Literacy")]
    Literacy,
    #[strum(to_string = "Phonics")]
    Phonics,
    #[strum(to_string = "History")]
    History,
    #[strum(to_string = "Science")]
    Science,
    #[strum(to_string = "Social Studies")]
    SocialStudies,
    #[strum(to_string = "Other")]
    Other,
}
impl fmt::Display for SubjectEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SubjectEnum::Reading => "Reading".to_string(),
                SubjectEnum::Math => "Math".to_string(),
                SubjectEnum::Literacy => "Literacy".to_string(),
                SubjectEnum::Phonics => "Phonics".to_string(),
                SubjectEnum::History => "History".to_string(),
                SubjectEnum::Science => "Science".to_string(),
                SubjectEnum::SocialStudies => "Social Studies".to_string(),
                SubjectEnum::Other => "Other".to_string(),
            }
        )
    }
}
impl FromStr for SubjectEnum {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Reading" => Ok(SubjectEnum::Reading),
            "Math" => Ok(SubjectEnum::Math),
            "Literacy" => Ok(SubjectEnum::Literacy),
            "Phonics" => Ok(SubjectEnum::Phonics),
            "History" => Ok(SubjectEnum::History),
            "Science" => Ok(SubjectEnum::Science),
            "Social Studies" => Ok(SubjectEnum::SocialStudies),
            "Other" => Ok(SubjectEnum::Other),
            _ => Err(format!("Invalid subject value: {}", s)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Assessment {
    pub name: String,
    pub frequency: Option<i32>,
    pub grade: Option<GradeEnum>,
    pub version: i32,
    pub id: Uuid,
    pub tests: Vec<Uuid>,
    pub composite_score: Option<i32>,
    pub risk_benchmarks: Option<Vec<RangeCategory>>,
    pub national_benchmarks: Option<Vec<RangeCategory>>,
    pub subject: SubjectEnum,
    pub scope: Option<ScopeEnum>,
    pub course_id: Option<i32>,
    pub test_sequence: Option<Vec<TestSequenceItem>>,
}
impl Assessment {
    pub fn new(
        name: String,
        frequency: Option<i32>,
        grade: Option<GradeEnum>,
        version: i32,
        id: Uuid,
        tests: Vec<Uuid>,
        composite_score: Option<i32>,
        risk_benchmarks: Option<Vec<RangeCategory>>,
        national_benchmarks: Option<Vec<RangeCategory>>,
        subject: SubjectEnum,
        scope: Option<ScopeEnum>,
        course_id: Option<i32>,
    ) -> Assessment {
        Assessment {
            name,
            frequency,
            grade,
            version,
            id,
            tests,
            composite_score,
            risk_benchmarks,
            national_benchmarks,
            subject,
            scope,
            course_id,
            test_sequence: None,
        }
    }
    pub fn new_with_sequence(
        name: String,
        frequency: Option<i32>,
        grade: Option<GradeEnum>,
        version: i32,
        id: Uuid,
        composite_score: Option<i32>,
        risk_benchmarks: Option<Vec<RangeCategory>>,
        national_benchmarks: Option<Vec<RangeCategory>>,
        subject: SubjectEnum,
        scope: Option<ScopeEnum>,
        course_id: Option<i32>,
        test_sequence: Vec<TestSequenceItem>,
    ) -> Assessment {
        // Extract test IDs for backward compatibility
        let tests: Vec<Uuid> = test_sequence.iter().map(|item| item.test_id).collect();

        Assessment {
            name,
            frequency,
            grade,
            version,
            id,
            tests,
            composite_score,
            risk_benchmarks,
            national_benchmarks,
            subject,
            scope,
            course_id,
            test_sequence: Some(test_sequence),
        }
    }
    //helper function to get the first test in the sequence
    pub fn get_first_test(&self) -> Option<Uuid> {
        self.test_sequence
            .as_ref()?
            .iter()
            .min_by_key(|item| item.sequence_order)
            .map(|item| item.test_id)
    }
    //helper function get the next test based on the current test and score
    pub fn get_next_test(&self, current_test_id: Uuid, score: Option<i32>) -> Option<Uuid> {
        let sequence = self.test_sequence.as_ref()?;
        let current_item = sequence
            .iter()
            .find(|item| item.test_id == current_test_id)?;

        match current_item.sequence_behavior {
            SequenceBehavior::Node | SequenceBehavior::Diagnostic => {
                // For nodes and diagnostics, just get the next test in sequence order
                self.get_next_in_sequence(current_item.sequence_order)
            }
            SequenceBehavior::Optional => {
                // Optional tests can be skipped, continue to next
                self.get_next_in_sequence(current_item.sequence_order)
            }
            SequenceBehavior::Attainment => {
                let score = score?;
                let required_score = current_item.required_score?;

                if score >= required_score {
                    // Passed: use next_on_pass or next in sequence
                    current_item
                        .next_on_pass
                        .or_else(|| self.get_next_in_sequence(current_item.sequence_order))
                } else {
                    // Failed: use next_on_fail or end sequence
                    current_item.next_on_fail
                }
            }
            SequenceBehavior::Remediation => {
                // After remediation, typically return to main sequence
                self.get_next_in_sequence(current_item.sequence_order)
            }
            SequenceBehavior::Branching => {
                let score = score?;
                if let Some(ranges) = &current_item.score_ranges {
                    // Find the appropriate score range
                    for range in ranges {
                        if score >= range.min && score <= range.max {
                            return range.next_test;
                        }
                    }
                }
                // If no range matches, continue to next in sequence
                self.get_next_in_sequence(current_item.sequence_order)
            }
        }
    }
    //helper function to get the next test in sequence order
    pub fn get_next_in_sequence(&self, current_order: i32) -> Option<Uuid> {
        self.test_sequence
            .as_ref()?
            .iter()
            .filter(|item| item.sequence_order > current_order)
            .min_by_key(|item| item.sequence_order)
            .map(|item| item.test_id)
    }
    //helper function to check if the test should be shown based on the prerequisites
    pub fn should_show_test(&self, test_id: Uuid, completed_tests: &[Uuid]) -> bool {
        let sequence = match self.test_sequence.as_ref() {
            Some(seq) => seq,
            None => return true,
        };

        let test_item = match sequence.iter().find(|item| item.test_id == test_id) {
            Some(item) => item,
            None => return true,
        };

        // Check prerequisites
        if let Some(prerequisites) = &test_item.prerequisite_tests {
            if !prerequisites
                .iter()
                .all(|req| completed_tests.contains(req))
            {
                return false;
            }
        }

        // Check skip conditions
        if let Some(skip_conditions) = &test_item.skip_conditions {
            if skip_conditions
                .iter()
                .any(|skip| completed_tests.contains(skip))
            {
                return false;
            }
        }

        // Special logic for remediation tests
        if test_item.sequence_behavior == SequenceBehavior::Remediation {
            // Only show if prerequisite tests were failed
            if let Some(prerequisites) = &test_item.prerequisite_tests {
                // This would need additional logic to check if prerequisites were failed
                // rather than just completed
                return true; // Placeholder - implement based on your scoring system
            }
        }

        true
    }
    //helper function to get available attempts remaining
    pub fn get_attempts_remaining(&self, test_id: Uuid, attempts_used: &[Uuid]) -> Option<i32> {
        let sequence = self.test_sequence.as_ref()?;
        let test_item = sequence.iter().find(|item| item.test_id == test_id)?;

        test_item
            .max_attempts
            .map(|max| (max - attempts_used.len() as i32).max(0))
    }
    //helper function to check if assessment uses sequences
    pub fn uses_sequences(&self) -> bool {
        self.test_sequence.is_some() && !self.test_sequence.as_ref().unwrap().is_empty()
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, Clone)]
pub struct CreateNewAssessmentRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    pub frequency: Option<i32>,
    pub grade: Option<GradeEnum>,
    pub version: i32,
    pub tests: Vec<Uuid>,
    pub composite_score: Option<i32>,
    pub risk_benchmarks: Option<Vec<RangeCategory>>,
    pub national_benchmarks: Option<Vec<RangeCategory>>,
    pub subject: SubjectEnum,
    pub scope: Option<ScopeEnum>,
    pub course_id: Option<i32>,
    pub test_sequence: Option<Vec<TestSequenceItem>>,
}
impl CreateNewAssessmentRequest {
    pub fn new(
        name: String,
        frequency: Option<i32>,
        grade: Option<GradeEnum>,
        version: i32,
        tests: Vec<Uuid>,
        composite_score: Option<i32>,
        risk_benchmarks: Option<Vec<RangeCategory>>,
        national_benchmarks: Option<Vec<RangeCategory>>,
        subject: SubjectEnum,
        scope: Option<ScopeEnum>,
        course_id: Option<i32>,
    ) -> CreateNewAssessmentRequest {
        CreateNewAssessmentRequest {
            name,
            frequency,
            grade,
            version,
            tests,
            composite_score,
            risk_benchmarks,
            national_benchmarks,
            subject,
            scope,
            course_id,
            test_sequence: None,
        }
    }
    pub fn new_with_sequence(
        name: String,
        frequency: Option<i32>,
        grade: Option<GradeEnum>,
        version: i32,
        composite_score: Option<i32>,
        risk_benchmarks: Option<Vec<RangeCategory>>,
        national_benchmarks: Option<Vec<RangeCategory>>,
        subject: SubjectEnum,
        scope: Option<ScopeEnum>,
        course_id: Option<i32>,
        test_sequence: Vec<TestSequenceItem>,
    ) -> CreateNewAssessmentRequest {
        let tests: Vec<Uuid> = test_sequence.iter().map(|item| item.test_id).collect();

        CreateNewAssessmentRequest {
            name,
            frequency,
            grade,
            version,
            tests,
            composite_score,
            risk_benchmarks,
            national_benchmarks,
            subject,
            scope,
            course_id,
            test_sequence: Some(test_sequence),
        }
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, Clone)]
pub struct UpdateAssessmentRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    pub frequency: Option<i32>,
    pub grade: Option<GradeEnum>,
    pub version: i32,
    pub id: Uuid,
    pub tests: Vec<Uuid>,
    pub composite_score: Option<i32>,
    pub risk_benchmarks: Option<Vec<RangeCategory>>,
    pub national_benchmarks: Option<Vec<RangeCategory>>,
    pub subject: SubjectEnum,
    pub scope: Option<ScopeEnum>,
    pub course_id: Option<i32>,
    pub test_sequence: Option<Vec<TestSequenceItem>>,
}
impl UpdateAssessmentRequest {
    pub fn new(
        name: String,
        frequency: Option<i32>,
        grade: Option<GradeEnum>,
        version: i32,
        id: Uuid,
        tests: Vec<Uuid>,
        composite_score: Option<i32>,
        risk_benchmarks: Option<Vec<RangeCategory>>,
        national_benchmarks: Option<Vec<RangeCategory>>,
        subject: SubjectEnum,
        scope: Option<ScopeEnum>,
        course_id: Option<i32>,
    ) -> UpdateAssessmentRequest {
        UpdateAssessmentRequest {
            name,
            frequency,
            grade,
            version,
            id,
            tests,
            composite_score,
            risk_benchmarks,
            national_benchmarks,
            subject,
            scope,
            course_id,
            test_sequence: None,
        }
    }
    pub fn new_with_sequence(
        name: String,
        frequency: Option<i32>,
        grade: Option<GradeEnum>,
        version: i32,
        id: Uuid,
        composite_score: Option<i32>,
        risk_benchmarks: Option<Vec<RangeCategory>>,
        national_benchmarks: Option<Vec<RangeCategory>>,
        subject: SubjectEnum,
        scope: Option<ScopeEnum>,
        course_id: Option<i32>,
        test_sequence: Vec<TestSequenceItem>,
    ) -> UpdateAssessmentRequest {
        let tests: Vec<Uuid> = test_sequence.iter().map(|item| item.test_id).collect();

        UpdateAssessmentRequest {
            name,
            frequency,
            grade,
            version,
            id,
            tests,
            composite_score,
            risk_benchmarks,
            national_benchmarks,
            subject,
            scope,
            course_id,
            test_sequence: Some(test_sequence),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeleteAssessmentRequest {
    pub version: i32,
    pub id: Uuid,
}
impl DeleteAssessmentRequest {
    pub fn new(version: i32, id: Uuid) -> DeleteAssessmentRequest {
        DeleteAssessmentRequest { version, id }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use sqlx::{Postgres, Encode, Decode, Type, postgres::{PgTypeInfo, PgValueRef, PgArgumentBuffer, PgHasArrayType}, encode::IsNull};
        use sqlx::prelude::*;
        use sqlx::types::Json;

        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for SubjectEnum {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                let s = self.to_string();
                <&str as Encode<Postgres>>::encode(&s.as_str(), buf)
            }
        }
        impl<'r> sqlx::decode::Decode<'r, sqlx::Postgres> for SubjectEnum {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let s: &str = Decode::<sqlx::Postgres>::decode(value)?;
                SubjectEnum::from_str(s).map_err(|_| format!("Invalid SubjectEnum: {}", s).into())
            }
        }
        impl Type<Postgres> for SubjectEnum {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("subject_enum")
            }
        }

        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for RangeCategory {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                Json(self).encode_by_ref(buf)
            }
        }
        impl sqlx::Type<sqlx::Postgres> for RangeCategory {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("jsonb")
            }
        }
        impl<'r> sqlx::decode::Decode<'r, sqlx::Postgres> for RangeCategory {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let json: Json<RangeCategory> = sqlx::decode::Decode::decode(value)?;
                Ok(json.0)
            }
        }
        impl sqlx::postgres::PgHasArrayType for RangeCategory {
            fn array_type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("_jsonb")
            }
        }
        impl <'q> sqlx::encode::Encode<'q, sqlx::Postgres> for ScopeEnum {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                let s = self.to_string();
                <&str as Encode<Postgres>>::encode(&s.as_str(), buf)
            }
        }
        impl<'r> sqlx::decode::Decode<'r, sqlx::Postgres> for ScopeEnum {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let s: &str = Decode::<sqlx::Postgres>::decode(value)?;
                ScopeEnum::from_str(s).map_err(|_| format!("Invalid ScopeEnum: {}", s).into())
            }
        }
        impl sqlx::Type<sqlx::Postgres> for ScopeEnum {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("assessment_scope_enum")
            }
        }

        ///Create wrapper for new type Vec<RangeCategory> to solve orphan rule issue in rust
        #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
        pub struct RangeCategoriesWrapper(pub Vec<RangeCategory>);

        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for RangeCategoriesWrapper {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                Json(&self.0).encode_by_ref(buf)
            }
        }

        impl<'r> sqlx::decode::Decode<'r, sqlx::Postgres> for RangeCategoriesWrapper {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let json: Json<Vec<RangeCategory>> = sqlx::decode::Decode::decode(value)?;
                Ok(RangeCategoriesWrapper(json.0))
            }
        }
    }
}
