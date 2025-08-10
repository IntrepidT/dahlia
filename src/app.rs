use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::hooks::*;

// Core modules - organized alphabetically
pub mod components;
pub mod db;
pub mod errors;
pub mod middleware;
pub mod models;
pub mod pages;
pub mod routes;
pub mod server_functions;
pub mod services;
pub mod utils;
pub mod websockets;

// Module for app routing
pub mod app_routes;

// Re-exports for convenience
pub use utils::{BenchmarkStats, BenchmarkUtils};

// Import what we need for the main app
use app_routes::AppRoutes;
use components::{
    auth::authorization_components::AuthProvider,
    enhanced_login_form::provide_student_mapping_service,
};
use middleware::global_settings::SettingsProvider;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let student_mapping_context = provide_student_mapping_service();
    provide_context(student_mapping_context);

    view! {
        <AuthProvider>
            <SettingsProvider>
                <Router>
                    <AppRoutes />
                </Router>
            </SettingsProvider>
        </AuthProvider>
    }
}
