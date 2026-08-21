# React Component Guidelines

Components are functions with typed props. Prefer semantic HTML and native
controls before custom widgets; `App.tsx` uses real buttons, links, labels, and
a select so keyboard behavior exists without extra code.

## Accessibility

- Every icon-only or brand link needs an accessible name.
- Decorative marks use `aria-hidden`.
- Visible sections connect headings with `aria-labelledby` where needed.
- Preserve `:focus-visible`, reduced-motion behavior, readable contrast, and
  the 720×520 minimum desktop-window layout in `styles.css`. Widths below 720px
  are not a mobile-product target.
- A disabled or unavailable domain action must expose the reason in text, not
  only color.
- Modal dialogs are mutually exclusive, trap focus, close on Escape when safe,
  and restore focus to their trigger or a stable fallback when that trigger was
  removed by the confirmed action.

Task dialogs with persistent local tabs use the tabs as their single navigation
surface. A footer may expose a Pencil-approved shortcut into a secondary tab,
but the destination tab must not add a reciprocal navigation action: that reads
like a competing primary command. While a non-cancellable operation is active,
the footer replaces navigation shortcuts with localized progress/safety copy.

```tsx
{operation === "install" ? (
  <small>{copy.commandContinues}</small>
) : tab === "search" ? (
  <button onClick={() => setTab("source")}>{copy.installFromSourceAction}</button>
) : null}
```

Destructive dialogs initially focus the safe action. After confirmation removes
the trigger, restore focus to a stable owning heading instead of a detached DOM
node. Recovery states keep their actions inside the affected pane: Empty exposes
Find & Install, while Preview failure exposes both Retry and Reveal File through
the shared Application Command dispatcher.

Settings uses a fixed three-row dialog grid: header, five-section navigation and
footer remain stationary while only the content viewport scrolls. The approved
wide dialog is 720×548; at the 720×520 application minimum it is 684×496 with
18px horizontal and 12px vertical insets. Do not let a long Agent list turn the
entire dialog into the scroll container.

Section-key navigation calculates from the button receiving the keyboard event,
not from the currently selected section. This matters after Tab moves focus to
an inactive button.

```tsx
const index = sections.indexOf(event.currentTarget.dataset.section as Section);
const next = sections[(index + delta + sections.length) % sections.length];
setSection(next);
sectionRefs[next].current?.focus();
```

Support Left/Right wrap and Home/End, open on General with its button focused,
and retain ordinary Tab/Shift+Tab order. Escape, header Close and footer Close
all dismiss through `ModalShell` and restore the Settings-command focus.

Render untrusted Markdown through `react-markdown` with raw HTML disabled.
Override links and images so Preview cannot navigate or trigger remote resource
loads. Code/plain text stays in a read-only `<pre>`.

## Icons and Deterministic UI Review

Source production icons from the approved Iconify collection at build time and
map them behind the typed application `Icon` adapter. Import explicit glyphs;
never pass unresolved icon-name strings to an Iconify runtime because that can
trigger third-party API requests and makes an offline desktop control disappear.

```tsx
// Correct: Vite compiles one explicit glyph into a static React SVG.
import SearchIcon from "virtual:icons/lucide/search";

// Wrong: an unresolved name can require the Iconify API at runtime.
<IconifyIcon icon="lucide:search" />
```

Maintain deterministic visual scenarios through the development-only review
entry under `src/review/`. Install typed mocked IPC before mounting the regular
`App`; scenarios must not invoke the real CLI, network or user filesystem. Keep
the review HTML outside the production Vite entry graph and prove the production
bundle contains no review entry, scenario identifiers or canonical fixture
payloads. Review screenshots are temporary evidence; record the matrix and
conclusions as text instead of committing screenshot binaries.

## Inventory and Preview

Inventory row headings identify the CLI-listed Skill. Start with no selection;
Arrow keys move focus while click/Enter selects. Use `aria-selected`, selected
and keyboard-focus states that are distinguishable from each other and from the
default state without relying on color alone. Their specific shape and styling
belong to an Approved Visual Direction. The file popover has no root row, begins
at `aria-level="1"`, retains full slash paths in accessible names/data, and
supports keyboard selection. Unsupported files remain selectable so their
type/size state and Reveal action are available.

