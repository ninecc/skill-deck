# Adopt macOS-first release validation

## Goal

Make release claims match available hardware and evidence: macOS is the formally supported release target, while Windows and Linux remain continuously built but explicitly unverified preview artifacts.

## Background

- CI currently runs the full quality gate, builds Windows NSIS, macOS universal DMG, and Linux x86_64 AppImage artifacts on every main push/PR, and uploads all three without support-tier labels.
- The current release checklist predates ADR-0011. It refers to removed Managed Library, Linked Installation, Copy Fallback, projection health, and `projection_contract_smoke` concepts.
- The current product delegates lifecycle operations to one session-pinned upstream Skills CLI. Inventory is returned by `skills list -g --json`; add/update/remove use the upstream CLI, and Preview/translation remain app-owned bounded features.
- The user currently has only a Mac and approved macOS formal support with Windows/Linux build-preview status.
- The available host is Apple Silicon (`arm64`) running macOS 26.5.2, with Node 24.17.0, npm/npx 11.13.0, Codex CLI 0.155.1, and Claude Code 2.1.267.
- `/Applications/Skill Deck.app` is currently absent, so the macOS smoke can use the normal installation path without overwriting an existing app.
- The newest downloadable CI artifacts belong to commit `be84df0`, while the local audited HEAD is `0c18c29`; the existing DMG is therefore stale and cannot establish current-release evidence.
- The local branch contains audited/remediation commits that are not yet on `origin/main`. A current immutable DMG requires pushing a task branch and running CI for that exact ref.

## Requirements

- R1: Define macOS as the only formally supported release platform for the current release policy.
- R2: Keep Windows x64 NSIS and Linux x86_64 AppImage builds in CI as preview evidence, without claiming native usability or support.
- R3: Label Windows/Linux CI artifacts and documentation clearly as preview/unverified while keeping the macOS artifact identity stable for formal validation.
- R4: Rewrite `docs/release-smoke.md` around the current upstream Skills CLI lifecycle and current Application Command model.
- R5: The macOS checklist must cover artifact provenance/hash, universal architectures, signing/notarization/Gatekeeper observations, installed launch, runtime/Inventory, search, inert fixture add/update/remove, Codex/Claude recognition, Preview, Reveal, native menus/shortcuts/context menu, relaunch/persistence, and cleanup.
- R6: Windows/Linux sections must document build-only checks now and the exact native smoke required before either platform can be promoted to supported.
- R7: No checklist step may restore superseded Managed Library/projection concepts, bypass host protections, expose tokens/config contents, or ambiguously delete existing user data.
- R8: Execute the complete macOS native smoke on this host using a CI-produced universal DMG for the exact committed task revision.
- R9: The smoke may temporarily install `/Applications/Skill Deck.app`, populate npm cache, and add/update/remove one uniquely named inert global Skill fixture; it must prove cleanup and preserve unrelated state.

## Acceptance Criteria

- [ ] The support-tier policy is explicit and consistent across CI artifact naming and release documentation.
- [ ] CI continues to build all three packages; Windows/Linux artifacts are unmistakably preview/unverified.
- [ ] The macOS checklist is executable against the current product and distinguishes local diagnostic builds from immutable CI artifacts.
- [ ] The lifecycle smoke uses one collision-resistant inert Skill fixture, verifies positive and negative Codex/Claude recognition, and proves cleanup without touching unrelated Skills.
- [ ] Windows/Linux cannot be described as supported until their platform-specific native checklists pass.
- [ ] Existing automated quality and package-contract gates remain intact.
- [ ] A CI artifact from the exact task revision is installed and the macOS checklist passes, or every failed/blocking step is recorded without overstating support.
- [ ] The smoke evidence records commit, CI run, artifact digest/hash, host/tool versions, architecture, signature/security observations, functional results, and cleanup outcome without sensitive contents.

## Out of Scope

- Dropping Windows/Linux source compatibility or CI packaging.
- Claiming Windows/Linux native support without matching hosts and evidence.
- Redesigning the UI, changing lifecycle ownership, or adding new product features.
- Provisioning new signing/notarization credentials; the smoke records the actual signature/notarization state and treats unmet release requirements honestly.

## Deferred Decision

- On 2026-09-22 the user deferred release work in favor of functional development.
- Do not push a task branch, trigger GitHub Actions, install a DMG, modify `/Applications`, or mutate global Skills until this task is explicitly resumed.
- When resumed, obtain fresh authorization to push/trigger CI, then use the exact CI artifact for the committed task revision. A stale artifact or local diagnostic build cannot establish formal macOS release evidence.
