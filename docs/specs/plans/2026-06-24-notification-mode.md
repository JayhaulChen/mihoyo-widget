# 纯系统通知模式 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**目标:** 添加纯系统通知模式——系统托盘切换窗口/通知模式，通知规则可自定义（开关+时间），含每日摘要。

**架构:** `NotificationConfig` 挂载在 `Settings` 上，序列化到 `runtime.json`。`check_rules` 加 `&NotificationConfig` 参数做条件判断。`run_poller` 逻辑不变，通知模式只隐藏窗口，WebView 进程仍在。

**Tech Stack:** Rust (Tauri), Vanilla JS frontend, tauri-plugin-notification

**全局约束:**
- 遵循现有项目结构: packages/core (共享), packages/game-hsr (星铁), apps/desktop (Tauri壳), packages/frontend (Vanilla JS + Vite)
- `check_rules` 调用位置不可变（force_refresh 和 run_poller 各一处）
- `NotificationConfig::default()` 默认为通知全开，`notification_mode: false`
- 前端设置页用现有 `save_config` command 保存，不新增 Tauri command
- 时间格式统一: `"HH:MM"`（每天）或 `"EEE HH:MM"`（每周，三字母英文缩写）

---

### Task 1: NotificationConfig 数据结构

**Files:**
- Modify: `packages/core/src/config/settings.rs`

**Interfaces:**
- Produces: `Settings::notification: NotificationConfig`, `NotificationConfig` struct with Default impl

- [ ] **Step 1: 在 Settings 结构体后追加 NotificationConfig**

`packages/core/src/config/settings.rs`:

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct NotificationConfig {
    pub notification_mode: bool,
    pub stamina_enabled: bool,
    pub stamina_threshold_mild: f64,
    pub stamina_threshold_urgent: f64,
    pub expedition_enabled: bool,
    pub reserve_stamina_enabled: bool,
    pub sign_reminder_enabled: bool,
    pub sign_reminder_time: String,
    pub rogue_reminder_enabled: bool,
    pub rogue_reminder_time: String,
    pub digest_enabled: bool,
    pub digest_time: String,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            notification_mode: false,
            stamina_enabled: true,
            stamina_threshold_mild: 0.80,
            stamina_threshold_urgent: 0.95,
            expedition_enabled: true,
            reserve_stamina_enabled: true,
            sign_reminder_enabled: true,
            sign_reminder_time: "20:00".into(),
            rogue_reminder_enabled: true,
            rogue_reminder_time: "Sun 20:00".into(),
            digest_enabled: false,
            digest_time: "09:00".into(),
        }
    }
}
```

- [ ] **Step 2: 在 Settings 结构体追加 notification 字段**

```rust
pub struct Settings {
    // ... 现有字段不变 ...
    pub poll_interval_secs: u64,
    #[serde(default)]
    pub notification: NotificationConfig,
}
```

注意: `#[serde(default)]` 保证旧版 `runtime.json`（无 notification 字段）也能正常反序列化。

- [ ] **Step 3: 更新 Settings::from_json 和 from_env**

`from_json` 尾部加一行:
```rust
notification: json.get("notification").map(|v| serde_json::from_value(v.clone()).unwrap_or_default()).unwrap_or_default(),
```

`from_env` 尾部加一行（环境变量不覆盖通知配置，全默认）:
```rust
// notification 字段由 runtime.json 或默认值决定，环境变量不覆盖
```

- [ ] **Step 4: 编译检查**

```bash
cargo check 2>&1 | head -20
```
期望: 无错误

- [ ] **Step 5: Commit**

```bash
git add packages/core/src/config/settings.rs
git commit -m "feat: add NotificationConfig struct with defaults"
```

---

### Task 2: 通知规则引擎 — `is_time_reached`、config 感知规则、每日摘要

**Files:**
- Modify: `packages/game-hsr/src/notify/rules.rs`
- Modify: `packages/game-hsr/src/notify/mod.rs`

**Interfaces:**
- Consumes: `NotificationConfig` from `mihoyo_core::config::settings`
- Produces: `check_rules(data, old, app, config)`, `check_digest(data, app, config)`

- [ ] **Step 1: 导入 NotificationConfig 并修改 check_rules 签名**

`packages/game-hsr/src/notify/rules.rs` 现有导入加:
```rust
use mihoyo_core::config::settings::NotificationConfig;
```

函数签名改:
```rust
pub fn check_rules(
    data: &WidgetData,
    old: Option<&WidgetData>,
    app: &tauri::AppHandle,
    config: &NotificationConfig,
) {
```

- [ ] **Step 2: 添加 is_time_reached 工具函数**

