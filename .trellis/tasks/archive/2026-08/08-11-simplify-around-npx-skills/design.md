# Technical Design

## Design Read

This is a redesign of a desktop developer tool for people managing Agent Skills. Preserve the existing restrained warm-neutral/green identity, but replace the current card-heavy lifecycle control plane with a denser native-tool workspace. Design variance 3, motion 2, visual density 7.

## Architecture

Skill Deck stops being a package manager. The installed filesystem and lock data owned by the Skills CLI are the only durable source of truth. At startup, the app resolves `skills@latest` once and pins the returned exact version for the lifetime of that app session.

```text
React UI
  -> typed Tauri commands
     -> CLI adapter -> local node/npx -> skills@latest
     -> preview reader -> installed Skill directories
     -> translator -> anonymous Google Translate provider

Durable Skill state: owned only by skills CLI
Skill Deck persistence: UI preferences in localStorage only
```

The rewrite deletes the Managed Library, app state manifest, ownership/adoption model, Agent configuration editors, revision history, Git updater and their UI. No compatibility migration is required because deleting Skill Deck state must not alter CLI-owned installed Skills.

## Backend Boundaries

### CLI adapter

A single Rust module owns process discovery, argument construction, execution and output decoding.

- Resolve `node` and `npx` only through the desktop process `PATH`.
- Run commands with `std::process::Command` and an argument vector, never through a shell.
- At startup invoke `npx --yes skills@latest --version`, then invoke `npx --yes skills@<resolved-version> ...` for every command in that session. The outer `--yes` is distinct from the skills command's own `-y`.
- Set `DO_NOT_TRACK=1` on every skills CLI child process; do not inherit the CLI telemetry default or expose a telemetry preference.
- Run the resolved version and `skills list -g --json` as the compatibility probe. A new latest is considered only after the next app launch.
- Decode list JSON once into open-string DTOs: Agent names are `String`, not a closed Rust/TypeScript enum.
- Treat non-zero exit, invalid UTF-8, invalid JSON and incompatible JSON shape as structured command errors containing operation, exit code when present, and sanitized stderr. Do not parse human stdout to infer success state; add/remove may report partial failures while exiting 0.
- Serialize mutations through one in-process gate. Snapshot the relevant Inventory state before add/remove/update, refresh from `list -g --json` afterward, and determine the outcome from the observed state transition; exit status and sanitized output remain diagnostics.

Commands:

```text
runtime_status() -> RuntimeStatus
list_skills() -> InstalledSkill[]
search_skills(query) -> SearchResult[]
add_skill(source, skill?, settings) -> CommandResult
remove_skill(name) -> CommandResult
update_skill(name?) -> CommandResult
```

`search_skills` uses the same JSON search endpoint used by the official CLI because `skills find` has no documented JSON flag. This is isolated in the CLI integration module and fails as an unavailable search feature; it never affects installed inventory.

Installing a search result always passes its exact `--skill` name. Direct-source installation deliberately omits `--skill`; non-interactive CLI behavior may therefore install every Skill discovered in that source, which the source input discloses before execution.

`remove_skill` removes the global Installed Skill across all Agent Targets and always follows a GUI confirmation. `update_skill(name)` powers row-level Update; `update_skill(None)` powers an explicitly confirmed Update All. There is no background updater or per-Agent removal in MVP.

Settings supply only optional overrides:

- no Agent override: omit `--agent`, allowing CLI automatic detection;
- explicit Agent IDs: append `--agent <ids...>`;
- automatic install mode: omit `--copy`;
- copy: append `--copy`.

The explicit Agent picker contains only app-verified IDs. The automatic default remains compatible with Agents added later by the CLI; the explicit list advances with Skill Deck releases and has no free-form Agent ID entry.

### Preview reader

The preview reader owns filesystem containment and viewer classification. It accepts an installed Skill identity from the current CLI inventory, never an arbitrary root from React.

