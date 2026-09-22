# Skill Deck functionality completion audit

## Executive result

Refresh date: 2026-09-22 (Asia/Shanghai)  
Audited revision: `fc42e11e61c7faf463ded80b8d54df53c88877c5`  
Remediation revision: `32f04549f39c38f9a5d1913b704b4ad867df4a59`

**Conservative current-MVP completion: 69.6%.** All 27 inventoried current-MVP
capabilities have implementation plus executable or observed evidence, and no
capability is wholly absent. The score remains deliberately lower than
implementation presence because 15 integration-sensitive capabilities cannot
score above partial without current-HEAD real CLI, filesystem, provider,
native-menu, OS-helper, or packaged-application evidence. The four previously
confirmed implementation defects are now resolved and tested: catalog search is
independent of runtime/session state, Update All requires confirmation, active
startup/translation states no longer present as Ready, and empty/narrow
translation states remain recoverable with visible egress disclosure.

**Hypothetical implemented upper bound at the current implementation: 100%.**
This means every row now has a contract-compliant implementation and local
behavior evidence; it assumes all missing real/native integration checks later
pass. It is not a claim that the product is 100% complete or release-ready.

**Evidence coverage: 27/27 capability rows (100%).** Every row has a passing
automated behavior check, a deterministic UI observation, or both. Coverage only
means each row has some executable evidence: it does not mean every contract
branch is tested. Strong current-HEAD real-environment evidence is still absent
for the 15 integration-sensitive rows.

**UI completion: accepted MVP behavior is complete in the audited matrix.** The
accepted two-pane structure and major interaction surfaces exist, and the
previously confirmed empty/loading/translation defects now have focused tests
and recorded 1180x800/720x520 deterministic review. Command hierarchy, repeated
metadata, compact target sizing, and Settings density remain possible visual
polish, not demonstrated current-MVP functional defects. ADR-0016 still prevents
turning subjective aesthetic preference into a completion score.

**Release readiness: NOT READY.** Local frontend and Rust quality gates are
green and the package workflow now proves that its exact named contract test
exists once before executing it. Current-architecture native
installation/runtime-recognition smoke still does not exist for macOS, Windows,
or Linux. This is an evidence gap, not proof that the packaged applications are
functionally broken.

**Confirmed current-MVP implementation defects: none in this 27-row audit.**
Remaining work is verification and release evidence: hermetic CLI/filesystem
integration, controlled external-provider checks, native menus/helpers, and
current artifacts on all three platforms.

## Authority and method

The denominator follows accepted ADRs 0011-0016 and the active Trellis specs,
then `CONTEXT.md`. Current implementation and tests are evidence, not silent
product authority. `docs/roadmap.md` is historical where it describes the
Managed Library, ownership, reconciliation, projection, rollback, or other
pre-ADR-0011 lifecycle concepts.

Each of the six groups has equal weight. Within a group, each row has equal
weight and is scored `complete = 1`, `partial = 0.5`, `unimplemented = 0`.
Integration-sensitive rows are capped at partial without matching real/native
evidence. Ordinary rows require implementation plus a passing behavior test.

The raw capability count is also shown for transparency, but it is not the
headline calculation because the groups contain different numbers of rows.

## Capability matrix

Legend: **C** = complete (1), **P** = partial (0.5). Confidence describes the
conclusion supported by the cited evidence, not release confidence.

### 1. Runtime and Inventory — 1.5/3 = 50.0%

