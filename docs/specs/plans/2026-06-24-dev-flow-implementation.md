# Mihoyo Dev Flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 创建 /dev-flow 对话式开发工作流，整合 git-guardrails、domain-modeling、codebase-memory 三件已有工具，覆盖 Grill → Spec → Implement → Review → Ship → Retrospect 全流程

**Architecture:** 核心是一个 model-invoked skill（`.claude/skills/mihoyo-dev-flow/SKILL.md`），通过 description 域自动触发。六阶段流程在其 body 中定义，各阶段引用现有 skill/tool。另加一个独立 release command 和一个 rules 文件。

**Tech Stack:** Claude Code skills + commands + rules 体系，无需额外安装

---

### Task 1: 创建项目规则文件 `.claude/rules/dev-flow.md`

**Files:**
- Create: `.claude/rules/dev-flow.md`

- [ ] **Step 1: 创建 dev-flow rules 文件**

文件内容：
```markdown
---
paths: "apps/desktop/src/** packages/core/src/** packages/game-hsr/src/**"
---
# Mihoyo Widget — 开发流程规则

## 项目结构
- Rust workspace: packages/core（共享基础设施）、packages/game-hsr（星铁逻辑）、apps/desktop（Tauri 桌面壳）
- 前端: packages/frontend（Vanilla JS + Vite）
- 领域术语定义在项目根 CONTEXT.md
- 架构决策记录在 docs/adr/

## 已有工具
- codebase-memory-mcp: 代码知识图谱，用于架构查询和影响分析
- git-guardrails: 拦截 git push / reset --hard / branch -D 等危险操作
- CONTEXT.md + ADR: 领域术语和架构决策的持久化

## 编译调试
- 调试: F5（launch config "🔧 Rust + Vite (hot reload)"）
- cargo check / clippy 在提交前必须过
- 前端用 Vite dev server (localhost:5173)，tauri.conf.json 中 devUrl 指向它
```

- [ ] **Step 2: 验证文件已写入**

```bash
cat /media/jayhaul/dev/Code/mihoyo-widget/.claude/rules/dev-flow.md
```
Expected: 文件内容正常显示

---

### Task 2: 创建主编排 Skill `mihoyo-dev-flow`

**Files:**
- Create: `.claude/skills/mihoyo-dev-flow/SKILL.md`

- [ ] **Step 1: 创建 skill 目录**

```bash
mkdir -p /media/jayhaul/dev/Code/mihoyo-widget/.claude/skills/mihoyo-dev-flow
```

- [ ] **Step 2: 编写 SKILL.md**

```markdown
---
name: mihoyo-dev-flow
description: Mihoyo Widget 全流程开发工作流。当用户说要加功能、改东西、修bug、发版本时触发。包含 Grill → Spec → Implement → Review → Ship → Retrospect 六个阶段。自动融合 codebase-memory、CONTEXT.md、ADR 和 git-guardrails。单人Rust Tauri 桌面端项目。
disable-model-invocation: false
---

# Mihoyo Dev Flow

一套 6 阶段开发工作流，覆盖从想法到发布的完整过程。对话式触发 — 你说出需求，我带你走完。

开始前，先确认：
1. 读 CONTEXT.md 和已有 ADR 了解当前架构上下文
2. 调 `codebase-memory get_architecture` 看当前项目全景
3. 开始 Grill 阶段

---

## Phase 1: Grill（需求打磨）

**目标：** 把模糊想法敲实，明确范围和成功标准。

**做法：**
1. 听你描述需求
2. 追问边界（做什么 / 不做什么）
3. 用 CONTEXT.md 统一术语
4. 用 `codebase-memory` 定位影响模块
5. 输出到 `docs/specs/YYYY-MM-DD-<topic>.md`

**完成条件：**
- [ ] 功能边界清晰
- [ ] 涉及模块已定位
- [ ] 领域术语一致
- [ ] 可以进入设计阶段

## Phase 2: Spec（接口设计）

**目标：** 敲定数据结构和接口签名，但不动实现代码。

**做法：**
1. 读相关 ADR 不踩坑
2. 定义 Rust struct / type / API 签名，给你确认
3. 跨 crate 改动时用 `codebase-memory trace_path` 追踪调用链
4. 评估通知规则影响（notify/rules.rs）

**完成条件：**
- [ ] 接口签名和数据结构已确认
- [ ] 改动范围明确
- [ ] 可进入实现

## Phase 3: Implement（实现）

**目标：** 实现代码，TDD 方式：类型定义 → 测试 → 实现。

**做法：**
1. 先写类型/接口定义
2. 再写单元测试
3. 再写实现
4. 每单元 `cargo check` + `cargo clippy`
5. F5 热重载验证
6. git-guardrails 自动保护

**依赖：** VSCode launch config（F5）、git-guardrails

## Phase 4: Review（审查）

**目标：** 人工过代码，确保质量。

**做法：**
1. `codebase-memory detect_changes` 看影响范围
2. `cargo clippy` + `cargo test` 全量跑
3. `search_graph` / `trace_path` 验证调用链
4. 逐文件 review，讨论有疑问的地方
5. 你确认通过

**完成条件：**
- [ ] clippy + test 全过
- [ ] detect_changes 无异常
- [ ] 你人工确认

## Phase 5: Ship（发布）

**目标：** 版本发布，更新 changelog + tag。

**做法：**
1. 运行 conventional-changelog 更新 CHANGELOG.md
2. 版本号 bump（major/minor/patch 按改动类型）
3. commit + tag
4. 触发 CI release workflow

**快捷方式：** 也可独立用 `/release` 命令

## Phase 6: Retrospect（复盘沉淀）

**目标：** 把学到的记下来，下次少走弯路。

**做法：**
1. CONTEXT.md 是否有新术语？ → 更新
2. 是否有值得记的 ADR？ → 写 docs/adr/
3. 是否有重复踩的坑？ → 记 .claude/rules/
```

