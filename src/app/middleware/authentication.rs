// Server-specific implementation gated behind "ssr" feature
#[cfg(feature = "ssr")]
mod server {
    use actix_web::{
        dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
        web, Error, HttpMessage,
    };
    use futures::future::{ready, LocalBoxFuture, Ready};
    use log::{debug, error, info};
    use sqlx::PgPool;
    use std::rc::Rc;
    use std::sync::Arc;
    use std::task::{Context, Poll};

    use crate::app::db::user_database;
    use crate::app::models::user::{User, UserJwt, UserRole};

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
                // Store user data outside the extensions borrow scope
                let mut authenticated_user: Option<UserJwt> = None;

                // Try to extract the database pool
                let pool = req.app_data::<web::Data<PgPool>>();

                if let Some(pool) = pool {
                    // Get session cookie from request
                    if let Some(cookies) = req.cookies().ok() {
                        if let Some(session_cookie) = cookies.iter().find(|c| c.name() == "session")
                        {
                            let session_token = session_cookie.value();

                            // Validate session using the actual database function
                            match user_database::validate_session(&pool, session_token).await {
                                Ok(Some(user)) => {
                                    debug!("Valid session found for user: {}", user.username);
                                    authenticated_user = Some(user);
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

                // Only insert into extensions if we have a valid user
                // and do it in a separate, scoped block
                if let Some(user) = authenticated_user {
                    // Carefully scope the mutable borrow to avoid conflicts
                    {
                        let mut extensions = req.extensions_mut();
                        extensions.insert(user);
                    } // Borrow is dropped here
                    debug!("User successfully added to request extensions");
                }

                // Continue with the request regardless of authentication status
                let res = service.call(req).await?;
                Ok(res)
            })
        }
    }

    // Helper function to extract current user from request extensions
    pub fn get_current_user_from_request(req: &ServiceRequest) -> Option<UserJwt> {
        req.extensions().get::<UserJwt>().cloned()
    }

    // Helper function to check if user has required role
    pub fn user_has_role(user: &UserJwt, required_role: UserRole) -> bool {
        match required_role {
            UserRole::Guest => true, // All users can access guest-level content
            UserRole::User => matches!(
                user.role,
                UserRole::User | UserRole::Admin | UserRole::SuperAdmin
            ),
            UserRole::Admin => matches!(user.role, UserRole::Admin | UserRole::SuperAdmin),
            UserRole::SuperAdmin => matches!(user.role, UserRole::SuperAdmin),
            UserRole::Teacher => matches!(
                user.role,
                UserRole::Teacher | UserRole::Admin | UserRole::SuperAdmin
            ),
        }
    }
}

// Reexport server types when "ssr" feature is enabled
#[cfg(feature = "ssr")]
pub use server::*;
