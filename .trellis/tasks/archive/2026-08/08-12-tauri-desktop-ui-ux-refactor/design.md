# Technical Design

## Boundaries

- Keep `src/api.ts`, command DTOs, CLI lifecycle and preview/translation data flow unchanged. Extend only the validated local preferences schema for UI Language Preference and migrate the legacy locale key.
- Keep Application Command authority in the React UI layer because its availability derives from current runtime, selection, document, operation and modal state. The installed Tauri Menu API is only a native presentation adapter; backend command DTOs remain unchanged.
- Keep standard decorated windows. No overlay titlebar and therefore no drag-region contract.
- Treat the product as a single-window management utility: no New command, document model, or secondary Settings window.

## Shared UI Architecture

```text
Native window + platform menu
└── React app shell
    ├── compact command toolbar
    ├── master/detail workspace OR narrow navigation stack
    ├── fixed status container with dynamic low-noise content
    └── one modal dialog / one file popover at a time
```

- One `selected` state remains the source for wide detail and narrow push state; CSS receives an explicit app/view class instead of deriving state with `:has()`.
- A typed Command Registry owns command IDs, structured availability results, execution and lifecycle. Menu, shortcuts, Toolbar and Context Menu dispatch IDs only; the Dispatcher revalidates availability immediately before execution.
- Platform roles remain native/predefined menu items and bypass the Application Command Registry by design; their system-owned availability and execution are not mirrored in React.
- A small shared Modal Shell uses ordinary DOM + ARIA rather than HTML `<dialog>` and owns initial focus, Tab containment, Escape, backdrop policy and return focus for Settings/discovery/confirmation across the minimum WebViews.
- Status rendering uses a fixed container: the left slot is derived from active Busy state, latest Outcome, or Ready; the right slot renders stable Inventory/session facts outside the live region.
- Command lifecycle records an explicit operation kind and one current feedback object with summary, severity and optional diagnostics. Starting a command replaces previous feedback and closes diagnostics details; no activity history exists.
- Feedback classification is evidence-only: thrown structured errors are Error; unmet deterministic targets are Partial; Success requires observable proof. Update completion is neutral, and zero-change direct-source completion remains neutral with Review.
- The current concurrency policy keeps all lifecycle mutations mutually exclusive. Operation kind remains explicit for truthful status and later refinement; it does not justify a speculative concurrency matrix now.

## Native Commands

- The three platforms share one Command Model and availability rules. macOS renders the complete system Menu Bar; Windows/Linux render the same commands through their native/application-menu conventions plus the shared Toolbar rather than cloning macOS presentation.
- A frontend native-menu adapter builds Tauri predefined App/Edit/Window roles plus application commands, forwards activation to the Dispatcher, and synchronizes enabled state from the Command Registry. Rust does not duplicate menu availability or execution logic.
- App-specific IDs are grouped under App, Inventory and Skill (`find-installed`, `find-install`, `refresh-inventory`, `settings`, `update-all`, `translate-skill`, `reveal-skill`, `update-skill`, `remove-skill`). Menu events are emitted to the main WebView.
- Find & Install uses `Cmd/Ctrl+Shift+I`; `Cmd/Ctrl+N` remains unassigned because the app does not create a document or window.
- Refresh Inventory uses the existing `runtime_status` command, which reuses the resolved CLI Session and reads current Inventory; it is not Update All and does not clear/re-resolve the Session.
- Refresh reconciliation preserves filter/list scroll. A surviving selection reloads its tree and original path, falling back to `SKILL.md`/first previewable file; a missing Skill clears selection with an Outcome. Preview reload failure is scoped below runtime readiness.
- React listens once, applies current availability guards, and invokes existing UI actions. DOM shortcuts use the same command dispatcher so behavior is testable without a native shell.
- Installed-row context menus use the already-installed Tauri menu API and dispatch applicable Translate/Reveal/Update/Remove Skill Commands; right-click selection itself establishes the Preview, and visible controls remain the fallback.
- Right-click updates the single selected Skill and Preview before opening its menu, without changing keyboard focus or scrolling. There is no persistent context target.
- Update All dispatches immediately when enabled. Whole-Skill Removal opens the shared confirmation modal; the codebase no longer uses blocking WebView confirms.
- Settings edits ordinary preferences directly; its modal owns only close/focus lifecycle. Proxy remains the sole local draft. Discovery keeps its query/result/source context on failed or partial outcomes and closes only after a uniquely resolvable success/already-installed result.
- One `topLevelModal` discriminated state enforces modal exclusivity. Opening it first dismisses transient surfaces; workflow-internal confirmation changes that modal's state rather than nesting a second dialog.
- One `transientSurface` discriminated state owns app popovers. Opening another/modal/native context menu clears it; Escape consumes transient close before modal/navigation. Native context-menu visibility remains system-owned.
- Modal close never implies command cancellation. Install/Remove execution survives Escape/Close, remains observable in Status, and disables duplicate submission. Recovery for a closed workflow is exposed from its feedback rather than forcing a modal to remain open.
- One unresolved discovery workflow context may survive a closed modal. Feedback Review restores it; retry occurs only inside the restored workflow. A new/resolved workflow replaces or clears it, so this cannot become an Activity Log.
- Search-result resolution uses exact requested name against refreshed Inventory. Direct-source resolution uses exactly one `changedSkills` entry that exists in Inventory; all other outcomes remain in the dialog without heuristics or a new chooser.
- Diagnostics Details is a non-modal popover anchored to Status. It renders selectable preformatted text, closes on Escape/outside click/new command/modal open, and never enters the screen-reader announcement in full.

