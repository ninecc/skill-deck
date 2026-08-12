# Tauri 三端桌面 UI/UX 重构

## Goal

在不改变 Skill lifecycle、Tauri command DTO、翻译和预览业务逻辑的前提下，把 Skill Deck 重构为 macOS 优先、Windows/Linux 自然可用的桌面工具：提供可发现的原生命令、完整键盘操作、可靠焦点管理、清晰的 master/detail 层级、紧凑一致的组件状态和稳健的窄窗口布局。

## Background

- 当前应用是 Tauri 2 + React 19 + strict TypeScript；窗口默认 1180×800，最小 760×560，保留标准系统装饰。
- UI 已有语义 theme tokens、Light/Dark/System/Sand/Plum、系统字体回退、`focus-visible`、reduced-motion、source-list Arrow 导航和响应式 Preview。
- 既有设计结论已否决网站式顶部导航、巨型标题、card everywhere、营销页结构、Google Fonts 与 GSAP；确认 compact master/detail + narrow navigation stack。
- 当前基线通过 format、lint、typecheck 和 22 个 Vitest 测试。

## Requirements

### R1 — Shared desktop structure and hierarchy

- 保留共享的 Inventory master / Preview detail 信息架构；Find & Install、Settings 仍是临时 utility surface，不成为顶级页面。
- 顶部区域收敛为紧凑桌面 command toolbar；高频 Find & Install、Refresh Inventory、Update All、Settings 可发现，语言切换移入 Settings 后仍可立即使用。
- 运行结果使用始终存在的固定状态容器；内容按当前状态动态更新且保持低噪声，不依赖瞬时颜色提示，也不因出现/消失推动布局。
- Status 左侧显示当前 Command Lifecycle 的 Busy 或最近 Outcome；无活动且无历史结果时显示 Ready。新命令开始即替换旧 Outcome，Success/Partial/Error 均不自动消失。右侧始终显示 Installed 数量与 CLI Session Version 等稳定环境信息。
- 窗口宽度约 820px 以下切换为 Inventory → Preview 单栈，不维护第二套页面。

### R2 — Native window and platform adaptation

- 保留 Tauri 标准窗口装饰、原生 resize/fullscreen/traffic lights/window controls；不实现自定义 titlebar。
- 因未覆盖系统 titlebar，不添加无意义的 `data-tauri-drag-region`，避免可点击工具栏误触窗口拖动。
- 将最小窗口调整到仍可用的约 720×520，并使 720–820px 范围实际覆盖窄窗导航。
- 只在真实差异处适配：macOS 使用 Meta 快捷键和标准菜单约定；Windows/Linux 使用 Ctrl/Alt 约定；共享页面与 CSS 不复制。

### R3 — Menu, keyboard and context commands

- 三端共享完整 Command Model、命令 ID、快捷键和 Context Menu。macOS 使用完整系统 Menu Bar；Windows/Linux 使用符合各自桌面习惯的原生菜单/应用菜单 + Toolbar，不强制复制 macOS 的呈现形态。
- Command Model 使用 App、Inventory、Edit、Skill、Window、Help 等稳定命令域；`Skill` 表示对当前 Installed Skill 的 Translate、Reveal、Update、Remove 等命令，`Preview Session` 仅描述只读内容视图。
- Command Model 暴露 Find Installed、Find & Install、Refresh Inventory、Update All、Settings 及适用的平台快捷键；同一命令的 enable/disable 状态在所有入口一致。`Cmd/Ctrl+R` 只调用现有 `runtime_status` 重新读取 Inventory；Update All 不分配快捷键。
- 每个 Application Command 由 Single Command Authority 管理唯一的 availability、execution 和 lifecycle。Toolbar、Menu、Keyboard Shortcut、Context Menu 只能通过 Command Dispatcher 触发，不能直接调用对应业务 action；Dispatcher 执行前再次检查 availability。
- Platform Role Command（Cut/Copy/Paste/Select All、Hide、Quit、Minimize、Zoom、Fullscreen 等）使用 Tauri predefined/native role，可记录在 Command Model 中，但不进入 React Command Registry，也不复制系统 availability 或 execution。
- Availability 返回 `{ enabled, reason }`，reason 使用结构化枚举（至少覆盖 runtime-unavailable、inventory-empty、no-skill-selected、mutation-active、modal-active、unsupported-document）。本轮不要求每个 reason 都可视化，但菜单状态、Tooltip、Accessibility 与诊断不得另造冲突判断。
- Mutation lifecycle 保留明确 operation kind，不收缩成长期单一 `loading: boolean`；本轮只实现当前需要的冲突规则，同时为未来按 operation kind 细化保留真实状态。
- 本轮所有 lifecycle mutation 互斥。Mutation active 时 Refresh Inventory 与其他 mutation unavailable；Settings、Find Installed 和已加载 Preview 的只读浏览保持可用。Find & Install 提交期间禁止重复提交，只有后续实际需求证明安全时才放宽并发。
- `Cmd/Ctrl+F` 聚焦 Inventory filter，`Cmd/Ctrl+Shift+I` 打开 Find & Install，`Cmd/Ctrl+,` 打开 Settings，Escape 关闭最上层临时 surface；窄窗 Back 支持平台惯用组合。
- Skill Deck 是单窗口管理工具，不占用 `Cmd/Ctrl+N`，不把 Find & Install 伪装成 New Window/New Document；Settings 保持主窗口内模态 utility surface。
- Installed Skill 右键先同步 Selection/Preview，再显示当前适用的 Skill Commands（Translate、Reveal、Update、Remove）；关键操作不能只存在于 Context Menu。
- 不给 Update/Remove 添加容易误触的单键快捷键。

