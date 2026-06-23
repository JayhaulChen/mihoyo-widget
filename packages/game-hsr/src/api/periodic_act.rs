/// Periodic activities — 差分宇宙赛季, 财富造物手段位
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PeriodicAct {
    /// List from `data.act_info_list`
    pub acts: Vec<ActSeason>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActSeason {
    #[serde(rename = "act_type")]
    pub season_type: String,
    pub season_id: String,
    pub season_level: String,
    pub season_name: String,
    pub division_level: String,
}

// Raw API wrapper: { data: { act_info_list: [...] } }
#[derive(serde::Deserialize)]
pub(crate) struct PeriodicActRaw {
    pub act_info_list: Vec<ActSeason>,
}

impl From<PeriodicActRaw> for PeriodicAct {
    fn from(raw: PeriodicActRaw) -> Self {
        PeriodicAct { acts: raw.act_info_list }
    }
}
