use mihoyo_core::api::client::{build_web_headers, ds2_get, shared_client, SALT_X4};
use mihoyo_core::config::Settings;
use serde::de::DeserializeOwned;
use std::time::{SystemTime, UNIX_EPOCH};

/// Standard miHoYo API response wrapper
#[derive(serde::Deserialize)]
struct ApiResponse<T> {
    retcode: i64,
    data: Option<T>,
}

pub struct HsrApiClient {
    pub settings: Settings,
    pub cookie: String,
}

impl HsrApiClient {
    pub fn new(settings: Settings) -> Self {
        let cookie = if !settings.cookie.is_empty() {
            settings.cookie.clone()
        } else {
            settings.stoken_cookie()
        };
        Self { settings, cookie }
    }

    /// Build headers from current device info
    fn headers(&self) -> reqwest::header::HeaderMap {
        build_web_headers(&self.settings.device_id, &self.settings.device_fp)
    }

    /// Generic GET -> parse `{ retcode, data }` -> return `T`
    async fn get<T: DeserializeOwned>(&self, url: &str, query: &str) -> Result<T, String> {
        let ds = ds2_get(SALT_X4, query);
        let resp = shared_client()
            .get(url)
            .headers(self.headers())
            .header("Cookie", &self.cookie)
            .header("DS", &ds)
            .send()
            .await
            .map_err(|e| format!("HTTP error: {}", e))?;

        let text = resp.text().await.map_err(|e| format!("Read error: {}", e))?;
        let parsed: ApiResponse<T> = serde_json::from_str(&text)
            .map_err(|e| format!("Parse error: {} body: {}", e, &text[..text.len().min(200)]))?;

        if parsed.retcode != 0 {
            return Err(format!("API error retcode={}", parsed.retcode));
        }
        parsed.data.ok_or_else(|| "No data in response".into())
    }

    // -- Business methods --

    /// Real-time note (stamina, expeditions, rogue, etc.)
    pub async fn get_note(&self) -> Result<crate::api::widget::WidgetData, String> {
        let q = format!(
            "role_id={}&server={}",
            self.settings.uid, self.settings.region
        );
        let url = format!(
            "https://api-takumi-record.mihoyo.com/game_record/app/hkrpg/api/note?{}",
            q
        );
        let mut data: crate::api::widget::WidgetData = self.get(&url, &q).await?;
        // Merge sign-in status (non-fatal)
        match self.check_sign_status().await {
            Ok(signed) => data.has_signed = signed,
            Err(e) => log::warn!("Sign check failed: {}", e),
        }
        Ok(data)
    }

    /// Player index (stats, avatar list)
    pub async fn get_player_index(&self) -> Result<crate::api::player::PlayerInfo, String> {
        let q = format!(
            "role_id={}&server={}",
            self.settings.uid, self.settings.region
        );
        let url = format!(
            "https://api-takumi-record.mihoyo.com/game_record/app/hkrpg/api/index?{}",
            q
        );
        self.get(&url, &q).await
    }

    /// Monthly ledger summary (star jade / rail pass)
    pub async fn get_ledger(&self) -> Result<crate::api::ledger::LedgerData, String> {
        let q = format!(
            "server={}&role_id={}",
            self.settings.region, self.settings.uid
        );
        let url = format!(
            "https://api-takumi-record.mihoyo.com/game_record/app/hkrpg/api/get_ledger_month_info?{}",
            q
        );
        self.get(&url, &q).await
    }

    /// Ledger detail (per-action breakdown for a month)
    #[allow(dead_code)]
    pub async fn get_ledger_detail(
        &self,
        month: &str,
        type_id: i32,
        page: i32,
    ) -> Result<crate::api::ledger::LedgerDetail, String> {
        let q = format!(
            "uid={}&region={}&month={}&type={}&current_page={}&page_size=20",
            self.settings.uid, self.settings.region, month, type_id, page
        );
        let url = format!(
            "https://api-takumi.mihoyo.com/event/srledger/month_detail?{}",
            q
        );
        self.get(&url, &q).await
    }

    /// Banner / event calendar
    pub async fn get_banners(&self) -> Result<crate::api::banner::BannerData, String> {
        let q = format!(
            "server={}&role_id={}",
            self.settings.region, self.settings.uid
        );
        let url = format!(
            "https://api-takumi-record.mihoyo.com/game_record/app/hkrpg/api/get_act_calender?{}",
            q
        );
        let raw: crate::api::banner::ActCalenderData = self.get(&url, &q).await?;
        Ok(crate::api::banner::convert_to_banner_data(raw))
    }

