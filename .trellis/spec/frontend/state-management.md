# Frontend State Management

Use React local state for short-lived UI choices such as the locale selector in
`App.tsx`. Do not add a global state library or client-side persistence layer.

Inventory and operation results come from typed Tauri commands. Refresh them
from Rust after each committed operation rather than mutating a second durable
copy in React. Derived filters and counts should be computed from the current
inventory response.

The locale preference may be persisted when settings are implemented, but the
static catalogs in `src/i18n.ts` remain the only source for user-facing strings.
