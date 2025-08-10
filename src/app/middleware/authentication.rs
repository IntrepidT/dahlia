use leptos::prelude::*;
#[cfg(feature = "ssr")]
mod server {
    use actix_web::{
        dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
        web, Error, HttpMessage, HttpResponse,
    };
    use futures::future::{ready, LocalBoxFuture, Ready};
    use log::{debug, error};
    use sqlx::PgPool;
    use std::rc::Rc;
    use std::task::{Context, Poll};

    use crate::app::db::user_database;
    use crate::app::models::user::{SessionUser, UserRole};

    pub struct Authentication;

    impl Authentication {
        pub fn new() -> Self {
            Authentication
        }
    }

    impl<S, B> Transform<S, ServiceRequest> for Authentication
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
        S::Future: 'static,
        B: 'static,
    {
        type Response = ServiceResponse<B>;
        type Error = Error;
        type Transform = AuthenticationMiddleware<S>;
        type InitError = ();
        type Future = Ready<Result<Self::Transform, Self::InitError>>;

        fn new_transform(&self, service: S) -> Self::Future {
            ready(Ok(AuthenticationMiddleware {
                service: Rc::new(service),
            }))
        }
    }

    pub struct AuthenticationMiddleware<S> {
        service: Rc<S>,
    }

    impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
        S::Future: 'static,
        B: 'static,
    {
        type Response = ServiceResponse<B>;
        type Error = Error;
        type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

        forward_ready!(service);

        fn call(&self, req: ServiceRequest) -> Self::Future {
            let service = self.service.clone();

            Box::pin(async move {
                let mut authenticated_user: Option<SessionUser> = None;

                // Extract database pool
                if let Some(pool) = req.app_data::<web::Data<PgPool>>() {
                    // Check for session cookie
                    if let Ok(cookies) = req.cookies() {
                        if let Some(session_cookie) = cookies.iter().find(|c| c.name() == "session")
                        {
                            let session_token = session_cookie.value();

                            // Validate session - returns SessionUser (not User)
                            match user_database::validate_session(&pool, session_token).await {
                                Ok(Some(user)) => {
                                    debug!("Valid session found for user: {}", user.username);
                                    authenticated_user = Some(user); // Already SessionUser
                                }
                                Ok(None) => {
                                    debug!("Invalid or expired session token");
                                }
                                Err(e) => {
                                    error!("Error validating session: {:?}", e);
                                }
                            }
                        }
                    }
                } else {
                    error!("Database pool not found in middleware");
                }

                // Insert SessionUser into request extensions if authenticated
                if let Some(user) = authenticated_user {
                    req.extensions_mut().insert(user);
                    debug!("User successfully added to request extensions");
                }

                // Continue with request
                service.call(req).await
            })
        }
    }

    // Helper functions for extracting user data from requests
    pub fn get_current_user_from_request(req: &ServiceRequest) -> Option<SessionUser> {
        req.extensions().get::<SessionUser>().cloned()
    }

    // Helper function to check if user has required role
    pub fn user_has_role(user: &SessionUser, required_role: UserRole) -> bool {
        match required_role {
            UserRole::Guest => true, // All users can access guest-level content
            UserRole::User => user.is_user(),
            UserRole::Teacher => user.is_teacher(),
            UserRole::Admin => user.is_admin(),
            UserRole::SuperAdmin => user.is_super_admin(),
        }
    }

    // Helper to check if user has any of the required roles
    pub fn user_has_any_role(user: &SessionUser, required_roles: &[UserRole]) -> bool {
        required_roles.iter().any(|role| user_has_role(user, *role))
    }

    // Helper to get user ID from request
    pub fn get_user_id_from_request(req: &ServiceRequest) -> Option<i64> {
        get_current_user_from_request(req).map(|user| user.id)
    }

    // Helper to check if user is authenticated
    pub fn is_authenticated(req: &ServiceRequest) -> bool {
        get_current_user_from_request(req).is_some()
    }

    // Helper to check if user is admin
    pub fn is_admin(req: &ServiceRequest) -> bool {
        get_current_user_from_request(req)
            .map(|user| user.is_admin())
            .unwrap_or(false)
    }

    // Helper to check if user is teacher or admin
    pub fn is_teacher_or_admin(req: &ServiceRequest) -> bool {
        get_current_user_from_request(req)
            .map(|user| user.is_teacher())
            .unwrap_or(false)
    }
}

// Re-export the Authentication struct at the module root level
#[cfg(feature = "ssr")]
pub use server::Authentication;

// Provide a stub implementation for non-ssr builds
#[cfg(not(feature = "ssr"))]
pub struct Authentication;

#[cfg(not(feature = "ssr"))]
impl Authentication {
    pub fn new() -> Self {
        Authentication
    }
}
