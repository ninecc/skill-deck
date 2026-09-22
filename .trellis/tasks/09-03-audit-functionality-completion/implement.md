# Functionality completion audit execution plan

## Checklist

1. Reconfirm the authority hierarchy and extract a normalized current-MVP capability inventory from `CONTEXT.md`, accepted ADRs 0011–0016, active frontend/backend specs, and the approved redesign contract.
2. Trace every capability through frontend commands/UI, Tauri DTOs, Rust implementation, preferences/native-menu adapters, tests, and deterministic review scenarios. Record file-and-line anchors in `evidence.md`.
3. Run the complete frontend quality gate:
   - `npm run format:check`
   - `npm run lint`
   - `npm run typecheck`
   - `npm test -- --run`
   - `npm run build`
4. Inspect production `dist/` for accidental review-only assets, fixture markers, or Iconify runtime endpoints as required by the frontend quality spec.
5. Run the complete backend quality gate:
   - `cargo fmt --check --manifest-path src-tauri/Cargo.toml`
   - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
   - `cargo test --manifest-path src-tauri/Cargo.toml --all-features`
6. Perform a privacy-minimized real runtime/Inventory readiness check where supported. Permit normal `npx skills@latest` network resolution and npm cache writes, record only sanitized readiness, version, count, and outcome evidence, and never execute real add, update, or remove operations.
7. Launch the deterministic review entry and inspect representative scenarios at 1180×800 and 720×520, covering startup/loading/empty/error, selection and file preview, translation states, install/update/remove flows, settings, themes, localization, focus/keyboard affordances, and responsive operability. Score functional clarity and operation, not aesthetic taste. Record unsupported or untested states explicitly.
8. Assess native release readiness using current bundle configuration, repository CI/release evidence, and `docs/release-smoke.md`; do not infer Windows/Linux/macOS validation from another platform. Mark current HEAD not release-ready when required proof is stale or missing, while distinguishing that evidence gap from demonstrated functional failure.
9. Produce the final capability matrix, six equal-weight group subtotals, conservative MVP score, hypothetical implemented upper bound, evidence-coverage ratio, separate release-readiness gate, non-aggregated historical R1–R4 milestone view, and prioritized remediation list in `evidence.md`.
10. Cross-check every partial/missing/unverifiable finding against the actual code and tests, and verify the final report satisfies each PRD acceptance criterion.

## Validation of the audit itself

- Every current-MVP capability row has a requirement source and at least one implementation or explicit absence anchor.
- Every executed command includes its exit outcome and useful counts; environmental blockers are not reclassified as product failures.
- Every claimed UI observation names the scenario, viewport, theme, and locale used.
- Functional score arithmetic is reproducible from the matrix.
- Release readiness and historical roadmap progress remain separate from the MVP score.
- `git diff -- src src-tauri package.json package-lock.json` shows no product change attributable to this task.

## Risk and stop conditions

- If a contract conflict remains after applying the agreed authority order, mark the capability not verifiable and surface the conflict; do not invent product intent.
- If a check would require external credentials, network mutation, installation, or publishing, skip it and record the limitation.
- Never execute a real Skill add, update, or remove command against the user's global environment.
- If native packaging or runtime recognition cannot be exercised safely on the current host, rely only on explicit existing evidence and keep the platform gate open.
- If a reported defect would benefit from a fix, create no remediation task or code change without separate user consent.
- A severe finding stops only its affected or potentially mutating check; continue all other safe, read-only checks and place the finding first in the final report.
