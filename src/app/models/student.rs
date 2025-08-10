use chrono::NaiveDate;
use leptos::prelude::*;
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
pub enum ESLEnum {
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
impl fmt::Display for ESLEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ESLEnum::NotApplicable => "Not Applicable".to_string(),
                ESLEnum::Spanish => "Spanish".to_string(),
                ESLEnum::Arabic => "Arabic".to_string(),
                ESLEnum::Mandarin => "Mandarin".to_string(),
                ESLEnum::Cantonese => "Cantonese".to_string(),
                ESLEnum::Vietnamese => "Vietnamese".to_string(),
                ESLEnum::Nepali => "Nepali".to_string(),
                ESLEnum::French => "French".to_string(),
                ESLEnum::Russian => "Russian".to_string(),
                ESLEnum::Somali => "Somali".to_string(),
                ESLEnum::Amharic => "Amharic".to_string(),
                ESLEnum::Hindi => "Hindi".to_string(),
                ESLEnum::Telugu => "Telugu".to_string(),
                ESLEnum::Tamil => "Tamil".to_string(),
                ESLEnum::Other => "Other".to_string(),
            }
        )
    }
}
impl FromStr for ESLEnum {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Not Applicable" => Ok(ESLEnum::NotApplicable),
            "Spanish" => Ok(ESLEnum::Spanish),
            "Arabic" => Ok(ESLEnum::Arabic),
            "Mandarin" => Ok(ESLEnum::Mandarin),
            "Cantonese" => Ok(ESLEnum::Cantonese),
            "Vietnamese" => Ok(ESLEnum::Vietnamese),
            "Nepali" => Ok(ESLEnum::Nepali),
            "French" => Ok(ESLEnum::French),
            "Russian" => Ok(ESLEnum::Russian),
            "Somali" => Ok(ESLEnum::Somali),
            "Amharic" => Ok(ESLEnum::Amharic),
            "Hindi" => Ok(ESLEnum::Hindi),
            "Telugu" => Ok(ESLEnum::Telugu),
            "Tamil" => Ok(ESLEnum::Tamil),
            "Other" => Ok(ESLEnum::Other),
            _ => Err(format!("Invalid ESL value: {}", s)),
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
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, EnumIter)]
pub enum InterventionEnum {
    Literacy,
    Math,
    Both,
}
impl fmt::Display for InterventionEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                InterventionEnum::Literacy => "Literacy".to_string(),
                InterventionEnum::Math => "Math".to_string(),
                InterventionEnum::Both => "Literacy and Math".to_string(),
            }
        )
    }
}
impl FromStr for InterventionEnum {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Literacy" => Ok(InterventionEnum::Literacy),
            "Math" => Ok(InterventionEnum::Math),
            "Literacy and Math" => Ok(InterventionEnum::Both),
            _ => Err(format!("Invalid intervention enum value: {}", s)),
        }
    }
}
//this object instantiates a student on the client side for use in reading and writing data from
//the database
#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Student {
    pub firstname: Option<String>, //firstname, lastname, and pin are considered PII and need to
    //have Option<> wrapper so they can be nulled out
    pub lastname: Option<String>,
    pub preferred: String,
    pub gender: GenderEnum,
    pub date_of_birth: NaiveDate,
    #[validate(range(min = 0, max = 2000000000))]
    pub student_id: i32,
    pub esl: ESLEnum,
    pub current_grade_level: GradeEnum,
    pub teacher: String,
    pub iep: bool,
    pub bip: bool,
    pub student_504: bool,
    pub readplan: bool,
    pub gt: bool,
    pub intervention: Option<InterventionEnum>,
    pub eye_glasses: bool,
    pub notes: String,
    pub pin: Option<i32>,
}

