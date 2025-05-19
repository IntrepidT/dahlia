use crate::app::db::user_database;
use crate::app::models::user::User;
use leptos::*;

#[server(GetUsers, "/api")]
pub async fn get_users() -> Result<Vec<User>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        use sqlx::PgPool;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Fetching users from the database");

        match user_database::get_all_users(&pool).await {
            Ok(users) => {
                log::info!("Successfully retrieved all users from the database");
                Ok(users)
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}
