# Settings Redesign — iOS Drill-Down Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor flat settings panel into iOS-style grouped menu with drill-down subpages and auto-save.

**Architecture:** Single-page vanilla JS. Settings area is one `tab-content` div containing a navigation bar, a root menu page (`#settings-root`), and 4 subpage divs. A JS navigation stack controls which page is visible via CSS classes and transforms. Auto-save fires on toggle change, input blur (debounced), and page pop.

**Tech Stack:** Vanilla HTML/CSS/JS, Vite, Tauri v2 API.

**Global Constraints**
- Use existing SVG icon family (Lucide-style, stroke-width=2, 14×14px for inline, 13×13px for buttons) — no emoji as structural icons
- Window is 360×590px non-resizable — subpages must fill available height without overflow
- CSS custom properties for theming (`--bg*`, `--text*`, `--blue`, `--green`, `--gray-light`) already exist — reuse, don't hardcode
- Functions use `$(id)` shorthand (`document.getElementById`)
- All settings data lives in global `config` variable, loaded via `invoke('load_env_config')`
- No Rust backend changes — only frontend restructuring

---

### Task 1: Settings Navigation CSS

**Files:**
- Modify: `packages/frontend/src/style.css` (append after existing settings section at line 952)

**Produces:** All new CSS classes consumed by Task 2–6.

- [ ] **Append new settings nav + menu + subpage styles**

After `.button-row` block at line 952, add:

