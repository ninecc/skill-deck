# Skill Deck Roadmap

> 基于 2026-08-11 的[市场调查](research/skill-manager-landscape.md)与当前实现校准。

## 当前阶段

Skill Deck 已完成最初路线中大部分“可信 lifecycle”能力：Codex/Claude Code user-level inventory、本地与公开 HTTPS Git 导入、Managed Library、所有权与 Configuration Provenance、link/copy 安装、Adoption、启停、Git ancestry 更新、Content Drift 防护、单步 rollback、detach/uninstall/remove、资源边界、只读恢复和本地诊断。

因此后续不再按“从零建设包管理器”的顺序推进。当前产品是一个**可靠的双 Agent lifecycle 内核**，下一个目标是把它升级成能主动解释状态、能力和效果的 control plane。

## 路线原则

1. **先完成主动对账，再扩 host。** Managed Installation 的异常应在 inventory 时出现，而不是等到 mutation preflight 才暴露。
2. **先展示证据，再执行策略。** Capability 先做到可追溯 disclosure；没有 host enforcement 时明确标为 advisory。
3. **先建立小型 eval，再做推荐。** 没有 revision-level 效果数据时，不用安装量、星标或启发式包装成质量推荐。
4. **增加一个真实 host 后再抽象 adapter。** 不为尚不存在的第五、第六个 Agent 建通用插件框架。
5. **继续保持 manager/runtime 分离。** Skill Deck 不执行第三方 Skill 脚本，不自建 agent loop。

## 分阶段路线

### R1 — 完成可信 Inventory

目标：让首页成为真实 reconciliation 结果，而不是 persisted state 与目录扫描的并列展示。

- [x] 为每个 Managed Installation 计算 `healthy / missing / drifted / retargeted / configuration_drift / broken`。
- [x] 核对 logical path、resolved target、deployment mode、installed fingerprint、library revision 和 configuration provenance。
- [x] 为异常提供与所有权边界一致的 Restore、Detach、Reapply、Forget 或只读诊断入口。
- [x] 增加 Codex/Claude Code Agent Projection Contract smoke tests，并在三平台打包前执行。
- [ ] 完成 Windows NSIS、macOS DMG、Linux AppImage 安装包人工烟测与 Agent Runtime Recognition 验证。

**进度：** 主动对账和 Agent Projection Contract smoke 已完成；R1 仅剩三平台安装包人工烟测。

**完成门槛：** 启动扫描即可发现受管内容丢失、改写、重新指向和配置漂移；任何自动修复都不越过已证明的 ownership。

### R2 — Evidence-backed Capability Card

目标：把现有 scripts/references/unknown-fields 摘要升级为安装和更新决策所需的事实卡片。

- 规范化 `tool / mcp / command / network / file / secret` capability kind。
- 每条 capability 记录 evidence path、提取方式和 `advisory / host-enforced` 状态。
- Import 展示完整 disclosure；Update 同时展示 content diff 与 capability diff。
- 不给“安全/不安全”结论，不生成不可解释的风险总分。
- 增加第三个高重合 Agent Target；优先验证 Copilot/VS Code，真实重复出现后再提取 adapter seam。

**完成门槛：** 用户能回答“新 revision 新增了什么能力、证据在哪里、目标 host 能否约束”，且未知仍显示为未知。

### R3 — Revision-level Eval

目标：开始证明 Skill 是否有效，而不只证明其来源和完整性。

- 兼容 `skill-up`/Anthropic 风格的 3–5 case 最小 suite。
- 支持 without-skill 与 with-skill baseline comparison。
- 首批维度只做 trigger、task success、cost、latency 和 coexistence regression。
- 结果绑定 revision、host、model、suite hash；未运行显示“未评估”。
- Skill 运行交给目标 Agent；Skill Deck 只编排、记录和比较证据。

**完成门槛：** 更新候选可以用相同 suite 与当前 revision 比较，并能因明显回归而停止 promote。

### R4 — Local Router 与安全演化

目标：在已有本地证据上选择最小 Skill set，并把失败转成可审查的候选改进。

- 使用 repo signals、当前任务、token budget、冲突图和 eval 结果做确定性排序。
- 明确解释 selected/excluded 原因；没有足够证据时不伪装成个性化推荐。
- Usage Event 区分 `included / invoked / completed / failed / feedback` 及证据置信度。
- 失败只生成 candidate patch + regression case，经 capability diff、eval、人审和 canary 后才能 promote。

**完成门槛：** Router 的选择可以被解释和回放；任何自动演化都不能静默修改 current revision。

## 暂不建设

- 自建公共 Marketplace、账号、支付和云同步。
- 通用 Skill dependency solver。
- 内置 Agent runtime、第三方脚本执行器或自建沙箱。
- 后台静默更新、静默演化和在线自动切流。
- 不可解释的单一质量分或安全分。
- 为小规模 inventory 提前把版本化 JSON 迁移到 SQLite；只有 usage/eval 事件量和查询需求实际出现后再迁移。

## 产品叙事

短期定位：

> 最安全、最可解释的跨 Agent Skill inventory 与更新回滚工具。

长期定位：

> 能证明某个 Skill revision 在特定 Agent、模型和任务上是否真正有效的 local-first evidence control plane。

产品演进顺序：

```text
可靠清单 → 主动对账 → 能力证据 → Revision 级评价 → 本地路由 → 安全演化
```
