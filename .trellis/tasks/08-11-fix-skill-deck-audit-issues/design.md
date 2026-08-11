# 修复 Skill Deck 审计问题：技术设计

## Architecture and Boundaries

保持现有 Rust inventory → typed Tauri DTO → React 展示链路，不新增命令、状态库或领域模块。为区分非 Installation 的未知 root entry，允许对现有 inventory DTO 做最小扩展。

```text
Agent root entry
  -> inventory.rs 识别并过滤 Agent Root Artifact
  -> inventory.rs 将结构上不可能成为 Skill Package 的未知普通/特殊文件分类为 Unexpected Agent Root Entry
  -> inspect_entry / skill.rs 校验可能的 Installation
  -> externalInstallations[] | attentionEntries[]
  -> api.ts 原样保留 typed DTO
  -> App.tsx 本地化并展示 reason + offending path
```

## Backend Inventory

- 在 `inventory_for_roots` 把 root entry 交给 `inspect_entry` 前，把所有 root 的 `.DS_Store` 识别为 OS-owned Agent Root Artifact；仅当 `agent == Codex && legacy` 时把 `.system` 识别为 Agent-owned Artifact。
- 过滤只发生在 Agent root 的第一层；Skill Package 内部同名文件仍由 `skill.rs` 计数、校验和指纹处理。
- 不使用“跳过所有隐藏项”的宽泛规则，避免吞掉需要诊断的未知目录。
- 未知普通文件或特殊文件不再进入 `ExternalInstallation`；inventory 返回独立的结构化 Unexpected Agent Root Entry 诊断集合。
- 目录与链接仍交给 `inspect_entry` / `skill.rs`；目录或健康链接的目标内容校验失败时归为 Invalid Installation Candidate，避免隐藏可修复的候选。
- Broken External Installation 只覆盖无法解析、循环或越界的链接拓扑错误；健康链接目标的 metadata、内部链接或其他结构错误不得继续映射为 `BrokenLink`。
- `skill.rs`、Resource Boundary、内部 symlink/junction 拒绝策略不变。

## Frontend Data and State

- External diagnostic 继续使用 `commandErrorMessage` 的现有本地化能力，但不再覆写 `path: null`；logical installation path 固定单独展示，仅当完整绝对 offending path 不同时再展示该诊断路径。
- `visibleExternal` 在启用筛选非 `all` 时继续排除 External；新增一条静态 catalog 文案，并在筛选激活期间始终解释启用状态只适用于 Managed Skill。
- 所有权与启用状态筛选保持互相独立；不可能组合保留用户选择并显示范围说明，不自动重置。
- 用户可见筛选名改为 Management Scope；“Library 外部”包含有效 External、Broken 与 Invalid Candidate。为控制改动，React 内部 `managed/external` state 枚举保持不变。
- Broken 与 Invalid Candidate 保留在主 `skill-list`，但只有只读诊断信息，不提供 Adoption 或其他修改操作；Unexpected 不进入主列表。
- library summary 分别展示有效 External 数量与一个 `attentionCount`；后者汇总 Broken、Invalid Candidate 与 Unexpected，Settings/Diagnostics 保留各自领域分类和详情。
- inventory 加载错误以原始 payload 的包装对象保存在本地 state 中，渲染时才使用当前 locale 格式化。这样 `refresh` 不再闭包捕获 `copy.errors`，其依赖可稳定为空。
- 切换 locale 仍只更新 `localStorage` 与 React locale state，不触发 inventory/state command。

## Contracts

- 不改变 Tauri command 名称；`externalInstallations` 只包含通过 Structural Validation 的 External Installation，新增单一 `attentionEntries` 判别联合，以 `broken_external_installation`、`invalid_installation_candidate`、`unexpected_agent_root_entry` 三个稳定 kind 承载最小公共诊断字段。
- 不引入统一大而全的 `InventoryEntry`，也不为三类需处理条目各建数组；前端按 kind 决定主列表或 Settings/Diagnostics 展示位置。
- `DiagnosticsReport` 增加 `attentionCount` 与裁剪后的需处理条目；每项保留领域分类、Agent、logical path 和结构化错误，不读取或序列化文件内容。
- 错误仍由 Rust 生成稳定 `code/message/path`，React 只负责当前语言的展示。
- 外部路径和错误文本通过 React 文本节点展示，不渲染 HTML。

## Testing

- Rust：扩展 `inventory.rs` 单元测试，证明 `.DS_Store`、`.system` 被忽略，另一个普通无效目录仍保留 diagnostic。
- Rust：扩展 diagnostics 测试，证明 root diagnostic 计数和导出字段正确且不包含条目内容。
- React：使用 Vitest、现有 `react-dom` 与最小 DOM 环境渲染 `App`，mock Tauri `invoke`；覆盖 offending path 展示、启用筛选说明和语言切换不产生额外 invoke。
- 添加 `jsdom` dev dependency 作为 Vitest DOM 环境；复用原生 DOM 查询和现有 React DOM，不引入 Testing Library。

## Compatibility and Rollback

- 变更不迁移持久状态、不修改 Managed Library 或 Agent roots。
- 回滚只需恢复 inventory 的两个 entry 过滤条件和 App 展示/state 调整；没有数据回滚。
- Codex legacy `.system` 被视为 Agent 自有系统 Skill 容器而非用户可管理 Skill；其他 Agent root 中的同名目录仍进入诊断。若未来 Codex 规范改变其语义，应单独扩展 inventory root model，而不是删除 Skill Package 校验。

## Trade-offs

- 精确忽略两个噪声名比通用隐藏文件规则更保守，后续新增平台噪声需有实机证据再加入。
- 保持 External 在 enabled/disabled 下被排除，筛选语义准确，但用户需要切回“全部”才能查看 External；范围说明负责降低困惑。