### R4 — Accessibility and interaction model

- Settings、Find & Install 对话框打开时获得合理初始焦点，Tab 不逃逸，Escape 关闭，关闭后焦点回到触发控件。
- File tree popover 支持打开后的合理焦点、Arrow 导航、Escape 关闭并归还焦点；translation tabs 支持完整 ARIA 关系与方向键切换。
- 新错误使用 `role="alert"` 或等价 live region；busy/disabled/pressed/selected 状态同时有语义和视觉表达。
- 所有 icon-only control 保留可访问名称；键盘 focus 与 selection 视觉不混用。
- 只有 Busy/Outcome 等有意义的动态变化进入 live region；Ready 和右侧稳定环境信息不宣告。
- Command feedback 使用 Summary + Details：Status 单行直接区分 Success/Partial/Error；有 diagnostics 时显示 Details，打开可选择、复制、内部滚动的非模态 Diagnostics Popover。完整 diagnostics 不进入 live region，也不保留历史。
- Outcome 只按结构化证据分类：Error 来自 Command Error；Partial 只表示确定目标未被观察到；Success 只在目标完成可证明时使用。Update/Update All 使用中性完成文案。Direct Source `changedSkills` 为零时使用中性 Outcome + Review，不解析 diagnostics 或猜测。
- 所有 `window.confirm` 调用移除。不可撤销的 Whole-Skill Removal 使用共享 Confirmation Modal，Cancel 是安全退出路径，destructive action 明确标识；若未来删除获得可靠 Undo，则重新评估并优先取消确认。
- Update All 在 runtime ready、Inventory 非空且无冲突 busy operation 时直接执行，不显示确认。
- Settings 使用 live-save preference model：普通 Preference 立即应用并持久化，没有全局 Save/Cancel transaction；Done 与 Escape 只关闭且不回滚，Backdrop 不关闭。Proxy 保留 dialog-local draft 与局部 Apply，未 Apply draft 在关闭时丢弃。
- Modal Shell 使用普通 DOM + ARIA，不依赖 HTML `<dialog>` 或新增依赖；统一处理 `role="dialog"`、`aria-modal`、初始焦点、Tab/Shift+Tab 约束、Escape、Backdrop 策略与焦点恢复，以覆盖最低 macOS 12 WebKit、Windows WebView2 与 Linux WebKitGTK。
- Find & Install 是一次性 Command Dialog。单个 Skill 安装成功或已安装且能唯一解析目标时，关闭 Dialog、使用刷新后的 Inventory 选择目标并进入其 Preview Session；失败或部分成功时保持 Dialog 和用户上下文，并提供恢复操作。不增加 Success Modal/Toast。
- Command Lifecycle 独立于 Modal Lifecycle。Install/Remove 提交后用户仍可 Escape/Close，界面明确说明关闭不会取消命令；命令继续由 Status 跟踪且重复提交保持禁用。Dialog 已关闭后的 failure/partial recovery 由 Status Summary + Details 提供。
- 安装 failure/partial 且 Dialog 已关闭时，Status Summary 提供 Review，恢复最近一个未解决 Find & Install workflow 的 query、results、source 与目标上下文；Retry 只在恢复后的 Dialog 中执行，Diagnostics Details 不承担恢复。新的 workflow 开始或问题解决时替换/清除该 context，不保存历史。
- 安装目标只严格解析：Search Result 按请求的 exact Skill name 在 refreshed Inventory 唯一匹配（包括 Already Installed）；Direct Source 仅在 `changedSkills` 恰有一个且存在于 Inventory 时定位。零个或多个不使用 name/source/path heuristic、不新增结果选择器，Dialog 保持打开并说明结果。
- UI Language Preference 从 Toolbar 移入 Settings，与 Theme 同属 application preferences；支持 System Default 与 explicit locale override，变更立即生效并持久化。现有 `skill-deck-locale` 显式选择在首次读取新版 preferences 时迁移，避免升级重置。
- Preference 保存用户意图，Effective UI Locale 使用受支持的 BCP 47 ID 从 `navigator.languages` / `navigator.language` 派生。启动和切回 System Default 时解析；System Default 下监听标准 `window.languagechange` 作 best-effort 更新。explicit locale 不受系统变化影响；不轮询、不监听 focus、不新增 Tauri plugin。
- 采用 Single Top-level Modal Policy：Settings、Find & Install、Remove Confirmation 互斥。Popover、Context Menu、File Tree、Diagnostics 不计入顶层 Modal，但打开 Modal 前统一 dismiss。Modal 内需要进一步确认时切换当前 workflow state，不叠加第二个 Modal。
- 采用 Single App-owned Transient Surface Policy：File Tree 与 Diagnostics 等应用 transient 任意时刻最多一个，打开新的先关闭旧的。Native Context Menu 前关闭应用 transient，但其关闭生命周期由系统管理，不复制为 React 状态。Escape 优先关闭 transient，再处理 Modal 或窄窗 Back。
- Modal 存在时只禁用会打开另一顶层 Modal或与当前 workflow 冲突的命令；Dialog 内合法命令与系统 Edit 行为保持可用。关闭后优先恢复触发控件，目标消失时恢复稳定上下文。
- Effective UI Locale 变化时自定义 Application Command label 即时更新；Tauri predefined/native role 保持系统本地化。Menu presentation 幂等更新/重建，不重建 Registry、不改变 availability、不触发业务状态。
- Selection 改变时立即清除旧 Preview，显示 Skill-scoped loading；tree/file 原子发布，旧请求继续通过 generation/request token 作废。加载期间 Reveal Skill Root、Update、Remove 可用；file-scoped Reveal/Translate unavailable，reason 为 document-loading 或 unsupported-document。
- 窄窗显式 Back 恢复原列表 scroll position，并优先聚焦 selected row；row 不存在时聚焦 Filter，再回退 Inventory heading。断点 resize 不主动移动焦点、滚动或重载。
- File Tree 保持完全展开，不增加折叠/lazy/expansion state。目录只表达层级且不可选择；文件支持 Up/Down、Home/End、Enter/Space，Escape 关闭并恢复 path trigger；不宣称 Left/Right expand/collapse。
- Translate 是 Toggle Application Command：label 为 Show/Hide Translation，Toolbar 使用 `aria-pressed`，Native Skill Menu 使用 checked item。Hide、切换 Skill/file 终止 Translation Session；请求中 Hide 立即作废旧 generation。
- Preview tree/file read 或 refresh failure 在 Detail 内显示原因与 Retry，不改变 Runtime ready、不覆盖最近 Command Outcome，也不使用全局 Banner/Toast；Retry 受 Preview generation token 保护，仍适用的 Skill Commands 保持可用。
- Refresh Inventory 成功后：当前 Skill 仍存在则保持 Selection，并优先重新打开原路径；原文件消失时回退 `SKILL.md` 或首个可预览文件。Skill 消失则清除 Selection、返回 Inventory 并生成 Outcome。Preview 重载失败只产生 Skill-scoped Error，不污染 Runtime；filter、列表滚动和 CLI Session 保持。

