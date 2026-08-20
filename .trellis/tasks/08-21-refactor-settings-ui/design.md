# Technical design

Keep `SettingsDialog` responsible for section selection, proxy draft/error and
Agent filtering. Persisted preference ownership stays in the parent through the
existing typed `onChange` contract. `ModalShell` continues to own focus trap,
dismissal and restoration.

Use a fixed three-row dialog grid. Navigation remains stable; only the Settings
content viewport scrolls. Theme tiles, fields, radios, target rows and inline
validation consume shared semantic tokens and Iconify/Lucide icons.

Do not sort `agentOptions`; filter the current order. Preserve existing
preference types and validation. Split visual work by shared chrome, short
sections, Translation and long Installation content for rollback.