```css
/* ═══════════════════════════════════════
   SETTINGS DRILL-DOWN NAVIGATION
   ═══════════════════════════════════════ */

/* ── Settings Nav Bar ── */
#settings-nav {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 36px;
  padding: 0 4px;
  flex-shrink: 0;
}
#settings-back {
  background: none;
  border: none;
  color: var(--blue);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 8px;
  transition: background 0.15s;
  display: flex;
  align-items: center;
  gap: 2px;
  min-width: 56px;
}
#settings-back:hover {
  background: var(--bg-elevated);
}
#settings-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
}
#settings-done {
  background: none;
  border: none;
  color: var(--blue);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 8px;
  transition: background 0.15s;
  min-width: 56px;
  text-align: right;
}
#settings-done:hover {
  background: var(--bg-elevated);
}
#settings-back.hidden,
#settings-done.hidden {
  visibility: hidden;
  pointer-events: none;
}

/* ── Settings Pages Container ── */
#settings-pages {
  position: relative;
  flex: 1;
  overflow: hidden;
  min-height: 0;
}

/* ── Individual Page ── */
.settings-page {
  position: absolute;
  inset: 0;
  overflow-y: auto;
  padding: 4px 8px;
  opacity: 0;
  transform: translateX(30px);
  transition: transform 0.3s ease, opacity 0.25s ease;
  will-change: transform, opacity;
  pointer-events: none;
  -webkit-overflow-scrolling: touch;
}
.settings-page.active {
  opacity: 1;
  transform: translateX(0);
  pointer-events: auto;
}
.settings-page.exit-left {
  opacity: 0;
  transform: translateX(-30px);
}
.settings-page.enter-right {
  opacity: 0;
  transform: translateX(30px);
}

/* ── Settings Group (iOS-style grouped list) ── */
.settings-group {
  background: var(--bg-elevated);
  border-radius: 12px;
  margin-bottom: 12px;
  overflow: hidden;
}
.settings-group:last-child {
  margin-bottom: 0;
}
.settings-group-header {
  font-size: 11px;
  color: var(--text-dim);
  font-weight: 500;
  padding: 8px 12px 4px;
  letter-spacing: -0.1px;
  text-transform: uppercase;
}

/* ── Menu Row (root page) ── */
.settings-menu-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  cursor: pointer;
  transition: background 0.12s;
  min-height: 40px;
  border-bottom: 0.5px solid var(--gray-light);
}
.settings-menu-row:last-child {
  border-bottom: none;
}
.settings-menu-row:hover {
  background: var(--bg-secondary);
}
.settings-menu-row:active {
  background: var(--gray-light);
}
.settings-menu-icon {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.settings-menu-icon svg {
  width: 18px;
  height: 18px;
  stroke: var(--blue);
  stroke-width: 2;
}
.settings-menu-label {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}
.settings-menu-title {
  font-size: 13px;
  color: var(--text);
  font-weight: 500;
}
.settings-menu-summary {
  font-size: 10px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.settings-menu-chevron {
  font-size: 14px;
  color: var(--text-muted);
  opacity: 0.5;
  flex-shrink: 0;
}

/* ── Subpage Content ── */
.setting-field-row {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  min-height: 36px;
  border-bottom: 0.5px solid var(--gray-light);
}
.setting-field-row:last-child {
  border-bottom: none;
}
.setting-field-label {
  font-size: 13px;
  color: var(--text);
  font-weight: 500;
  flex-shrink: 0;
  min-width: 64px;
}
.setting-field-input {
  flex: 1;
  background: transparent;
  border: none;
  padding: 4px 8px;
  color: var(--text);
  font-size: 13px;
  outline: none;
  text-align: right;
  font-family: 'SF Mono', 'Cascadia Code', 'JetBrains Mono', monospace;
}
.setting-field-input::placeholder {
  color: var(--text-muted);
  font-size: 11px;
}
.setting-field-input:focus {
  background: var(--bg-secondary);
  border-radius: 8px;
}

/* ── Password field with reveal ── */
.setting-password-wrap {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 4px;
}
.setting-password-wrap input {
  flex: 1;
  text-align: right;
}
.setting-password-toggle {
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  border-radius: 6px;
  transition: background 0.12s;
}
.setting-password-toggle:hover {
  background: var(--bg-secondary);
}
.setting-password-toggle svg {
  width: 14px;
  height: 14px;
}

/* ── Action row (e.g. login button) ── */
.setting-action-row {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 12px;
  cursor: pointer;
  transition: background 0.12s;
  color: var(--blue);
  font-size: 13px;
  font-weight: 500;
  gap: 6px;
}
.setting-action-row:hover {
  background: var(--bg-secondary);
}
.setting-action-row:active {
  background: var(--gray-light);
}
.setting-action-row svg {
  width: 16px;
  height: 16px;
  stroke: var(--blue);
  stroke-width: 2;
}

/* ── Subpage footer hint ── */
.setting-page-hint {
  font-size: 10px;
  color: var(--text-muted);
  text-align: center;
  padding: 12px 8px;
}

/* ── Storage subpage specific ── */
.setting-storage-path {
  font-size: 11px;
  color: var(--text-muted);
  font-family: 'SF Mono', 'Cascadia Code', 'JetBrains Mono', monospace;
  padding: 4px 12px 8px;
  word-break: break-all;
  line-height: 1.4;
}
.setting-storage-btn {
  display: block;
  width: calc(100% - 24px);
  margin: 8px 12px 12px;
  padding: 8px;
  background: var(--bg-secondary);
  border: 0.5px solid var(--blue);
  border-radius: 10px;
  color: var(--blue);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}
.setting-storage-btn:hover {
  background: var(--blue);
  color: white;
}

/* ── Notification subpage (keeps existing .notif-group etc.) ── */
#settings-notifications .settings-scroll {
  padding: 0 4px;
}
#settings-notifications .notif-group {
  background: var(--bg-elevated);
  border-radius: 12px;
  overflow: hidden;
}

/* ── General subpage ── */
.setting-info-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-bottom: 0.5px solid var(--gray-light);
}
.setting-info-row:last-child {
  border-bottom: none;
}
.setting-info-label {
  font-size: 13px;
  color: var(--text);
  font-weight: 500;
}
.setting-info-value {
  font-size: 12px;
  color: var(--text-muted);
  font-family: 'SF Mono', 'Cascadia Code', 'JetBrains Mono', monospace;
}
```

- [ ] **Commit**

