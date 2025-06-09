cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use sqlx::PgPool;
        use leptos::*;
        use crate::app::models::global::{GlobalSetting, SettingsCache};
        use sqlx::Row;

        pub async fn get_all_global_settings(pool: &sqlx::PgPool) -> Result<SettingsCache, ServerFnError> {
            let rows = sqlx::query("SELECT key, value, updated_by, updated_at FROM global_settings")
                .fetch_all(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database query failed: {}", e)))?;

            let settings = rows.into_iter().map(|row| {
                GlobalSetting {
                    key_name: row.get("key"),
                    value: row.get::<serde_json::Value, _>("value").to_string(),
                    updated_by: row.get("updated_by"),
                    updated_at: row.get("updated_at"),
                }
            }).collect();

            Ok(SettingsCache::from_settings(settings))
        }

        pub async fn get_global_settings(pool: &sqlx::PgPool) -> Result<Vec<GlobalSetting>, ServerFnError> {
            let rows = sqlx::query("SELECT key, value, updated_by, updated_at FROM global_settings")
                .fetch_all(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database query failed: {}", e)))?;

            let settings = rows.into_iter().map(|row| {
                GlobalSetting {
                    key_name: row.get("key"),
                    value: row.get::<serde_json::Value, _>("value").to_string(),
                    updated_by: row.get("updated_by"),
                    updated_at: row.get("updated_at"),
                }
            }).collect();

            Ok(settings)
        }

        pub async fn get_global_setting(pool: &sqlx::PgPool, key: &str) -> Result<Option<GlobalSetting>, ServerFnError> {
            let row = sqlx::query("SELECT key, value, updated_by, updated_at FROM global_settings WHERE key = $1")
                .bind(key)
                .fetch_optional(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database query failed: {}", e)))?;

            let result = row.map(|r| GlobalSetting {
                key_name: r.get("key"),
                value: r.get::<serde_json::Value, _>("value").to_string(),
                updated_by: r.get("updated_by"),
                updated_at: r.get("updated_at"),
            });

            Ok(result)
        }

        pub async fn update_global_setting(pool: &sqlx::PgPool, key: &str, value: serde_json::Value, updated_by: i32) -> Result<(), ServerFnError> {
            sqlx::query(
                r#"
                INSERT INTO global_settings (key, value, updated_by, updated_at)
                VALUES ($1, $2, $3, CURRENT_TIMESTAMP)
                ON CONFLICT (key) 
                DO UPDATE SET 
                    value = EXCLUDED.value,
                    updated_by = EXCLUDED.updated_by,
                    updated_at = CURRENT_TIMESTAMP
                "#
            )
            .bind(key)
            .bind(value)
            .bind(updated_by)
            .execute(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database update failed: {}", e)))?;

            Ok(())
        }
    }
}
