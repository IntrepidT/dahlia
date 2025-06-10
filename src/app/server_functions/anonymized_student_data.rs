use leptos::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::PgPool;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StudentValidationRequest {
    pub app_ids: Vec<i32>, // Changed from u32 to i32 for consistency
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StudentValidationResponse {
    pub valid_app_ids: Vec<i32>,
    pub invalid_app_ids: Vec<i32>,
    pub success: bool,
    pub message: String,
}

#[server(ValidateStudentIds, "/api")]
pub async fn validate_student_ids(
    request: StudentValidationRequest,
) -> Result<StudentValidationResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::server_functions::auth::get_current_user;
        use actix_web::web;
        use leptos_actix::extract;

        // Ensure user is authenticated
        let user = get_current_user().await?;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Database connection error: {}", e)))?;

        // Check which app_ids exist in the database and belong to this user/session
        let mut valid_app_ids = Vec::new();
        let mut invalid_app_ids = Vec::new();

        for app_id in request.app_ids {
            // Adjust this query based on your actual database schema
            let exists = sqlx::query!(
                "SELECT COUNT(*) as count FROM students 
                 WHERE student_id = $1 AND (user_id = $2 OR session_user_id = $2)",
                app_id,
                user.id as i32
            )
            .fetch_one(&**pool)
            .await?;

            if exists.count.unwrap_or(0) > 0 {
                valid_app_ids.push(app_id);
            } else {
                invalid_app_ids.push(app_id);
            }
        }

        Ok(StudentValidationResponse {
            valid_app_ids,
            invalid_app_ids,
            success: true,
            message: format!(
                "Validated {} app_ids: {} valid, {} invalid",
                request.app_ids.len(),
                valid_app_ids.len(),
                invalid_app_ids.len()
            ),
        })
    }
    #[cfg(not(feature = "ssr"))]
    {
        // Client-side fallback
        Ok(StudentValidationResponse {
            valid_app_ids: vec![],
            invalid_app_ids: vec![],
            success: false,
            message: "This function can only be called server-side".to_string(),
        })
    }
}

// Helper function to get student data with de-anonymization
#[server(GetStudentData, "/api")]
pub async fn get_student_data(
    app_ids: Vec<i32>, // Changed from u32 to i32 for consistency
) -> Result<Vec<StudentRecord>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::server_functions::auth::get_current_user;
        use actix_web::web;
        use leptos_actix::extract;

        let user = get_current_user().await?;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Database connection error: {}", e)))?;

        // Fetch anonymized student records
        let records = sqlx::query_as!(
            StudentRecord,
            "SELECT student_id as app_id  
             FROM students 
             WHERE student_id = ANY($1) AND (user_id = $2 OR session_user_id = $2)",
            &app_ids,
            user.id as i32
        )
        .fetch_all(&**pool)
        .await?;

        Ok(records)
    }
    #[cfg(not(feature = "ssr"))]
    {
        // Client-side fallback
        Ok(vec![])
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StudentRecord {
    pub app_id: i32, // Changed from u32 to i32 for consistency
}
