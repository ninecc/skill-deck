# Pencil design architecture

## Authority and boundaries

The supplied HTML reference and the approved Direction B artifacts are the
visual authority. Existing product behavior remains the domain authority. This
task writes only `pen-design/skill-deck-2.pen` plus Trellis planning/check
artifacts; it does not edit production UI code.

## Canvas architecture

The root canvas becomes a small ordered library rather than a screen gallery:

1. `00 Library Index` — map of themes, token layers, content sections and
   links/coordinates for review.
2. `01 Shared Masters` — reusable component masters, icon primitives and shell
   anatomy used by every theme.
3. `10 Theme · System` — dual Light/Dark resolution contract and themed output.
4. `20 Theme · Light` — explicit Light token and pattern collection.
5. `30 Theme · Dark` — explicit Dark collection plus dark-resolved full-page
   validation references only.
6. `40 Theme · Sand` — warm neutral token and pattern collection.
7. `50 Theme · Plum` — plum-dark token and pattern collection.

Each theme is a large, clipped, top-level collection frame with consistent
internal regions: header/identity, Tokens, Page Flow, Components, Icons,
Patterns & States, Responsive Composition and optional Reference Compositions.
The layout grows left-to-right within a theme and top-to-bottom across themes.

The existing sixteen roots are directly reorganized: reusable foundations move
to Shared Masters, the flow map becomes the Page Flow source, dark-resolved
screens move into Dark Reference Compositions, and the existing Light proof
`Zraur` moves into Light Reference Compositions. They are not deleted or
allowed to remain as unrelated roots. Foreign-theme references are forbidden;
only System may present Light and Dark together as explicit resolution branches.

## Token model

Pencil variables use a three-layer architecture across the five theme values:

1. **Primitive** — graphite/accent/status values, spacing, type scale, radius,
   shadow, motion and icon metrics.
2. **Semantic** — window/chrome/panel/document/control/code surfaces; primary,
   muted and separator content; accent/focus/emphasis actions; success/warning/
   danger feedback.
3. **Component** — button, search/input, Inventory row, file row, tab, status,
   popover, dialog and theme-control assignments and states.

Pencil variables provide semantic roles for all themes:

- surfaces: window, chrome, panel, document, control, code, scrim
- content: text-primary, text-secondary, separator, separator-strong
- actions: accent, focus, emphasis, emphasis-text
- feedback: success, warning, danger
- numbers: spacing 4/8/12/16/20/24/32, radii 4/6/8/10, control heights,
  and icon sizes
- strings: system-like sans and monospaced font families

The values are sourced from `src/styles.css`: Light and Dark graphite mappings,
Sand warm neutrals, and Plum dark surfaces/accent. System documents resolution
to Light or Dark rather than creating a fifth fixed palette. Theme axes are
used on specimens and component instances wherever Pencil supports them.

## Component model

Reusable components cover:

- button variants: ghost, emphasis, danger, icon-only
- search field and compact current-file trigger
- Inventory row: default and selected descendants
- segmented/document tab
- file row: file, folder, selected
- status item and state-message block
- dialog header/footer shell and theme tile

Mac desktop density wins over generic touch sizing: controls are 28–32 px high,
with icon-only hit-area intent noted in context rather than rendered as giant
buttons. Icons use one local Pencil-supported library and a consistent optical
size/stroke.

## Theme section contracts

### Page Flow

Use compact, labeled flow nodes and shell thumbnails for Preview → file popover
→ Translation; global commands → Settings/Install; Remove confirmation; and
startup/empty/error recovery. Connect causes and dismiss/retry paths. Do not
wrap every node in a full 1180×800 app shell.

### Components and icons

Shared masters define anatomy; each theme board shows themed instances and a
state matrix. Component families are commands, inputs, Inventory/files, tabs,
status/feedback, dialogs and theme controls. Icons are grouped by function and
show 14/16 px optical size, consistent stroke and semantic role.

File navigation is represented by a `File Tree` family rather than isolated
flat rows. Its anatomy is compact icon Trigger → Popover → Root header →
hierarchical rows. The Trigger is an icon-only square button with an accessible
tooltip/label; it does not repeat the current filename or include a chevron.
At page level, the Trigger is the leading control in the Skill identity row,
immediately followed by the Skill title with a compact 8–12 px gap. This local
group is left-aligned in wide and compact headers rather than split across the
header as unrelated controls.
File rows contain spacer + file icon + basename and never have a disclosure
control. Folder rows contain disclosure + folder icon + folder name and define
collapsed/expanded variants. Children use 16 px indentation steps. The final
composition uses compact 28–30 px rows and follows true parent/child order.

Selection uses both an accent rail/wash and text/icon treatment; focus remains
a separate ring in the Row States matrix. In the Final Composition, keyboard
focus follows selection to avoid presenting two unexplained current items.
Long names stay on one line with ellipsis; full paths are available only through
a tooltip specimen, not a persistent footer. The root header shows only Skill
name and file count, without the installed path. File counts exclude folders
and must match the sample data.

The File Tree documentation is split into five labeled groups: Anatomy, Row
States, Indentation, Long-name Behavior and Final Composition. State specimens
may use synthetic examples; Final Composition must contain no duplicated row,
path-as-label or semantically impossible file chevron.

The page-level document identity is intentionally sparse: Skill title plus the
current-file trigger. Owner/repository breadcrumbs and the installed path are
removed from this header because they repeat context already available from
the Skill and File Tree surfaces.

The previous `Tab/Active` master is repurposed as a non-interactive
`Translation Column Header`. Translation comparison uses two persistent columns
with a vertical separator and small labels local to each content pane. There is
no separate full-width comparison header, tab underline, selected tab,
segmented control or show/hide affordance because the layout already
communicates Original versus Translation.

### Patterns & States

Show isolated popover, translation, Settings/Install/Remove, loading, empty,
runtime failure and Preview failure specimens with their recovery/dismissal
actions. These specimens communicate behavior without redundant chrome.

The translation specimen must show the two columns directly, using only local
pane labels and no global comparison header. The file-popover specimen must use the new hierarchical File
Tree component and demonstrate at least one expanded folder, one collapsed
folder, a selected file and keyboard focus distinct from selection.

### Responsive Composition

Show one 1180×800 two-pane shell and one 720×520 compact two-pane shell per
theme as reduced but legible compositions. Dark additionally owns the existing
fourteen high-fidelity screens as a reference-validation appendix.

## Verification strategy

- After each board/screen batch, query Pencil bounds and warnings before
  clearing placeholders.
- Capture screenshots for the Library Index, each theme token board, each
  theme's component/icon/pattern regions, representative responsive shells and
  the Dark reference appendix.
- Review against a checklist: hierarchy, clipping, alignment, 4/8 rhythm,
  readable measure, complete three-layer theme mappings, focus/selection
  distinction, icon consistency, modal scrim, action priority, recovery paths,
  and the absence of scattered full-page roots.
- Fix existing nodes directly; do not delete completed boards merely to revise
  them.

## Rollback

The rollback boundary is the single target `.pen` file. Because the file starts
empty and is a new untracked design artifact, an unsatisfactory generation can
be replaced only before it becomes user-reviewed work; after review begins,
all changes must be direct updates preserving existing node identity.
