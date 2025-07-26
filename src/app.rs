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
        <Script>
            {r#"
            if (typeof window !== 'undefined' && !window.Chart) {
                const script = document.createElement('script');
                script.src = 'https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.js';
                script.onload = function() {
                    console.log('Chart.js loaded successfully');
                    window.dispatchEvent(new Event('chartjs-loaded'));
                };
                script.onerror = function() {
                    console.error('Failed to load Chart.js');
                };
                document.head.appendChild(script);
            }
            "#}
        </Script>

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
