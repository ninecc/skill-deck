# AI Agent Skill 管理器

## Goal

为同时使用 Codex 与 Claude Code 的个人开发者提供一个本地优先的图形化工具，统一发现、安装、启停、更新和安全卸载用户级 Agent Skill。

## Background

- 仓库目前只有 Trellis 骨架，没有既有应用代码或技术栈。
- MVP 采用 Agent Skills 开放规范中的“目录 + `SKILL.md`”模型。
- Codex 与 Claude Code 的用户级目录、启停配置和专属元数据不同，必须分别适配。
- 官方技术约定与跨平台结论记录在 `research.md`。

## In Scope

### Skill compatibility

- 按 Agent Skills 开放规范校验 `name`、`description` 和目录名；保留未知字段、Agent 专属字段及整个 Skill 目录内容。
- 一个 Managed Skill Package 是稳定的用户实体，其下可有 Codex、Claude Code 两个 Installation；更新只改变 Installed Revision，不改变 Managed Skill Package 身份。
- Git Skill Source 由 repository URL、Skill subpath 和 tracked branch 稳定标识；Git commit OID 只属于 Installed Revision。
- 同一 Managed Skill Package 的所有 Installation 必须对应同一个 Installed Revision；启用状态由各 Installation 独立持有。
- 同名且内容指纹一致的 External Installation 可聚合提示，但只有用户明确确认后才同时接管；同名异内容不自动合并。
- Managed Library 中规范化 Skill 名称全局唯一；不同 Source 的同名 package 不能并存。
- 扫描 Codex 用户级 `$HOME/.agents/skills` 与 Claude Code personal `~/.claude/skills`。
- 仅管理用户级全局 Skill，不扫描或修改项目、系统、企业或插件作用域。

### Management operations

- 自动发现两个目标 Agent 中已有的 Skill，并显示来源、安装目标、启用状态和所有权状态。
- 从本地文件夹或公开 HTTPS Git 仓库导入一个有效 Skill；本地导入是快照，不持续监听或同步原目录。
- Git 仓库可由用户选择根目录或其中一个 Skill 子目录。
- 多 Skill Git 仓库一次只导入一个 Skill Package，不提供批量事务。
- Import 默认不选择 Agent Target；用户可显式选择一个、两个，或以零目标 Add to Library。
- 将 Skill 安装到 Codex、Claude Code，或同时安装到两者。
- 默认使用 Linked Installation：macOS/Linux relative directory symlink，Windows absolute directory junction；失败时显示原因，只有用户确认后才使用 Copy Fallback。
- 按各 Agent 的原生配置方式启用或停用 Skill。
- 主动检查 tracked branch 的远端 HEAD；只有 installed commit 是远端 HEAD 祖先时才作为普通 fast-forward 更新，并由用户确认执行。
- Tag 与 Skill metadata version 仅展示，不参与更新决策。
- Local Skill Source 只允许以新本地快照替换 Installed Revision；MVP 不允许切换 Source 类型或改变 Git repository URL、subpath、tracked branch。
- 更新候选改变 `SKILL.md.name` 或目录名时视为不兼容更新，必须作为新 Managed Skill Package 导入。
- 安全卸载由管理器安装或已接管的 Skill。
- 未检测到 Agent 客户端时仍允许用户确认创建其官方用户级 Skill 根目录并安装，界面明确提示客户端未检测到。
- 新 Installation 在完成验证、能力披露和最终确认后默认启用；Externally Controlled Configuration 仍优先。

### Ownership and data safety