- [ ] **Step 3: 验证 skill 文件可读**

```bash
cat /media/jayhaul/dev/Code/mihoyo-widget/.claude/skills/mihoyo-dev-flow/SKILL.md | head -5
```
Expected: 显示 frontmatter

---

### Task 3: 创建 Release Command `/release`

**Files:**
- Create: `.claude/commands/release.md`

- [ ] **Step 1: 创建 command 文件**

```markdown
---
name: release
description: 发布新版本 — 更新 CHANGELOG、bump 版本号、commit + tag
---

# Release

执行发布流程。需要先确认 Review 已通过、代码已合入 main。

## Steps

1. 确认当前在 main 分支，没有未提交改动
2. 确认前一次 commit 到现在的改动类型（breaking / feature / fix）
3. **更新 CHANGELOG**
   ```bash
   cd /media/jayhaul/dev/Code/mihoyo-widget/packages/frontend && npx conventional-changelog -p conventionalcommits -i CHANGELOG.md -s
   ```
   或全量重生成（包括历史）：
   ```bash
   cd /media/jayhaul/dev/Code/mihoyo-widget/packages/frontend && npx conventional-changelog -p conventionalcommits -i CHANGELOG.md -s -r 0
   ```
4. **版本号 bump**
   根据改动类型推断 major/minor/patch，更新 packages/frontend/package.json 中的 version 字段
5. **提交**
   ```bash
   git add CHANGELOG.md package.json
   git commit -m "chore: release v<version>"
   git tag v<version>
   ```
6. 推送到远程
7. 触发 GitHub Actions release workflow（如果有）
```

- [ ] **Step 2: 验证文件可读**

```bash
cat /media/jayhaul/dev/Code/mihoyo-widget/.claude/commands/release.md | head -5
```
Expected: 显示 frontmatter

---

### Task 4: 验证端到端工作流

**Files:** 不修改

- [ ] **Step 1: 确认所有文件存在**

```bash
ls -la /media/jayhaul/dev/Code/mihoyo-widget/.claude/skills/mihoyo-dev-flow/SKILL.md
ls -la /media/jayhaul/dev/Code/mihoyo-widget/.claude/commands/release.md
ls -la /media/jayhaul/dev/Code/mihoyo-widget/.claude/rules/dev-flow.md
```

- [ ] **Step 2: 验证 git-guardrails 仍在工作**

```bash
echo '{"tool_input":{"command":"git push origin main"}}' | /media/jayhaul/dev/Code/mihoyo-widget/.claude/hooks/block-dangerous-git.sh 2>&1; echo "exit: $?"
```
Expected: exit 2, BLOCKED message