    /// Check HSR daily sign-in status via luna API
    pub async fn check_sign_status(&self) -> Result<bool, String> {
        let q = format!(
            "lang=zh-cn&act_id=e202304121516551&region={}&uid={}",
            self.settings.region, self.settings.uid
        );
        let url = format!(
            "https://api-takumi.mihoyo.com/event/luna/info?{}",
            q
        );
        #[derive(serde::Deserialize)]
        struct SignStatus {
            is_sign: bool,
        }
        let data: SignStatus = self.get(&url, &q).await?;
        Ok(data.is_sign)
    }

    /// Forgotten Hall (forgotten hall) current season
    pub async fn get_forgotten_hall(
        &self,
    ) -> Result<crate::api::challenge::ChallengeInfo, String> {
        let q = format!(
            "schedule_type=1&server={}&role_id={}&need_all=true",
            self.settings.region, self.settings.uid
        );
        let url = format!(
            "https://api-takumi-record.mihoyo.com/game_record/app/hkrpg/api/challenge?{}",
            q
        );
        let raw: crate::api::challenge::ForgottenHallRaw = self.get(&url, &q).await?;
        Ok(raw.into())
    }

    /// Pure Fiction (pure fiction) current season
    pub async fn get_pure_fiction(
        &self,
    ) -> Result<crate::api::challenge::ChallengeInfo, String> {
        let q = format!(
            "schedule_type=1&server={}&role_id={}&isPrev=1&need_all=true",
            self.settings.region, self.settings.uid
        );
        let url = format!(
            "https://api-takumi-record.mihoyo.com/game_record/app/hkrpg/api/challenge_story?{}",
            q
        );
        let groups: crate::api::challenge::ChallengeGroupsData = self.get(&url, &q).await?;
        let raw: crate::api::challenge::PureFictionRaw = groups.into();
        Ok(raw.into())
    }

    /// Apocalyptic Shadow (apocalyptic shadow) current season
    pub async fn get_apocalyptic_shadow(
        &self,
    ) -> Result<crate::api::challenge::ChallengeInfo, String> {
        let q = format!(
            "schedule_type=1&server={}&role_id={}&isPrev=1&need_all=true",
            self.settings.region, self.settings.uid
        );
        let url = format!(
            "https://api-takumi-record.mihoyo.com/game_record/app/hkrpg/api/challenge_boss?{}",
            q
        );
        let groups: crate::api::challenge::ChallengeGroupsData = self.get(&url, &q).await?;
        let raw: crate::api::challenge::ApocalypticShadowRaw = groups.into();
        Ok(raw.into())
    }

    /// Periodic activities (periodic activities)
    pub async fn get_periodic_act(
        &self,
    ) -> Result<crate::api::periodic_act::PeriodicAct, String> {
        let q = format!(
            "server={}&role_id={}",
            self.settings.region, self.settings.uid
        );
        let url = format!(
            "https://api-takumi-record.mihoyo.com/game_record/app/hkrpg/api/periodic_act?{}",
            q
        );
        let raw: crate::api::periodic_act::PeriodicActRaw = self.get(&url, &q).await?;
        Ok(raw.into())
    }

    /// Challenge Peak (challenge peak) — endgame boss rush
    pub async fn get_challenge_peak(&self) -> Result<crate::api::peak::PeakData, String> {
        let q = format!(
            "server={}&role_id={}&schedule_type=1&need_all=true",
            self.settings.region, self.settings.uid
        );
        let url = format!(
            "https://api-takumi-record.mihoyo.com/game_record/app/hkrpg/api/challenge_peak?{}",
            q
        );
        let raw: crate::api::peak::PeakRaw = self.get(&url, &q).await?;
        Ok(raw.into())
    }

    /// Rogue Nous (rogue nous) archive progress
    pub async fn get_rogue_nous(
        &self,
    ) -> Result<crate::api::rogue_archive::NousRaw, String> {
        let q = format!(
            "server={}&role_id={}",
            self.settings.region, self.settings.uid
        );
        let url = format!(
            "https://api-takumi-record.mihoyo.com/game_record/app/hkrpg/api/rogue_nous?{}",
            q
        );
        self.get(&url, &q).await
    }

