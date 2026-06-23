/// Deserialize i64 that tolerates `""` or other non-number → returns 0
fn de_i64_or_default<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
    match v {
        serde_json::Value::Number(n) => n.as_i64().ok_or_else(|| serde::de::Error::custom("not i64")),
        _ => Ok(0),
    }
}

/// Deserialize miHoYo date format — either object `{year,month,day,hour,minute}`
/// or plain string. Public so other modules can reuse.
pub fn de_mihoyo_date<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
    match v {
        serde_json::Value::Object(m) => {
            let y = m.get("year").and_then(|v| v.as_i64()).unwrap_or(0);
            let mo = m.get("month").and_then(|v| v.as_i64()).unwrap_or(1);
            let d = m.get("day").and_then(|v| v.as_i64()).unwrap_or(1);
            let h = m.get("hour").and_then(|v| v.as_i64()).unwrap_or(0);
            let mi = m.get("minute").and_then(|v| v.as_i64()).unwrap_or(0);
            Ok(format!("{:04}-{:02}-{:02} {:02}:{:02}", y, mo, d, h, mi))
        }
        serde_json::Value::String(s) => Ok(s),
        _ => Ok(String::new()),
    }
}

/// Challenge mode data — light struct with only widget-relevant fields
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChallengeInfo {
    #[serde(default, deserialize_with = "de_i64_or_default")]
    pub schedule_id: i64,
    #[serde(default, deserialize_with = "de_mihoyo_date")]
    pub begin_time: String,
    #[serde(default, deserialize_with = "de_mihoyo_date")]
    pub end_time: String,
    #[serde(default, deserialize_with = "de_i64_or_default")]
    pub star_num: i64,
    #[serde(default)]
    pub max_star: i64,
    #[serde(default)]
    pub has_data: bool,
}

/// Forgotten Hall (忘却之庭) raw response — flat structure
#[derive(serde::Deserialize, Default)]
pub(crate) struct ForgottenHallRaw {
    #[serde(default, deserialize_with = "de_i64_or_default")]
    pub schedule_id: i64,
    #[serde(default, deserialize_with = "de_mihoyo_date")]
    pub begin_time: String,
    #[serde(default, deserialize_with = "de_mihoyo_date")]
    pub end_time: String,
    #[serde(default, deserialize_with = "de_i64_or_default")]
    pub star_num: i64,
    #[serde(default, deserialize_with = "de_i64_or_default")]
    pub max_floor: i64,
    #[serde(default)]
    pub has_data: bool,
}

/// Group item inside `challenge_story` / `challenge_boss` `groups` array
#[derive(serde::Deserialize)]
pub(crate) struct ChallengeGroupItem {
    #[serde(default, deserialize_with = "de_i64_or_default")]
    pub schedule_id: i64,
    #[serde(default, deserialize_with = "de_mihoyo_date")]
    pub begin_time: String,
    #[serde(default, deserialize_with = "de_mihoyo_date")]
    pub end_time: String,
    #[serde(default, deserialize_with = "de_i64_or_default")]
    pub star_num: i64,
    #[serde(default, deserialize_with = "de_i64_or_default")]
    pub max_floor: i64,
}

/// Groups wrapper — `{ groups: [...] }` from challenge_story / challenge_boss API
#[derive(serde::Deserialize)]
pub(crate) struct ChallengeGroupsData {
    #[serde(default)]
    pub groups: Vec<ChallengeGroupItem>,
}

/// Pure Fiction (虚构叙事) raw response
#[derive(serde::Deserialize, Default)]
pub(crate) struct PureFictionRaw {
    #[serde(default, deserialize_with = "de_i64_or_default")]
    pub schedule_id: i64,
    #[serde(default, deserialize_with = "de_mihoyo_date")]
    pub begin_time: String,
    #[serde(default, deserialize_with = "de_mihoyo_date")]
    pub end_time: String,
    #[serde(default, deserialize_with = "de_i64_or_default")]
    pub star_num: i64,
    #[serde(default, deserialize_with = "de_i64_or_default")]
    pub max_floor: i64,
    #[serde(default)]
    pub has_data: bool,
}

/// Apocalyptic Shadow (末日幻影) raw response
#[derive(serde::Deserialize, Default)]
pub(crate) struct ApocalypticShadowRaw {
    #[serde(default, deserialize_with = "de_i64_or_default")]
    pub schedule_id: i64,
    #[serde(default, deserialize_with = "de_mihoyo_date")]
    pub begin_time: String,
    #[serde(default, deserialize_with = "de_mihoyo_date")]
    pub end_time: String,
    #[serde(default, deserialize_with = "de_i64_or_default")]
    pub star_num: i64,
    #[serde(default, deserialize_with = "de_i64_or_default")]
    pub max_floor: i64,
    #[serde(default)]
    pub has_data: bool,
}

impl From<ChallengeGroupsData> for PureFictionRaw {
    fn from(d: ChallengeGroupsData) -> Self {
        d.groups.into_iter().next().map(|g| PureFictionRaw {
            schedule_id: g.schedule_id,
            begin_time: g.begin_time,
            end_time: g.end_time,
            star_num: g.star_num,
            max_floor: g.max_floor,
            has_data: true,
        }).unwrap_or_default()
    }
}

impl From<ChallengeGroupsData> for ApocalypticShadowRaw {
    fn from(d: ChallengeGroupsData) -> Self {
        d.groups.into_iter().next().map(|g| ApocalypticShadowRaw {
            schedule_id: g.schedule_id,
            begin_time: g.begin_time,
            end_time: g.end_time,
            star_num: g.star_num,
            max_floor: g.max_floor,
            has_data: true,
        }).unwrap_or_default()
    }
}

impl From<ForgottenHallRaw> for ChallengeInfo {
    fn from(r: ForgottenHallRaw) -> Self {
        ChallengeInfo {
            schedule_id: r.schedule_id,
            begin_time: r.begin_time,
            end_time: r.end_time,
            star_num: r.star_num,
            max_star: r.max_floor * 3,
            has_data: r.has_data,
        }
    }
}

impl From<PureFictionRaw> for ChallengeInfo {
    fn from(r: PureFictionRaw) -> Self {
        ChallengeInfo {
            schedule_id: r.schedule_id,
            begin_time: r.begin_time,
            end_time: r.end_time,
            star_num: r.star_num,
            max_star: if r.max_floor > 0 { r.max_floor * 3 } else { 12 },
            has_data: r.has_data,
        }
    }
}

impl From<ApocalypticShadowRaw> for ChallengeInfo {
    fn from(r: ApocalypticShadowRaw) -> Self {
        ChallengeInfo {
            schedule_id: r.schedule_id,
            begin_time: r.begin_time,
            end_time: r.end_time,
            star_num: r.star_num,
            max_star: if r.max_floor > 0 { r.max_floor * 3 } else { 12 },
            has_data: r.has_data,
        }
    }
}
