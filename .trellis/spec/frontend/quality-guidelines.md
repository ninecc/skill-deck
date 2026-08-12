# Frontend Quality Guidelines

Keep the frontend thin: presentation, confirmation, short-lived Preview state
and validated UI preferences. CLI validation, mutation truth, filesystem reads
and provider access belong in Rust. Do not infer lifecycle state from paths or
diagnostic strings; render refreshed Inventory and structured outcomes.

Core workflow and error strings must exist in both `zh-CN` and `en`. Tests
should target behavior and contracts rather than snapshots of decorative DOM.

Required commands:

```bash
npm run format:check
npm run lint
npm run typecheck
npm test -- --run
npm run build
```