```rust
/// 判断当前时间是否已到达指定时刻。
///
/// 格式:
///   "20:00"        — 每天, 当日 HH:MM 已过则 true
///   "Sun 20:00"   — 每周固定日, 当日为指定星期且 HH:MM 已过则 true
fn is_time_reached(time_str: &str) -> bool {
    let now = chrono::Local::now();
    let parts: Vec<&str> = time_str.trim().split_whitespace().collect();
    
    let (hour, minute) = match parts.len() {
        1 => {
            // "HH:MM" 格式
            let t = parts[0];
            let hm: Vec<&str> = t.split(':').collect();
            if hm.len() != 2 { return false; }
            (hm[0].parse::<u32>().unwrap_or(99), hm[1].parse::<u32>().unwrap_or(99))
        }
        2 => {
            // "EEE HH:MM" 格式 — 先检查星期
            let weekday = parts[0];
            let t = parts[1];
            let hm: Vec<&str> = t.split(':').collect();
            if hm.len() != 2 { return false; }
            let weekday_now = now.format("%a").to_string();
            if !weekday_now.eq_ignore_ascii_case(weekday) {
                return false;
            }
            (hm[0].parse::<u32>().unwrap_or(99), hm[1].parse::<u32>().unwrap_or(99))
        }
        _ => return false,
    };
    
    let current_minutes = now.hour() * 60 + now.minute();
    let target_minutes = hour * 60 + minute;
    current_minutes >= target_minutes
}
```

需要 `chrono` 依赖。检查 `packages/game-hsr/Cargo.toml` 是否已有 chrono:

```bash
grep chrono packages/game-hsr/Cargo.toml
```

如无，加:
```toml
chrono = "0.4"
```

- [ ] **Step 3: 更新 check_rules 中各规则**

```rust
pub fn check_rules(
    data: &WidgetData,
    old: Option<&WidgetData>,
    app: &tauri::AppHandle,
    config: &NotificationConfig,
) {
    // 1. 体力提醒
    if config.stamina_enabled && data.max_stamina > 0 {
        let pct = data.current_stamina as f64 / data.max_stamina as f64;
        if pct >= config.stamina_threshold_urgent {
            notify(app, "体力快满了", &format!("当前 {}/{}", data.current_stamina, data.max_stamina));
        } else if pct >= config.stamina_threshold_mild {
            if let Some(old) = old {
                let old_pct = old.current_stamina as f64 / old.max_stamina as f64;
                if old_pct < config.stamina_threshold_mild {
                    notify(app, &format!("体力超过{}%", (config.stamina_threshold_mild * 100.0) as u32), &format!("当前 {}/{}", data.current_stamina, data.max_stamina));
                }
            }
        }
    }

    // 2. 派遣完成
    if config.expedition_enabled
        && data.total_expedition_num > 0
        && data.accepted_expedition_num == 0
    {
        if let Some(old) = old {
            if old.accepted_expedition_num > 0 {
                notify(app, "派遣全部完成", "所有委托已返回");
            }
        }
    }

    // 3. 备用体力满
    if config.reserve_stamina_enabled && data.is_reserve_stamina_full {
        notify(app, "备用体力已满", "请及时使用");
    }

    // 4. 签到提醒 — 到达指定时间后才触发
    if config.sign_reminder_enabled && !data.has_signed {
        if is_time_reached(&config.sign_reminder_time) {
            if let Some(old) = old {
                if old.has_signed {
                    notify(app, "今日未签到", "星穹铁道今日还未签到");
                }
            }
        }
    }

    // 5. 模拟宇宙未打 — 到达指定时间后才触发
    if config.rogue_reminder_enabled
        && data.max_rogue_score > 0
        && data.current_rogue_score == 0
    {
        if is_time_reached(&config.rogue_reminder_time) {
            if let Some(old) = old {
                if old.current_rogue_score > 0 {
                    notify(app, "模拟宇宙未打", "本周模拟宇宙积分还未获取");
                }
            }
        }
    }
}
```

保持 `notify` 辅助函数不变。

- [ ] **Step 4: 添加 check_digest**

