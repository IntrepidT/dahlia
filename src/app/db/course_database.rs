use leptos::prelude::*;
cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::app::models::course::{Course, CreateCourseRequest, UpdateCourseRequest};
        use crate::app::models::student::GradeEnum;
        use crate::app::models::enrollment::AcademicYear;
        use chrono::{DateTime, Utc};
        use log::{debug, error, info, warn};
        use sqlx::prelude::*;
        use sqlx::PgPool;

        pub async fn get_all_courses(pool: &PgPool) -> Result<Vec<Course>, ServerFnError> {
            let rows = sqlx::query(
                "SELECT id, name, subject, course_code, course_level, teacher_id, academic_year, semester_period, credits, description, max_students, room_number, created_at, updated_at FROM courses ORDER BY academic_year DESC, course_level, subject, name"
            )
            .fetch_all(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let courses: Vec<Course> = rows
                .into_iter()
                .map(|row| {
                    Course {
                        id: row.get("id"),
                        name: row.get("name"),
                        subject: row.get("subject"),
                        course_code: row.get("course_code"),
                        course_level: row.get("course_level"),
                        teacher_id: row.get("teacher_id"),
                        academic_year: row.get("academic_year"),
                        semester_period: row.get("semester_period"),
                        credits: row.get("credits"),
                        description: row.get("description"),
                        max_students: row.get("max_students"),
                        room_number: row.get("room_number"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                    }
                })
                .collect();

            Ok(courses)
        }

        pub async fn get_course_by_id(pool: &PgPool, course_id: i32) -> Result<Course, ServerFnError> {
            let row = sqlx::query(
                "SELECT id, name, subject, course_code, course_level, teacher_id, academic_year, semester_period, credits, description, max_students, room_number, created_at, updated_at FROM courses WHERE id = $1"
            )
            .bind(course_id)
            .fetch_one(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let course = Course {
                id: row.get("id"),
                name: row.get("name"),
                subject: row.get("subject"),
                course_code: row.get("course_code"),
                course_level: row.get("course_level"),
                teacher_id: row.get("teacher_id"),
                academic_year: row.get("academic_year"),
                semester_period: row.get("semester_period"),
                credits: row.get("credits"),
                description: row.get("description"),
                max_students: row.get("max_students"),
                room_number: row.get("room_number"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };

            Ok(course)
        }

        pub async fn get_course_by_code(pool: &PgPool, course_code: &str) -> Result<Course, ServerFnError> {
            let row = sqlx::query(
                "SELECT id, name, subject, course_code, course_level, teacher_id, academic_year, semester_period, credits, description, max_students, room_number, created_at, updated_at FROM courses WHERE course_code = $1"
            )
            .bind(course_code)
            .fetch_one(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let course = Course {
                id: row.get("id"),
                name: row.get("name"),
                subject: row.get("subject"),
                course_code: row.get("course_code"),
                course_level: row.get("course_level"),
                teacher_id: row.get("teacher_id"),
                academic_year: row.get("academic_year"),
                semester_period: row.get("semester_period"),
                credits: row.get("credits"),
                description: row.get("description"),
                max_students: row.get("max_students"),
                room_number: row.get("room_number"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };

            Ok(course)
        }

        pub async fn add_course(pool: &PgPool, request: CreateCourseRequest) -> Result<Course, ServerFnError> {
            let row = sqlx::query(
                "INSERT INTO courses (name, subject, course_code, course_level, teacher_id, academic_year, semester_period, credits, description, max_students, room_number) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id, name, subject, course_code, course_level, teacher_id, academic_year, semester_period, credits, description, max_students, room_number, created_at, updated_at"
            )
            .bind(&request.name)
            .bind(&request.subject)
            .bind(&request.course_code)
            .bind(&request.course_level)
            .bind(&request.teacher_id)
            .bind(&request.academic_year)
            .bind(&request.semester_period)
            .bind(&request.credits)
            .bind(&request.description)
            .bind(&request.max_students)
            .bind(&request.room_number)
            .fetch_one(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let course = Course {
                id: row.get("id"),
                name: row.get("name"),
                subject: row.get("subject"),
                course_code: row.get("course_code"),
                course_level: row.get("course_level"),
                teacher_id: row.get("teacher_id"),
                academic_year: row.get("academic_year"),
                semester_period: row.get("semester_period"),
                credits: row.get("credits"),
                description: row.get("description"),
                max_students: row.get("max_students"),
                room_number: row.get("room_number"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };

            Ok(course)
        }

        pub async fn update_course(pool: &PgPool, course_id: i32, request: UpdateCourseRequest) -> Result<Course, ServerFnError> {
            // First, get the current course to use as defaults for None values
            let current_course = get_course_by_id(pool, course_id).await?;

            let row = sqlx::query(
                "UPDATE courses SET name = $2, subject = $3, course_code = $4, course_level = $5, teacher_id = $6, academic_year = $7, semester_period = $8, credits = $9, description = $10, max_students = $11, room_number = $12, updated_at = CURRENT_TIMESTAMP WHERE id = $1 RETURNING id, name, subject, course_code, course_level, teacher_id, academic_year, semester_period, credits, description, max_students, room_number, created_at, updated_at"
            )
            .bind(course_id)
            .bind(request.name.as_ref().unwrap_or(&current_course.name))
            .bind(request.subject.as_ref().unwrap_or(&current_course.subject))
            .bind(request.course_code.as_ref().unwrap_or(&current_course.course_code))
            .bind(request.course_level.as_ref().unwrap_or(&current_course.course_level))
            .bind(request.teacher_id.unwrap_or(current_course.teacher_id))
            .bind(request.academic_year.as_ref().unwrap_or(&current_course.academic_year))
            .bind(request.semester_period.as_ref().unwrap_or(&current_course.semester_period))
            .bind(request.credits.unwrap_or(current_course.credits))
            .bind(request.description.as_ref().unwrap_or(&current_course.description))
            .bind(request.max_students.unwrap_or(current_course.max_students))
            .bind(
                match &request.room_number {
                    Some(new_room) => new_room.as_ref(), // Some(Some(val)) -> Some(val), Some(None) -> None
                    None => current_course.room_number.as_ref(), // Use current value if not specified
                }
            )
            .fetch_one(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let course = Course {
                id: row.get("id"),
                name: row.get("name"),
                subject: row.get("subject"),
                course_code: row.get("course_code"),
                course_level: row.get("course_level"),
                teacher_id: row.get("teacher_id"),
                academic_year: row.get("academic_year"),
                semester_period: row.get("semester_period"),
                credits: row.get("credits"),
                description: row.get("description"),
                max_students: row.get("max_students"),
                room_number: row.get("room_number"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };

            Ok(course)
        }

        pub async fn delete_course(pool: &PgPool, course_id: i32) -> Result<(), ServerFnError> {
            let result = sqlx::query("DELETE FROM courses WHERE id = $1")
                .bind(course_id)
                .execute(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            if result.rows_affected() == 0 {
                return Err(ServerFnError::new("Course not found".to_string()));
            }

            Ok(())
        }

        // Additional helper functions
        pub async fn get_courses_by_teacher(pool: &PgPool, teacher_id: i32) -> Result<Vec<Course>, ServerFnError> {
            let rows = sqlx::query(
                "SELECT id, name, subject, course_code, course_level, teacher_id, academic_year, semester_period, credits, description, max_students, room_number, created_at, updated_at FROM courses WHERE teacher_id = $1 ORDER BY academic_year DESC, course_level, subject, name"
            )
            .bind(teacher_id)
            .fetch_all(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let courses: Vec<Course> = rows
                .into_iter()
                .map(|row| {
                    Course {
                        id: row.get("id"),
                        name: row.get("name"),
                        subject: row.get("subject"),
                        course_code: row.get("course_code"),
                        course_level: row.get("course_level"),
                        teacher_id: row.get("teacher_id"),
                        academic_year: row.get("academic_year"),
                        semester_period: row.get("semester_period"),
                        credits: row.get("credits"),
                        description: row.get("description"),
                        max_students: row.get("max_students"),
                        room_number: row.get("room_number"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                    }
                })
                .collect();

            Ok(courses)
        }

        pub async fn get_courses_by_academic_year(pool: &PgPool, academic_year: AcademicYear) -> Result<Vec<Course>, ServerFnError> {
            let rows = sqlx::query(
                "SELECT id, name, subject, course_code, course_level, teacher_id, academic_year, semester_period, credits, description, max_students, room_number, created_at, updated_at FROM courses WHERE academic_year = $1 ORDER BY course_level, subject, name"
            )
            .bind(&academic_year)
            .fetch_all(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let courses: Vec<Course> = rows
                .into_iter()
                .map(|row| {
                    Course {
                        id: row.get("id"),
                        name: row.get("name"),
                        subject: row.get("subject"),
                        course_code: row.get("course_code"),
                        course_level: row.get("course_level"),
                        teacher_id: row.get("teacher_id"),
                        academic_year: row.get("academic_year"),
                        semester_period: row.get("semester_period"),
                        credits: row.get("credits"),
                        description: row.get("description"),
                        max_students: row.get("max_students"),
                        room_number: row.get("room_number"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                    }
                })
                .collect();

            Ok(courses)
        }

        pub async fn get_courses_by_level(pool: &PgPool, level: GradeEnum) -> Result<Vec<Course>, ServerFnError> {
            let rows = sqlx::query(
                "SELECT id, name, subject, course_code, course_level, teacher_id, academic_year, semester_period, credits, description, max_students, room_number, created_at, updated_at FROM courses WHERE course_level = $1 ORDER BY academic_year DESC, subject, name"
            )
            .bind(&level)
            .fetch_all(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let courses: Vec<Course> = rows
                .into_iter()
                .map(|row| {
                    Course {
                        id: row.get("id"),
                        name: row.get("name"),
                        subject: row.get("subject"),
                        course_code: row.get("course_code"),
                        course_level: row.get("course_level"),
                        teacher_id: row.get("teacher_id"),
                        academic_year: row.get("academic_year"),
                        semester_period: row.get("semester_period"),
                        credits: row.get("credits"),
                        description: row.get("description"),
                        max_students: row.get("max_students"),
                        room_number: row.get("room_number"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                    }
                })
                .collect();

            Ok(courses)
        }

        pub async fn check_course_exists(pool: &PgPool, course_code: &str) -> Result<bool, ServerFnError> {
            let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM courses WHERE course_code = $1)")
                .bind(course_code)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let exists: bool = row.get("exists");
            Ok(exists)
        }

        pub async fn get_course_enrollment_count(pool: &PgPool, course_id: i32) -> Result<i64, ServerFnError> {
            let row = sqlx::query("SELECT COUNT(*) as count FROM student_enrollments WHERE course_id = $1")
                .bind(course_id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let count: i64 = row.get("count");
            Ok(count)
        }
    }
}