| Capability | Contract and implementation evidence | Executed/observed evidence | Status | Confidence and next action |
| --- | --- | --- | --- | --- |
| Discover Node/npx, enforce Node >=22.20, resolve `skills@latest` once and pin the compatible v1 version | `.trellis/spec/backend/command-contracts.md`; `CONTEXT.md:55-57`; `src-tauri/src/cli.rs:242-304,446-540` | Rust tests pin the declared minimum-version tuple and cover executable/sibling/path-normalization discovery (`src-tauri/src/cli.rs:685-693,769-827`); they do not directly drive the lower-version rejection branch. Local Node 24.17.0 was found, but the read-only `skills@latest --version` attempt again exceeded 30 s. | **P** | High implementation confidence; real session resolution is unverified due to network timeout. Add direct lower/equal/higher Node-version branch tests, then repeat on a network-capable host and retain only version/readiness. |
| Publish startup/Retry readiness and the exact atomic Inventory without a second list | `.trellis/spec/backend/command-contracts.md` atomic-startup scenario; `src-tauri/src/cli.rs:157-180,250-310`; `src/App.tsx:483-490,826-862` | `src/App.test.tsx:61-73,245-274`; serialized failure Inventory test at `src-tauri/src/cli.rs:733-746`; deterministic `shell-ready` and localized runtime-failure observations | **P** | High static/mocked confidence; real Inventory could not be read because version resolution timed out. Re-run read-only startup against the real CLI. |
| Structurally decode open Agent strings, keep CLI Inventory as lifecycle truth, and expose stable runtime errors | `CONTEXT.md:15-25,59-61`; `src-tauri/src/cli.rs:49-73,296-310,419-428,430-545`; `src/api.ts:3-29,109-133` | Open Agent/malformed-shape test at `src-tauri/src/cli.rs:676-681`, stable/sanitized runtime tests at `src-tauri/src/cli.rs:733-765`, and `src/api.test.ts:27-32`; 29/29 Rust tests passed | **P** | High code/test confidence; current real Inventory shape remains unverified. Capture a sanitized successful list count/shape on a network-capable host. |

### 2. Search / Install / Update / Remove lifecycle — 2.5/5 = 50.0%

| Capability | Contract and implementation evidence | Executed/observed evidence | Status | Confidence and next action |
| --- | --- | --- | --- | --- |
| Search the upstream catalog independently of Inventory and rank results by installs | `.trellis/spec/backend/command-contracts.md`; `src-tauri/src/cli.rs:138-152,183-200,366-389`; `src/api.ts:79-80`; `src/App.tsx:1418-1472`. Search now validates and calls an injected catalog request without touching session or Inventory state. | Runtime/session poisoning plus ranking test and sanitized request/shape-error tests at `src-tauri/src/cli.rs:575-674`; frontend discovery behavior in `src/App.test.tsx`; deterministic mocked-IPC `lifecycle-discovery-search` at 720x520; 29/29 Rust tests passed. | **P** | High implementation confidence; the confirmed isolation defect is closed. A privacy-safe live catalog endpoint smoke is still missing, so this network-sensitive row remains capped at partial. |
| Install an exact search result or direct source with global/noninteractive defaults and optional Agent/copy overrides | `CONTEXT.md:27-29`; `src-tauri/src/cli.rs:202-216,392-405`; `src/api.ts:81-85`; discovery modal in `src/App.tsx:1371-1548` | `src/api.test.ts:13-25`; discovery/install tests in `src/App.test.tsx:315-605`; exact defaults/override argv at `src-tauri/src/cli.rs:684-723` | **P** | High implementation confidence; no real mutation was permitted. Run a collision-safe packaged-app fixture smoke in a disposable test environment. |
| Update one Skill and Update All, with confirmation/availability and refreshed Inventory | `CONTEXT.md:19-21,59-61`; `.trellis/spec/backend/command-contracts.md:34-35`; `src-tauri/src/cli.rs:224-229,411-416`; `src/App.tsx:405-410,1550-1582`. Update All now opens a safe-default confirmation before invoking `update_skill`. | `src/commands.test.ts:20-60`; exact Rust argv contract at `src-tauri/src/cli.rs:697-723`; Cancel/Escape/no-invocation, localized single invocation, focus restoration, and stale-modal ownership tests at `src/App.test.tsx:607-752`. | **P** | High implementation confidence; the confirmation defect is closed. Real CLI update/refresh evidence is still absent, so the integration-sensitive row remains partial. |
| Whole-Skill removal with explicit confirmation and refreshed observed outcome | `CONTEXT.md:63-65`; `src-tauri/src/cli.rs:219-221,313-362,407-409`; `src/App.tsx:1584-1630` | `src/App.test.tsx:754-799`; exact Rust argv test `src-tauri/src/cli.rs:697-723`; deterministic `lifecycle-remove` at 720x520 | **P** | High UI/argv confidence; no real removal was run. Verify packaged removal and post-command Inventory with an inert test fixture. |
| Serialize lifecycle mutations, report busy, and derive outcomes from before/after Inventory rather than terminal prose | `.trellis/spec/backend/command-contracts.md`; `src-tauri/src/cli.rs:313-362` | Static implementation plus frontend active-operation behavior (`src/commands.test.ts:41-60`, `src/App.test.tsx:521-605`). No focused Rust test drives the mutex and observed-diff branches. | **P** | Medium. Add a fake executable/session seam test for concurrency, before/after diffs, target observation, and bounded diagnostics. |

