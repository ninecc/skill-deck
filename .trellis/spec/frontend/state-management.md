# Frontend State Management

Use React local state for Inventory, selection, Preview, translation and dialog
state. Do not add a global state library.

Only UI preferences persist in localStorage: locale, one theme preset, target
language, optional Agent overrides and automatic/copy install mode. Validate
stored values against the closed UI option lists. Inventory and translation
content are never persisted.

Inventory and mutation results come from typed Tauri commands. Replace the
current Inventory with every refreshed response rather than reconstructing
CLI state in React. Derive filters and counts from that response. Theme token
changes must not remount or reset selection, Preview, translation or scroll.
