# Implementation Plan

## 1. Establish the CLI source of truth

- [ ] Add the minimal CLI adapter and structured errors.
- [ ] Detect `node` and `npx`, resolve `skills@latest` once at startup, probe list JSON compatibility, and retain the exact CLI version for all commands in that app session.
- [ ] Decode `list -g --json` into open-string Agent DTOs.
- [ ] Add non-interactive global add/remove/update commands with a shared execution gate, pre-operation Inventory snapshot, post-operation refresh and observed-state outcome.
- [ ] Add search through the official CLI's current JSON search backend, isolated from installed inventory.
- [ ] Pass an exact Skill name for search-result installs; disclose and preserve CLI install-all behavior for a direct multi-Skill source.
- [ ] Add focused Rust tests with fake node/npx executables for argv, exit status, malformed JSON, missing runtime and arbitrary Agent names.

Rollback point: new adapter is additive and old runtime still compiles.

## 2. Replace the product surface

- [ ] Rewrite the TypeScript command boundary around runtime status, installed Skills, search and CLI actions.
- [ ] Replace `App.tsx` with the installed list, independent search/source-install flow, whole-Skill remove confirmation, row Update and confirmed Update All.
- [ ] Simplify Settings to one horizontal Appearance theme picker plus target language, Agent override and automatic/copy install mode; persist preferences in localStorage.
- [ ] Model Appearance as one enum with System (Default), Light, Dark, Sand and Plum; do not retain separate mode/palette state.
- [ ] Preview each theme tile with a small fixed set of representative semantic token colors so adjacent presets remain visually distinguishable.
- [ ] Render System's theme preview as stable Light|Dark split swatches for chrome, content and interaction; do not derive the tile from the currently resolved OS mode.
- [ ] Add semantic token maps for all presets, resolve `system` through `prefers-color-scheme`, and react to system changes without remounting application state.
- [ ] Map selected fill, tag surface/border, control/path surface/border and Markdown code surface/border per theme; add only the minimal missing semantic roles (`tag-border`, `code-border`) and keep danger/warning independent.
- [ ] Select one Iconify collection, verify its license, vendor or import only the used offline icon data, and add no runtime Iconify API/CDN path.
- [ ] Apply one icon size/weight/currentColor contract; keep labels for primary/destructive actions and accessible names/tooltips for icon-only controls.
- [ ] Render Installed selection with semantic selected fill plus accent edge, no visible `Selected` label, while retaining `aria-selected` and an independent focus ring.
- [ ] Initialize with no selected Skill; show a compact action-free detail placeholder on wide windows, keep Inventory on narrow windows, and commit selection only on click or Enter rather than Arrow focus movement.
- [ ] Use behavior labels ending in `(Default)` and show the actual CLI version.
- [ ] Keep Chinese and English catalogs aligned.
- [ ] Add frontend tests for runtime blocking, list/search/actions, arbitrary Agent display, default flag omission, theme resolution/change handling and Settings persistence.
- [ ] Test that all icon-only controls have accessible names and that icons render with networking unavailable in every theme.
- [ ] Use neutral Update completion copy that does not invent a revision status absent from Inventory.

Rollback point: CLI-backed core flows work before deleting old lifecycle modules.

## 3. Add bounded all-file preview

- [ ] Add a preview module that derives roots from current CLI inventory, walks without following links and validates every relative read.
- [ ] Return a complete compact tree with viewer kind, size and unsupported reason.
- [ ] Add bounded text and raster-image reads plus validated file-manager reveal.
- [ ] Add the path-button popover tree and single-pane Markdown/code/text/image/unsupported viewers.
- [ ] Omit a redundant `/` tree row because the path trigger already represents the Skill root; begin top-level entries at `aria-level="1"`, use locally bundled Phosphor folder/file icons plus indentation, retain full paths in `data-path`/accessible names, and add no expand/collapse state.
- [ ] Use `react-markdown` with raw HTML disabled; do not add a code editor or general binary viewer.
- [ ] Test traversal, internal links, invalid UTF-8, size limits, unsupported types, keyboard navigation and no persistent empty file column.

## 4. Add replaceable read-only translation

- [ ] Add one narrow translation module with an anonymous Google Translate implementation; keep replacement behind its exported request/result function without a trait, factory or plugin registry.
- [ ] Keep provider request/response/chunking inside the translation module and mark the reliability ceiling with a `ponytail:` comment.
- [ ] Segment Markdown so frontmatter, fenced/inline code, link URLs and structure bypass translation while prose is translated.
- [ ] Restrict translation commands to bounded Markdown/plain-text content from the preview reader.
- [ ] Add target-language system-locale default, unsupported-locale English fallback and user override.
- [ ] Use the agreed fixed common-language list; do not fetch or mirror Google's complete catalog.
- [ ] Add persistent Google egress copy beside the translation control, desktop two-pane original/translation layout and sub-900 px tabs with independent loading/error state.
- [ ] Keep translation mode active across file selection and automatically translate only newly selected eligible documents.
- [ ] Test UTF-8 chunk boundaries, response order, provider failures, eligibility and zero filesystem writes.

## 5. Delete the parallel manager

- [ ] Delete Managed Library, state, adoption, configuration, lifecycle, revision, Git source, reconciliation, diagnostics and projection modules/tests that no longer have callers.
- [ ] Delete their React dialogs, DTOs, styles and localized copy.
- [ ] Remove now-unused Rust/npm dependencies and packaging checks tied only to the old manager.
- [ ] Update roadmap and Trellis specs so future work follows the CLI-adapter/preview/translation boundaries, not the removed lifecycle model.
- [ ] Confirm old app-data is ignored but not automatically deleted.

## Validation

Run after each major slice, then once across the final tree:

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

Manual checks on the current Mac:

- [ ] packaged app detects PATH-visible Node/npm and reports absent runtime clearly;
- [ ] failed latest resolution disables management/preview, offers Retry and never falls back to a persisted version;
- [ ] list/search/add/remove/update operate only on global Skills;
- [ ] CLI automatic defaults omit Agent/install-mode flags; overrides add only selected flags;
- [ ] file popover, all supported viewers, unsupported state and Finder reveal work;
- [ ] file tree uses local folder/file icons, correct accessible levels/full paths, and no redundant `Folder:` or `Selected` copy;
- [ ] translation opt-in produces a two-pane view and never changes file bytes;
- [ ] application restart retains Settings but no translated content.
- [ ] All theme presets preserve contrast and state parity; switching theme preserves selection, Preview file, translation mode and scroll state.
- [ ] Under simulated OS Light and Dark, the System tile remains the same dual preview and never becomes visually identical to the fixed Dark tile.
- [ ] First launch has no selected Skill; keyboard focus movement alone does not open Preview.

Native release follow-up:

- [ ] Windows verifies inherited PATH behavior and Explorer reveal.
- [ ] Linux verifies inherited PATH behavior and desktop file-manager reveal.

## Review Gates

- [ ] Diff deletes more lifecycle surface than it adds replacement surface.
- [ ] No second ownership, installation, revision or update model remains.
- [ ] No arbitrary path from React reaches filesystem reads or reveal.
- [ ] No CLI human output is parsed into product state.
- [ ] No closed Codex/Claude Agent enum remains at the command boundary.
- [ ] Translation failure cannot block installed inventory or CLI actions.
- [ ] Full frontend/backend quality commands pass.
