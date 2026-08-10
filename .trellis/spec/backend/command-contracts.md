# Desktop Command Contracts

## 1. Scope / Trigger

Use this contract for every Tauri command that previews or mutates Skill,
state, Agent-root, configuration, or Git content. It prevents React and Rust
from independently inventing transaction semantics.

## 2. Signatures

Mutation families use a preview/commit pair:

```text
plan_*(domain inputs) -> { id, ...deterministic preview }
commit_*(planId, ...explicit confirmation) -> { package?, restartMessage? }
```

The exceptions are read-only commands (`inventory`, `state_status`,
`check_git_update`) and the explicit drift resolution command:

```text
resolve_configuration(packageId, agent, resolution: reapply|forget)
cancel_staging() -> ()
```

Rust command parameters are `snake_case`; Tauri/TypeScript payload fields are
`camelCase`. Agent is the closed `codex|claude` enum.

## 3. Contracts

- Plans live only in process memory and carry a unique `id`; restart makes them
  invalid.
- A state file with a future `stateVersion` always enters read-only recovery;
  never replace it from an older backup, which could destroy newer state.
- Every commit re-reads state/source/topology and rejects a stale preview.
- One shared `Arc<Mutex<()>>` guards all mutation managers. Plan stores remain
  manager-local.
- Import targets are empty by default. Only `plan_install` receives Agent
  targets, and the backend resolves their real roots.
- Successful Agent-affecting writes return the exact restart fallback from
  `install::RESTART_MESSAGE`.
- `cancel_staging` acquires the shared gate and removes only
  `<app-data>/staging`; it never follows a staging-root symlink.
- `resolve_configuration(..., reapply)` overwrites only the matching entry with
  the recorded enabled value. `forget` changes provenance to External without
  editing the configuration file.
- Supported environment inputs are `CODEX_HOME` and `CLAUDE_CONFIG_DIR`; both
  must be absolute native paths. Codex user Skills remain fixed at
  `$HOME/.agents/skills`.

## 4. Validation & Error Matrix

| Condition | Stable code |
| --- | --- |
| Shared mutation gate held | `busy` |
| Missing/consumed/restarted plan | `invalid_plan` |
| Source differs from preview | `source_changed` |
| Existing destination/name | `conflict` |
| External content/topology changed | `content_drift` / `topology_changed` |
| App-owned configuration changed | `configuration_drift` |
| User/third-party configuration owns entry | `configuration_externally_controlled` |
| Link failed before explicit fallback consent | `copy_fallback_required` |
| Resource boundary exceeded | `resource_limit_exceeded` with `limit` and `observed` |

Errors serialize as `{code,message,path?,limit?,observed?}`. Do not parse message
text to make UI decisions.

## 5. Good / Base / Bad Cases

- Good: preview, show disclosed changes, commit the same plan, refresh inventory
  from Rust.
- Base: close a preview, call `cancel_staging`, and discard its in-memory id.
- Bad: retain a plan across restart, infer a Copy Fallback, overwrite drift, or
  mutate React state as if it were durable state.

## 6. Tests Required

- DTO serialization asserts camelCase fields and enum values.
- State recovery asserts that an unknown future primary version cannot fall
  back to or overwrite an older backup.
- Every commit covers success plus stale plan/source/topology.
- Multi-target writes inject a partial failure and assert filesystem, config,
  and state rollback.
- Shared-gate tests assert `busy` across different managers.
- Cancellation asserts exact staging cleanup and no traversal through links.
- Configuration drift tests assert both explicit Reapply and Forget outcomes.
- Resource tests assert exact-limit acceptance and one-over rejection with
  structured `limit`/`observed` values.

## 7. Wrong vs Correct

Wrong:

```ts
await invoke("commit_install", { path: userPath, copyFallback: true });
```

Correct:

```ts
const plan = await planInstall(packageId, ["codex"], createMissingRoots);
await commitInstall(plan.id, confirmCopyFallback);
await loadInventory();
```

The correct flow keeps path resolution, validation, ownership, and durable
state inside Rust while React owns only explicit user confirmation.
