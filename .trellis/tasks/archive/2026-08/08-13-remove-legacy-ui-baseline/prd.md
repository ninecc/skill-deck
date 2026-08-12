# 解除旧 UI 视觉基准

## Goal

解除当前 UI Token、原型 G、Graphite/Azure/Sand/Plum 色板和具体组件造型对后续设计任务的默认权威，使下一个视觉设计任务可以从新的艺术方向出发，而不必继承现有外观。

本任务只改变文档权威和未来 Agent 的解读方式，不设计新 UI，不改变当前运行时外观或功能。

## Background

- 归档任务 `08-11-simplify-around-npx-skills` 选定了原型 G，并定义了低变化、低动效、高密度以及 Graphite/Azure/Sand/Plum 视觉系统。
- 归档任务 `08-12-tauri-desktop-ui-ux-refactor` 要求保留当时的 Token maps，只允许补充语义状态 Token。
- 归档任务是历史证据，不应被改写；但活跃的领域文档、ADR 和前端规范不应继续把这些选择表述为未来必须保留的设计。

## Requirements

### R1 — 明确新的文档权威

- 新增 ADR，声明当前主题名称、色板、Token 数值、字体、密度、间距、圆角、阴影、品牌标记和组件造型只是当前实现，不是未来视觉基准。
- 统一使用 **Historical Visual Direction** 表示已失去默认权威但保留为项目历史的旧视觉选择，避免与已有 `Legacy App State` 术语混淆。
- 明确原型 G、旧 `ux-design.md`、旧 UI Audit 和归档任务不得被后续 Agent 默认当作视觉权威；它们仍可作为历史、功能或反例证据。
- 旧资料仍可用于证明交互、Accessibility、功能契约或失败方向，但后续任务引用时必须说明所用的非视觉结论，不得将整份历史资料注入为设计权威。
- 归档设计文档和原型不得整份加入后续视觉实现或视觉检查的 context manifest。为非视觉契约引用时，manifest `reason` 必须精确指出所需的行为、Accessibility 或反例证据。
- 定义 **Approved Visual Direction**：只有用户无歧义地指定具体方向、UI 作用域和平台作用域的任务内视觉方向，才能成为 UI 实现权威。模糊的积极反馈、当前代码、平台 HIG、原型、Agent 自评、测试通过或 design skill 输出都不会自动获得该地位。
- Approved Visual Direction 只对它显式声明的状态、组件、界面和平台生效。未覆盖部分保持当前实现；后批准的方向只在明确重叠的作用域内取代较早方向。
- Approved Visual Direction 只拥有视觉表达权，始终从属于产品需求、领域语言、Accessibility、安全边界、平台行为和已接受的架构 ADR。若方向需要改变这些契约，必须先在对应任务中显式修订，不得通过视觉批准隐式覆盖。
- 在任务执行期间，已批准的任务产物是该任务的视觉权威。需要约束未来任务的决定必须在完成前提炼到活跃 `.trellis/spec/` 或 ADR；未被提升的探索稿、截图和原型在归档后只是 Historical Visual Direction。
- 用户显式撤回的 Approved Visual Direction 保留原产物和历史记录，但在声明的作用域内重新归类为 Historical Visual Direction，不删除或改写历史。
- 本任务只建立显式批准原则，不规定后续任务必须产出的方案数量、截图矩阵或 skill 组合。

### R2 — 保留非视觉契约

- 保留语义色角色、可读对比度、`focus-visible`、reduced motion、键盘操作、状态可辨识性和系统控件兼容性等功能性契约。
- Theme Accent 与 System Accent 分离的架构原则保留；五个当前主题的名称、数量和具体映射不再被 ADR 保证为永久产品约束。
- 当前运行时仍然保留 `system | light | dark | sand | plum`；后续设计任务若要改变产品行为，必须另行规划与迁移。
- `Theme Preference` 在领域语言中只表达用户对应用外观的持久化意图；当前五个枚举属于实现事实，不是该领域术语的永久定义。
- 语义 Token 保留的是可读性、选中/焦点可区分、Danger/Warning 等非装饰性结果，不是当前 Token 名称、数量或分层。后续 Approved Visual Direction 可以合并、拆分、重命名或替换 Token taxonomy，但必须保留这些结果并安全迁移。

### R3 — 放宽活跃前端规范

