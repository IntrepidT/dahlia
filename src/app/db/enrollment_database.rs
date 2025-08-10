use leptos::prelude::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")]{
        use crate::app::models::enrollment::{Enrollment, EnrollmentStatus, AcademicYear};
        use crate::app::models::student::GradeEnum;
        use log::{debug, error, info, warn};
        use chrono::NaiveDate;
        use sqlx::PgPool;
        use sqlx::prelude::*;

        pub async fn get_all_enrollments(pool: &PgPool) -> Result<Vec<Enrollment>, ServerFnError> {
            let rows = sqlx::query("SELECT student_id, academic_year, grade_level, teacher_id, status, enrollment_date, status_change_date, notes FROM student_enrollments ORDER BY enrollment_date DESC")
                .fetch_all(pool)
                .await?;

            let enrollments: Vec<Enrollment> = rows
                .into_iter()
                .map(|row| {
                    let student_id: i32 = row.get("student_id");
                    let academic_year: AcademicYear = row.get("academic_year");
                    let grade_level: GradeEnum = row.get("grade_level");
                    let teacher_id: i32 = row.get("teacher_id");
                    let status: EnrollmentStatus = row.get("status");
                    let enrollment_date: chrono::NaiveDate = row.get("enrollment_date");
                    let status_change_date: Option<chrono::NaiveDate> = row.get("status_change_date");
                    let notes: Option<String> = row.get("notes");

                    Enrollment {
                        student_id,
                        academic_year,
                        grade_level,
                        teacher_id,
                        status,
                        enrollment_date,
                        status_change_date,
                        notes,
                    }
                })
                .collect();
            Ok(enrollments)
        }

        pub async fn get_enrollments_by_student(student_id: &i32, pool: &PgPool) -> Result<Vec<Enrollment>, ServerFnError> {
            let rows = sqlx::query("SELECT student_id, academic_year, grade_level, teacher_id, status, enrollment_date, status_change_date, notes FROM student_enrollments WHERE student_id = $1 ORDER BY enrollment_date DESC")
                .bind(student_id)
                .fetch_all(pool)
                .await?;

            let enrollments: Vec<Enrollment> = rows
                .into_iter()
                .map(|row| {
                    Enrollment {
                        student_id: row.get("student_id"),
                        academic_year: row.get("academic_year"),
                        grade_level: row.get("grade_level"),
                        teacher_id: row.get("teacher_id"),
                        status: row.get("status"),
                        enrollment_date: row.get("enrollment_date"),
                        status_change_date: row.get("status_change_date"),
                        notes: row.get("notes"),
                    }
                })
                .collect();
            Ok(enrollments)
        }

        pub async fn get_enrollment_by_student_and_year(
            pool: &PgPool,
            student_id: i32,
            academic_year: AcademicYear,
        ) -> Result<Enrollment, ServerFnError> {
            let row = sqlx::query("SELECT student_id, academic_year, grade_level, teacher_id, status, enrollment_date, status_change_date, notes FROM student_enrollments WHERE student_id = $1 AND academic_year = $2 LIMIT 1")
                .bind(student_id)
                .bind(academic_year)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let enrollment = Enrollment {
                student_id: row.get("student_id"),
                academic_year: row.get("academic_year"),
                grade_level: row.get("grade_level"),
                teacher_id: row.get("teacher_id"),
                status: row.get("status"),
                enrollment_date: row.get("enrollment_date"),
                status_change_date: row.get("status_change_date"),
                notes: row.get("notes"),
            };

            Ok(enrollment)
        }

        pub async fn get_enrollments_by_academic_year(academic_year: &AcademicYear, pool: &PgPool) -> Result<Vec<Enrollment>, ServerFnError> {
            let rows = sqlx::query("SELECT student_id, academic_year, grade_level, teacher_id, status, enrollment_date, status_change_date, notes FROM student_enrollments WHERE academic_year = $1 ORDER BY enrollment_date DESC")
                .bind(academic_year)
                .fetch_all(pool)
                .await?;

            let enrollments: Vec<Enrollment> = rows
                .into_iter()
                .map(|row| {
                    Enrollment {
                        student_id: row.get("student_id"),
                        academic_year: row.get("academic_year"),
                        grade_level: row.get("grade_level"),
                        teacher_id: row.get("teacher_id"),
                        status: row.get("status"),
                        enrollment_date: row.get("enrollment_date"),
                        status_change_date: row.get("status_change_date"),
                        notes: row.get("notes"),
                    }
                })
                .collect();
            Ok(enrollments)
        }

        pub async fn get_enrollments_by_teacher(teacher_id: i32, pool: &PgPool) -> Result<Vec<Enrollment>, ServerFnError> {
            let rows = sqlx::query("SELECT student_id, academic_year, grade_level, teacher_id, status, enrollment_date, status_change_date, notes FROM student_enrollments WHERE teacher_id = $1 ORDER BY enrollment_date DESC")
                .bind(teacher_id)
                .fetch_all(pool)
                .await?;

            let enrollments: Vec<Enrollment> = rows
                .into_iter()
                .map(|row| {
                    Enrollment {
                        student_id: row.get("student_id"),
                        academic_year: row.get("academic_year"),
                        grade_level: row.get("grade_level"),
                        teacher_id: row.get("teacher_id"),
                        status: row.get("status"),
                        enrollment_date: row.get("enrollment_date"),
                        status_change_date: row.get("status_change_date"),
                        notes: row.get("notes"),
                    }
                })
                .collect();
            Ok(enrollments)
        }

        pub async fn get_current_enrollment(student_id: &i32, pool: &PgPool) -> Result<Option<Enrollment>, ServerFnError> {
            let row = sqlx::query("SELECT student_id, academic_year, grade_level, teacher_id, status, enrollment_date, status_change_date, notes FROM student_enrollments WHERE student_id = $1 AND status = 'active' ORDER BY enrollment_date DESC LIMIT 1")
                .bind(student_id)
                .fetch_optional(pool)
                .await?;

            match row {
                Some(row) => {
                    let enrollment = Enrollment {
                        student_id: row.get("student_id"),
                        academic_year: row.get("academic_year"),
                        grade_level: row.get("grade_level"),
                        teacher_id: row.get("teacher_id"),
                        status: row.get("status"),
                        enrollment_date: row.get("enrollment_date"),
                        status_change_date: row.get("status_change_date"),
                        notes: row.get("notes"),
                    };
                    Ok(Some(enrollment))
                }
                None => Ok(None),
            }
        }

        pub async fn add_enrollment(new_enrollment: &Enrollment, pool: &PgPool) -> Result<Enrollment, ServerFnError> {
            let row = sqlx::query("INSERT INTO student_enrollments (student_id, academic_year, grade_level, teacher_id, status, enrollment_date, status_change_date, notes) VALUES($1, $2::school_year_enum, $3::grade_enum, $4, $5::enrollment_status_enum, $6, $7, $8) RETURNING student_id, academic_year, grade_level, teacher_id, status, enrollment_date, status_change_date, notes")
                .bind(&new_enrollment.student_id)
                .bind(&new_enrollment.academic_year)
                .bind(&new_enrollment.grade_level)
                .bind(&new_enrollment.teacher_id)
                .bind(&new_enrollment.status)
                .bind(&new_enrollment.enrollment_date)
                .bind(&new_enrollment.status_change_date)
                .bind(&new_enrollment.notes)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let enrollment = Enrollment {
                student_id: row.get("student_id"),
                academic_year: row.get("academic_year"),
                grade_level: row.get("grade_level"),
                teacher_id: row.get("teacher_id"),
                status: row.get("status"),
                enrollment_date: row.get("enrollment_date"),
                status_change_date: row.get("status_change_date"),
                notes: row.get("notes"),
            };

            Ok(enrollment)
        }

        pub async fn update_enrollment_status(
            student_id: &i32,
            academic_year: &AcademicYear,
            new_status: EnrollmentStatus,
            status_change_date: NaiveDate,
            pool: &PgPool
        ) -> Result<Option<Enrollment>, ServerFnError> {
            let row = sqlx::query("UPDATE student_enrollments SET status = $3::enrollment_status_enum, status_change_date = $4 WHERE student_id = $1 AND academic_year = $2::school_year_enum RETURNING student_id, academic_year, grade_level, teacher_id, status, enrollment_date, status_change_date, notes")
                .bind(student_id)
                .bind(academic_year)
                .bind(new_status)
                .bind(status_change_date)
                .fetch_optional(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            match row {
                Some(row) => {
                    let enrollment = Enrollment {
                        student_id: row.get("student_id"),
                        academic_year: row.get("academic_year"),
                        grade_level: row.get("grade_level"),
                        teacher_id: row.get("teacher_id"),
                        status: row.get("status"),
                        enrollment_date: row.get("enrollment_date"),
                        status_change_date: row.get("status_change_date"),
                        notes: row.get("notes"),
                    };
                    Ok(Some(enrollment))
                }
                None => Ok(None),
            }
        }

        pub async fn update_enrollment(
            student_id: &i32,
            academic_year: &AcademicYear,
            grade_level: GradeEnum,
            teacher_id: i32,
            status: EnrollmentStatus,
            enrollment_date: NaiveDate,
            status_change_date: Option<NaiveDate>,
            notes: Option<String>,
            pool: &PgPool
        ) -> Result<Option<Enrollment>, ServerFnError> {
            let row = sqlx::query("UPDATE student_enrollments SET grade_level = $3::grade_enum, teacher_id = $4, status = $5::enrollment_status_enum, enrollment_date = $6, status_change_date = $7, notes = $8 WHERE student_id = $1 AND academic_year = $2::school_year_enum RETURNING student_id, academic_year, grade_level, teacher_id, status, enrollment_date, status_change_date, notes")
                .bind(student_id)
                .bind(academic_year)
                .bind(grade_level)
                .bind(teacher_id)
                .bind(status)
                .bind(enrollment_date)
                .bind(status_change_date)
                .bind(notes)
                .fetch_optional(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            match row {
                Some(row) => {
                    let enrollment = Enrollment {
                        student_id: row.get("student_id"),
                        academic_year: row.get("academic_year"),
                        grade_level: row.get("grade_level"),
                        teacher_id: row.get("teacher_id"),
                        status: row.get("status"),
                        enrollment_date: row.get("enrollment_date"),
                        status_change_date: row.get("status_change_date"),
                        notes: row.get("notes"),
                    };
                    Ok(Some(enrollment))
                }
                None => Ok(None),
            }
        }

        pub async fn delete_enrollment(
            student_id: i32,
            academic_year: AcademicYear,
            pool: &PgPool
        ) -> Result<Option<Enrollment>, ServerFnError> {
            let row = sqlx::query("DELETE FROM student_enrollments WHERE student_id = $1 AND academic_year = $2::school_year_enum RETURNING student_id, academic_year, grade_level, teacher_id, status, enrollment_date, status_change_date, notes")
                .bind(student_id)
                .bind(academic_year)
                .fetch_optional(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            match row {
                Some(row) => {
                    let enrollment = Enrollment {
                        student_id: row.get("student_id"),
                        academic_year: row.get("academic_year"),
                        grade_level: row.get("grade_level"),
                        teacher_id: row.get("teacher_id"),
                        status: row.get("status"),
                        enrollment_date: row.get("enrollment_date"),
                        status_change_date: row.get("status_change_date"),
                        notes: row.get("notes"),
                    };
                    Ok(Some(enrollment))
                }
                None => Ok(None),
            }
        }

        pub async fn bulk_insert_enrollments(pool: &PgPool, enrollments: &[Enrollment]) -> Result<usize, ServerFnError> {
            if enrollments.is_empty() {
                return Ok(0);
            }

            let mut tx = pool.begin().await?;

            let result = bulk_insert_with_unnest(enrollments, &mut tx).await;

            match result {
                Ok(count) => {
                    tx.commit().await?;
                    Ok(count)
                }
                Err(e) => {
                    tx.rollback().await?;
                    Err(ServerFnError::new(format!("Bulk insert failed: {}", e)))
                }
            }
        }

        pub async fn bulk_insert_enrollments_batch(pool: &PgPool, enrollments: &[Enrollment]) -> Result<usize, ServerFnError> {
            if enrollments.is_empty() {
                return Ok(0);
            }

            let mut tx = pool.begin().await?;
            let batch_size = 100;
            let mut total_inserted = 0;

            for chunk in enrollments.chunks(batch_size) {
                let count = insert_enrollment_batch(chunk, &mut tx).await?;
                total_inserted += count;
            }

            tx.commit().await?;
            Ok(total_inserted)
        }

        async fn bulk_insert_with_unnest(
            enrollments: &[Enrollment],
            tx: &mut sqlx::Transaction<'_, sqlx::Postgres>
        ) -> Result<usize, sqlx::Error> {
            let mut student_ids = Vec::new();
            let mut academic_years = Vec::new();
            let mut grade_levels = Vec::new();
            let mut teacher_ids = Vec::new();
            let mut statuses = Vec::new();
            let mut enrollment_dates = Vec::new();
            let mut status_change_dates = Vec::new();
            let mut notes = Vec::new();

            for enrollment in enrollments {
                student_ids.push(&enrollment.student_id);
                academic_years.push(enrollment.academic_year.to_string());
                grade_levels.push(enrollment.grade_level.to_string());
                teacher_ids.push(enrollment.teacher_id);
                statuses.push(enrollment.status.to_string());
                enrollment_dates.push(enrollment.enrollment_date);
                status_change_dates.push(enrollment.status_change_date);
                notes.push(enrollment.notes.as_ref().map(|s| s.as_str()));
            }

            let query = r#"
                INSERT INTO student_enrollments (
                    student_id, academic_year, grade_level, teacher_id, status, 
                    enrollment_date, status_change_date, notes
                )
                SELECT * FROM UNNEST(
                    $1::int4[], $2::school_year_enum[], $3::grade_enum[], $4::int4[], 
                    $5::enrollment_status_enum[], $6::date[], $7::date[], $8::text[]
                )
            "#;

            let result = sqlx::query(query)
                .bind(&student_ids)
                .bind(&academic_years)
                .bind(&grade_levels)
                .bind(&teacher_ids)
                .bind(&statuses)
                .bind(&enrollment_dates)
                .bind(&status_change_dates)
                .bind(&notes)
                .execute(&mut **tx)
                .await?;

            Ok(result.rows_affected() as usize)
        }

        async fn insert_enrollment_batch(
            enrollments: &[Enrollment],
            tx: &mut sqlx::Transaction<'_, sqlx::Postgres>
        ) -> Result<usize, sqlx::Error> {
            let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO student_enrollments (student_id, academic_year, grade_level, teacher_id, status, enrollment_date, status_change_date, notes) "
            );

            query_builder.push_values(enrollments, |mut b, enrollment| {
                b.push_bind(&enrollment.student_id)
                 .push_bind(&enrollment.academic_year)
                 .push_bind(&enrollment.grade_level)
                 .push_bind(enrollment.teacher_id)
                 .push_bind(&enrollment.status)
                 .push_bind(enrollment.enrollment_date)
                 .push_bind(&enrollment.status_change_date)
                 .push_bind(&enrollment.notes);
            });

            let query = query_builder.build();
            let result = query.execute(&mut **tx).await?;

            Ok(result.rows_affected() as usize)
        }

        pub async fn get_enrollment_counts_by_grade(
            academic_year: &AcademicYear,
            pool: &PgPool
        ) -> Result<Vec<(GradeEnum, i64)>, ServerFnError> {
            let rows = sqlx::query("SELECT grade_level, COUNT(*) as count FROM student_enrollments WHERE academic_year = $1 AND status = 'active' GROUP BY grade_level ORDER BY grade_level")
                .bind(academic_year)
                .fetch_all(pool)
                .await?;

            let counts: Vec<(GradeEnum, i64)> = rows
                .into_iter()
                .map(|row| {
                    let grade_level: GradeEnum = row.get("grade_level");
                    let count: i64 = row.get("count");
                    (grade_level, count)
                })
                .collect();

            Ok(counts)
        }

        pub async fn get_enrollment_counts_by_teacher(
            academic_year: &AcademicYear,
            pool: &PgPool
        ) -> Result<Vec<(i32, i64)>, ServerFnError> {
            let rows = sqlx::query("SELECT teacher_id, COUNT(*) as count FROM student_enrollments WHERE academic_year = $1 AND status = 'active' GROUP BY teacher_id ORDER BY teacher_id")
                .bind(academic_year)
                .fetch_all(pool)
                .await?;

            let counts: Vec<(i32, i64)> = rows
                .into_iter()
                .map(|row| {
                    let teacher_id: i32 = row.get("teacher_id");
                    let count: i64 = row.get("count");
                    (teacher_id, count)
                })
                .collect();

            Ok(counts)
        }

        pub async fn get_enrollment_status_summary(
            academic_year: &AcademicYear,
            pool: &PgPool
        ) -> Result<Vec<(EnrollmentStatus, i64)>, ServerFnError> {
            let rows = sqlx::query("SELECT status, COUNT(*) as count FROM student_enrollments WHERE academic_year = $1 GROUP BY status ORDER BY status")
                .bind(academic_year)
                .fetch_all(pool)
                .await?;

            let counts: Vec<(EnrollmentStatus, i64)> = rows
                .into_iter()
                .map(|row| {
                    let status: EnrollmentStatus = row.get("status");
                    let count: i64 = row.get("count");
                    (status, count)
                })
                .collect();

            Ok(counts)
        }
    }
}
