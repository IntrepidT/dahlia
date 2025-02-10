use chrono::preulude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Debug, Display};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Score {
    pub student_name: String,
    pub date_administered: DateTime,
    pub test_data: HashMap,
    pub test_variant: i32,
    pub evaluator: String,
    //in theory it should be most efficient to use a hashmap whereby the key to the map is the
    //qnumber from the questions_table and links to the tuple: (points, comments)
}

impl Score {
    pub fn new(
        student_name: String,
        student_id: i32,
        date_administered: DateTime,
        //I think i want this to be Local date offset on the front end but save to UTC on the back
        test_data: HashMap,
        test_variant: i32,
        evaluator: String,
    ) -> Score {
        Scores {
            student_name,
            date_administered,
            test_data,
            test_variant,
            evaluator,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct CreateScoreRequest {
    pub student_id: i32,
    pub test_variant: i32,
    pub test_data: HashMap,
}

impl CreateScoreRequest {
    pub fn new(student_id: i32, test_variant: i32, test_data: HashMap) -> CreateScoreRequest {
        CreateScoreRequest {
            student_id,
            test_variant,
            test_data,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct UpdateScoreRequest {
    pub student_name: String,
    pub student_id: i32,
    pub date_administered: DateTime,
    //I think i want this to be Local date offset on the front end but save to UTC on the back
    pub test_data: HashMap,
    pub test_variant: i32,
    pub evaluator: String,
}

impl UpdateScoreRequest {
    pub fn new(
        student_name: String,
        student_id: i32,
        date_administered: DateTime,
        //I think i want this to be Local date offset on the front end but save to UTC on the back
        test_data: HashMap,
        test_variant: i32,
        evaluator: String,
    ) -> UpdateScoreRequest {
        UpdateScoreRequest {
            student_name,
            student_id,
            date_administered,
            test_data,
            test_variant,
            evaluator,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct DeleteScoreRequest {
    pub student_id: i32,
    //test_id refers to the key used to create the hashmap
    pub test_id: i32,
    pub test_variant: i32,
}

impl DeleteScoreRequest {
    pub fn new(student_id: i32, test_id: i32, test_variant: i32) -> DeleteScoreRequest {
        DeleteScoreRequest {
            student_id,
            test_id,
            test_variant,
        }
    }
}