- Canonicalize the CLI-reported Skill root once and retain it only in an in-memory inventory map.
- Walk with `symlink_metadata`; list internal links as unsupported entries and never follow them.
- Resolve every requested relative path against the retained root and reject absolute paths, parent traversal, special files or a canonical target outside the root.
- Cap text reads at 1 MiB and raster image reads at 10 MiB. Larger files remain listed with metadata and an unsupported/too-large state.
- Classify Markdown, plain text, source code and common raster images. JSON/YAML/source code use a read-only code viewer but are not translatable. Unknown/binary formats expose metadata only.
- Reveal the current file or Skill root through the platform file manager only after the same containment check.

The frontend uses `react-markdown` without raw HTML support for Markdown, a native monospace `<pre>` for code/text, and an `<img>` backed by bounded bytes for raster images. No Monaco/CodeMirror, Office/PDF/media viewer or editor is added.

### Translation module

Translation is a small replaceable module explicitly requested by the product, not a plugin system or runtime provider registry.

```text
TranslationRequest { text, targetLanguage }
translate(request) -> TranslationResult { translatedText, detectedSourceLanguage? }
```

The module has one anonymous Google Translate implementation. It may chunk bounded UTF-8 input safely and reassemble responses in order. Its endpoint and response decoding stay private to the module so replacing Google does not change preview callers. No one-implementation trait/factory is required. Provider failures are structured and affect only the translation pane.

Translation is allowed only for Markdown and plain-text documentation. Markdown is segmented so frontmatter, fenced code, inline code, link URLs and document structure are copied unchanged while natural-language prose is translated. Results stay in React session state, are never cached durably and never write to the Skill directory.

The target picker is a fixed common-language list rather than a provider catalog: English, Simplified Chinese, Traditional Chinese, Japanese, Korean, Spanish, French, German, Portuguese, Italian, Russian, Arabic and Hindi. While translation mode remains on, selecting another eligible document automatically translates it; ineligible files never trigger a request.

## Frontend and UX

### Main surface

- Runtime prerequisite failure replaces management actions with a direct Node/npm requirement message.
- The installed list comes from CLI JSON and supports text/Agent filtering without ownership or enabled-state filters.
- Search, install, remove and update are short dialogs/actions over the CLI adapter. Search failure leaves source installation available. Each row has Update, Update All requires confirmation, and destructive whole-Skill remove requires confirmation.
- Settings contains one Appearance theme picker, translation target language, Agent target behavior and install method. Appearance is a compact horizontal single-select group with System (Default), Light, Dark, Sand and Plum; each tile previews chrome, content and interaction tokens rather than one ambiguous accent dot. System uses stable Light|Dark split swatches, so its preview never collapses into a duplicate of Dark when the OS is dark. It is not split into mode and palette controls. Labels describe behavior and append `(Default)` instead of saying only “CLI default”. The actual `skills` version is visible.

Preferences use the existing local state/localStorage pattern. There is no new global state library or Rust settings file.

Theme is one persisted enum, not a CLI override or a `{ mode, palette }` pair. `system` maps through `prefers-color-scheme` and listens for changes; every explicit preset ignores later system changes until System is restored. Light and Dark use the Graphite/Azure token maps, Sand is a fixed warm-light cream/tan map with amber/rust interaction roles, and Plum is a fixed deep-aubergine map with lavender interaction roles. Each map defines window, chrome, surface, text, border, focus, selection, danger, warning, code and overlay roles. Theme changes only remap tokens and never recreate Inventory, Preview or translation state. There is no theme editor, arbitrary palette or cross-product of color and mode.

Theme detail stays semantic rather than embedding component hex values. Existing control, tag, selected and code-surface roles are supplemented only by `tag-border` and `code-border`; Agent labels, path/control borders, selected file rows and Markdown code blocks consume these roles. Sand maps them to warm tan/rust relationships, Plum to aubergine/lavender, and Light/Dark to restrained Graphite/Azure. Danger and warning remain independent semantic roles instead of palette derivatives.

