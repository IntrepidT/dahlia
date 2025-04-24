use crate::app::models::Question;
use crate::app::server_functions::questions;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[cfg(feature = "ssr")]
use {
    crate::app::db::websocket_session_database,
    crate::app::models::question::QuestionType,
    crate::app::models::websocket_session::{Session, SessionType},
    crate::app::server_functions::questions::get_questions,
    crate::app::websockets::messages::{
        ClientActorMessage, Connect, Disconnect, TestMessageType, TestSessionMessage, WsMessage,
    },
    actix::prelude::{Actor, Context, Handler, Recipient},
    serde_json::{json, Value},
    sqlx::PgPool,
};

#[cfg(feature = "ssr")]
type Socket = Recipient<WsMessage>;

#[cfg(feature = "ssr")]
pub struct Lobby {
    sessions: HashMap<Uuid, Socket>,
    rooms: HashMap<Uuid, HashSet<Uuid>>,
    room_roles: HashMap<(Uuid, Uuid), String>,
    active_tests: HashMap<Uuid, String>,
    test_questions: HashMap<String, Vec<Value>>,
    db_pool: Option<sqlx::PgPool>,
}

#[cfg(feature = "ssr")]
impl Default for Lobby {
    fn default() -> Lobby {
        Lobby {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            room_roles: HashMap::new(),
            active_tests: HashMap::new(),
            test_questions: HashMap::new(),
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
            room_roles: HashMap::new(),
            active_tests: HashMap::new(),
            test_questions: HashMap::new(),
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

    fn handle_test_message(&mut self, msg: TestSessionMessage) {
        match msg.message_type {
            TestMessageType::StartTest => {
                if let Some(test_id) = msg.payload.get("test_id").and_then(|id| id.as_str()) {
                    self.active_tests.insert(msg.room_id, test_id.to_string());

                    if !self.test_questions.contains_key(test_id) {
                        let test_id_copy = test_id.to_string();
                        let db_pool = self.db_pool.clone();

                        if let Some(pool) = db_pool {
                            actix::spawn(async move {
                                match get_questions(test_id_copy.clone()).await {
                                    Ok(questions) => {}
                                    Err(e) => {
                                        log::error!("Failed to fetch test questions: {}", e);
                                    }
                                }
                            });
                        }
                    }

                    if let Some(room_users) = self.rooms.get(&msg.room_id) {
                        let start_msg = json!({
                            "type": "test_started",
                            "test_id": test_id,
                        });

                        for user_id in room_users.iter() {
                            self.send_message(&start_msg.to_string(), user_id);
                        }
                    }
                }
            }
            TestMessageType::SubmitAnswer => {
                // A student is submitting an answer
                if let Some(room_users) = self.rooms.get(&msg.room_id) {
                    // Forward answers only to teacher(s)
                    let answer_msg = json!({
                        "type": "student_answer",
                        "student_id": msg.id.to_string(),
                        "answer_data": msg.payload,
                    });

                    // Send only to teachers in the room
                    for user_id in room_users.iter() {
                        if let Some(role) = self.room_roles.get(&(msg.room_id, *user_id)) {
                            if role == "teacher" {
                                self.send_message(&answer_msg.to_string(), user_id);
                            }
                        }
                    }
                }
            }
            TestMessageType::TeacherComment => {
                // Teacher is providing feedback - broadcast to everyone
                if let Some(room_users) = self.rooms.get(&msg.room_id) {
                    let comment_msg = json!({
                        "type": "teacher_comment",
                        "comment": msg.payload,
                    });

                    for user_id in room_users.iter() {
                        self.send_message(&comment_msg.to_string(), user_id);
                    }
                }
            }
            TestMessageType::EndTest => {
                // End the test and remove it from active tests
                self.active_tests.remove(&msg.room_id);

                // Notify all users in the room
                if let Some(room_users) = self.rooms.get(&msg.room_id) {
                    let end_msg = json!({
                        "type": "test_ended",
                    });

                    for user_id in room_users.iter() {
                        self.send_message(&end_msg.to_string(), user_id);
                    }
                }
            }
            TestMessageType::StudentJoined => {
                // Notify the teacher that a new student joined
                if let Some(room_users) = self.rooms.get(&msg.room_id) {
                    let student_msg = json!({
                        "type": "student_joined",
                        "student_id": msg.id.to_string(),
                        "student_data": msg.payload,
                    });

                    // Send only to teachers
                    for user_id in room_users.iter() {
                        if let Some(role) = self.room_roles.get(&(msg.room_id, *user_id)) {
                            if role == "teacher" {
                                self.send_message(&student_msg.to_string(), user_id);
                            }
                        }
                    }
                }
            }
            TestMessageType::StudentLeft => {
                // Notify the teacher that a student left
                if let Some(room_users) = self.rooms.get(&msg.room_id) {
                    let student_msg = json!({
                        "type": "student_left",
                        "student_id": msg.id.to_string(),
                    });

                    // Send only to teachers
                    for user_id in room_users.iter() {
                        if let Some(role) = self.room_roles.get(&(msg.room_id, *user_id)) {
                            if role == "teacher" {
                                self.send_message(&student_msg.to_string(), user_id);
                            }
                        }
                    }
                }
            }
            TestMessageType::QuestionFocus => {
                // Teacher wants to focus everyone on a specific question
                if let Some(room_users) = self.rooms.get(&msg.room_id) {
                    let focus_msg = json!({
                        "type": "focus_question",
                        "question_data": msg.payload,
                    });

                    for user_id in room_users.iter() {
                        self.send_message(&focus_msg.to_string(), user_id);
                    }
                }
            }
            TestMessageType::TimeUpdate => {
                // Update time remaining for the test
                if let Some(room_users) = self.rooms.get(&msg.room_id) {
                    let time_msg = json!({
                        "type": "time_update",
                        "time_data": msg.payload,
                    });

                    for user_id in room_users.iter() {
                        self.send_message(&time_msg.to_string(), user_id);
                    }
                }
            }
            _ => {
                //Basic pass-through for other message types
                if let Some(room_users) = self.rooms.get(&msg.room_id) {
                    let msg_json = json!({
                        "type": format!("{:?}", msg.message_type).to_lowercase(),
                        "from": msg.id.to_string(),
                        "data": msg.payload,
                    });

                    for user_id in room_users.iter() {
                        self.send_message(&msg_json.to_string(), user_id);
                    }
                }
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

        if !self.room_roles.contains_key(&(msg.lobby_id, msg.self_id)) {
            let is_test_room = self.active_tests.contains_key(&msg.lobby_id);
            let room_empty = self
                .rooms
                .get(&msg.lobby_id)
                .map_or(true, |users| users.len() <= 1);

            let role = if is_test_room && room_empty {
                "teacher"
            } else {
                "student"
            };

            self.room_roles
                .insert((msg.lobby_id, msg.self_id), role.to_string());
        }

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

        if let Some(test_id) = self.active_tests.get(&msg.lobby_id) {
            let role = self.room_roles.get(&(msg.lobby_id, msg.self_id)).unwrap();

            let role_msg = json!({
                "type": "role_assigned",
                "role": role
            });
            self.send_message(&role_msg.to_string(), &msg.self_id);

            if let Some(questions) = self.test_questions.get(test_id) {
                let filtered_questions = if role == "teacher" {
                    questions.clone()
                } else {
                    questions
                        .iter()
                        .map(|q| {
                            let mut q_copy = q.clone();
                            if let Some(obj) = q_copy.as_object_mut() {
                                obj.remove("correct_answer");
                            }
                            q_copy
                        })
                        .collect()
                };

                let test_data = json!({
                    "type": "test_data",
                    "test_id": test_id,
                    "questions": filtered_questions
                });

                self.send_message(&test_data.to_string(), &msg.self_id);
            }
        }

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

#[cfg(feature = "ssr")]
impl Handler<TestSessionMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: TestSessionMessage, _ctx: &mut Context<Self>) -> Self::Result {
        self.handle_test_message(msg);
    }
}
