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
        }
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