```rust
use std::sync::atomic::{AtomicBool, Ordering};

/// 每日定时摘要推送。在 run_poller 每轮调用，内部记录发送日避免重复。
pub fn check_digest(data: &WidgetData, app: &tauri::AppHandle, config: &NotificationConfig) {
    if !config.digest_enabled || !is_time_reached(&config.digest_time) {
        return;
    }

    use std::sync::OnceLock;
    static LAST_DIGEST_DAY: OnceLock<std::sync::Mutex<i32>> = OnceLock::new();
    let today = chrono::Local::now().ordinal(); // 年内第几天
    let lock = LAST_DIGEST_DAY.get_or_init(|| std::sync::Mutex::new(-1));
    let mut last = lock.lock().unwrap();
    if *last == today {
        return; // 今天已发
    }
    *last = today;
    drop(lock);

    let stamina_line = format!("体力 {}/{}", data.current_stamina, data.max_stamina);
    let expedition_line = if data.total_expedition_num > 0 {
        format!("| 派遣 {}/{}", data.accepted_expedition_num, data.total_expedition_num)
    } else {
        String::new()
    };
    let sign_line = if data.has_signed { "| 已签到" } else { "| 未签到" };

    let body = format!("{} {} {}", stamina_line, expedition_line, sign_line);
    notify(app, "Mihoyo Widget 每日摘要", &body);
}
```

- [ ] **Step 5: 更新 notify/mod.rs 导出 check_digest**

```rust
pub mod rules;
pub use rules::check_digest;  // check_rules 已通过 rules::check_rules 访问
```

- [ ] **Step 6: 编译检查**

```bash
cargo check 2>&1 | head -30
```
期望: 无错误

- [ ] **Step 7: Commit**

```bash
git add packages/game-hsr/src/notify/rules.rs packages/game-hsr/src/notify/mod.rs
git commit -m "feat: notification rules with config-driven conditions and digest"
```

---

### Task 3: 桌面层 — 系统托盘菜单切换、启动窗口隐藏、check_digest 接入

**Files:**
- Modify: `apps/desktop/src/lib.rs`

**Interfaces:**
- Consumes: `NotificationConfig`, `check_digest`
- Produces: 右键菜单切换模式、启动时根据 `notification_mode` 决定是否隐藏窗口

- [ ] **Step 1: 导入 check_digest**

```rust
use game_hsr::notify::rules::check_rules;
// 替换为:
use game_hsr::notify::{check_rules, check_digest};
```

- [ ] **Step 2: 添加 rebuild_tray_menu 函数**

```rust
fn rebuild_tray_menu(app: &tauri::AppHandle, notif_mode: bool) {
    let menu = {
        let m = tauri::menu::MenuBuilder::new(app)
            .item(&tauri::menu::MenuItemBuilder::with_id("show", "显示/隐藏窗口")
                .accelerator("CmdOrCtrl+Shift+H").build(app).unwrap())
            .item(&tauri::menu::MenuItemBuilder::with_id("refresh", "刷新数据")
                .build(app).unwrap())
            .separator();
        let m = if notif_mode {
            m.item(&tauri::menu::MenuItemBuilder::with_id("toggle-notification-mode", "✓ 切换到窗口模式")
                .build(app).unwrap())
        } else {
            m.item(&tauri::menu::MenuItemBuilder::with_id("toggle-notification-mode", "切换到通知模式")
                .build(app).unwrap())
        };
        let m = m.separator();
        let m = m.item(&tauri::menu::MenuItemBuilder::with_id("quit", "退出")
                .accelerator("CmdOrCtrl+Q").build(app).unwrap());
        m.build().unwrap()
    };

    // 更新托盘菜单
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_menu(Some(menu));
    }
}
```

注意: TrayIconBuilder 中 `.id("main")` 需确认。检查既有代码中 `.build(app)` 是否指定 id。当前代码 `let _ = builder.menu(&menu).build(app);` 未显式指定 id → 默认 id 为 "main"。需在 build 时加 `.id("main")`。

现有代码:
```rust
let mut builder = tauri::tray::TrayIconBuilder::new()
    .tooltip("Mihoyo Widget")
    .show_menu_on_left_click(false)
    .on_menu_event(handle_tray_menu);
```
改为:
```rust
let mut builder = tauri::tray::TrayIconBuilder::new()
    .id("main")
    .tooltip("Mihoyo Widget")
    .show_menu_on_left_click(false)
    .on_menu_event(handle_tray_menu);
```

- [ ] **Step 3: 更新 handle_tray_menu 添加 toggle-notification-mode**

