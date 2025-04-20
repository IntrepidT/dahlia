use uuid::Uuid;
#[cfg(feature = "ssr")]
use {
    crate::app::db::websocket_session_database,
    crate::app::models::websocket_session::SessionStatus,
    crate::app::websockets::lobby::Lobby,
    crate::app::websockets::ws::WsConn,
    actix::Addr,
    actix_web::{get, web::Data, web::Path, web::Payload, Error, HttpRequest, HttpResponse},
    actix_web_actors::ws,
};

#[cfg(feature = "ssr")]
#[get("/{group_id}")]
pub async fn start_connection(
    req: HttpRequest,
    stream: Payload,
    path: Path<String>,
    srv: Data<Addr<Lobby>>,
    pool: Data<sqlx::PgPool>,
) -> Result<HttpResponse, Error> {
    let group_id_str = path.into_inner();
    let group_id = Uuid::parse_str(&group_id_str)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid UUID format"))?;

    // Verify the session exists and is active
    match websocket_session_database::get_session(group_id, &pool).await {
        Ok(Some(session)) => {
            if session.status != SessionStatus::Active {
                return Err(actix_web::error::ErrorBadRequest("Session is not active").into());
            }

            // Check if session is full
            if session.max_users > 0 && session.current_users >= session.max_users {
                return Err(actix_web::error::ErrorBadRequest("Session is full").into());
            }

            // Create WebSocket connection
            let ws = WsConn::new(group_id, srv.get_ref().clone());
            let resp = ws::start(ws, &req, stream)?;
            Ok(resp)
        }
        Ok(None) => Err(actix_web::error::ErrorNotFound("Session not found").into()),
        Err(e) => {
            log::error!("Database error when verifying session: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Database error").into())
        }
    }
}

// Add a new endpoint to get information about active rooms
#[cfg(feature = "ssr")]
#[get("/rooms")]
pub async fn list_active_rooms(pool: Data<sqlx::PgPool>) -> Result<HttpResponse, Error> {
    match websocket_session_database::get_active_sessions(&pool).await {
        Ok(sessions) => {
            // Convert to a format suitable for client consumption
            let room_summaries = sessions
                .into_iter()
                .map(|session| crate::app::models::websocket_session::SessionSummary::from(session))
                .collect::<Vec<_>>();

            Ok(HttpResponse::Ok().json(room_summaries))
        }
        Err(e) => {
            log::error!("Error fetching active rooms: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Database error").into())
        }
    }
}
