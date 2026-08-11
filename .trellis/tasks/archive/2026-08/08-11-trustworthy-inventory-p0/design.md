# Design: trustworthy inventory P0

## Boundary and data flow

```text
state.json + Agent roots + Managed Library + Agent config
  -> Rust reconciliation
  -> Inventory.managedInstallationStatuses
  -> typed TypeScript DTO
  -> Managed Installation row/status/actions
  -> existing plan/commit or configuration resolution command
  -> refresh inventory
```

Rust remains the only owner of filesystem classification and mutation safety. React renders typed status and chooses from the closed status/action table; every action still relies on backend revalidation.

## Inventory contract

Add a derived `managedInstallationStatuses` array to `Inventory` rather than changing persisted `ManagedSkillPackage` or `Installation`:

- `packageId`
- `agent`
- `status: healthy | missing | drifted | retargeted | configuration_drift | broken`
- `diagnostic: InventoryDiagnostic | null`
- `evidence`: typed status-specific expected/observed paths, deployment mode, targets, fingerprints, and configuration provenance
- `availableActions`: a closed Rust-owned action enum

Add a `managedPackageReconciliations` array keyed by package id. Each entry contains Rust-owned available package actions and at most one structured current-library diagnostic, allowing a zero-Installation package to report a broken Managed Library. Installation rows still receive the primary `broken` status, but React renders the shared package root cause once.

The serialized enum remains `broken`; domain/UI copy uses Broken Managed Installation so it cannot be confused with the existing Broken External Installation attention kind.

The package id plus Agent pair is already unique in lifecycle lookup and lets the frontend attach status to the existing installation row with minimal DTO churn. State serialization stays at version 1. Evidence is explanatory and does not authorize mutation.

## Reconciliation algorithm

Implement one non-mutating classifier in the backend and reuse existing validation primitives.

1. Validate the app-owned Managed Library revision against `installedRevision`.
2. Inspect the logical path with `symlink_metadata`.
3. If absent, return `missing`.
4. Check deployment shape:
   - Copy Fallback must be a real directory.
   - Symlink must be a symlink.
   - Junction uses the existing Windows topology rule.
5. Resolve linked targets and compare actual and recorded targets with the canonical current library revision. A resolvable mismatch is `retargeted`; an unresolvable link is `broken`.
6. Validate Copy Fallback content. A valid Skill with a mismatched fingerprint is `drifted`; invalid structure is `broken`.
7. Validate Skill Deck-owned configuration. A stable `configuration_drift` error maps to that status; other errors map to `broken`.
8. Otherwise return `healthy`. External Configuration Provenance is disclosed as externally controlled rather than treated as drift.
9. Derive the ownership-safe available-action set in Rust from the completed reconciliation result and provenance.

This ordering implements the PRD precedence and ensures one repair at a time. When an earlier check blocks later checks, the diagnostic explains that they were deferred; a refresh after repair may reveal the next lower-priority issue.

## Recovery behavior

### Missing Restore

Extend the existing Restore plan/commit family instead of creating a second recovery command.

- Plan accepts a missing installation in addition to a drifted Copy Fallback.
- The preview records deployment mode and `operation: recreate | replace`; only `replace` sets the overwrite warning and requires overwrite confirmation.
- Commit revalidates state/library and requires the destination to remain absent.
- If the Agent Root is missing, the plan reports the directories that would be created and commit requires explicit confirmation; no root is created implicitly.
- Copy Fallback recreates the directory through staging and atomic rename.
- Symlink/junction reuses the existing install projection helper for the recorded mode; it does not silently fall back to copy.
- A successful commit keeps the same installation record and refreshes its last-known fingerprint.

### Drifted Detach

Relax Detach only for a structurally valid drifted Copy Fallback. It removes the management record without modifying the observed directory or its configuration. Linked, retargeted, missing, or broken projections remain blocked.

### Configuration drift

Reuse `resolve_configuration` directly. The row exposes Reapply/Forget immediately based on reconciliation status; the existing mutation still verifies provenance and current bytes.

### Forget Installation

Add a lifecycle plan/commit pair for an explicit state-only escape hatch. It is offered for `missing`, `retargeted`, and `broken` records, re-reads state, verifies that the plan still identifies the same Installation, re-runs reconciliation, and rejects the plan if the primary status is no longer eligible. Commit removes only that persisted record and does not modify the Agent path, resolved target, or configuration. The preview states that observed artifacts become external and may remain unusable.

### Broken Managed Library removal

Extend Remove from Library only for a zero-Installation package whose package root is absent or is still a real, non-link directory under the app-owned Managed Library boundary. Do not require the current revision to pass Structural Validation. Preserve exact-name confirmation and add an explicit warning that invalid content cannot be exported or recovered. If the package root has unknown or link topology, remain read-only rather than following or deleting it.

## Frontend

- Build one lookup from the derived status array per inventory response.
- Render a localized status badge, logical path, and diagnostic beside each installation.
- Use exhaustive switches for status/evidence presentation and render only backend-provided available actions. Existing dialogs are reused for Restore and Detach; Reapply/Forget call the existing resolution handler directly.
- Healthy and unhealthy rows both render only their backend-provided actions; existing dialogs and handlers are reused where the action remains available.
- Healthy rows omit Restore. If any Installation status is non-Healthy, backend package action availability freezes Install, Update, Replace, and Roll Back until reconciliation returns all Healthy or abnormal records are forgotten.
- Display full expected/observed paths and abbreviated fingerprints. Keep full fingerprints in the DTO for local diagnosis; do not parse localized diagnostic messages.
- Externally Controlled Configuration is an adjacent provenance disclosure on a healthy row, not another Installation Status.
- Complete expected/observed evidence remains in the local inventory DTO only. Diagnostics export preserves its existing privacy boundary and omits resolved targets and full fingerprints.
- Do not split a new component unless the row interaction becomes independently reusable during implementation.

## Compatibility and safety

- No state migration or automatic filesystem write occurs during inventory.
- Read-only recovery continues to omit managed state reconciliation because no trustworthy state is available.
- Retargeted or broken paths are never overwritten or removed; Forget Installation changes only persisted ownership state.
- Plan/commit stale checks remain authoritative; a status can become stale between scan and click without weakening safety.
- No batch Forget-and-Remove command is added; state ownership changes remain one Installation per preview/commit.
- P0 does not reconstruct a broken Managed Library from a source or Previous Revision because those flows require source-specific recovery transactions.

## Rollback

The API addition is backward-compatible inside this single desktop release. If UI integration fails, the derived field can remain unused; no persisted data changes need rollback. Mutation failures use existing staging/state rollback rules. A missing package root removes only the state record; a safe real package root continues to use rename-to-staging before state commit.
