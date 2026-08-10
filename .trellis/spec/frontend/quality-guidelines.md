# Frontend Quality Guidelines

Keep the frontend thin: presentation, user confirmation, and accessibility.
Validation, ownership, filesystem changes, Git, and durable state belong in
Rust. Do not duplicate Rust rules in form handlers; render structured preflight
results instead.

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
