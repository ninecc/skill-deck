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

For changes to icons or deterministic review scenarios, also inspect the final
`dist/` output. It must contain only the production HTML/assets and must not
contain Iconify API/runtime endpoints, `review.html`, review scenario IDs or
canonical fixture markers/payloads. Exercise owned review scenarios at their
approved viewport sizes in a real layout engine; jsdom behavior tests do not
prove geometry or visual fidelity.
