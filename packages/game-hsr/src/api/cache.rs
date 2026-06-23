use mihoyo_core::cache::store::CacheDb;
use serde::{Deserialize, Serialize};
use crate::api::*;

/// All cached data returned to frontend on startup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllCachedData {
    pub widget: Option<WidgetData>,
    pub player: Option<PlayerInfo>,
    pub forgotten_hall: Option<ChallengeInfo>,
    pub pure_fiction: Option<ChallengeInfo>,
    pub apocalyptic_shadow: Option<ChallengeInfo>,
    pub ledger: Option<LedgerData>,
    pub banners: Option<BannerData>,
    pub periodic_act: Option<PeriodicAct>,
    pub challenge_peak: Option<PeakData>,
    pub rogue_archive: Option<RogueArchive>,
}

/// TTL constants (seconds)
pub const TTL_PLAYER: i64 = 3600;
pub const TTL_CHALLENGE: i64 = 14400;
pub const TTL_LEDGER: i64 = 21600;
pub const TTL_BANNER: i64 = 21600;

pub trait HsrCache {
    fn save_widget(&self, data: &WidgetData);
    fn get_latest(&self) -> Option<WidgetData>;
    fn save_player_info(&self, data: &PlayerInfo);
    fn get_player_info(&self) -> Option<PlayerInfo>;
    fn has_signed_today(&self) -> bool;
    fn last_check_time(&self) -> i64;
    fn should_refresh_player(&self) -> bool;
    fn save_challenges(&self, fh: &ChallengeInfo, pf: &ChallengeInfo, as_: &ChallengeInfo);
    fn save_ledger(&self, data: &LedgerData);
    fn save_banners(&self, data: &BannerData);
    fn save_periodic_act(&self, data: &PeriodicAct);
    fn save_peak(&self, data: &PeakData);
    fn save_rogue_archive(&self, data: &RogueArchive);
    fn get_all_cached(&self) -> AllCachedData;
    fn ledger_expired(&self) -> bool;
    fn banners_expired(&self) -> bool;
    fn challenge_expired(&self) -> bool;
    fn get_challenge(&self, key: &str) -> Option<ChallengeInfo>;
    fn get_ledger(&self) -> Option<LedgerData>;
    fn get_banners(&self) -> Option<BannerData>;
    fn get_periodic_act(&self) -> Option<PeriodicAct>;
    fn get_peak(&self) -> Option<PeakData>;
    fn get_rogue_archive(&self) -> Option<RogueArchive>;
}

impl HsrCache for CacheDb {
    fn save_widget(&self, data: &WidgetData) {
        if let Ok(json) = serde_json::to_string(data) {
            let now = chrono::Utc::now().timestamp();
            self.conn.execute(
                "INSERT OR REPLACE INTO kv_cache (key, json, created_at) VALUES (?1, ?2, ?3)",
                rusqlite::params!["widget_data", json, now],
            ).ok();
        }
    }

    fn get_latest(&self) -> Option<WidgetData> {
        self.kv_get("widget_data").and_then(|(json, _)| serde_json::from_str(&json).ok())
    }

    fn save_player_info(&self, data: &PlayerInfo) {
        if let Ok(json) = serde_json::to_string(data) {
            self.kv_set("player_info", &json);
        }
    }

    fn get_player_info(&self) -> Option<PlayerInfo> {
        self.kv_get("player_info").and_then(|(json, _)| serde_json::from_str(&json).ok())
    }

    fn has_signed_today(&self) -> bool {
        self.get_latest().map(|d| d.has_signed).unwrap_or(false)
    }

    fn last_check_time(&self) -> i64 {
        self.kv_get("widget_data").map(|(_, t)| t).unwrap_or(0)
    }

    fn should_refresh_player(&self) -> bool {
        let last = self.kv_get("player_info").map(|(_, t)| t).unwrap_or(0);
        chrono::Utc::now().timestamp() - last > TTL_PLAYER
    }

    fn save_challenges(&self, fh: &ChallengeInfo, pf: &ChallengeInfo, as_: &ChallengeInfo) {
        if let Ok(json) = serde_json::to_string(fh) { self.kv_set("forgotten_hall", &json); }
        if let Ok(json) = serde_json::to_string(pf) { self.kv_set("pure_fiction", &json); }
        if let Ok(json) = serde_json::to_string(as_) { self.kv_set("apocalyptic_shadow", &json); }
    }

    fn get_challenge(&self, key: &str) -> Option<ChallengeInfo> {
        self.kv_get(key).and_then(|(json, _)| serde_json::from_str(&json).ok())
    }

    fn challenge_expired(&self) -> bool {
        self.kv_is_expired("forgotten_hall", TTL_CHALLENGE)
    }

    fn save_ledger(&self, data: &LedgerData) {
        if let Ok(json) = serde_json::to_string(data) { self.kv_set("ledger", &json); }
    }

    fn get_ledger(&self) -> Option<LedgerData> {
        self.kv_get("ledger").and_then(|(json, _)| serde_json::from_str(&json).ok())
    }

    fn ledger_expired(&self) -> bool {
        self.kv_is_expired("ledger", TTL_LEDGER)
    }

    fn save_banners(&self, data: &BannerData) {
        if let Ok(json) = serde_json::to_string(data) { self.kv_set("banners", &json); }
    }

    fn get_banners(&self) -> Option<BannerData> {
        self.kv_get("banners").and_then(|(json, _)| serde_json::from_str(&json).ok())
    }

    fn banners_expired(&self) -> bool {
        self.kv_is_expired("banners", TTL_BANNER)
    }

    fn save_periodic_act(&self, data: &PeriodicAct) {
        if let Ok(json) = serde_json::to_string(data) { self.kv_set("periodic_act", &json); }
    }

    fn get_periodic_act(&self) -> Option<PeriodicAct> {
        self.kv_get("periodic_act").and_then(|(json, _)| serde_json::from_str(&json).ok())
    }

    fn save_peak(&self, data: &PeakData) {
        if let Ok(json) = serde_json::to_string(data) { self.kv_set("challenge_peak", &json); }
    }

    fn get_peak(&self) -> Option<PeakData> {
        self.kv_get("challenge_peak").and_then(|(json, _)| serde_json::from_str(&json).ok())
    }

    fn save_rogue_archive(&self, data: &RogueArchive) {
        if let Ok(json) = serde_json::to_string(data) { self.kv_set("rogue_archive", &json); }
    }

    fn get_rogue_archive(&self) -> Option<RogueArchive> {
        self.kv_get("rogue_archive").and_then(|(json, _)| serde_json::from_str(&json).ok())
    }

    fn get_all_cached(&self) -> AllCachedData {
        AllCachedData {
            widget: self.get_latest(),
            player: self.get_player_info(),
            forgotten_hall: self.get_challenge("forgotten_hall"),
            pure_fiction: self.get_challenge("pure_fiction"),
            apocalyptic_shadow: self.get_challenge("apocalyptic_shadow"),
            ledger: self.get_ledger(),
            banners: self.get_banners(),
            periodic_act: self.get_periodic_act(),
            challenge_peak: self.get_peak(),
            rogue_archive: self.get_rogue_archive(),
        }
    }
}
