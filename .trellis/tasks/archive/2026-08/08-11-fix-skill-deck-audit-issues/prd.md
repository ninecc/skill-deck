# 修复 Skill Deck 审计问题

## Goal

让 Skill Deck 的本机 inventory 可读、可诊断、可筛选，并避免无关 UI 状态变化触发昂贵的全量文件扫描。

## Background

- 当前 macOS 实机 inventory 共发现 163 个 External Installation，其中 104 个显示为无效；94 个由内部 symlink/junction 校验策略触发，5 个结构无效，5 个 metadata 无效。
- Rust 侧严格拒绝 Skill Package 内部链接是既有安全与跨 Agent 可移植性约束，本任务默认不放宽该信任边界（`src-tauri/src/skill.rs:180`）。
- 当前后端将 Agent Skill 根目录下的每个直接子项都送入校验，因此 `.DS_Store`、`.system` 等非 Skill 项也进入 inventory（`src-tauri/src/inventory.rs:156`）。
- 后端 diagnostic 携带具体错误路径，但 External 列表渲染时主动把 `path` 清空，用户无法定位内部哪个文件触发校验（`src/App.tsx:511`）。
- `enabledFilter !== "all"` 时所有 External Installation 被直接排除，界面没有解释该过滤维度只适用于 Managed Installation（`src/App.tsx:112`）。
- inventory 刷新 callback 依赖当前语言的错误文案，切换语言会重新递归读取并哈希所有 Skill 文件（`src/App.tsx:80`、`src-tauri/src/skill.rs:286`）。
- 当前质量门全部通过：80 个 Rust 测试、5 个前端测试、格式、ESLint、TypeScript、生产构建和 Clippy；现有前端测试未覆盖 inventory 组件交互。

## Requirements

- R1：inventory 将所有 Agent Skill root 中的 `.DS_Store`，以及仅 Codex legacy root 中的 `.system`，识别为 Agent Root Artifact，并从 inventory、计数和诊断中排除；不得忽略其他可能承载有效 `SKILL.md` 的普通目录或链接。
- R1a：未知普通文件或特殊文件等结构上不可能成为 Skill Package 的项，不得建模或展示为 External Installation；必须作为 Unexpected Agent Root Entry 保留结构化诊断。目录和链接继续作为可能的 Installation 进入 Structural Validation。
- R1b：无法解析、循环或越界的外部链接归为 Broken External Installation；目录或可安全解析链接的目标内容未通过 Structural Validation 时归为 Invalid Installation Candidate，不得误报为 Broken External Installation 或 External Installation。
- R1c：`externalInstallations` 只返回通过 Structural Validation 的 External Installation；新增单一 `attentionEntries` 数组，以稳定判别字段区分 Broken External Installation、Invalid Installation Candidate 与 Unexpected Agent Root Entry。
- R2：External Installation 的无效状态必须展示可操作的具体诊断，包括后端返回的 offending path；不得展示 Skill 正文或新增安全评级。
- R2a：logical path 固定单独展示；只有 offending path 与 logical path 不同时才额外展示，避免重复路径。
- R3：启用状态过滤与 External Installation 的“无启用状态”语义必须一致；选择“已启用/已停用”时排除 External Installation，并显示“启用状态仅适用于 Managed Skill”的范围说明。
- R3a：启用筛选非“全部”时始终在筛选器附近显示范围说明，不依赖当前结果是否为空。
- R3b：Broken External Installation 与 Invalid Installation Candidate 保留在主列表，Unexpected Agent Root Entry 只在 Settings/Diagnostics 展示；首页 summary 使用一个 `attentionCount` 汇总三者，有效 External Installation 单独计数。
- R3c：Invalid Installation Candidate 只展示分类、logical path、非重复 offending path 和错误原因；通过后续扫描的 Structural Validation 前不提供 Adoption 或其他修改操作。
- R3d：用户可见的“所有权”筛选改为“管理范围”，选项为全部、Skill Deck 管理和 Library 外部；Library 外部包含有效 External、Broken 与 Invalid Candidate，内部筛选枚举无需改名。
- R4：切换语言只更新展示文案，不触发新的 inventory/state Tauri command。
- R5：新增最小自动化回归检查，覆盖根目录噪声过滤、具体诊断展示、启用状态筛选和语言切换不重扫。
- R6：保持既有内部 symlink/junction 拒绝策略、只读 External 所有权规则、离线 inventory 与 Resource Boundary 不变。

## Key Decisions

