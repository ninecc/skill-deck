# Reviewable visual directions

Status: proposals only. Neither direction is an Approved Visual Direction until
the user explicitly approves it and names UI and platform scope.

Direction B was selected on 2026-08-13 for high-fidelity refinement. Direction
A is retained only as comparison history; it is no longer being refined. This
selection is not yet an ADR-0016 production approval.

The refined B review surface now uses exact 1180×800 and 720×520 frames, light
and dark semantic mappings, a scope-separated command hierarchy, two-line
Inventory identity, provenance typography, refined document rhythm, explicit
focus/selection separation, and bounded Settings/Install/Remove dialogs. Its
state selector covers the full approval set. It remains throwaway planning
evidence outside `src/`.

The latest visual-detail pass follows the user's “extremely restrained
professional tool” preference: low-chroma graphite surfaces, quiet separators,
monochrome 1.5 px icons, neutral command fills, and accent color restricted to
focus, selection rails, and small state signals. Semantic success, warning, and
danger colors stay independent. This resolves the palette/icon preference but
does not satisfy the ADR-0016 approval gate.

“Restrained” is not interpreted as completely monochrome. A subsequent detail
pass adds sparse Theme Accent cues to one stroke of the product mark, the Find
& Install icon, the selected Inventory rail/wash, active-tab underlines, the
code-block edge, and contextual state symbols. Large surfaces and ordinary
commands remain neutral, so these details support orientation without becoming
decoration.

The selected-Skill workspace remains a two-pane layout: Inventory and Preview.
The user explicitly rejected a persistent third pane in favor of a floating
file tree. A compact current-file control anchors a 244 px popover with root
identity, file count, 30 px file/folder rows, disclosure chevrons, and the same
selected-rail grammar as Inventory. Source and install path remain readable
provenance rather than breadcrumb arrows competing with the file control. The
header keeps two explicit command scopes: Translate/Reveal, then Update
Skill/Remove. At the 720×520 minimum width, the popover is width-bounded and
commands remain on a compact, non-wrapping second row. Esc closes the tree and
restores focus to its trigger. This is presentational adaptation over the
existing command and selection contracts.

Viewport scope is now explicit: 1180×800 is the default and 720×520 is the
supported compact-desktop floor. At 720×520, Inventory and Preview remain a
two-pane desktop workspace; the header may use a second command row and the
file tree remains a bounded popover. Layouts below 720 px are out of scope, and
no mobile-style single-pane navigation or touch-first adaptation is proposed.
The user explicitly confirmed this viewport strategy on 2026-08-13.

An interactive visual review surface was produced outside the repository in the
thread-owned visualization directory. It shows both directions with identical
content and selectable 1180×800 / 720×520, Preview, Translation, Settings,
Find & Install, Remove, startup loading, runtime failure, empty Inventory and
Preview error scenes. It is throwaway review evidence, not an implementation
source or project visual authority.

Both proposals preserve the exact behavior inventory in `prd.md`. The diagrams
show layout and hierarchy, not new commands or lifecycle concepts.

## Direction A — Indexed Workbench

**Design proposition:** Skill Deck becomes a compact three-stage index — Skill,
file, document — so the user's physical path through an installed Skill is
always visible and every command is anchored to the object it affects.

This is specific to Skill Deck because its core object is not a generic record:
an upstream-CLI Inventory entry resolves to a local file tree, then to a
read-only document that may be revealed or translated. The signature is a thin
**scope rail** connecting the selected Skill, selected file and Preview title;
it is structural wayfinding, not decoration.

### 1180×800 wireframe

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Skill Deck   [Find & Install]   [Refresh Inventory]      [Update All] [⚙]  │
├──────────────────────┬──────────────────┬────────────────────────────────────┤
│ INSTALLED 48         │ FILES            │ ask-matt / SKILL.md                 │
│ [Filter…           ] │ ▾ ask-matt       │ [Translate] [Reveal] [Update] [⋯] │
│                      │   SKILL.md        ├────────────────────────────────────┤
│ ▌ask-matt            │   README.md       │ ORIGINAL                 EN → 中文 │
│  mattpocock/skills   │ ▸ references      │                                    │
│  ~/.agents/…         │ ▸ scripts         │ # Ask Matt                         │
│  banner-design       │   icon.png        │                                    │
│  brand               │                  │ Readable Markdown body, max 72ch…  │
│  code-review         │                  │                                    │
│  …                   │                  │                                    │
├──────────────────────┴──────────────────┴────────────────────────────────────┤
│ ● Ready     48 installed · CLI 1.5.22                          [Details]     │
└──────────────────────────────────────────────────────────────────────────────┘
   278 px                184 px              flexible Preview (min 620 px)
