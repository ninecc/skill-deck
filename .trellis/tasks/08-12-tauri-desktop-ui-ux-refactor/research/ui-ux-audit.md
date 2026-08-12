# Skill Deck UI/UX Audit

## Top Issues

### P0

1. **[macOS] [Windows] [Linux] [Accessibility] [Tauri] Native command surface is absent.** `src-tauri/src/lib.rs` registers commands but no application menu or menu-event bridge; `src/App.tsx:302-342` exposes only toolbar controls. Common commands are not discoverable through macOS Menu Bar and have no shared desktop shortcuts.
2. **[Shared] [UX] [Accessibility] Dialog focus lifecycle is incomplete.** `src/App.tsx:645-756` and `src/SettingsDialog.tsx:32-50` use manual backdrop + `role="dialog"` but do not autofocus, contain Tab, handle Escape, or restore trigger focus.
3. **[Shared] [Accessibility] Popover/tabs keyboard contracts are partial.** `src/App.tsx:455-519` handles only file-tree Up/Down and does not return focus on Escape; `src/App.tsx:585-601` lacks tabpanel relationships and arrow-key tab switching.
4. **[Shared] [Accessibility] Operational errors are not consistently announced.** `src/App.tsx:343-348`, `614-626`, and `697` visually render errors, but only the runtime replacement owns a live region.
5. **[macOS] [Windows] [Linux] [Tauri] Skill actions have no native context menu.** Installed rows at `src/App.tsx:416-433` only handle click and Arrow focus, despite Preview/Update/Reveal/Remove already existing as visible actions.

### P1

6. **[Shared] [UX] Toolbar hierarchy is split across unrelated regions.** Update All/Settings/locale are in `src/App.tsx:302-342`, while Find & Install is buried in the Inventory heading at `389-400`; the primary acquisition command is less discoverable than a bulk mutation.
7. **[Shared] [Tauri] Responsive stack is unreachable in the real window.** Tauri minimum width is 760 (`src-tauri/tauri.conf.json:18`), but the list-to-detail stack starts only below 620px (`src/styles.css:699`), so the configured desktop window never exercises it.
8. **[Shared] [UX] Status feedback behaves like a top banner instead of desktop operational state.** `src/styles.css:199-210` inserts transient status above workspace and shifts content; it does not expose ready/count/version context.
9. **[Shared] [UX] [macOS] Utility surfaces look like web drawers.** `.settings-sheet` at `src/styles.css:530-536` is a full-height trailing panel for both Settings and discovery; a bounded modal/sheet is more natural across desktop platforms and improves focus containment.

### P2

10. **[Shared] [Tauri] Critical CSS depends on newer selectors/functions.** `src/styles.css:258`, `581`, `729-732` rely on `:has()` and `src/styles.css:145` relies on `color-mix()` despite macOS 12 WebView support; narrow navigation and theme selection should have explicit state classes/tokens.
11. **[Shared] [UX] Interaction state coverage is incomplete.** `src/styles.css:125-128` changes only border on hover; no shared pressed state or motion tokens exist, while selected/focus/disabled are otherwise well established.
12. **[Shared] [Accessibility] Native form appearance is not told the resolved color scheme.** Theme tokens switch at `src/styles.css:30-88`, but no `color-scheme` is declared, so selects/inputs may mismatch dark and Plum surfaces.
13. **[Shared] [UX] Font source is misleading.** `src/styles.css:2-7` names Inter without bundling it; the actual successful behavior is the system stack and should be explicit/offline.

### P3

14. **[Shared] [UX] Viewer typography can be tightened.** `src/styles.css:441-476` has comfortable reading width/line height but no max text measure, heading rhythm, selection styling, or tabular number roles.
15. **[macOS] Window state persistence remains absent.** This is a HIG improvement, but adding a plugin/manual persistence is deferred until there is a demonstrated product need; it is not worth a new dependency in this focused refactor.

## What Already Works

- **[Shared]** Compact Inventory/Preview master-detail and independent translation panes are appropriate desktop structure.
- **[Shared] [Accessibility]** Semantic controls, accessible icon labels, `aria-selected`, visible focus, native disabled state, reduced-motion and Arrow list navigation exist.
- **[Shared] [Tauri]** Standard window decoration preserves traffic lights and native controls; min size and unrestricted max size are configured.
- **[Shared]** Theme roles, offline SVG icon family, system locale, safe Markdown rendering, typed Tauri boundary and local preference ownership should remain.

## Refactor Direction

- **Window structure:** Keep native titlebar; compact the web command toolbar below it; move runtime feedback to a stable bottom status region; lower the minimum window to 720×520.
- **Navigation:** Preserve master/detail above ~820px and use one state-driven narrow stack below it; never clone pages.
- **Design system:** Extend existing semantic tokens and explicit state classes; remove critical `:has()`/`color-mix()` dependencies.
- **Visual hierarchy:** Promote Find & Install, subordinate destructive/bulk actions, center utility dialogs, reduce web-banner/drawer cues.
- **Interaction model:** Add native menu, shared keyboard commands, row context menu, complete dialog/popover/tab focus contracts.
- **Platform adaptation:** Standard native window everywhere; Meta on macOS, Ctrl/Alt elsewhere; no macOS-only page fork or custom traffic-light layout.

## Skill Validation

- **macos-design-guidelines:** Standard decoration, Menu Bar, Meta shortcuts, compact toolbar, source list, pointer/context menu, full keyboard control.
- **ui-ux-pro-max:** Semantic tokens, AA-oriented contrast, one primary action, consistent states, visible focus, concise hierarchy, no marketing pattern.
- **userinterface-wiki:** 120–180ms state transitions, instant keyboard navigation, opacity/background animation only, reduced-motion, stable hit areas, system typography.

