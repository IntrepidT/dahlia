use std::collections::{HashMap, HashSet};
use uuid::Uuid;
#[cfg(feature = "ssr")]
use {
    crate::app::db::websocket_session_database,
    crate::app::websockets::messages::{ClientActorMessage, Connect, Disconnect, WsMessage},
    actix::prelude::{Actor, Context, Handler, Recipient},
    sqlx::PgPool,
};

#[cfg(feature = "ssr")]
type Socket = Recipient<WsMessage>;

#[cfg(feature = "ssr")]
pub struct Lobby {
    sessions: HashMap<Uuid, Socket>,
    rooms: HashMap<Uuid, HashSet<Uuid>>,
    db_pool: Option<sqlx::PgPool>,
}

#[cfg(feature = "ssr")]
impl Default for Lobby {
    fn default() -> Lobby {
        Lobby {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            db_pool: None,
        }
    }
}

#[cfg(feature = "ssr")]
impl Lobby {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Lobby {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            db_pool: Some(pool),
        }
    }

    fn send_message(&self, message: &str, id_to: &Uuid) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient.do_send(WsMessage(message.to_owned()));
        } else {
            println!("attempting to send message but couldn't find user id");
        }
    }

    // Helper method to update session counts in the database
    async fn update_db_session_count(&self, room_id: &Uuid, increment: bool) {
        if let Some(pool) = &self.db_pool {
            if let Err(e) =
                websocket_session_database::update_session_user_count(*room_id, increment, pool)
                    .await
            {
                log::error!("Failed to update session count: {}", e);
            }
        }
    }
}

#[cfg(feature = "ssr")]
impl Actor for Lobby {
    type Context = Context<Self>;
}

#[cfg(feature = "ssr")]
impl Handler<Disconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, ctx: &mut Context<Self>) {
        if self.sessions.remove(&msg.id).is_some() {
            if let Some(room_users) = self.rooms.get(&msg.room_id) {
                // Notify all users in the room about disconnection
                for user_id in room_users.iter().filter(|id| **id != msg.id) {
                    self.send_message(&format!("{} disconnected", &msg.id), user_id);
                }
            }

            if let Some(lobby) = self.rooms.get_mut(&msg.room_id) {
                if lobby.len() > 1 {
                    lobby.remove(&msg.id);

                    // Update database session count (this is async but we're in a sync context)
                    let room_id = msg.room_id;
                    if let Some(pool) = self.db_pool.clone() {
                        actix::spawn(async move {
                            if let Err(e) = websocket_session_database::update_session_user_count(
                                room_id, false, &pool,
                            )
                            .await
                            {
                                log::error!("Failed to update session count: {}", e);
                            }
                        });
                    }
                } else {
                    self.rooms.remove(&msg.room_id);

                    // Mark session as inactive in database
                    let room_id = msg.room_id;
                    if let Some(pool) = self.db_pool.clone() {
                        actix::spawn(async move {
                            use crate::app::models::websocket_session::SessionStatus;
                            if let Err(e) = websocket_session_database::update_session_status(
                                room_id,
                                SessionStatus::Inactive,
                                &pool,
                            )
                            .await
                            {
                                log::error!("Failed to mark session as inactive: {}", e);
                            }
                        });
                    }
                }
            }
        }
    }
}

#[cfg(feature = "ssr")]
impl Handler<Connect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // Add user to the room
        self.rooms
            .entry(msg.lobby_id)
            .or_insert_with(HashSet::new)
            .insert(msg.self_id);

        // Notify existing users
        if let Some(room_users) = self.rooms.get(&msg.lobby_id) {
            for conn_id in room_users.iter().filter(|id| **id != msg.self_id) {
                self.send_message(&format!("{} just joined!", msg.self_id), conn_id);
            }
        }

        // Store the connection
        self.sessions.insert(msg.self_id, msg.addr);

        // Send welcome message
        self.send_message(&format!("your id is {}", msg.self_id), &msg.self_id);

        // Update database session count
        let room_id = msg.lobby_id;
        if let Some(pool) = self.db_pool.clone() {
            actix::spawn(async move {
                if let Err(e) =
                    websocket_session_database::update_session_user_count(room_id, true, &pool)
                        .await
                {
                    log::error!("Failed to update session count: {}", e);
                }
            });
        }
    }
}

#[cfg(feature = "ssr")]
impl Handler<ClientActorMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, _ctx: &mut Context<Self>) -> Self::Result {
        // Handle private messages (whispers)
        if msg.msg.starts_with("\\w") {
            if let Some(id_to) = msg.msg.split(' ').collect::<Vec<&str>>().get(1) {
                self.send_message(&msg.msg, &Uuid::parse_str(id_to).unwrap());
            }
        } else {
            // Broadcast to all users in the room
            if let Some(room_users) = self.rooms.get(&msg.room_id) {
                for client in room_users.iter() {
                    self.send_message(&msg.msg, client);
                }
            }
        }
    }
}
