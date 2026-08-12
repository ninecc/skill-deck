# Fix translation state and loading layout

## Goal

让翻译在当前代理环境下更可靠，并确保翻译与启动 loading 的 UI 状态只影响其实际所属的 Skill 和布局区域。

## Background and Confirmed Facts

- 用户截图显示 Brand Skill 的翻译曾触发 `translation_timeout`，并且切换到另一个 Skill 后仍保留双栏和“正在翻译”状态。
- 当前设置已应用 `http://127.0.0.1:7890`。使用当前运行构建对同一 Brand `SKILL.md` 连续实测两次均成功，第二次约 1.5 秒，说明超时是间歇性 provider/代理时延，不是稳定的代理字段遗漏。
- 翻译 UI 使用全局 `translationOn`。`chooseSkill` 会作废旧请求和清空翻译结果，但不会关闭翻译开关，因此新 Skill 的 Preview 返回后会自动进入双栏并发起翻译（`src/App.tsx`）。
- 启动时 `.runtime-screen` 与隐藏但仍参与布局的 `.workspace` 占据同一行；二者没有显式 grid column，CSS Grid 自动生成第二列，Header 只落在第一列（`src/styles.css`）。
- 后端保持 5 秒连接超时和一次操作共享 15 秒 deadline；Markdown 批次最多四路并发且原子发布（`src-tauri/src/translation.rs`、`.trellis/spec/backend/command-contracts.md`）。

## Requirements

1. 切换 Installed Skill 或当前 Preview 文件时关闭 Translation Session、丢弃会话结果、作废旧请求并回到单栏原文；新 Skill/文件不自动发起翻译，切回原 Skill/文件也不恢复旧结果。
2. 旧 Skill 的迟到翻译结果或错误不得出现在新 Skill。
3. 启动 loading 时保留完整 Header；“全部更新”保持可见但禁用。Header、loading 区和隐藏 workspace 必须占满同一应用宽度，不得因 Grid 自动放置产生隐式第二列，加载完成时不得发生 Header 布局跳动。
4. 保持 Settings 和语言切换在启动 loading 中可用，runtime 相关工作区继续不可操作。
5. 翻译操作继续遵守 15 秒共享 deadline。每个 provider 请求可在该 deadline 内对连接/超时类瞬态失败自动重试一次；不得因重试延长总等待，也不得重试不兼容响应。
6. 自动重试期间继续显示“正在翻译”；只有最终失败才显示错误和手动 Retry，不增加中间重试状态或文案。
7. runtime pending 与失败状态使用同一完整、全宽 Header；“全部更新”保持可见但禁用，Settings 与语言保持可用。
8. 结束 Translation Session 后，generation token 必须阻止旧结果或错误发布；本轮不增加跨 Tauri 主动取消，旧后台请求仍受 15 秒 deadline 约束。
9. Translation Session 开启时，用户应用新的目标语言或代理后立即用新参数重新翻译当前文档；这是显式设置操作，不要求再次点击翻译。
10. 不增加状态库、布局组件、翻译 provider 或新依赖。

## Acceptance Criteria

- [ ] 翻译 Skill A 时选择 Skill B，B 首屏为单栏原文且翻译按钮关闭，不显示 A 的译文、错误或“正在翻译”。
- [ ] A 的翻译 Promise 在切换后完成或失败也不能改变 B 的界面。
- [ ] 重新对 B 主动点击翻译后，才显示双栏和 B 自己的翻译进度/结果。
- [ ] 同一 Skill 内切换文件也关闭翻译并丢弃旧结果；新文件只有再次点击翻译后才发送。
- [ ] runtime pending 的 DOM/CSS 布局中 Header 和 loading 区覆盖完整窗口宽度，没有隐式第二列。
- [ ] loading 时 Settings 与语言选择仍可操作，workspace 不可见且不可交互。
- [ ] 当前代理下 Brand `SKILL.md` 能完成翻译；超时仍显示安全、本地化错误和 Retry，不泄漏请求 URL 或原文。
- [ ] 第一次连接/请求超时后可在同一 15 秒 deadline 内重试一次；第二次失败或 deadline 耗尽后立即返回超时，且不发布部分译文。
- [ ] 自动重试不产生额外中间 UI；最终失败仍提供现有手动 Retry。
- [ ] 关闭翻译或切换 Skill/文件后，旧后台请求即使完成、失败或自动重试也不能改变当前 UI；操作仍在原 15 秒 deadline 内结束。
- [ ] 应用新目标语言或代理会为当前文档发起一次使用新参数的翻译，并继续拒绝旧参数请求的迟到结果。
- [ ] 现有 Preview、文件切换、移动端翻译 tab、翻译原子发布及请求 generation 行为不回归。
- [ ] frontend 与 Rust 的格式、lint/typecheck、测试和 build/clippy 门禁通过。

## Out of Scope

- 更换 Google provider、离线翻译、凭据管理、自动代理发现或代理连通性测试器。
- 将翻译状态持久化或为每个 Skill 建立翻译缓存。
- 重做启动 skeleton、Header 或主工作区信息架构。

## Evidence

- 用户截图：`/var/folders/08/gxxgv031471g7c5njzclrnp80000ks/T/codex-clipboard-01632b2b-3a44-40a5-a073-d263e6f91b64.png`
- 用户截图：`/var/folders/08/gxxgv031471g7c5njzclrnp80000ks/T/codex-clipboard-25dec13f-0b26-43ad-a4ed-39b75e37387b.png`
- 当前构建实机：Brand 翻译通过 `http://127.0.0.1:7890` 成功；重复测量约 1.5 秒。