### 3. File tree / Preview / Reveal — 2.5/4 = 62.5%

| Capability | Contract and implementation evidence | Executed/observed evidence | Status | Confidence and next action |
| --- | --- | --- | --- | --- |
| Derive roots only from current Inventory; walk a bounded tree using `symlink_metadata` without following links | `CONTEXT.md:67-69`; `.trellis/spec/backend/command-contracts.md`; `src-tauri/src/preview.rs:47-52,169-223,225-289` | Rust path/link containment tests at `src-tauri/src/preview.rs:350-374`; frontend tree behavior tests `src/App.test.tsx:1107-1408`; deterministic `content-tree` observation | **P** | High path/UI confidence; no real CLI-derived root was exercised and the Rust walker lacks a focused traversal test. Add an Inventory-rooted filesystem integration fixture covering directories, links, and special files. |
| Read only contained regular files with exact byte limits, UTF-8 validation, and explicit viewer classification | `.trellis/spec/backend/quality-guidelines.md`; `src-tauri/src/preview.rs:54-118,178-223,291-337` | Rust classification/boundary tests at `src-tauri/src/preview.rs:376-390`; frontend inert/unsupported test `src/App.test.tsx:1004-1104` | **P** | High logic confidence; filesystem integration remains fixture-only. Add a packaged-app preview smoke covering text, image, unsupported file, and boundary failure. |
| Reveal a selected file/root through the platform helper while preserving containment | `.trellis/spec/backend/command-contracts.md`; `src-tauri/src/preview.rs:120-167,193-223`; `src/api.ts:94-95` | Containment test `src-tauri/src/preview.rs:363-381`; recovery dispatch test `src/App.test.tsx:609-663` | **P** | High containment confidence, low native-helper confidence. Exercise `open -R`, Explorer, and `xdg-open` independently on native hosts. |
| Present safe Markdown/text/code/image/unsupported Preview and an accessible navigable file tree | `.trellis/spec/frontend/component-guidelines.md`; `src/App.tsx:923-1229,1597-1670`; raw HTML/remote resources are constrained by the renderer | `src/App.test.tsx:609-663,1004-1408`; deterministic `shell-ready`, `content-tree`, and narrow `shell-long` observations | **C** | High. Keep tree keyboard, inert Markdown, unsupported selection, compact-layout, and focus-return tests in the regular gate. |

### 4. Translation and network boundary — 3/4 = 75.0%

