# Complete trustworthy inventory P0

## Goal

Turn the home inventory into an active reconciliation result: immediately after startup, every persisted Managed Installation explains whether its filesystem projection, installed revision, and Skill Deck-owned Agent configuration still match recorded state, and exposes only recovery actions that preserve ownership boundaries.

## Background

- `docs/roadmap.md:21-30` defines R1 and requires `healthy`, `missing`, `drifted`, `retargeted`, `configuration_drift`, and `broken` Managed Installation states.
- `src-tauri/src/inventory.rs:100-130` currently scans external entries, loads persisted packages, and removes matching paths from external/attention results without reconciling Managed Installations.
- `src-tauri/src/lifecycle.rs:372-440` already validates library fingerprints, deployment topology, link targets, and Copy Fallback fingerprints, but only as mutation preflight errors.
- `src/App.tsx:152-201` exposes configuration Reapply/Forget only after a toggle attempt fails with `configuration_drift`.
- Existing Restore handles only a present, drifted Copy Fallback installation; missing projections are not recoverable from inventory.

## Requirements

### R1. Reconciliation result

- Inventory returns one typed reconciliation entry for every persisted Managed Installation, keyed by package id and Agent.
- Each entry has exactly one primary status using this precedence: `broken`, `missing`, `retargeted`, `drifted`, `configuration_drift`, `healthy`.
- A primary status selects the safest next action; when an earlier check prevents later checks, the diagnostic states that lower-priority checks were deferred until the next inventory refresh.
- Rust, not React, owns classification, structured diagnostic `{code,message,path}`, reconciliation evidence, and the available-action set. React never reconstructs ownership policy from status or evidence.
- Reconciliation checks the recorded logical path, resolved target, deployment mode, last-known and installed revision fingerprints, Managed Library revision, and configuration provenance.
- Inventory also returns package-level reconciliation containing Rust-owned available package actions and one Managed Library diagnostic when the current library revision is missing, invalid, or has a mismatched fingerprint, including for zero-Installation packages. Installation rows affected by that failure are `broken`, while the root cause is displayed once at package level.

### R2. Status semantics

- `missing`: the recorded logical path does not exist.
- `retargeted`: a recorded symlink/junction still exists but resolves somewhere other than the app-owned current library revision.
- `drifted`: a structurally valid Copy Fallback installation fails to equal both its recorded fingerprint and the current library revision fingerprint; a mismatch against either expected value is drift.
- `configuration_drift`: the installation projection is otherwise healthy, but a Skill Deck-owned configuration entry no longer has the recorded value/shape.
- `broken`: the UI/domain label is Broken Managed Installation. It means the library revision is missing/invalid, the recorded deployment shape is incompatible with the path, a link cannot be resolved safely, installed content is structurally invalid, or reconciliation encounters another non-recoverable topology/validation error. It is distinct from Broken External Installation.
- `healthy`: all applicable checks pass. External or absent configuration provenance is not mislabeled as Skill Deck-owned drift.
- A healthy Installation with External Configuration Provenance remains `healthy`, but explicitly discloses Externally Controlled Configuration and does not offer Enable/Disable, Reapply, or Forget Configuration.

### R3. Home-page disclosure

- Each Managed Installation row displays its localized status, logical path, and a structured diagnostic when unhealthy.
- Status-specific Reconciliation Evidence exposes expected and observed targets/fingerprints as typed fields; UI displays full paths and abbreviated fingerprints while retaining full values for local diagnosis.
- Search/filter identity remains the Skill name and Agent; status is an adjacent badge, not a replacement heading.
- Configuration drift is visible immediately after inventory load, without first attempting an Enable/Disable mutation.
- Both `zh-CN` and `en` catalogs cover every new status and action description.

### R4. Recovery actions

