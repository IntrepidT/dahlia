use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSetting {
    pub key_name: String,
    pub value: String,
    pub updated_by: i32,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsCache {
    pub student_protections: bool,
    // Add other global settings here as needed
    pub maintenance_mode: bool,
    pub max_upload_size: i64,
    // etc.
}

impl SettingsCache {
    pub fn from_settings(settings: Vec<GlobalSetting>) -> Self {
        let mut cache = SettingsCache::default();

        for setting in settings {
            match setting.key_name.as_str() {
                "student_protections" => {
                    cache.student_protections = setting.value.parse().unwrap_or(false);
                }
                "maintenance_mode" => {
                    cache.maintenance_mode = setting.value.parse().unwrap_or(false);
                }
                "max_upload_size" => {
                    cache.max_upload_size = setting.value.parse().unwrap_or(10485760);
                    // 10MB default
                }
                _ => {} // Ignore unknown settings
            }
        }

        cache
    }
}