```bash
git add packages/frontend/src/style.css
git commit -m "style: add settings drill-down navigation CSS"
```

---

### Task 2: Settings HTML — Root Menu + Subpage Containers

**Files:**
- Modify: `packages/frontend/index.html` (lines 205–321)

- [ ] **Replace entire settings view HTML**

Old: lines 205–321 (`<div id="settings-view" class="tab-content">` through its closing `</div>`).

New:

```html
      <!-- ════════ Settings View ════════ -->
      <div id="settings-view" class="tab-content">
        <!-- Navigation bar -->
        <div id="settings-nav">
          <button id="settings-back" class="hidden">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 12H5"/><path d="M12 19l-7-7 7-7"/></svg>
            <span id="settings-back-text">设置</span>
          </button>
          <span id="settings-title">设置</span>
          <button id="settings-done">完成</button>
        </div>

        <!-- Pages container -->
        <div id="settings-pages">

          <!-- ── Root Menu ── -->
          <div id="settings-root" class="settings-page active">
            <div class="settings-group">
              <div class="settings-menu-row" data-page="settings-account">
                <div class="settings-menu-icon">
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                </div>
                <div class="settings-menu-label">
                  <span class="settings-menu-title">账号</span>
                  <span class="settings-menu-summary" id="summary-account">未配置</span>
                </div>
                <div class="settings-menu-chevron">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M9 18l6-6-6-6"/></svg>
                </div>
              </div>
            </div>

            <div class="settings-group">
              <div class="settings-menu-row" data-page="settings-storage">
                <div class="settings-menu-icon">
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z"/></svg>
                </div>
                <div class="settings-menu-label">
                  <span class="settings-menu-title">数据存储</span>
                  <span class="settings-menu-summary" id="summary-storage">默认位置</span>
                </div>
                <div class="settings-menu-chevron">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M9 18l6-6-6-6"/></svg>
                </div>
              </div>
            </div>

            <div class="settings-group">
              <div class="settings-menu-row" data-page="settings-notifications">
                <div class="settings-menu-icon">
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>
                </div>
                <div class="settings-menu-label">
                  <span class="settings-menu-title">通知</span>
                  <span class="settings-menu-summary" id="summary-notifications">关闭</span>
                </div>
                <div class="settings-menu-chevron">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M9 18l6-6-6-6"/></svg>
                </div>
              </div>
            </div>

            <div class="settings-group">
              <div class="settings-menu-row" data-page="settings-general">
                <div class="settings-menu-icon">
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
                </div>
                <div class="settings-menu-label">
                  <span class="settings-menu-title">通用</span>
                  <span class="settings-menu-summary" id="summary-general">轮询 90s</span>
                </div>
                <div class="settings-menu-chevron">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M9 18l6-6-6-6"/></svg>
                </div>
              </div>
            </div>
          </div>

          <!-- ── Account Subpage ── -->
          <div id="settings-account" class="settings-page">
            <div class="settings-group">
              <div class="setting-field-row">
                <label class="setting-field-label">Cookie</label>
                <div class="setting-password-wrap">
                  <input type="password" class="setting-field-input" id="input-cookie" placeholder="完整 Cookie" autocomplete="off" />
                  <button class="setting-password-toggle" data-for="input-cookie" title="显示/隐藏">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
                  </button>
                </div>
              </div>
              <div class="setting-field-row">
                <label class="setting-field-label">SToken</label>
                <div class="setting-password-wrap">
                  <input type="password" class="setting-field-input" id="input-stoken" placeholder="SToken" autocomplete="off" />
                  <button class="setting-password-toggle" data-for="input-stoken" title="显示/隐藏">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
                  </button>
                </div>
              </div>
              <div class="setting-field-row">
                <label class="setting-field-label">UID</label>
                <input type="text" class="setting-field-input" id="input-uid" placeholder="游戏 UID" />
              </div>
              <div class="setting-field-row">
                <label class="setting-field-label">STUID</label>
                <input type="text" class="setting-field-input" id="input-stuid" placeholder="账户 STUID" />
              </div>
              <div class="setting-field-row">
                <label class="setting-field-label">MID</label>
                <input type="text" class="setting-field-input" id="input-mid" placeholder="账户 MID" />
              </div>
            </div>
            <div class="settings-group">
              <div class="setting-action-row" id="settings-webview-login">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>
                从米游社登录
              </div>
            </div>
            <div class="setting-page-hint">修改后自动保存</div>
          </div>

          <!-- ── Storage Subpage ── -->
          <div id="settings-storage" class="settings-page">
            <div class="settings-group">
              <div class="setting-field-row">
                <label class="setting-field-label">存储位置</label>
                <span style="font-size:12px;color:var(--text-muted);flex:1;text-align:right">默认位置</span>
              </div>
              <div class="setting-storage-path" id="settings-storage-path">~/.config/mihoyo-widget</div>
              <button class="setting-storage-btn" id="settings-pick-dir">选择其他目录</button>
            </div>
            <div class="setting-page-hint">游戏数据、缓存和配置文件存储于此</div>
          </div>

          <!-- ── Notifications Subpage ── -->
          <div id="settings-notifications" class="settings-page">
            <div class="settings-scroll">
              <div class="notif-group">
                <div class="setting-row">
                  <label><span>体力提醒</span></label>
                  <input type="checkbox" class="notif-toggle" data-key="stamina_enabled" />
                </div>
                <div class="notif-sub" data-depends="stamina_enabled">
                  <div class="notif-sub-row">
                    <div class="threshold-group">
                      <label>轻度</label>
                      <input type="number" class="notif-input" data-key="stamina_threshold_mild" min="0" max="100" step="5" />
                    </div>
                    <div class="threshold-group">
                      <label>紧急</label>
                      <input type="number" class="notif-input" data-key="stamina_threshold_urgent" min="0" max="100" step="5" />
                    </div>
                  </div>
                </div>
                <div class="setting-row">
                  <label><span>派遣完成</span></label>
                  <input type="checkbox" class="notif-toggle" data-key="expedition_enabled" />
                </div>
                <div class="setting-row">
                  <label><span>备用体力满</span></label>
                  <input type="checkbox" class="notif-toggle" data-key="reserve_stamina_enabled" />
                </div>
                <div class="setting-row">
                  <label><span>签到提醒</span></label>
                  <input type="checkbox" class="notif-toggle" data-key="sign_reminder_enabled" />
                </div>
                <div class="notif-sub" data-depends="sign_reminder_enabled">
                  <div class="notif-sub-row">
                    <label>时间</label>
                    <input type="time" class="notif-input" data-key="sign_reminder_time" value="20:00" />
                  </div>
                </div>
                <div class="setting-row">
                  <label><span>模拟宇宙未打</span></label>
                  <input type="checkbox" class="notif-toggle" data-key="rogue_reminder_enabled" />
                </div>
                <div class="notif-sub" data-depends="rogue_reminder_enabled">
                  <div class="notif-sub-row">
                    <label>星期</label>
                    <select class="notif-input" data-key="rogue_reminder_day">
                      <option value="Sun">周日</option>
                      <option value="Mon">周一</option>
                      <option value="Tue">周二</option>
                      <option value="Wed">周三</option>
                      <option value="Thu">周四</option>
                      <option value="Fri">周五</option>
                      <option value="Sat">周六</option>
                    </select>
                    <label>时间</label>
                    <input type="time" class="notif-input" data-key="rogue_reminder_time" value="20:00" />
                  </div>
                </div>
                <div class="setting-row">
                  <label><span>每日摘要</span></label>
                  <input type="checkbox" class="notif-toggle" data-key="digest_enabled" />
                </div>
                <div class="notif-sub" data-depends="digest_enabled">
                  <div class="notif-sub-row">
                    <label>时间</label>
                    <input type="time" class="notif-input" data-key="digest_time" value="09:00" />
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- ── General Subpage ── -->
          <div id="settings-general" class="settings-page">
            <div class="settings-group">
              <div class="setting-info-row">
                <span class="setting-info-label">刷新间隔</span>
                <span class="setting-info-value" id="settings-poll-interval">90s</span>
              </div>
              <div class="setting-info-row">
                <span class="setting-info-label">通知模式</span>
                <span class="setting-info-value" id="settings-notif-mode">弹窗</span>
              </div>
              <div class="setting-info-row">
                <span class="setting-info-label">版本号</span>
                <span class="setting-info-value">0.1.0</span>
              </div>
            </div>
            <div class="settings-group">
              <div class="setting-action-row" id="settings-show-welcome">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>
                重新引导
              </div>
            </div>
          </div>

        </div><!-- /settings-pages -->
      </div>
```