| Capability | Contract and implementation evidence | Executed/observed evidence | Status | Confidence and next action |
| --- | --- | --- | --- | --- |
| Translate only bounded Markdown/plain text through the Preview reader and return session-only results | `CONTEXT.md:71-89`; `src-tauri/src/translation.rs:29-70`; `src/App.tsx:506-537,1090-1229` | `src/App.test.tsx:1411-1658`; Rust eligibility test `src-tauri/src/preview.rs:376-381`; deterministic success/error views | **P** | High local confidence; the anonymous Google endpoint was not called. Run a privacy-reviewed provider smoke or replace it with a supported, testable provider boundary. |
| Keep proxy scope translation-only and enforce connect/attempt/shared deadlines, retry rules, stable errors, and atomic failure | `.trellis/spec/backend/command-contracts.md` bounded-translation scenario; `src-tauri/src/translation.rs:11-20,72-180`; `src/preferences.ts:175-195` | Proxy/error/deadline/retry/atomic-failure tests in `src-tauri/src/translation.rs:628-706`; proxy UI/API tests `src/preferences.test.ts:31-49`, `src/api.test.ts:34-44`; loopback retry passed outside sandbox | **P** | High implementation/test confidence; no real provider/proxy interoperability evidence. Add controlled provider and proxy integration smoke without retaining text. |
| Preserve Markdown structure/frontmatter/code/URLs; escape, batch, order, validate markers, bound concurrency, and publish atomically | `.trellis/spec/backend/command-contracts.md`; `src-tauri/src/translation.rs:223-568` | Focused Rust tests `src-tauri/src/translation.rs:601-625,695-840` cover chunks, structure, ranges, markers, ordering, panic containment, and atomic failure | **C** | High. These are deterministic logic contracts and all focused tests passed. |
| Keep original content accessible; expose disclosure, progress, hide/retry, and discard stale translation on Skill/document changes | `CONTEXT.md:83-89`; `src/App.tsx:519-552,670-675,1138-1275`; `src/styles.css:1572-1576`. Egress disclosure remains visible at <=760px and a newly failed translation selects the translation pane while Original stays available. | Translation disclosure/status/error/Retry/original/stale-result behavior at `src/App.test.tsx:1668-1721` and following stale-state assertions; archived remediation acceptance records deterministic review at 1180x800 and 720x520 in affected locales. | **C** | High. The previously confirmed narrow disclosure and hidden-error defects are closed; preserve these focused behavior and viewport checks. |

### 5. Application Commands / menu / shortcuts / status — 4/5 = 80.0%

| Capability | Contract and implementation evidence | Executed/observed evidence | Status | Confidence and next action |
| --- | --- | --- | --- | --- |
| Use one typed command registry with structured availability and dispatch-time revalidation | ADR-0015; `CONTEXT.md:51-53`; `src/commands.ts:1-87`; `src/App.tsx:291-456` | All five command tests at `src/commands.test.ts:20-60`; failure UI exposes structured reasons | **C** | High. |
| Adapt the same commands to Toolbar and Skill context menu without duplicating execution rules | ADR-0013/0015; `.trellis/spec/frontend/component-guidelines.md`; `src/App.tsx:735-795,858-896`; `src/nativeMenu.ts:118-153` | Command revalidation tests plus `shell-ready`, runtime-failure, and compact-toolbar observations. These exercise Toolbar/shared dispatch behavior, not Tauri's native context-menu popup. | **P** | High for the shared React command adapter, but the context-menu half is integration-sensitive and has no current-HEAD native evidence. Add a native smoke that opens the Skill context menu, verifies availability/labels, and dispatches through the shared command path. |
| Install a native application menu with platform role items, localized labels, toggle state, accelerators, and synchronized availability | ADR-0012-0015; `src/nativeMenu.ts:13-115`; app integration at `src/App.tsx:583-596` | `src/nativeMenu.test.ts:6-15` checks labels/toggle projection only. No current-HEAD native menu action/enablement smoke exists. | **P** | Medium. Add a Tauri/native smoke for menu construction, enablement refresh, accelerators, and dispatch on each platform. |
| Support shared Find, Find & Install, Settings, Refresh, and narrow-back shortcuts without claiming New | ADR-0012; `src/commands.ts:89-109`; `src/App.tsx:568-587` | `src/commands.test.ts:62-86`; Settings keyboard/focus tests and deterministic observation | **C** | High. |
| Keep visible localized loading, success, failure, diagnostics, operation progress, and availability reasons | ADR-0015; `.trellis/spec/frontend/component-guidelines.md`; status precedence and busy semantics at `src/App.tsx:733-743,1287-1311`; active status styling at `src/styles.css:953-966` | Startup asserts active/not-ready/`aria-busy` at `src/App.test.tsx:140-168`; translation asserts active/not-ready/busy then Ready after success at `src/App.test.tsx:1672-1716`; Retry, lifecycle progress, runtime failure, and preview recovery tests also pass. | **C** | High. The previously confirmed startup/translation Ready-state defect is closed. |

