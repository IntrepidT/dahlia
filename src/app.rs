use leptos::*;
pub mod db;
pub mod errors;
pub mod models;
pub mod server_functions;
use leptos_meta::*;
use leptos_router::*;
pub mod pages;
pub mod routes;
pub mod websockets;
use crate::app::components::test_components::test_variation_manager::TestVariationManager;
use crate::app::middleware::global_settings::SettingsProvider;
use components::enhanced_login_form::provide_student_mapping_service;
use components::live_test::RealtimeTestSession;
use components::login_components::{RequestPasswordResetForm, ResetPasswordForm};
use components::test_templates::{FlashCardSet, GridTest};
use pages::{
    AdminDashboard, AdministerTest, Assessment, AssessmentPage, Dashboard, Gradebook, HomePage,
    LoginPage, MyAccount, ReviewTest, Settings, StudentView, Teachers, TestBuilder,
    TestResultsPage, TestSessionsList, UnifiedTestManager,
};
pub mod components;
use components::auth::authorization_components::{
    AuthProvider, RequireAdminOrTeacher, RequireAuth, RequireRole,
};
use components::saml_admin::SamlAdminPanel;
pub mod middleware;
pub mod services;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let student_mapping_context = provide_student_mapping_service();
    provide_context(student_mapping_context);
    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/dahlia.css"/>
        <link data-trunk rel="tailwind-css" href="/style/input.css" />
        <link rel="icon" href="/assets/favicon.ico" />
        // sets the document title
        <Title text="Teapot Testing"/>

        <script src="https://cdn.plot.ly/plotly-2.24.1.min.js"></script>

        // Wrap everything in the AuthProvider
        <AuthProvider>
            // content for this welcome page
            <SettingsProvider>
                <Router>
                    <main>
                        <Body />
                        <Routes>
                            <Route path="/" view=move || {
                                view! {
                                    <HomePage />
                                }
                            }/>
                            <Route path="/dashboard" view=move || {
                                view! {
                                    <Dashboard />
                                }
                            }/>
                            <Route path="/admin/saml" view=SamlAdminPanel />
                            <Route path="/studentview" view=StudentView />
                            <Route path="/studentview/:student_id/results" view=TestResultsPage />
                            <Route path="/admintest" view=AdministerTest />
                            <Route path="/teachers" view=Teachers />
                            <Route path="/admindashboard" view=AdminDashboard />
                            <Route path="/assessments" view=AssessmentPage />
                            <Route path="/myaccount" view=MyAccount />
                            <Route path="/settings" view=Settings />
                            <Route path="/login" view=LoginPage />
                            <Route path="/forgot-password" view=RequestPasswordResetForm />
                            <Route path="/reset-password/:token" view=ResetPasswordForm />
                            <Route path="/gradebook" view=Gradebook />
                            <Route path="/test-manager" view=UnifiedTestManager />
                            <Route path="/testbuilder" view=TestBuilder />
                            <Route path="/test-variations" view=TestVariationManager />
                            <Route path="/testbuilder/:test_id" view=TestBuilder />
                            <Route path="/assessment/:test_id" view=Assessment />
                            <Route path="/flashcardset/:test_id" view=FlashCardSet />
                            <Route path="/reviewtest/:test_id/:student_id/:test_variant/:attempt" view=ReviewTest />
                            <Route path="/test-session/:test_id" view=RealtimeTestSession/>
                            <Route path="/tests/:test_id/sessions/:session_id" view=RealtimeTestSession/>
                            <Route path="/testsessions" view=TestSessionsList/>
                            <Route path="/gridtest/:test_id" view=GridTest/>
                            <Route path="/*any" view=NotFound/>
                        </Routes>
                    </main>
                </Router>
            </SettingsProvider>
        </AuthProvider>
    }
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}
