use serde::Serialize;

#[derive(Serialize)]
pub struct Stats {
    pub ws_connected: u32,
    pub sessions_active: u32,
    pub unique_users: u64,
}

#[derive(Serialize)]
pub struct BasicSessionInfo {
    pub id: String,
    pub player_count: u32,
    pub active: bool,
    pub created: u64,
}

#[derive(Serialize)]
pub struct PlayerData {
    pub user_id: u32,
    pub name: String,
    // TODO typed roles
    pub role: Option<String>,
    pub joined: u64,
    pub state: String,
}
