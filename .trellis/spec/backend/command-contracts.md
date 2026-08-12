# Desktop Command Contracts

## Runtime and inventory

The upstream Skills CLI is the only lifecycle source of truth. At app startup,
resolve `skills@latest` once with `npx --yes`, validate its major version and
`list -g --json` shape, then pin that exact version for the process lifetime.
Every CLI child receives `DO_NOT_TRACK=1`; commands use argument vectors and
never shell interpolation.

```text
runtime_status() -> { ready, version?, nodeVersion?, message? }
list_skills() -> InstalledSkill[]
search_skills(query) -> SearchResult[]
add_skill(source, skill?, settings) -> CommandResult
remove_skill(name) -> CommandResult
update_skill(name?) -> CommandResult
```

`InstalledSkill.agents` is `String[]`, not a closed Agent enum. Inventory JSON
must be structurally decoded; never parse terminal tables or ANSI output.
Search uses the upstream JSON endpoint and remains isolated from runtime and
inventory availability.

## Mutation contracts

- Serialize add, remove, and update through one in-process gate.
- Always pass global scope and non-interactive confirmation (`-g -y`).
- Omit `--agent` and `--copy` for automatic defaults. A search-result install
  passes its exact `--skill`; a direct source deliberately omits it.
- Snapshot Inventory, execute the pinned CLI, refresh Inventory, and report the
  observed DTO change. Human output is bounded diagnostic text, not truth.
- Update has no observable revision contract; completion copy may say only that
  the command completed and Inventory refreshed.
- Remove is whole-Skill across Agent targets and requires frontend confirmation;
  Update All also requires confirmation.

Errors serialize as `{code,message,operation?,exitCode?,diagnostics?}`. Stable
wrapper codes include `runtime_unavailable`, `node_too_old`, `incompatible_cli`,
`invalid_input`, `command_failed`, `busy`, and preview/translation codes. React
normalizes the DTO but never infers domain state from message text.

## Preview and translation

```text
preview_tree(skill) -> FileEntry[]
read_preview(skill, path) -> FileContent
reveal_path(skill, path?) -> ()
translate_preview(skill, path, targetLanguage) -> TranslationResult
```

React supplies an installed Skill name plus a relative path. Rust resolves the
root only from the current CLI Inventory, rejects absolute/traversing paths,
does not follow listed links, rechecks containment, and bounds reads. Translation
reuses the preview reader, accepts only Markdown/plain text and never writes.

## Tests required

- Exact CLI argv for defaults and overrides, outer `npx --yes`, and
  `DO_NOT_TRACK=1`.
- Open Agent strings plus malformed/invalid list JSON.
- Mutation serialization and refreshed observed outcomes.
- Preview traversal, links, special files, exact read limits, invalid UTF-8 and
  viewer classification.
- Translation language eligibility, UTF-8 chunk order, Markdown structure
  preservation, provider failure and zero writes.
