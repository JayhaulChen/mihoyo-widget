#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LedgerData {
    pub current_hcoin: i64,
    pub current_rails_pass: i64,
    pub last_hcoin: i64,
    pub last_rails_pass: i64,
    pub hcoin_rate: f64,
    pub rails_rate: f64,
}

/// Per-action breakdown of ledger for a specific month+type
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LedgerDetail {
    pub list: Vec<LedgerDetailItem>,
    pub total: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LedgerDetailItem {
    pub action: String,
    pub action_name: String,
    pub time: String,
    pub number: i32,
}
