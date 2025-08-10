use crate::app::db::user_database;
use crate::app::models::setting_data::{UserSettings, UserSettingsUpdate};
use leptos::prelude::*;

#[server]
pub async fn get_user_settings(user_id: i64) -> Result<UserSettings, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        use sqlx::PgPool;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Fetching settings for user ID {}", user_id);

        match user_database::get_user_settings(&pool, user_id).await {
            Ok(settings) => {
                log::info!(
                    "Successfully retrieved user settings for user ID {}",
                    user_id
                );
                Ok(settings)
            }
            Err(e) => {
                log::error!("Database error getting user settings: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server]
pub async fn update_user_settings(
    user_id: i64,
    settings_update: UserSettingsUpdate,
) -> Result<UserSettings, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        use sqlx::PgPool;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Updating settings for user ID {}", user_id);

        match user_database::update_user_settings(&pool, user_id, settings_update).await {
            Ok(settings) => {
                log::info!("Successfully updated user settings for user ID {}", user_id);
                Ok(settings)
            }
            Err(e) => {
                log::error!("Database error updating user settings: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server]
pub async fn reset_user_settings(user_id: i64) -> Result<UserSettings, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        use sqlx::PgPool;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Resetting settings for user ID {}", user_id);

        match user_database::reset_user_settings(&pool, user_id).await {
            Ok(settings) => {
                log::info!("Successfully reset user settings for user ID {}", user_id);
                Ok(settings)
            }
            Err(e) => {
                log::error!("Database error resetting user settings: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

// Convenience functions for updating specific settings
#[server]
pub async fn update_dark_mode(
    user_id: i64,
    dark_mode: bool,
) -> Result<UserSettings, ServerFnError> {
    let settings_update = UserSettingsUpdate {
        ui: Some(crate::app::models::setting_data::UiSettingsUpdate {
            dark_mode: Some(dark_mode),
            pinned_sidebar: None,
        }),
    };

    update_user_settings(user_id, settings_update).await
}

#[server]
pub async fn update_pinned_sidebar(
    user_id: i64,
    pinned_sidebar: bool,
) -> Result<UserSettings, ServerFnError> {
    let settings_update = UserSettingsUpdate {
        ui: Some(crate::app::models::setting_data::UiSettingsUpdate {
            dark_mode: None,
            pinned_sidebar: Some(pinned_sidebar),
        }),
    };

    update_user_settings(user_id, settings_update).await
}
