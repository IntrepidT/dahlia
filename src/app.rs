use leptos::*;
pub mod db;
pub mod errors;
pub mod models;
pub mod server_functions;
use leptos_meta::*;
use leptos_router::*;
pub mod pages;
pub mod websockets;
use crate::app::middleware::global_settings::SettingsProvider;
use components::enhanced_login_form::provide_student_mapping_service;
use components::live_test::RealtimeTestSession;
use components::login_components::{RequestPasswordResetForm, ResetPasswordForm};
use components::test_templates::{FlashCardSet, GridTest};
use pages::{
    AdminDashboard, AdministerTest, Assessment, AssessmentPage, Dashboard, Gradebook, HomePage,
    LoginPage, MathTesting, MyAccount, ReadingTesting, ReviewTest, Settings, StudentView, Teachers,
    TestBuilder, TestResultsPage, TestSessionsList,
};
pub mod components;
use components::auth::authorization_components::{
    AuthProvider, RequireAdminOrTeacher, RequireAuth, RequireRole,
};
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
                            <Route path="/dashboard" view=Dashboard />
                            <Route path="/studentview" view=StudentView />
                            <Route path="/studentview/:student_id/results" view=TestResultsPage />
                            <Route path="/admintest" view=move || {
                                view!{
                                    <RequireRole role="teacher".to_string()>
                                        <AdministerTest />
                                    </RequireRole>
                                }
                            }/>
                            <Route path="/teachers" view=move || {
                                view! {
                                    <RequireRole role="admin".to_string()>
                                        <Teachers />
                                    </RequireRole>
                                }
                            }/>
                            <Route path="/admindashboard" view=move || {
                                view! {
                                    <RequireRole role="admin".to_string()>
                                        <AdminDashboard />
                                    </RequireRole>
                                }
                            }/>
                            <Route path="/assessments" view=move || {
                                view! {
                                    <RequireRole role="admin".to_string()>
                                        <AssessmentPage />
                                    </RequireRole>
                                }
                            }/>
                            <Route path="/myaccount" view=|| {
                                view!{
                                    <RequireAuth>
                                        <MyAccount />
                                    </RequireAuth>
                                }
                            }/>
                            <Route path="/settings" view=|| {
                                view! {
                                    <RequireAuth>
                                        <Settings />
                                    </RequireAuth>
                                }
                            }/>
                            <Route path="/login" view=|| {
                                view!{
                                    <LoginPage />
                                }
                            }/>
                            <Route path="/forgot-password" view=|| {
                                view!{
                                    <RequestPasswordResetForm />
                                }
                            }/>
                            <Route path="/reset-password/:token" view=|| {
                                view!{
                                    <ResetPasswordForm />
                                }
                            }/>
                            // Fixed: Use RequireAdminOrTeacher instead of nested RequireRole
                            <Route path="/gradebook" view=|| {
                                view!{
                                    <RequireAdminOrTeacher>
                                        <Gradebook />
                                    </RequireAdminOrTeacher>
                                }
                            }/>
                            <Route path="/mathtesting" view=|| {
                                view!{
                                    <RequireAdminOrTeacher>
                                        <MathTesting />
                                    </RequireAdminOrTeacher>
                                }
                            }/>
                            <Route path="/readingtesting" view=|| {
                                view!{
                                    <RequireAdminOrTeacher>
                                        <ReadingTesting />
                                    </RequireAdminOrTeacher>
                                }
                            }/>
                            <Route path="/testbuilder" view=|| {
                                view!{
                                    <RequireAdminOrTeacher>
                                        <TestBuilder />
                                    </RequireAdminOrTeacher>
                                }
                            }/>
                            <Route path="/testbuilder/:test_id" view= || {
                                view!{
                                    <RequireAdminOrTeacher>
                                        <TestBuilder />
                                    </RequireAdminOrTeacher>
                                }
                            }/>
                            <Route path="/assessment/:test_id" view=|| {
                                view!{
                                    <RequireAdminOrTeacher>
                                        <Assessment />
                                    </RequireAdminOrTeacher>
                                }
                            }/>
                            <Route path="/flashcardset/:test_id" view=|| {
                                view!{
                                    <RequireAdminOrTeacher>
                                        <FlashCardSet />
                                    </RequireAdminOrTeacher>
                                }
                            }/>
                            <Route path="/reviewtest/:test_id/:student_id/:test_variant/:attempt" view=|| {
                                view! {
                                    <RequireAdminOrTeacher>
                                        <ReviewTest />
                                    </RequireAdminOrTeacher>
                                }
                            }/>
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