- 管理器安装的 Skill 可直接管理；启动前已存在的 Skill 默认只读，必须经用户明确确认“接管”后才可修改。
- 同名目标目录发生冲突时阻止操作并显示冲突路径，绝不自动覆盖。
- 写操作前检测内容漂移；发现外部修改时阻止覆盖并提示用户。
- Content Drift 只可通过显式 Restore Installation 或 Detach Installation 解除；不自动 merge，也不让单个漂移副本成为 canonical revision。
- 导入和更新先在 staging 中完成下载、解析与校验，再替换目标；失败时保留或恢复原内容。
- 管理器不执行 Skill 中的脚本。
- 提供三层信息：Structural Validation 判断是否可安装；Capability Disclosure 展示脚本、声明工具、引用和未知字段；Change Disclosure 展示更新前后新增或移除的能力。
- 不提供 Safe/Unsafe 结论、风险评分、恶意代码检测或“扫描通过”徽章。
- Uninstall 只移除指定 Agent 的 Installation；零 Installation 的 Managed Skill Package 仍保留在 library，用户可另行 Remove from Library。
- Remove from Library 仅在零 Installation 时可用；其永久删除语义与确认流程见本节末尾的完整约束。
- Source Diverged 时停止普通更新，只允许保留当前版本或手工移除后重新导入；不提供强制更新、merge 或 rebase。
- 来源异常明确区分 Source Unreachable、Source Missing 和 Source Diverged，且都不改变当前 Installed Revision 或 Installation。
- 任一 Installation 发生 Content Drift 时，整个 Managed Skill Package 暂停更新，直到 Restore 或 Detach。
- Capability Disclosure 只报告结构化确定事实，不从 Markdown 或脚本正文推断网络、删除等行为。
- Skill Deck 不拥有其未创建的 Agent 配置状态；Adoption 保留 Configuration Provenance。
- Enable/Disable 只修改 Skill Deck 自己创建的配置；Detach 保留当前配置，Uninstall 不删除用户或第三方配置。
- 存在 Externally Controlled Configuration 时显示真实状态和配置路径，但锁定 Enable/Disable；用户自行移除外部配置后，Skill Deck 才能创建自己的状态。
- Skill Deck 创建的配置被外部修改时标记 Configuration Drift；仅允许用户显式 Reapply Configuration 或 Forget Configuration，不静默抢回控制。
- 状态损坏时先恢复最后一个有效备份；无法恢复则进入 Read-only Recovery，不从目录或指纹自动推断所有权。
- 启动与 inventory 全程离线；只有用户主动检查 Git 更新时联网，不做启动时、后台或定时 fetch。
- MVP 不包含遥测、分析 SDK 或自动 crash upload；诊断日志仅保存在本机并由用户主动导出。
- 所有 Skill Source 都是不可信输入，进入 Managed Library 前必须通过可解释、可展示、可测试的 Resource Boundary；任一超限会中止并清理整个 import transaction。
- Resource Boundary 只限制资源消耗，不表示恶意性扫描或安全结论。
- MVP Resource Boundary 固定且不可配置：Git transfer 250 MiB、checkout repository 500 MiB、Skill Package 100 MiB、10,000 files、single file 50 MiB、`SKILL.md` 1 MiB。
- 每个 Managed Skill Package 保留当前 Installed Revision 和一个 Previous Revision，并提供显式 Roll Back Revision；不保存完整历史。
- Roll Back Revision 交换 current 与 previous，因此允许单步 redo；Git Skill Source 不变。
- 卸载 Skill Deck 应用本体绝不删除 Agent Installation 或启停配置，并按操作系统默认保留 Managed Library 与状态；若 library 被另行删除，Copied Installation 降级为 External Installation，linked entry 则成为 Broken External Installation。
- 每个 Installation 记录 Deployment Mode、logical path、resolved target 与创建 provenance。
- `npx skills add` 等第三方 topology 只作为相关 External Installation 展示；lock 只作来源提示，不证明所有权或 Git Skill Source。
- Managed Library 位于 Skill Deck 私有 app-data；不复用 `$HOME/.agents/skills` 作为内部存储，也绝不写入第三方 `.skill-lock.json`。
- Adoption 第三方链接时先将解析内容校验并复制到 Managed Library，再原子切换选中的 Agent 入口；入口之外的 external target 绝不删除。
- 第三方链接 Adoption 后建立 Local Skill Source snapshot；只有重新通过公开 HTTPS URL 导入才能建立 Git Skill Source。
- Skill Package 内部 symlink/junction 始终拒绝；受支持的链接仅限 Agent Installation 入口指向 Managed Library。
- Linked Installation 执行 Detach 时先原子转换为 standalone copy，成功后才撤销管理记录。
- 清除 library 前必须先将 Linked Installation 转为 standalone copy 或明确删除；外部直接删除 library 会产生 Broken External Installation。
- broken、cyclic 或指向不安全位置的 external link 只读展示，不可 Adoption、Restore、Detach、Uninstall 或应用内清理。
- Codex legacy `$CODEX_HOME/skills` 扫描为只读 Legacy External Installation；迁移流程复制校验到 Managed Library，并以当前 `$HOME/.agents/skills` 为最终目标，不自动删除 legacy 目录。
- Legacy Migration 只完成 Add to Library；legacy entry 仍存在时禁止创建 current-root Installation，用户在应用外移除 legacy 后才能继续安装。
- Claude Personal Root 支持检测 `CLAUDE_CONFIG_DIR` 及受约束的显式覆盖；Codex State Root 支持检测 `CODEX_HOME` 及显式覆盖，但 Codex Skill root 固定由 OS home API 解析为 `$HOME/.agents/skills`。
- Settings 展示检测值、覆盖值和最终绝对路径；变更后重新执行 inventory。
- 所有成功写操作使用统一提示：“变更已保存；若 Agent 未反映，请重启”，不推断或区分各 Agent 的 live-reload 状态。
- Windows native Skill Deck 不扫描或管理 WSL 文件系统；WSL 视为独立 Linux runtime。
- Import/Update 的 staging 阶段允许取消并完整清理；进入 commit/rollback 阶段后暂时不可取消。
- Remove from Library 是永久动作：显示 Source、current/previous revision 与空间占用；Local snapshot 强调最后副本风险，提供当前 revision 的目录导出，并要求输入 package name 确认。

