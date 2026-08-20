# Technical design

Use `ModalShell` as the single focus/dismissal mechanism. Find & Install and
Remove supply content/actions but must not duplicate dialog mechanics. Keep
command availability/execution in the shared dispatcher and feed dialogs with
typed state.

Find & Install has local Search/From source navigation while preserving the
existing catalog and source-install command paths. Search is the default tab.
Tab switches retain both local drafts/results. A normal new opening resets both;
the existing unresolved-discovery branch preserves retry context. The Search
footer retains Pencil's `Install from source…` shortcut, but the From source
footer does not add a reciprocal Search action: the persistent tabs already own
that navigation and the source form's Install button remains its sole primary
action.

Operational Loading/Runtime/Empty/Preview-failure states remain mutually clear
render branches within the shared shell. Status feedback consumes structured
severity/summary/diagnostics and retains visible recovery actions.

Apply Pencil styling through shared semantic classes and tokens. Do not create
state-specific palette constants. Roll back dialog and operational-state work
independently if behavior regressions appear.