- [ ] **Commit**

```bash
git add packages/frontend/index.html
git commit -m "feat: replace flat settings with drill-down menu structure"
```

---

### Task 3: JS Navigation Stack + Push/Pop/Close

**Files:**
- Modify: `packages/frontend/src/main.js`

**Interfaces:**
- Consumes: `settings-back`, `settings-done`, `.settings-menu-row[data-page]` from HTML (Task 2)
- Consumes: CSS classes `.active`, `.hidden`, `.exit-left`, `.enter-right` from CSS (Task 1)
- Produces: `pushSubpage(pageId)`, `popSubpage()`, `closeSettings()`, `updateSettingsSummary()` — called by Task 4–6

- [ ] **Add navigation stack state and functions**

After line 19 (`let isSettingsOpen = false;`), add:

```js
let settingsStack = ['settings-root'];
```

- [ ] **Add pushSubpage, popSubpage, closeSettings, renderSettingsNav**

Insert a new section after the `renderTab()` function (after line 77), before the `// ════════ TAB 1` comment at line 80:

```js
// ── Settings drill-down navigation ──
function pushSubpage(pageId) {
  const current = settingsStack[settingsStack.length - 1];
  const currentEl = $(current);
  const nextEl = $(pageId);
  if (!nextEl || current === pageId) return;
  // Save current page before leaving
  saveCurrentSettings();
  // Animate out current, in next
  currentEl.classList.remove('active');
  currentEl.classList.add('exit-left');
  nextEl.classList.add('enter-right');
  // Force reflow for transition
  nextEl.offsetHeight;
  nextEl.classList.remove('enter-right');
  nextEl.classList.add('active');
  settingsStack.push(pageId);
  renderSettingsNav();
  // Cleanup after animation
  setTimeout(() => {
    currentEl.classList.remove('exit-left');
  }, 300);
}

function popSubpage() {
  if (settingsStack.length <= 1) return;
  saveCurrentSettings();
  const leaving = settingsStack.pop();
  const target = settingsStack[settingsStack.length - 1];
  const leavingEl = $(leaving);
  const targetEl = $(target);
  // Animate out current (right), in target (from left)
  leavingEl.classList.remove('active');
  leavingEl.classList.add('exit-left');
  targetEl.style.transform = 'translateX(-30px)';
  targetEl.style.opacity = '0';
  targetEl.classList.add('active');
  targetEl.offsetHeight;
  targetEl.style.transform = '';
  targetEl.style.opacity = '';
  renderSettingsNav();
  setTimeout(() => {
    leavingEl.classList.remove('exit-left');
  }, 300);
}

function closeSettings() {
  settingsStack = ['settings-root'];
  // Reset all pages to clean state
  document.querySelectorAll('.settings-page').forEach((el) => {
    el.classList.remove('active', 'exit-left', 'enter-right');
    el.style.transform = '';
    el.style.opacity = '';
  });
  $('settings-root')?.classList.add('active');
  isSettingsOpen = false;
  currentTab = previousTab;
  updateTabBar();
  renderTab();
}

function renderSettingsNav() {
  const isRoot = settingsStack.length <= 1;
  const backBtn = $('settings-back');
  const doneBtn = $('settings-done');
  const titleEl = $('settings-title');
  const currentPage = settingsStack[settingsStack.length - 1];
  // Title mapping
  const titles = {
    'settings-root': '设置',
    'settings-account': '账号',
    'settings-storage': '数据存储',
    'settings-notifications': '通知',
    'settings-general': '通用',
  };
  titleEl.textContent = titles[currentPage] || '设置';
  if (isRoot) {
    backBtn.classList.add('hidden');
    doneBtn.classList.remove('hidden');
    doneBtn.textContent = '完成';
  } else {
    backBtn.classList.remove('hidden');
    const parentTitle = titles[settingsStack[settingsStack.length - 2]] || '设置';
    document.getElementById('settings-back-text').textContent = parentTitle;
    doneBtn.classList.add('hidden');
  }
}
```

