# Frontend State Management

Use React local state for Inventory, selection, Preview, translation and dialog
state. Do not add a global state library.

Only UI preferences persist in localStorage: locale, one theme preset, target
language, optional Agent overrides, automatic/copy install mode, and an
optional credential-free Translation Proxy Override. Proxy input stays in
dialog-local draft state until Apply; validate it before updating Preferences
so invalid values or credentials never enter localStorage. Rust independently
validates the command input. Inventory and translation content are never
persisted.

Translation requests use a generation token. Increment it synchronously when
translation closes, selection/input changes, or Retry begins; only the latest
generation may publish text or an error.

Inventory and mutation results come from typed Tauri commands. Replace the
current Inventory with every refreshed response rather than reconstructing
CLI state in React. Derive filters and counts from that response. Theme token
changes must not remount or reset selection, Preview, translation or scroll.
