# 修复 Skill Deck 审计问题：实施计划

## Checklist

1. 在 `src-tauri/src/inventory.rs` 增加 owner-aware Agent Root Artifact 过滤，并将校验失败项移入单一 `attention_entries` 判别联合；修正健康链接内容校验错误被映射成 `BrokenLink` 的问题，确保 `external_installations` 只包含有效 External，并添加最小回归测试。
2. 在 `src-tauri/src/diagnostics.rs` 为 diagnostics report 增加统一 `attentionCount` 和裁剪后的需处理条目，并验证领域分类完整且不导出文件内容。
3. 在 `src/api.ts` 同步 Invalid Installation Candidate 与 Unexpected Agent Root Entry DTO；在 `src/App.tsx` 保留非重复的完整 diagnostic path、拆分有效 External 和 attention summary，并把 inventory error 改为原始 payload 延迟本地化，使 `refresh` 不依赖 locale catalog。
4. 在 `src/SettingsDialog.tsx` 展示全部需处理详情；在 `src/i18n.ts` 为中英文添加 attention、Management Scope 与启用筛选范围文案，筛选激活期间始终显示后者；复用现有 `managed/external` state 枚举。
5. 把 `jsdom` 加入 devDependencies 并添加单个最小 `App` 交互测试文件；复用 React DOM，不引入 Testing Library。
6. 搜索 `.DS_Store`、`.system`、`path: null`、`enabledFilter`、`copy.errors`、diagnostics report 的所有相关路径，确认没有并行逻辑遗漏。

## Validation

```bash
npm run format:check
npm run lint
npm run typecheck
npm test -- --run
npm run build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
```

完成自动化检查后，用当前 macOS Skill Deck 实机确认：

- `.DS_Store`、`.system` 不再出现在列表和 external count。
- 一个内部链接无效项显示具体 offending path。
- 启用筛选显示 Managed-only 范围说明。
- 中英文切换不出现 inventory loading/reset。

## Risk and Rollback Points

- `inventory.rs` 过滤发生在递归校验之前；测试必须证明只影响 root-entry，避免修改包内指纹语义。
- `App.tsx` 错误 state 类型变化会影响 loading/error 分支；先完成对应交互测试再调整渲染。
- 若 DOM 测试需要大量模拟，停止扩展测试基础设施，保留一份覆盖三个行为的单文件测试，不引入第二个测试框架。
- 任一步导致 DTO 或持久状态格式变化时回到规划；当前设计不授权这类扩展。