- [ ] **Bind nav event listeners**

Find the settings-btn click handler (currently line 841: `$('settings-btn').addEventListener('click', ...)`). **After** `renderSettingsNav()` function, add event bindings — insert after the new nav code, before the existing `$('settings-btn')` handler:

```js
// Settings nav event bindings
document.getElementById('settings-pages')?.addEventListener('click', (e) => {
  const menuRow = e.target.closest('.settings-menu-row[data-page]');
  if (menuRow) {
    pushSubpage(menuRow.dataset.page);
    return;
  }
});

$('settings-back')?.addEventListener('click', popSubpage);

// Close on Escape
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape' && isSettingsOpen) {
    if (settingsStack.length > 1) {
      popSubpage();
    } else {
      closeSettings();
    }
  }
});
```

Also modify the existing `$('settings-btn')` click handler to reset the nav stack when opening settings. Find the current handler at line 841:

```js
$('settings-btn').addEventListener('click', () => switchTab('settings'));
```

Change to:

```js
$('settings-btn').addEventListener('click', () => {
  if (!isSettingsOpen) {
    settingsStack = ['settings-root'];
    document.querySelectorAll('.settings-page').forEach((el) => {
      el.classList.remove('active', 'exit-left', 'enter-right');
      el.style.transform = '';
      el.style.opacity = '';
    });
    $('settings-root')?.classList.add('active');
    renderSettingsNav();
  }
  switchTab('settings');
});
```

