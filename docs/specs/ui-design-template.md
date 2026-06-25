# Mihoyo Widget — UI Design 规范

**日期**: 2026-06-25
**状态**: Active

## 用途

UI Design 阶段产出。当 Grill 评估为"布局大改"或"新增多页面"时，写此文档。

## 模板

```
# UI Design: <主题>

**日期**: YYYY-MM-DD
**关联 Spec**: docs/specs/YYYY-MM-DD-<topic>.md

## 布局策略

[iOS Settings 行列表 / 卡片 grid / 导航堆栈 / 其他？]

## 组件层级

[根视图 → Tab → Section → Card → Row 的层级关系]

## 交互行为

[点击 → 动画效果 → 数据加载 → 状态变更]

## 状态管理

[哪些数据在 JS 变量、DOM、localStorage、后端 cache 中]

## 暗色模式

[所有自定义颜色使用 --blue / --text 等 token，无硬编码 hex]

## 风格约束

- 所有图标用 SVG inline（不用 emoji、不用字体图标）
- touch target ≥ 44px
- 行间距用 0.5px 分隔线实现 iOS 分割效果
- 禁用态 / loading 态必需
```

## 已有 UI 模式（直接复用）

| 模式 | 适用场景 | 参考实现 |
|------|----------|---------|
| `more-section` + `more-section-header` + `more-group-card` | 分组列表页 | 挑战/活动 tab |
| `more-section` + `more-section-header` + `ov-stat` 行列表 | 状态列表 | 仪表盘底部 |
| `more-section` + `more-section-header` + `ov-stamina-card` | 非对称布局（环+信息） | 仪表盘顶部 |
| `widget-group` + `setting-row` + iOS Switch | 设置页表单 | 通知设置 |
| `notif-sub` + `notif-sub-row` | 条件出现的高级选项 | 通知阈值/时间 |
| 导航堆栈 + push/pop（settings-drilldown） | 多级设置菜单 | settings-drilldown |