```rust
fn handle_tray_menu(app: &tauri::AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id();
    let window = app.get_webview_window("main");
    match id.as_ref() {
        "show" => {
            if let Some(w) = window {
                if w.is_visible().unwrap_or(false) {
                    let _ = w.hide();
                } else {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        }
        "refresh" => {
            if let Some(w) = window {
                let _ = w.emit("manual-refresh", ());
            }
        }
        "toggle-notification-mode" => {
            let state = app.state::<AppState>();
            let mut settings = state.config_data.blocking_lock();
            let new_mode = !settings.notification.notification_mode;
            settings.notification.notification_mode = new_mode;
            let _ = settings.save_to_runtime();
            let config = settings.notification.clone();
            drop(settings);
            drop(state);

            let window = app.get_webview_window("main");
            if new_mode {
                // 进入通知模式 → 隐藏窗口
                if let Some(w) = window {
                    let _ = w.hide();
                }
            } else {
                // 退出通知模式 → 显示窗口 + 刷数据
                if let Some(w) = window {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
                // 发一次最新缓存给前端
                let state = app.state::<AppState>();
                let cache = state.cache_data.blocking_lock();
                let all = cache.get_all_cached();
                drop(cache);
                drop(state);
                let _ = app.emit("data-updated", serde_json::json!({
                    "widget": all.widget,
                    "player": all.player,
                    "ledger": all.ledger,
                    "banners": all.banners,
                    "forgotten_hall": all.forgotten_hall,
                    "pure_fiction": all.pure_fiction,
                    "apocalyptic_shadow": all.apocalyptic_shadow,
                    "periodic_act": all.periodic_act,
                    "challenge_peak": all.challenge_peak,
                    "rogue_archive": all.rogue_archive,
                }));
            }

            rebuild_tray_menu(app, new_mode);
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}
```

- [ ] **Step 4: 修改 force_refresh 中 check_rules 调用传 config**

```rust
// 找到 force_refresh 中调用:
if let Some(ref old_data) = old {
    check_rules(&widget_data, Some(old_data), &app);
}
// 改为:
if let Some(ref old_data) = old {
    let state = app.state::<AppState>();
    let config = state.config_data.lock().await.notification.clone();
    drop(state);
    check_rules(&widget_data, Some(old_data), &app, &config);
}
```

- [ ] **Step 5: 修改 run_poller 中 check_rules 调用传 config**

```rust
// run_poller 中:
if let Some(ref old_data) = old {
    check_rules(&data, Some(old_data), &app);
}
// 改为:
if let Some(ref old_data) = old {
    let state = app.state::<AppState>();
    let config = state.config_data.lock().await.notification.clone();
    drop(state);
    check_rules(&data, Some(old_data), &app, &config);
}
```

- [ ] **Step 6: run_poller 中每轮成功拉取后调用 check_digest**

```rust
// 在 run_poller 的 Ok(data) 分支末尾, emit data-updated 之前:
{
    let state = app.state::<AppState>();
    let config = state.config_data.lock().await.notification.clone();
    drop(state);
    check_digest(&data, &app, &config);
}
```

- [ ] **Step 7: 启动时根据 notification_mode 隐藏窗口**

```rust
// 在 setup() 末尾, spawn(run_poller) 之前:
if settings.notification.notification_mode {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
}
```

同时重建菜单使其匹配 notification_mode。在 setup 末尾 (builder.run 之前) 调用:
```rust
rebuild_tray_menu(app.handle(), settings.notification.notification_mode);
```

注意: rebuild_tray_menu 在 setup 中调用时，app.handle() 可用。

- [ ] **Step 8: 编译检查**

```bash
cargo check 2>&1 | head -40
```
期望: 无错误

- [ ] **Step 9: Commit**

```bash
git add apps/desktop/src/lib.rs
git commit -m "feat: tray menu toggle for notification mode, window hide on startup"
```

---

### Task 4: 前端设置页面 — 通知配置 UI

**Files:**
- Modify: `packages/frontend/src/main.js`
- Modify: `packages/frontend/index.html`

**Interfaces:**
- Consumes: `Settings::notification` from `load_env_config` / `save_config`
- Produces: 设置页底部通知配置区块，toggle + 时间输入

- [ ] **Step 1: index.html — 在 settings-view 底部追加通知区块**

在 `packages/frontend/index.html` 的 `#settings-view` 内，现有字段之后、`button-row` 之前追加:

