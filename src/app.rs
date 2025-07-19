use leptos::*;
use leptos_meta::*;
use leptos_router::*;

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
        <Stylesheet id="leptos" href="/pkg/dahlia.css"/>
        <link data-trunk rel="tailwind-css" href="/style/input.css" />
        <link rel="icon" href="/assets/favicon.ico" />
        <Title text="Teapot Testing"/>
        <script src="https://cdn.plot.ly/plotly-2.24.1.min.js"></script>

        <AuthProvider>
            <SettingsProvider>
                <Router>
                    <main>
                        <Body />
                        <AppRoutes />
                    </main>
                </Router>
            </SettingsProvider>
        </AuthProvider>
    }
}
