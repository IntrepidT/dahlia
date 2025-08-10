use crate::app::models::student::GradeEnum;
use chrono::NaiveDate;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display};
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, EnumIter)]
pub enum EnrollmentStatus {
    Active,
    Inactive,
    Graduated,
    Transferred,
    Dropped,
}
impl FromStr for EnrollmentStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(EnrollmentStatus::Active),
            "inactive" => Ok(EnrollmentStatus::Inactive),
            "graduated" => Ok(EnrollmentStatus::Graduated),
            "transferred" => Ok(EnrollmentStatus::Transferred),
            "dropped" => Ok(EnrollmentStatus::Dropped),
            _ => Err(format!("Invalid enrollment status: {}", s)),
        }
    }
}
impl fmt::Display for EnrollmentStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EnrollmentStatus::Active => "active".to_string(),
                EnrollmentStatus::Inactive => "inactive".to_string(),
                EnrollmentStatus::Graduated => "graduated".to_string(),
                EnrollmentStatus::Transferred => "transferred".to_string(),
                EnrollmentStatus::Dropped => "dropped".to_string(),
            }
        )
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, EnumIter, Hash)]
pub enum AcademicYear {
    Year2023_2024,
    Year2024_2025,
    Year2025_2026,
    Year2026_2027,
    Year2027_2028,
    Year2028_2029,
    Year2029_2030,
}
impl FromStr for AcademicYear {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "2023-2024" => Ok(AcademicYear::Year2023_2024),
            "2024-2025" => Ok(AcademicYear::Year2024_2025),
            "2025-2026" => Ok(AcademicYear::Year2025_2026),
            "2026-2027" => Ok(AcademicYear::Year2026_2027),
            "2027-2028" => Ok(AcademicYear::Year2027_2028),
            "2028-2029" => Ok(AcademicYear::Year2028_2029),
            "2029-2030" => Ok(AcademicYear::Year2029_2030),
            _ => Err(format!("Invalid academic year: {}", s)),
        }
    }
}
impl Display for AcademicYear {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AcademicYear::Year2023_2024 => "2023-2024",
                AcademicYear::Year2024_2025 => "2024-2025",
                AcademicYear::Year2025_2026 => "2025-2026",
                AcademicYear::Year2026_2027 => "2026-2027",
                AcademicYear::Year2027_2028 => "2027-2028",
                AcademicYear::Year2028_2029 => "2028-2029",
                AcademicYear::Year2029_2030 => "2029-2030",
            }
        )
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Enrollment {
    pub student_id: i32,
    pub academic_year: AcademicYear,
    pub grade_level: GradeEnum,
    pub teacher_id: i32,
    pub status: EnrollmentStatus,
    pub enrollment_date: NaiveDate,
    pub status_change_date: Option<NaiveDate>, // Changed to Option<NaiveDate> to match database
    pub notes: Option<String>,
}

impl Enrollment {
    pub fn new(
        student_id: i32,
        academic_year: AcademicYear,
        grade_level: GradeEnum,
        teacher_id: i32,
        status: EnrollmentStatus,
        enrollment_date: NaiveDate,
        status_change_date: Option<NaiveDate>, // Changed to Option<NaiveDate>
        notes: Option<String>,
    ) -> Self {
        Enrollment {
            student_id,
            academic_year,
            grade_level,
            teacher_id,
            status,
            enrollment_date,
            status_change_date,
            notes,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct CreateEnrollmentRequest {
    pub student_id: i32,
    pub course_id: i32,
    pub academic_year: AcademicYear,
    pub grade_level: GradeEnum,
    pub teacher_id: i32,
    pub status: EnrollmentStatus,
    pub enrollment_date: NaiveDate,
    pub status_change_date: Option<NaiveDate>, // Changed to Option<NaiveDate>
    pub notes: Option<String>,
}
impl CreateEnrollmentRequest {
    pub fn new(
        student_id: i32,
        course_id: i32,
        academic_year: AcademicYear,
        grade_level: GradeEnum,
        teacher_id: i32,
        status: EnrollmentStatus,
        enrollment_date: NaiveDate,
        status_change_date: Option<NaiveDate>, // Changed to Option<NaiveDate>
        notes: Option<String>,
    ) -> Self {
        CreateEnrollmentRequest {
            student_id,
            course_id,
            academic_year,
            grade_level,
            teacher_id,
            status,
            enrollment_date,
            status_change_date,
            notes,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct UpdateEnrollmentRequest {
    pub student_id: i32,
    pub course_id: i32,
    pub academic_year: AcademicYear,
    pub grade_level: GradeEnum,
    pub teacher_id: i32,
    pub status: EnrollmentStatus,
    pub enrollment_date: NaiveDate,
    pub status_change_date: Option<NaiveDate>, // Changed to Option<NaiveDate>
    pub notes: Option<String>,
}
impl UpdateEnrollmentRequest {
    pub fn new(
        student_id: i32,
        course_id: i32,
        academic_year: AcademicYear,
        grade_level: GradeEnum,
        teacher_id: i32,
        status: EnrollmentStatus,
        enrollment_date: NaiveDate,
        status_change_date: Option<NaiveDate>, // Changed to Option<NaiveDate>
        notes: Option<String>,
    ) -> Self {
        UpdateEnrollmentRequest {
            student_id,
            course_id,
            academic_year,
            grade_level,
            teacher_id,
            status,
            enrollment_date,
            status_change_date,
            notes,
        }
    }
}

//the following functions are all gated behind ssr but allow encoding and decoding of
//EnrollmentStatus and AcademicYear by sqlx
cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use sqlx::{Postgres, Encode, Decode, Type, postgres::{PgTypeInfo, PgValueRef, PgArgumentBuffer}, encode::IsNull};
        use sqlx::prelude::*;

        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for EnrollmentStatus {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                let s = self.to_string();
                <&str as Encode<Postgres>>::encode(&s.as_str(), buf)
            }
        }
        impl <'r> sqlx::decode::Decode<'r, Postgres> for EnrollmentStatus {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let s: &str = Decode::<sqlx::Postgres>::decode(value)?;
                EnrollmentStatus::from_str(s).map_err(|_| format!("Invalid enrollment status: {:?}", s).into())
            }
        }
        impl Type<Postgres> for EnrollmentStatus {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("enrollment_status_enum")
            }
        }

        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for AcademicYear {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                let s = self.to_string();
                <&str as Encode<Postgres>>::encode(&s.as_str(), buf)
            }
        }
        impl <'r> sqlx::decode::Decode<'r, Postgres> for AcademicYear {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let s: &str = Decode::<sqlx::Postgres>::decode(value)?;
                AcademicYear::from_str(s).map_err(|_| format!("Invalid academic year: {:?}", s).into())
            }
        }
        impl Type<Postgres> for AcademicYear {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("school_year_enum")
            }
        }
    }
}
