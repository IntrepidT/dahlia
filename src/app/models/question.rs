use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};
use std::str::FromStr;
use strum_macros::EnumIter;
use validator::Validate;

//these following enum is defined for use within the question struct
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, EnumIter)]
pub enum QuestionType {
    MultipleChoice,
    Written,
    Selection,
    TrueFalse,
}

impl fmt::Display for QuestionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                QuestionType::MultipleChoice => "Multiple choice".to_string(),
                QuestionType::Written => "Written".to_string(),
                QuestionType::Selection => "Selection".to_string(),
                QuestionType::TrueFalse => "True False".to_string(),
            }
        )
    }
}

impl FromStr for QuestionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Multiple choice" => Ok(QuestionType::MultipleChoice),
            "Written" => Ok(QuestionType::Written),
            "Selection" => Ok(QuestionType::Selection),
            "True False" => Ok(QuestionType::TrueFalse),
            _ => Err(format!("Invalid QuestionType (enum) value: {}", s)),
        }
    }
}

//the following Question object is for use on the client side when reading and writing data
#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Question {
    #[validate(length(min = 1, message = "please that a question is asked"))]
    pub word_problem: String,
    #[validate(range(min = -20000, max = 99999, message = "please ensure a score is associated with this question"))]
    pub point_value: i32,
    pub question_type: QuestionType,
    #[validate(length(min = 1, message = "please ensure that an answer is provided"))]
    pub options: Vec<String>,
    #[validate(length(min = 1, message = "please provide the correct answer"))]
    pub correct_answer: String,
    pub qnumber: i32,
    pub testlinker: String,
}

impl Question {
    pub fn new(
        word_problem: String,
        point_value: i32,
        question_type: QuestionType,
        options: Vec<String>,
        correct_answer: String,
        qnumber: i32,
        testlinker: String,
    ) -> Question {
        Question {
            word_problem,
            point_value,
            question_type,
            options,
            correct_answer,
            qnumber,
            testlinker,
        }
    }
}

//the following Objects are for use in making requests to the database on the client-side
#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct CreateNewQuestionRequest {
    #[validate(length(min = 1, message = "please that a question is asked"))]
    pub word_problem: String,
    #[validate(range(min = -20000, max = 99999, message = "please ensure a score is associated with this question"))]
    pub point_value: i32,
    pub question_type: QuestionType,
    #[validate(length(min = 1, message = "please ensure that an answer is provided"))]
    pub options: Vec<String>,
    #[validate(length(min = 1, message = "please provide the correct answer"))]
    pub correct_answer: String,
    pub qnumber: i32,
    pub testlinker: String,
}

impl CreateNewQuestionRequest {
    pub fn new(
        word_problem: String,
        point_value: i32,
        question_type: QuestionType,
        options: Vec<String>,
        correct_answer: String,
        qnumber: i32,
        testlinker: String,
    ) -> CreateNewQuestionRequest {
        CreateNewQuestionRequest {
            word_problem,
            point_value,
            question_type,
            options,
            correct_answer,
            qnumber,
            testlinker,
        }
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct UpdateQuestionRequest {
    #[validate(length(min = 1, message = "please that a question is asked"))]
    pub word_problem: String,
    #[validate(range(min = -20000, max = 99999, message = "please ensure a score is associated with this question"))]
    pub point_value: i32,
    pub question_type: QuestionType,
    #[validate(length(min = 1, message = "please ensure that an answer is provided"))]
    pub options: Vec<String>,
    #[validate(length(min = 1, message = "please provide the correct answer"))]
    pub correct_answer: String,
    pub qnumber: i32,
    pub testlinker: String,
}

impl UpdateQuestionRequest {
    pub fn new(
        word_problem: String,
        point_value: i32,
        question_type: QuestionType,
        options: Vec<String>,
        correct_answer: String,
        qnumber: i32,
        testlinker: String,
    ) -> UpdateQuestionRequest {
        UpdateQuestionRequest {
            word_problem,
            point_value,
            question_type,
            options,
            correct_answer,
            qnumber,
            testlinker,
        }
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct DeleteQuestionRequest {
    pub qnumber: i32,
    pub testlinker: String,
}

impl DeleteQuestionRequest {
    pub fn new(qnumber: i32, testlinker: String) -> DeleteQuestionRequest {
        DeleteQuestionRequest {
            qnumber,
            testlinker,
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use sqlx::{Postgres, Encode, Decode, Type, postgres::{PgTypeInfo, PgValueRef, PgArgumentBuffer}, encode::IsNull};
        use sqlx::prelude::*;

        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for QuestionType {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                let s = self.to_string();
                <&str as Encode<Postgres>>::encode(&s.as_str(), buf)
            }
        }

        impl<'r> sqlx::decode::Decode<'r, sqlx::Postgres> for QuestionType {
            fn decode(value: sqlx::postgres::PgValueRef<'r>)-> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let s: &str = Decode::<sqlx::Postgres>::decode(value)?;
                QuestionType::from_str(s).map_err(|_| format!("Invalid QuestionType: {}", s).into())
            }
        }

        impl Type<Postgres> for QuestionType {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("questiontype_enum")
            }
        }
    }
}
