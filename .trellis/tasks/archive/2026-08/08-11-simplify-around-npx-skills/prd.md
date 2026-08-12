# Simplify Skill Deck around `npx skills`

## Goal

把 Skill Deck 从自建 Skill lifecycle manager 收缩为 `npx skills` 的桌面 GUI，并只补充 CLI 没有提供的已安装 Skill 预览与翻译体验，降低产品概念、持久状态和维护成本。

## Background and Confirmed Facts

- 当前实现自建了 Managed Library、package/installation/ownership、adoption、configuration、revision、drift reconciliation、rollback 和 Git update 等完整生命周期。
- 当前前端 DTO 同时暴露 managed/external package、installation reconciliation、configuration provenance 和多类 plan/commit 操作，首页需要协调多套状态模型。
- `skills` 官方 CLI 已提供 `add`、`list`、`find`、`remove`、`check`、`update` 和 `init`；其中 `update` 已是官方能力，不需要 Skill Deck 再实现 Git 更新器。
- 当前官方 `skills list -g --json` 提供包含 name、path、scope、agents 和 source 的机器可读结果，GUI 不需要解析终端展示文本。
- 产品决定每次应用启动时通过 npm 解析一次 `skills@latest`，随后在该会话内固定调用解析出的精确版本；下次启动再获取新的 latest。
- `skills add` 已负责 canonical copy、macOS/Linux symlink、Windows junction 与 copy fallback，并支持项目级/全局 scope 和多 Agent。
- 官方 CLI 是 Node CLI；2026-08-11 的 `skills@latest` 为 1.5.22，要求 Node.js >= 22.20.0。Skill Deck 当前成品尚未声明 Node/npm 为运行时前置条件。
- 产品决定调用用户本机的 Node/npm，不在应用内打包、下载或管理 Node runtime。
- 官方 lock/list 能支持 CLI 自己的更新流程，但历史研究确认它们不完整表达所有 Agent target 与 link/copy topology；若 GUI 只调用 CLI，就不应再维护第二套“更完整”的所有权模型。

## Requirements

### Product boundary