Icons are selected through Iconify from one verified open-source outline collection and bundled locally. The application never passes string names to an Iconify component that fetches missing data from the public API. Prefer the smallest offline integration that embeds only selected icon data or generated SVG components; use `currentColor`, one glyph size/weight scale and accessible text/labels. Primary, ambiguous and destructive commands retain visible text instead of relying on icon recognition.

### Preview workspace

`selectedSkillId` starts as `null`. In wide master/detail, the empty detail is a compact instruction to choose a Skill and contains no path, translation or Skill actions. In the narrow navigation stack, startup remains on Inventory. Arrow keys move list focus without mutating selection; click or Enter commits a selection and opens Preview. This avoids presenting the first Installed Skill as a user choice that never occurred.

Normal preview:

```text
[ current/path.md v ] [Translate · sends content to Google] [Reveal]
------------------------------------------------
single file viewer
```

The path button opens a keyboard-navigable popover tree and already identifies the Skill root, so the tree does not repeat a focusable `/` root row. Top-level files and folders begin at `aria-level="1"`; folder and file glyphs from the same local Phosphor Regular set combine with indentation to express hierarchy without `Folder:` prefixes or visible path punctuation. `data-path` and accessible names retain complete slash paths such as `references/`. The MVP tree is statically expanded and does not add collapse state. Closing it leaves no reserved column and preserves selected state for the preview session. Installed selection uses row tint plus an accent edge without visible `Selected` copy; `aria-selected` and a separate focus ring preserve non-visual and keyboard state.

Translation preview at widths of 900 px and above:

```text
[ current/path.md v ] [Target language] [Translate on]
------------------------------------------------------
original, independently scrollable | translation, independently scrollable
```

Below 900 px, the same content uses accessible Original/Translation tabs rather than squeezing two unreadable columns. Translation loading and error states occupy only the translation side; original content remains usable.

## Compatibility and Failure Policy

- `skills@latest` is mutable between app launches but pinned to one resolved exact version within a session. Contract probe failure stops CLI-dependent actions and reports the actual version; Skill Deck never silently parses terminal tables or falls back to its deleted manager.
- Failure to resolve latest leaves management and preview unavailable with a Retry action. There is no persisted last-known-version fallback in MVP.
- Because Inventory exposes no revision or content hash, a successful-looking Update reports only that the command completed and Inventory refreshed. It never claims “up to date” or “updated to latest” without evidence.
- Node/npm visible only inside an interactive shell is considered unavailable in MVP. The verified 1.5.22 floor is Node 22.20.0, while the runtime probe remains responsible for future latest changes.
- The anonymous translation endpoint is an acknowledged best-effort ceiling. A `ponytail:` comment at the provider records that a supported API/provider should replace it when reliability or quota matters.
- Existing installed Skills remain on disk when old Skill Deck state and app-owned library data become unused. The rewrite neither migrates nor automatically deletes old app-data and does not retain a compatibility manager; a missing CLI Inventory entry may only offer reinstall from its original source.
- Windows/Linux behavior needs native verification for PATH lookup and file-manager reveal. macOS-only checks cannot claim those platforms.

## Security

- No shell interpolation; all CLI inputs are individual process arguments.
- Search queries, source strings, Skill names and Agent IDs are validated for emptiness/size before invocation, while the CLI remains authoritative for domain validation.
- Preview never follows internal links or reads outside a current CLI-listed root.
- Untrusted Markdown raw HTML is disabled.
- Translation sends the selected documentation text to Google only after an explicit user action; UI copy states that the content leaves the machine.

## Rollout and Rollback

Implement as one replacement, not a compatibility layer. Keep the last pre-rewrite Git commit as rollback. Before deleting old modules, make the new CLI inventory compile and pass focused tests; then remove old frontend/backend paths in the same branch so there is only one runtime model at completion.
