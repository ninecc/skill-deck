# Improve startup loading UX

## Goal

缩短 Skill Deck 冷启动等待时间，并让启动、Inventory 加载与空预览状态使用符合用户心智的表达；压缩已安装 Skill 列表中的无效视觉信息。

## Background

- `runtime_status` 当前在解析并固定 `skills@latest` 后执行一次 `list -g --json` 来验证 Inventory；React 收到 ready 状态后又调用 `list_skills`，因此冷启动连续读取两次相同的全局 Inventory（`src-tauri/src/cli.rs`, `src/App.tsx`）。
- runtime 未完成时 UI 显示“正在连接 Skills CLI”，暴露了实现细节；Inventory 尚未返回时空数组会短暂触发“尚未安装”的错误语义（`src/i18n.ts`, `src/App.tsx`）。
- Inventory 行为每个 Agent 展示一个标签，45 个 Skill 时形成高密度重复信息；用户明确认为这些标签无效且冗余。
- 未选择 Skill 时，Preview 使用居中的图标和文案占据整个详情区，视觉重心过空。
- “查找 Skills”与 source 安装常驻 Inventory 列表底部并限制在 `max-height: 43%` 的独立滚动区；大量 Installed Skills 或较矮窗口下，查找输入和安装表单互相挤压，主要 Inventory 也失去可用高度。
- 真实网络探测已确认 `http://127.0.0.1:7890` 能访问应用使用的 Google Translate endpoint；当前正在运行的 Skill Deck 设置页却没有“翻译代理”字段且仍显示旧 Agent 标签，表明验证对象不是当前源码对应的构建。
- 已归档的启动任务已保证 Header 先绘制、阻塞工作经 `spawn_blocking` 执行；本任务保留该行为，不重建启动架构。

## Requirements

1. 冷启动只执行一次用于兼容性验证和首屏 Inventory 的 `skills list -g --json`，且继续在同一进程生命周期固定已解析的 Skills CLI 版本；本轮不改变 `skills@latest` 解析策略，也不承诺固定启动时长。
2. 启动等待使用轻量 CSS spinner 与面向用户的“正在加载 Skills”文案，不显示“连接 Skills CLI”等实现细节；系统开启减少动态效果时停止旋转。
3. Inventory 返回前不得显示“尚未安装”；该空状态仅在加载成功、没有筛选词且 Inventory 确实为空时出现。非空 Inventory 的筛选无结果显示独立的“没有匹配的 Skill”文案。
4. 翻译等待使用独立的“正在翻译”文案，不复用启动 loading 文案。
5. 已安装 Skill 行保留名称与 source/path，移除逐 Agent 标签；筛选只匹配可见的名称、source 和 path，不匹配隐藏的 Agent Target；不改变 `InstalledSkill.agents` 数据合同。
6. 未选择 Skill 时保留可理解的提示，但移除大号居中图标，并将提示收敛为详情区顶部的紧凑辅助文本。
7. 保持中英文文案、键盘选择、`aria-selected`、选中态、Preview、安装/更新/移除与错误 Retry 行为不回归。
8. “查找与安装”从 Inventory 底部移出，在已安装列表标题区提供明确入口；入口打开复用现有视觉语言的右侧面板，完整容纳 catalog 搜索、搜索结果和 source 安装，面板独立滚动且不再压缩 Inventory。
9. 使用当前源码构建验证翻译代理：设置并应用 `http://127.0.0.1:7890` 后，从真实 Installed Skill 触发翻译。只有当前构建仍可重复超时且能定位到应用代码时，才修改 15 秒共享 deadline 或代理实现；不得因旧构建现象盲目放宽超时。

## Acceptance Criteria

- [ ] 冷启动从 runtime 探测到首屏 Inventory 可用仅调用一次 `skills list -g --json`；不再由前端追加 `list_skills` 调用。
- [ ] 启动中的中英文界面显示轻量 spinner 与“Loading Skills…”/“正在加载 Skills…”，不出现“Connecting/连接 Skills CLI”；`prefers-reduced-motion` 下没有旋转动画。
- [ ] 慢启动及筛选无匹配时不显示“未安装”结论；仅成功返回空 Inventory 且没有筛选词时显示真实空状态，筛选无匹配显示独立文案。
- [ ] 翻译等待显示“Translating…”/“正在翻译…”。
- [ ] Inventory 行不渲染 Agent 标签，只显示 Skill 名称和 source/path；筛选可匹配这三个可见字段且不匹配隐藏 Agent Target。
- [ ] 未选择 Skill 时详情区顶部显示紧凑提示，不渲染大号文件图标或居中占位块。
- [ ] Inventory pane 不再常驻查找/安装表单；“查找与安装”入口打开独立右侧面板，较矮窗口下搜索框、source 输入、说明和安装按钮均可通过面板滚动访问。
- [ ] 右侧面板支持关闭、键盘焦点可见、窄屏占满可用宽度，并保留现有 search/add 行为和错误展示。
- [ ] 通过真实 `127.0.0.1:7890` 代理完成当前构建的最小翻译回归，或留下可重复的当前构建失败证据与根因；旧构建现象不作为修改超时的依据。
- [ ] frontend 与 Rust 的格式、lint/typecheck、测试和 build/clippy 门禁全部通过。

## Out of Scope

- 持久化或展示缓存 Inventory、后台预热、离线启动或旧 Inventory 回退。
- 改变 upstream Skills CLI、Node/`npx` 解析策略或版本固定策略。
- 重做 Inventory 信息架构、增加 Agent 筛选器或新的列表设置。
- 为短暂 loading 增加进度百分比、骨架屏组件或动画依赖。
- 自动探测本机代理、修改系统网络设置、增加代理连通性测试 UI，或在没有当前构建失败证据时改变翻译 deadline。