Folder rows are real accessible tree disclosures, not decorative divs. Keep the
single roving tab stop in React state so collapse/expand rerenders cannot leave
DOM focus on a `tabIndex={-1}` node. Support Up/Down/Home/End across visible
treeitems, Right to expand/enter, and Left to collapse/return to the parent.
Initialize a refreshed tree expanded to preserve existing file visibility;
reopening reveals and focuses the selected file without discarding unrelated
folder toggles. Escape, outside dismissal and file activation restore focus to
the file-tree trigger.

```tsx
const [treeFocusPath, setTreeFocusPath] = useState<string | null>(null);
<button tabIndex={treeFocusPath === entry.path ? 0 : -1} />;
```

At the 720×520 compact boundary, secondary provenance and visible command copy
may be hidden, but the full path must remain in non-visual metadata and every
icon-only command must retain its localized accessible name, tooltip and shared
availability/execution path. Use visually distinct glyphs for file browsing and
Reveal File so two different commands do not collapse into the same symbol.

## Application Commands and Desktop Layout

Toolbar buttons, native menu items, shortcuts, and context-menu items are
adapters over the same typed Application Command dispatcher. Each command has
one availability calculation and one execution path; presentation surfaces must
not duplicate behavior or invent their own disabled state. Read-only Preview
commands remain available during mutations unless they conflict with the active
modal or document state.

The approved shared desktop composition is a Toolbar/Content/Status grid with a
two-pane Inventory/Preview content row at both 1180×800 and 720×520. The narrow
desktop adaptation may hide secondary command labels and provenance, but must
not replace the two panes with mobile navigation. Core workflows must remain
operable, status feedback visible, and content correctly scrollable. Keep each
grid surface pinned to the declared application column (`grid-column: 1` or an
equivalent explicit area); colocated runtime and workspace surfaces without an
explicit column create implicit side-by-side columns and can make the app fill
only half the window.

Transient controls anchored inside provenance must escape the toolbar without
being clipped and render above Preview content. Scope flex/grid selectors to
direct provenance children: a broad selector such as `.skill-provenance div`
also matches descendants inside the file-tree popover and can flatten its
header and rows horizontally. The file popover remains a vertical tree with a
separate title/count/root header, full path in accessible metadata, and focus
restoration to the file picker on Escape.

## Visual Authority

The current `styles.css`, component appearance, Prototype G, historical
`ux-design.md`, UI Audits, and archived task designs are implementation or
Historical Visual Direction, not authority for future visual work. The same is
true of current theme names and values, typography, density, spacing, radii,
shadows, brand marks, and the particular composition or appearance of Toolbar,
master/detail, Status, Modal, and Popover surfaces, including their dimensions,
proportions, and positions. Existing appearance remains the regression surface
for work that has no Approved Visual Direction; removing the old baseline is not
permission for incidental redesign.

Only a user-approved, task-local direction with explicit UI and platform scope
is an Approved Visual Direction. Its visual choices apply only within that
scope. It does not override command discoverability, status feedback, keyboard
paths, focus restoration, responsive operability, domain language,
localization, accessibility, safety, platform behavior, or accepted ADRs. In
particular, the single-window utility model, cross-platform Command Model, and
single Application Command authority in ADR-0012, ADR-0013, and ADR-0015 remain
in force. Copy from historical prototypes has no authority. UX Writing changes
require explicit task scope; visual approval alone does not authorize them.

Every Approved Visual Direction must state its platform scope. Approval for
macOS does not approve a Windows or Linux direction; shared changes there must
be limited to what is necessary to preserve the implementation without reducing
existing usability. Platform guidance such as the macOS HIG remains a behavior,
native-control, and usability guardrail. Its aesthetic recommendations are
inputs rather than approval.

Do not add an archived design document or prototype wholesale to the context
manifest for visual implementation or visual review. When historical material
is needed for a non-visual contract, include only the necessary artifact and
make the manifest `reason` identify the exact behavioral, accessibility, or
counterexample evidence being used.

When implementing an Approved Visual Direction, treat its task-local visual
artifact as an executable comparison target, not merely inspiration. Before
reporting completion, compare the built desktop App—not only a browser preview—
at every approved reference size for surface dimensions, pane structure,
typography/measure, command hierarchy, selected/focus states, popover stacking,
dialog composition, and loading/empty/error states. Automated behavior tests
do not constitute visual acceptance. Record any unavailable native-platform
smoke separately instead of treating shared WebView coverage as proof of native
menu/window behavior.
