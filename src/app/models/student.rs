use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display};
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use validator::Validate;

//this section instantiates all the enums and their corresponding formatting
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, EnumIter)]
pub enum GenderEnum {
    Male,
    Female,
    Nonbinary,
}
impl fmt::Display for GenderEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GenderEnum::Male => "Male".to_string(),
                GenderEnum::Female => "Female".to_string(),
                GenderEnum::Nonbinary => "Non-binary".to_string(),
            }
        )
    }
}
impl FromStr for GenderEnum {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Male" => Ok(GenderEnum::Male),
            "Female" => Ok(GenderEnum::Female),
            "Non-binary" => Ok(GenderEnum::Nonbinary),
            _ => Err(format!("Invalid gender value: {}", s)),
        }
    }
}
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, EnumIter)]
pub enum ELLEnum {
    NotApplicable,
    Spanish,
    Arabic,
    Mandarin,
    Cantonese,
    Vietnamese,
    Nepali,
    French,
    Russian,
    Somali,
    Amharic,
    Hindi,
    Telugu,
    Tamil,
    Other,
}
impl fmt::Display for ELLEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ELLEnum::NotApplicable => "Not Applicable".to_string(),
                ELLEnum::Spanish => "Spanish".to_string(),
                ELLEnum::Arabic => "Arabic".to_string(),
                ELLEnum::Mandarin => "Mandarin".to_string(),
                ELLEnum::Cantonese => "Cantonese".to_string(),
                ELLEnum::Vietnamese => "Vietnamese".to_string(),
                ELLEnum::Nepali => "Nepali".to_string(),
                ELLEnum::French => "French".to_string(),
                ELLEnum::Russian => "Russian".to_string(),
                ELLEnum::Somali => "Somali".to_string(),
                ELLEnum::Amharic => "Amharic".to_string(),
                ELLEnum::Hindi => "Hindi".to_string(),
                ELLEnum::Telugu => "Telugu".to_string(),
                ELLEnum::Tamil => "Tamil".to_string(),
                ELLEnum::Other => "Other".to_string(),
            }
        )
    }
}
impl FromStr for ELLEnum {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Not Applicable" => Ok(ELLEnum::NotApplicable),
            "Spanish" => Ok(ELLEnum::Spanish),
            "Arabic" => Ok(ELLEnum::Arabic),
            "Mandarin" => Ok(ELLEnum::Mandarin),
            "Cantonese" => Ok(ELLEnum::Cantonese),
            "Vietnamese" => Ok(ELLEnum::Vietnamese),
            "Nepali" => Ok(ELLEnum::Nepali),
            "French" => Ok(ELLEnum::French),
            "Russian" => Ok(ELLEnum::Russian),
            "Somali" => Ok(ELLEnum::Somali),
            "Amharic" => Ok(ELLEnum::Amharic),
            "Hindi" => Ok(ELLEnum::Hindi),
            "Telugu" => Ok(ELLEnum::Telugu),
            "Tamil" => Ok(ELLEnum::Tamil),
            "Other" => Ok(ELLEnum::Other),
            _ => Err(format!("Invalid ELL value: {}", s)),
        }
    }
}
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, EnumIter)]
pub enum GradeEnum {
    Kindergarten,
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
    Ninth,
    Tenth,
    Eleventh,
    Twelfth,
}
impl fmt::Display for GradeEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GradeEnum::Kindergarten => "Kindergarten".to_string(),
                GradeEnum::First => "1st Grade".to_string(),
                GradeEnum::Second => "2nd Grade".to_string(),
                GradeEnum::Third => "3rd Grade".to_string(),
                GradeEnum::Fourth => "4th Grade".to_string(),
                GradeEnum::Fifth => "5th Grade".to_string(),
                GradeEnum::Sixth => "6th Grade".to_string(),
                GradeEnum::Seventh => "7th Grade".to_string(),
                GradeEnum::Eighth => "8th Grade".to_string(),
                GradeEnum::Ninth => "9th Grade".to_string(),
                GradeEnum::Tenth => "10th Grade".to_string(),
                GradeEnum::Eleventh => "11th Grade".to_string(),
                GradeEnum::Twelfth => "12th Grade".to_string(),
            }
        )
    }
}
impl FromStr for GradeEnum {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Kindergarten" => Ok(GradeEnum::Kindergarten),
            "1st Grade" => Ok(GradeEnum::First),
            "2nd Grade" => Ok(GradeEnum::Second),
            "3rd Grade" => Ok(GradeEnum::Third),
            "4th Grade" => Ok(GradeEnum::Fourth),
            "5th Grade" => Ok(GradeEnum::Fifth),
            "6th Grade" => Ok(GradeEnum::Sixth),
            "7th Grade" => Ok(GradeEnum::Seventh),
            "8th Grade" => Ok(GradeEnum::Eighth),
            "9th Grade" => Ok(GradeEnum::Ninth),
            "10th Grade" => Ok(GradeEnum::Tenth),
            "11th Grade" => Ok(GradeEnum::Eleventh),
            "12th Grade" => Ok(GradeEnum::Twelfth),
            _ => Err(format!("Invalid grade value: {}", s)),
        }
    }
}
//this object instantiates a student on the client side for use in reading and writing data from
//the database
#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Student {
    #[validate(length(min = 1, message = "name is required"))]
    pub firstname: String,
    #[validate(length(min = 1, message = "name is required"))]
    pub lastname: String,
    pub gender: GenderEnum,
    pub date_of_birth: NaiveDate,
    #[validate(range(min = 0, max = 2000000000))]
    pub student_id: i32,
    pub ell: ELLEnum,
    pub grade: GradeEnum,
    pub teacher: String,
    pub iep: bool,
    pub student_504: bool,
    pub readplan: bool,
    pub gt: bool,
    pub intervention: bool,
    pub eye_glasses: bool,
}

