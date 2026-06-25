# Mihoyo Widget — 开发工作流设计

**日期**：2026-06-24

**状态**：Draft

**融合工具：**
- `git-guardrails-claude-code` — git 安全保护
- `domain-modeling` — CONTEXT.md + ADR 维护
- `codebase-memory-mcp` — 代码知识图谱查询
- VSCode launch config — F5 热重载

---

## 整体骨架

```
Grill → Spec → Implement → Review → Ship → Retrospect
```

**触发方式：** 对话式。你说"我有个想法"／"加个功能"／"改个东西"即开始流程。
**工作流名称：** `/dev-flow`（slash command 入口，但通常对话式触发）

---

## Phase 1: Grill（需求打磨）

**触发条件：** 用户提出一个新想法 / 功能 / 改动。

**流程：**
1. 用户自由描述需求
2. 同时查 `codebase-memory get_architecture` 掌握当前架构
3. 读 `CONTEXT.md` 掌握领域术语
4. 读相关 `docs/adr/` 避免踩已有决策
5. 一轮接一轮追问，敲实：做什么、不做什么、成功标准
6. 输出到 `docs/specs/<日期>-<功能名>.md`

**判定条件：**
- [ ] 功能边界清晰（做什么 vs 不做什么）
- [ ] 涉及的模块 / seam 已定位
- [ ] 领域术语与 CONTEXT.md 一致
- [ ] 可以进入设计阶段

**输出物：** `docs/specs/YYYY-MM-DD-<topic>.md`

---

## Phase 2: Spec（接口 / 架构设计）

**触发条件：** Grill 完成，功能边界已敲实。

**流程：**
1. 重新读相关 ADR，确认没有踩到已被否决的方向
2. 如果需要新数据类型 → 先定义 Rust struct / type，用户过目
3. 如果需要新 API → 定义接口签名和边界，用户过目
4. 如果需要跨 crate 改动 → 查 `codebase-memory trace_path` 找调用链
5. 确认设计符合 CONTEXT.md 中的领域语言
6. 确认是否影响现有通知规则（`notify/rules.rs`）

**判定条件（全部满足才进实现）：**
- [ ] 数据结构 / API 签名已确认
- [ ] 不影响（或已更新）现有 ADR
- [ ] 与 CONTEXT.md 术语一致
- [ ] 改动范围 / 调用链已明确（codebase-memory 确认）
- [ ] 是否要写测试已有明确计划
- [ ] 通知规则影响已评估

**输出物：** 确认好接口签名 + 改动范围即可。

---

## Phase 3: Implement（实现）

**触发条件：** Spec 确认完成。

**流程：**
1. 如果改动涉及新增逻辑 → 先写核心类型/接口定义
2. 再写测试（小范围单元测试）
3. 再写实现
4. 每完成一个逻辑单元就 `cargo check` / `cargo clippy`
5. F5 启动验证（前端热重载连上后端）
6. 每次 git 操作前 `git-guardrails` 自动保护

**判定条件（全部满足才进 Review）：**
- [ ] `cargo check` 无错误
- [ ] `cargo clippy` 无新增 warning（或记录过的 intentional）
- [ ] 新代码有测试覆盖（至少核心逻辑）
- [ ] F5 运行验证：前端页面加载正常
- [ ] codebase-memory 重新索引后架构无异常
- [ ] 未改动不需要改的部分（YAGNI）

---

## Phase 4: Review（审查）

**触发条件：** 实现完成，代码已编译通过。

**流程：**
1. `codebase-memory detect_changes` → 看改了哪些文件、影响符号
2. `cargo check` + `cargo clippy` 全量过一遍
3. `codebase-memory search_graph` / `trace_path` → 确认调用链完整
4. 逐文件过代码，有疑问的地方讨论
5. 用户确认后 review 通过

**判定条件（全部满足才进 Ship）：**
- [ ] `cargo clippy` 无新增 warning
- [ ] `cargo test` 全通过
- [ ] codebase-memory 检测无异常（死代码、未用导出等）
- [ ] CONTEXT.md / ADR 是否需要更新已评估
- [ ] 用户人工确认过改动

---

## Phase 5: Ship（发布）

**触发条件：** Review 通过。

**流程：**
1. 运行 `conventional-changelog` 更新 CHANGELOG.md
2. 版本号 bump（根据改动推断 major / minor / patch）
3. `git add` → `git commit`（message 含 changelog 内容）
4. `git tag v<version>`
5. 触发 GitHub Actions release workflow（可选手动触发）

**判定条件：**
- [ ] CHANGELOG.md 已更新
- [ ] 版本号合理（major=breaking / minor=feature / patch=fix）
- [ ] 已 commit + tag

---

## Phase 6: Retrospect（复盘沉淀）

**触发条件：** Ship 完成。

**流程：**

CONTEXT.md 检查清单：
- 有没有新概念在本轮引入？ → 当场更新 CONTEXT.md
- 有没有现有术语需要修正？ → 当场更新

ADR 检查清单：
- 有没有值得记的架构决策？（hard to reverse + surprising + had alternatives）
- 有 → 写 `docs/adr/<number>-<topic>.md`

Rules 检查清单：
- 有没有反复踩的坑？ → 记到 `.claude/rules/<topic>.md`
- 有没有下次该自动知道的约束？ → 同理

**判定条件：**
- [ ] CONTEXT.md 已反映本次改动的新术语
- [ ] 需要记的 ADR 已写
- [ ] 重复性教训已记入 rules/

---

## 与现有工具的融合

| 工具 | 出现环节 | 作用 |
|------|----------|------|
| git-guardrails | Phase 3, 5 | 防误 git 操作 |
| CONTEXT.md | Phase 1, 2, 6 | 统一术语 / 记录新术语 |
| ADR | Phase 2, 6 | 避免重复决策 / 记录新决策 |
| codebase-memory | Phase 1-4 | 架构查询 / 影响分析 / 调用链追踪 |
| F5 launch config | Phase 3 | 热重载调试 |
| conventional-changelog | Phase 5 | 自动生成 changelog |
| GitHub Actions | Phase 5 | 自动发布构建 |

## 未覆盖 / 待定

- 跨 session 手递（`handoff`） — 当前用 /dev-flow 在一个 session 内走完流程，不需要拆分
- 原型验证（`prototype`） — 需要时独立调用
- 跨模型协作（`codex:review`） — 当前单人开发不需要