### Distribution

- Windows 10/11 x64：单一安装包。
- macOS 12+：Intel 与 Apple Silicon 通用包。
- Linux x86_64：在固定兼容基线上构建 AppImage。
- UI 提供简体中文与 English，默认跟随系统并允许手动切换；其他 locale 延后。

## Out of Scope

- 用户账号、云同步、自建市场、发布审核后台和内置 Skill 编辑器。
- 项目级、系统级、企业级或 Agent 插件作用域的 Skill 管理。
- 私有 HTTPS 仓库、SSH、SSO、凭据管理、Git LFS、submodule 和本地修改自动合并。
- 后台自动更新、依赖解析、Skill 脚本执行或安全沙箱。
- Windows ARM、Linux ARM、macOS 12 之前版本及 AppImage 之外的 Linux 安装包。
- Windows 版跨 `\\wsl$` 管理 WSL Agent runtime。
- Codex/Claude Code 之外的 Agent 私有插件格式。

## Acceptance Criteria

- [ ] Windows 10/11 x64、macOS 12+ Intel/Apple Silicon 和经验证的 x86_64 Linux 环境可安装并启动对应构建产物。
- [ ] 应用能发现两个官方用户级目录中的有效 Skill，并清楚区分 Agent、启用状态和只读/已接管状态。
- [ ] 同一 Managed Skill Package 的多个 Installation 聚合展示；同名同内容 External Installation 仅建议聚合接管，同名异内容保持分离并标示冲突。
- [ ] Managed Library 拒绝第二个同名 package，并显示已有实体；不同 Source 不能绕过唯一性。
- [ ] 有效本地目录和公开 HTTPS Git 来源可被导入；无效 frontmatter、目录名、危险路径或不支持的 Git 特性会被拒绝且不改变现有文件。
- [ ] 用户可选择 Codex、Claude Code 或两者作为安装目标；默认创建平台适当的 Linked Installation，失败时不会静默 Copy Fallback，任一目标冲突时不会覆盖已有目录。
- [ ] 多 Skill repository 一次只确认并导入一个子目录；Agent 客户端未检测到时可经明确确认创建官方用户级根目录。
- [ ] Import 默认零目标，可 Add to Library；只有用户明确选择的 Agent Target 才创建 Installation。
- [ ] 新 Installation 默认启用，除非有效的 Externally Controlled Configuration 决定其他状态。
- [ ] 启停操作使用对应 Agent 的配置机制，并保留配置文件中不属于本应用的内容。
- [ ] 未接管的既有 Skill 不能被启停、更新或卸载；接管必须经过明确确认。
- [ ] Content Drift 会阻止破坏性操作，并可通过 Restore Installation 或 Detach Installation 解除；两种动作都不自动 merge。
- [ ] Git 更新按 tracked branch 与 commit ancestry 检查；非 fast-forward 变化不显示为普通更新。
- [ ] 多 Installation 更新是单一 Installed Revision 的全有或全无事务；任一 Content Drift 会暂停 package 更新。
- [ ] 更新候选改变 Skill 名称时被拒绝，不隐式迁移安装路径或 Agent 配置。
- [ ] 更新确认界面展示确定性的文件和能力变化，但不作安全结论。
- [ ] 卸载只删除管理器拥有的安装副本，并清理由本应用写入的启停配置，不影响其他 Skill。
- [ ] Managed Skill Package 可在零 Installation 状态保留；Remove from Library 与 Uninstall 是两个独立动作。
- [ ] Source Diverged 不会被当作普通更新，也不存在强制接受新历史的快捷操作。
- [ ] Source Unreachable、Source Missing 与 Source Diverged 可被用户区分，且不会改变已安装内容。
- [ ] Adoption、Enable/Disable、Detach 和 Uninstall 均保留 Configuration Provenance，不删除 Skill Deck 未创建的配置。
- [ ] Externally Controlled Configuration 不可在应用内切换；Configuration Drift 必须通过 Reapply 或 Forget 显式解决。
- [ ] 状态无法恢复时应用进入 Read-only Recovery；External Installation 和 Orphaned Package 不会被自动认定为已管理。
- [ ] 未经用户动作不会发生网络请求；Git 检查只由用户显式触发。
- [ ] 构建产物不包含遥测/分析 SDK 或自动 crash upload；日志导出必须由用户主动触发。
- [ ] 任一 Resource Boundary 超限会在写入 Managed Library 前终止完整事务，界面显示触发的限制与观测值。
- [ ] 固定 Resource Boundary 的六项阈值都有边界值自动化测试，且用户不能在 MVP 中绕过。
- [ ] 更新后可在 current 与唯一 Previous Revision 间交换；所有 Installation 保持同 revision。
- [ ] 卸载应用本体不会移除任何 Agent Installation 或修改其配置；状态缺失后的重装不推断管理权。
- [ ] 第三方 link topology 可被发现但保持 External；Adoption 不原地管理或删除未知 target，且不把第三方 lock 当作所有权证明。
- [ ] Managed Library 使用私有 app-data，应用不会创建或修改第三方 lock 文件。
- [ ] Detach Linked Installation 会在同一路径保留 standalone content；转换失败时原 linked managed 状态不变。
- [ ] Package 内部链接和 Broken External Installation 均被拒绝管理，并提供可解释诊断。
- [ ] Legacy External Installation 可迁移到 Managed Library，但 legacy 目录不会被自动删除。
- [ ] Legacy entry 存在时 current-root install 被阻止，避免 Codex 重复加载同名 Skill。
- [ ] Claude/Codex 路径覆盖只影响官方允许的 personal/state root；最终解析路径可见，Codex user Skill root 不可任意改写。
- [ ] 写操作后显示统一 restart fallback 提示；Windows native 不探测 WSL roots。
- [ ] 用户取消 staging 会清理全部临时内容；事务提交开始后取消不可用且最终状态保持全有或全无。
- [ ] Remove from Library 仅对零 Installation package 开放，并在导出选项、风险披露和名称确认后永久删除 current/previous/Source。
- [ ] 中英文界面可切换，首次启动跟随系统；所有用户可见核心流程不存在硬编码单语文案。
- [ ] 核心文件与配置操作有自动化测试覆盖成功、冲突、无效输入、外部漂移和回滚路径。

## Risks and Deferred Items

- Agent 官方目录和配置约定可能变化，发布前必须按当期官方文档和真实客户端回归。
- AppImage 兼容范围由构建基线、glibc 与 WebKitGTK 决定，不能宣称支持所有 Linux 发行版。
- macOS 公分发需要签名与 notarization；Windows 签名也需要外部证书，代码可预留 CI 接入但凭据配置不属于 MVP。
