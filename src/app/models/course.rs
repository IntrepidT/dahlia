use crate::app::models::enrollment::AcademicYear;
use crate::app::models::student::GradeEnum;
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Course {
    pub id: i32,
    pub name: String,
    pub subject: String,
    pub course_code: String,
    pub course_level: GradeEnum,
    pub teacher_id: i32,
    pub academic_year: AcademicYear,
    pub semester_period: String,
    pub credits: Decimal,
    pub description: String,
    pub max_students: i32,
    pub room_number: Option<String>, // This should be Option<String> to match DB schema
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Course {
    pub fn new(
        id: i32,
        name: String,
        subject: String,
        course_code: String,
        course_level: GradeEnum,
        teacher_id: i32,
        academic_year: AcademicYear,
        semester_period: String,
        credits: Decimal,
        description: String,
        max_students: i32,
        room_number: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Course {
            id,
            name,
            subject,
            course_code,
            course_level,
            teacher_id,
            academic_year,
            semester_period,
            credits,
            description,
            max_students,
            room_number,
            created_at,
            updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateCourseRequest {
    pub name: String,
    pub subject: String,
    pub course_code: String,
    pub course_level: GradeEnum,
    pub teacher_id: i32,
    pub academic_year: AcademicYear,
    pub semester_period: String,
    pub credits: Decimal,
    pub description: String,
    pub max_students: i32,
    pub room_number: Option<String>, // Changed to Option<String> to match DB schema
}

impl CreateCourseRequest {
    pub fn new(
        name: String,
        subject: String,
        course_code: String,
        course_level: GradeEnum,
        teacher_id: i32,
        academic_year: AcademicYear,
        semester_period: String,
        credits: Decimal,
        description: String,
        max_students: i32,
        room_number: Option<String>, // Changed to Option<String>
    ) -> Self {
        CreateCourseRequest {
            name,
            subject,
            course_code,
            course_level,
            teacher_id,
            academic_year,
            semester_period,
            credits,
            description,
            max_students,
            room_number,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateCourseRequest {
    pub name: Option<String>,
    pub subject: Option<String>,
    pub course_code: Option<String>,
    pub course_level: Option<GradeEnum>,
    pub teacher_id: Option<i32>,
    pub academic_year: Option<AcademicYear>,
    pub semester_period: Option<String>,
    pub credits: Option<Decimal>,
    pub description: Option<String>,
    pub max_students: Option<i32>,
    pub room_number: Option<Option<String>>, // This is Option<Option<String>> to allow setting to NULL
}

impl UpdateCourseRequest {
    pub fn new(
        name: Option<String>,
        subject: Option<String>,
        course_code: Option<String>,
        course_level: Option<GradeEnum>,
        teacher_id: Option<i32>,
        academic_year: Option<AcademicYear>,
        semester_period: Option<String>,
        credits: Option<Decimal>,
        description: Option<String>,
        max_students: Option<i32>,
        room_number: Option<Option<String>>, // Changed to Option<Option<String>>
    ) -> Self {
        UpdateCourseRequest {
            name,
            subject,
            course_code,
            course_level,
            teacher_id,
            academic_year,
            semester_period,
            credits,
            description,
            max_students,
            room_number,
        }
    }

    // Helper method to create a partial update
    pub fn partial_update() -> Self {
        UpdateCourseRequest {
            name: None,
            subject: None,
            course_code: None,
            course_level: None,
            teacher_id: None,
            academic_year: None,
            semester_period: None,
            credits: None,
            description: None,
            max_students: None,
            room_number: None,
        }
    }
}
