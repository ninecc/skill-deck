# React Component Guidelines

Components are functions with typed props. Prefer semantic HTML and native
controls before custom widgets; `App.tsx` uses real buttons, links, labels, and
a select so keyboard behavior exists without extra code.

## Accessibility

- Every icon-only or brand link needs an accessible name.
- Decorative marks use `aria-hidden`.
- Visible sections connect headings with `aria-labelledby` where needed.
- Preserve `:focus-visible`, reduced-motion behavior, readable contrast, and a
  320px minimum layout in `styles.css`.
- A disabled or unavailable domain action must expose the reason in text, not
  only color.
- Modal dialogs are mutually exclusive, trap focus, close on Escape when safe,
  and restore focus to their trigger or a stable fallback when that trigger was
  removed by the confirmed action.

Render untrusted Markdown through `react-markdown` with raw HTML disabled.
Override links and images so Preview cannot navigate or trigger remote resource
loads. Code/plain text stays in a read-only `<pre>`.

## Inventory and Preview

Inventory row headings identify the CLI-listed Skill. Start with no selection;
Arrow keys move focus while click/Enter selects. Use `aria-selected`, selected
and keyboard-focus states that are distinguishable from each other and from the
default state without relying on color alone. Their specific shape and styling
belong to an Approved Visual Direction. The file popover has no root row, begins
at `aria-level="1"`, retains full slash paths in accessible names/data, and
supports keyboard selection. Unsupported files remain selectable so their
type/size state and Reveal action are available.

## Application Commands and Desktop Layout

Toolbar buttons, native menu items, shortcuts, and context-menu items are
adapters over the same typed Application Command dispatcher. Each command has
one availability calculation and one execution path; presentation surfaces must
not duplicate behavior or invent their own disabled state. Read-only Preview
commands remain available during mutations unless they conflict with the active
modal or document state.

The current 820px breakpoint and Toolbar/Content/Status three-row composition
are regression surfaces, not permanent visual requirements. An Approved Visual
Direction may replace them. At the minimum supported window size, core workflows
must remain operable, status feedback visible, and content correctly scrollable.
Narrow-detail navigation supports `Alt+Left` and macOS `Meta+[` without
resize-driven React state.

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
