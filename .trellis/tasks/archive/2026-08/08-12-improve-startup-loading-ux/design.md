# Technical Design

## Boundary and data flow

Keep the existing runtime command and React-local state. Extend every
`RuntimeStatus` DTO variant with the validated startup Inventory:

```text
React runtime_status()
  -> spawn_blocking
    -> resolve/pin skills version
    -> skills list -g --json once
    -> { ready, version, nodeVersion, inventory }
  -> set runtime and Inventory in one frontend state transition
```

`inventory: InstalledSkill[]` is required, not optional. On failure it is empty,
and the existing stable runtime error contract
and Retry UI remain unchanged. `retry_runtime` returns the same DTO shape after
clearing the failed session. Remove the now-unused public `list_skills` Tauri
command and frontend wrapper; mutation results continue to replace Inventory.

The initialization path must return the exact `Vec<InstalledSkill>` produced by
its compatibility-validating `list_with` call while still updating the existing
name-to-root map used by Preview. It must not call `list_with` again and must not
add a second full-Inventory cache. This preserves startup validation against
real list JSON without a cache, a second command, or a new state machine. It
also makes “not installed” observable only after the startup response is
complete.

Keep `skills@latest` resolution and process-lifetime version pinning unchanged.
The measurable performance claim is one fewer global Inventory CLI invocation,
not a fixed startup duration.

## Presentation

- Split the shared loading copy into startup (`loadingSkills`) and translation
  (`translating`) strings in both catalogs.
- Pair startup copy with one CSS-only indeterminate spinner. Stop its animation
  under `prefers-reduced-motion`; add no component or animation dependency.
- Remove only the rendered Agent tag block. Keep `InstalledSkill.agents` in the
  cross-layer DTO, but filter only by visible name, source, and path fields.
- Distinguish a genuinely empty Inventory from a non-empty Inventory with no
  filter matches; the latter gets dedicated localized “no matches” copy.
- Reduce row minimum height after tag removal.
- Render the no-selection instruction as one compact paragraph at the top of
  the detail pane; remove its decorative file icon and centered layout.

## Find and install placement

Remove the persistent `.discovery` section from the bottom of the Inventory
flex column. Add one “Find & install” action beside the Installed Skills heading.
It opens a right-side sheet using the existing backdrop, surface, spacing, close
button, focus styles, and responsive full-width behavior already used by
Settings. Move the existing catalog search/results and source-install forms into
that independently scrollable sheet without changing their command handlers or
introducing tabs, routing, or a new dependency.

The signature is structural rather than decorative: Installed Skills owns the
list; Find & install is an explicit task surface. This gives both operations a
stable amount of space and keeps the primary sidebar quiet.

## Translation proxy verification

The proxy endpoint itself is proven reachable with the same provider URL and a
non-sensitive `hello` request. The running GUI lacks fields already present in
current `SettingsDialog.tsx`, so treat it as a stale-build signal. Build and
launch the current app, confirm the proxy draft is applied, and exercise the
real `translate_preview` path before considering code changes. Preserve the
5-second connect timeout and shared 15-second operation deadline unless that
current-build loop remains red and isolates them as the cause.

Current-build testing isolated long Markdown timeouts to one provider call per
protected prose fragment; four-way fragment concurrency still exceeded the
shared deadline. HTML-escape fragment bodies and pack indexed inert spans into
provider-sized batches. Run at most four batches concurrently, strictly validate
the exact sequential unique markers, reject raw tags or unknown entities, decode
only the emitted entities, and assemble borrowed source ranges and detected
language in document order only after every batch succeeds. Size escaped batches
in Unicode characters and split an oversized fragment safely. Use scoped
standard-library workers; add no runtime or dependency.
Contain a worker panic as an atomic internal failure so protected Markdown and
partial translations never escape.

## Compatibility and rollback

The DTO change is internal to the jointly shipped Tauri/React desktop bundle;
there is no persisted migration. Existing mutation and Preview contracts do not
change. Rollback is the product-code commit; no cached state needs cleanup.

## Trade-offs

- Do not retain a refresh-only `list_skills` command with no caller.
- Do not add cached Inventory: the single startup list is already required for
  compatibility validation and supplies the first frame.
- Do not summarize Agent tags (for example “9 agents”): the user identified the
  row metadata as non-actionable.
- Removing `list_skills` changes `.trellis/spec/backend/command-contracts.md`;
  the implementation must update that command signature before final check.
- Reuse the existing sheet pattern instead of creating an install navigation
  system or keeping a second constrained sidebar scroller.
