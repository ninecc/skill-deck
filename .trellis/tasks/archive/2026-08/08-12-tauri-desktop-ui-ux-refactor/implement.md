# Implementation Plan

1. **P0 — Native commands and shared dispatcher**
   - Add a typed Command Registry/Dispatcher with structured availability reasons, explicit operation kinds and one lifecycle path per command.
   - Build the native Tauri menu as a frontend presentation adapter and synchronize enabled state from the Registry.
   - Route Toolbar, platform menu, DOM shortcuts and Context Menu through the Dispatcher only.
   - Keep predefined Platform Role Commands outside the Registry and all lifecycle mutations mutually exclusive.
   - Add Installed-row native context menu using existing actions.
   - Remove blocking confirms: Update All dispatches directly; Whole-Skill Removal uses the shared confirmation modal.
   - Name the selected-item menu `Skill`; add non-mutating Refresh Inventory through existing `runtime_status`, separate from Update All.
   - Reconcile Refresh selection/file fallback and keep Preview reload errors scoped below Runtime.
   - Run focused React command tests; run Rust fmt/clippy/tests only if the implementation actually changes Rust.

2. **P0 — Accessible temporary surfaces**
   - Replace duplicated manual sheet behavior with one ordinary-DOM ARIA Modal Shell; do not use HTML `<dialog>`.
   - Add initial focus, Tab containment, Escape close and trigger focus return.
   - Complete file popover and translation-tab keyboard/ARIA behavior.
   - Add alert/live semantics to operational errors.
   - Keep Settings live-save with a local Proxy draft; keep failed/partial discovery context and close/locate only on unique success.
   - Enforce one discriminated top-level modal state; dismiss transient surfaces before open and restore focus to trigger/stable fallback on close.
   - Keep Command Lifecycle alive when Install/Remove modal closes; communicate non-cancellation and expose closed-workflow recovery through feedback.
   - Preserve one unresolved discovery context for Status Review, and use strict exact-name/one-changed-skill target resolution without heuristics.
   - Classify feedback from structured target evidence only; keep Update and zero-change direct source outcomes neutral.
   - Add one app-owned transient state with deterministic Escape/modal/native-context-menu dismissal.
   - Add Status Summary + Diagnostics Details popover without history or timers.

3. **P1/P2 — Shared shell, tokens and responsive layout**
   - Promote Find & Install in the toolbar and establish an always-present bottom status container with dynamic low-noise content.
   - Add explicit view-state classes; change narrow stack to ~820px and Tauri minimum to 720×520.
   - Extend semantic state/color-scheme/motion tokens, preserve System Accent on native controls, and remove critical `:has()` and `color-mix()` dependencies.
   - Tighten system typography, hover/active/disabled/selected/focus states and bounded dialog layout.
   - Implement the four-command Toolbar hierarchy, reliable disabled-reason affordances and no overflow/multi-selection UI.
   - Move Language into validated preferences with System Default, explicit override and migration of the legacy locale key.
   - Localize custom native-menu items from Effective UI Locale while leaving predefined roles system-owned.
   - Clear stale Preview on selection, publish tree/file atomically, and encode root/document command availability during loading.
   - Complete fully expanded file-tree keyboard handling and Translate checked/pressed/dynamic-label synchronization.
   - Implement narrow Back scroll/focus recovery without any resize-driven React effect.

4. **P2/P3 — Verification and polish**
   - Add minimal behavior tests for command availability/dispatcher revalidation, native-menu state mapping, shortcuts, modal focus/Escape, context selection/actions, status feedback and locale resolution/migration.
   - Run `npm run format:check`, `npm run lint`, `npm run typecheck`, `npm test -- --run`, `npm run build`.
   - Run Rust fmt/clippy/tests only if implementation changes Rust; native-menu UI state should not require backend edits.
   - Inspect light/dark and 1180/820/720 layouts; verify reduced motion and keyboard-only core flow.
   - Run the current macOS app for native Menu Bar, shortcuts, Context Menu, resize and focus smoke. Record Windows/Linux as unverified runtime platforms and cap their review rating at Good.

## Rollback Points

- Commit-sized logical groups remain separable: native menu, accessibility surfaces, shared styling/layout.
- The only persisted migration is legacy locale → validated UI Language Preference; keep it backward-compatible and idempotent. Other changes are UI-local and reverting their files restores prior behavior.

## Review Gates

- macOS: Menu Bar, Meta shortcuts, traffic lights, compact pointer density, focus and context menu.
- Windows/Linux: standard decoration, Ctrl/Alt shortcuts, native menu/context menu availability and 720px usability.
- WebView: no critical `:has()`/`color-mix()`, no layout animation, reduced-motion honored, nested scroll limited to intentional source/viewer panes.
