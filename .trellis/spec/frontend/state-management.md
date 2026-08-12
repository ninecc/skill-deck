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

Store UI locale intent as `system` or an explicit supported BCP 47 locale.
Derive the Effective UI Locale from that intent plus the runtime preferred
languages; never overwrite `system` with the currently resolved locale. A
`languagechange` event recomputes the effective locale while preserving all
other application state.

Translation requests use a generation token. Increment it synchronously when
translation closes, selection/input changes, or Retry begins; only the latest
generation may publish text or an error.

Inventory and mutation results come from typed Tauri commands. Replace the
current Inventory with every refreshed response rather than reconstructing
CLI state in React. Derive filters and counts from that response. Theme token
changes must not remount or reset selection, Preview, translation or scroll.
Refresh must publish selection-removal feedback before clearing the missing
selection, and retryable Preview failures must retain the requested path so
Retry repeats the failed request rather than falling back to another document.
