use crate::app::models::assessment::ScopeEnum;
use crate::app::models::student::GradeEnum;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};
use std::str::FromStr;
use strum_macros::EnumString;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct BenchmarkCategory {
    pub min: i32,
    pub max: i32,
    pub label: String,
}

impl BenchmarkCategory {
    // Original constructor (now for ranges)
    pub fn new(min: i32, max: i32, label: String) -> BenchmarkCategory {
        BenchmarkCategory { min, max, label }
    }

    // Constructor for range (alias for clarity)
    pub fn new_range(min: i32, max: i32, label: String) -> BenchmarkCategory {
        BenchmarkCategory { min, max, label }
    }

    // Constructor for single value
    pub fn new_single(value: i32, label: String) -> BenchmarkCategory {
        BenchmarkCategory {
            min: value,
            max: value,
            label,
        }
    }

    // Helper method to check if this is a single value
    pub fn is_single_value(&self) -> bool {
        self.min == self.max
    }

    // Helper method to check if a score falls within this category
    pub fn contains(&self, score: i32) -> bool {
        score >= self.min && score <= self.max
    }

    // Helper method to get the display text for the range
    pub fn range_display(&self) -> String {
        if self.is_single_value() {
            self.min.to_string()
        } else {
            format!("{}-{}", self.min, self.max)
        }
    }

    // Helper method to get the value for single-value categories
    pub fn get_single_value(&self) -> Option<i32> {
        if self.is_single_value() {
            Some(self.min)
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, EnumString)]
pub enum TestType {
    #[strum(to_string = "Reading")]
    Reading,
    #[strum(to_string = "Math")]
    Math,
    #[strum(to_string = "Other")]
    Other,
}

impl fmt::Display for TestType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TestType::Reading => "Reading".to_string(),
                TestType::Math => "Math".to_string(),
                TestType::Other => "Other".to_string(),
            }
        )
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Test {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    #[validate(range(min = -20000, max = 99999, message = "score is required"))]
    pub score: i32,
    pub instructions: Option<String>,
    pub comments: String,
    pub testarea: TestType,
    pub school_year: Option<String>,
    pub benchmark_categories: Option<Vec<BenchmarkCategory>>,
    pub test_variant: i32,
    pub grade_level: Option<GradeEnum>,
    pub test_id: String,
    pub scope: Option<ScopeEnum>,
    pub course_id: Option<i32>,
}

impl Test {
    pub fn new(
        name: String,
        score: i32,
        instructions: Option<String>,
        comments: String,
        testarea: TestType,
        school_year: Option<String>,
        benchmark_categories: Option<Vec<BenchmarkCategory>>,
        test_variant: i32,
        grade_level: Option<GradeEnum>,
        test_id: String,
        scope: Option<ScopeEnum>,
        course_id: Option<i32>,
    ) -> Test {
        Test {
            name,
            score,
            instructions,
            comments,
            testarea,
            school_year,
            benchmark_categories,
            test_variant,
            grade_level,
            test_id,
            scope,
            course_id,
        }
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct CreateNewTestRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    #[validate(range(min = 0, max = 99999, message = "a score is required"))]
    pub score: i32,
    pub instructions: Option<String>,
    pub comments: String,
    pub testarea: TestType,
    pub school_year: Option<String>,
    pub benchmark_categories: Option<Vec<BenchmarkCategory>>,
    pub test_variant: i32,
    pub grade_level: Option<GradeEnum>,
    pub scope: Option<ScopeEnum>,
    pub course_id: Option<i32>,
}

impl CreateNewTestRequest {
    pub fn new(
        name: String,
        score: i32,
        instructions: Option<String>,
        comments: String,
        testarea: TestType,
        school_year: Option<String>,
        benchmark_categories: Option<Vec<BenchmarkCategory>>,
        test_variant: i32,
        grade_level: Option<GradeEnum>,
        scope: Option<ScopeEnum>,
        course_id: Option<i32>,
    ) -> CreateNewTestRequest {
        CreateNewTestRequest {
            name,
            score,
            instructions,
            comments,
            testarea,
            school_year,
            benchmark_categories,
            test_variant,
            grade_level,
            scope,
            course_id,
        }
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct UpdateTestRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    #[validate(range(min = -20000, max = 99999, message = "a score is required"))]
    pub score: i32,
    pub instructions: Option<String>,
    pub comments: String,
    pub testarea: TestType,
    pub school_year: Option<String>,
    pub benchmark_categories: Option<Vec<BenchmarkCategory>>,
    pub test_variant: i32,
    pub grade_level: Option<GradeEnum>,
    pub test_id: String,
    pub scope: Option<ScopeEnum>,
    pub course_id: Option<i32>,
}

impl UpdateTestRequest {
    pub fn new(
        name: String,
        score: i32,
        instructions: Option<String>,
        comments: String,
        testarea: TestType,
        school_year: Option<String>,
        benchmark_categories: Option<Vec<BenchmarkCategory>>,
        test_variant: i32,
        grade_level: Option<GradeEnum>,
        test_id: String,
        scope: Option<ScopeEnum>,
        course_id: Option<i32>,
    ) -> UpdateTestRequest {
        UpdateTestRequest {
            name,
            score,
            instructions,
            comments,
            testarea,
            school_year,
            benchmark_categories,
            test_variant,
            grade_level,
            test_id,
            scope,
            course_id,
        }
    }
}

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
        use sqlx::{Postgres, Encode, Decode, Type, postgres::{PgTypeInfo, PgValueRef, PgArgumentBuffer, PgHasArrayType}, encode::IsNull};
        use sqlx::prelude::*;
        use sqlx::types::Json;

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

        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for BenchmarkCategory {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                Json(self).encode_by_ref(buf)
            }
        }

        impl sqlx::Type<sqlx::Postgres> for BenchmarkCategory {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("jsonb")
            }
        }

        impl<'r> sqlx::decode::Decode<'r, sqlx::Postgres> for BenchmarkCategory {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let json: Json<BenchmarkCategory> = sqlx::decode::Decode::decode(value)?;
                Ok(json.0)
            }
        }

        impl sqlx::postgres::PgHasArrayType for BenchmarkCategory {
            fn array_type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("_jsonb")
            }
        }

        // Create a newtype wrapper for Vec<BenchmarkCategory> to solve the orphan rule issue
        #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
        pub struct BenchmarkCategories(pub Vec<BenchmarkCategory>);

        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for BenchmarkCategories {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                Json(&self.0).encode_by_ref(buf)
            }
        }

        impl<'r> sqlx::decode::Decode<'r, sqlx::Postgres> for BenchmarkCategories {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let json: Json<Vec<BenchmarkCategory>> = sqlx::decode::Decode::decode(value)?;
                Ok(BenchmarkCategories(json.0))
            }
        }

    }
}
