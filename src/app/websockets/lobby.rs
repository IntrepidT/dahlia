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
        ClientActorMessage, Connect, Disconnect, TestMessageType, TestSessionMessage,
        UserInfoMessage, WsMessage,
    },
    actix::prelude::{Actor, Context, Handler, Message, Recipient},
    serde_json::{json, Value},
    sqlx::PgPool,
};

#[cfg(feature = "ssr")]
type Socket = Recipient<WsMessage>;

#[cfg(feature = "ssr")]
#[derive(Debug, Clone)]
pub struct AnonymousStudent {
    pub id: String,
    pub name: String,
    pub session_id: Uuid,
    pub test_id: String,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(feature = "ssr")]
pub struct Lobby {
    sessions: HashMap<Uuid, Socket>,
    rooms: HashMap<Uuid, HashSet<Uuid>>,
    room_roles: HashMap<(Uuid, Uuid), String>,
    active_tests: HashMap<Uuid, String>,
    test_questions: HashMap<String, Vec<Value>>,
    anonymous_students: HashMap<Uuid, AnonymousStudent>, // New field for anonymous students
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
            anonymous_students: HashMap::new(),
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
            anonymous_students: HashMap::new(),
        }
    }

    fn handle_user_info_message(&mut self, msg: serde_json::Value, user_id: Uuid, room_id: Uuid) {
        if let (Some(role_str), Some(is_teacher), Some(is_admin)) = (
            msg.get("role").and_then(|r| r.as_str()),
            msg.get("is_teacher").and_then(|t| t.as_bool()),
            msg.get("is_admin").and_then(|a| a.as_bool()),
        ) {
            log::info!(
                "Received user info - role: {}, is_teacher: {}, is_admin: {}",
                role_str,
                is_teacher,
                is_admin
            );

            // Assign role based on actual permissions
            let assigned_role = if is_admin || is_teacher {
                "teacher"
            } else {
                "student"
            };

            // Update or set the role
            self.room_roles
                .insert((room_id, user_id), assigned_role.to_string());

            // Send role confirmation
            let role_msg = serde_json::json!({
                "type": "role_assigned",
                "role": assigned_role,
                "user_id": user_id.to_string(),
                "room_id": room_id.to_string()
            });

            log::info!(
                "Assigning role '{}' to user {} based on permissions",
                assigned_role,
                user_id
            );
            self.send_message(&role_msg.to_string(), &user_id);
        } else {
            log::warn!("Invalid user info message format");
        }
    }

    // NEW: Handle anonymous student join
    fn handle_anonymous_student_join(
        &mut self,
        msg: serde_json::Value,
        user_id: Uuid,
        room_id: Uuid,
    ) {
        if let (Some(name), Some(student_id), Some(test_id)) = (
            msg.get("student_name").and_then(|n| n.as_str()),
            msg.get("student_id").and_then(|s| s.as_str()),
            msg.get("test_id").and_then(|t| t.as_str()),
        ) {
            log::info!(
                "Anonymous student joining: {} (ID: {}) for test {}",
                name,
                student_id,
                test_id
            );

            // Store anonymous student info
            let anon_student = AnonymousStudent {
                id: student_id.to_string(),
                name: name.to_string(),
                session_id: room_id,
                test_id: test_id.to_string(),
                joined_at: chrono::Utc::now(),
            };

            self.anonymous_students
                .insert(user_id, anon_student.clone());

            // Assign student role automatically
            self.room_roles
                .insert((room_id, user_id), "student".to_string());

            // Send role confirmation with student info
            let role_msg = json!({
                "type": "role_assigned",
                "role": "student",
                "user_id": user_id.to_string(),
                "room_id": room_id.to_string(),
                "student_info": {
                    "name": name,
                    "student_id": student_id,
                    "is_anonymous": true
                }
            });

            log::info!("Assigning student role to anonymous user {}", user_id);
            self.send_message(&role_msg.to_string(), &user_id);

            // Broadcast student joined event with proper student data
            self.broadcast_user_event(
                &room_id,
                &user_id,
                "student_joined",
                Some(json!({
                    "name": name,
                    "student_id": student_id,
                    "is_anonymous": true,
                    "joined_at": anon_student.joined_at.to_rfc3339()
                })),
            );
        } else {
            log::warn!("Invalid anonymous student join message format");
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

    // ENHANCED: Send participants list with anonymous student info
    fn send_participants_list(&self, room_id: &Uuid, requesting_user_id: &Uuid) {
        if let Some(room_users) = self.rooms.get(room_id) {
            let participants: Vec<serde_json::Value> = room_users
                .iter()
                .map(|user_id| {
                    let role = self
                        .room_roles
                        .get(&(*room_id, *user_id))
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());

                    // Check if this is an anonymous student
                    if let Some(anon_student) = self.anonymous_students.get(user_id) {
                        json!({
                            "id": anon_student.id,
                            "name": anon_student.name,
                            "type": "Student",
                            "status": "Connected",
                            "role": role,
                            "is_anonymous": true,
                            "joined_at": anon_student.joined_at.to_rfc3339()
                        })
                    } else {
                        // Regular logged-in user
                        json!({
                            "id": user_id.to_string(),
                            "name": format!("User {}", user_id.to_string()[..8].to_uppercase()),
                            "type": match role.as_str() {
                                "teacher" => "Teacher",
                                "student" => "Student",
                                _ => "User"
                            },
                            "status": "Connected",
                            "role": role,
                            "is_anonymous": false
                        })
                    }
                })
                .collect();

            let participants_msg = json!({
                "type": "participants_list",
                "participants": participants,
                "room_id": room_id.to_string(),
                "total_count": participants.len()
            });

            log::info!(
                "Sending participants list to {}: {} participants",
                requesting_user_id,
                participants.len()
            );
            self.send_message(&participants_msg.to_string(), requesting_user_id);
        } else {
            log::warn!("Room {} not found when requesting participants", room_id);
        }
    }

    // ENHANCED: Broadcast user event with anonymous student support
    fn broadcast_user_event(
        &self,
        room_id: &Uuid,
        user_id: &Uuid,
        event_type: &str,
        user_data: Option<serde_json::Value>,
    ) {
        if let Some(room_users) = self.rooms.get(room_id) {
            let role = self
                .room_roles
                .get(&(*room_id, *user_id))
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            let event_msg = if let Some(provided_data) = user_data {
                // Use provided data (for anonymous students)
                json!({
                    "type": event_type,
                    "id": provided_data.get("student_id").unwrap_or(&json!(user_id.to_string())),
                    "user_data": provided_data,
                    "room_id": room_id.to_string(),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
            } else {
                // Default data for regular users
                json!({
                    "type": event_type,
                    "id": user_id.to_string(),
                    "user_data": json!({
                        "name": format!("User {}", user_id.to_string()[..8].to_uppercase()),
                        "username": format!("User {}", user_id.to_string()[..8].to_uppercase()),
                        "role": role,
                        "is_anonymous": false
                    }),
                    "room_id": room_id.to_string(),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
            };

            log::info!(
                "Broadcasting {} event for user {} in room {}",
                event_type,
                user_id,
                room_id
            );

            for other_user_id in room_users.iter() {
                if other_user_id != user_id {
                    self.send_message(&event_msg.to_string(), other_user_id);
                }
            }
        }
    }

    fn handle_test_message(&mut self, msg: TestSessionMessage) {
        log::info!("Handling test message: {:?}", msg.message_type);

        match msg.message_type {
            TestMessageType::RequestParticipants => {
                log::info!(
                    "Received request_participants from user {} in room {}",
                    msg.id,
                    msg.room_id
                );
                self.send_participants_list(&msg.room_id, &msg.id);
            }
            TestMessageType::StartTest => {
                if let Some(test_id) = msg.payload.get("test_id").and_then(|id| id.as_str()) {
                    self.active_tests.insert(msg.room_id, test_id.to_string());

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
                if let Some(room_users) = self.rooms.get(&msg.room_id) {
                    // Get student info for the answer
                    let student_info =
                        if let Some(anon_student) = self.anonymous_students.get(&msg.id) {
                            json!({
                                "student_id": anon_student.id,
                                "name": anon_student.name,
                                "is_anonymous": true
                            })
                        } else {
                            json!({
                                "student_id": msg.id.to_string(),
                                "name": format!("User {}", msg.id.to_string()[..8].to_uppercase()),
                                "is_anonymous": false
                            })
                        };

                    let answer_msg = json!({
                        "type": "student_answer",
                        "student_id": msg.id.to_string(),
                        "student_info": student_info,
                        "answer_data": msg.payload,
                    });

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
                self.active_tests.remove(&msg.room_id);

                if let Some(room_users) = self.rooms.get(&msg.room_id) {
                    let end_msg = json!({
                        "type": "test_ended",
                    });

                    for user_id in room_users.iter() {
                        self.send_message(&end_msg.to_string(), user_id);
                    }
                }
            }
            TestMessageType::QuestionFocus => {
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
        log::info!("User {} disconnecting from room {}", msg.id, msg.room_id);

        if self.sessions.remove(&msg.id).is_some() {
            // ENHANCED: Handle anonymous student disconnect
            if let Some(anon_student) = self.anonymous_students.remove(&msg.id) {
                log::info!(
                    "Anonymous student {} ({}) disconnected from test session",
                    anon_student.name,
                    anon_student.id
                );

                // Broadcast student left event with student info
                self.broadcast_user_event(
                    &msg.room_id,
                    &msg.id,
                    "student_left",
                    Some(json!({
                        "name": anon_student.name,
                        "student_id": anon_student.id,
                        "is_anonymous": true
                    })),
                );
            } else {
                // Broadcast regular user left event
                self.broadcast_user_event(&msg.room_id, &msg.id, "user_left", None);
            }

            // Remove user from room
            if let Some(lobby) = self.rooms.get_mut(&msg.room_id) {
                lobby.remove(&msg.id);

                if lobby.is_empty() {
                    self.rooms.remove(&msg.room_id);
                    log::info!("Room {} is now empty and removed", msg.room_id);

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
                } else {
                    log::info!("Room {} now has {} users", msg.room_id, lobby.len());
                }
            }

            // Remove role assignment
            self.room_roles.remove(&(msg.room_id, msg.id));

            // Update database session count
            let room_id = msg.room_id;
            if let Some(pool) = self.db_pool.clone() {
                actix::spawn(async move {
                    if let Err(e) =
                        websocket_session_database::update_session_user_count(room_id, false, &pool)
                            .await
                    {
                        log::error!("Failed to update session count: {}", e);
                    }
                });
            }
        }

        log::info!(
            "User {} successfully disconnected from room {}",
            msg.id,
            msg.room_id
        );
    }
}

#[cfg(feature = "ssr")]
impl Handler<Connect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        log::info!("User {} connecting to room {}", msg.self_id, msg.lobby_id);

        // Add user to the room
        self.rooms
            .entry(msg.lobby_id)
            .or_insert_with(HashSet::new)
            .insert(msg.self_id);

        // Improved role assignment logic
        if !self.room_roles.contains_key(&(msg.lobby_id, msg.self_id)) {
            let room_users_count = self.rooms.get(&msg.lobby_id).map_or(0, |users| users.len());

            // For now, assign teacher to first user in test sessions
            // You might want to pass user permissions through the Connect message
            let role = if room_users_count <= 1 {
                // First user in the room gets teacher role
                "teacher"
            } else {
                "student"
            };

            self.room_roles
                .insert((msg.lobby_id, msg.self_id), role.to_string());
            log::info!(
                "Assigned role '{}' to user {} in room {} (room size: {})",
                role,
                msg.self_id,
                msg.lobby_id,
                room_users_count
            );
        }

        // Store the connection
        self.sessions.insert(msg.self_id, msg.addr);

        // Send role assignment immediately with more detailed logging
        if let Some(role) = self.room_roles.get(&(msg.lobby_id, msg.self_id)) {
            let role_msg = json!({
                "type": "role_assigned",
                "role": role,
                "user_id": msg.self_id.to_string(),
                "room_id": msg.lobby_id.to_string()
            });

            log::info!(
                "Sending role assignment to user {}: {} (message: {})",
                msg.self_id,
                role,
                role_msg.to_string()
            );
            self.send_message(&role_msg.to_string(), &msg.self_id);
        } else {
            log::error!("Failed to assign role to user {}", msg.self_id);
        }

        // Broadcast user joined event to other participants
        self.broadcast_user_event(&msg.lobby_id, &msg.self_id, "user_joined", None);

        // Send welcome message with user ID
        let welcome_msg = json!({
            "type": "welcome",
            "user_id": msg.self_id.to_string(),
            "room_id": msg.lobby_id.to_string(),
            "message": format!("Welcome! Your ID is {}", msg.self_id)
        });
        self.send_message(&welcome_msg.to_string(), &msg.self_id);

        // Send current participants list to the new user
        self.send_participants_list(&msg.lobby_id, &msg.self_id);

        // If there's an active test, send test data
        if let Some(test_id) = self.active_tests.get(&msg.lobby_id) {
            if let Some(questions) = self.test_questions.get(test_id) {
                let role = self.room_roles.get(&(msg.lobby_id, msg.self_id)).unwrap();
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

        log::info!(
            "User {} successfully connected to room {}",
            msg.self_id,
            msg.lobby_id
        );
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

#[cfg(feature = "ssr")]
impl Handler<UserInfoMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: UserInfoMessage, _ctx: &mut Context<Self>) -> Self::Result {
        self.handle_user_info_message(msg.user_data, msg.user_id, msg.room_id);
    }
}

// NEW: Handler for anonymous student joins
#[cfg(feature = "ssr")]
#[derive(Message)]
#[rtype(result = "()")]
pub struct AnonymousStudentJoinMessage {
    pub student_data: serde_json::Value,
    pub user_id: Uuid,
    pub room_id: Uuid,
}

#[cfg(feature = "ssr")]
impl Handler<AnonymousStudentJoinMessage> for Lobby {
    type Result = ();

    fn handle(
        &mut self,
        msg: AnonymousStudentJoinMessage,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        self.handle_anonymous_student_join(msg.student_data, msg.user_id, msg.room_id);
    }
}
