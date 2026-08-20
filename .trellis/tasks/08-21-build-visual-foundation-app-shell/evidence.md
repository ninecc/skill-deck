# Child 1 implementation evidence

## Build size

Production Vite output before Child 1:

| Asset | Raw | Gzip |
| --- | ---: | ---: |
| CSS | 23.35 kB | 5.38 kB |
| JavaScript | 371.88 kB | 114.37 kB |

Production Vite output after Child 1:

| Asset | Raw | Gzip | Delta |
| --- | ---: | ---: | ---: |
| CSS | 23.56 kB | 5.42 kB | +0.21 kB raw / +0.04 kB gzip |
| JavaScript | 374.36 kB | 114.74 kB | +2.48 kB raw / +0.37 kB gzip |

`src/icons.tsx` has twelve explicit Lucide virtual imports behind the existing
thirteen-name domain adapter (`download` intentionally aliases the same glyph as
`update-skill`). The production output contains static SVG component code, not
the Lucide collection or an Iconify runtime.

The final `dist` scan found none of:

- `api.iconify.design` or any `iconify` runtime string
- `SKILL_DECK_CANONICAL_REVIEW_FIXTURE`
- `fixture-skill-`, `shell-ready`, `review.html`, or the canonical source payload

## Deterministic browser matrix

The dev review entry was exercised in a real Chromium layout engine. Every row
resolved the requested scenario and filled the viewport without document-level
horizontal or vertical overflow.

| Theme | Scenario | 1180×800 | 720×520 |
| --- | --- | --- | --- |
| Dark | Ready Preview | Pass | Pass |
| Dark | Startup loading | Pass | Pass |
| Dark | Empty Inventory | Pass | Pass |
| Dark | Long content pressure | Pass | Pass |
| Dark | Simplified Chinese pressure | Pass | Pass |
| System | Ready Preview | — | Pass (resolved dark on test host) |
| Light | Ready Preview | Pass | Pass |
| Sand | Ready Preview | — | Pass |
| Plum | Ready Preview | — | Pass |

At 720×520, measured application geometry was:

- Toolbar: 720×44
- Inventory: 144×446
- Preview: 576×446
- Status: 720×30
- Document scroll extent: exactly 720×520

The console contained no errors or warnings. The wide and compact Dark Ready
frames were captured for stage review but are not committed, per the approved
binary-evidence policy.

The full matrix above was rerun after the independent check changed the global
minimum height. Every wide case measured exactly 1180×800 and every compact
case exactly 720×520 with matching document/body scroll extents and no console
warnings or errors.

## Native macOS smoke

- The default 1180×800 configured launch rendered in the real Tauri WebView
  with macOS system-provided title bar/window controls; no React title bar or
  traffic-light simulation appeared.
- The live window was resized to the configured 720×520 content minimum. The
  captured outer frame was 753×545 pixels including native decoration and the
  WebView switched to the approved compact shell: 144px Inventory plus 576px
  Preview, icon-only secondary toolbar commands and persistent Status.
- The real runtime reached Ready with the local CLI/inventory and the compact
  content remained scrollable without page-level overflow.
- The temporary local launch-size edit was restored. `tauri.conf.json` has no
  remaining diff and still declares 1180×800 with a 720×520 minimum.

## Remaining acceptance gates

- Final child commit/archive after review corrections, if any

## User visual approval

The labeled wide/compact Dark frames, Loading, Empty, Chinese pressure and
System/Light/Sand/Plum representative frames were presented after the final
check and matrix rerun. The user explicitly approved Child 1 without requesting
additional corrections.
