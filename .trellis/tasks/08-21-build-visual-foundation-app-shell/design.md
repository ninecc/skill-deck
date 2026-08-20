# Technical design

## Tokens

Keep the existing CSS custom-property mechanism but normalize it into primitive,
semantic and component layers. Theme selectors override primitives/semantics;
shell components consume only semantic roles and component dimensions.

## Offline icons

Use `unplugin-icons` with `@iconify-json/lucide` during the Vite build. Import
only the approved Lucide glyphs and map them behind the existing typed `Icon`
API. The runtime receives static React SVG components and has no Iconify client,
API loader or unresolved icon-name string. CSS normalizes icons to 14/16 px,
`currentColor` and the design's 1.5 stroke.

These two build-time development dependencies are explicitly approved.

## Shell

Retain `App` as state/command orchestrator. Restructure markup and CSS only as
needed for the Pencil hierarchy: application toolbar, two-pane workspace,
inventory list, Preview document and status bar. Preserve semantic headings,
native buttons, `aria-selected`, focus states and explicit grid areas.

This is an incremental migration with strict Pencil fidelity, not an `App`
rewrite. Intentional visual deviations require review.

## Review entry

Add a dedicated dev/review entry that installs typed mocked IPC before mounting
the application. Keep it outside the production entry graph. Initial scenarios
cover startup/shell/ready/empty states needed by this child; later children add
their states without changing production behavior.
Use Pencil's canonical sample content plus separate long/empty/Chinese stress
fixtures. Retain the documented entry as developer tooling after delivery.

Tauri keeps native decorations on Windows, macOS and Linux. Exclude the Pencil
macOS title bar from React layout and comparison; start at Application Toolbar.
Use 1180×800 and 720×520 as WebView content viewport targets.
Keep 720×520 as the minimum. Accessibility and native platform behavior override
pixel fidelity when they conflict, with the variance documented for review.
Apply shared tokens/base controls to unfinished surfaces so intermediate commits
remain visually coherent without prematurely rebuilding their structures.

Use behavior tests plus fixed-size manual/Pencil comparison. Do not introduce a
pixel-baseline framework or commit review screenshots.
Measure production bundle size before/after and verify on-demand imports do not
pull the full Lucide dataset.
Also scan production output for review-entry names, fixture identifiers and
canonical fixture payloads.
Capture the exhaustive shell matrix through this entry and run representative
native Tauri shell smoke at each approved size.

## Compatibility and rollback

Later transient surfaces continue to render during this child, even if they
retain old styling. Keep changes separable into dependency/icon work, token work
and shell work so any build or layout regression can be rolled back locally.