## Styling and Compatibility

- Preserve current token maps; add only semantic state tokens and `color-scheme` mappings.
- Theme Accent styles custom application components. Native controls retain platform appearance/System Accent when supported; success, warning, and danger remain independent semantic roles.
- UI locale becomes part of validated preferences with `system` plus explicit supported BCP 47 locales. Loading migrates the previous standalone locale key once. Effective locale resolves `navigator.languages` then `navigator.language`, applies stable language/region matching and fallback, and listens to `languagechange` only while preference is System Default.
- The native-menu adapter updates localized Application Command labels when Effective UI Locale changes; predefined role labels remain system-owned. Adapter rebuild/update is idempotent and independent from Registry identity/state.
- Selection starts a Preview generation: previous tree/file clear immediately, a scoped loading view renders, and the generation atomically publishes the new tree/file. Root-scoped commands remain available while document-scoped commands return document-loading/unsupported-document.
- File tree rows remain a fully expanded projection: directory rows are non-selectable hierarchy labels and file rows form the keyboard navigation set (Up/Down/Home/End/Enter/Space). Escape restores the path trigger; no expansion state exists.
- Translate is a toggle command derived from Translation Session state. Menu checked/dynamic label and Toolbar pressed state are projections of that command; Hide invalidates the current translation generation immediately.
- Preview-generation failures render inline recovery in Detail and do not modify Runtime or Command feedback; retry advances the generation before reading again.
- The responsive boundary is CSS-only: `min-width: 821px` is master/detail and `max-width: 820px` is the single navigation stack. React has no resize observer, viewport state or layout-triggered data effect.
- Narrow Back restores saved list scroll and focuses selected row, then filter, then Inventory heading. CSS breakpoint transitions never invoke this navigation recovery.
- Inventory remains single-select. The Toolbar has four direct surfaces and no overflow: Find & Install stays labeled, Refresh/Update All labels hide at ≤820px with reliable disabled-reason wrappers, and Settings remains icon-only.
- Replace layout-critical `:has()` with explicit classes/data attributes. Replace `color-mix()` danger border with a token.
- Use `@media (max-width: 820px)` for stack mode and retain 900px translation tabs.
- Transitions are limited to background, border-color and opacity; `prefers-reduced-motion` removes them.

## Compatibility and Rollback

- macOS: standard traffic lights/titlebar remain native; the complete global Menu Bar and Meta accelerators improve discovery.
- Windows/Linux: standard decorations remain; the shared Command Model is presented through native/application menus, Toolbar, Ctrl/Alt shortcuts and Context Menu without forcing macOS menu chrome.
- Validation claims follow evidence: macOS receives a real-app smoke; Windows/Linux remain at static/shared/Tauri/build evidence and cannot rate above Good before real-platform smoke.
- Rollback is file-local: native menu setup can be removed without changing backend commands; UI refactor preserves DTO contracts. The only local migration consumes the legacy explicit locale key into validated preferences.

## Deferred

- Window-state persistence, multi-window, Spotlight/Services/Quick Look, drag/drop and custom vibrancy require product evidence or dependencies and are not part of this refactor.