impl Student {
    pub fn new(
        firstname: String,
        lastname: String,
        gender: GenderEnum,
        date_of_birth: NaiveDate,
        student_id: i32,
        ell: ELLEnum,
        grade: GradeEnum,
        teacher: String,
        iep: bool,
        student_504: bool,
        readplan: bool,
        gt: bool,
        intervention: bool,
        eye_glasses: bool,
    ) -> Student {
        Student {
            firstname,
            lastname,
            gender,
            date_of_birth,
            student_id,
            ell,
            grade,
            teacher,
            iep,
            student_504,
            readplan,
            gt,
            intervention,
            eye_glasses,
        }
    }
}

//this function is the client-side initalization Writing a new Student into the database
#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct AddStudentRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub firstname: String,
    #[validate(length(min = 1, message = "name is required"))]
    pub lastname: String,
    pub gender: GenderEnum,
    pub date_of_birth: NaiveDate,
    #[validate(range(min = 0, max = 2000000000))]
    pub student_id: i32,
    pub ell: ELLEnum,
    pub grade: GradeEnum,
    pub teacher: String,
    pub iep: bool,
    pub student_504: bool,
    pub readplan: bool,
    pub gt: bool,
    pub intervention: bool,
    pub eye_glasses: bool,
}

impl AddStudentRequest {
    pub fn new(
        firstname: String,
        lastname: String,
        gender: GenderEnum,
        date_of_birth: NaiveDate,
        student_id: i32,
        ell: ELLEnum,
        grade: GradeEnum,
        teacher: String,
        iep: bool,
        student_504: bool,
        readplan: bool,
        gt: bool,
        intervention: bool,
        eye_glasses: bool,
    ) -> AddStudentRequest {
        AddStudentRequest {
            firstname,
            lastname,
            gender,
            date_of_birth,
            student_id,
            ell,
            grade,
            teacher,
            iep,
            student_504,
            readplan,
            gt,
            intervention,
            eye_glasses,
        }
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct UpdateStudentRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub firstname: String,
    #[validate(length(min = 1, message = "name is required"))]
    pub lastname: String,
    pub gender: GenderEnum,
    pub date_of_birth: NaiveDate,
    #[validate(range(min = 0, max = 2000000000))]
    pub student_id: i32,
    pub ell: ELLEnum,
    pub grade: GradeEnum,
    pub teacher: String,
    pub iep: bool,
    pub student_504: bool,
    pub readplan: bool,
    pub gt: bool,
    pub intervention: bool,
    pub eye_glasses: bool,
}

impl UpdateStudentRequest {
    pub fn new(
        firstname: String,
        lastname: String,
        gender: GenderEnum,
        date_of_birth: NaiveDate,
        student_id: i32,
        ell: ELLEnum,
        grade: GradeEnum,
        teacher: String,
        iep: bool,
        student_504: bool,
        readplan: bool,
        gt: bool,
        intervention: bool,
        eye_glasses: bool,
    ) -> UpdateStudentRequest {
        UpdateStudentRequest {
            firstname,
            lastname,
            gender,
            date_of_birth,
            student_id,
            ell,
            grade,
            teacher,
            iep,
            student_504,
            readplan,
            gt,
            intervention,
            eye_glasses,
        }
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct DeleteStudentRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub firstname: String,
    #[validate(length(min = 1, message = "name is required"))]
    pub lastname: String,
    pub student_id: i32,
}

impl DeleteStudentRequest {
    pub fn new(firstname: String, lastname: String, student_id: i32) -> DeleteStudentRequest {
        DeleteStudentRequest {
            firstname,
            lastname,
            student_id,
        }
    }
}
//the following functions are all gated behind the ssr feature but they allow for the encoding and decoding of the different enums inside of student
cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use sqlx::{Postgres, Encode, Decode, Type, postgres::{PgTypeInfo, PgValueRef, PgArgumentBuffer}, encode::IsNull};
        use sqlx::prelude::*;

        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for GenderEnum {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                let s = self.to_string();
                <&str as Encode<Postgres>>::encode(&s.as_str(), buf)
            }
        }
        impl <'r> sqlx::decode::Decode<'r, sqlx::Postgres> for GenderEnum {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let s: &str = Decode::<sqlx::Postgres>::decode(value)?;
                GenderEnum::from_str(s).map_err(|_| format!("Invalid GenderEnum: {:?}", s).into())
            }
        }
        impl Type<Postgres> for GenderEnum {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("gender_enum")
            }
        }
        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for ELLEnum {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                let s = self.to_string();
                <&str as Encode<Postgres>>::encode(&s.as_str(), buf)
            }
        }
        impl <'r> sqlx::decode::Decode<'r, sqlx::Postgres> for ELLEnum {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let s: &str = Decode::<sqlx::Postgres>::decode(value)?;
                ELLEnum::from_str(s).map_err(|_| format!("Invalid ELLEnum: {:?}", s).into())
            }
        }
        impl Type<Postgres> for ELLEnum {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("ell_enum")
            }
        }
        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for GradeEnum {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                let s = self.to_string();
                <&str as Encode<Postgres>>::encode(&s.as_str(), buf)
            }
        }
        impl <'r> sqlx::decode::Decode<'r, sqlx::Postgres> for GradeEnum {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let s: &str = Decode::<sqlx::Postgres>::decode(value)?;
                GradeEnum::from_str(s).map_err(|_| format!("Invalid GradeEnum: {:?}", s).into())
            }
        }
        impl Type<Postgres> for GradeEnum {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("grade_enum")
            }
        }
    }
}
