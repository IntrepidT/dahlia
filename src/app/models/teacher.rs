use crate::app::models::employee::{Employee, EmployeeRole, StatusEnum};
use crate::app::models::student::GradeEnum;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

impl Employee {
    pub fn new_teacher(
        id: i32,
        firstname: String,
        lastname: String,
        status: StatusEnum,
        grade: Option<GradeEnum>,
    ) -> Self {
        Self::new(
            id,
            firstname,
            lastname,
            status,
            EmployeeRole::Teacher { grade },
        )
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AddNewTeacherRequest {
    pub firstname: String,
    pub lastname: String,
}

impl AddNewTeacherRequest {
    pub fn new(firstname: String, lastname: String) -> AddNewTeacherRequest {
        AddNewTeacherRequest {
            firstname,
            lastname,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct DeleteTeacherRequest {
    pub id: i32,
}

impl DeleteTeacherRequest {
    pub fn new(id: i32) -> DeleteTeacherRequest {
        DeleteTeacherRequest { id }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct UpdateTeacherRequest {
    pub id: i32,
    pub firstname: String,
    pub lastname: String,
    pub status: StatusEnum,
    pub grade: Option<GradeEnum>,
}

impl UpdateTeacherRequest {
    pub fn new(
        id: i32,
        firstname: String,
        lastname: String,
        status: StatusEnum,
        grade: Option<GradeEnum>,
    ) -> UpdateTeacherRequest {
        UpdateTeacherRequest {
            id,
            firstname,
            lastname,
            status,
            grade,
        }
    }
}
