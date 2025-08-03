use crate::app::models::assessment::RangeCategory;
use crate::app::models::assessment::SubjectEnum;
use crate::app::models::assessment::{
    Assessment, CreateNewAssessmentRequest, DeleteAssessmentRequest, UpdateAssessmentRequest,
};
use leptos::*;
#[cfg(feature = "ssr")]
use {
    crate::app::db::assessment_database, crate::app::db::database, actix_web::web, chrono::Local,
    sqlx::PgPool, std::error::Error, uuid::Uuid,
};

//this file contains a list of api functions that will be called on the server side
//lowercase functions denote functions that are server side while upper/camel case functions
//indicate Client side Objects/functions

#[server(GetAssessments, "/api")]
pub async fn get_assessments() -> Result<Vec<Assessment>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to retrieve all assessments from database");

        match assessment_database::get_all_assessments(&pool).await {
            Ok(assessments) => {
                log::info!("Successfully retrieved all assessments from database");
                Ok(assessments)
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new(
            "Server function called in client context",
        ))
    }
}

#[server(GetAssessment, "/api")]
pub async fn get_assessment(id: String) -> Result<Assessment, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to retrieve assessment");

        match assessment_database::get_assessment(id, &pool).await {
            Ok(assessment) => {
                log::info!("Successfully retrieved assessment from database");
                Ok(assessment)
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new(
            "Server function called in client context",
        ))
    }
}

#[server(AddAssessment, "/api")]
pub async fn add_assessment(
    add_assessment_request: CreateNewAssessmentRequest,
) -> Result<Assessment, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        use uuid::Uuid;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to add new assessment to the database");

        let id = Uuid::new_v4();

        let buffer_assessment = if add_assessment_request.test_sequence.is_some() {
            Assessment::new_with_sequence(
                add_assessment_request.name,
                add_assessment_request.frequency,
                add_assessment_request.grade,
                add_assessment_request.version,
                id,
                add_assessment_request.composite_score,
                add_assessment_request.risk_benchmarks,
                add_assessment_request.national_benchmarks,
                add_assessment_request.subject,
                add_assessment_request.scope,
                add_assessment_request.course_id,
                add_assessment_request.test_sequence.unwrap(),
            )
        } else {
            Assessment::new(
                add_assessment_request.name,
                add_assessment_request.frequency,
                add_assessment_request.grade,
                add_assessment_request.version,
                id,
                add_assessment_request.tests,
                add_assessment_request.composite_score,
                add_assessment_request.risk_benchmarks,
                add_assessment_request.national_benchmarks,
                add_assessment_request.subject,
                add_assessment_request.scope,
                add_assessment_request.course_id,
            )
        };

        assessment_database::add_assessment(&buffer_assessment, &pool)
            .await
            .map_err(|e| {
                log::error!("Database error while adding assessment: {}", e);
                ServerFnError::new(format!("Database error: {}", e))
            })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new(
            "Server function called in client context",
        ))
    }
}

#[server(DeleteAssessment, "/api")]
pub async fn delete_assessment(
    delete_assessment_request: DeleteAssessmentRequest,
) -> Result<Assessment, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to delete assessment");

        match assessment_database::delete_assessment(
            delete_assessment_request.id.to_string(),
            &pool,
        )
        .await
        {
            Ok(deleted) => Ok(deleted),
            Err(e) => Err(ServerFnError::new(format!(
                "Error in deleting assessment: {}",
                e
            ))),
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new(
            "Server function called in client context",
        ))
    }
}

#[server(UpdateAssessmentScore, "/api")]
pub async fn update_assessment_score(test_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        match assessment_database::update_all_assessments_referencing_test(&test_id, &pool).await {
            Ok(()) => Ok(()),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to update assessment scores: {}",
                e
            ))),
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new(
            "Server function called in client context",
        ))
    }
}

#[server(EditAssessment, "/api")]
pub async fn update_assessment(
    update_assessment_request: UpdateAssessmentRequest,
) -> Result<Assessment, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to update assessment");

        let buffer_assessment = if update_assessment_request.test_sequence.is_some() {
            Assessment::new_with_sequence(
                update_assessment_request.name,
                update_assessment_request.frequency,
                update_assessment_request.grade,
                update_assessment_request.version,
                update_assessment_request.id,
                update_assessment_request.composite_score,
                update_assessment_request.risk_benchmarks,
                update_assessment_request.national_benchmarks,
                update_assessment_request.subject,
                update_assessment_request.scope,
                update_assessment_request.course_id,
                update_assessment_request.test_sequence.unwrap(),
            )
        } else {
            Assessment::new(
                update_assessment_request.name,
                update_assessment_request.frequency,
                update_assessment_request.grade,
                update_assessment_request.version,
                update_assessment_request.id,
                update_assessment_request.tests,
                update_assessment_request.composite_score,
                update_assessment_request.risk_benchmarks,
                update_assessment_request.national_benchmarks,
                update_assessment_request.subject,
                update_assessment_request.scope,
                update_assessment_request.course_id,
            )
        };

        match assessment_database::update_assessment(&buffer_assessment, &pool).await {
            Ok(updated_assessment) => Ok(updated_assessment),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to update assessment: {}",
                e
            ))),
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new(
            "Server function called in client context",
        ))
    }
}

#[server(GetTestSequence, "/api")]
pub async fn get_test_sequence(
    assessment_id: String,
) -> Result<Vec<(String, String)>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to retrieve test sequence for assessment");

        match assessment_database::get_test_sequence(&assessment_id, &pool).await {
            Ok(test_sequence) => Ok(test_sequence),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to retrieve test sequence: {}",
                e
            ))),
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new(
            "Server function called in client context",
        ))
    }
}