- 本任务采用聚焦 MVP，只修复 R1-R6，即已验证的 P0/P1 问题。
- 列表按名称/指纹聚合、中文 catalog 完整翻译、设置页版本/app-data 信息、CSP 和更广泛 UI 测试延后。
- 延续既有信任边界：Skill Package 内部 symlink/junction 仍被拒绝，不把结构校验包装成安全评级。
- 启用状态只属于 Managed Installation；External 在“已启用/已停用”结果中排除，界面同时解释筛选范围。
- Agent Root Artifact 不是 External Installation；首批只识别经实机确认的全 root `.DS_Store` 与 Codex legacy `.system`。
- Unexpected Agent Root Entry 保留诊断但不是 Installation；这项领域修正允许扩展 inventory DTO。
- Inventory 不改造成统一大而全的 entry union，也不为三类需处理条目分别增加数组；使用一个 `attentionEntries` 判别联合承载最小公共诊断字段。
- 目录和链接仍是 Installation candidate；即使 Structural Validation 失败也不归入 Unexpected Agent Root Entry。
- Broken External Installation 只表示链接拓扑故障；目录或健康链接的内容校验失败统一称为 Invalid Installation Candidate。
- Managed 与 External 计数保持原语义，仅通过 Structural Validation 的 External Installation 计入 External；Broken、Invalid Candidate 与 Unexpected 汇总为一个 `attentionCount`。
- diagnostics export 包含需处理条目的领域分类、Agent、logical path 与结构化错误，不读取或导出条目内容。
- offending path 使用后端返回的完整绝对路径，不新增相对路径转换。
- offending path 与 logical path 相同时不重复展示。
- 不可能的筛选组合保持用户选择并显示范围说明，不自动重置另一个筛选器。
- 启用筛选非“全部”时始终显示 Managed-only 范围说明。
- “Library 外部”是筛选范围而非新的领域实体；它包含有效 External、Broken 与 Invalid Candidate，Unexpected 仍只在 Settings/Diagnostics 中出现。
- 允许增加仅开发期的 `jsdom` 依赖以覆盖真实 React 交互，不引入额外测试框架。

## Acceptance Criteria

- [ ] `.DS_Store` 不再出现在任何 Agent inventory、计数或列表中；Codex legacy `.system` 同样被排除，但其他 root 的同名未知目录不会被静默忽略。
- [ ] 未知普通文件或特殊文件作为 Unexpected Agent Root Entry 提供结构化诊断，不计作 External Installation。
- [ ] `externalInstallations` 中每项均包含通过 Structural Validation 的 Skill；三类无效或异常条目只出现在带稳定 kind 的 `attentionEntries` 中。
- [ ] 无效目录与健康链接仍保留为可诊断的 Invalid Installation Candidate 行，不被普通文件分类规则吞掉，也不计作 External Installation。
- [ ] 只有链接无法解析、循环或越界时才显示 Broken External Installation；健康链接目标的 metadata 或结构错误不得误报为 BrokenLink。
- [ ] 内部链接导致的无效 Skill 显示具体 offending path，用户可以定位触发项。
- [ ] offending path 与 logical path 相同时只显示一次路径。
- [ ] 选择“已启用/已停用”时排除 External Installation，并显示该筛选仅适用于 Managed Skill；交互测试固定该行为。
- [ ] Managed-only 范围说明在启用筛选激活期间始终可见。
- [ ] 首页 summary 分别显示有效 External 数量与统一 `attentionCount`；Broken 和 Invalid Candidate 保留在主列表，Unexpected 不进入主列表。
- [ ] Invalid Installation Candidate 只有只读诊断信息，通过 Structural Validation 前没有 Adoption 或其他修改操作。
- [ ] “Library 外部”筛选显示有效 External、Broken 与 Invalid Candidate；启用状态筛选激活时排除这些条目并持续显示 Managed-only 说明。
- [ ] Settings/Diagnostics 可查看全部需处理条目；diagnostics export 包含领域分类、Agent、logical path 和结构化错误，不包含文件正文。
- [ ] 切换中英文不会调用 `inventory` 或 `state_status`。
- [ ] Rust 与前端新增的回归检查通过，既有完整质量门保持通过。
- [ ] inventory 仍不联网，内部链接仍被拒绝，External Installation 仍默认只读。

## Out of Scope for Recommended MVP

- 放宽或自动跟随 Skill Package 内部 symlink/junction。
- 列表虚拟化或新增生产运行时依赖。
- 市场、批量修复、自动修改第三方 Skill 内容。
- P2/P3 体验与安全加固项，除非用户选择扩展范围。

## Open Questions

- 无。

## Planning Sign-off

- 2026-08-11：用户确认完整领域分类、筛选语义、DTO 边界、只读操作范围、测试范围与实施计划；grilling 完成，规划冻结，等待单独授权实现。
