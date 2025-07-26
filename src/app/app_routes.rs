use leptos::*;
use leptos_router::*;
// Importing necessary components and pages
use crate::app::components::{
    live_testing::{test_session::RealtimeTestSession, AnonymousStudentTest},
    login_components::{RequestPasswordResetForm, ResetPasswordForm},
    saml_admin::SamlAdminPanel,
    test_components::test_variation_manager::TestVariationManager,
    test_templates::{FlashCardSet, GridTest},
};
use crate::app::pages::*;

#[component]
pub fn AppRoutes() -> impl IntoView {
    view! {
        <Routes>
            // Public routes
            <Route path="/" view=HomePage/>
            <Route path="/login" view=LoginPage/>
            <Route path="/forgot-password" view=RequestPasswordResetForm/>
            <Route path="/reset-password/:token" view=ResetPasswordForm/>

            // Dashboard routes
            <Route path="/dashboard" view=Dashboard/>
            <Route path="/admindashboard" view=AdminDashboard/>
            <Route path="/studentview" view=StudentView/>

            // Test management routes
            <Route path="/test-manager" view=UnifiedTestManager/>
            <Route path="/testbuilder" view=TestBuilder/>
            <Route path="/testbuilder/:test_id" view=TestBuilder/>
            <Route path="/test-variations" view=TestVariationManager/>

            // Assessment routes
            <Route path="/assessments" view=AssessmentPage/>
            <Route path="/admintest" view=AdministerTest/>
            <Route path="/gradebook" view=Gradebook/>

            // Test session routes
            <Route path="/test-session/:test_id" view=RealtimeTestSession/>
            <Route path="/student-test/:test_id/:session_id" view=AnonymousStudentTest/>
            <Route path="/tests/:test_id/sessions/:session_id" view=RealtimeTestSession/>

            // Test template routes
            <Route path="/flashcardset/:test_id" view=FlashCardSet/>
            <Route path="/gridtest/:test_id" view=GridTest/>

            // Review and results routes
            <Route path="/reviewtest/:test_id/:student_id/:test_variant/:attempt" view=ReviewTest/>
            <Route path="/studentview/:student_id/results" view=TestResultsPage/>

            // Settings and admin routes
            <Route path="/settings" view=Settings/>
            <Route path="/myaccount" view=MyAccount/>
            <Route path="/teachers" view=Teachers/>
            <Route path="/admin/saml" view=SamlAdminPanel/>

            // 404 fallback
            <Route path="/*any" view=NotFound/>
        </Routes>
    }
}

#[component]
pub fn NotFound() -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50">
            <div class="text-center">
                <h1 class="text-6xl font-bold text-gray-400">"404"</h1>
                <h2 class="text-2xl font-semibold text-gray-600 mt-4">"Page Not Found"</h2>
                <p class="text-gray-500 mt-2">"The page you're looking for doesn't exist."</p>
                <a href="/" class="mt-6 inline-block px-6 py-3 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors">
                    "Go Home"
                </a>
            </div>
        </div>
    }
}
