use crate::app::db::user_database;
use crate::app::models::user::{User, UserRole};
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

#[server(GetUser, "/api")]
pub async fn get_user(id: i64) -> Result<User, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        use sqlx::PgPool;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Fetching user with ID {} from the database", id);

        match user_database::get_user(id, &pool).await {
            Ok(user) => {
                log::info!(
                    "Successfully retrieved user with ID {} from the database",
                    id
                );
                Ok(user)
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server(UpdateUserPermissions, "/api")]
pub async fn update_user_permissions(user_id: i64, role: UserRole) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        use sqlx::PgPool;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Updating user permissions in the database");

        match user_database::update_permissions(user_id, role, &pool).await {
            Ok(_) => {
                log::info!("Successfully updated user permissions");
                Ok(())
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server(UpdateUser, "/api")]
pub async fn update_user(new_user_data: User) -> Result<User, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        use sqlx::PgPool;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool")))?;

        /*log::info!(
            "Updating data for user: {} {}",
            new_user_data.first_name.unwrap_or("None".to_string()),
            new_user_data.last_name.unwrap_or("None".to_string())
        );*/

        match user_database::update_user_data(new_user_data, &pool).await {
            Ok(user) => {
                log::info!("Successfully updated user data");
                Ok(user)
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}