### 6. Settings / theme / localization / accessibility / responsive — 6/6 = 100.0%

| Capability | Contract and implementation evidence | Executed/observed evidence | Status | Confidence and next action |
| --- | --- | --- | --- | --- |
| Persist only validated UI/CLI-override preferences, migrate the legacy locale once, and never persist Inventory/Preview/translation state | `CONTEXT.md:19-21,27-33,43-49,87-93`; `src/preferences.ts:99-200` | `src/preferences.test.ts:12-58`; `src/App.test.tsx:665-830` | **C** | High. |
| Preserve `system/light/dark/sand/plum`, follow system theme, and keep semantic success/warning/danger separate from theme accent | ADR-0014/0016; `src/preferences.ts:202-207`; `src/App.tsx:481-498`; `src/styles.css:20-136` | Theme preference test `src/preferences.test.ts:12-16`; deterministic light/dark/sand/plum scenarios | **C** | High for behavior; aesthetics are intentionally outside this audit. |
| Provide structurally aligned English and zh-CN catalogs and apply system/explicit UI language immediately | `CONTEXT.md:43-49`; `src/i18n.ts:1-300`; `src/App.tsx:87-91,500-507` | `src/i18n.test.ts:10-30`; `src/App.test.tsx:739-830,983-1001`; English and zh-CN deterministic scenarios | **C** | High. |
| Use semantic controls, accessible names/roles, focus-visible, modal containment, Escape, and reliable focus restoration | `.trellis/spec/frontend/component-guidelines.md`; `src/ModalShell.tsx`; `src/App.tsx:749-757,797-833,836-921,923-1229`; `src/styles.css:190-194,1638-1644` | `src/ModalShell.test.tsx:24-79`; `src/App.test.tsx:562-607,833-909,1107-1408`; AX focus observed in Settings | **C** | High for tested flows. A full assistive-technology/platform audit remains release hardening, not an identified MVP absence. |
| Keep the approved two-pane utility operable and scrollable at 1180x800 and the 720x520 minimum | `src-tauri/tauri.conf.json:13-20`; `.trellis/spec/frontend/component-guidelines.md`; responsive rules `src/styles.css:1471-1646`; unified empty-state branches at `src/App.tsx:932-978` | Empty Inventory, filtered-empty recovery, matching guidance, Preview action, and non-contradictory copy are asserted at `src/App.test.tsx:170-243`; archived remediation acceptance records `shell-empty` at 1180x800 and `empty-720` plus translation error/egress at 720x520. | **C** | High. The previously confirmed empty/filter/narrow defects are closed; continue the two-viewport deterministic review gate. |
| Provide localized Settings sections for general, appearance, translation, installation, and about; validate proxy and Agent/copy overrides | `src/SettingsDialog.tsx:42-357`; `src/preferences.ts:99-195` | Settings persistence/keyboard tests `src/App.test.tsx:665-909`; narrow English/Chinese settings observations, including contained Agent scroll and auto-detect disabled state | **C** | High. |

## Score arithmetic