- `drifted` Copy Fallback: offer Restore and Detach. Restore overwrites only after the existing explicit confirmation; Detach keeps the observed standalone content and configuration.
- `missing`: offer Restore and Forget Installation when the app-owned library revision is healthy; if the library is broken, only Forget Installation remains. Restore recreates the recorded deployment mode without overwriting another entry. If the Agent Root is also absent, creating it requires explicit confirmation in the Restore preview.
- `configuration_drift`: offer Reapply and Forget through the existing backend resolution command.
- `retargeted` and `broken`: do not overwrite, remove, or claim the observed path/target. Offer Forget Installation as a state-only ownership escape hatch; it leaves the path, target content, and configuration untouched.
- Restore previews distinguish `recreate` from `replace`: only `replace` reports content overwrite and requires overwrite confirmation.
- Forget Installation commit re-runs reconciliation and is stale if the Installation is no longer `missing`, `retargeted`, or `broken`.
- `healthy`: offer Enable/Disable when configuration is Skill Deck-controlled, plus Detach and Uninstall. Do not offer Restore when there is nothing to restore.
- A zero-Installation package whose app-owned package root is absent or remains provably safe may still be removed after exact-name confirmation and an explicit warning that invalid content cannot be exported or recovered. Unknown or unsafe package-root topology remains read-only.
- Every mutation re-reads state and topology; inventory status is explanatory evidence, never mutation authorization.
- Rust returns `availableActions` for every reconciliation entry; React renders that closed set exhaustively. Commit-time validation remains authoritative.
- There is no batch Forget-and-Remove operation. Each Installation is forgotten explicitly before a zero-Installation package can be removed.
- If any Installation is not `healthy`, freeze Install-to-another-Agent, Update, Replace, and Roll Back for the whole package. Repair or Forget all abnormal Installations before reopening revision-expanding actions.

## Acceptance Criteria

- [ ] Startup inventory classifies healthy link/junction and Copy Fallback installations as `healthy`.
- [ ] Removing a managed logical path yields `missing`; Restore recreates the recorded projection from a validated app-owned library revision and a refresh yields `healthy`.
- [ ] Restore never creates a missing Agent Root without explicit confirmation.
- [ ] A missing Installation can be forgotten without touching filesystem or configuration state.
- [ ] Redirecting a managed link to another target yields `retargeted` and exposes no destructive recovery action.
- [ ] Editing valid Copy Fallback content yields `drifted`; Restore and Detach are both usable, with the documented overwrite/keep-content semantics.
- [ ] Replacing the expected deployment shape, breaking a link, invalidating installed content, or damaging/removing the library revision yields `broken` with a structured diagnostic; the only mutating escape is state-only Forget Installation when eligible.
- [ ] Editing a Skill Deck-owned Codex or Claude configuration entry yields `configuration_drift` on initial inventory; Reapply and Forget each resolve it as specified and refresh the inventory.
- [ ] Forget Installation removes only the persisted Installation record and leaves the observed path, target, and configuration byte-for-byte unchanged.
- [ ] Forget Installation rejects a stale preview when the Installation has become healthy, drifted, or configuration-drifted.
- [ ] A damaged or missing current Managed Library revision is diagnosed once at package level, including for a package with zero Installations; affected Installation rows are `broken`.
- [ ] A broken zero-Installation package can be removed only when its app-owned package-root topology is absent or safe; exact-name confirmation and unrecoverable-content warning remain mandatory.
- [ ] React exhaustively renders all six backend statuses and does not infer them from paths or message text.
- [ ] React exhaustively renders backend-provided available actions and never duplicates status-to-action ownership rules.
- [ ] Retargeted and drifted rows show typed expected/observed evidence; fingerprints are abbreviated only for display.
- [ ] Externally Controlled Configuration can coexist with `healthy`, is visibly disclosed, and never exposes Skill Deck-owned configuration mutations.
- [ ] Multiple broken Installations require individual Forget confirmations; no batch ownership change is introduced.
- [ ] Healthy Installations do not display Restore, and any non-Healthy Installation freezes package Install/Update/Replace/Roll Back actions with a visible reason.
- [ ] Managed and External broken states use distinct user-facing domain labels.
- [ ] Full reconciliation targets/fingerprints remain local to inventory/UI; diagnostics exports omit resolved targets and full fingerprints.
- [ ] Existing external inventory classification, read-only recovery, ownership safeguards, and lifecycle operations continue to pass.
- [ ] Focused Rust and React regressions cover status precedence, action visibility, Restore/Detach behavior, and both locales.

## Out of Scope

- R1 host contract smoke tests and three-platform manual installer smoke tests (the previously assessed P1 work).
- Capability Cards, a third Agent target, revision-level eval, router/evolution, marketplace, SQLite migration, or background repair.
- Rebuilding a broken Managed Library from Previous Revision, Git Skill Source, or Local Skill Source; P0 diagnoses and provides ownership-safe exit paths only.
- Automatic repair during inventory, overwriting retargeted paths, deleting unknown targets, or inferring ownership from a link alone.

## Technical Notes

- Keep persisted `state.json` version and `Installation` shape unchanged; reconciliation is derived inventory data.
- Prefer reusing the existing lifecycle, configuration inspection, library validation, and install projection helpers over duplicating filesystem rules.
- No blocking product or scope questions remain; this plan implements the P0 scope agreed in the preceding roadmap assessment.
