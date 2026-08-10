# Technical Design

## Architecture

采用 Tauri 2 桌面壳、React + TypeScript 前端与 Rust 后端。前端负责列表、导入向导、确认和错误展示；所有文件系统、Agent 配置、所有权校验与 Git 操作都在 Rust 中完成。

```text
React UI
  -> typed Tauri commands
  -> SkillManager deep module
       -> CodexAgent adapter
       -> ClaudeAgent adapter
       -> local/Git source implementations
       -> JSON state + app-owned library
       -> transactional filesystem operations
```

不引入数据库、后台服务或通用插件框架。状态规模很小，使用 Skill Deck 私有 app-data 中的版本化 JSON 清单与 Managed Library；不复用 Agent canonical root，也不写第三方 lock。

## Deep Modules and Seams

### SkillManager module

这是前端和测试共用的主要 interface，隐藏扫描合并、校验、所有权、内容指纹、事务、Agent 配置和 Git 细节。MVP 只暴露三个入口：

- `inventory()`：返回规范化的 Skill 视图和诊断信息。
- `plan(operation)`：对 import/adopt/install、启停、检查或应用更新、rollback、restore、detach、uninstall、remove/export 等领域操作执行所需的 staging、校验和预览，返回一次性 `OperationPlan`。
- `commit(plan_id)`：验证 plan 仍新鲜后，在单写入锁内提交或回滚；无需写入的检查、导出等 operation 直接返回结果。

`Operation` 是带类型参数的封闭枚举，不提供任意插件命令。需要用户确认的差异、路径、资源观测值和 Copy Fallback 都进入 `OperationPlan`，避免前端绕开相同的 preflight 规则。

Tauri commands 只做输入反序列化和结果序列化，不重新实现业务规则。

### Agent seam

Codex 与 Claude Code 是两个真实 adapter，因此建立一个窄的内部 interface：解析用户级根目录、扫描安装、读取/修改启用状态和清理由本应用拥有的配置条目。

- Codex adapter：Skill 根目录 `$HOME/.agents/skills`；停用状态写入 resolved Codex State Root 下 `config.toml` 的路径级配置。
- Claude adapter：默认 Skill 根目录 `~/.claude/skills`；使用 Claude Code 的 `skillOverrides` 状态。

路径解析分别建模：OS home API 决定 Codex `$HOME/.agents/skills`；Codex State Root 从有效 `CODEX_HOME` 或默认 `~/.codex` 得到 config 路径；Claude Personal Root 从 `CLAUDE_CONFIG_DIR`、受约束 override 或默认 `~/.claude` 得到 skills/settings。Settings 展示 detected/override/resolved 三组值。Windows native 不枚举 WSL。

配置编辑必须使用保留未知字段的结构化编辑方式，并采用同目录临时文件 + 原子替换；修改前保留可回滚内容。

### Source implementations

本地目录和公开 HTTPS Git 是两个真实来源实现，但 seam 保持在 `SkillManager` 内部：两者最终都产出一个 staging 目录、来源元数据和可选 commit OID。Git 使用 `git2`/libgit2，不依赖终端用户安装 Git。

Skill Source 在 MVP 中不可转换：Local 只能用新本地快照替换 revision；Git 的 repository URL、subpath 和 tracked branch 不可变。需要换源时删除零 Installation package 后重新导入。

## Data Model

版本化 JSON 状态至少包含：

- `ManagedSkillPackage`：稳定 ID、规范化名称、app-owned library 路径、Skill Source 和 Installed Revision。
- `SkillSource`：`local_snapshot`，或 `git { repository_url, subpath, tracked_branch }`。
- `InstalledRevision`：内容指纹；Git-backed revision 另含 commit OID。
- `PreviousRevision`：上一 Installed Revision 的唯一保留副本；新 revision 成功后替换更老的 Previous Revision。
- `Installation`：Agent、logical path、resolved target、`symlink | junction | copy_fallback` Deployment Mode、独立启用状态、接管时间、最后已知内容指纹与 Configuration Provenance。
- `StateVersion`：用于未来显式迁移；未知的新版本必须停止写入而不是猜测兼容。

扫描结果把状态清单与两个 Agent 根目录的真实文件合并：同一 Managed Skill Package 聚合其多个 Installation，且所有 Installation 指向同一个 Installed Revision；清单中不存在的安装显示为 External Installation；同名同指纹可建议聚合但不自动接管；同名异内容保持分离；清单存在但文件缺失或指纹变化时显示诊断状态。

Managed Library 对规范化 name 建立唯一索引；同名 import 在 staging 校验后、写入前失败并返回现有 package ID。一个多-Skill repository 每次只选择并提交一个 subpath。