- Skill Deck 的核心管理能力以官方 `skills` CLI 的语义和结果为准，不再维护平行的 package manager。
- MVP 只管理 global Skills；所有 list/add/remove/update 命令显式使用 global scope，不维护项目目录或最近项目状态。
- CLI 行为选择集中放在 Settings，并建模为可选 override；未设置时不传对应 flag，使用 `skills@latest` 自己的默认行为。
- Agent target 默认使用 CLI 自动检测与默认选择；用户可在 Settings 中保存显式 Agent target override，后续安装才追加 `--agent` 参数。
- 产品面向 `skills@latest` 支持的全部 Agent，不再把 Agent 建模为 `codex|claude` 封闭枚举；列表展示 CLI JSON 返回的 Agent 名称。
- 自动 Agent 模式可立即继承 CLI 新增的 Agent；显式 override 选择器使用 Skill Deck 当前兼容清单并随应用更新，CLI 拒绝未知 ID 时原样返回结构化命令失败。
- Settings 不使用孤立的 “CLI default” 文案，而展示默认行为并在末尾标注 “(Default)”，例如 “Automatically detect installed agents (Default)” 和 “Automatically choose link or copy (Default)”。
- MVP 的 CLI overrides 只包含 Agent targets 与安装方式（automatic 或 copy）；`full-depth`、internal skills、owner filter 等低频 flags 不进入 Settings。
- Appearance 只保存一个主题值，不再把 mode 与 palette 拆成两个设置。确定为横排五选一：System (Default)、Light、Dark、Sand、Plum；每项使用多个代表性 token 色块展示完整主题，System tile 固定同时预览 Light/Dark 两半而不复制当前系统模式，选择跨重启持久化。
- System 在运行时跟随操作系统 Light/Dark 变化；Light 与 Dark 为固定 Graphite/Azure 映射，Sand 为暖色浅色基底与 amber/rust 交互色，Plum 为深 aubergine 基底与 lavender 交互色。MVP 不支持同一彩色主题再叠加独立明暗模式。
- 产品图标从 Iconify 选择一个一致的开源 collection，只打包实际使用的图标数据；桌面应用运行时不得依赖 Iconify API、CDN 或联网加载图标。
- GUI 覆盖已安装 Skill 列表、查找、安装、移除和执行更新。每个 Installed Skill 提供 Update，另提供需要确认的 Update All；不自动更新。
- Remove 删除该 global Installed Skill 在全部 Agent Targets 上的安装，执行前明确确认；MVP 不提供按 Agent 移除。
- `update` 直接封装官方 `skills update`；不自建更新可用状态、diff、Git ancestry、revision 或 rollback 系统。
- `preview` 和 `translate` 只作用于已安装 Skill，不创建第二份受管安装记录。
- `translate` 只生成供用户阅读的即时译文，不写入、导出或替换已安装 Skill，因此不影响 Agent 实际指令、官方 lock 或后续更新。
- 翻译能力是独立模块，通过窄的 provider contract 接收原文与目标语言并返回译文；MVP 不建设动态插件加载、provider marketplace 或通用 AI SDK。
- MVP 的首个 provider 匿名调用最简单的 Google Translate 非正式接口，不要求 Google Cloud 项目、API key、OAuth 或凭据存储。
- 该接口按 best-effort 能力处理：限流、协议变化或不可用只使翻译失败，不得阻塞 preview 或任何 Skill 管理操作；独立 provider 边界用于以后替换它。
- UI 直接使用 CLI 可表达的 scope、Agent 和来源概念；不保留 Skill Deck 独有的 ownership/adoption/configuration provenance 概念。
- 旧 Managed Library 状态不迁移、不自动删除，也不保留兼容管理器；若某个旧条目不在 CLI Inventory 中，界面只提示用户从原 source 重新安装。
- Search 失败只影响搜索结果，并始终保留直接输入 source 安装的入口。
- 从搜索结果安装时传入准确 Skill 名称；直接输入 source 时沿用 CLI 非交互语义，若 source 包含多个 Skills 则全部安装，并在入口旁明确提示该行为。

### Preview and translation UX

- Preview 必须展示已安装 Skill 目录的完整文件树，不只展示 `SKILL.md`。
- 用户选择文件后，由与文件类型匹配的只读 viewer 展示内容；preview 不提供编辑或写回。
- Markdown、代码、纯文本和图片使用内嵌 viewer；其他文件显示“不支持预览”、文件类型与大小。
- Preview 提供系统文件管理器入口，可定位当前文件，也可打开 Skill 根目录；不支持的文件不要求自动交给外部应用打开。
- 文件树不永久占据独立栏。预览顶部显示当前文件路径，点击后展开紧凑的文件树浮层，选择文件后收起并把空间还给 viewer。
- 文件树使用本地 Phosphor Regular 图标与缩进表达层级：Skill 根目录已由顶部路径按钮表达，因此树不重复显示 `/` 根节点；顶层文件与目录直接作为第一层，目录显示 folder icon 与可读名称，文件显示 file icon，不增加 `Folder:` 等重复类型前缀。完整 slash path 保留在交互数据与无障碍名称中。
- Installed Skill 的当前选择只用整行配色高亮与左侧 accent edge 表达，不显示额外的 `Selected` 文案；控件仍保留 `aria-selected` 和独立的键盘 focus ring。
- 启动时不默认选择任何 Installed Skill。宽窗口 Preview 区显示紧凑的选择提示且不显示文件、翻译或 Skill 操作；窄窗口停留在 Inventory 列表。方向键只移动焦点，点击或 Enter 才确认选择。
- 默认内容区为单个文件 viewer。用户开启翻译后切换为左右分栏，左侧原文、右侧译文；文件导航仍通过顶部浮层进入。
- 翻译开关旁持续展示“内容会发送给 Google”一类的出站披露，不额外增加首次确认弹窗。
- 翻译模式在当前 Preview Session 内保持开启；用户切换到另一个可翻译文件时自动发送并翻译新文件，不逐文件重复确认。
- 翻译只对 viewer 判定为可翻译的文本文件开放；切换文件或关闭翻译不修改磁盘。
- 翻译只对 Markdown 与纯文本说明文件开放；代码、JSON、YAML 和其他结构化配置只显示原文。
- Markdown 翻译只翻译自然语言正文，保留 frontmatter、围栏代码块、行内代码、链接 URL 和文档结构不变。
- 用户可在 Settings 中选择翻译目标语言；选择以 provider-neutral 的标准语言代码持久化，再由当前 provider 映射到其请求格式。
- MVP 使用精简固定列表：English、简体中文、繁體中文、日本語、한국어、Español、Français、Deutsch、Português、Italiano、Русский、العربية、हिन्दी；不维护 Google 的完整语言目录。
- 首次启动时目标语言从操作系统 locale 推导；若当前 provider 不支持则回退到 English。用户手动选择后持久化该选择，不再随系统或 UI 语言变化。
- 文件读取必须限制在所选 Skill 的解析根目录内，并对文件大小、无效编码和读取失败提供明确状态。

