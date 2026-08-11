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

## Scenario: Inventory entry classification

### 1. Scope / Trigger

Apply this contract whenever Agent-root discovery or the `inventory` response
shape changes. Rust owns filesystem classification; React must not reconstruct
it from paths, filenames, or diagnostic messages.

### 2. Signatures

```text
inventory() -> {
  externalInstallations: ExternalInstallation[],
  attentionEntries: AttentionEntry[],
  managedPackages: ManagedSkillPackage[],
  targets: AgentTarget[]
}

AttentionEntry.kind =
  broken_external_installation |
  invalid_installation_candidate |
  unexpected_agent_root_entry
```

### 3. Contracts

- `externalInstallations` contains only entries with a validated Skill payload.
- `attentionEntries` carries `kind`, `agent`, `logicalPath`, optional
  `resolvedTarget`, and structured `{code,message,path}` diagnostics.
- A link is `broken_external_installation` only when its topology cannot be
  safely resolved. A resolved link whose content fails validation is an
  `invalid_installation_candidate`.
- Non-directory, non-link root entries are `unexpected_agent_root_entry`.
- Root `.DS_Store` entries and Codex legacy-root `.system` are artifacts and
  appear in neither array. Other `.system` directories are still inspected.
- Diagnostics exports may include kind, Agent, logical path, and structured
  error, but never Skill bodies or resolved targets.

### 4. Validation & Error Matrix

| Root entry | Inventory result |
| --- | --- |
| Valid Skill directory or resolved link | `externalInstallations` |
| Missing, cyclic, or unsafe external link | `broken_external_installation` |
| Directory or resolved link failing Structural Validation | `invalid_installation_candidate` |
| Ordinary or special root file | `unexpected_agent_root_entry` |
| `.DS_Store`; Codex legacy `.system` | omitted artifact |

### 5. Good / Base / Bad Cases

- Good: Rust returns a discriminated entry and React switches on `kind`.
- Base: a diagnostic path equal to the logical path is shown only once.
- Bad: label every failed linked package as broken, or infer entry kind from a
  localized error string in React.

### 6. Tests Required

- Rust tests cover valid directory, broken link, invalid content behind a
  healthy link, ordinary file, `.DS_Store`, and owner-aware `.system` handling.
- Diagnostics tests assert attention kind/path/error fields and absence of
  content and resolved targets.
- React tests assert actionable offending paths, Managed-only enabled-filter
  behavior, Settings detail visibility, and locale changes without new
  `inventory` or `state_status` calls.

### 7. Wrong vs Correct

Wrong:

```ts
const broken = entry.logicalPath.endsWith(".link") || error.message.includes("link");
```

Correct:

```ts
switch (entry.kind) {
  case "broken_external_installation":
  case "invalid_installation_candidate":
  case "unexpected_agent_root_entry":
    renderAttention(entry);
}
```

## Scenario: Managed Installation reconciliation and recovery

### 1. Scope / Trigger

Apply this contract whenever Managed Installation inventory, recovery actions,
or package mutation eligibility changes. Reconciliation is read-only evidence;
only plan/commit commands authorize writes.

### 2. Signatures

```text
inventory() -> {
  managedInstallationStatuses: ManagedInstallationReconciliation[],
  managedPackageReconciliations: ManagedPackageReconciliation[], ...
}
plan_restore_installation(packageId, agent) -> {
  operation: recreate|replace, rootExists, willOverwrite, ...
}
commit_restore_installation(planId, confirmOverwrite, confirmCreateRoot)
plan_forget_installation(packageId, agent) -> ForgetInstallationPlan
commit_forget_installation(planId) -> LifecycleResult
```

`ManagedInstallationReconciliation` carries `packageId`, `agent`, primary
`status`, structured `diagnostic`, typed expected/observed `evidence`, and a
closed `availableActions` set. `ManagedPackageReconciliation` carries
`packageId`, one optional `libraryDiagnostic`, and package actions.

### 3. Contracts

Writable inventory derives one `managedInstallationStatuses` entry per
package-id/Agent pair and one `managedPackageReconciliations` entry per package.
The primary status precedence is `broken`, `missing`, `retargeted`, `drifted`,
`configuration_drift`, then `healthy`. Rust returns typed expected/observed
evidence and the closed `availableActions` sets; React renders them without
reconstructing ownership policy. A Managed Library failure is diagnosed once at
package level and makes every affected Installation `broken`, including when
the package has no Installations.

- Restore previews declare `operation: recreate|replace`; only `replace`
  requires overwrite confirmation. Recreating a missing Agent root requires a
  separate explicit confirmation and preserves the recorded deployment mode.
- Drifted, structurally valid Copy Fallback content may be detached without
  changing its bytes or configuration.
- `plan_forget_installation` / `commit_forget_installation` is available only
  for `missing|retargeted|broken`. Commit reruns reconciliation and removes only
  the persisted Installation record; a changed status invalidates the plan.
- A zero-Installation package with an absent or real non-link app-owned package
  root may be removed even when its current revision is invalid. Unknown or link
  topology remains read-only, and exact-name confirmation remains mandatory.
