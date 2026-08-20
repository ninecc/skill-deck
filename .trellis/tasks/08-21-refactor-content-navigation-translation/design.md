# Technical design

Keep file selection and translation state owned by `App`. Extract presentation
only if it clarifies seams without moving effects or command dispatch.

The file tree is an absolutely positioned transient surface inside provenance,
with vertical tree layout and stacking above Preview. Scoped selectors must not
flatten descendants. The trigger owns `aria-expanded`/`aria-controls`; tree rows
retain semantic levels and full paths.

Directory rows join the roving tree focus model and own disclosure state. Use
standard Click/Enter/Left/Right semantics and approved localized trigger copy.
Track expanded normalized directory paths in a React `Set`. Backend preorder and
directory trailing slashes allow ancestor visibility checks without a DTO
change. Initialize all expanded, reset with a new tree/Skill and auto-reveal only
selected ancestors on popover reopen.

Translation remains a one- or two-column viewer composition driven by current
state. At compact width, any pane switcher must preserve the approved two-pane
desktop shell and keep both original and translated content reachable.
The compact switcher controls visibility only; it must not discard either pane's
state or restart translation.

Rollback independently at the popover and translation boundaries. Do not alter
backend DTOs or translation request semantics for visual fidelity.
