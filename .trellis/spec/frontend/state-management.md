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

Multi-tab task dialogs keep drafts and results in React state for one opening.
Switching tabs must not clear either tab. A normal close/reopen starts a fresh
task at the default tab and clears both drafts/results; an unresolved mutation
is the exception and retains the active tab, last target, drafts and Retry
context across close/reopen. Tests must distinguish all three transitions:

- tab switch -> preserve both panels' state;
- normal reopen -> reset to the default tab and empty drafts/results;
- unresolved reopen -> restore the exact retry target and context.

Do not infer unresolved state from an error string. Track it explicitly beside
the typed last-operation target, and let the existing command path perform the
retry.

Settings preference boundaries are explicit. UI locale intent, theme,
translation target, install method, Agent mode and individual Agent targets call
the typed parent `onChange` immediately. Translation Proxy is different: keep a
dialog-local draft and validation error, and update Preferences only after a
successful Apply. Invalid credentials, paths or non-HTTP(S) values stay inline
and must never publish a false saved state.

Filtering Agent targets preserves the upstream `agentOptions` order:

```tsx
const visibleAgents = agentOptions.filter((agent) => matches(agent, query));
// Do not append `.sort()` here; registration order is product data.
```

Tests must assert immediate persistence for at least one scalar preference and
one individual Agent checkbox, plus proxy draft retention across section
switches, invalid rejection and valid Apply persistence.
