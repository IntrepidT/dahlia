use crate::app::models::assessment::RangeCategory;
use crate::app::models::student::GradeEnum;
use leptos::ServerFnError;
use uuid::Uuid;
cfg_if::cfg_if! {

    if #[cfg(feature = "ssr")]{
        use crate::app::models::assessment::{Assessment, SubjectEnum, RangeCategoriesWrapper};
        use sqlx::PgPool;
        use sqlx::types::Json;
        use leptos::*;
        use sqlx::prelude::*;

        pub async fn get_all_assessments(pool: &sqlx::PgPool) -> Result<Vec<Assessment>, ServerFnError> {
            let rows = sqlx::query("SELECT name, frequency, grade, version, id, tests, composite_score, risk_benchmarks, national_benchmarks, subject FROM assessments ORDER BY name ASC")
                .fetch_all(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let assessments: Vec<Assessment> = rows
                .into_iter()
                .map(|row| {
                    let name: String = row.get("name");
                    let frequency: Option<i32> = row.get("frequency");
                    let grade: Option<GradeEnum> = row.get("grade");
                    let version: i32 = row.get("version");
                    let id: Uuid = row.get("id");
                    let tests: Vec<Uuid> = row.get("tests");
                    let composite_score: Option<i32> = row.get("composite_score");

                    let risk_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Option<Json<Vec<RangeCategory>>>, _>("risk_benchmarks") {
                        Ok(Some(json)) => Some(json.0),
                        _ => None,
                    };
                    let national_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Option<Json<Vec<RangeCategory>>>, _>("national_benchmarks") {
                        Ok(Some(json)) => Some(json.0),
                        _ => None,
                    };
                    let subject: SubjectEnum = row.get("subject");

                    Assessment {
                        name,
                        frequency,
                        grade,
                        version,
                        id,
                        tests,
                        composite_score,
                        risk_benchmarks,
                        national_benchmarks,
                        subject,
                    }
                })
                .collect();

            Ok(assessments)
        }

        pub async fn get_assessment(id: String, pool: &sqlx::PgPool) -> Result<Assessment, ServerFnError> {
            let uuid = Uuid::parse_str(&id)
                .map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

            let row = sqlx::query("SELECT name, frequency, grade, version, id, tests, composite_score, risk_benchmarks, national_benchmarks, subject FROM assessments WHERE id = $1")
                .bind(&uuid)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let risk_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Option<Json<Vec<RangeCategory>>>, _>("risk_benchmarks") {
                Ok(Some(json)) => Some(json.0),
                _ => None,
            };

            let national_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Option<Json<Vec<RangeCategory>>>, _>("national_benchmarks") {
                Ok(Some(json)) => Some(json.0),
                _ => None,
            };


            let assessment = Assessment {
                name: row.get("name"),
                frequency: row.get("frequency"),
                grade: row.get("grade"),
                version: row.get("version"),
                id: row.get("id"),
                tests: row.get("tests"),
                composite_score: row.get("composite_score"),
                risk_benchmarks,
                national_benchmarks,
                subject: row.get("subject"),
            };

            Ok(assessment)
        }


        pub async fn add_assessment(new_assessment: &Assessment, pool: &PgPool) -> Result<Assessment, ServerFnError> {
            let risk_benchmarks = match &new_assessment.risk_benchmarks {
                Some(categories) => Json(categories.clone()),
                None => Json(Vec::new()),
            };
            let national_benchmarks = match &new_assessment.national_benchmarks {
                Some(categories) => Json(categories.clone()),
                None => Json(Vec::new()),
            };

            let row = sqlx::query("INSERT INTO assessments (name, frequency, grade, version, id, tests, composite_score, risk_benchmarks, national_benchmarks, subject) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING name, frequency, grade, version, id, tests, composite_score, risk_benchmarks, national_benchmarks, subject")
                .bind(&new_assessment.name)
                .bind(&new_assessment.frequency)
                .bind(&new_assessment.grade)
                .bind(&new_assessment.version)
                .bind(&new_assessment.id)
                .bind(&new_assessment.tests)
                .bind(&new_assessment.composite_score)
                .bind(&risk_benchmarks)
                .bind(&national_benchmarks)
                .bind(&new_assessment.subject)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let risk_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Option<Json<Vec<RangeCategory>>>, _>("risk_benchmarks") {
                Ok(Some(json)) => Some(json.0),
                _ => None,
            };
            let national_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Option<Json<Vec<RangeCategory>>>, _>("national_benchmarks") {
                Ok(Some(json)) => Some(json.0),
                _ => None,
            };

            let id: Uuid = row.get("id");

            let assessment = Assessment {
                name: row.get("name"),
                frequency: row.get("frequency"),
                grade: row.get("grade"),
                version: row.get("version"),
                id,
                tests: row.get("tests"),
                composite_score: row.get("composite_score"),
                risk_benchmarks,
                national_benchmarks,
                subject: row.get("subject"),
            };
            Ok(assessment)
        }

        pub async fn update_assessment(assessment: &Assessment, pool: &sqlx::PgPool) -> Result<Assessment, ServerFnError> {
            let risk_benchmarks = match &assessment.risk_benchmarks {
                Some(categories) => Json(categories.clone()),
                None => Json(Vec::new()),
            };
            let national_benchmarks = match &assessment.national_benchmarks {
                Some(categories) => Json(categories.clone()),
                None => Json(Vec::new()),
            };

            let row = sqlx::query("UPDATE assessments SET name = $1, frequency = $2, grade = $3, version = $4, tests = $5, composite_score = $6, risk_benchmarks = $7, national_benchmarks = $8, subject = $9 WHERE id = $10 RETURNING name, frequency, grade, version, id, tests, composite_score, risk_benchmarks, national_benchmarks, subject")
                .bind(&assessment.name)
                .bind(&assessment.frequency)
                .bind(&assessment.grade)
                .bind(&assessment.version)
                .bind(&assessment.tests)
                .bind(&assessment.composite_score)
                .bind(&risk_benchmarks)
                .bind(&national_benchmarks)
                .bind(&assessment.subject)
                .bind(&assessment.id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let risk_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Option<Json<Vec<RangeCategory>>>, _>("risk_benchmarks") {
                Ok(Some(json)) => Some(json.0),
                _ => None,
            };
            let national_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Option<Json<Vec<RangeCategory>>>, _>("national_benchmarks") {
                Ok(Some(json)) => Some(json.0),
                _ => None,
            };

            let id: Uuid = row.get("id");

            let assessment = Assessment {
                name: row.get("name"),
                frequency: row.get("frequency"),
                grade: row.get("grade"),
                version: row.get("version"),
                id,
                tests: row.get("tests"),
                composite_score: row.get("composite_score"),
                risk_benchmarks,
                national_benchmarks,
                subject: row.get("subject"),
            };

            Ok(assessment)
        }


        pub async fn delete_assessment(id: String, pool: &sqlx::PgPool) -> Result<Assessment, ServerFnError> {
            let uuid = Uuid::parse_str(&id)
                .map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

            let row = sqlx::query("DELETE from assessments WHERE id = $1 RETURNING name, frequency, grade, version, id, tests, composite_score, risk_benchmarks, national_benchmarks, subject")
                .bind(&uuid)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let risk_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Option<Json<Vec<RangeCategory>>>, _>("risk_benchmarks") {
                Ok(Some(json)) => Some(json.0),
                _ => None,
            };
            let national_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Option<Json<Vec<RangeCategory>>>, _>("national_benchmarks") {
                Ok(Some(json)) => Some(json.0),
                _ => None,
            };

            let id: Uuid = row.get("id");

            let assessment = Assessment {
                name: row.get("name"),
                frequency: row.get("frequency"),
                grade: row.get("grade"),
                version: row.get("version"),
                id,
                tests: row.get("tests"),
                composite_score: row.get("composite_score"),
                risk_benchmarks,
                national_benchmarks,
                subject: row.get("subject"),
            };

            Ok(assessment)
        }
    }
}