所有 mutation 由进程级单写入锁串行化；第二个窗口或 command 可读取最新 inventory，但 mutation 返回 busy 状态而不并发触碰 state、library 或 Agent roots。

## Validation and Trust Rules

- 使用 OS home-directory API 解析路径，不硬编码平台用户目录。
- 将外部文件和 YAML 视为不可信输入；限制 frontmatter 大小并按 Agent Skills 规范校验名称与描述。
- 在任何内容进入 Managed Library 前统一执行固定版本化 Resource Boundary：Git transfer 250 MiB、checkout repository 500 MiB、Skill Package 100 MiB、10,000 files、single file 50 MiB、`SKILL.md` 1 MiB。策略返回限制名、阈值和观测值，MVP 无覆盖设置。
- 目录名必须等于规范化 `name`，目标路径只由已验证名称拼接，禁止 `..`、绝对路径和路径分隔符。
- MVP 拒绝 Skill Package 内容中的 symlink/junction，避免复制时逃逸来源目录；只允许由 Skill Deck 创建、Agent logical path 指向 Managed Library 的入口链接。不执行任何脚本。
- Git 仅允许公开 `https://` URL；禁用 submodule/LFS 自动处理，clone 后仍执行与本地来源相同的校验。
- UI 不直接渲染未经净化的 Skill Markdown HTML；MVP 可显示纯文本元数据与文件路径。

## Transaction Flows

### Import/install

1. 将来源复制或 clone 到 app-data staging。
2. 校验目录结构、`SKILL.md`、路径和不支持特性。
3. 检查 app library 与所有目标 Agent 的同名冲突。
4. 写入 Managed Library；macOS/Linux 为 Agent logical path 准备 relative directory symlink，Windows 准备 absolute directory junction。
5. 逐个原子切换到最终入口；链接失败时停止并请求用户确认 Copy Fallback，不静默复制。任一步失败则删除本次创建的入口并恢复状态清单。
6. 原子写入状态 JSON，再刷新 inventory。

Import 默认零目标，因而可仅 Add to Library。staging/validation 可接收取消信号并清理；进入跨目标 commit/rollback 临界区后忽略取消直到事务得到确定结果。

Agent 客户端探测仅用于提示，不是安装前置条件。用户确认后可创建缺失的官方用户级 Skill 根目录；新 Installation 默认 enabled，除非外部 configuration provenance 决定其他状态。

### Adoption

对用户明确选择的一个或多个同名同指纹 External Installation 重新校验。普通目录内容和健康 link target 都先复制到 Managed Library；选中的 Agent 入口再原子切换为 Skill Deck-owned link/junction。入口之外的 external target 与第三方 lock 不修改，来源建立为 Local Skill Source snapshot。记录指纹、Deployment Mode 和既有 Configuration Provenance。

### Update

fetch tracked branch 并计算 installed commit 与 remote HEAD 的 ancestry；相等表示无更新，installed commit 是 remote HEAD 祖先时才进入普通更新，其他关系报告 Source Diverged 且不替换。网络错误报告 Source Unreachable，branch 或 subpath 消失报告 Source Missing。候选名称或目录名改变时拒绝为不兼容更新。

只有 app-owned revision 与全部 Installation 都未漂移时才更新；任一 drift 会冻结整个 package 的更新。新 revision 在 staging 校验，所有 Installation 作为一个事务替换；每个目标先保留同目录备份，全部成功后提交新 Installed Revision，失败则全部恢复。

更新预览生成确定性的文件变更与 Capability Disclosure 差异；tag 和 Skill metadata version 只展示，不影响决策。

成功提交新 revision 后，旧 Installed Revision 成为唯一 Previous Revision，更老 revision 被删除。Roll Back Revision 以多 Installation 原子事务交换 current 与 previous，允许单步 redo，但不产生无限历史；Git Skill Source 保持不变。

### Enable/disable and uninstall

启停只通过 Agent adapter 修改配置，不用移动/改名目录模拟通用机制。Adapter 记录 Configuration Provenance，Enable/Disable 只修改 Skill Deck 创建的配置。Externally Controlled Configuration 锁定切换控件；Configuration Drift 只可通过 Reapply 或 Forget 显式解决。Detach 保持外部配置原样；Uninstall 只清理本应用创建的配置，不删除用户或第三方条目。Managed Skill Package 与 app-owned revision 在无 Installation 时仍保留。Remove from Library 仅对零 Installation package 开放。