| Equal-weight group | Earned / possible | Group score |
| --- | ---: | ---: |
| Runtime / Inventory | 1.5 / 3 | 50.0% |
| Lifecycle Search / Install / Update / Remove | 2.5 / 5 | 50.0% |
| File tree / Preview / Reveal | 2.5 / 4 | 62.5% |
| Translation / network boundary | 3 / 4 | 75.0% |
| Application Commands / menu / shortcuts / status | 4 / 5 | 80.0% |
| Settings / theme / localization / accessibility / responsive | 6 / 6 | 100.0% |
| **Equal-weight group mean** | `(50 + 50 + 62.5 + 75 + 80 + 100) / 6` | **69.58% -> 69.6%** |

Raw capability points are `19.5/27 = 72.2%`; this is diagnostic only and does
not replace the agreed equal-group score. If all missing integration/native
evidence passes, every currently partial row can reach complete and the
hypothetical implementation upper bound is `27/27 = 100%`. The four prior
implementation defects no longer reduce that upper bound.

There are no rows classified unimplemented or not verifiable. The real-runtime
attempt was an environment-limited observation attached to partial rows, not a
not-verifiable product requirement.

## Executed quality gates

| Check | Result | Evidence |
| --- | --- | --- |
| Node toolchain selection | PASS after environment correction | The inherited PATH exposed unsupported Node 14.19.3, causing ESLint/Vitest syntax failures and an unreliable Vite exit. The audit gates were rerun with an already-installed supported Node toolchain; the final independent review repeated every frontend gate with bundled Node 24.19.0. This is a host PATH issue, not a product-test failure. |
| `npm run format:check` | PASS | Prettier: all matched files use Prettier style |
| `npm run lint` | PASS | ESLint exit 0 under Node 24.17.0 |
| `npm run typecheck` | PASS | TypeScript project build exit 0 under Node 24.17.0 |
| `npm test -- --run` | PASS | 10 files, 73 tests passed |
| `npm run build` | PASS | Vite 6.4.3; 228 modules; production build exit 0 |
| Production `dist/` inspection | PASS | Only `index.html` and one CSS/JS asset pair; no `review.html`, review scenario/fixture markers, or Iconify API/runtime endpoint found |
| `cargo fmt --check --manifest-path src-tauri/Cargo.toml` | PASS | exit 0 |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` | PASS | exit 0 |
| `cargo test --manifest-path src-tauri/Cargo.toml --all-features` | PASS | 29 tests passed; 0 failed. The three added catalog tests cover runtime isolation, ranking, validation, stable errors, and incompatible shapes. |
| Exact CI named-test guard | PASS | The configured selector listed exactly one match, then executed 1 passing test with 28 filtered out. A deliberately nonexistent selector produced zero matches and the same count guard rejected it before execution. |
| Read-only real runtime/Inventory readiness | ENVIRONMENT BLOCKED | Node 24.17.0 found. On 2026-09-22, `npx --yes skills@latest --version` again produced no result within 30 s and was terminated. Inventory was not requested because no version was resolved. No Skill names, paths, or content were emitted or retained. No add/update/remove command was run. |

## Deterministic UI observations

These observations use the development-only review entry and mocked Tauri IPC.
They prove WebView/browser rendering and interaction shape, not real CLI,
filesystem, provider, native-menu, or packaged-application integration.

The initial audit inspected ten representative states at 1180x800 and 720x520.
It confirmed the two-pane shell, lifecycle dialogs, Preview/tree interaction,
translation success/error surfaces, localized Settings, focus restoration, and
responsive scrolling. Its original “no obvious overflow or inaccessible primary
control” conclusion was too broad for the sampled states.

An independent review on 2026-09-09 exercised eighteen states and retained the
pre-remediation screenshots under `research/ui-review/`. Those images remain the
defect baseline, not evidence of current behavior. Remediation commit `32f0454`
then completed focused browser review at 1180x800 and 720x520 in the affected
locales; the archived parent and UI-child acceptance criteria are checked and an
independent reviewer reported no blocking contract or UI finding.

| Former baseline finding | Current resolution evidence |
| --- | --- |
| Startup appeared Ready | `status-active`, not `status-ready`, and `aria-busy=true` are asserted in `src/App.test.tsx:140-168`. |
| Empty Inventory contradicted Preview and could become blank under a filter | Both panes now present coherent installation recovery; filtered empty Inventory remains recoverable (`src/App.test.tsx:170-243`). |
| Translation progress appeared Ready | Active/busy semantics during the request and Ready only after success are asserted at `src/App.test.tsx:1672-1716`. |
| Narrow translation failure hid Retry and egress disclosure | Failure selects the translation pane, Retry is immediately present, Original remains available, and `.egress` is visible at <=760px (`src/App.test.tsx:1680-1716`; `src/styles.css:1572-1576`). |
| Remove/Preview recovery were already operable | Existing safe focus, Retry, Reveal, and diagnostic behavior remains covered by the passing frontend suite. |

The other retained screenshots cover ready/long-name shell, discovery
search/source, tree, translation success, Settings sections and validation,
appearance, and runtime failure. They strengthen WebView scenario coverage but
still do not replace native desktop evidence.

## Release-readiness gate

| Gate | Status | Basis / next action |
| --- | --- | --- |
| Local source quality | **PASS** | All required frontend and Rust checks pass at the audited revision. |
| Production web bundle hygiene | **PASS** | `dist/` contains only production entry/assets and no known review/runtime-icon leakage. |
| Package workflow contract precheck | **PASS LOCALLY** | `.github/workflows/ci.yml:79-96` lists the fully qualified current argv-contract test, requires exactly one match, then executes it. Current selector: 1 listed and 1 passed; deliberately missing selector: 0 listed and rejected by the count guard. Remote matrix execution is not claimed. |
| Bundle configuration | **PRESENT, NOT SMOKED** | `src-tauri/tauri.conf.json:26-39` declares bundle targets and PNG/ICNS/ICO resources, and the files exist. No current artifacts were built or installed in this audit. |
| macOS current-HEAD native smoke | **OPEN** | The only native pass is 2026-08-11 at `b9373b3`, 63 commits before current HEAD and before the lifecycle rewrite `7c69e26`. It was unsigned and Gatekeeper was already disabled. First replace the superseded lifecycle steps in `docs/release-smoke.md`, then run the current-model smoke without bypassing protections. |
| Windows current-HEAD native smoke | **OPEN** | No NSIS native install/launch/runtime-recognition/removal/cleanup evidence. Run independently on Windows. |
| Linux current-HEAD native smoke | **OPEN** | No AppImage native install/launch/runtime-recognition/removal/cleanup evidence. Run independently on Linux. |
| Signing/notarization/security prompts | **OPEN** | Historical macOS smoke did not establish these. Record them per platform for the current artifact. |

`docs/release-smoke.md:1-5,86-89` still establishes the valid requirement for
separate native proof per platform. However, its named
`projection_contract_smoke` precheck and Managed Library/Linked Installation
workflow at lines 14 and 58-70 predate ADR-0011 and cannot be followed as the
current lifecycle procedure; the checklist must be updated before native smoke.
The historical macOS evidence at
`.trellis/tasks/archive/2026-08/08-11-macos-installer-smoke/evidence.md:1-41`
proves the old artifact's launch and runtime recognition, while lines 85-92
leave Windows/Linux open. Commit history confirms `b9373b3` is an ancestor 63
commits behind HEAD and `7c69e26` later delegated lifecycle to the upstream
CLI. It is therefore not current-architecture proof.

## Historical R1-R4 roadmap view

This view is intentionally non-aggregated and does not affect the MVP score.

| Historical milestone | State | Assessment |
| --- | --- | --- |
| R1 — trusted Inventory/reconciliation | **Superseded, with native release validation still partial** | The Managed Library, installation ownership, reconciliation, restore/detach/reapply, projection, and rollback model in `docs/roadmap.md:21-35` was superseded by ADR-0011 and must not be restored or counted as missing MVP work. Its still-relevant native validation intent is incomplete: the old macOS artifact passed under limitations; current macOS and all Windows/Linux current-HEAD smoke are open. The historical projection-smoke concept is superseded; the current package workflow instead guards a live upstream-CLI argv contract. |
| R2 — evidence-backed Capability Card | **Not started** | Current source/Agent metadata and preview are useful foundations, but there is no normalized capability-kind/evidence/enforcement card, import/update capability diff, or evidence-backed third-host program described at `docs/roadmap.md:37-47`. This is future scope, not an MVP defect. |
| R3 — revision-level Eval | **Not started** | No revision/host/model/suite-bound baseline comparison or eval orchestration described at `docs/roadmap.md:49-59` exists in the accepted MVP. |
| R4 — local router and safe evolution | **Not started** | No evidence-ranked Skill router, usage-event model, or candidate-patch/canary promotion loop described at `docs/roadmap.md:61-70` exists in the accepted MVP. |

## Prioritized remediation sequence

1. **Add a hermetic fake-CLI integration seam.** Exercise Node boundary cases,
   session pinning,
   atomic startup Inventory, open Agent values, exact argv/env, mutation
   serialization, before/after refresh, observed outcomes, diagnostics bounds,
   and failure codes without touching the user's global Skills.
2. **Add an Inventory-rooted filesystem integration fixture.** Exercise bounded
   tree walking, links/special files, exact read limits, viewer classification,
   and contained Reveal targets before relying on packaged native helpers.
3. **Restore current native release evidence.** First update
   `docs/release-smoke.md` from the superseded Managed Library/projection flow to
   the upstream Skills CLI lifecycle. Build one immutable current-HEAD artifact
   set, then run macOS, Windows, and Linux smoke independently, including real
   Inventory, inert fixture add/remove,
   recognition, cleanup, signing/notarization/security prompts, menu behavior,
   Reveal helper, and bundle icons.
4. **Add controlled external-boundary smokes.** Verify the real catalog search
   response and translation provider/proxy interoperability using privacy-safe
   fixtures and retain no user content. Treat the anonymous translation endpoint
   as best-effort until a supported provider/SLA decision is made.
5. **Treat broader visual polish as a separately approved direction.** Command
   hierarchy, repeated counts/copy, compact control sizing, and Settings density
   merit a focused UI task, but ADR-0016 prohibits incidental restyling without
   an Approved Visual Direction.

Search isolation, Update All confirmation, the exact named-test CI guard, and
empty/startup/translation/narrow/egress semantics are complete and are no longer
remaining-work items. The independent remediation reviewer reported no blocking
contract or UI issues.

## Audit acceptance cross-check

- Every major current-MVP area has a requirement, implementation anchor,
  executed/observed evidence, status, confidence statement, and next action.
- All required frontend and Rust gates pass at current HEAD: 10 frontend files /
  73 tests and 29 Rust tests. The final independent review repeated the frontend
  gates with bundled Node 24.19.0; the inherited Node 14 PATH was recorded and
  corrected rather than misclassified as a product failure.
- Eighteen deterministic review states across the original and independent
  passes cover both approved viewports, five themes, both locales,
  startup/runtime failure, lifecycle, empty states, Preview/tree, translation,
  Settings, focus, and responsive operation.
- Conservative score (69.6%), implementation upper bound (100%), evidence
  coverage (27/27), group subtotals, and raw count (19.5/27) are arithmetically
  reproducible.
- Release readiness and historical R1-R4 progress are separate from the MVP
  score.
- No real add, update, or remove operation was executed; no Inventory identity,
  path, or content was retained.
- No product code, dependency, spec, or existing product documentation was
  changed by this audit. The task-local evidence report and review screenshots
  are the only audit outputs.
