cfg_if::cfg_if! {

    if #[cfg(feature = "ssr")] {

        use crate::app::models::{Score, CreateScoreRequest};
        use chrono::{Local, DateTime, Utc, NaiveDateTime};
        use leptos::*;
        use uuid::Uuid;
        use log::{debug, error, info, warn};
        use sqlx::prelude::*;
        use sqlx::PgPool;

        pub async fn get_score(student_id: i32, test_id: String, test_variant: i32, pool: &PgPool)-> Result<Score, ServerFnError> {
            let ID = Uuid::parse_str(&test_id).expect("Invalid UUID format");

            let row = sqlx::query("SELECT student_id, date_administered, test_id::text, test_scores, comments, test_variant, evaluator FROM scores WHERE student_id = $1 AND test_id = $2 AND test_variant = $3").bind(&student_id).bind(ID).bind(&test_variant).fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let naive_datetime: NaiveDateTime = row.get("date_administered");



            let score = Score {
                student_id: row.get("student_id"),
                date_administered: DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc),
                test_id: row.get("test_id"),
                test_scores: row.get("test_scores"),
                comments: row.get("comments"),
                test_variant: row.get("test_variant"),
                evaluator: row.get("evaluator"),
            };

            Ok(score)
        }

        pub async fn add_score(new_score_request: &CreateScoreRequest, pool: &sqlx::PgPool) -> Result<Score, ServerFnError> {
            let ID = Uuid::parse_str(&new_score_request.test_id).expect("Invalid UUID format");
            let timestamp = Local::now();
            let row = sqlx::query("INSERT INTO scores (student_id, date_administered, test_id, test_scores, comments, test_variant, evaluator) VALUES($1, $2, $3, $4, $5, $6, $7) RETURNING student_id, date_administered, test_id::text, test_scores, comments, test_variant, evaluator")
                .bind(&new_score_request.student_id)
                .bind(timestamp)
                .bind(ID)
                .bind(&new_score_request.test_scores)
                .bind(&new_score_request.comments)
                .bind(&new_score_request.test_variant)
                .bind(&new_score_request.evaluator)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let naive_datetime: NaiveDateTime = row.get("date_administered");

            let score = Score {
                student_id: row.get("student_id"),
                date_administered: DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc),
                test_id: row.get("test_id"),
                test_scores: row.get("test_scores"),
                comments: row.get("comments"),
                test_variant: row.get("test_variant"),
                evaluator: row.get("evaluator"),
            };

            Ok(score)
        }

        pub async fn delete_score(student_id: i32, test_id: String, test_variant: i32, pool: &sqlx::PgPool) -> Result<Score, ServerFnError> {
            let ID = Uuid::parse_str(&test_id).expect("Invalid UUID format");

            let row = sqlx::query("DELETE FROM scores WHERE student_id = $1 AND test_id = $2 AND test_variant = $3 RETURNING student_id, date_administered, test_id::text, test_scores, comments, test_variant, evaluator")
                .bind(&student_id)
                .bind(ID)
                .bind(&test_variant)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let naive_datetime: NaiveDateTime = row.get("date_administered");

            let deleted_score = Score {
                student_id: row.get("student_id"),
                date_administered: DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc),
                test_id: row.get("test_id"),
                test_scores: row.get("test_scores"),
                comments: row.get("comments"),
                test_variant: row.get("test_variant"),
                evaluator: row.get("evaluator"),
            };

            Ok(deleted_score)

        }
    }
}
