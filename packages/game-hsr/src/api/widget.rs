/// Note API response from HSR — real-time game info
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WidgetData {
    pub current_stamina: u64,
    pub max_stamina: u64,
    pub stamina_recover_time: u64,
    #[serde(alias = "accepted_epedition_num")]
    pub accepted_expedition_num: u64,
    pub total_expedition_num: u64,
    pub expeditions: Vec<Expedition>,
    pub current_train_score: u64,
    pub max_train_score: u64,
    pub current_rogue_score: u64,
    pub max_rogue_score: u64,
    #[serde(default)]
    pub current_reserve_stamina: u64,
    #[serde(default)]
    pub is_reserve_stamina_full: bool,
    // Weekly boss (历战余响)
    #[serde(default)]
    pub weekly_cocoon_cnt: u64,
    #[serde(default)]
    pub weekly_cocoon_limit: u64,
    // Divergent universe (差分宇宙周期演算)
    #[serde(default)]
    pub rogue_tourn_weekly_cur: u64,
    #[serde(default)]
    pub rogue_tourn_weekly_max: u64,
    #[serde(alias = "rogue_tourn_weekly_unlocked", default)]
    pub rogue_tourn_week_unlocked: bool,
    // Currency war (货币战争)
    #[serde(default)]
    pub grid_fight_weekly_cur: u64,
    #[serde(default)]
    pub grid_fight_weekly_max: u64,
    #[serde(default)]
    pub has_signed: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Expedition {
    pub avater_id: u64,
    pub avatar_name: String,
    pub remaining_time: u64,
    pub item_url: String,
    pub name: String,
    pub status: String,
}