### Simplification target

- 删除 Managed Library、自有状态清单、Adoption、Configuration 管理、Content Drift reconciliation、单步 rollback、自建 Git import/update 和相关 UI。
- 不解析或修改第三方 `.skill-lock.json` 来推断额外所有权；让官方 CLI 独占其状态。
- Tauri 后端仅承担安全调用 CLI、读取已安装 Skill 内容以供预览/翻译，以及返回适合 GUI 的结果。
- 应用在调用 CLI 前检测 `node` 与 `npx` 是否可执行，并校验 Node 是否满足所采用 `skills` 版本的最低要求；检测失败时提供明确的安装或升级提示，不自动修改用户环境。
- Node 与 `npx` 只从桌面进程继承的 `PATH` 查找；MVP 不探测 nvm/fnm/Volta shell 初始化，也不提供可执行路径 override。
- 每次启动解析并记录一次 `skills@latest` 的实际版本，验证当前依赖的 `list -g --json` 输出，然后在整个应用会话中使用 `skills@<resolved-version>`；latest 不兼容时明确报错，不回退到解析终端文本或自建 lifecycle。
- 若启动时无法解析 latest，管理与 Preview 功能保持不可用并提供 Retry；MVP 不持久化或回退到上次成功解析的版本。
- 解析 latest 和执行精确版本时，外层 npx 均使用 `npx --yes` 避免包下载确认；add/remove/update 的 `-y` 只用于关闭 skills CLI 自己的交互确认。
- add/remove/update 的终端输出和退出码不是完整成功协议；操作后必须刷新 Inventory，并根据目标操作前后的实际 Inventory 判定结果。经过清理的命令诊断只作为辅助信息展示。
- Inventory 不包含 revision 或内容摘要，因此 Update 不显示无法证明的“已更新到最新版本”；刷新成功时显示“更新命令已完成，Inventory 已刷新”，并保留错误或异常诊断。
- Skill Deck 启动的所有 skills CLI 子进程统一设置 `DO_NOT_TRACK=1`，禁用官方 telemetry 与相关审计请求；MVP 不提供 telemetry 设置项。

## Acceptance Criteria