```html
<div class="divider"></div>
<div class="tab-section-title">
  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>
  通知设置
</div>
<div class="widget-group">
  <div class="setting-row toggle-row">
    <label><span>体力提醒</span>
      <input type="checkbox" class="notif-toggle" data-key="stamina_enabled" />
    </label>
  </div>
  <div class="setting-row threshold-row" data-depends="stamina_enabled">
    <label>阈值 轻度:</label>
    <input type="number" class="notif-input" data-key="stamina_threshold_mild" min="0" max="100" step="5" />
    <label>阈值 紧急:</label>
    <input type="number" class="notif-input" data-key="stamina_threshold_urgent" min="0" max="100" step="5" />
  </div>
  <div class="setting-row toggle-row">
    <label><span>派遣完成</span>
      <input type="checkbox" class="notif-toggle" data-key="expedition_enabled" />
    </label>
  </div>
  <div class="setting-row toggle-row">
    <label><span>备用体力满</span>
      <input type="checkbox" class="notif-toggle" data-key="reserve_stamina_enabled" />
    </label>
  </div>
  <div class="setting-row toggle-row">
    <label><span>签到提醒</span>
      <input type="checkbox" class="notif-toggle" data-key="sign_reminder_enabled" />
    </label>
  </div>
  <div class="setting-row" data-depends="sign_reminder_enabled">
    <label>提醒时间 (HH:MM):</label>
    <input type="text" class="notif-input" data-key="sign_reminder_time" placeholder="20:00" />
  </div>
  <div class="setting-row toggle-row">
    <label><span>模拟宇宙未打</span>
      <input type="checkbox" class="notif-toggle" data-key="rogue_reminder_enabled" />
    </label>
  </div>
  <div class="setting-row" data-depends="rogue_reminder_enabled">
    <label>提醒时间 (EEE HH:MM):</label>
    <input type="text" class="notif-input" data-key="rogue_reminder_time" placeholder="Sun 20:00" />
  </div>
  <div class="setting-row toggle-row">
    <label><span>每日摘要</span>
      <input type="checkbox" class="notif-toggle" data-key="digest_enabled" />
    </label>
  </div>
  <div class="setting-row" data-depends="digest_enabled">
    <label>推送时间 (HH:MM):</label>
    <input type="text" class="notif-input" data-key="digest_time" placeholder="09:00" />
  </div>
</div>
```

- [ ] **Step 2: main.js — loadSettingsForm 追加通知字段读取**

在 `loadSettingsForm()` 中, `$('input-mid').value = ...` 后追加:

```javascript
// 通知设置
const notif = config.notification || {};
document.querySelectorAll('.notif-toggle').forEach(el => {
  const key = el.dataset.key;
  if (key in notif) {
    el.checked = notif[key];
  }
  // 无 old config 时不覆盖默认值（checkbox 默认 unchecked）
});
document.querySelectorAll('.notif-input').forEach(el => {
  const key = el.dataset.key;
  if (key in notif) {
    el.value = key.startsWith('stamina_threshold')
      ? Math.round(notif[key] * 100)
      : notif[key];
  }
});
// 切 toggle 控制依赖字段显隐
updateNotifDependencies();
```

- [ ] **Step 3: main.js — 添加 updateNotifDependencies 和 toggle 联动**

```javascript
function updateNotifDependencies() {
  document.querySelectorAll('[data-depends]').forEach(el => {
    const depKey = el.dataset.depends;
    const toggle = document.querySelector(`.notif-toggle[data-key="${depKey}"]`);
    el.style.display = toggle && toggle.checked ? '' : 'none';
  });
}

document.querySelectorAll('.notif-toggle').forEach(el => {
  el.addEventListener('change', updateNotifDependencies);
});
```

- [ ] **Step 4: main.js — save_config 中包含通知配置**

在 `settings-save` 的 click 处理器中，构建 nc 对象时追加:

```javascript
// 收集通知设置
const notif = {};
document.querySelectorAll('.notif-toggle').forEach(el => {
  notif[el.dataset.key] = el.checked;
});
document.querySelectorAll('.notif-input').forEach(el => {
  const key = el.dataset.key;
  let val = el.value;
  if (key.startsWith('stamina_threshold')) {
    val = parseInt(val) / 100 || 0;
  }
  notif[key] = val;
});
nc.notification = notif;
```

将此代码放在 `nc.poll_interval_secs = ...` 之后、`await invoke('save_config', ...)` 之前。

- [ ] **Step 5: 前端验证**

```bash
cd packages/frontend && npm run build 2>&1 | tail -10
```
期望: 构建成功，无错误

- [ ] **Step 6: Commit**

```bash
git add packages/frontend/src/main.js packages/frontend/index.html
git commit -m "feat: notification settings UI in frontend"
```

---

### 验证步骤

完整的 F5 验证流程:
1. 启动 app → 正常显示窗口
2. 右键托盘 → "切换到通知模式" → 窗口隐藏
3. 右键托盘 → "切换到窗口模式" → 窗口重新出现，数据刷新
4. 切回通知模式 → 重启 app → 窗口自动隐藏（notification_mode 存入 settings）
5. 设置 → 通知设置 → 开关各规则、改阈值 → 保存 → 重新加载设置页 → 值正确
6. 修改签到提醒时间为当前时间前 → 等轮询触发 → 如未签到则发通知
7. cargo check + cargo clippy 全过
