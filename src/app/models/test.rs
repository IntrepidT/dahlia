use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};
use std::str::FromStr;
use strum_macros::EnumString;
use validator::Validate;
//this section of code defines the TestType enum for use on the server-side of the application
//should it be needed
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, EnumString)]
pub enum TestType {
    #[strum(to_string = "Reading")]
    Reading,
    #[strum(to_string = "Math")]
    Math,
}

impl fmt::Display for TestType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TestType::Reading => "Reading".to_string(),
                TestType::Math => "Math".to_string(),
            }
        )
    }
}
//this is the main test object, utilized when reading data out of postgres and creating this data
//on the client side
#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Test {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    #[validate(range(min = -20000, max = 99999, message = "score is required"))]
    pub score: i32,
    pub comments: String,
    pub testarea: TestType,
    pub test_id: String,
}

impl Test {
    pub fn new(
        name: String,
        score: i32,
        comments: String,
        testarea: TestType,
        test_id: String,
    ) -> Test {
        Test {
            name,
            score,
            comments,
            testarea,
            test_id,
        }
    }
}

//client-side request to be processed when writing a new Test object
#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct CreateNewTestRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    #[validate(range(min = 0, max = 99999, message = "a score is required"))]
    pub score: i32,
    pub comments: String,
    pub testarea: TestType,
}

impl CreateNewTestRequest {
    pub fn new(
        name: String,
        score: i32,
        comments: String,
        testarea: TestType,
    ) -> CreateNewTestRequest {
        CreateNewTestRequest {
            name,
            score,
            comments,
            testarea,
        }
    }
}

//client side object to be called when making modifications to a data in the Test Table
#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct UpdateTestRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    #[validate(range(min = -20000, max = 99999, message = "a score is required"))]
    pub score: i32,
    pub comments: String,
    pub testarea: TestType,
    pub test_id: String,
}
impl UpdateTestRequest {
    pub fn new(
        name: String,
        score: i32,
        comments: String,
        testarea: TestType,
        test_id: String,
    ) -> UpdateTestRequest {
        UpdateTestRequest {
            name,
            score,
            comments,
            testarea,
            test_id,
        }
    }
}

//client side object to be called when making deleting data from the Test table
#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct DeleteTestRequest {
    pub test_id: String,
}

impl DeleteTestRequest {
    pub fn new(test_id: String) -> DeleteTestRequest {
        DeleteTestRequest { test_id }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use sqlx::{Postgres, Encode, Decode, Type, postgres::{PgTypeInfo, PgValueRef, PgArgumentBuffer}, encode::IsNull};
        use sqlx::prelude::*;

        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for TestType {
           fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
               let s = self.to_string();
                <&str as Encode<Postgres>>::encode(&s.as_str(), buf)
           }
       }

        impl<'r> sqlx::decode::Decode<'r, sqlx::Postgres> for TestType {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let s: &str = Decode::<sqlx::Postgres>::decode(value)?;
                TestType::from_str(s).map_err(|_| format!("Invalid TestType: {}", s).into())
            }
        }

        impl Type<Postgres> for TestType {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("testarea_enum")
            }
        }
    }
}
