# Skill Deck

Skill Deck is a desktop GUI over the upstream Skills CLI. The CLI owns installed Skill lifecycle state; Skill Deck adds bounded read-only preview and translation.

## Language

**Skill Consumer**:
An individual developer who installs and maintains Skill Packages for personal use across one or more Agent Targets.
_Avoid_: Skill author, administrator, publisher

**Skill Package**:
A directory whose root contains a valid `SKILL.md` and whose remaining files belong to that Skill as one portable package.
_Avoid_: Plugin, extension, script

**Agent Target**:
A global-scope AI coding agent supported by the upstream Skills CLI. Agent identifiers are an open upstream vocabulary; explicit overrides are limited to Skill Deck's verified list.
_Avoid_: Platform, provider, runtime, Codex-or-Claude enum

**Inventory**:
The current read-only list of global Installed Skills returned by `skills list -g --json`. Skill Deck refreshes it after every management command and never persists it as app-owned lifecycle state.
_Avoid_: Directory scan, registry, Managed Library, Skill Deck state

**Installed Skill**:
A global Skill Package reported by the upstream Skills CLI, together with its canonical path, source metadata, and current Agent Target names.
_Avoid_: Managed Skill Package, Installation record, local package

**CLI Override**:
An optional preference that adds an explicit upstream CLI flag. When absent, Skill Deck omits the flag and preserves the CLI's automatic behavior.
_Avoid_: Skill Deck policy, CLI default copy

**Theme Preference**:
One persisted application-identity preset selected from `system`, `light`, `dark`, `sand`, or `plum`. It controls custom surfaces and the Theme Accent through semantic tokens; native platform affordances retain their system appearance and System Accent where the platform and WebView support it.
_Avoid_: CLI Override, separate mode/palette pair, System Accent override, custom palette, UI locale

**UI Language Preference**:
An application-level preference that follows the system locale by default or stores one explicit supported locale override. It changes Skill Deck's interface language immediately and is independent of the Translation Target.
_Avoid_: Toolbar command, Translation Target, per-window locale

**Effective UI Locale**:
The supported BCP 47 locale currently used to render Skill Deck. It is derived from the UI Language Preference and the runtime's reported preferred languages without rewriting the stored user intent.
_Avoid_: Raw navigator language, Translation Target, persisted system locale

**Application Command**:
A user-invokable Skill Deck behavior with one identity, availability result, execution path and lifecycle, independent of whether it is presented by a Toolbar, menu, shortcut or Context Menu.
_Avoid_: Button handler, menu-only action, duplicated surface action

**CLI Session Version**:
The exact Skills CLI version resolved from `skills@latest` once when the app starts and reused for every command until that app session ends.
_Avoid_: Permanently pinned version, resolve latest per command

**Command Outcome**:
The result of an add, remove, or update operation determined from the refreshed Inventory where that state is observable, with exit status and sanitized CLI output retained as diagnostics. Update completion never claims a content revision that Inventory cannot prove.
_Avoid_: Exit-code success, terminal-output parsing

**Whole-Skill Removal**:
Removal of one global Installed Skill from all of its Agent Targets after explicit confirmation.
_Avoid_: Per-Agent uninstall, Remove from Library

**Preview Session**:
The in-memory read-only view of one Installed Skill's bounded file tree and selected file. It never edits Skill content or follows links outside the Installed Skill root.
_Avoid_: Editor, file manager, persisted workspace

**Translatable Document**:
A Markdown or plain-text documentation file eligible for read-only translation. Markdown frontmatter, code, URLs, and structure remain unchanged while natural-language prose may be sent for translation.
_Avoid_: Source code, JSON, YAML, arbitrary binary file

**Translation Target**:
The provider-neutral target language saved in Settings, initially derived from the system locale and falling back to English when unsupported.
_Avoid_: UI locale, source language

**Translation Proxy Override**:
An optional persisted, credential-free HTTP(S) proxy URL used only by the Translation Module. When absent, translation uses the HTTP client's automatic environment proxy behavior.
_Avoid_: System proxy, global network setting, authenticated proxy

**Translation Module**:
The replaceable boundary that accepts document text and a Translation Target and returns session-only translated text. The MVP implementation sends eligible content to anonymous Google Translate after persistent UI disclosure.
_Avoid_: Plugin system, credential manager, document writer

**Translation Session**:
The temporary translated view for the currently selected Translatable Document in one Installed Skill. It ends when translation is closed or the selected Skill or document changes, and is never restored automatically.
_Avoid_: Translation cache, persisted translation, global translation mode

**Legacy App State**:
The former Skill Deck Managed Library and lifecycle metadata. The redesigned app neither migrates nor deletes it and never uses it to populate Inventory; a missing Skill may only be reinstalled from its original source.
_Avoid_: Compatibility inventory, migration source, fallback manager