- [ ] 用户可以在 GUI 中完成官方 CLI 对应的 list/find/add/remove/update 主流程。
- [ ] 应用通过 `skills list -g --json` 构建已安装列表，不解析 ANSI 或面向人的表格输出。
- [ ] `skills@latest` 的版本或 JSON 契约不兼容时，应用停止相关操作并显示实际版本与升级 Skill Deck 的提示。
- [ ] 同一应用会话只解析一次 latest；Settings 显示解析出的精确版本，所有后续 CLI 命令使用该版本，下次启动才重新解析。
- [ ] latest 解析失败时不使用持久化旧版本，管理与 Preview 不可用且用户可以 Retry。
- [ ] 同一操作的 scope、Agent、安装方式和最终磁盘结果与所采用的官方 CLI 版本一致。
- [ ] Settings 中所有 CLI override 都有明确的行为型默认状态；恢复该状态后命令不再携带对应 override flag。
- [ ] 默认选项用可理解的行为描述加 “(Default)” 展示，并在 Settings 中显示实际 `skills` 版本。
- [ ] Appearance 只显示一个横排主题选择器，不出现独立的 mode/palette 两组配置；当前五个选项为 System (Default)、Light、Dark、Sand、Plum，并通过多个 token 色块清楚区分。
- [ ] 无论当前 OS 是 Light 还是 Dark，System tile 都同时展示两种模式的 chrome/content/interaction 预览，并与固定 Dark tile 清楚可辨；实际应用的 System 仍跟随 OS。
- [ ] System 根据系统选择 Light 或 Dark，并在系统主题变化时即时响应；任一显式主题跨重启保留且不再响应系统明暗变化。
- [ ] 所有主题覆盖窗口、toolbar、source list、Preview viewers、popover、sheet、状态栏、loading、focus、selected、warning、danger 和 disabled 状态；Agent tags、路径/控件 border、文件选中背景、Markdown code surface/border 均使用协调的主题语义 token，正文与控件文本满足 WCAG AA。
- [ ] 切换主题不重置当前 Inventory selection、Preview 文件、翻译模式、滚动位置或进行中的安全状态。
- [ ] Toolbar、Preview、sheet、文件动作和状态图标来自同一个 Iconify collection，所有主题中均保持清晰；icon-only 控件具有 accessible name 与 tooltip，关键或破坏性操作仍显示文字。
- [ ] 断网启动时所有图标仍立即可用，应用不向 Iconify API/CDN 发起请求。
- [ ] 显式 Agent target override 只影响后续安装，不迁移或重装现有 Skills；安装结果仍以 `list -g --json` 为准。
- [ ] CLI 返回的非 Codex/Claude Agent 可正常显示、筛选和参与默认安装流程，不需要新增前端 enum 分支。
- [ ] GUI 不再创建 Managed Library 或 Skill Deck package/installation 状态。
- [ ] 旧 Skill Deck 状态既不迁移也不删除；不在 CLI Inventory 中的旧条目不会进入新列表，必要时只提示从 source 重新安装。
- [ ] 已安装 Skill 可以被只读预览。
- [ ] Preview 文件树列出 Skill 根目录下的所有可发现文件和目录，并支持键盘选择。
- [ ] 文件树不显示冗余 `/` 根节点；顶层文件/目录从 `aria-level="1"` 开始，子文件使用下一层级，并以一致的本地 Phosphor 图标与缩进表达层级。界面中不出现 `Folder:` 前缀，tree item 保留完整路径名称。
- [ ] 文件树关闭时不保留空白栏；当前文件路径始终可见，重新打开文件树不丢失展开与选择位置。
- [ ] Installed Skill 选中行通过所有主题中均清晰的背景与 accent edge 高亮，不显示 `Selected` 文案；读屏仍能获得选中状态，键盘焦点与选中态可区分。
- [ ] 首次进入 Inventory 时没有默认选中项，所有行 `aria-selected="false"`；宽窗只显示紧凑选择提示，窄窗保持列表，点击或 Enter 后才进入 Preview。
- [ ] 选择受支持文件时使用合适的只读 viewer；无法内嵌展示的文件仍有明确的文件类型、大小和可用动作。
- [ ] 不支持的文件显示明确状态，并可从系统 Finder、Explorer 或文件管理器中定位；该操作不修改文件。
- [ ] 已安装的 Markdown 与纯文本说明文件可以进入只读翻译流程。
- [ ] 翻译结果只存在于预览会话中，关闭后不要求持久化，且安装目录内容与修改时间不发生变化。
- [ ] 代码与结构化配置 viewer 不显示翻译开关；Markdown 和纯文本 viewer 可进入双栏翻译。
- [ ] 翻译提供商的请求与错误处理不泄漏到 Skill 列表/预览 UI；替换 provider 不要求修改这些调用方。
- [ ] 用户无需配置账号或凭据即可请求翻译；网络或 provider 失败时保留原文并显示简短错误。
- [ ] 可翻译文件旁持续提示内容将发送至 Google；用户无需处理额外的一次性确认弹窗。
- [ ] Markdown 译文保留 frontmatter、代码块、行内代码、URL 和结构，只翻译自然语言正文。
- [ ] 翻译模式开启时切换至新的可翻译文件会自动翻译；切换到不可翻译文件不会发送内容。
- [ ] Settings 可选择并持久化目标语言；修改后下一次翻译使用新语言，不改写已展示的原文。
- [ ] Settings 只展示已确定的精简目标语言列表，不依赖运行时抓取 Google 语言目录。
- [ ] 开启翻译后，原文和译文左右并排且各自独立滚动；窄窗口降级行为在设计中明确。
- [ ] 删除原自建 lifecycle 后，应用仍能清楚展示 CLI 失败、缺失运行时和不支持的输入。
- [ ] 未安装 Node/npm、缺少 `npx` 或 Node 版本过低时，应用不执行管理命令，并展示可操作的提示。
- [ ] add/remove 出现“退出 0 但部分目标失败”时，GUI 不宣称全部成功，而展示刷新后的实际 Agent 安装结果和命令诊断。
- [ ] 每行 Update 直接更新单个 Installed Skill；Update All 需确认且不会在后台自动执行。
- [ ] Update 完成提示不声称 Inventory 无法证明的内容版本状态，只陈述命令完成与 Inventory 刷新结果。
- [ ] Remove 需确认并移除该 global Installed Skill 的全部 Agent 安装；MVP 不出现按 Agent Remove 控件。
- [ ] Search 不可用时 Installed Skills 与 source 安装仍可用。
- [ ] 搜索结果安装一个准确 Skill；直接 source 安装明确提示并接受 CLI 可能安装其中全部 Skills 的语义。
- [ ] CLI 子进程环境包含 `DO_NOT_TRACK=1`，且该值有 argv/environment 单元测试覆盖。
- [ ] macOS、Windows、Linux 的运行时分发策略明确且可验证。

