# 系统集成增强 v1

桌面集成增强：全局快捷键（自定义+冲突检测）、开机自启、macOS 菜单栏左键优化。

## 1. 全局快捷键

### 数据结构

`packages/core/src/config/settings.rs` 新增：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ShortcutConfig {
    /// action_id → accelerator string
    /// e.g. "toggle_window" → "Ctrl+Shift+H"
    pub bindings: BTreeMap<String, String>,
    /// 系统冲突列表（运行时填充，不持久化）
    #[serde(skip)]
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SystemConfig {
    pub left_click_toggle: bool,
}
```

`Settings` 结构体新增：
- `shortcuts: ShortcutConfig` — `#[serde(default)]`
- `system: SystemConfig` — `#[serde(default)]`

| 字段 | 默认值 |
|------|--------|
| `shortcuts.bindings["toggle_window"]` | `"Ctrl+Shift+H"` |
| `shortcuts.bindings["refresh"]` | `"Ctrl+Shift+R"` |
| `shortcuts.bindings["quit"]` | `"Ctrl+Shift+Q"` |
| `system.left_click_toggle` | `true` |

### Tauri Commands

```
register_shortcuts(config: ShortcutConfig) → Result<Vec<String>, String>
  注销旧快捷键 → 遍历 bindings 逐个注册 → 返回冲突列表 → 成功保存到 state

is_autostart_enabled() → Result<bool, String>
  托 tauri-plugin-autostart

toggle_autostart(enabled: bool) → Result<(), String>
  托 tauri-plugin-autostart
```

### 注册行为

- `setup()` 注册一次全部快捷键
- 前端改设置后调 `register_shortcuts` 全量重注册
- 每个快捷键回调：
  - `toggle_window` → 显隐 main window
  - `refresh` → emit `"manual-refresh"`
  - `quit` → `app.exit(0)`
- 注册失败 → 收集到 conflicts 返回前端
- tray menu accelerator（`CmdOrCtrl+Shift+H` / `CmdOrCtrl+Q`）作为 fallback 保留

### 冲突检测

两层：
1. 前端预检：bindings 内 action 之间不能重复
2. 系统检测：OS 注册失败 → 标记冲突

### 前端交互

设置页新增 subpage「快捷键」：
- 遍历 `bindings` 每行显示 action 名 + 当前快捷键
- 点击行 → 进入录制模式 → 键盘事件捕获组合键 → 预检 → 调 `register_shortcuts`
- 冲突行红色高亮 + 文字提示
- 录制中动画（闪烁/呼吸）

## 2. 开机自启

`tiuri-plugin-autostart`：
```toml
tauri-plugin-autostart = "2"
```

setup 注册：
```rust
.plugin(tauri_plugin_autostart::init(
    tauri_plugin_autostart::MacosLauncher::LaunchAgent,
    Some(2),
))
```

前端直接调：
- `is_autostart_enabled()` → bool
- `toggle_autostart(enabled: bool)` → ok/err

只在 bundle 模式生效（`cargo tauri build`），dev 模式静默失败。

### 前端

设置→通用页新增一行：
- 标题「开机启动」+ Toggle（默认关）

## 3. macOS 菜单栏左键增强

`lib.rs` `setup()` TracyIconBuilder 加 `on_tray_icon_event`：
```rust
.on_tray_icon_event(|tray, event| {
    if let tauri::tray::TrayIconEvent::Click { .. } = event {
        let app = tray.app_handle();
        let state = app.state::<AppState>();
        let config = state.config_data.blocking_lock();
        if !config.system.left_click_toggle { return; }
        drop(config);
        if let Some(w) = app.get_webview_window("main") {
            if w.is_visible().unwrap_or(false) {
                let _ = w.hide();
            } else {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
    }
})
```

### 前端

设置→通用页新增一行：
- 标题「左键点击切换窗口」+ Toggle（默认开）

## Appendix A — UI 线框图

### 设置根菜单新增行

```
┌─────────────────────────────┐
│ 账号           未配置       >
│ 数据存储       默认位置     >
│ 通知            关闭        >
│ 快捷键          >           >  ← 新增
│ 通用           轮询 90s     >
└─────────────────────────────┘
```

### 快捷键子页面（录制态 / 冲突态）

正常：
```
┌─────────────────────────────┐
│ 显示/隐藏窗口               │
│ ⇧ Ctrl+Shift+H              │
├─────────────────────────────┤
│ 刷新数据                    │
│ ⇧ Ctrl+Shift+R              │
├─────────────────────────────┤
│ 退出应用                    │
│ ⇧ Ctrl+Shift+Q              │
└─────────────────────────────┘
  点击任意行进入录制模式
```

录制中：
```
┌─────────────────────────────┐
│ 显示/隐藏窗口               │
│ [按下快捷键...]  闪烁 ...    │  ← 背景高亮
└─────────────────────────────┘
```

冲突提示：
```
┌─────────────────────────────┐
│ 刷新数据                    │
│ Ctrl+Shift+H  ⚠ 已被占用    │  ← 红色文字
└─────────────────────────────┘
```

### 通用页改动

```
┌─────────────────────────────┐
│ 刷新间隔              90s  │
│ 通知模式              弹窗  │
│ 开机启动                 ○  │  ← 新增
│ 左键点击切换窗口         ●  │  ← 新增
│ 版本号              0.1.0  │
└─────────────────────────────┘
```

## 向后兼容

- `ShortcutConfig` / `SystemConfig` 均有 `#[serde(default)]`
- `Settings` 新字段 `shortcuts`, `system` 均有 `#[serde(default)]`
- 旧 `runtime.json` 缺少这些字段 → 使用 Default 值
- 全局快捷键注册失败不影响 app 启动
- 开机自启 plugin 在 dev 模式下静默失败，不影响运行

## 文件变更清单

| 文件 | 变更 |
|------|------|
| `apps/desktop/Cargo.toml` | 加 `tauri-plugin-global-shortcut`, `tauri-plugin-autostart` |
| `apps/desktop/capabilities/default.json` | 加 autostart permission（如有需要） |
| `packages/core/src/config/settings.rs` | 加 `ShortcutConfig`, `SystemConfig`，嵌入 `Settings`；加 `use std::collections::BTreeMap` |
| `apps/desktop/src/lib.rs` | setup 注册插件 + 快捷键 + 左键回调；加 `register_shortcuts` command；加 `is_autostart_enabled`, `toggle_autostart` command |
| `packages/frontend/src/main.js` | 加快捷键 subpage + 录制交互；通用页加两行 toggle |
| `packages/frontend/src/style.css` | 加快捷键录制 UI 样式；冲突红色高亮 |

## 实现逐项验收清单

### 数据结构
- [x] `ShortcutConfig` — `bindings: BTreeMap<String,String>`, `conflicts: Vec<String>`（`#[serde(skip)]`）
- [x] `SystemConfig` — `left_click_toggle: bool` 默认 true
- [x] `Settings` 新增 `shortcuts: ShortcutConfig` + `system: SystemConfig` 均 `#[serde(default)]`
- [x] `Settings::from_json` / `from_env` 处理新字段（`unwrap_or_default` / 保持默认）
- [x] `Settings::Default` 填充默认快捷键值

### 后端
- [x] `apps/desktop/Cargo.toml` 加 `tauri-plugin-global-shortcut` + `tauri-plugin-autostart`
- [x] `lib.rs` setup 中 `.plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(...))`
- [x] `lib.rs` setup 中 `.plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![])))`
- [x] `register_shortcuts` command — 注销旧 → 遍历 bindings 注册 → 返回 conflicts
- [x] `is_autostart_enabled` command
- [x] `toggle_autostart` command
- [x] invoke_handler 注册新 command
- [x] setup() 中启动时注册快捷键一次
- [x] `on_tray_icon_event` 左键回调（检查 `left_click_toggle`）
- [x] 全局快捷键回调：toggle_window / refresh / quit 各对应操作
- [x] tray menu accelerator 保持不动
- [x] capabilities/default.json 加权限

### 前端 HTML
- [x] 快捷键子页面 `settings-shortcuts` — 遍历 bindings 渲染行
- [x] 通用页 `settings-general` 加「开机启动」Toggle
- [x] 通用页 `settings-general` 加「左键点击切换窗口」Toggle
- [x] `renderSettingsNav` 添加 `settings-shortcuts` 标题映射
- [x] 设置根菜单加「快捷键」入口行

### 前端 JS
- [x] `saveCurrentSettings` carry forward `shortcuts` + `system` 字段
- [x] 快捷键录制模式：click → keydown → display → 预检 → 调 register_shortcuts
- [x] 冲突 UI 提示（红色 + 文字）
- [x] 开机自启 Toggle → 调 `toggle_autostart` / 初始化时调 `is_autostart_enabled`
- [x] 左键切换 Toggle → 存 `system.left_click_toggle`
- [x] 前端预检：bindings 内 action 间不冲突

### CSS
- [x] 快捷键绑定行录制态样式（focus/recording animation）
- [x] 冲突行红色高亮
- [x] 暗色/亮色兼容
