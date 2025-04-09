use std::collections::HashMap;
use std::sync::{Arc, Mutex};
#[cfg(feature = "ssr")]
use {crate::app::websockets::actors::TestSessionActor, actix::Addr};

// Global state to store session actors
#[cfg(feature = "ssr")]
pub struct AppState {
    pub sessions: Arc<Mutex<HashMap<String, Addr<TestSessionActor>>>>,
}

#[cfg(feature = "ssr")]
impl AppState {
    pub fn new() -> Self {
        AppState {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
