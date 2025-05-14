use crate::app::models::score::*;
use leptos::*;
use uuid::Uuid;

#[cfg(feature = "ssr")]
use {
    crate::app::db::database, crate::app::db::score_database, actix_web::web, sqlx::PgPool,
    std::error::Error,
};

#[server(GetScores, "/api")]
pub async fn get_scores() -> Result<Vec<Score>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to retrieve all scores from database");
        use crate::app::db::score_database;

        match score_database::get_all_scores(&pool).await {
            Ok(scores) => {
                log::info!(
                    "Successfully retrieved {} scores from database",
                    scores.len()
                );
                Ok(scores)
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server(GetScoresByTest, "/api")]
pub async fn get_scores_by_test(test_ids: Vec<Uuid>) -> Result<Vec<Score>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to get scores based upon test IDs");

        match score_database::get_scores_by_test(test_ids, &pool).await {
            Ok(scores) => {
                log::info!("Successfully retrieved scores from database");
                Ok(scores)
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server(GetScore, "/api")]
pub async fn get_score(
    student_id: i32,
    test_id: String,
    test_variant: i32,
    attempt: i32,
) -> Result<Score, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to retrieve all scores from database");

        match score_database::get_score(student_id, test_id, test_variant, attempt, &pool).await {
            Ok(score) => {
                log::info!("Successfully retrieved score from database");
                Ok(score)
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server(GetStudentScores, "/api")]
pub async fn get_student_scores(student_id: i32) -> Result<Vec<Score>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!(
            "Attempting to retrieve all scores for student: {}",
            student_id
        );

        match score_database::get_all_student_scores(student_id, &pool).await {
            Ok(scores) => {
                log::info!(
                    "Successfully retrieved scores from database for student: {}",
                    student_id
                );
                Ok(scores)
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server(AddScore, "/api")]
pub async fn add_score(add_score_request: CreateScoreRequest) -> Result<Score, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to add new score to the database");

        match score_database::add_score(&add_score_request, &pool).await {
            Ok(created_score) => {
                log::info!(
                    "Successfully created score for student {}",
                    created_score.student_id
                );
                Ok(created_score)
            }
            Err(e) => {
                log::info!("Failed to create question: {:?}", e);
                Err(ServerFnError::new(format!(
                    "The score created was not saved correctly"
                )))
            }
        }
    }
}

#[server(DeleteScore, "/api")]
pub async fn delete_score(
    delete_score_request: DeleteScoreRequest,
) -> Result<Score, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to delete score from the database");

        match score_database::delete_score(
            delete_score_request.student_id,
            delete_score_request.test_id,
            delete_score_request.test_variant,
            delete_score_request.attempt,
            &pool,
        )
        .await
        {
            Ok(deleted) => Ok(deleted),
            Err(_) => Err(ServerFnError::new(
                "Failed to delete score from the database",
            )),
        }
    }
}
/*
#[server(EditQuestion, "/api")]
pub async fn edit_question(
    test_id: String,
    edit_question_request: UpdateQuestionRequest,
) -> Result<Question, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to update question from the database");

        let buffer_question = Question::new(
            edit_question_request.word_problem,
            edit_question_request.point_value,
            edit_question_request.question_type,
            edit_question_request.options,
            edit_question_request.correct_answer,
            edit_question_request.qnumber,
            edit_question_request.testlinker,
        );

        match database::update_question(&buffer_question, &pool).await {
            Ok(Some(updated_student)) => Ok(updated_student),
            Ok(None) => Err(ServerFnError::new(format!(
                "Failed to correctly existing student in the database"
            ))),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to update student: {}",
                e
            ))),
        }
    }
}
*/
