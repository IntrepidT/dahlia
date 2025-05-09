use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Score {
    pub student_id: i32,
    pub date_administered: DateTime<Utc>,
    pub test_id: String,
    pub test_scores: Vec<i32>,
    pub comments: Vec<String>,
    pub test_variant: i32,
    pub evaluator: String,
    pub attempt: i32,
    //in theory it should be most efficient to use a hashmap whereby the key to the map is the
    //qnumber from the questions_table and links to the tuple: (points, comments)
}

impl Score {
    pub fn new(
        student_id: i32,
        date_administered: DateTime<Utc>,
        //I think i want this to be Local date offset on the front end but save to UTC on the back
        test_id: String,
        test_scores: Vec<i32>,
        comments: Vec<String>,
        test_variant: i32,
        evaluator: String,
        attempt: i32,
    ) -> Score {
        Score {
            student_id,
            date_administered,
            test_id,
            test_scores,
            comments,
            test_variant,
            evaluator,
            attempt,
        }
    }
    pub fn get_total(&self) -> i32 {
        self.test_scores.iter().sum()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct CreateScoreRequest {
    pub student_id: i32,
    pub test_id: String,
    pub test_scores: Vec<i32>,
    pub comments: Vec<String>,
    pub test_variant: i32,
    pub evaluator: String,
}

impl CreateScoreRequest {
    pub fn new(
        student_id: i32,
        test_id: String,
        test_scores: Vec<i32>,
        comments: Vec<String>,
        test_variant: i32,
        evaluator: String,
    ) -> CreateScoreRequest {
        CreateScoreRequest {
            student_id,
            test_id,
            test_scores,
            comments,
            test_variant,
            evaluator,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct UpdateScoreRequest {
    pub student_id: i32,
    pub date_administered: DateTime<Utc>,
    //I think i want this to be Local date offset on the front end but save to UTC on the back
    pub test_id: String,
    pub test_scores: Vec<i32>,
    pub comments: Vec<String>,
    pub test_variant: i32,
    pub evaluator: String,
    pub attempt: i32,
}

impl UpdateScoreRequest {
    pub fn new(
        student_id: i32,
        date_administered: DateTime<Utc>,
        test_id: String,
        test_scores: Vec<i32>,
        comments: Vec<String>,
        test_variant: i32,
        //I think i want this to be Local date offset on the front end but save to UTC on the back
        evaluator: String,
        attempt: i32,
    ) -> UpdateScoreRequest {
        UpdateScoreRequest {
            student_id,
            date_administered,
            test_id,
            test_scores,
            comments,
            test_variant,
            evaluator,
            attempt,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct DeleteScoreRequest {
    pub student_id: i32,
    //test_id refers to the key used to create the hashmap
    pub test_id: String,
    pub test_variant: i32,
    pub attempt: i32,
}

impl DeleteScoreRequest {
    pub fn new(
        student_id: i32,
        test_id: String,
        test_variant: i32,
        attempt: i32,
    ) -> DeleteScoreRequest {
        DeleteScoreRequest {
            student_id,
            test_id,
            test_variant,
            attempt,
        }
    }
}
