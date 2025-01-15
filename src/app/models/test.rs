use crate::app::models::question::{Question, QuestionType};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display};
use strum_macros::EnumString;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy, EnumString)]
pub enum test_type {
    Reading,
    Math,
}

impl fmt::Display for test_type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Test {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    #[validate(range(min = -20000, max = 99999, message = "score is required"))]
    pub score: i32,
    pub comments: String,
    pub test_area: test_type,
    #[validate(range(
        min = 1,
        max = 99999,
        message = "this is not a valid test ID: please reload the form"
    ))]
    pub test_identifier: i64,
    pub questions: Vec<Question>,
    pub date: String,
}

impl Test {
    pub fn new(
        name: String,
        score: i32,
        comments: String,
        test_area: test_type,
        test_identifier: i64,
        date: String,
    ) -> Test {
        let mut questions: Vec<Question> = Vec::new();
        Test {
            name,
            score,
            comments,
            test_area,
            test_identifier,
            questions,
            date,
        }
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct CreateNewTestRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    #[validate(range(min = -20000, max = 99999, message = "a score is required"))]
    pub score: i32,
    pub comments: String,
    pub test_area: test_type,
    #[validate(range(
        min = 1,
        max = 99999,
        message = "this is not a valid test ID: please reload the form"
    ))]
    pub test_identifier: i64,
}

impl CreateNewTestRequest {
    pub fn new(
        name: String,
        score: i32,
        comments: String,
        test_area: test_type,
        test_identifier: i64,
    ) -> CreateNewTestRequest {
        CreateNewTestRequest {
            name,
            score,
            comments,
            test_area,
            test_identifier,
        }
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct EditTestRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    #[validate(range(min = -20000, max = 99999, message = "a score is required"))]
    pub score: i32,
    pub comments: String,
    pub test_area: test_type,
    #[validate(range(
        min = 1,
        max = 99999,
        message = "this is not a valid test ID: please reload the form"
    ))]
    pub test_identifier: i64,
}
impl EditTestRequest {
    pub fn new(
        name: String,
        score: i32,
        comments: String,
        test_area: test_type,
        test_identifier: i64,
    ) -> EditTestRequest {
        EditTestRequest {
            name,
            score,
            comments,
            test_area,
            test_identifier,
        }
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct DeleteTestRequest {
    #[validate(range(min = 1, max = 99999))]
    pub test_identifier: i64,
}

impl DeleteTestRequest {
    pub fn new(test_identifier: i64) -> DeleteTestRequest {
        DeleteTestRequest { test_identifier }
    }
}