## Out of Scope

- 自建 Skill registry、依赖解析、版本协议或 lock 文件。
- Project-scoped Skill 管理、项目目录选择和最近项目列表。
- 自建 Agent 配置启停、安装 ownership 接管或漂移修复。
- 自建 Git clone/update/rollback 流程。
- 自建更新检查、更新 diff 或“有新版本”状态。
- 后台自动更新、Skill 执行、安全评分、eval、router 或 marketplace。
- 主题编辑器、用户自定义/导入主题、任意色板、独立的“配色 × 明暗模式”组合矩阵，以及当前精选主题之外的扩展主题库。
- 运行时 Iconify 图标搜索、动态图标 collection、远程图标加载或完整 Iconify 图标库打包。
- 为官方 CLI 尚未承诺的内部数据结构建立长期兼容层。

## Evidence

- 官方 CLI README（核实于 2026-08-11）：<https://github.com/vercel-labs/skills/blob/main/README.md>
- 官方 package metadata（核实于 2026-08-11）：<https://github.com/vercel-labs/skills/blob/main/package.json>
- 官方 list JSON 实现（核实于 2026-08-11）：<https://github.com/vercel-labs/skills/blob/main/src/list.ts>
- 当前 GUI 契约研究：`research/official-skills-cli-contract.md`
- 原始产品规划：`../archive/2026-08/08-10-ai-agent-skill-manager/prd.md`
- 原始官方研究：`../archive/2026-08/08-10-ai-agent-skill-manager/research.md`
