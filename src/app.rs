use leptos::*;
pub mod db;
pub mod errors;
pub mod models;
pub mod server_functions;
use leptos_meta::*;
use leptos_router::*;
pub mod pages;
pub mod websockets;
use components::test_templates::FlashCardSet;
use pages::{
    AdministerTest, Assessment, Dashboard, HomePage, LoginPage, MathTesting, MyAccount,
    ReadingTesting, ReviewTest, StudentView, Teachers, TestBuilder,
};
pub mod components;
use components::auth::*;
pub mod middleware;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/dahlia.css"/>
        <link data-trunk rel="tailwind-css" href="/style/input.css" />
        <link rel="icon" href="/assets/favicon.ico" />
        // sets the document title
        <Title text="Teapot Testing"/>

        // Wrap everything in the AuthProvider
        <AuthProvider>
            // content for this welcome page
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
                        <Route path="/studentview" view=move || {
                            view!{
                                <RequireRole role="admin".to_string()>
                                    <StudentView />
                                </RequireRole>
                            }
                        }/>
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
                        <Route path="/myaccount" view=|| {
                            view!{
                                <RequireAuth>
                                    <MyAccount />
                                </RequireAuth>
                            }
                        }/>
                        <Route path="/login" view=|| {
                            view!{
                                <LoginPage />
                            }
                        }/>
                        <Route path="/mathtesting" view=|| {
                            view!{
                                <MathTesting />
                            }
                        }/>
                        <Route path="/readingtesting" view=|| {
                            view!{
                                <ReadingTesting />
                            }
                        }/>
                        <Route path="/testbuilder" view=|| {
                            view!{
                                <RequireRole role="teacher".to_string()>
                                    <RequireRole role="admin".to_string()>
                                        <TestBuilder />
                                    </RequireRole>
                                </RequireRole>
                            }
                        }/>
                        <Route path="/testbuilder/:test_id" view= || {
                            view!{
                                <RequireRole role="admin".to_string()>
                                    <RequireRole role="teacher".to_string()>
                                        <TestBuilder />
                                    </RequireRole>
                                </RequireRole>
                            }
                        }/>
                        <Route path="/assessment/:test_id" view=|| {
                            view!{
                                <RequireRole role="teacher".to_string()>
                                    <RequireRole role="admin".to_string()>
                                        <Assessment />
                                    </RequireRole>
                                </RequireRole>
                            }
                        }/>
                        <Route path="/flashcardset/:test_id" view=|| {
                            view!{
                                <RequireRole role="teacher".to_string()>
                                    <RequireRole role="admin".to_string()>
                                        <FlashCardSet />
                                    </RequireRole>
                                </RequireRole>
                            }
                        }/>
                        <Route path="/reviewtest/:test_id/:student_id/:test_variant" view=|| {
                            view! {
                                <RequireRole role="teacher".to_string()>
                                    <RequireRole role="admin".to_string()>
                                        <ReviewTest />
                                    </RequireRole>
                                </RequireRole>
                            }
                        }/>
                        <Route path="/*any" view=NotFound/>
                    </Routes>
                </main>
            </Router>
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
