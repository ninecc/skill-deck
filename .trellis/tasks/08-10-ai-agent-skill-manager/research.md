# AI Agent Skill 管理器：技术研究

> 查阅日期：2026-08-10。仅引用官方文档、官方仓库与开放规范；带“建议/推论”的内容不是厂商承诺。

## 结论摘要

- **Tauri 2 适合当前 MVP 发行矩阵**：可生成 Windows MSI/NSIS、macOS Universal App 与 Linux x86_64 AppImage。真正的约束在各平台原生构建、签名和 Linux glibc/WebKitGTK 基线，而不在 GUI 能力。
- **两端应采用独立适配器**。Codex 当前官方用户级目录是 `$HOME/.agents/skills`；Claude Code 是 `~/.claude/skills`。不要继续把 `~/.codex/skills` 当 Codex 的官方安装目标。
- **可移植 Skill 校验采用 Agent Skills 开放规范的严格交集**；vendor 专属字段保留但不跨端解释。
- **Git 最小可靠方案建议内嵌 `git2`/libgit2，仅支持公开 HTTPS 仓库**。这样不依赖用户安装 Git，也避开桌面 GUI 的 `PATH` 问题；私有仓库、SSH、LFS、submodule 延后。

## 1. Codex、Claude Code 与 Agent Skills 格式

### 1.1 开放格式的共同基线