Content Drift 会冻结会覆盖或删除内容的动作。Restore Installation 展示 diff 并二次确认后用 Installed Revision 覆盖目标。Detach Linked Installation 先在 logical path 原子落地 standalone copy，再撤销关联和配置 provenance；Copied Installation 直接撤销关联。失败时保持原 managed 状态。

broken、cyclic、outside-policy external link 只进入只读诊断，不允许生命周期动作。第三方 lock/list 数据只能作为 UI 来源提示。

Codex legacy `$CODEX_HOME/skills` 进入 Legacy External Installation inventory。Legacy Migration 只复制内容到 Managed Library；只要同名 legacy entry 仍存在，就阻止面向 current user root 的 Installation。用户在应用外移除 legacy 后，下一次 inventory 才解锁安装。

### Recovery

状态文件每次事务写入前保留最后一个有效备份。主状态损坏时只从有效备份恢复；两者都不可用时进入 Read-only Recovery：Agent 目录全部视为 External Installation，app-owned 内容视为 Orphaned Package，不根据路径、名称或指纹推断所有权。

## Frontend Shape

- Inventory 首页：搜索与按 Agent、启用状态、所有权过滤；显示冲突和漂移诊断。
- Import 流程：本地目录或公开 HTTPS URL、Git 子目录选择、目标 Agent 选择、校验预览、确认。
- Detail/Actions：接管、启停、检查更新、更新、Restore Installation、Detach Installation、Uninstall 与 Remove from Library；危险操作必须显示目标路径和语义差异。
- Import/Update disclosure：分别显示 Structural Validation、Capability Disclosure 和 Change Disclosure，始终避免安全评级措辞。
- Settings/Diagnostics：实际解析出的 Agent 根目录、应用数据目录、版本与构建信息。
- Remove from Library：展示 Source、current/previous revision 和磁盘占用；可把 current revision 复制导出到用户选择的目录；Local snapshot 显示最后副本警告，并要求输入规范化 name。
- 每次成功写操作统一显示 restart fallback，不实现按 Agent 区分的 reload 状态机。

前端只保留短生命周期 UI 状态；inventory 每次操作后从 Rust 重新读取，不建立第二套持久化真相。

应用启动和 inventory 不调用网络。Git fetch 只存在于用户显式触发的检查/更新 command 中。

应用不集成 analytics 或 crash-reporting SDK。结构化诊断日志保存在本地，默认避免 Skill 正文、完整 home path 之外的不必要敏感信息；导出由用户动作触发并先展示包含内容。

UI strings 通过本地静态 catalog 集中管理，提供 `zh-CN` 与 `en`，初始 locale 跟随系统并允许持久化覆盖；领域枚举与错误码不直接充当展示文案。

## Compatibility and Distribution

- Windows：Windows runner 构建 x64 NSIS 安装包。
- macOS：macOS runner 以 `universal-apple-darwin` 构建，最低系统设为 12.0，并产出 DMG。
- Linux：固定 Ubuntu 22.04 兼容基线构建 x86_64 AppImage，并记录实际烟测发行版。
- 三端原生 CI 分别构建，不做跨平台交叉打包。
- 签名与 notarization 通过 CI secrets 接入；没有证书时仍可产出开发验证包，但不得宣称为无警告的公开发行版本。
- 应用卸载器不调用任何 SkillManager lifecycle operation，不删除 Agent Installation、配置或 Managed Library。若 library 被外部清除，Copied Installation 仍可成为 External Installation，而 link/junction 只能识别为 Broken External Installation。任何应用内清库动作必须先转换或删除全部 Linked Installation。
- Windows native 构建不访问 `\\wsl$`；WSL 使用独立 Linux home/runtime 语义。

## Trade-offs

- link preferred：复用单一 Installed Revision 并匹配现有生态，但 Managed Library 成为 Linked Installation 的运行依赖；Copy Fallback 增加同步与漂移处理成本。
- JSON 而非 SQLite：查询能力较弱，但 MVP 状态量小、迁移和调试简单。
- `git2` 而非系统 Git：增加 Rust 依赖和构建时间，但消除终端用户 Git/PATH 前置条件。
- 严格可移植校验：会拒绝 Claude Code 能宽松接受的部分 Skill，但保证双 Agent 安装语义一致。

## Operational and Rollback Notes

- 所有写操作返回结构化错误码、用户可读消息和相关路径；日志不得记录凭据或完整 Skill 正文。
- 应用崩溃后启动时清理过期 staging，并检测遗留备份；无法安全判断时停止自动写入并提示人工恢复路径。
- 状态 JSON、Agent 配置和安装目录的故障注入测试覆盖中途失败与恢复。