impl Student {
    pub fn new(
        firstname: Option<String>,
        lastname: Option<String>,
        preferred: String,
        gender: GenderEnum,
        date_of_birth: NaiveDate,
        student_id: i32,
        esl: ESLEnum,
        current_grade_level: GradeEnum,
        teacher: String,
        iep: bool,
        bip: bool,
        student_504: bool,
        readplan: bool,
        gt: bool,
        intervention: Option<InterventionEnum>,
        eye_glasses: bool,
        notes: String,
        pin: Option<i32>,
    ) -> Student {
        Student {
            firstname,
            lastname,
            preferred,
            gender,
            date_of_birth,
            student_id,
            esl,
            current_grade_level,
            teacher,
            iep,
            bip,
            student_504,
            readplan,
            gt,
            intervention,
            eye_glasses,
            notes,
            pin,
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
    pub preferred: String,
    pub gender: GenderEnum,
    pub date_of_birth: NaiveDate,
    #[validate(range(min = 0, max = 2000000000))]
    pub student_id: i32,
    pub esl: ESLEnum,
    pub current_grade_level: GradeEnum,
    pub teacher: String,
    pub iep: bool,
    pub bip: bool,
    pub student_504: bool,
    pub readplan: bool,
    pub gt: bool,
    pub intervention: Option<InterventionEnum>,
    pub eye_glasses: bool,
    pub notes: String,
    pub pin: i32,
}

impl AddStudentRequest {
    pub fn new(
        firstname: String,
        lastname: String,
        preferred: String,
        gender: GenderEnum,
        date_of_birth: NaiveDate,
        student_id: i32,
        esl: ESLEnum,
        current_grade_level: GradeEnum,
        teacher: String,
        iep: bool,
        bip: bool,
        student_504: bool,
        readplan: bool,
        gt: bool,
        intervention: Option<InterventionEnum>,
        eye_glasses: bool,
        notes: String,
        pin: i32,
    ) -> AddStudentRequest {
        AddStudentRequest {
            firstname,
            lastname,
            preferred,
            gender,
            date_of_birth,
            student_id,
            esl,
            current_grade_level,
            teacher,
            iep,
            bip,
            student_504,
            readplan,
            gt,
            intervention,
            eye_glasses,
            notes,
            pin,
        }
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct UpdateStudentRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub firstname: String,
    #[validate(length(min = 1, message = "name is required"))]
    pub lastname: String,
    pub preferred: String,
    pub gender: GenderEnum,
    pub date_of_birth: NaiveDate,
    #[validate(range(min = 0, max = 2000000000))]
    pub student_id: i32,
    pub esl: ESLEnum,
    pub current_grade_level: GradeEnum,
    pub teacher: String,
    pub iep: bool,
    pub bip: bool,
    pub student_504: bool,
    pub readplan: bool,
    pub gt: bool,
    pub intervention: Option<InterventionEnum>,
    pub eye_glasses: bool,
    pub notes: String,
    pub pin: i32,
}

impl UpdateStudentRequest {
    pub fn new(
        firstname: String,
        lastname: String,
        preferred: String,
        gender: GenderEnum,
        date_of_birth: NaiveDate,
        student_id: i32,
        esl: ESLEnum,
        current_grade_level: GradeEnum,
        teacher: String,
        iep: bool,
        bip: bool,
        student_504: bool,
        readplan: bool,
        gt: bool,
        intervention: Option<InterventionEnum>,
        eye_glasses: bool,
        notes: String,
        pin: i32,
    ) -> UpdateStudentRequest {
        UpdateStudentRequest {
            firstname,
            lastname,
            preferred,
            gender,
            date_of_birth,
            student_id,
            esl,
            current_grade_level,
            teacher,
            iep,
            bip,
            student_504,
            readplan,
            gt,
            intervention,
            eye_glasses,
            notes,
            pin,
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
        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for ESLEnum {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                let s = self.to_string();
                <&str as Encode<Postgres>>::encode(&s.as_str(), buf)
            }
        }
        impl <'r> sqlx::decode::Decode<'r, sqlx::Postgres> for ESLEnum {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let s: &str = Decode::<sqlx::Postgres>::decode(value)?;
                ESLEnum::from_str(s).map_err(|_| format!("Invalid ESLEnum: {:?}", s).into())
            }
        }
        impl Type<Postgres> for ESLEnum {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("esl_enum")
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
        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for InterventionEnum {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                let s = self.to_string();
                <&str as Encode<Postgres>>::encode(&s.as_str(), buf)
            }
        }
        impl <'r> sqlx::decode::Decode<'r, sqlx::Postgres> for InterventionEnum {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let s: &str = Decode::<sqlx::Postgres>::decode(value)?;
                InterventionEnum::from_str(s).map_err(|_| format!("Invalid InterventionEnum: {:?}", s).into())
            }
        }
        impl Type<Postgres> for InterventionEnum {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("intervention_enum")
            }
        }
    }
}
