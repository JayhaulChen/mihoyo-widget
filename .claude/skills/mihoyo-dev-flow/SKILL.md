---
name: mihoyo-dev-flow
description: Mihoyo Widget 全流程开发工作流。当用户说要加功能、改东西、修bug、发版本时触发。包含 Grill → Spec → Implement → Review → Ship → Retro 六个阶段。自动融合 codebase-intelligence、CONTEXT.md、ADR 和 git-guardrails。单人Rust Tauri 桌面端项目。
model: auto
disable-model-invocation: false
---

# Mihoyo Dev Flow

一套 6 阶段开发工作流，覆盖从想法到发布的完整过程。对话式触发 — 你说出需求，我带你走完。

**前置检查**（每次进入工作流时执行）：
1. 读 CONTEXT.md 和已有 ADR 了解当前架构上下文
2. 调 `codebase-memory get_architecture` 看当前项目全景
3. 读 memory 目录（`/home/jayhaul/.claude/projects/-media-jayhaul-dev-Code-mihoyo-widget/memory/`）了解已有教训和偏好

---

## Phase 1: Grill（需求打磨）

**目标：** 把模糊想法敲实，明确范围和成功标准。

**做法：**
1. 听你描述需求
2. 追问边界（做什么 / 不做什么）
3. 用 CONTEXT.md 统一术语
4. 用 `codebase-memory` 定位影响模块
5. **UI scope 严格评估：** 逐页面检查——哪些页面/组件/数据流受影响？布局新增 vs 改 vs 删？新 UI 元素数量？
   - **UI 三选一**：纯微调（padding/margin/颜色）→ 无独立 UI spec
   - **中等**（新增 1-2 个页面/面板）→ 写划线框图
   - **大改**（布局重构、多页面、导航变更）→ 写完整 UI spec + 先确认再编码
6. 评估 token 获取方式的可行性和后端影响（WebView 登录 / QR / 手动输入）
7. 输出到 `docs/specs/YYYY-MM-DD-<topic>.md`

**完成条件：**
- [ ] 功能边界清晰（做什么/不做什么）
- [ ] 涉及模块已定位
- [ ] 领域术语一致
- [ ] **UI scope 三选一结果已确认**
- [ ] 可以进入设计阶段

---

## Phase 2: Spec（接口设计）

**目标：** 敲定数据结构和接口签名，但不动实现代码。

**做法：**
1. 读相关 ADR 不踩坑
2. 定义 Rust struct / type / API 签名，给你确认
3. 跨 crate 改动时用 `codebase-memory trace_path` 追踪调用链
4. 评估通知规则影响（notify/rules.rs、settings 等）+ 数据目录（data_dir）、路径抽象等基础设施

**UI 处理原则（替代旧的独立 UI Design 阶段）：**
- Grill 阶段已确认 UI scope，无需再开独立阶段
- 中等以上 UI 改动：在 spec 文件末尾加 Appendix A — UI 线框图，前端实现时以此为锚
- 微调：直接在前端实现，不加 spec

**关键产出——兼容性清单：**
实现开始前，写一个 markdown checklist 嵌入 spec 文件尾：
```
## 实现逐项验收清单（编码时逐条对照）
- [ ] 数据结构：Settings 新增了哪些字段、是否 serde(default)
- [ ] 路径：runtime_config_path 是否支持 data_dir
- [ ] 命令：所有新 Tauri command 是否有前端对应调用
- [ ] 事件：所有新 emit/listen 是否配对
- [ ] 前端页面（HTML 结构改变时）：DOM id 列表
- [ ] UI 元素（新 spec 附加）：每个新元素 ID
```

**完成条件：**
- [ ] 接口签名和数据结构已确认
- [ ] 改动范围明确  
- [ ] UI 线框图已确认（如适用）
- [ ] 兼容性清单已写入 spec
- [ ] 可进入实现

---

## Phase 3: Implement（实现）

**目标：** 分模块实现，覆盖 spec 所有项不遗漏。

**做法：**
1. 先读 spec 文件的「实现逐项验收清单」，每完成一项手动勾一项
2. 先写类型/接口定义
3. 再写单元测试
4. 再写实现
5. **每次 commit 前必须：**
   - Rust 改动：`cargo check && cargo clippy && cargo test`
   - 前端改动：`npm run build`
   - 全部跑过
6. **实现完成后、commit 前**：重新打开 spec 文件，逐条对照验收清单，确认全部完成。遗漏项补代码，不补不 commit。
7. F5 热重载验证
8. git-guardrails 自动保护

**完成条件：**
- [ ] cargo check + clippy + test 全过
- [ ] npm run build 通过
- [ ] 新代码有测试覆盖
- [ ] **spec 验收清单全部打勾 → 无遗漏**
- [ ] F5 运行验证前端加载正常
- [ ] 未改动不需要改的部分（YAGNI）

**依赖：** VSCode launch config（F5）、git-guardrails

---

## Phase 4: Review（审查）

**目标：** 人工过代码，确保质量。

**做法：**
1. `codebase-memory detect_changes` 看影响范围
2. `cargo clippy` + `cargo test` + `npm run build` 全量跑
3. `search_graph` / `trace_path` 验证调用链
4. 前端改动也走 review（特别检查：CSS 变量一致性、暗色模式、touch target）
5. 逐文件 review，讨论有疑问的地方
6. 你确认通过

**完成条件：**
- [ ] clippy + test + build 全过
- [ ] detect_changes 无异常
- [ ] 前端 review 过
- [ ] 你人工确认

---

## Phase 5: Ship（发布）

**目标：** 版本发布，更新 changelog + tag。

**做法：**
1. 运行 conventional-changelog 更新 CHANGELOG.md
2. 版本号 bump（major/minor/patch 按改动类型）
3. commit + tag
4. 触发 CI release workflow

**完成条件：**
- [ ] CHANGELOG.md 已更新
- [ ] 版本号合理（major=breaking / minor=feature / patch=fix）
- [ ] 已 commit + tag

**快捷方式：** 也可独立用 `/release` 命令

---

## Phase 6: Retrospect（复盘沉淀）

**目标：** 把学到的记下来，下次少走弯路。

**做法：**
1. CONTEXT.md 是否有新术语？ → 更新
2. 是否有值得记的 ADR？ → 写 docs/adr/
   - 判定标准：不可逆、无上下文则意外、真正做过取舍。三者缺一不写
3. 是否有重复踩的坑？ → 记 .claude/rules/
4. **检查 memory 系统：** 用户偏好、反复决策模式写入 memory 目录

**完成条件：**
- [ ] CONTEXT.md 已反映本轮新术语
- [ ] 需要记的 ADR 已写
- [ ] 重复性教训已记入 rules/
- [ ] 有值得记的用户偏好或决策模式 → 写入 memory
- [ ] 本轮 feature 的 fix commit 数 ≤ feat commit 数（目标：fix ≤ 20%）
