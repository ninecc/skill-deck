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

Do not render untrusted Skill Markdown as HTML. Display structured metadata and
paths as text unless a separately reviewed sanitizer is introduced.

## Inventory identity and diagnostics

Inventory row headings identify the Skill and must not be replaced by a status
such as `Needs attention`. For an invalid external entry, use the final path
component as the fallback name, render the diagnostic state as an adjacent
badge, and keep the localized reason and logical path as separate text. This
keeps search results and row identity stable while preserving the full error.
