use crate::app::models::student::GradeEnum;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};
use std::str::FromStr;
use strum_macros::EnumIter;
use validator::Validate;

//Defining the employee roles here
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, EnumIter)]
pub enum EmployeeRole {
    Teacher { grade: Option<GradeEnum> },
    AssistantPrincipal,
    Principal,
    Interventionist,
    IntegratedServices,
    Speech,
    OT,
    Psychologist,
    ParaProf,
    AssessmentCoordinator,
    Other,
}

impl fmt::Display for EmployeeRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EmployeeRole::Teacher { grade: _ } => "Teacher".to_string(),
                EmployeeRole::AssistantPrincipal => "Assistant Principal".to_string(),
                EmployeeRole::Principal => "Principal".to_string(),
                EmployeeRole::Interventionist => "Interventionist".to_string(),
                EmployeeRole::IntegratedServices => "Integrated Services".to_string(),
                EmployeeRole::Speech => "Speech".to_string(),
                EmployeeRole::OT => "O/T".to_string(),
                EmployeeRole::Psychologist => "Psychologist".to_string(),
                EmployeeRole::ParaProf => "Para-Professional".to_string(),
                EmployeeRole::AssessmentCoordinator => "Assessment Coordinator".to_string(),
                EmployeeRole::Other => "Other".to_string(),
            }
        )
    }
}

impl FromStr for EmployeeRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Teacher" => Ok(EmployeeRole::Teacher { grade: None }),
            "Assistant Principal" => Ok(EmployeeRole::AssistantPrincipal),
            "Principal" => Ok(EmployeeRole::Principal),
            "Interventionist" => Ok(EmployeeRole::Interventionist),
            "Integrated Services" => Ok(EmployeeRole::IntegratedServices),
            "Speech" => Ok(EmployeeRole::Speech),
            "O/T" => Ok(EmployeeRole::OT),
            "Psychologist" => Ok(EmployeeRole::Psychologist),
            "Para-Professional" => Ok(EmployeeRole::ParaProf),
            "Assessment Coordinator" => Ok(EmployeeRole::AssessmentCoordinator),
            "Other" => Ok(EmployeeRole::Other),
            _ => Err(format!("Invalid status value: {}", s)),
        }
    }
}

//Defining the Status for an Employee here
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, EnumIter)]
pub enum StatusEnum {
    Active,
    OnLeave,
    PartTime,
    NotApplicable,
}

impl fmt::Display for StatusEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                StatusEnum::Active => "Active".to_string(),
                StatusEnum::OnLeave => "On Leave".to_string(),
                StatusEnum::PartTime => "Part-time".to_string(),
                StatusEnum::NotApplicable => "Not Applicable".to_string(),
            }
        )
    }
}

impl FromStr for StatusEnum {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Active" => Ok(StatusEnum::Active),
            "On Leave" => Ok(StatusEnum::OnLeave),
            "Part-time" => Ok(StatusEnum::PartTime),
            "Not Applicable" => Ok(StatusEnum::NotApplicable),
            _ => Err(format!("Invalid status value: {}", s)),
        }
    }
}

//defining the employee here
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Employee {
    pub id: i32,
    pub firstname: String,
    pub lastname: String,
    pub status: StatusEnum,
    pub role: EmployeeRole,
}

impl Employee {
    //generic constructor for all roles
    pub fn new(
        id: i32,
        firstname: String,
        lastname: String,
        status: StatusEnum,
        role: EmployeeRole,
    ) -> Self {
        Employee {
            id,
            firstname,
            lastname,
            status,
            role,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, PartialEq, Eq, Clone)]
pub struct AddNewEmployeeRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub firstname: String,
    #[validate(length(min = 1, message = "lastname is required"))]
    pub lastname: String,
    pub status: StatusEnum,
    pub role: EmployeeRole,
    pub grade: Option<GradeEnum>,
}

impl AddNewEmployeeRequest {
    pub fn new(
        firstname: String,
        lastname: String,
        status: StatusEnum,
        role: EmployeeRole,
        grade: Option<GradeEnum>,
    ) -> AddNewEmployeeRequest {
        AddNewEmployeeRequest {
            firstname,
            lastname,
            status,
            role,
            grade,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, PartialEq, Eq, Clone)]
pub struct UpdateEmployeeRequest {
    pub id: i32,
    #[validate(length(min = 1, message = "name is required"))]
    pub firstname: String,
    #[validate(length(min = 1, message = "lastname is required"))]
    pub lastname: String,
    pub status: StatusEnum,
    pub role: EmployeeRole,
    pub grade: Option<GradeEnum>,
}

impl UpdateEmployeeRequest {
    pub fn new(
        id: i32,
        firstname: String,
        lastname: String,
        status: StatusEnum,
        role: EmployeeRole,
        grade: Option<GradeEnum>,
    ) -> UpdateEmployeeRequest {
        UpdateEmployeeRequest {
            id,
            firstname,
            lastname,
            status,
            role,
            grade,
        }
    }
}

//these are ssr methods used to assist in the conversion/serialization/deserialization of
//StatusEnum from postgres
cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")]{
        use sqlx::{Postgres, Encode, Decode, Type, postgres::{PgTypeInfo, PgValueRef, PgArgumentBuffer}, encode::IsNull};
        use sqlx::prelude::*;

        impl<'q>sqlx::encode::Encode<'q, sqlx::Postgres> for StatusEnum {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                let s = self.to_string();
                <&str as Encode<Postgres>>::encode(&s.as_str(), buf)
            }
        }
        impl <'r>sqlx::decode::Decode<'r, sqlx::Postgres> for StatusEnum {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let s: &str = Decode::<sqlx::Postgres>::decode(value)?;
                StatusEnum::from_str(s).map_err(|_| format!("Invalid StatusEnum: {:?}", s).into())
            }
        }
        impl Type<Postgres> for StatusEnum {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("status_enum")
            }
        }
        impl<'q>sqlx::encode::Encode<'q, sqlx::Postgres> for EmployeeRole {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                let s = self.to_string();
                <&str as Encode<Postgres>>::encode(&s.as_str(), buf)
            }
        }
        impl <'r>sqlx::decode::Decode<'r, sqlx::Postgres> for EmployeeRole {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let s: &str = Decode::<sqlx::Postgres>::decode(value)?;
                EmployeeRole::from_str(s).map_err(|_| format!("Invalid EmployeeRole: {:?}", s).into())
            }
        }
        impl Type<Postgres> for EmployeeRole {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("employee_role")
            }
        }

    }
}