- 将 Inventory 选中态的具体“填充 + accent edge”规定改为语义要求：选中与键盘焦点必须可区分，且不能只依赖颜色；具体造型由后续获批准的设计决定。
- 在前端规范中明确：现有 `styles.css` 和组件外观是待重新设计的实现状态，不是视觉保持约束。
- 取消原型 G 对 Toolbar、master/detail、Status、Modal、Popover 等具体构图的视觉权威，但保留命令可发现性、状态反馈、键盘路径、焦点恢复和响应式可操作性等交互契约。具体高度、宽度、比例、位置和外观由后续 Approved Visual Direction 决定。
- ADR-0012 的单窗口工具模型、ADR-0013 的跨平台 Command Model 和 ADR-0015 的 Single Command Authority 保持完整有效；后续若要改变这些交互架构，必须单独修订对应 ADR。
- Application Command 名称、领域语言和本地化契约不属于纯视觉授权。旧原型文案没有权威，但后续修改当前 UX Writing 必须显式纳入任务范围，不得仅凭 Approved Visual Direction 自动改写。
- 当前 `820px` 断点与 Toolbar/Content/Status 三行结构作为当前实现的回归表面保留，但不是永久视觉基准，可被后续 Approved Visual Direction 显式替换。长期契约只要求最小窗口下核心流程可操作、状态反馈可见且内容正确滚动。
- 没有 Approved Visual Direction 授权的非设计任务必须把当前外观当作回归表面并控制视觉 diff，不得因“旧基准已解除”而顺手重设计。
- Approved Visual Direction 必须显式声明平台作用域。仅批准 macOS 时，Windows/Linux 只允许接受为维持共享实现所必需且不破坏现有可用性的变化，不得声称它们的视觉方向已获批准。
- macOS HIG 等平台指南作为平台行为、原生控件和可用性护栏；其视觉建议只是设计输入，除非被明确纳入 Approved Visual Direction，不得以“HIG 推荐”为由越过用户批准。

### R4 — 记录部分取代关系

- 新增状态为 `accepted` 的 ADR-0016，只取代 ADR-0014 中“必须保留五套完整主题及当前视觉身份”的部分，不取代 Theme Accent 与 System Accent 分离的架构原则。
- ADR-0014 保留原编号和历史上下文，但必须显式指向新 ADR 说明部分取代关系。
- ADR-0016 只管理视觉权威的来源、作用域、优先级和生命周期，不包含新色板、风格、布局、品牌或组件处方。本任务完成后，项目明确处于“没有 Approved Visual Direction”的状态，直到后续独立设计任务获得无歧义批准。

## Acceptance Criteria

- [ ] 有一份活跃 ADR 明确说明哪些旧设计资产被取消默认权威，哪些语义、Accessibility 和平台契约仍然有效。
- [ ] `docs/adr/0014-separate-theme-accent-from-system-accent.md` 不再承诺必须保留五套完整产品主题，并指向新 ADR 的视觉权威边界。
- [ ] `CONTEXT.md` 清楚区分当前 Theme Preference 运行时枚举与未来视觉基准。
- [ ] `.trellis/spec/frontend/component-guidelines.md` 保留选中态语义要求，但不固定 fill/edge 造型，并声明现有 CSS 不是视觉权威。
- [ ] 活跃文档使用 `Approved Visual Direction` 作为唯一的新视觉权威术语，并明确它必须来自用户对任务内可审阅设计产物的显式批准。
- [ ] 当前方形 `S`、已发布 App Icon 和 `design/app-icon-concepts/` 中的概念均被归类为无默认权威的视觉资产，但本任务不修改或删除它们；`Skill Deck` 产品名称和领域语言保持有效。
- [ ] ADR-0016 明确 Approved Visual Direction 从属于非视觉契约，定义任务内权威、跨任务提升和撤回后的 Historical Visual Direction 身份。
- [ ] ADR-0016 明确本任务不产生新视觉处方，任务完成时项目不存在 Approved Visual Direction。
- [ ] 归档任务、原型、生产 UI 代码、主题偏好 schema 和当前用户外观均未被修改。
- [ ] 全库检索可证明：旧视觉处方只存在于历史归档或当前实现，不再作为活跃的未来设计保留要求。

## Out of Scope

- 新视觉方向、新原型、参考图、品牌设计或 UI 审美评审。
- 修改 `src/`、`src-tauri/`、当前 Token 值、主题选项、布局、组件或任何运行时行为。
- 修改或删除 `.trellis/tasks/archive/` 中的历史任务和原型。
- 修复本轮实屏检查发现的 CLI 无 deadline / 无限 loading 问题。

## Planning Status

这是一个轻量文档任务，`prd.md` 足以表达实施边界，不需要额外的 `design.md` 或 `implement.md`。当前没有未决的产品或范围问题。