- Any non-healthy Installation removes Install, Update, Replace, and Roll Back
  from package `availableActions` until repair or explicit Forget completes.

Inventory evidence is explanatory only. Every mutation rereads state,
filesystem topology, Managed Library content, and configuration provenance at
commit time. Diagnostics exports omit reconciliation targets and full
fingerprints.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Managed Library missing/invalid/drifted | Package diagnostic; affected Installations `broken` |
| Logical path absent | `missing`; Restore/Forget according to library health |
| Link resolves away from recorded/current library target | `retargeted`; Forget only |
| Valid Copy Fallback fingerprint mismatch | `drifted`; Restore or Detach |
| Skill Deck-owned config shape/value changed | `configuration_drift`; Reapply or Forget Configuration |
| Deployment shape invalid, link unresolved, or installed structure invalid | `broken`; Forget only |
| Restore root disappears after preview | stale/invalid plan; never create without confirmation |
| Forget status becomes ineligible before commit | stale/invalid plan; preserve state and filesystem |
| Any Installation non-healthy during Install/Update/Replace/Roll Back plan or commit | reject with the reconciliation diagnostic |
| Broken package root missing or real non-link app-owned directory | exact-name Remove allowed at zero Installations |
| Broken package root link/unknown topology or invalid package name | reject without traversal or deletion |

### 5. Good / Base / Bad Cases

- Good: inventory reports a retargeted link with expected/observed targets and
  only `forget_installation`; commit reruns reconciliation before dropping the
  state record and leaves link, target, and configuration bytes unchanged.
- Base: a healthy Installation exposes ordinary lifecycle actions but not
  Restore; externally controlled configuration remains healthy while config
  mutation actions stay absent.
- Bad: React derives actions from status, Restore silently creates a vanished
  Agent root, or a stale/direct package command bypasses the non-healthy freeze.

### 6. Tests Required

- Rust classification tests cover all six statuses, precedence, Copy Fallback
  `resolvedTarget`, package-level diagnostic deduplication, and zero-Installation
  library failure.
- Restore tests cover recreate/replace, overwrite confirmation, explicit root
  creation, and a root disappearing after preview.
- Forget tests assert eligible statuses, stale-status rejection, state-only
  removal, and byte-preservation of link, target, and configuration.
- Package mutation tests call plan and commit directly and prove non-healthy
  Installations cannot bypass Install/Update/Replace/Roll Back freezes.
- Broken removal tests cover safe real root, absent root without staging
  creation, unsafe topology, exact-name confirmation, and name traversal.
- React tests assert exhaustive backend action rendering, one package-level
  library diagnostic, evidence disclosure, freeze reasons, and both locales.
- Diagnostics tests assert resolved targets and full fingerprints remain absent.

### 7. Wrong vs Correct

Wrong:

```ts
if (entry.status === "missing") actions.push("restore");
await commitRestoreInstallation(plan.id, false, true);
```

Correct:

```ts
for (const action of entry.availableActions) renderManagedAction(action);
await commitRestoreInstallation(
  plan.id,
  plan.operation === "replace" && confirmOverwrite,
  !plan.rootExists && confirmCreateRoot,
);
```

The backend action set explains current eligibility; commit-time reconciliation
remains the authority when state changes after inventory or preview.

## Scenario: Agent Projection Contract smoke

### 1. Scope / Trigger

Apply this contract whenever Codex or Claude Skill roots, configuration formats,
native link behavior, inventory reconciliation, or packaging CI changes.

### 2. Contracts

- Codex projects user Skills to `$HOME/.agents/skills` and records Skill Deck-owned
  overrides in `CODEX_HOME/config.toml` as `[[skills.config]]` entries targeting
  the installed `SKILL.md`.
- Claude projects user Skills to `CLAUDE_CONFIG_DIR/skills` and records overrides
  in `settings.json.skillOverrides`.
- Contract tests inject temporary `AgentRoots`; they never mutate process-global
  `HOME`, `CODEX_HOME`, or `CLAUDE_CONFIG_DIR` and never access the network.
- Both Agents run Import, native linked Install, Healthy, Configuration Drift,
  Reapply, Missing, Restore, and final Healthy checks. Retarget/Forget separately
  proves external content remains unchanged.
- Every native packaging runner executes the focused smoke test before artifact
  production. A passing projection test does not claim runtime discovery or Skill
  invocation.

Official sources (last reviewed: 2026-08-11):

- Codex Skills: https://learn.chatgpt.com/docs/build-skills
- Codex environment variables: https://learn.chatgpt.com/docs/config-file/environment-variables
- Claude Skills: https://code.claude.com/docs/en/skills
- Claude settings: https://code.claude.com/docs/en/settings
- Claude environment variables: https://code.claude.com/docs/en/env-vars

Release preparation must manually revalidate these sources; CI remains offline.

### 3. Tests Required

```bash
cargo test --manifest-path src-tauri/Cargo.toml --all-features projection_contract_smoke
```

Run this before `tauri build` on Windows, macOS, and Linux. Windows exercises
junctions; macOS and Linux exercise symlinks. The macOS runner does not prove
that the smoke binary executed both architectures in a universal artifact.
