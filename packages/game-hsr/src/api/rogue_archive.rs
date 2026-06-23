/// Simulated Universe permanent archives — 虫灾/智识/黄金
/// All three are nearly static progress that changes only on game version updates.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct RogueArchive {
    /// 智识令使 (Nous) — from api/rogue_nous
    pub nous_progress: String,
    pub nous_miracle: i64,
    pub nous_nerve: i64,
    /// 黄金与机械 (Magic) — from api/rogue_magic
    pub magic_linear: String,
    pub magic_compendium: String,
    pub magic_secrets: String,
    /// 寰宇蝗灾 (Locust) — from api/rogue_locust
    pub locust_narrow: i64,
    pub locust_miracle: i64,
    pub locust_event: i64,
    pub locust_destinies: Vec<DestinyLevel>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DestinyLevel {
    pub name: String,
    pub level: i64,
}

// ── Raw API types ──

#[derive(Debug, Clone, serde::Deserialize)]
pub struct NousRaw {
    pub basic: NousBasic,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct NousBasic {
    pub cur_progress: i64,
    pub max_progress: i64,
    pub max_rolling: i64,
    pub active_nerve: i64,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MagicRaw {
    pub basic_info: MagicBasic,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MagicBasic {
    pub linear_tree_num: String,
    pub magic_compendium: String,
    pub discover_secrets: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct LocustRaw {
    pub basic: LocustBasic,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct LocustBasic {
    pub cnt: LocustCnt,
    pub destiny: Vec<DestinyRaw>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct LocustCnt {
    pub narrow: i64,
    pub miracle: i64,
    pub event: i64,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DestinyRaw {
    pub desc: String,
    pub level: i64,
}

impl From<NousRaw> for RogueArchive {
    fn from(n: NousRaw) -> Self {
        RogueArchive {
            nous_progress: format!("{}/{}", n.basic.cur_progress, n.basic.max_progress),
            nous_miracle: n.basic.max_rolling,
            nous_nerve: n.basic.active_nerve,
            magic_linear: String::new(),
            magic_compendium: String::new(),
            magic_secrets: String::new(),
            locust_narrow: 0,
            locust_miracle: 0,
            locust_event: 0,
            locust_destinies: vec![],
        }
    }
}

impl From<MagicRaw> for RogueArchive {
    fn from(m: MagicRaw) -> Self {
        RogueArchive {
            nous_progress: String::new(),
            nous_miracle: 0,
            nous_nerve: 0,
            magic_linear: m.basic_info.linear_tree_num,
            magic_compendium: m.basic_info.magic_compendium,
            magic_secrets: m.basic_info.discover_secrets,
            locust_narrow: 0,
            locust_miracle: 0,
            locust_event: 0,
            locust_destinies: vec![],
        }
    }
}

impl From<LocustRaw> for RogueArchive {
    fn from(l: LocustRaw) -> Self {
        RogueArchive {
            nous_progress: String::new(),
            nous_miracle: 0,
            nous_nerve: 0,
            magic_linear: String::new(),
            magic_compendium: String::new(),
            magic_secrets: String::new(),
            locust_narrow: l.basic.cnt.narrow,
            locust_miracle: l.basic.cnt.miracle,
            locust_event: l.basic.cnt.event,
            locust_destinies: l.basic.destiny.into_iter()
                .map(|d| DestinyLevel { name: d.desc, level: d.level })
                .collect(),
        }
    }
}

/// Merge three archive sources into one struct
impl RogueArchive {
    pub fn merge(self, other: RogueArchive) -> RogueArchive {
        RogueArchive {
            nous_progress: or_else(self.nous_progress, other.nous_progress),
            nous_miracle: or_default(self.nous_miracle, other.nous_miracle),
            nous_nerve: or_default(self.nous_nerve, other.nous_nerve),
            magic_linear: or_else(self.magic_linear, other.magic_linear),
            magic_compendium: or_else(self.magic_compendium, other.magic_compendium),
            magic_secrets: or_else(self.magic_secrets, other.magic_secrets),
            locust_narrow: or_default(self.locust_narrow, other.locust_narrow),
            locust_miracle: or_default(self.locust_miracle, other.locust_miracle),
            locust_event: or_default(self.locust_event, other.locust_event),
            locust_destinies: if self.locust_destinies.is_empty() { other.locust_destinies } else { self.locust_destinies },
        }
    }
}

fn or_else(a: String, b: String) -> String {
    if a.is_empty() { b } else { a }
}

fn or_default(a: i64, b: i64) -> i64 {
    if a == 0 { b } else { a }
}