```

When Translation is on, the Preview column alone splits 1:1; the Skill and File
indexes stay fixed. At widths below 1040 px, Files collapses back into the same
path-button tree popover used today, protecting document measure.

### 720×520 wireframe

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Skill Deck      [Find] [Refresh] [Update All] [⚙]                          │
├──────────────────────────────────────────────────────────────────────────────┤
│ ‹ Installed  /  ask-matt  /  [SKILL.md ▾]                                  │
│ [Translate] [Reveal] [Update] [Remove…]                                     │
├──────────────────────────────────────────────────────────────────────────────┤
│ ORIGINAL                                            EN → 中文                │
│                                                                              │
│ # Ask Matt                                                                  │
│ Readable document content; file tree opens as anchored popover.             │
│ Translation uses Original | Translation tabs, never squeezed columns.       │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│ ● Ready                                      48 installed · CLI 1.5.22      │
└──────────────────────────────────────────────────────────────────────────────┘
```

Inventory mode replaces the content region with a full-width compact list;
Alt+Left / Meta+[ and Esc return to it exactly as today. This is narrow desktop
navigation, not mobile navigation.

### Surface plans

- **Main window:** 48 px shared toolbar, 30 px status bar, hairline-separated
  indexed workspace. Refresh Inventory stays with Inventory scope; Update All is
  visually separated by a divider and batch badge; Update Skill lives only in
  the selected-Skill command group. Refresh uses a circular-arrow icon, Update
  uses a downward package arrow, and Update All adds a stacked-package glyph.
- **File tree:** persistent 184 px index only when ≥1040 px and a Skill is
  selected; directories and files use 28 px rows, full paths remain in
  accessible names and hover/focus tooltip. Collapse is deterministic, not a
  user preference. A user-resizable index is deferred because it adds persisted
  layout behavior not required to solve the problem.
- **Settings:** 680×560 utility sheet with a 148 px left section index:
  General, Appearance, Translation, Installation, About. Only the right pane
  scrolls. Agent targets live under Installation with Automatic / Choose
  specific mode, search, selected count and a bounded virtual-looking (but
  ordinary DOM) checkbox list. A top-right “Saved” indicator confirms immediate
  fields. Translation Proxy shows “Not applied” after edits and “Apply proxy”;
  footer text says “Changes save immediately except Translation Proxy,” with a
  single Close button.
- **Find & Install:** 700×560 sheet with Search and From source as two tabs.
  Search results are dense rows (name, source, install count, Install). The
  source tab contains the direct source field and current install options
  summary. Switching tabs does not clear either draft. Running/unresolved/error
  state remains in the active tab and mirrors to the global status bar.
- **Remove confirmation:** 420 px alert-style dialog; Skill name and root path
  are selectable text, Cancel receives initial focus, Remove uses independent
  danger semantics, and the running-command note remains visible.
- **Status bar:** stable left status dot + sentence, right Inventory/CLI facts.
  Partial is amber with “Review”; error is red with the recovery action; success
  is green and never uses Theme Accent. Diagnostics opens upward and never
  covers the current status sentence.
- **Runtime/loading/empty/error:** startup chrome remains visible. Runtime
  loading uses a compact centered progress row; failure uses a bounded message,
  exact next step and Retry. Empty Inventory occupies the Inventory index and
  offers Find & Install; no match stays local with “Clear filter.” Preview and
  translation failures replace only their respective pane. Unresolved install
  keeps the dialog open with Retry and preserved input.

### Tokens

Three layers are used: primitives (raw values), semantics (surface/text/status),
then components (toolbar, row, tree, viewer, dialog). Component CSS never refers
directly to raw colors.

| Token family | Proposed values |
|---|---|
| Light color primitives | `mist-50 #F7F8F9`, `mist-100 #EEF1F3`, `mist-300 #CBD2D7`, `ink-900 #182126`, `ink-600 #5F6D75`, `teal-700 #216B72` |
| Dark color primitives | `coal-950 #14191C`, `coal-900 #1B2226`, `coal-750 #303B40`, `mist-50 #F1F5F6`, `mist-400 #A9B5BA`, `teal-400 #65B8BD` |
| Semantic colors | `surface-window`, `surface-index`, `surface-document`, `text-primary`, `text-muted`, `theme-accent`; independent `focus #0A6FDB`, `success #2E7D4B`, `warning #9A6700`, `danger #B53C3C` with dark variants |
| Typography | system UI 13 px/18 body; 12 px/16 metadata; 11 px/14 section label with 0.06em tracking; 15 px/20 Skill title; local monospace 12.5 px/19 for paths/code; Markdown 15 px/24, H1 24 px/30 |
| Spacing | 2, 4, 6, 8, 12, 16, 24 px; related controls use 6–8, groups 16–24 |
| Density | controls 30 px default, icon buttons 30×30, index/tree rows 28 px, Inventory rows 48–54 px |
| Radius | 3 px small, 5 px control, 8 px popover/dialog; no radius on structural panes or rows |
| Border/elevation | 1 px semantic separator; inset selection rail; only popover `0 8px 24px rgb(0 0 0 / .18)` and modal `0 18px 48px rgb(0 0 0 / .24)` elevate |
| Focus | 2 px high-contrast ring + 2 px offset; list focus uses ring/inset marker distinct from selected fill; `:focus-within` on composite search; theme tile has separate checked mark and focus ring |
| Motion | hover 120 ms, small state 180 ms, popover exit 140 ms; no list/keyboard animation; spinner only for indeterminate work; reduced motion removes rotation and all nonessential transitions |

The five persisted theme values map at the semantic layer. `system` resolves to
light/dark, `light` and `dark` use the tables above, while `sand` and `plum` get
new accessible semantic mappings without changing stored values. Native
controls keep System Accent; custom selection uses Theme Accent.

### Platforms, accessibility and behavior impact

- **Shared:** identical DOM hierarchy, command labels/IDs, semantic tokens,
  density, breakpoint behavior, focus order and state placements.
- **macOS:** native menu remains primary discovery; system font resolves to SF;
  Cmd labels and traffic lights remain native. No simulated vibrancy or custom
  title bar.
- **Windows:** Segoe UI resolution, Ctrl labels, application menu convention and
  native form controls. Avoid macOS-only glyph metaphors.
- **Linux:** system sans/monospace fallback, Ctrl labels, application menu and
  stronger 1 px separators for variable WebView/theme rendering.
- Keyboard order is Toolbar → Inventory/Files → Preview commands/content →
  Status actions. Arrow keys remain local to lists/tree, tabs use left/right,
  Esc closes transient/modal surfaces, and focus restoration is unchanged.
  Every icon has a localized accessible name; status does not rely on color;
  WCAG AA contrast and 200% zoom at 720×520 are implementation gates.
- **Behavior-contract impact:** zero intended. The only interaction proposal
  requiring explicit UI approval is the conditional persistent Files index; it
  projects the current file tree and does not change tree data or selection.

### Cost, risks and rollback

- Estimated implementation: high-medium. Main costs are extracting layout
  regions, responsive collapse of Files, Settings section navigation and new
  focus tests.
- Primary risk: the persistent third column can make Translation cramped around
  1040–1180 px and introduces more responsive states. Mitigation is the explicit
  1040 px collapse and unchanged narrow tabs.
- Rollback boundary: CSS/token work, presentational component extraction and
  Settings/file-tree projection. `commands.ts`, API DTOs, Rust commands,
  preference schema and lifecycle logic remain untouched and independently
  revertible.

## Direction B — Quiet Reading Desk

**Design proposition:** Skill Deck becomes a calm two-pane reference desk: a
dense installed shelf on the left and a generous document workspace on the
right, with commands grouped by scope and all transient complexity appearing
only when requested.

This is specific to Skill Deck because most time is spent reading and comparing
the files of a locally installed Skill, not managing a dashboard. The signature
is a compact **provenance line** under the selected Skill name — source → root →
file — that preserves technical identity while the document remains visually
quiet.

### 1180×800 wireframe

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Skill Deck   [Find & Install] [Refresh Inventory]      [Update All] [⚙]    │
├──────────────────────────┬───────────────────────────────────────────────────┤
│ Installed Skills     48  │ ask-matt                              [Update] [⋯]│
│ [Filter name/source/path]│ mattpocock/skills → ~/.agents/… → SKILL.md [▾]   │
├──────────────────────────┼───────────────────────────────────────────────────┤
│ ask-matt                 │ [Original] [Translation]     [Translate] [Reveal]│
│ mattpocock/skills        │                                                   │
│ ~/.agents/…              │          # Ask Matt                               │
│──────────────────────────│                                                   │
│ banner-design            │          Readable Markdown at 72ch.               │
│ brand                    │          Code remains horizontally scrollable.    │
│ code-review              │                                                   │
│ …                        │                                                   │
├──────────────────────────┴───────────────────────────────────────────────────┤
│ Ready · 48 installed · CLI 1.5.22             Last action: Inventory loaded │
└──────────────────────────────────────────────────────────────────────────────┘
   300 px                    flexible document workspace
```

Translation at ≥900 px becomes two reading columns inside the document region.
The file tree remains an anchored path popover, so Preview retains maximum
width and implementation stays close to the proven interaction model.

### 720×520 wireframe

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Skill Deck            [Find] [Refresh] [Update All] [⚙]                    │
├──────────────────────────────────────────────────────────────────────────────┤
│ Installed Skills 48      [Filter name, source, or path…]                    │
├──────────────────────────────────────────────────────────────────────────────┤
│ ask-matt                                                                    │
│ mattpocock/skills                                                           │
│ ~/.agents/skills/ask-matt                                                   │
│──────────────────────────────────────────────────────────────────────────────│
│ banner-design                                                               │
│ brand                                                                       │
│ code-review                                                                 │
│ …                                                                           │
├──────────────────────────────────────────────────────────────────────────────┤
│ Ready · 48 installed · CLI 1.5.22                                           │
└──────────────────────────────────────────────────────────────────────────────┘
```

After selection, the same region becomes Preview with a clear “‹ Installed”
back control, provenance line and Original/Translation tabs. This preserves the
current narrow master/detail behavior and shortcut contract.

### Surface plans

- **Main window:** stable two-pane structure at wide widths. Inventory metadata
  uses two lines rather than one concatenated ellipsis: source first, root path
  second, each with middle-ellipsis where supported and full tooltip/accessibility
  text. Toolbar uses text+icon for Find, Refresh and Update All; selected-Skill
  actions live in the Preview header. The generous Preview surface is the
  dominant visual mass.
- **File tree:** current anchored popover is refined into a 360–440 px outline
  with a sticky selected-path header, type/size metadata for unsupported files,
  full-path tooltip and 28 px rows. No new persistent pane or resize state.
- **Settings:** a 700×540 preference sheet with a compact top tab strip:
  Appearance, Language, Translation, Installation, About. Each tab is a single
  non-scrolling pane except Installation, where the Agent list scrolls inside a
  bounded region. Immediate-save tabs show a quiet “Saved automatically” line;
  Translation Proxy alone shows draft state and Apply. The footer always says
  “Close”; it never implies Cancel or Save All.
- **Find & Install:** one 680×560 sheet with Search at the top and results in the
  main scroll region. “Install from source” is a clearly separated disclosure
  section at the bottom, expanded on keyboard/click without changing data or
  command semantics. Unresolved installation pins a recovery strip above the
  footer so input and Retry remain visible.
- **Remove confirmation:** same safe semantics as A, but rendered as a compact
  centered confirmation with no extra iconography. Name/path provide context;
  Cancel first, Remove last.
- **Status bar:** one sentence rather than a dashboard of badges. Left begins
  with semantic status text; Inventory/CLI facts follow; the most recent action
  result occupies the right and truncates only after its recovery action.
  Diagnostics is an anchored popover.
- **Runtime/loading/empty/error:** every content-owning region has the same
  three-part state grammar: short title, factual sentence, one primary recovery
  action. Startup/runtime uses the full workspace; Inventory empty/no-match uses
  the shelf; Preview/translation uses the reading region; install errors use the
  sheet. Loading uses stable reserved space, not skeleton cards.

### Tokens

| Token family | Proposed values |
|---|---|
| Light color primitives | `paper-50 #FBFCFA`, `stone-100 #EFF1EE`, `stone-250 #D2D7D1`, `ink-900 #202622`, `ink-600 #626C65`, `indigo-650 #4B5E91` |
| Dark color primitives | `night-950 #171A18`, `night-875 #202421`, `night-700 #39403B`, `paper-75 #F2F5F1`, `stone-400 #AAB3AC`, `indigo-400 #93A7E3` |
| Semantic colors | `window`, `shelf`, `document`, `text`, `text-subtle`, `theme-accent`; independent `focus #006FE6`, `success #317A4B`, `warning #986500`, `danger #B33A44` with dark variants |
| Typography | system UI 13 px/18 body; 12 px/16 metadata; 16 px/21 selected Skill; local monospace 12 px/18 provenance/code; Markdown 15.5 px/25, H1 25 px/31 |
| Spacing | 4, 6, 8, 12, 16, 20, 28 px; document inset 28–56 px with 72ch measure |
| Density | controls 30–32 px; toolbar 50 px; Inventory rows 58–64 px to expose source and path; status 30 px |
| Radius | 4 px fields, 6 px controls, 10 px modal/popover; structural panes remain square |
| Border/elevation | quiet 1 px separators; selected row uses fill + 2 px leading marker; elevation only for transient surfaces with two neutral shadow layers |
| Focus | 2 px external focus ring; composite input uses `:focus-within`; row focus has inner outline distinct from selected marker; checked theme uses check glyph, never the focus ring |
| Motion | hover 120 ms, disclosure 180 ms, modal opacity 160 ms; keyboard and pane switching immediate; reduced-motion removes transform/rotation and leaves only instant state changes |

The same three-layer token and five-value preference mapping rules as Direction
A apply. Direction B's visual identity comes from document proportion,
provenance typography and quiet separators, not from preserving any historical
Sand palette.

### Platforms, accessibility and behavior impact

- Shared markup, commands, breakpoints, tokens and state grammar across all
  platforms. System fonts resolve naturally; no remote font or imitation-native
  material is introduced.
- macOS keeps complete native menus, Cmd labels and standard traffic lights;
  Windows/Linux keep Ctrl labels and their application-menu conventions.
  Platform form-control differences are normalized only enough for size and
  contrast, not restyled into fake AppKit controls.
- Toolbar commands have visible labels at 1180 and localized tooltips/accessible
  names when compact at 720. Tab/list/tree keyboard patterns, Esc, focus trap and
  trigger restoration remain unchanged. Status uses icon/text/shape, not color
  alone. Paths are selectable and exposed whole to assistive technology.
- **Behavior-contract impact:** zero intended. Unlike A, this direction adds no
  new persistent/collapsible structural region and keeps the existing file-tree
  interaction model.

### Cost, risks and rollback

- Estimated implementation: medium. Main work is token migration, component
  hierarchy, Settings tabs, command grouping, two-line identifier handling and
  state components.
- Primary risk: the calmer document emphasis can understate batch operations or
  make Inventory scanning slower because rows are taller. Mitigation is strong
  scope placement, stable status text and a density ceiling of 64 px per row.
- Rollback boundary: mostly CSS and presentational components. Settings tabs can
  revert independently to the current sequential DOM; command/data/Rust and
  preference schema are untouched.

## Comparison and recommendation

Scores are relative, 1 (weak) to 5 (strong), based on the current behavior and
window constraints rather than appearance alone.

| Criterion | A — Indexed Workbench | B — Quiet Reading Desk | Reason |
|---|---:|---:|---|
| Task completion speed | 5 | 4 | A keeps Skill → file → document simultaneously visible at wide widths; B needs the path popover. |
| State readability | 5 | 4 | A's scope rail and dedicated index/state locations make ownership explicit; B is calmer but more transient. |
| Command discoverability | 5 | 5 | Both separate Inventory, batch and selected-Skill commands and retain menus/tooltips. |
| Information density | 5 | 4 | A fits more navigational context; B spends more vertical space on provenance. |
| Preview readability | 4 | 5 | B reserves the widest, quietest document surface; A may collapse Files to protect measure. |
| Cross-platform consistency | 4 | 5 | B relies on fewer structural adaptations and maps cleanly to all WebViews. |
| Accessibility/keyboard | 4 | 5 | Both can comply; B has fewer focus zones and responsive mode changes. |
| Implementation risk | 3 | 5 | A adds a conditional third pane and more breakpoint behavior; B retains the current two-pane/tree model. |

### Recommendation: Direction B — Quiet Reading Desk

Direction B is recommended for the first production redesign. It gives Preview
the strongest reading surface, fixes Settings hierarchy and focus defects,
clarifies command scope, improves long identifier presentation and creates one
state grammar without adding a new persistent Files pane or resize contract.
That yields the best balance of task speed, status readability, command
discovery, compact density, shared-platform fidelity and reversible
implementation.

Direction A is the stronger choice only if simultaneous file-tree visibility is
valuable enough to accept another responsive seam and less Preview width. If A
is approved, the conditional Files index must be named in the approved UI scope;
approval of colors or Settings alone would not authorize it.

## Required approval format

Please confirm with exactly scoped language such as:

> 批准方向 B；UI 范围为主窗口、Settings、Find & Install、Remove
> confirmation、文件树、状态栏及 runtime/loading/empty/error/translation
> 状态；平台范围为 macOS/Windows/Linux 共享 WebView UI，并保留文档所述原生
> 平台差异。

Vague approval such as “不错”, “继续” or “按建议” does not satisfy ADR-0016.