- [ ] **Commit**

```bash
git add packages/frontend/src/main.js
git commit -m "feat: add settings navigation stack with push/pop/close"
```

---

### Task 4: Auto-Save + `saveCurrentSettings()` + Replace Old Save/Close

**Files:**
- Modify: `packages/frontend/src/main.js`

**Interfaces:**
- Consumes: `pushSubpage()`, `popSubpage()` from Task 3
- Produces: `saveCurrentSettings()` — called by Task 3's push/pop/close, called by Task 6 for summaries

- [ ] **Write `saveCurrentSettings()` function**

This replaces the old inline save handler. Insert after `renderSettingsNav()` and the event bindings from Task 3. Then **remove** the old `$('settings-save')` and `$('settings-close')` handlers (lines 847–916 and the data dir picker at 918–928).

New function:

```js
let _saveTimeout = null;
function saveCurrentSettings() {
  if (_saveTimeout) {
    clearTimeout(_saveTimeout);
    _saveTimeout = null;
  }
  const cookie = $('input-cookie')?.value.trim() || config?.cookie || '';
  const uid = $('input-uid')?.value.trim() || config?.uid || '';
  if (!cookie) return; // Not yet configured, skip save
  const nc = {
    cookie,
    stoken: $('input-stoken')?.value || config?.stoken || '',
    uid,
    stuid: $('input-stuid')?.value || config?.stuid || '',
    mid: $('input-mid')?.value || config?.mid || '',
    device_id: config?.device_id || '',
    device_fp: config?.device_fp || '',
    seed_id: config?.seed_id || '',
    seed_time: config?.seed_time || '',
    region: config?.region || 'prod_gf_cn',
    poll_interval_secs: config?.poll_interval_secs || 90,
    data_dir: config?.data_dir || '',
  };
  // Collect notification settings
  const notif = {};
  document.querySelectorAll('.notif-toggle').forEach((el) => {
    notif[el.dataset.key] = el.checked;
  });
  document.querySelectorAll('.notif-input').forEach((el) => {
    const key = el.dataset.key;
    if (key === 'rogue_reminder_day') return;
    let val = el.value;
    if (key.startsWith('stamina_threshold')) {
      val = parseInt(val) / 100 || 0;
    }
    notif[key] = val;
  });
  const rogueDay = document.querySelector('.notif-input[data-key="rogue_reminder_day"]');
  const rogueTime = document.querySelector('.notif-input[data-key="rogue_reminder_time"]');
  if (rogueDay && rogueTime) {
    notif.rogue_reminder_time = `${rogueDay.value} ${rogueTime.value}`;
  }
  if (config?.notification?.notification_mode != null) {
    notif.notification_mode = config.notification.notification_mode;
  }
  nc.notification = notif;
  invoke('save_config', { newConfig: nc }).catch((e) => {
    console.error('保存失败:', e);
  });
  config = nc;
}

// Debounced auto-save on input blur
function setupAutoSave() {
  document.querySelectorAll('#settings-view input, #settings-view select').forEach((el) => {
    el.addEventListener('change', () => {
      if (_saveTimeout) clearTimeout(_saveTimeout);
      _saveTimeout = setTimeout(saveCurrentSettings, 200);
    });
    el.addEventListener('blur', () => {
      if (_saveTimeout) clearTimeout(_saveTimeout);
      _saveTimeout = setTimeout(saveCurrentSettings, 300);
    });
  });
}
```

