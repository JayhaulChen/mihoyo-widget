---
title: 全局快捷键在 Wayland 下不可用
date: 2026-06-27
related: ADR-0004
---

`tauri-plugin-global-shortcut` 底层依赖 `global_hotkey` crate，该 crate 的 Linux 实现只支持 **X11**。Wayland 下注册不报错但事件永远不会触发，因为 Wayland compositor 不会向非焦点客户端派发按键事件。

处理方式：
- 前端检测 `XDG_SESSION_TYPE`，Wayland 下在快捷键设置页显示橙色提示横幅
- `register_shortcuts` 继续正常调用（无副作用，X11 session 下可用）
- tray menu accelerator 不受影响（走系统原生菜单机制，D-Bus/AppIndicator）
- 不尝试在 Wayland 下 fallback 或模拟全局快捷键

相关代码: `apps/desktop/src/lib.rs` — `get_session_type` command、`register_shortcut_binding`
`packages/frontend/src/main.js` — `renderShortcuts()` 中的 Wayland 检测
