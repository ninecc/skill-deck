# Research: Official `skills@latest` CLI contract for a desktop GUI

- Query: What official `vercel-labs/skills` contracts can a non-interactive, global-only GUI rely on for runtime requirements, inventory, discovery, add/remove/update, Agent IDs, telemetry, output, and compatibility?
- Scope: mixed (official source, official service/API, official npm registry/docs, and local task/spec context)
- Date: 2026-08-11
- Verified release: npm `latest` = `skills@1.5.22` (queried with `npm view skills dist-tags.latest version engines --json`); the tagged source below is used wherever possible.

## Findings

### Executive contract

For `1.5.22`, the only purpose-built machine-readable CLI result needed by this GUI is:

```text
npx --yes skills@latest list -g --json
```

Its stdout is a JSON array. `find`, `add`, `remove`, and `update` have no JSON mode. Their terminal output is presentation text and their exit codes do not consistently distinguish every partial or semantic failure. Consequently:

1. Use argument arrays without a shell.
2. Put npm/npx's `--yes` **before** `skills@latest` to suppress npx's package-download prompt, and put the skills CLI's `-y` **after** the subcommand arguments to suppress skills prompts.
3. Always pass `-g` for this product's global-only boundary.
4. After every successful-looking mutation, refresh from `list -g --json`; do not reconstruct inventory from mutation output.
5. Treat command stdout/stderr as diagnostic text, not a stable data protocol.

This matches the task's “CLI owns lifecycle; GUI refreshes inventory” boundary (`prd.md:23-32`, `prd.md:56-61`) but contradicts the earlier runtime statement at `prd.md:15`: the current package requires Node `>=22.20.0`, not Node `>=18`.

### Node, npm, and npx requirements