- [ ] **Remove old save/close handlers and data dir picker handler**

Delete these blocks from `main.js`:

1. The `$('settings-save').addEventListener('click', async () => {...})` block (currently lines 847–909)
2. The `$('settings-close').addEventListener('click', () => {...})` block (lines 911–916)
3. The `$('settings-pick-dir')?.addEventListener('click', async () => {...})` block (lines 918–928)

- [ ] **Wire auto-save into settings load**

At the end of `loadSettingsForm()` (after the `updateNotifDependencies()` call at line 641), add:

```js
  setupAutoSave();
  updateSettingsSummary();
```

- [ ] **Remove old `$('settings-btn')` close handler (the `document.addEventListener('contextmenu', ...)` block)**

Find and remove lines 842–845:

```js
document.addEventListener('contextmenu', (e) => {
  e.preventDefault();
  switchTab('settings');
});
```

And replace with the new close-on-Escape already added in Task 3.

- [ ] **Commit**

```bash
git add packages/frontend/src/main.js
git commit -m "feat: replace manual save/close with auto-save on change/blur"
```

---

### Task 5: Subpage Event Wiring (Login, Picker, Welcome)

**Files:**
- Modify: `packages/frontend/src/main.js`

- [ ] **Wire storage picker, webview login, and welcome re-launch**

Insert after the `setupAutoSave()` function:

```js
// ── Settings subpage actions ──

// Storage picker
$('settings-pick-dir')?.addEventListener('click', async () => {
  try {
    const dir = await invoke('pick_data_dir');
    if (dir) {
      config.data_dir = dir;
      await invoke('set_data_dir', { dataDir: dir });
      updateSettingsSummary();
      saveCurrentSettings();
    }
  } catch (e) {
    console.warn('Dir pick failed:', e);
  }
});

// WebView login
$('settings-webview-login')?.addEventListener('click', async () => {
  try {
    await invoke('open_login_webview');
  } catch (e) {
    console.warn('Login webview failed:', e);
  }
});

// Re-launch welcome
$('settings-show-welcome')?.addEventListener('click', () => {
  closeSettings();
  showWelcome();
});
```

- [ ] **Wire password reveal toggles**

