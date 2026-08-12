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
fill and an accent edge without visible `Selected` text. The file popover has no
root row, begins at `aria-level="1"`, retains full slash paths in accessible
names/data, and supports keyboard selection. Unsupported files remain selectable
so their type/size state and Reveal action are available.

## Application Commands and Desktop Layout

Toolbar buttons, native menu items, shortcuts, and context-menu items are
adapters over the same typed Application Command dispatcher. Each command has
one availability calculation and one execution path; presentation surfaces must
not duplicate behavior or invent their own disabled state. Read-only Preview
commands remain available during mutations unless they conflict with the active
modal or document state.

At widths up to 820px the workspace may stack, but the window must retain three
visible rows: toolbar, scrollable content, and Status. Do not place Status after
an unconstrained content row that can push it below the viewport. Narrow-detail
navigation supports `Alt+Left` and macOS `Meta+[` without resize-driven React
state.
