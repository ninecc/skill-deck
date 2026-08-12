# Desktop Command Contracts

## Runtime and inventory

The upstream Skills CLI is the only lifecycle source of truth. At app startup,
resolve `skills@latest` once with `npx --yes`, validate its major version and
`list -g --json` shape, then pin that exact version for the process lifetime.
Every CLI child receives `DO_NOT_TRACK=1`; commands use argument vectors and
never shell interpolation.

```text
runtime_status() -> { ready, errorCode?, version?, nodeVersion?, message?, inventory }
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
translate_preview(skill, path, targetLanguage, translationProxy) -> TranslationResult
```

React supplies an installed Skill name plus a relative path. Rust resolves the
root only from the current CLI Inventory, rejects absolute/traversing paths,
does not follow listed links, rechecks containment, and bounds reads. Translation
reuses the preview reader, accepts only Markdown/plain text and never writes.

Every command whose implementation may launch a process, wait for network I/O,
walk/read the filesystem, or wait for an OS helper is `async` at the Tauri
boundary and runs its synchronous implementation through
`tauri::async_runtime::spawn_blocking`.

On macOS, runtime discovery scans inherited PATH entries first, followed by
`/opt/homebrew/bin` and `/usr/local/bin`. A candidate directory must contain
executable sibling `node` and `npx` files. Relative PATH entries are normalized
to absolute paths, and the selected directory is prepended to CLI child PATH so
the `npx` shebang resolves its sibling Node. Other platforms use inherited PATH
only. The selected absolute toolchain and resolved `skills` version remain
pinned in the successful session.

Runtime discovery failures expose a stable `RuntimeStatus.errorCode`; raw
spawn, PATH and OS error strings are never included in its user-facing message.

## Scenario: atomic startup runtime and Inventory

### 1. Scope / Trigger

Changing startup runtime probing or the initial Inventory response is a
cross-layer contract change. Startup must not validate Inventory and then ask
the CLI for the same Inventory again.

### 2. Signatures

```text
runtime_status() -> RuntimeStatus
retry_runtime() -> RuntimeStatus
RuntimeStatus.inventory: InstalledSkill[] // required
```

There is no separate `list_skills` command. Mutation commands still return
their refreshed Inventory in `CommandResult`.

### 3. Contracts

- A successful startup returns the exact decoded `list -g --json` result used
  to validate the pinned CLI session; that invocation also refreshes Preview's
  name-to-root map.
- `ready == true` publishes runtime fields and Inventory atomically.
- `ready == false` always returns `inventory == []` with the stable runtime
  error fields.
- Each successful initial startup or Retry executes exactly one Inventory
  list. Failures during runtime discovery or version validation may execute no
  list. Retry clears the failed session before repeating resolution and
  validation.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Compatible runtime and valid Inventory JSON | `ready=true` with decoded `inventory` |
| Runtime discovery/version failure | `ready=false`, stable `errorCode`, empty `inventory` |
| Invalid Inventory JSON or incompatible CLI | `ready=false`, `incompatible_cli`, empty `inventory` |

### 5. Good/Base/Bad Cases

- Good: one startup command returns runtime readiness and 45 Installed Skills.
- Base: a valid empty list returns `ready=true` and `inventory=[]`.
- Bad: `runtime_status` validates with one list and React invokes a second list.

### 6. Tests Required

- Assert every serialized `RuntimeStatus` includes an Inventory array.
- Assert React startup invokes only `runtime_status`, then renders its Inventory.
- Assert failure followed by Retry invokes only `runtime_status` and
  `retry_runtime`, and publishes the Retry Inventory.

### 7. Wrong vs Correct

```text
Wrong: runtime_status() validates Inventory; list_skills() fetches it again.
Correct: runtime_status() returns the same Inventory that completed validation.
```

## Scenario: bounded translation networking

### 1. Scope / Trigger

Changing the translation proxy request field or provider timing/error behavior
is a cross-layer contract change spanning React preferences, the Tauri command,
and the blocking HTTP client.

### 2. Signatures

```text
translate_preview(
  skill: String,
  path: String,
  target_language: String,
  translation_proxy: String,
) -> Result<TranslationResult, CommandError>
```

### 3. Contracts

- `translation_proxy == ""` preserves reqwest automatic environment proxies.
- A non-empty override affects translation only; it never changes CLI or search.
- Connect timeout is 5 seconds. All chunks share one 15-second operation
  deadline and receive only its remaining duration.
- Each provider request gets at most two attempts. An attempt is capped at the
  smaller of seven seconds and the shared deadline's remaining duration; only
  connection and timeout failures are retried.
- Markdown prose fragments are HTML-escaped and packed into indexed inert spans
  within the provider-size bound measured in Unicode characters; an oversized
  fragment is split safely. At most four batches run concurrently; exact
  sequential unique span markers are required, and raw tags or unknown entities
  are rejected before entity decoding and document-order publication after
  every batch succeeds. Detected language follows document order, not worker
  completion order.
- Markdown reconstruction uses each prose slice's original source range, so
  repeated text inside protected syntax cannot be mistaken for translatable
  prose. A worker panic becomes a stable `internal` failure with no partial
  result.
- `TranslationResult` is published only after every chunk succeeds; translated
  content remains session-only.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Empty proxy | automatic environment proxy |
| More than 2,048 bytes, non-HTTP(S), missing host, path/query/fragment, or credentials | `invalid_proxy` |
| Connect or HTTP status failure | `translation_unavailable` |
| Request or operation deadline timeout | `translation_timeout` |
| Decode, response-shape, or empty-segment failure | `translation_response` |

Provider error strings and query URLs never cross the command boundary.

### 5. Good/Base/Bad Cases

- Good: `http://127.0.0.1:7890` is applied only to the translation client.
- Base: empty override uses the existing environment behavior.
- Bad: `http://user:password@proxy` is rejected before any request.

### 6. Tests Required

- Assert proxy acceptance/rejection and blank override behavior without external
  network access.
- Assert later chunks receive the remaining shared deadline and any failed chunk
  yields no partial `TranslationResult`.
- Assert escaping/decoding, Unicode batch size and oversized fragments, strict
  marker/tag/entity validation, bounded batch concurrency, stable output and
  detected-language order, and atomic failure.
- Assert repeated protected/prose text reconstructs from the correct ranges and
  worker panic is contained as an atomic failure.
- Assert every provider failure returns only the stable code and sanitized copy.
- Assert a blocking command future yields while its worker is pending.

### 7. Wrong vs Correct

```text
Wrong: one 15-second timeout per chunk; serialize reqwest errors to React.
Correct: one 15-second operation deadline; return stable sanitized errors.
```

## Tests required

- Exact CLI argv for defaults and overrides, outer `npx --yes`, and
  `DO_NOT_TRACK=1`.
- Open Agent strings plus malformed/invalid list JSON.
- Mutation serialization and refreshed observed outcomes.
- Preview traversal, links, special files, exact read limits, invalid UTF-8 and
  viewer classification.
- Translation language eligibility, UTF-8 chunk order, Markdown structure
  preservation, proxy validation, shared deadline, atomic publication,
  sanitized provider failure and zero writes.
