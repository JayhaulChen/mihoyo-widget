use md5::{Md5, Digest};
use rand::Rng;
use serde::de::DeserializeOwned;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::config::Settings;

/// DS salt for game_record APIs (4X, client_type=5 web)
pub const SALT_X4: &str = "xV8v4Qu54lUKrEYFZkJhB8cuOh9Asafs";

/// Shared reqwest client (connection pool reused across all API calls)
pub fn shared_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Linux; Android 15; 2206123SC) AppleWebKit/537.36 miHoYoBBS/2.102.1")
            .build()
            .expect("Failed to create HTTP client")
    })
}

/// Generate DS2 for GET requests with sorted query params
pub fn ds2_get(salt: &str, query: &str) -> String {
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let mut rng = rand::thread_rng();
    let r = rng.gen_range(100001..=200000);

    let mut params: Vec<&str> = if query.is_empty() { vec![] } else { query.split('&').collect() };
    params.sort();
    let sorted_query = params.join("&");

    let main = format!("salt={}&t={}&r={}&b=&q={}", salt, t, r, sorted_query);
    let mut hasher = Md5::new();
    hasher.update(main.as_bytes());
    let ds = format!("{:x}", hasher.finalize());
    format!("{},{},{}", t, r, ds)
}

/// Standard miHoYo API response wrapper
#[derive(serde::Deserialize)]
pub struct ApiResponse<T> {
    retcode: i64,
    data: Option<T>,
}

/// Device fingerprint data (inner)
#[derive(serde::Deserialize)]
pub struct DeviceFpData {
    device_fp: String,
}

/// Common headers for game_record API (client_type=5, web)
pub fn build_web_headers(device_id: &str, device_fp: &str) -> reqwest::header::HeaderMap {
    use reqwest::header::*;
    fn hv(s: &str) -> HeaderValue { HeaderValue::from_str(s).unwrap_or_else(|_| HeaderValue::from_static("")) }
    let mut h = HeaderMap::new();
    h.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Linux; Android 15; 2206123SC) AppleWebKit/537.36 miHoYoBBS/2.102.1"));
    h.insert("x-rpc-client_type", HeaderValue::from_static("5"));
    h.insert("x-rpc-app_version", HeaderValue::from_static("2.102.1"));
    h.insert("x-rpc-device_id", hv(device_id));
    if !device_fp.is_empty() {
        h.insert("x-rpc-device_fp", hv(device_fp));
    }
    h.insert(REFERER, HeaderValue::from_static("https://webstatic.mihoyo.com"));
    h.insert(ACCEPT, HeaderValue::from_static("application/json, text/plain, */*"));
    h
}

// ── Unified API Client ──
// Shared base for game-specific API clients. Currently unused directly;
// game-hsr crate duplicates with HsrApiClient. Keep for future game crate reuse.

#[allow(dead_code)]
pub struct MihoyoApiClient {
    pub settings: Settings,
    pub cookie: String,
}

#[allow(dead_code)]
impl MihoyoApiClient {
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

    /// Generic GET → parse `{ retcode, data }` → return `T`
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

    // ── Shared utility methods ──

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
        let parsed: ApiResponse<DeviceFpData> = serde_json::from_str(&text)
            .map_err(|e| format!("Device FP parse error: {} body: {}", e, &text[..text.len().min(200)]))?;

        if parsed.retcode != 0 {
            return Err(format!("Device FP API error retcode={}", parsed.retcode));
        }
        parsed.data.map(|d| d.device_fp).ok_or_else(|| "No device_fp".into())
    }
}