```js
// Password reveal toggles in account subpage
document.querySelectorAll('.setting-password-toggle').forEach((btn) => {
  btn.addEventListener('click', () => {
    const inputId = btn.dataset.for;
    const input = $(inputId);
    if (!input) return;
    const isPassword = input.type === 'password';
    input.type = isPassword ? 'text' : 'password';
    btn.innerHTML = isPassword
      ? '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/><line x1="1" y1="1" x2="23" y2="23"/></svg>'
      : '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>';
  });
});
```

- [ ] **Commit**

```bash
git add packages/frontend/src/main.js
git commit -m "feat: wire storage picker, webview login, welcome, password reveal"
```

---

### Task 6: Summary Text + Init State + Cleanup

**Files:**
- Modify: `packages/frontend/src/main.js`

- [ ] **Write `updateSettingsSummary()` and call on load**

```js
function updateSettingsSummary() {
  // Account summary
  const hasCookie = config && config.cookie;
  document.getElementById('summary-account').textContent = hasCookie ? '已配置' : '未配置';

  // Storage summary
  const dataDir = config?.data_dir || '';
  document.getElementById('summary-storage').textContent = dataDir
    ? (dataDir.length > 24 ? '…' + dataDir.slice(-24) : dataDir)
    : '默认位置';
  const pathEl = document.getElementById('settings-storage-path');
  if (pathEl) pathEl.textContent = dataDir || '默认位置 (~/.config/mihoyo-widget)';

  // Notifications summary
  const notif = config?.notification || {};
  const enabledCount = ['stamina_enabled', 'expedition_enabled', 'reserve_stamina_enabled',
    'sign_reminder_enabled', 'rogue_reminder_enabled', 'digest_enabled']
    .filter(k => notif[k]).length;
  document.getElementById('summary-notifications').textContent = enabledCount > 0 ? `${enabledCount} 项开启` : '关闭';

  // General summary
  const interval = config?.poll_interval_secs || 90;
  document.getElementById('summary-general').textContent = `轮询 ${interval}s`;
  document.getElementById('settings-poll-interval').textContent = `${interval}s`;
  document.getElementById('settings-notif-mode').textContent = notif.notification_mode ? '静默' : '弹窗';
}
```

- [ ] **Call `updateSettingsSummary()` in `loadSettingsForm()`**

At the end of `loadSettingsForm()` (after `updateNotifDependencies()` and `setupAutoSave`), ensure `updateSettingsSummary()` is called. Already added in Task 4.

Also call it when login cookies are captured. Find the login-cookies-captured listener — look for `listen('login-cookies-captured', ...)` near the bottom of main.js and after it reloads config, add:

```js
await loadSettingsForm();
updateSettingsSummary();
```

- [ ] **Cleanup: remove old `.setting-row` / `.button-row` CSS if no longer used**

Check if old `.setting-row` (non-notif version, lines 742–771) is still used by any remaining HTML. The new settings HTML uses `.setting-field-row` and `.notif-group .setting-row` (the notification toggles). The top-level `.setting-row` (in `#settings-view`) that was used for Cookie/SToken/UID inputs is no longer referenced — it can be left in CSS (unused but harmless) or removed. Leave it for now to minimize diff risk.

- [ ] **Commit**

```bash
git add packages/frontend/src/main.js
git commit -m "feat: add settings summary text and state sync"
```

---

### Task 7: Verify Build and Functionality

**Files:** (none)

- [ ] **Build frontend to verify no syntax errors**

```bash
cd /media/jayhaul/dev/Code/mihoyo-widget/packages/frontend && npm run build
```

Expected: Build succeeds, output in `dist/`.

- [ ] **Quick code review pass**

Check:
- All `$(id)` calls reference IDs that exist in the new HTML
- No references to deleted `$('settings-save')` or `$('settings-close')`
- `saveCurrentSettings()` handles the case where config is null (first load)
- `settingsStack` is reset when opening/closing settings
- All event listeners are bound after DOM is ready (or use the existing DOMContentLoaded pattern)

- [ ] **Commit any fixes**

```bash
git add -A
git commit -m "fix: address review issues from settings drill-down"
```
