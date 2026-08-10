// Tauri-managed handle to the one live database connection. Nothing opens
// a real encrypted file into this yet — that's login (US-M8.1, S5). Tests
// populate it directly with `db::open_seeded_in_memory()`.
use std::sync::Mutex;

use rusqlite::Connection;

pub struct DbState(pub Mutex<Option<Connection>>);

impl DbState {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }

    pub fn with_connection(conn: Connection) -> Self {
        Self(Mutex::new(Some(conn)))
    }
}

impl Default for DbState {
    fn default() -> Self {
        Self::new()
    }
}