    /// Rogue Magic (rogue magic) archive progress
    pub async fn get_rogue_magic(
        &self,
    ) -> Result<crate::api::rogue_archive::MagicRaw, String> {
        let q = format!(
            "server={}&role_id={}",
            self.settings.region, self.settings.uid
        );
        let url = format!(
            "https://api-takumi-record.mihoyo.com/game_record/app/hkrpg/api/rogue_magic?{}",
            q
        );
        self.get(&url, &q).await
    }

    /// Rogue Locust (rogue locust) archive progress
    pub async fn get_rogue_locust(
        &self,
    ) -> Result<crate::api::rogue_archive::LocustRaw, String> {
        let q = format!(
            "server={}&role_id={}",
            self.settings.region, self.settings.uid
        );
        let url = format!(
            "https://api-takumi-record.mihoyo.com/game_record/app/hkrpg/api/rogue_locust?{}",
            q
        );
        self.get(&url, &q).await
    }

    /// Register device fingerprint via miHoYo device-fp API
    pub async fn register_device_fp(&self) -> Result<String, String> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let seed_id = format!("{:016x}", rand::random::<u64>());
        let seed_time = now.as_millis().to_string();

        let body = serde_json::json!({
            "device_id": self.settings.device_id,
            "seed_id": seed_id,
            "seed_time": seed_time,
            "platform": "2",
            "device_fp": self.settings.device_fp,
            "app_name": "bbs_cn",
            "ext_fields": "{\"proxyStatus\":0,\"isRoot\":0,\"romCapacity\":\"512\",\"deviceName\":\"Pixel5\",\"productName\":\"6QG7C9\",\"romRemain\":\"512\",\"hostname\":\"db1ba5f7c000000\",\"screenSize\":\"1080x2400\",\"isTablet\":0,\"aaid\":\"\",\"model\":\"Pixel5\",\"brand\":\"google\",\"hardware\":\"windows_x86_64\",\"deviceType\":\"redfin\",\"devId\":\"REL\",\"serialNumber\":\"unknown\",\"sdCapacity\":125943,\"buildTime\":\"1704316741000\",\"buildUser\":\"cloudtest\",\"simState\":0,\"ramRemain\":\"124603\",\"appUpdateTimeDiff\":1716369357492,\"deviceInfo\":\"google\\/6QG7C9\\/redfin:13\\/TQ3A.230901.001\\/2311.40000.5.0:user\\/release-keys\",\"vaid\":\"\",\"buildType\":\"user\",\"sdkVersion\":\"33\",\"ui_mode\":\"UI_MODE_TYPE_NORMAL\",\"isMockLocation\":0,\"cpuType\":\"arm64-v8a\",\"isAirMode\":0,\"ringMode\":2,\"chargeStatus\":3,\"manufacturer\":\"Google\",\"emulatorStatus\":0,\"appMemory\":\"512\",\"osVersion\":\"13\",\"vendor\":\"unknown\",\"accelerometer\":\"\",\"sdRemain\":123276,\"buildTags\":\"release-keys\",\"packageName\":\"com.mihoyo.hyperion\",\"networkType\":\"WiFi\",\"oaid\":\"\",\"debugStatus\":1,\"ramCapacity\":\"125943\",\"magnetometer\":\"\",\"display\":\"TQ3A.230901.001\",\"appInstallTimeDiff\":1706444666737,\"packageVersion\":\"2.20.2\",\"gyroscope\":\"\",\"batteryStatus\":85,\"hasKeyboard\":10,\"board\":\"windows\"}",
            "bbs_device_id": self.settings.device_id,
        });

        let url = "https://public-data-api.mihoyo.com/device-fp/api/getFp";
        let resp = shared_client()
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Device FP HTTP error: {}", e))?;

        let text = resp.text().await.map_err(|e| format!("Device FP read error: {}", e))?;
        #[derive(serde::Deserialize)]
        struct FpApiResponse<T> {
            retcode: i64,
            data: Option<T>,
        }
        #[derive(serde::Deserialize)]
        struct DeviceFpData {
            device_fp: String,
        }
        let parsed: FpApiResponse<DeviceFpData> = serde_json::from_str(&text)
            .map_err(|e| format!("Device FP parse error: {} body: {}", e, &text[..text.len().min(200)]))?;

        if parsed.retcode != 0 {
            return Err(format!("Device FP API error retcode={}", parsed.retcode));
        }
        parsed.data.map(|d| d.device_fp).ok_or_else(|| "No device_fp".into())
    }
}
