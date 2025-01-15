use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display};
use strum_macros::EnumString;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy, EnumString)]
pub enum QuestionType {
    MultipleChoice,
    Written,
    Selection,
    TrueFalse,
}

impl fmt::Display for QuestionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Question {
    #[validate(length(min = 1, message = "please that a question is asked"))]
    pub word_problem: String,
    #[validate(range(min = -20000, max = 99999, message = "please ensure a score is associated with this question"))]
    pub point_value: i32,
    pub qtype: QuestionType,
    #[validate(length(min = 1, message = "please ensure that an answer is provided"))]
    pub options: Vec<String>,
    #[validate(length(min = 1, message = "please provide the correct answer"))]
    pub correct_answer: String,
    pub comments: String,
    pub qnumber: i64,
}

impl Question {
    pub fn new(
        word_problem: String,
        point_value: i32,
        qtype: QuestionType,
        options: Vec<String>,
        correct_answer: String,
        comments: String,
        qnumber: i64,
    ) -> Question {
        Question {
            word_problem,
            point_value,
            qtype,
            options,
            correct_answer,
            comments,
            qnumber,
        }
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct CreateNewQuestionRequest {
    #[validate(length(min = 1, message = "please that a question is asked"))]
    pub word_problem: String,
    #[validate(range(min = -20000, max = 99999, message = "please ensure a score is associated with this question"))]
    pub point_value: i32,
    pub qtype: QuestionType,
    #[validate(length(min = 1, message = "please ensure that an answer is provided"))]
    pub options: Vec<String>,
    #[validate(length(min = 1, message = "please provide the correct answer"))]
    pub correct_answer: String,
    pub comments: String,
    pub qnumber: i64,
}

impl CreateNewQuestionRequest {
    pub fn new(
        word_problem: String,
        point_value: i32,
        qtype: QuestionType,
        options: Vec<String>,
        correct_answer: String,
        comments: String,
        qnumber: i64,
    ) -> CreateNewQuestionRequest {
        CreateNewQuestionRequest {
            word_problem,
            point_value,
            qtype,
            options,
            correct_answer,
            comments,
            qnumber,
        }
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct EditQuestionRequest {
    #[validate(length(min = 1, message = "please that a question is asked"))]
    pub word_problem: String,
    #[validate(range(min = -20000, max = 99999, message = "please ensure a score is associated with this question"))]
    pub point_value: i32,
    pub qtype: QuestionType,
    #[validate(length(min = 1, message = "please ensure that an answer is provided"))]
    pub options: Vec<String>,
    #[validate(length(min = 1, message = "please provide the correct answer"))]
    pub correct_answer: String,
    pub comments: String,
    pub qnumber: i64,
}

impl EditQuestionRequest {
    pub fn new(
        word_problem: String,
        point_value: i32,
        qtype: QuestionType,
        options: Vec<String>,
        correct_answer: String,
        comments: String,
        qnumber: i64,
    ) -> EditQuestionRequest {
        EditQuestionRequest {
            word_problem,
            point_value,
            qtype,
            options,
            correct_answer,
            comments,
            qnumber,
        }
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct DeleteQuestionRequest {
    pub qnumber: i64,
}

impl DeleteQuestionRequest {
    pub fn new(qnumber: i64) -> DeleteQuestionRequest {
        DeleteQuestionRequest { qnumber }
    }
}
