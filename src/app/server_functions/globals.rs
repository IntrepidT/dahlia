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
    mapping_key: Option<String>, // Required when disabling protection
) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::server_functions::auth::get_current_user;
        use actix_web::web;
        use leptos_actix::extract;
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

            if enable {
                // First, update the global setting to track protection status
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

                // Execute the remove PII script
                let output = Command::new("psql")
                    .arg(env::var("DATABASE_URL").unwrap())
                    .arg("-f")
                    .arg("./scripts/remove_pii_data.sql")
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

                // Check if the CSV file was created in /tmp/
                let csv_check = Command::new("test")
                    .arg("-f")
                    .arg("/tmp/student_id_mapping.csv")
                    .status()
                    .map_err(|e| ServerFnError::new(format!("Failed to check CSV file: {}", e)))?;

                if !csv_check.success() {
                    // CSV wasn't created by SQL script, let's create it manually using psql
                    let csv_creation = Command::new("psql")
                        .arg(env::var("DATABASE_URL").unwrap())
                        .arg("-c")
                        .arg("COPY (SELECT new_student_id as app_id, old_student_id as student_id, created_at FROM student_id_mapping ORDER BY new_student_id) TO '/tmp/student_id_mapping.csv' WITH CSV HEADER;")
                        .output()
                        .map_err(|e| ServerFnError::new(format!("Failed to create CSV manually: {}", e)))?;

                    if !csv_creation.status.success() {
                        let csv_stderr = String::from_utf8_lossy(&csv_creation.stderr);
                        let csv_stdout = String::from_utf8_lossy(&csv_creation.stdout);

                        // If that also fails, try creating the CSV in current directory directly
                        let direct_csv = Command::new("psql")
                            .arg(env::var("DATABASE_URL").unwrap())
                            .arg("-c")
                            .arg("COPY (SELECT new_student_id as app_id, old_student_id as student_id, created_at FROM student_id_mapping ORDER BY new_student_id) TO './student_id_mapping.csv' WITH CSV HEADER;")
                            .output()
                            .map_err(|e| ServerFnError::new(format!("Failed to create CSV in current dir: {}", e)))?;

                        if !direct_csv.status.success() {
                            let direct_stderr = String::from_utf8_lossy(&direct_csv.stderr);
                            return Err(ServerFnError::new(format!(
                                "Failed to create CSV file. Original SQL output - STDOUT: {} STDERR: {}. Manual CSV creation - STDOUT: {} STDERR: {}. Direct CSV creation error: {}",
                                stdout, stderr, csv_stdout, csv_stderr, direct_stderr
                            )));
                        }

                        // If direct creation worked, we're done
                        return Ok("Student protection enabled. Mapping key saved to ./student_id_mapping.csv".to_string());
                    }
                }

                // If we get here, CSV should exist in /tmp/, so move it
                let csv_move_result = Command::new("cp")
                    .arg("/tmp/student_id_mapping.csv")
                    .arg("./student_id_mapping.csv")
                    .output()
                    .map_err(|e| ServerFnError::new(format!("Failed to copy CSV file: {}", e)))?;

                if !csv_move_result.status.success() {
                    let move_stderr = String::from_utf8_lossy(&csv_move_result.stderr);
                    return Err(ServerFnError::new(format!(
                        "Failed to move CSV file to current directory: {}",
                        move_stderr
                    )));
                }

                // Set proper permissions
                let _ = Command::new("chmod")
                    .arg("644")
                    .arg("./student_id_mapping.csv")
                    .output();

                Ok(
                    "Student protection enabled. Mapping key saved to ./student_id_mapping.csv"
                        .to_string(),
                )
            } else {
                // Validate mapping key is provided
                let _key = mapping_key.ok_or_else(|| {
                    ServerFnError::new(
                        "Mapping key required to disable student protection".to_string(),
                    )
                })?;

                // Check if the CSV file exists before attempting restoration
                let csv_check = Command::new("test")
                    .arg("-f")
                    .arg("./student_id_mapping.csv")
                    .status()
                    .map_err(|e| ServerFnError::new(format!("Failed to check CSV file: {}", e)))?;

                if !csv_check.success() {
                    return Err(ServerFnError::new(
                        "Mapping CSV file not found. Cannot restore student IDs without mapping data.".to_string()
                    ));
                }

                // Copy the CSV file to /tmp for the SQL script to access
                let csv_copy_result = Command::new("cp")
                    .arg("./student_id_mapping.csv")
                    .arg("/tmp/student_id_mapping.csv")
                    .output()
                    .map_err(|e| {
                        ServerFnError::new(format!("Failed to copy CSV file to /tmp: {}", e))
                    })?;

                if !csv_copy_result.status.success() {
                    return Err(ServerFnError::new(format!(
                        "Failed to copy CSV file: {}",
                        String::from_utf8_lossy(&csv_copy_result.stderr)
                    )));
                }

                // Execute the restore PII script
                let output = Command::new("psql")
                    .arg("postgresql://postgres:IntrepidTh13n32!@localhost/dahlia")
                    .arg("-f")
                    .arg("./scripts/restore_pii_data.sql")
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

                // Clean up temporary files
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

            // Execute the restore PII script
            let output = Command::new("psql")
                .arg("postgresql://postgres:IntrepidTh13n32!@localhost/dahlia")
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
                "Student IDs successfully restored from uploaded mapping file!\n\nUsed file path: {}\n\nProcess output:\n{}",
                final_path, stdout
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
