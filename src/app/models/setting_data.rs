use leptos::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserSettings {
    pub ui: UiSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiSettings {
    pub dark_mode: bool,
    pub pinned_sidebar: bool,
}
impl Default for UserSettings {
    fn default() -> Self {
        Self {
            ui: UiSettings::default(),
        }
    }
}
impl Default for UiSettings {
    fn default() -> Self {
        Self {
            dark_mode: false,
            pinned_sidebar: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSettingsUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui: Option<UiSettingsUpdate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiSettingsUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dark_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned_sidebar: Option<bool>,
}
