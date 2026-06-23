#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerInfo {
    pub stats: Stats,
    pub avatar_list: Vec<Avatar>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Stats {
    pub active_days: u64,
    pub avatar_num: u64,
    pub achievement_num: u64,
    pub chest_num: u64,
    pub abyss_process: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Avatar {
    pub id: u64,
    pub level: u64,
    pub name: String,
    pub element: String,
    pub icon: String,
    pub rarity: u64,
}