- `skills@1.5.22` declares `engines.node: ">=22.20.0"`; its executable is `bin/cli.mjs` ([tagged `package.json`:1-7,140-142](https://github.com/vercel-labs/skills/blob/v1.5.22/package.json#L1-L7)).
- The package declares **no npm engine range**. The application therefore cannot truthfully enforce a package-defined minimum npm version; it can require that `npx` exists and runs.
- npm's official installation guide treats Node.js and the npm CLI as separately checkable prerequisites (`node -v`, `npm -v`) and notes that Node installers normally install npm ([npm installation guide](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm/)).
- Modern `npx` is the `npm exec` front end. When a requested package is absent locally it installs it into the npm cache; npx prompts before doing so unless its own `-y`/`--yes` is supplied ([official npx docs](https://docs.npmjs.com/cli/commands/npx/)). This outer prompt is independent of `skills add/remove/update -y`.
- Minimal preflight: resolve `node` and `npx` from the inherited desktop `PATH`, parse `node --version`, require `>=22.20.0` for the verified release, then execute `npx --yes skills@latest --version`. The actual returned skills version must be recorded before accepting its JSON.

### `list -g --json` schema

The implementation recognizes `-g|--global`, `-a|--agent`, and `--json` ([`src/list.ts`:51-68](https://github.com/vercel-labs/skills/blob/v1.5.22/src/list.ts#L51-L68)). JSON mode maps installed skills to exactly these fields in `1.5.22` ([`src/list.ts`:104-119](https://github.com/vercel-labs/skills/blob/v1.5.22/src/list.ts#L104-L119)):

```ts
type SkillsListJsonV1_5_22 = Array<{
  name: string;
  path: string;
  scope: string;
  agents: string[];
  source: string | null;
  sourceUrl: string | null;
  sourceType: string | null;
}>;
```

Semantics and validation notes:

- `path` comes from `skill.canonicalPath`; it is not the shortened `~` display path.
- `agents` contains Agent **display names**, not `--agent` IDs (`agents[a].displayName`). It is untruncated in JSON mode.
- The three source fields are nullable when no lock entry supplies them. `source` alone is insufficient to preserve every source detail.
- An empty inventory prints `[]` and exits normally. Invalid `--agent` filters print human-readable valid/invalid lists and exit 1 ([`src/list.ts`:76-85](https://github.com/vercel-labs/skills/blob/v1.5.22/src/list.ts#L76-L85)).
- The output has no top-level schema or CLI-version field. Validate the top-level array and required field types, tolerate unknown additional fields, and gate the contract using the separately captured `--version`.

### Discovery / `find`

- `find` uses `https://skills.sh/api/search` (overrideable internally by `SKILLS_API_URL`) with `q`, fixed `limit=20`, and optional `owner` query parameters ([`src/find.ts`:14-16,79-104](https://github.com/vercel-labs/skills/blob/v1.5.22/src/find.ts#L14-L16)).
- The service response consumed by the CLI is `{ skills: [{ id, name, installs, source }] }`; the CLI maps that to `{ name, slug: id, source, installs }` and sorts by installs descending. A live official response verified on 2026-08-11 also included top-level `query`, `searchType`, `count`, and `duration_ms`, plus per-item `skillId`; these extra fields are not consumed by the CLI.
- `skills find <query> [--owner <github-owner>]` is non-interactive, but prints ANSI-decorated, human-oriented lines such as `source@name`, install counts, and skills.sh URLs ([`src/find.ts`:302-342](https://github.com/vercel-labs/skills/blob/v1.5.22/src/find.ts#L302-L342)). There is no `--json` option.
- With no query in a non-TTY, it prints a tip/usage message. No matches print a message and return success. Invalid `--owner` input prints errors and returns without setting a failing exit code ([`src/find.ts`:309-349](https://github.com/vercel-labs/skills/blob/v1.5.22/src/find.ts#L309-L349)).

Therefore the CLI itself offers no stable machine-readable discovery contract. The smallest viable GUI choice is a narrow client for the official `/api/search` JSON response, explicitly treated as an unversioned service contract. Do not parse `skills find` terminal text. If the product insists that every operation pass through the CLI, discovery must remain a displayed terminal-style result or be deferred until upstream adds JSON.

### Non-interactive add

Recommended global invocation shape:

```text
npx --yes skills@latest add <source> -g -y [--skill <name> ...] [--agent <id> ...] [--copy]
```

- Supported relevant flags are `-g|--global`, `-y|--yes`, `-a|--agent`, `-s|--skill`, `--copy`, and `--all`; the parser also accepts low-frequency `--list`, `--metadata`, `--full-depth`, and `--subagent` ([`src/add.ts`:1997-2067](https://github.com/vercel-labs/skills/blob/v1.5.22/src/add.ts#L1997-L2067)).
- `--all` is broader than “all skills”: it expands to `--skill '*' --agent '*' -y` ([`src/add.ts`:991-995](https://github.com/vercel-labs/skills/blob/v1.5.22/src/add.ts#L991-L995)). Do not use it for the GUI's ordinary single-skill install.
- With `-y` and no explicit Agent IDs, the CLI auto-selects detected Agents plus its universal Agents; if it detects none, it targets every known Agent. With multiple discovered skills and no `--skill`, `-y` selects all skills ([`src/add.ts`:597-646,666-668](https://github.com/vercel-labs/skills/blob/v1.5.22/src/add.ts#L597-L646)). A GUI installing one search result should pass its exact `--skill` name.
- `--copy` selects copy; otherwise non-interactive mode starts from symlink mode, though a single unique target directory collapses to copy and symlink failure can fall back to copy ([`src/add.ts`:716-742](https://github.com/vercel-labs/skills/blob/v1.5.22/src/add.ts#L716-L742)). The final filesystem result remains CLI-owned.
- Success and failure details are human-formatted. Critically, per-target failures are printed but do **not** set a nonzero exit status; the command still prints `Done!` ([`src/add.ts`:1884-1896](https://github.com/vercel-labs/skills/blob/v1.5.22/src/add.ts#L1884-L1896)). Fatal discovery/clone/validation errors do exit 1. Exit 0 therefore means only “the command reached normal completion,” not “every target installed.” Refresh inventory and compare the requested skill/targets.

### Non-interactive remove

Recommended global invocation shape:

```text
npx --yes skills@latest remove -g -y <exact-skill-name>...
```

- Relevant flags are `-g|--global`, `-y|--yes`, `-a|--agent`, and `--all`; `--skill` is documented but the `1.5.22` parser shown in source only separates positional skill names and these flags ([`src/remove.ts`:340-367](https://github.com/vercel-labs/skills/blob/v1.5.22/src/remove.ts#L340-L367)). Avoid relying on `--skill` for named GUI removal; pass names positionally.
- If `--agent` is omitted, remove targets all known Agents to clean ghost links ([`src/remove.ts`:164-171](https://github.com/vercel-labs/skills/blob/v1.5.22/src/remove.ts#L164-L171)).
- `-y` skips only confirmation; provide exact skill names to avoid the interactive selector ([`src/remove.ts`:137-189](https://github.com/vercel-labs/skills/blob/v1.5.22/src/remove.ts#L137-L189)).
- No installed skills, no matching skill, and cancellation all return success. Per-skill failures are printed but do not set a nonzero exit code; the command ends with `Done!` ([`src/remove.ts`:120-144,323-334](https://github.com/vercel-labs/skills/blob/v1.5.22/src/remove.ts#L120-L144)). Refresh `list -g --json` and verify absence rather than trusting status/output text.

### Non-interactive update (and why `check` is not a read-only contract)

Recommended global invocation shape:

```text
npx --yes skills@latest update -g -y [<exact-skill-name>...]
```

- Options are `-g|--global`, `-p|--project`, `-y|--yes`, plus positional names ([`src/update.ts`:54-71](https://github.com/vercel-labs/skills/blob/v1.5.22/src/update.ts#L54-L71)). Explicit `-g` is sufficient to avoid the scope prompt; `-y` also makes deleted-upstream handling non-interactive.
- `-y` does **not** mean “delete everything upstream removed.” Non-interactive update warns and skips those deletions ([`src/update.ts`:237-267](https://github.com/vercel-labs/skills/blob/v1.5.22/src/update.ts#L237-L267)).
- Updates are applied immediately by spawning the same CLI's `add ... -g -y` flow ([`src/update.ts`:619-666](https://github.com/vercel-labs/skills/blob/v1.5.22/src/update.ts#L619-L666)). There is no JSON result, dry run, or separate update-availability DTO.
- Although the dispatcher accepts both command names, `check` and `update` call the same `runUpdate(restArgs)` implementation ([`src/cli.ts`:360-368](https://github.com/vercel-labs/skills/blob/v1.5.22/src/cli.ts#L360-L368)). In this release, `check` is therefore mutating, not a safe read-only probe.
- The update command sets exit code 1 only when its accumulated update `failCount` is nonzero ([`src/update.ts`:879-933](https://github.com/vercel-labs/skills/blob/v1.5.22/src/update.ts#L879-L933)). Some source-check failures merely print a diagnostic and continue without incrementing that counter ([`src/update.ts`:376-382](https://github.com/vercel-labs/skills/blob/v1.5.22/src/update.ts#L376-L382)). Treat exit 0 plus output as insufficient evidence that all sources were checked; refresh inventory after execution and present raw sanitized diagnostics for failures.

### Supported Agent IDs

- The authoritative runtime set is the key set of the exported `agents` record; validation in list/add/remove uses `Object.keys(agents)` ([`src/agents.ts`:63 onward](https://github.com/vercel-labs/skills/blob/v1.5.22/src/agents.ts#L63), [`src/list.ts`:76-84](https://github.com/vercel-labs/skills/blob/v1.5.22/src/list.ts#L76-L84)).
- The official README publishes a Supported Agents table with each `--agent` ID and paths ([README Supported Agents](https://github.com/vercel-labs/skills/blob/v1.5.22/README.md#supported-agents)).
- There is no CLI command or JSON option that enumerates supported Agent IDs. `list --json` returns display names for Agents that contain an installed skill, so it cannot populate the override selector or reliably map display name back to ID.
- Supplying a deliberately invalid ID makes list/add/remove print a human-readable “Valid agents” list and exit 1, but that is an error-message side effect, not a supported discovery API; do not parse it.

The task's current decision is sound: automatic mode should omit `--agent` and inherit additions from the CLI, while the explicit override selector must ship a compatibility list tied to the app/version contract and be updated with the application (`prd.md:26-30`). Store stable IDs, not display names. On runtime rejection, return a structured GUI command failure and offer automatic mode; do not scrape the CLI's message into a new enum.

### Telemetry

- Telemetry is enabled by default. Either `DISABLE_TELEMETRY` or `DO_NOT_TRACK` disables it ([`src/telemetry.ts`:80-82](https://github.com/vercel-labs/skills/blob/v1.5.22/src/telemetry.ts#L80-L82)); the official README recommends value `1` and says this disables telemetry and security-audit requests ([README Telemetry](https://github.com/vercel-labs/skills/blob/v1.5.22/README.md#telemetry)).
- The implementation tests environment-variable presence/truthiness, not a parsed boolean. Even a non-empty value such as `"0"` disables telemetry; use the documented `1`.
- Events include install source/skills/Agents and optional skill-files/install URL metadata; remove, update, and find have their own fields ([`src/telemetry.ts`:2-56](https://github.com/vercel-labs/skills/blob/v1.5.22/src/telemetry.ts#L2-L56)). The CLI waits up to five seconds for pending telemetry at process exit, while swallowing request failures ([`src/telemetry.ts`:159-178](https://github.com/vercel-labs/skills/blob/v1.5.22/src/telemetry.ts#L159-L178)).
- A privacy-preserving desktop wrapper can consistently set `DO_NOT_TRACK=1` in the child environment, but that is a Skill Deck product decision, not the CLI default.

### `@latest` compatibility risks and minimum guard

npm defines `latest` as a mutable distribution tag that normally points at the publisher's stable release; tags are aliases, not version constraints ([official npm dist-tag docs](https://docs.npmjs.com/cli/dist-tag/)). Therefore `skills@latest` deliberately opts into contract changes between app launches—and potentially between separate invocations.

Observed concrete risks:

- Runtime floor drift is already real: the task's prior Node `>=18` fact (`prd.md:15`) became `>=22.20.0` in `1.5.22`.
- `list --json` is implemented but not declared as a versioned schema; fields, nullability, Agent display names, or meanings may change.
- Agent IDs and detection/path rules are source-owned and can be added, removed, or renamed without an enumeration API.
- Human output, ANSI behavior, prompt logic, and exit-code coverage are not machine contracts. Current add/remove partial failures and find validation demonstrate why output parsing is unsafe.
- `find` depends on an unversioned hosted JSON endpoint. Its live response already has more fields than the CLI consumes.
- `check` currently aliases mutating `update`; assuming command names imply semantics is unsafe.
- npx may need network access and may populate/use its cache. A package-download refusal, offline state, registry configuration, proxy, certificate issue, or npm cache problem is distinct from a skills operation failure.

Minimum guard compatible with the product's decision to retain `@latest`:

1. Run outer npx non-interactively (`npx --yes`).
2. Capture `skills@latest --version` once for the operation/session and show it.
3. Enforce a tested version compatibility policy (at minimum, known major and minimum required Node floor).
4. Validate `list -g --json` structurally before enabling management operations; never fall back to terminal parsing.
5. Keep Agent explicit overrides in an app-maintained list; automatic mode remains future-compatible by omitting `--agent`.
6. Serialize mutations, bound runtime/output, preserve separate stdout/stderr for diagnostics, refresh inventory after completion, and make no correctness decision from localized/presentation text.

Pinning `skills@1.5.22` would be the only way to make these exact contracts reproducible, but the task explicitly selects `@latest` (`prd.md:13`). The above is detection and fail-closed behavior, not equivalent stability.

## Files Found

- `.trellis/tasks/08-11-simplify-around-npx-skills/prd.md` — active product boundary and acceptance criteria; relevant lines 11-32 and 54-72.
- `.trellis/spec/backend/command-contracts.md` — existing structured-error/refresh principles; relevant lines 5-27 and 65-74, although its old closed Agent enum and preview/commit lifecycle are being replaced.
- `.trellis/spec/guides/cross-platform-thinking-guide.md` — process/path portability requirements; relevant lines 3-19 and 29-37.
- [`package.json` at `v1.5.22`](https://github.com/vercel-labs/skills/blob/v1.5.22/package.json) — package version, executable, and Node engine.
- [`src/cli.ts` at `v1.5.22`](https://github.com/vercel-labs/skills/blob/v1.5.22/src/cli.ts) — command dispatch, version output, and final exit behavior.
- [`src/list.ts` at `v1.5.22`](https://github.com/vercel-labs/skills/blob/v1.5.22/src/list.ts) — `list --json` parser and schema producer.
- [`src/find.ts` at `v1.5.22`](https://github.com/vercel-labs/skills/blob/v1.5.22/src/find.ts) — hosted search endpoint, response mapping, and terminal-only find output.
- [`src/add.ts` at `v1.5.22`](https://github.com/vercel-labs/skills/blob/v1.5.22/src/add.ts) — non-interactive selection, flags, install modes, and partial failure behavior.
- [`src/remove.ts` at `v1.5.22`](https://github.com/vercel-labs/skills/blob/v1.5.22/src/remove.ts) — removal targeting, flags, confirmation, and exit/output behavior.
- [`src/update.ts` at `v1.5.22`](https://github.com/vercel-labs/skills/blob/v1.5.22/src/update.ts) — update scope, immediate mutation, deletion handling, and failure accounting.
- [`src/agents.ts` at `v1.5.22`](https://github.com/vercel-labs/skills/blob/v1.5.22/src/agents.ts) — authoritative Agent ID/config registry and installed-Agent detection.
- [`src/telemetry.ts` at `v1.5.22`](https://github.com/vercel-labs/skills/blob/v1.5.22/src/telemetry.ts) — opt-out gate, payload fields, endpoints, and flush timeout.
- [Official Skills README at `v1.5.22`](https://github.com/vercel-labs/skills/blob/v1.5.22/README.md) — documented CLI options, Agent table, environment variables, and telemetry policy.
- [Official skills.sh CLI docs](https://www.skills.sh/docs/cli) — user-facing command and telemetry documentation.
- [Official npm npx docs](https://docs.npmjs.com/cli/commands/npx/) — npx package acquisition, cache, argument boundary, and outer confirmation behavior.
- [Official npm dist-tag docs](https://docs.npmjs.com/cli/dist-tag/) — semantics of the mutable `latest` tag.

## Related Specs

- `.trellis/spec/backend/command-contracts.md:5-7` — Rust/Tauri must own command semantics rather than React inventing them.
- `.trellis/spec/backend/command-contracts.md:65-66` — UI decisions should use structured errors, not parsed message text. Upstream skills does not provide them, so the wrapper must classify process/runtime/schema failures itself without pretending human output is structured.
- `.trellis/spec/backend/command-contracts.md:70-74` — refresh inventory after writes; directly applicable to CLI mutations.
- `.trellis/spec/guides/cross-platform-thinking-guide.md:8-19` — native path and link handling stays at the backend boundary.
- `.trellis/spec/guides/cross-platform-thinking-guide.md:31-37` — each target OS needs native runtime/process validation; a macOS check cannot establish Windows npx/path behavior.

## Caveats / Not Found

- No official JSON/machine-readable CLI mode was found for `find`, `add`, `remove`, `check`, or `update` in `1.5.22`.
- No official machine-readable CLI command was found for enumerating supported Agent IDs.
- No package-declared minimum npm version exists; only Node `>=22.20.0` is declared. `npx` availability/behavior must be probed operationally.
- No versioned schema commitment was found for `list --json` or the skills.sh search API.
- Current `check` is not a distinct read-only operation; it aliases mutating `update`.
- Add/remove may report per-item failures while exiting 0. Find owner validation and no-match cases also return 0. Update source-check errors can be printed without producing a nonzero exit. Exact mutation success requires post-operation inventory verification where expressible.
- Tagged GitHub source establishes the `1.5.22` implementation. `skills@latest` may resolve to a different version after this research date; re-run the version/schema gate rather than treating this file as a permanent upstream guarantee.