### R5 — Shared design tokens and WebView CSS

- 在现有 semantic tokens 上补齐 control hover/pressed、status、scrim、danger border、motion、radius 和原生 `color-scheme`；组件不新增零散色值。
- Theme Preference 控制 application identity 与自定义组件的 Theme Accent；浏览器/系统原生控件不强制染成产品色，在平台与 WebView 支持时保留系统外观与 System Accent。Success/Warning/Danger 独立于 Theme Accent。
- 控件采用桌面 28–34px 密度、4/8px 节奏、系统 sans/mono；不引入字体、UI、CSS 或动画依赖。
- 避免 macOS 12 WebView 风险：关键布局/选中状态不依赖 `:has()` 或无 fallback 的 `color-mix()`。
- 动效仅用于 120–180ms hover/pressed/surface state，使用 opacity/background；键盘导航即时，reduced-motion 禁用可选 transition。
- 每个交互元素具有 hover、active、focus-visible、disabled、selected/pressed（适用时）状态，且不通过 transform 引发布局抖动。

### R6 — Preserve product boundaries

- 不改变 backend command 行为、CLI lifecycle、DTO、业务数据模型、translation generation-token 契约或 Preview 安全边界；仅允许为 UI Language Preference 扩展并兼容迁移本地 preferences schema。
- 不新增全局状态库、UI 框架、动画库、字体或窗口状态插件。
- 继续保留 Markdown raw HTML 禁用、链接/远程图片惰性化和现有本地 preferences 规则。

