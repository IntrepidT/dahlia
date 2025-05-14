cfg_if::cfg_if! {

    if #[cfg(feature = "ssr")] {

        use crate::app::models::{Score, CreateScoreRequest};
        use chrono::{Local, DateTime, Utc, NaiveDateTime};
        use leptos::*;
        use uuid::Uuid;
        use log::{debug, error, info, warn};
        use sqlx::prelude::*;
        use sqlx::PgPool;

        pub async fn get_all_scores(pool: &PgPool) -> Result<Vec<Score>, ServerFnError> {
            let row = sqlx::query("SELECT student_id, date_administered, test_id::text, test_scores, comments, test_variant, evaluator, attempt FROM scores ORDER BY date_administered DESC")
                .fetch_all(pool)
                .await?;


            let scores: Vec<Score> = row
                .into_iter()
                .map(|row| {

                    let naive_datetime: NaiveDateTime = row.get("date_administered");

                    let student_id: i32 =  row.get("student_id");
                    let date_administered: DateTime<Utc> = DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc);
                   let test_id: String = row.get("test_id");
                   let test_scores: Vec<i32> = row.get("test_scores");
                   let comments: Vec<String> = row.get("comments");
                   let test_variant: i32 = row.get("test_variant");
                   let evaluator: String = row.get("evaluator");
                   let attempt: i32 = row.get("attempt");

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
                })
                .collect();
            Ok(scores)
        }

        pub async fn get_scores_by_test(test_ids: Vec<Uuid>, pool: &PgPool) -> Result<Vec<Score>, ServerFnError> {
            let row = sqlx::query("SELECT student_id, date_administered, test_id::text, test_scores, comments, test_variant, evaluator, attempt FROM scores WHERE test_id = ANY($1) ORDER BY date_administered DESC")
                .bind(&test_ids)
                .fetch_all(pool)
                .await?;

            let scores: Vec<Score> = row
                .into_iter()
                .map(|row| {

                    let naive_datetime: NaiveDateTime = row.get("date_administered");

                    let student_id: i32 =  row.get("student_id");
                    let date_administered: DateTime<Utc> = DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc);
                    let test_id: String = row.get("test_id");
                    let test_scores: Vec<i32> = row.get("test_scores");
                    let comments: Vec<String> = row.get("comments");
                    let test_variant: i32 = row.get("test_variant");
                    let evaluator: String = row.get("evaluator");
                    let attempt: i32 = row.get("attempt");

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
                })
                .collect();
            Ok(scores)
        }

        pub async fn get_score(student_id: i32, test_id: String, test_variant: i32, attempt: i32, pool: &PgPool)-> Result<Score, ServerFnError> {
            let ID = Uuid::parse_str(&test_id).expect("Invalid UUID format");

            let row = sqlx::query("SELECT student_id, date_administered, test_id::text, test_scores, comments, test_variant, evaluator, attempt FROM scores WHERE student_id = $1 AND test_id = $2 AND test_variant = $3 AND attempt = $4").bind(&student_id).bind(ID).bind(&test_variant).bind(&attempt).fetch_one(pool)
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
                attempt: row.get("attempt"),
            };

            Ok(score)
        }

        pub async fn get_all_student_scores(student_id: i32, pool: &PgPool) -> Result<Vec<Score>, ServerFnError> {
            let row = sqlx::query("SELECT student_id, date_administered, test_id::text, test_scores, comments, test_variant, evaluator, attempt FROM scores WHERE student_id = $1 ORDER BY date_administered DESC")
                .bind(&student_id)
                .fetch_all(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;


            let scores: Vec<Score> = row
                .into_iter()
                .map(|row| {

                    let naive_datetime: NaiveDateTime = row.get("date_administered");

                    let student_id: i32 =  row.get("student_id");
                    let date_administered: DateTime<Utc> = DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc);
                    let test_id: String = row.get("test_id");
                    let test_scores: Vec<i32> = row.get("test_scores");
                    let comments: Vec<String> = row.get("comments");
                    let test_variant: i32 = row.get("test_variant");
                    let evaluator: String = row.get("evaluator");
                    let attempt: i32 = row.get("attempt");

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
                })
                .collect();
            Ok(scores)
        }

        pub async fn add_score(new_score_request: &CreateScoreRequest, pool: &sqlx::PgPool) -> Result<Score, ServerFnError> {
            let ID = Uuid::parse_str(&new_score_request.test_id).expect("Invalid UUID format");
            let timestamp = Local::now();
            let row = sqlx::query("INSERT INTO scores (student_id, date_administered, test_id, test_scores, comments, test_variant, evaluator, attempt) VALUES($1, $2, $3, $4, $5, $6, $7, next_attempt_number($1, $3, $6)) RETURNING student_id, date_administered, test_id::text, test_scores, comments, test_variant, evaluator, attempt")
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
                attempt: row.get("attempt"),
            };

            Ok(score)
        }

        pub async fn delete_score(student_id: i32, test_id: String, test_variant: i32, attempt: i32, pool: &sqlx::PgPool) -> Result<Score, ServerFnError> {
            let ID = Uuid::parse_str(&test_id).expect("Invalid UUID format");

            let row = sqlx::query("DELETE FROM scores WHERE student_id = $1 AND test_id = $2 AND test_variant = $3 AND attempt = $4 RETURNING student_id, date_administered, test_id::text, test_scores, comments, test_variant, evaluator, attempt")
                .bind(&student_id)
                .bind(ID)
                .bind(&test_variant)
                .bind(&attempt)
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
                attempt: row.get("attempt"),
            };

            Ok(deleted_score)

        }
    }
}
