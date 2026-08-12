# Fix startup blank screen and translation network

## Goal

让 Skill Deck 启动后立即显示可理解的 loading 界面，并让翻译在网络不可达时保持 UI 响应、快速失败，同时支持桌面应用常见的本地代理环境。

## Background and Confirmed Facts

- macOS 实机验收已复现：窗口会先显示一段时间的空白 WebView，待 `skills@latest` 解析和 Inventory 加载结束后才绘制 React loading/主界面。
- `runtime_status`、Inventory mutation、search 和 `translate_preview` 当前都是同步 Tauri commands；其内部执行阻塞式 `std::process::Command` 或 `reqwest::blocking`，会占用应用事件线程。
- 翻译失败截图显示请求直接访问 `translate.googleapis.com`，期间应用出现系统忙碌光标，之后才显示包含完整请求 URL 的底层错误。
- 翻译 HTTP client 当前没有 connect timeout 或 total timeout；reqwest 默认没有总超时。
- reqwest 0.13 默认只从进程环境变量 `HTTP_PROXY`、`HTTPS_PROXY`、`ALL_PROXY` 读取代理。macOS Finder/Dock 启动的 `.app` 不保证继承交互式 shell 的代理环境。
- Settings 已持久化主题、目标语言、Agent override 和 copy mode 到 localStorage，可复用同一偏好机制存储非敏感网络选项。
- `tauri dev` 从终端继承包含 Node/npm 的 PATH；macOS Finder 直接启动 release `.app` 时 PATH 不包含 `/usr/local/bin`，当前 `Command::new("node")` 因 `os error 2` 失败并把底层英文错误直接显示给用户。

## Requirements

### Responsive startup and commands

- 在任何 Node/npm、`npx skills` 或网络探测开始前，窗口必须先绘制完整 Skill Deck Header 与明确的启动进度状态；不得出现无内容白屏。Settings 和界面语言切换保持可用，其他依赖 runtime/Inventory 的控件禁用。
- 所有可能阻塞的 Tauri command 必须离开 GUI/event loop 执行，包括 runtime/list、search、add/remove/update、preview tree/read、Reveal 和 translation；操作期间窗口、滚动和非冲突控件保持响应。
- 启动继续遵守每个 app session 只解析一次 `skills@latest`，失败时显示 Retry，不增加持久化旧版本回退。
- macOS packaged app 必须在不依赖交互式 shell 初始化脚本的情况下，从继承 PATH 或标准 Node 安装目录解析同目录的 `node` 与 `npx`；不得为此执行用户 shell 配置。
- runtime 缺失、版本过低或 CLI 不兼容时，启动页显示本地化标题、简短原因和可操作的修复提示，不展示 `os error`、PATH 或底层进程错误。

### Translation network behavior

- 翻译请求使用 5 秒连接超时；一次翻译操作（包括所有分块）共享 15 秒总截止时间，不可达或过慢时在该边界内失败。
- 翻译结果是原子发布的；只有全部分块成功才显示译文，超时或任一分块失败时丢弃本次部分结果、保留原文并显示 Retry。
- 翻译失败只影响译文 pane，原文、文件选择和 Inventory 保持可用；用户可 Retry。
- 用户错误不显示包含待翻译文本的完整 query URL，也不泄漏底层请求细节；只显示可操作的网络/代理提示。
- 默认网络模式继续使用 reqwest 自动环境代理，不要求配置。
- Settings 提供可选的翻译代理 URL override；设置后只影响翻译 HTTP client，不改变 `npx skills` 子进程、search 或其他系统网络行为。
- 代理值在应用侧做 URL scheme、host 和长度验证；空值恢复自动环境代理。
- MVP 代理 URL 不允许包含用户名或密码，避免凭据明文进入 localStorage；仅支持无需认证的 `http://` 或 `https://` 代理。
- 代理输入先保留为 Settings 本地草稿；只有通过前端校验并点击 Apply 后才进入 Preferences/localStorage，Rust 在 command 信任边界再次校验。
- 同一文件的重复翻译采用 latest-request-wins；关闭翻译、切换文件/语言/代理或 Retry 后，旧请求的迟到结果不得覆盖当前状态。

## Acceptance Criteria

- [ ] 冷启动时首个可见帧包含完整 Skill Deck Header 与 loading 文案/进度，不出现空白内容窗口。
- [ ] runtime 探测中或失败时，Settings 和界面语言切换可用；依赖 runtime/Inventory 的其他控件不可操作。
- [ ] 人为延迟 runtime probe 时 loading UI 仍可绘制，窗口保持可交互。
- [ ] 人为延迟翻译 endpoint 时 UI 不出现应用无响应，原文可滚动且可关闭翻译。
- [ ] 连接超时或整次翻译的 15 秒总截止时间触发后，显示本地化、无原文/query 泄漏的错误和 Retry。
- [ ] 任一分块失败或总截止时间触发时不显示部分译文，原文保持可用。
- [ ] 未配置代理时沿用环境代理；配置有效 HTTP(S) 代理后翻译 client 使用该代理。
- [ ] 清空 override 后不再显式设置代理。
- [ ] 非法 scheme、缺失 host、超长值或带凭据但产品未允许的代理被拒绝并给出明确提示。
- [ ] 含凭据或其他非法值的代理草稿不会写入 localStorage；合法值仅在 Apply 后持久化。
- [ ] 关闭翻译、切换请求条件或 Retry 后，迟到的旧结果不会覆盖当前译文或错误。
- [ ] 现有 list/search/add/remove/update/Preview 行为和 `skills@latest` session pinning 不回归。
- [ ] Finder 类最小 PATH 下，若 `/usr/local/bin` 或 `/opt/homebrew/bin` 存在合格的同目录 `node`/`npx`，release app 能完成 runtime probe 与 Inventory 加载。
- [ ] 找不到 Node/npm、Node 版本过低或 Skills CLI 不兼容时，中英文界面显示对应的可操作错误和 Retry，不出现底层英文 `os error`。

## Out of Scope

- 通用网络栈、PAC/自动代理发现、代理连通性测试器或证书管理。
- 修改操作系统代理、自动探测本地代理端口或管理代理软件。
- 为 `npx skills`、catalog search 和翻译统一建设账户级网络配置。
- 后台自动重试、离线翻译模型或更换翻译 provider。
- 执行 `.zshrc`/`.zprofile`，自动管理 nvm/fnm/asdf，或在 app 内下载/升级 Node.js。

## Evidence

- 用户截图：`/var/folders/08/gxxgv031471g7c5njzclrnp80000ks/T/codex-clipboard-81d64513-94a4-4d3b-899d-f9e78265dc63.png`
- reqwest 0.13 proxy/timeout contracts: <https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html>
- Existing command contracts: `.trellis/spec/backend/command-contracts.md`