[Agent Skills 规范](https://agentskills.io/specification)规定：Skill 是至少含一个 `SKILL.md` 的目录；`SKILL.md` 是 YAML frontmatter 加 Markdown 正文。`name` 与 `description` 必需：

- `name` 长度 1–64，只允许小写 ASCII 字母、数字和连字符，不得以连字符开头/结尾、不得连续使用连字符，并须与父目录名一致。
- `description` 长度 1–1024，应说明做什么以及何时使用。
- 标准可选字段是 `license`、`compatibility`、`metadata`、`allowed-tools`；其中 `allowed-tools` 仍标为 experimental。
- `scripts/`、`references/`、`assets/` 是常见可选目录；标准没有限制额外文件和目录。

因此管理器可把“目录 + `SKILL.md`”作为统一存储单位，导入时按开放规范严格校验 `name`/`description`，复制时保留整个目录及未知字段。Claude Code 接受的宽松输入不应被等同为跨产品兼容。

### 1.2 Codex 用户级 Skill

OpenAI 当前文档的用户级目录是 **`$HOME/.agents/skills`**；同页还列出 repo 级 `.agents/skills`、admin 级 `/etc/codex/skills` 与 Codex 内置 system skills。Codex 会跟随 Skill 目录的符号链接，同名 Skill 不合并，可能同时出现在选择器中。[OpenAI：Build skills / Where Codex loads local skills](https://developers.openai.com/codex/skills/)

Codex 要求 `SKILL.md` 有 `name` 与 `description`，支持 `scripts/`、`references/`、`assets/`；`agents/openai.yaml` 是可选的 OpenAI 专属 UI、调用策略和工具依赖元数据，不属于可移植基线。Codex 可以按 `description` 隐式选择 Skill，但文档没有承诺每次匹配都触发。[OpenAI：Build skills](https://developers.openai.com/codex/skills/)

Codex 正式停用方式不是移动/改名目录，而是在 `~/.codex/config.toml` 写入按 `SKILL.md` 绝对路径匹配的配置，随后重启：

```toml
[[skills.config]]
path = "/path/to/skill/SKILL.md"
enabled = false
```

来源：[OpenAI：Enable or disable local Codex skills](https://developers.openai.com/codex/skills/)。

### 1.3 Claude Code 用户级 Skill

Claude Code 的 personal Skill 位于 **`~/.claude/skills/<skill-name>/SKILL.md`**，适用于该用户的所有项目；另有 project、enterprise 和 plugin scope。同名优先级为 enterprise > personal > project；plugin Skill 使用 `plugin-name:skill-name` 命名空间。[Anthropic：Extend Claude with skills](https://code.claude.com/docs/en/skills)

Claude Code 会监听 personal/project Skill 变化并在当前 session 中生效；但若启动时顶层 skills 目录不存在、之后才新建，需重启 Claude Code。Claude 也提供 `skillOverrides` 的 `on`、`name-only`、`user-invocable-only`、`off` 状态；plugin Skill 不受该配置控制。[Anthropic：Live change detection 与 Override skill visibility](https://code.claude.com/docs/en/skills)

Claude Code 自身的 frontmatter 比开放规范宽松：所有字段都可省略，`description` 仅 recommended；缺少 `name` 时使用目录名，缺少 `description` 时使用 Markdown 首段。它还支持 `disable-model-invocation`、`user-invocable`、`context`、动态 shell 等专属字段。[Anthropic：Frontmatter reference](https://code.claude.com/docs/en/skills)

### 1.4 适配边界

MVP 只扫描和写入以上两个 user/personal direct-skill 根目录。Repo/project、admin/system/enterprise 和 plugin scope 都应排除；它们有不同所有权、优先级和生命周期，不能作为“可接管的用户 Skill”。OpenAI 也明确区分本地 direct skill 与用于分发的 plugin。[OpenAI：Distribute skills with plugins](https://developers.openai.com/codex/skills/)、[Anthropic：Plugins reference](https://code.claude.com/docs/en/plugins-reference)

路径中的 `$HOME`/`~` 应通过操作系统的用户 home API 解析后再拼接，不能硬编码 `/Users`、`/home` 或 `%USERPROFILE%`。这是跨平台实现推论；Codex 文档只给出 `$HOME/.agents/skills`，没有承诺 Windows 上某个字面环境变量或物理绝对路径。

## 2. Tauri 2 跨平台适配结论

### 2.1 发行矩阵

| 目标 | 官方能力与约束 | 结论 |
|---|---|---|
| Windows 10/11 x64 | Tauri 可生成 WiX `.msi` 或 NSIS `-setup.exe`；MSI 只能在 Windows 上生成。Windows 使用 Edge WebView2，Windows 10 1803+ 与 Windows 11 随系统分发该 runtime；安装器仍可选择在线 bootstrapper 或约 127 MB 的 offline installer。[Windows Installer](https://v2.tauri.app/distribute/windows-installer/)、[Prerequisites](https://v2.tauri.app/start/prerequisites/) | 适合。CI 用 Windows x64 runner；MVP 在 MSI 与 NSIS 中选一种即可，不必同时维护。 |
| macOS 12+ Intel / Apple Silicon | `--target universal-apple-darwin` 生成同时支持 Intel 和 Apple Silicon 的 Universal App；`bundle.macOS.minimumSystemVersion` 可设为 `12.0`。macOS 打包需要在 Mac 上运行；直接分发需签名与 notarization。[Universal build](https://v2.tauri.app/distribute/app-store/)、[macOS bundle](https://v2.tauri.app/distribute/macos-application-bundle/)、[Distribute](https://v2.tauri.app/distribute/) | 适合。CI 用 macOS runner，产出 Universal 包并明确最低系统 12.0。 |
| Linux x86_64 AppImage | Tauri 直接支持 AppImage，但必须在“计划支持的最老基础系统”构建；Tauri 2 要求 WebKitGTK 4.1。官方给出的合适基线示例是 Ubuntu 22.04 或 Debian 12；在更新系统构建会抬高 glibc 要求。[AppImage limitations](https://v2.tauri.app/distribute/appimage/)、[Linux prerequisites](https://v2.tauri.app/start/prerequisites/) | 适合，但“主流 Linux”不能只靠格式宣称。固定 Ubuntu 22.04（或同等已验证基线）构建并列出实测发行版。 |

额外约束：macOS 与 Linux GUI 应用不会继承 shell dotfiles 中的 `$PATH`。如果应用调用外部 `git`，即便用户终端能找到 Git，桌面进程也可能找不到。[Tauri AppImage note](https://v2.tauri.app/distribute/appimage/)

### 2.2 判断

Tauri 2 能覆盖选定矩阵，且 Rust 后端适合文件系统、原子替换和 Git 操作。建议保留三套原生 CI job，不做跨平台交叉打包：Windows 在 Windows 构建，macOS 在 Mac 构建，Linux 在固定旧基线容器/runner 构建。官方对 Windows NSIS 交叉编译也明确提示 caveats、测试较少，应作为 last resort。[Tauri Windows cross-compilation](https://v2.tauri.app/distribute/windows-installer/)

## 3. Git URL 导入与更新

### 3.1 选择

MVP 建议使用 Rust `git2`（libgit2 binding），而不是调用系统 Git CLI：

- libgit2 官方说明其为可嵌入、跨平台的 Git 核心实现，Linux/macOS/Windows 均受测试和支持。[libgit2](https://libgit2.org/)
- `git2-rs` 会随 `libgit2-sys` 带入 libgit2 源码，无需终端用户预装 libgit2；远程仓库需显式启用 `https`/`ssh` feature。[git2-rs](https://docs.rs/crate/git2/latest)
- 系统 Git CLI 的 `clone`、`fetch`、`pull --ff-only` 语义成熟，但不是三端系统组件；加上 GUI `$PATH` 问题，会把“Git 已安装且可发现”变成额外产品前置条件。[git clone](https://git-scm.com/docs/git-clone)、[git fetch](https://git-scm.com/docs/git-fetch)、[git pull --ff-only](https://git-scm.com/docs/git-pull)

这里选择 libgit2 是为了减少终端用户依赖，不代表其认证更简单。libgit2 对 HTTPS/SSH 认证要求应用提供 credential callback 并处理交互循环。[libgit2 authentication guide](https://libgit2.org/docs/guides/authentication/)

### 3.2 最小可靠流程

1. MVP 只接受公开 `https://` Git URL；clone 到管理器 app-data 下的 staging 目录，不直接 clone 到 Agent 目录。
2. 用户选择仓库中的一个 Skill 目录（仓库根本身也可），按开放规范验证；记录 canonical URL、相对子路径、远端默认分支和已安装 commit OID。来源与版本信息放在管理器自己的状态库，不改写第三方 `SKILL.md`。
3. “检查更新”只执行 fetch 并比较当前记录 OID 与远端目标 OID；不要用日期、tag 名或 `metadata.version` 猜版本。Git 官方定义 fetch 会下载 objects/refs 并更新 remote-tracking branches。[git fetch](https://git-scm.com/docs/git-fetch)
4. “执行更新”在新的 staging 目录 checkout 目标 OID、重新校验，再替换管理器拥有的安装副本；替换失败恢复旧副本。不要对目标目录执行 merge、rebase 或未经确认的 hard reset。
5. 只有管理器安装或明确接管且当前内容未被外部修改的 Skill 才能更新。内容已变化时先报告冲突，不覆盖。

该方案故意不做 shallow/sparse 优化；对 Skill 仓库而言，正确性和可恢复性优先。仓库体积被实际测量为问题后，再加入 shallow clone 或下载策略。

### 3.3 MVP 明确排除

- 私有 HTTPS、SSH key/agent、credential helper、SSO。
- Git submodule、Git LFS、单仓库多 Skill 的自动猜测。
- tag/semver 更新策略、自动后台更新、跨来源依赖解析。
- 本地修改的自动 merge/rebase。

加入私有仓库时再设计系统凭据存储、credential callback、host key/certificate UX；不能把 token 写入 URL 或 `SKILL.md`。

## 4. 官方没有保证的内容

- Agent Skills 规范没有定义 Git source、版本、依赖、安装所有权、接管、更新、卸载、事务/回滚、签名或冲突解决协议；`metadata.version` 只是任意 metadata，不是标准升级协议。[Agent Skills specification](https://agentskills.io/specification)
- 开放规范没有定义“启用/停用”语义。Codex 与 Claude Code 的关闭机制是 vendor 配置，管理器必须分别适配，不能靠移动目录假装通用协议。
- Vendor 专属 frontmatter、`agents/openai.yaml`、Claude 动态 shell/子代理字段在另一 Agent 上的含义与行为没有兼容保证。
- “符合 `SKILL.md` 格式”不等于安全、可信或可在当前机器执行；scripts 语言、外部命令、环境变量和网络依赖均由具体 Skill/客户端决定。
- Tauri 的 AppImage 支持不等于所有 Linux 发行版都兼容；最终下限由构建机 glibc、WebKitGTK 与实际测试矩阵决定。
- 官方没有保证 Codex/Claude 的目录与解析规则永久不变；适配路径应集中配置，并在发布时用当期官方文档和真实客户端回归测试。

## 5. `npx skills add` 的链接安装模型

> 核实日期：2026-08-10。以下“已验证事实”来自 `skills.sh` 官方文档、npm 官方包元数据以及 `vercel-labs/skills` 官方源码；“对 Skill Deck 的建议”是本项目设计判断，不代表该 CLI 的承诺。

### 5.1 已验证事实

#### 命令与包名

- 官方命令是 **`npx skills add <source>`**，不是 `npx skill add`。npm 包名与主可执行文件名都是 `skills`；包还提供兼容 bin 名 `add-skill`。来源：[skills.sh CLI 文档](https://www.skills.sh/docs/cli)、[`package.json`](https://github.com/vercel-labs/skills/blob/main/package.json#L1-L8)。
- 官方 README 把 `--copy` 定义为“复制文件而不是链接到 Agent 目录”；交互式安装提供 `Symlink (Recommended)` 与 `Copy` 两种方式。[官方 README：Options / Installation Methods](https://github.com/vercel-labs/skills#options)

#### 中央存储与目标路径

- 默认链接模式不是直接链接回 Git checkout 或用户传入的本地目录。实现会先把 Skill 内容复制到 canonical 目录，再从 Agent 目录创建链接；项目级 canonical 根为 `./.agents/skills`，全局 canonical 根为 `~/.agents/skills`。[`getCanonicalSkillsDir` 与安装流程](https://github.com/vercel-labs/skills/blob/main/src/installer.ts#L90-L93)、[链接模式实现](https://github.com/vercel-labs/skills/blob/main/src/installer.ts#L267-L380)
- Claude Code 的声明目标是项目级 `.claude/skills`、全局 `${CLAUDE_CONFIG_DIR:-~/.claude}/skills`。Codex 的声明目标是项目级 `.agents/skills`、全局 `${CODEX_HOME:-~/.codex}/skills`。[Agent registry](https://github.com/vercel-labs/skills/blob/main/src/agents.ts#L8-L9)、[Claude/Codex entries](https://github.com/vercel-labs/skills/blob/main/src/agents.ts#L136-L144)
- 但当前实现把 `skillsDir === '.agents/skills'` 的 Agent 定义为 universal；Codex 因而是 universal。`getAgentBaseDir` 对 universal Agent 直接返回 canonical 目录。因此在当前实现的链接模式中，Codex 实际直接读取 canonical `~/.agents/skills/<name>`，不会再创建 `~/.codex/skills/<name>` 链接；Claude Code 则从 `~/.claude/skills/<name>` 链接到 canonical 副本。[universal 判定](https://github.com/vercel-labs/skills/blob/main/src/agents.ts#L835-L849)、[`getAgentBaseDir`](https://github.com/vercel-labs/skills/blob/main/src/installer.ts#L103-L137)、[全局 universal 分支](https://github.com/vercel-labs/skills/blob/main/src/installer.ts#L331-L343)
- `--copy` 模式跳过 canonical-to-agent 链接步骤，直接清空并复制到各 Agent 的解析目录。只有一个唯一目标目录时，CLI 自动选择 copy，因为没有链接带来的复用价值。[copy 实现](https://github.com/vercel-labs/skills/blob/main/src/installer.ts#L310-L320)、[模式选择](https://github.com/vercel-labs/skills/blob/main/src/add.ts#L1452-L1485)

#### Windows、macOS、Linux 的链接与退化行为

- macOS/Linux 使用相对目录 symlink；Windows 使用指向绝对 canonical 路径的 directory junction。实现通过 Node `symlink(..., 'junction')` 创建 Windows junction。[`createSymlink`](https://github.com/vercel-labs/skills/blob/main/src/installer.ts#L178-L241)
- 创建链接的任意失败都会返回 `false`，随后安装器保留 canonical 副本，并把内容直接复制到该 Agent 目录；结果标记 `symlinkFailed: true`，CLI 对用户显示为 `copied`。因此 fallback 是逐 Agent 的 copy，不是整次安装失败。[fallback 实现](https://github.com/vercel-labs/skills/blob/main/src/installer.ts#L361-L380)、[结果展示](https://github.com/vercel-labs/skills/blob/main/src/add.ts#L339-L369)
- 复制 Skill 内容时，源码会 dereference Skill 内部的 symlink；无法解析的 broken symlink 会被跳过并警告。因此“Agent 安装目录是链接”与“Skill 包内部含链接”是两个独立问题。[`copyDirectory`](https://github.com/vercel-labs/skills/blob/main/src/installer.ts#L428-L473)

#### 发现、所有权、更新和卸载

- 列表扫描同时遍历 canonical 与各 Agent 目录；目录 symlink 会被跟随并按 Skill 目录处理。但公开的 `InstalledSkill` 记录只有 `path`、`canonicalPath`、scope 与 agents，不公开 link kind、link target 或 fallback-copy 状态；扫描结果也不会仅凭链接声明所有权。[`InstalledSkill` 与扫描流程](https://github.com/vercel-labs/skills/blob/main/src/installer.ts#L989-L1047)、[symlink 目录扫描](https://github.com/vercel-labs/skills/blob/main/src/installer.ts#L1086-L1128)
- 全局来源记录位于 `$XDG_STATE_HOME/skills/.skill-lock.json` 或默认的 `~/.agents/.skill-lock.json`。v3 条目记录 source、source type、source URL、ref、skill path、folder hash 与时间戳；**不记录安装到哪些 Agent，也不记录 symlink/copy 模式或具体 link target**。[`SkillLockEntry` 与路径](https://github.com/vercel-labs/skills/blob/main/src/skill-lock.ts#L5-L68)
- 当前 update 通过比较 folder hash 找到变化，然后重新调用本 CLI 的 `add ... -g -y`。由于全局 lock 不保存 Agent 集合或安装模式，该数据结构本身不足以精确重建原有链接拓扑。[更新重新调用 add](https://github.com/vercel-labs/skills/blob/main/src/update.ts#L619-L655)、[lock schema](https://github.com/vercel-labs/skills/blob/main/src/skill-lock.ts#L11-L56)
- remove 先清理目标 Agent 路径；只有没有其余已检测 Agent 使用 Skill 时，才删除 canonical 目录和 lock entry。这说明 canonical 内容与 Agent exposure 是两个生命周期，但其“仍被使用”判断依赖当前磁盘/Agent 检测结果。[remove 实现](https://github.com/vercel-labs/skills/blob/main/src/remove.ts#L200-L280)

### 5.2 对 Skill Deck 的设计建议

- **MVP 应支持 app-owned link installation。** Managed Library 保存唯一 Installed Revision；每个 Installation 明确记录 `deploymentMode = symlink | junction | copy-fallback`、Agent、logical path、resolved target 与创建 provenance。不要照搬 `skills` 全局 lock 的信息缺口。
- **三端采用同一语义、不同原语。** macOS/Linux 创建目录 symlink，Windows 创建 directory junction；链接创建失败时可在用户确认的预览中退化为 app-owned copy。fallback Installation 仍须参加 content drift 校验，并保持同一 package 的 revision 一致性。
- **发现所有链接，但不要把链接等同为所有权。** 对 symlink/junction 读取 logical path、resolved target、broken/loop/outside-library 状态；只有 state manifest 与 app-owned target 同时匹配才是 Managed Installation。`~/.agents/.skill-lock.json` 可作为来源提示，不能单独作为 Skill Deck 的 ownership 证明。
- **外部链接默认只读，但不再一律“不可接管”。** Adoption 应预览并复制/解引用 target 到 Managed Library，然后仅用 app-owned link 替换 Agent 入口；不得在原 external target 上原地更新，也不得在 Detach/Uninstall 时删除 external target。
- **更新只替换 app-owned revision，不穿透链接写入未知目标。** 所有 Agent link 应解析到同一个 app-owned current revision；更新、回滚先在 staging 完成，再切换 canonical 内容。copy-fallback Installation 在同一事务内同步，任一漂移或写入失败则整体停止/回滚。
- **卸载按 Installation 形态删除。** app-owned symlink/junction 只删除 link；copy-fallback 只删除经 provenance 验证的 app-owned 副本；`Remove from Library` 仍须等到零 Installation。broken link 也应保留 link target 文本用于可解释的修复或安全删除。

## 6. Agent 用户根目录、配置路径、热重载与 Windows 语义

> 核实日期：2026-08-10。本节只使用 OpenAI/Anthropic 官方文档与官方源码。先列“已验证事实”，再列官方没有承诺的边界；设计建议不是 Agent 客户端的产品保证。

### 6.1 已验证事实：Codex

#### 用户 Skill root 与 `CODEX_HOME` 是两套概念

- OpenAI 当前文档把 direct/local personal Skill 的 `USER` root 明确定义为 **`$HOME/.agents/skills`**。Repo root 是从 CWD 向 repository root 逐层扫描的 `.agents/skills`；另有 `/etc/codex/skills` 与 Codex bundled skills。[OpenAI：Where Codex loads local skills](https://developers.openai.com/codex/skills/)
- `CODEX_HOME` 默认是 `~/.codex`，是 **整个 Codex 本地状态根**，覆盖 config、auth、logs、sessions、skills 与 standalone package metadata；`config.toml`、`auth.json`、history、logs/caches 等都在该状态根下。设置后目录必须已存在。[OpenAI：Environment variables](https://learn.chatgpt.com/docs/config-file/environment-variables)、[OpenAI：Config and state locations](https://learn.chatgpt.com/docs/config-file/config-advanced#config-and-state-locations)
- 因此，设置 `CODEX_HOME=D:\codex-a` 会把 Codex 用户配置定位到 `D:\codex-a\config.toml`，而不是默认 `~/.codex/config.toml`；它不是“只改 Skill 目录”的开关。官方源码还会校验该路径存在且为目录，并将其 canonicalize。[`find_codex_home`](https://github.com/openai/codex/blob/main/codex-rs/utils/home-dir/src/lib.rs)
- 当前官方源码中的 `SkillsConfig` 只有 bundled 开关、是否注入 skills instructions，以及按 `path`/`name` 的 `enabled` 规则；**没有公开的任意 `userSkillRoot`/`skillsDir` 配置项**。[`SkillsConfig`](https://github.com/openai/codex/blob/main/codex-rs/config/src/skills_config.rs)
- Codex 仍为兼容性读取 `$CODEX_HOME/skills`；引入 `$HOME/.agents/skills` 的官方合并说明明确称前者是“for now until we fully deprecate”的 backwards compatibility 路径。它适合被管理器发现为既有/legacy 安装，但不应被当成长期稳定的跨 Agent authoring root。[OpenAI PR #10437](https://github.com/openai/codex/pull/10437)

#### 新增、更新与 enable/disable

- Codex 文档称 Skill 文件变化和新安装 Skill 会被自动检测；若没有显示，需重启 Codex。这是带 restart fallback 的行为，不是“永远无需重启”的强保证。[OpenAI：Build skills](https://developers.openai.com/codex/skills/)
- 本地 Skill 的 disable/enable 通过 Codex 用户 `config.toml` 中的 `[[skills.config]]` 完成，按 `SKILL.md` 绝对路径（或当前源码支持的 name selector）设置 `enabled`。官方明确要求修改配置后重启 Codex。[OpenAI：Enable or disable local Codex skills](https://developers.openai.com/codex/skills/)、[`SkillConfig`](https://github.com/openai/codex/blob/main/codex-rs/config/src/skills_config.rs)

### 6.2 已验证事实：Claude Code

#### `CLAUDE_CONFIG_DIR` 同时搬迁 personal Skill 与 settings

- 默认 personal Skill 是 `~/.claude/skills/<name>/SKILL.md`，personal settings 是 `~/.claude/settings.json`。Anthropic 明确说明：设置 `CLAUDE_CONFIG_DIR` 后，文档中的每一个 `~/.claude` 路径都改到该目录下；环境变量参考也说明 settings、session history、plugins，以及 Linux/Windows credentials 都存于该路径（macOS credentials 仍在 Keychain）。[Anthropic：Explore the .claude directory](https://code.claude.com/docs/en/claude-directory)、[Anthropic：`CLAUDE_CONFIG_DIR`](https://code.claude.com/docs/en/env-vars#variables)
- 所以 `CLAUDE_CONFIG_DIR=/data/claude-a` 时，personal Skill root 是 `/data/claude-a/skills`，personal settings 是 `/data/claude-a/settings.json`。Project `.claude/skills`、`.claude/settings.json` 与 `.claude/settings.local.json` 仍是 project-relative，不随 personal config root 搬迁。
- Shell 环境变量在 Claude Code 启动时读取，之后修改只在下一次 launch 生效。因此改变 `CLAUDE_CONFIG_DIR` 本身应重启/重新启动 Claude Code，不能期待运行中的进程切换根目录。[Anthropic：Environment variable precedence](https://code.claude.com/docs/en/env-vars#precedence)

#### 新增、更新与 enable/disable

- Claude Code 监听已存在的 personal/project/`--add-dir` Skill 目录；新增、编辑、删除 Skill 在当前 session 生效，无需重启。例外是 session 启动时顶层 `skills` 目录不存在，之后才创建该目录，此时需要重启才能建立 watcher。[Anthropic：Live change detection](https://code.claude.com/docs/en/skills#live-change-detection)
- 热更新只覆盖 `SKILL.md` 文本。Skill folder 同时是 plugin 时，`hooks/`、`.mcp.json`、`agents/`、`output-styles/` 的变化要执行 `/reload-plugins`。[Anthropic：Live change detection](https://code.claude.com/docs/en/skills#live-change-detection)
- 已经 invoke 的 Skill 内容会作为消息留在当前 conversation；Claude Code 后续 turn 不会重新读取该文件。因此磁盘更新能刷新 discovery/后续 invocation，但不会追溯替换已经进入对话历史的旧内容。[Anthropic：Skill content lifecycle](https://code.claude.com/docs/en/skills#skill-content-lifecycle)
- Claude Code 用 `skillOverrides` 的 `on`、`name-only`、`user-invocable-only`、`off` 控制非 plugin Skill 的可见性；`/skills` 菜单会把选择写入 project-local `.claude/settings.local.json`。官方页面没有另行要求重启，但也没有明确承诺“外部程序直接编辑 `skillOverrides` 后必定即时生效”。[Anthropic：Override skill visibility from settings](https://code.claude.com/docs/en/skills#override-skill-visibility-from-settings)

### 6.3 Windows home/path 的官方语义

| 客户端 | 官方明确语义 | 管理器不可擅自假定 |
|---|---|---|
| Claude Code native Windows | `~/.claude` 明确解析为 `%USERPROFILE%\.claude`；若设置 `CLAUDE_CONFIG_DIR`，则以该目录替换所有 personal `~/.claude` 路径。[Anthropic：Explore the .claude directory](https://code.claude.com/docs/en/claude-directory) | 不要把 `%APPDATA%`、`%LOCALAPPDATA%` 当作 Claude personal config/Skill 默认根。 |
| Codex native Windows | Skill 文档只写 `$HOME/.agents/skills`；Codex 源码用 Rust `dirs::home_dir()` 获取 OS home。`CODEX_HOME` 是独立状态根，设置后按原生路径解析、要求已存在并 canonicalize。[OpenAI：Build skills](https://developers.openai.com/codex/skills/)、[`find_codex_home`](https://github.com/openai/codex/blob/main/codex-rs/utils/home-dir/src/lib.rs) | OpenAI 文档没有把 `$HOME` 的 Windows 物理路径承诺为某个字面环境变量；不要仅拼 `%USERPROFILE%`、`HOME` 或 `HOMEDRIVE`+`HOMEPATH`。应使用 OS home API，并把 `CODEX_HOME` 单独解析。 |
| WSL | 对两个 CLI 都是 Linux 进程/文件系统语义；Claude 文档把 WSL 与 macOS/Linux 放在同一 shell 设置方式下。[Anthropic：Set environment variables](https://code.claude.com/docs/en/env-vars#set-environment-variables) | 不要自动把 WSL `~` 与 Windows `%USERPROFILE%` 合并为同一个安装位置；它们可能是两个独立 Agent runtime。 |

### 6.4 官方未保证项与 Skill Deck 设计结论

- **Codex 没有只重定向 direct user Skill root 的公开设置。** `CODEX_HOME` 搬迁整套 Codex state；`$CODEX_HOME/skills` 还是明确带退场意图的兼容路径。MVP 应分别探测 OS home、`CODEX_HOME` 和 legacy tree，不能将三者折叠成一个字符串规则。
- **Codex 自动检测不是强一致 API。** 官方给出的恢复动作就是 restart；对 symlink/junction、不同 Codex surface（CLI、IDE、desktop app）的 watcher 时序与一致性没有稳定契约。Skill Deck 完成写入后可显示“Agent reload may be required”，而不是宣称 live reload 成功。
- **Claude 的 content hot reload 不等于 conversation hot replacement。** 已经注入上下文的 Skill 仍是旧快照；管理器只能说明文件/目录已更新，不能声称当前对话已经采用新内容。
- **Claude `skillOverrides` 的外部编辑 reload 时序未被明确承诺。** 管理器写入后应保留“若当前 session 未反映，请 restart”的兜底提示；不能把 `/skills` 菜单内行为外推成任意第三方写文件的稳定 API。
- **环境根目录是进程启动上下文。** 修改 `CODEX_HOME`/`CLAUDE_CONFIG_DIR` 后，已运行 Agent 是否重新绑定根目录没有保证；inventory 应记录“本次探测所用环境与解析后的绝对路径”，并在环境变化后重新发现。