## Acceptance Criteria

- [ ] macOS 有标准原生菜单结构和 Meta 快捷键；Windows/Linux 对应 Ctrl/Alt 行为不冲突。
- [ ] 三端使用相同命令 ID、可用性规则、快捷键语义和 Context Menu；菜单的视觉呈现按平台惯例适配。
- [ ] 所有 Command Surface 只触发 Dispatcher；每个 Command 只有一个 availability result 和结构化 reason；Dispatcher 会拒绝 stale/unavailable 触发。
- [ ] Tauri 原生 Menu 的 enabled 状态同步自同一 Command State，不在 Rust/Toolbar/Menu 中复制业务判断。
- [ ] Platform Role Command 只使用 predefined/native execution；React Registry 不复制 Cut/Copy/Paste/Window/App 系统角色。
- [ ] `Cmd/Ctrl+R` 只刷新 Inventory，Update All 无快捷键且不与 Refresh 混淆；菜单命令域命名为 Skill 而非 Preview。
- [ ] 标准 titlebar 与平台窗口控件保持可见可用；主窗口自由 resize/fullscreen，720×520 仍可完成核心流程。
- [ ] 1180px 为 master/detail，720–820px 为单栈；resize 不清除 Skill、file、translation 或 preference 状态。
- [ ] Settings、Find & Install、file popover 可用键盘完整打开、操作、Escape 关闭并恢复焦点，没有 keyboard trap。
- [ ] Modal Shell 不调用 `<dialog>.showModal()`，在最低三端 WebView 上使用共享 DOM/ARIA 焦点契约且不新增依赖。
- [ ] Installed row 右键同步 Selection/Preview 后显示原生 Skill Context Menu，且其中每个命令都有可见替代入口。
- [ ] Light、Dark、System、Sand、Plum 的 text/border/focus/hover/pressed/disabled/status 均清晰；native controls 使用正确 `color-scheme`。
- [ ] 状态栏容器始终存在、内容低噪声且按状态更新；任何状态变化不改变 Workspace 几何位置。
- [ ] 状态左侧严格遵循 Busy > latest Outcome > Ready；新命令替换旧 Outcome，结果无消失计时器；右侧稳定信息不进入 live region。
- [ ] diagnostics 只通过 Status 的 Details 打开非模态 popover，支持选择/复制/滚动/Escape/外部关闭；新命令和顶层 Modal 会关闭并替换旧 details。
- [ ] Outcome severity 只由结构化字段决定；Update 使用中性文案；Direct Source 零 changedSkills 不被误报 Success/Partial。
- [ ] Update All 正常状态下直接执行；Whole-Skill Removal 使用共享 Confirmation Modal；仓库中不再调用 `window.confirm`。
- [ ] Settings 普通 Preference live-save；Done/Escape 不回滚；Backdrop 不关闭；未 Apply 的 Proxy draft 关闭时丢弃。
- [ ] Find & Install 成功/Already Installed 后定位唯一 Skill 并关闭；失败/部分成功保留 Dialog 上下文和恢复入口。
- [ ] Search Result 与 Direct Source 只按严格规则定位；零/多目标不猜测、不新增选择器；最多保留一个未解决 workflow context，Status Review 可恢复且 Retry 留在 Dialog。
- [ ] Install/Remove 命令运行时 Modal 仍可关闭且明确“不取消”；命令与结果继续在 Status 中可见，重复提交不可用。
- [ ] Language 位于 Settings，支持 System Default + explicit locale override；旧显式语言选择迁移且变更不刷新 Inventory 或重置 Preview Session。
- [ ] System Default 从受支持的 BCP 47 matching 派生 Effective UI Locale，并 best-effort 响应 `languagechange`；stored preference 不被系统变化改写。
- [ ] 任意时刻最多一个顶层 Modal；transient surface 在 Modal 打开前关闭；Modal 不会无差别禁用系统 Edit 或当前 workflow 内合法命令。
- [ ] 右键未选中 Skill 会同步 Selection 与 Preview 后再打开 Context Menu，但不产生额外动画、滚动或 keyboard focus 转移。
- [ ] Mutation active 时其他 mutation 与 Refresh unavailable，但 Settings、Find Installed 和已加载 Preview 只读浏览仍可用。
- [ ] Refresh 保留仍存在的 Skill 与原文件路径；文件缺失按约定回退；Skill 消失回 Inventory；Preview failure 不改变 Runtime ready 状态。
- [ ] Selection 切换不会短暂显示旧 Skill 内容；loading 期间 root-scoped 与 file-scoped Command availability 正确区分。
- [ ] Preview failure 只显示 inline reason + Retry，不覆盖 Runtime/Command Outcome；Retry 不允许旧 generation 发布。
- [ ] Inventory 维持单选，不出现 Cmd/Shift 多选、批量 Remove 或隐含批处理状态。
- [ ] Toolbar 只有 Find & Install 持续显示文字；宽窗 Refresh/Update All 显示文字，820px 以下转为可靠 Tooltip + accessible name 的图标按钮；Settings 始终 icon-only，不增加 Overflow。
- [ ] disabled Toolbar/Skill action 的本地化 reason 通过可聚焦包装或 `aria-describedby` 等可靠结构提供，不依赖 disabled control 自身接收 pointer/focus；Native Menu label 不拼 reason。
- [ ] 自定义菜单 label 随 Effective UI Locale 更新，predefined roles 仍由系统本地化，更新不触发业务状态。
- [ ] 821px 使用 master/detail，820px 与 819px 使用单栈；布局切换不触发 React resize state 或数据重载。
- [ ] 窄窗 Back 恢复 list scroll，并按 selected row > Filter > Inventory heading 恢复焦点；断点 resize 不执行导航副作用。
- [ ] File Tree 完全展开且只让文件可操作；Up/Down/Home/End/Enter/Space/Escape 均符合约定，不实现折叠键行为。
- [ ] 任意时刻最多一个 app-owned transient；Escape/Modal/Native Context Menu 的关闭优先级符合约定且不复制 native menu lifecycle。
- [ ] Translate 的 Toolbar pressed、native checked、动态 label 与 Translation Session 同步；Hide 能立即作废进行中的请求。
- [ ] 关键布局不依赖 `:has()`，danger border 不依赖无 fallback 的 `color-mix()`；无新增运行时 UI 依赖。
- [ ] reduced-motion 下无装饰 transition；键盘列表/树/tabs 切换即时。
- [ ] format、lint、typecheck、Vitest、production build 全部通过；Rust menu 变更通过 fmt、clippy、tests。
- [ ] 针对 menu events、shortcuts、dialog focus/Escape、context menu command mapping、narrow layout state 增加最小行为测试。
- [ ] macOS 完成真实 App 的 Menu/shortcut/context menu/resize/focus/theme/reduced-motion smoke；Windows/Linux 仅按共享实现、Tauri contract、测试与 build 给证据，真实 smoke 前评级不超过 Good。

## Out of Scope

- 自定义/无边框 titlebar、手工重画 Traffic Lights 或 Windows/Linux window controls。
- 三套页面、三套主题或平台 fork 的业务组件。
- 多窗口、窗口位置/尺寸持久化、Spotlight/Share/Services/Quick Look、Dock menu 与拖放导入。
- New Window/New Document 命令和独立 Settings Window。
- 业务流程、CLI/backend command、数据模型、网络协议和翻译算法重构。
- 新 UI/CSS/animation framework、Google Fonts、GSAP、复杂 onboarding 或装饰性视觉效果。
- Overflow Menu、Inventory multi-selection、diagnostics history/Activity Log、Direct Source 结果选择器。
