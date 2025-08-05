use crate::app::db::global_database;
use crate::app::models::global::{GlobalSetting, SettingsCache};
use leptos::*;
#[cfg(feature = "ssr")]
use sqlx::PgPool;
use std::collections::HashMap;
use std::env;

#[server(GetGlobalSettings, "/api")]
pub async fn get_global_setting() -> Result<Vec<GlobalSetting>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        match global_database::get_global_settings(&pool).await {
            Ok(settings) => Ok(settings),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to get global settings: {}",
                e
            ))),
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[server(GetAllGlobalSettings, "/api")]
pub async fn get_global_settings() -> Result<SettingsCache, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        match global_database::get_all_global_settings(&pool).await {
            Ok(settings) => Ok(settings),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to get all global settings: {}",
                e
            ))),
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[server(GetSingleGlobalSetting, "/api")]
pub async fn get_single_global_setting(
    key: String,
) -> Result<Option<GlobalSetting>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        match global_database::get_global_setting(&pool, &key).await {
            Ok(setting) => Ok(setting),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to get global setting: {}",
                e
            ))),
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[server(UpdateGlobalSetting, "/api")]
pub async fn update_global_setting_api(
    key: String,
    value: serde_json::Value,
) -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::server_functions::auth::get_current_user;
        use actix_web::web;
        use leptos_actix::extract;

        // Check if user is authenticated and is admin
        let current_user = get_current_user().await?;
        if let Some(user) = current_user {
            if !user.is_admin() {
                return Err(ServerFnError::new(
                    "Unauthorized: Admin access required".to_string(),
                ));
            }

            let pool = extract::<web::Data<PgPool>>()
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

            global_database::update_global_setting(&pool, &key, value, user.id.try_into().unwrap())
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            Ok(true)
        } else {
            Err(ServerFnError::new(
                "Unauthorized: Login required".to_string(),
            ))
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[server(ToggleStudentProtection, "/api")]
pub async fn toggle_student_protection(
    enable: bool,
    mapping_key: Option<String>,
) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::server_functions::auth::get_current_user;
        use actix_web::web;
        use leptos_actix::extract;
        use std::path::Path;
        use std::process::Command;

        let current_user = get_current_user().await?;
        if let Some(user) = current_user {
            if !user.is_super_admin() {
                return Err(ServerFnError::new(
                    "Unauthorized: Super admin access required".to_string(),
                ));
            }

            let pool = extract::<web::Data<PgPool>>()
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

            // Get DATABASE_URL from environment
            let database_url = env::var("DATABASE_URL").map_err(|_| {
                ServerFnError::new("DATABASE_URL environment variable not set".to_string())
            })?;

            if enable {
                // Check if script file exists
                let script_paths = vec![
                    "./scripts/remove_pii_data.sql",
                    "/app/scripts/remove_pii_data.sql",
                    "scripts/remove_pii_data.sql",
                ];

                let script_path = script_paths
                    .iter()
                    .find(|&path| Path::new(path).exists())
                    .ok_or_else(|| {
                        ServerFnError::new(
                            "Cannot find remove_pii_data.sql script file".to_string(),
                        )
                    })?;

                // Update global setting first
                global_database::update_global_setting(
                    &pool,
                    "student_protections",
                    serde_json::Value::Bool(true),
                    user.id.try_into().map_err(|e| {
                        ServerFnError::new(format!("User ID conversion error: {}", e))
                    })?,
                )
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to update setting: {}", e)))?;

                // Check if psql is available
                let psql_check = Command::new("which").arg("psql").output().map_err(|e| {
                    ServerFnError::new(format!("Failed to check psql availability: {}", e))
                })?;

                if !psql_check.status.success() {
                    return Err(ServerFnError::new(
                        "psql command not found. PostgreSQL client tools not installed."
                            .to_string(),
                    ));
                }

                // Execute the remove PII script - FIXED: Use DATABASE_URL instead of hardcoded localhost
                let output = Command::new("psql")
                    .arg(&database_url)  // Use the actual DATABASE_URL
                    .arg("-f")
                    .arg(script_path)
                    .output()
                    .map_err(|e| {
                        ServerFnError::new(format!("Failed to execute PII removal: {}", e))
                    })?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Err(ServerFnError::new(format!(
                        "PII removal failed. STDERR: {} STDOUT: {}",
                        stderr, stdout
                    )));
                }

                // Try to create CSV in multiple locations
                let csv_locations = vec![
                    "/tmp/student_id_mapping.csv",
                    "./student_id_mapping.csv",
                    "/app/student_id_mapping.csv",
                ];

                let mut csv_created = false;
                for csv_path in csv_locations {
                    if Path::new(csv_path).exists() {
                        csv_created = true;
                        // Try to copy to current directory if not already there
                        if csv_path != "./student_id_mapping.csv" {
                            let _ = Command::new("cp")
                                .arg(csv_path)
                                .arg("./student_id_mapping.csv")
                                .output();
                        }
                        break;
                    }
                }

                if !csv_created {
                    // Manually create CSV using psql - FIXED: Use DATABASE_URL
                    let csv_creation = Command::new("psql")
                        .arg(&database_url)  // Use the actual DATABASE_URL
                        .arg("-c")
                        .arg("COPY (SELECT new_student_id as app_id, old_student_id as student_id, created_at FROM student_id_mapping ORDER BY new_student_id) TO '/tmp/student_id_mapping.csv' WITH CSV HEADER;")
                        .output()
                        .map_err(|e| ServerFnError::new(format!("Failed to create CSV: {}", e)))?;

                    if csv_creation.status.success() {
                        let _ = Command::new("cp")
                            .arg("/tmp/student_id_mapping.csv")
                            .arg("./student_id_mapping.csv")
                            .output();
                    }
                }

                Ok(
                    "Student protection enabled. Check for student_id_mapping.csv file."
                        .to_string(),
                )
            } else {
                // Disable protection logic (restore functionality)
                let _key = mapping_key.ok_or_else(|| {
                    ServerFnError::new(
                        "Mapping key required to disable student protection".to_string(),
                    )
                })?;

                // Check for restore script
                let restore_script_paths = vec![
                    "./scripts/restore_pii_data.sql",
                    "/app/scripts/restore_pii_data.sql",
                    "scripts/restore_pii_data.sql",
                ];

                let restore_script_path = restore_script_paths
                    .iter()
                    .find(|&path| Path::new(path).exists())
                    .ok_or_else(|| {
                        ServerFnError::new(
                            "Cannot find restore_pii_data.sql script file".to_string(),
                        )
                    })?;

                // Check for CSV file
                if !Path::new("./student_id_mapping.csv").exists() {
                    return Err(ServerFnError::new(
                        "Mapping CSV file not found. Cannot restore without mapping data."
                            .to_string(),
                    ));
                }

                // Copy CSV to /tmp for script access
                let _ = Command::new("cp")
                    .arg("./student_id_mapping.csv")
                    .arg("/tmp/student_id_mapping.csv")
                    .output();

                // Execute restore script - FIXED: Use DATABASE_URL
                let output = Command::new("psql")
                    .arg(&database_url)  // Use the actual DATABASE_URL instead of hardcoded localhost
                    .arg("-f")
                    .arg(restore_script_path)
                    .output()
                    .map_err(|e| {
                        ServerFnError::new(format!("Failed to execute PII restoration: {}", e))
                    })?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    return Err(ServerFnError::new(format!(
                        "PII restoration failed. STDERR: {} STDOUT: {}",
                        stderr, stdout
                    )));
                }

                // Update global setting
                global_database::update_global_setting(
                    &pool,
                    "student_protections",
                    serde_json::Value::Bool(false),
                    user.id.try_into().map_err(|e| {
                        ServerFnError::new(format!("User ID conversion error: {}", e))
                    })?,
                )
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to update setting: {}", e)))?;

                // Clean up
                let _ = Command::new("rm")
                    .arg("/tmp/student_id_mapping.csv")
                    .output();

                Ok("Student protection disabled. Original student IDs restored.".to_string())
            }
        } else {
            Err(ServerFnError::new(
                "Unauthorized: Login required".to_string(),
            ))
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[server(RestoreStudentIdsFromFile, "/api")]
pub async fn restore_student_ids_from_file(file_content: String) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::server_functions::auth::get_current_user;
        use actix_web::web;
        use leptos_actix::extract;
        use std::process::Command;

        // Check authentication and admin privileges
        let current_user = get_current_user().await?;
        if let Some(user) = current_user {
            if !user.is_super_admin() {
                return Err(ServerFnError::new(
                    "Unauthorized: Super admin access required".to_string(),
                ));
            }

            let pool = extract::<web::Data<PgPool>>()
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

            // Get DATABASE_URL from environment - FIXED: Use actual Cloud SQL connection
            let database_url = env::var("DATABASE_URL").map_err(|_| {
                ServerFnError::new("DATABASE_URL environment variable not set".to_string())
            })?;

            // Try multiple file paths that are more likely to work
            let possible_paths = vec![
                "./student_id_mapping.csv",    // Current working directory
                "/tmp/student_id_mapping.csv", // Temporary directory
                "student_id_mapping.csv",      // Relative path
            ];

            let mut temp_file_path = None;
            let mut write_error = None;

            // Try each path until one works
            for path in possible_paths {
                match std::fs::write(path, &file_content) {
                    Ok(_) => {
                        temp_file_path = Some(path);
                        break;
                    }
                    Err(e) => {
                        write_error = Some(format!("Failed to write to {}: {}", path, e));
                        continue;
                    }
                }
            }

            let final_path = temp_file_path.ok_or_else(|| {
                ServerFnError::new(format!(
                    "Failed to write temporary file to any location. Last error: {}",
                    write_error.unwrap_or_else(|| "Unknown error".to_string())
                ))
            })?;

            // Execute the restore PII script - FIXED: Use DATABASE_URL instead of hardcoded localhost
            let output = Command::new("psql")
                .arg(&database_url)  // Use the actual DATABASE_URL from environment
                .arg("-f")
                .arg("./scripts/restore_pii_data.sql")
                .output()
                .map_err(|e| {
                    ServerFnError::new(format!("Failed to execute PII restoration: {}", e))
                })?;

            // Capture output for debugging
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if !output.status.success() {
                // Clean up temporary file on error
                let _ = std::fs::remove_file(final_path);

                return Err(ServerFnError::new(format!(
                    "PII restoration failed. STDERR: {} STDOUT: {}",
                    stderr, stdout
                )));
            }

            // Update global setting to reflect that protection is now disabled
            global_database::update_global_setting(
                &pool,
                "student_protections",
                serde_json::Value::Bool(false),
                user.id
                    .try_into()
                    .map_err(|e| ServerFnError::new(format!("User ID conversion error: {}", e)))?,
            )
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to update setting: {}", e)))?;

            // Clean up temporary file
            let _ = std::fs::remove_file(final_path);

            // Return success message with some output for debugging
            Ok(format!(
                "Student IDs successfully restored from uploaded mapping file!\n\nUsed file path: {}\n\nDatabase URL: {}\n\nProcess output:\n{}",
                final_path, 
                // Mask the password in the URL for security in logs
                database_url.replace(&database_url.split('@').nth(0).unwrap_or(""), "***:***"),
                stdout
            ))
        } else {
            Err(ServerFnError::new(
                "Unauthorized: Login required".to_string(),
            ))
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}
