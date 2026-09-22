# Functionality completion audit design

## Audit boundary

The audit treats the repository as one product with three evidence surfaces:

1. Product contract: `CONTEXT.md`, accepted ADRs 0011–0016, active Trellis specs, and the approved redesign requirements.
2. Implementation: React command/UI paths, Tauri command DTOs, Rust CLI/preview/translation boundaries, preferences, native menus, and review scenarios.
3. Verification: frontend and Rust checks, deterministic UI review scenarios, production bundle contents, and recorded native release-smoke evidence.

The historical roadmap is analyzed separately because its R1-era lifecycle model was superseded by ADR-0011.

## Evidence model

Each capability row in `evidence.md` will contain:

- capability and capability group;
- authoritative requirement source;
- implementation anchors;
- automated-test anchors and executed-check result;
- observed UI/runtime evidence when available;
- status: complete, partial, unimplemented, or not verifiable;
- confidence and gap/next action.

Claims are strongest when contract, implementation, and passing execution evidence agree. Static code alone can prove presence but not runtime correctness. A deterministic fixture can prove rendered behavior but not native CLI integration. Native packaging claims require per-platform smoke evidence.

## Scoring and reporting

- Use six equal-weight functional groups: Runtime/Inventory; lifecycle Search/Install/Update/Remove; file tree/Preview/Reveal; translation/network boundary; Application Commands/menu/shortcuts/status feedback; and Settings/theme/localization/accessibility/responsive behavior. Inside each group, independently observable capabilities are equal-weighted and scored as 1, 0.5, or 0.
- Require passing behavior tests for ordinary complete claims. Require matching integration or native evidence for CLI, filesystem, native-menu, OS-helper, and packaged-app capabilities; static/unit evidence alone caps those rows at partial.
- Treat unverifiable items as 0 in the conservative score and expose their count separately.
- Show group subtotals so a large number of small UI states cannot hide a missing lifecycle or trust-boundary capability.
- Report evidence coverage as the proportion of capability rows with executable or observed evidence, separate from implementation presence.
- Report release readiness as gated findings for local automated quality, native packaging, and Windows/macOS/Linux smoke status.
- Publish one conservative score plus a hypothetical implemented upper bound. The upper bound assumes missing integration evidence later passes and is never labeled complete.
- Summarize historical R1–R4 milestones as complete, partial, not started, or superseded without an aggregate roadmap percentage or any effect on the MVP score.

## Execution boundaries

The task is read-only with respect to product code. Generated build output and temporary local processes are permitted as audit evidence. The only durable edits belong under this Trellis task directory. If checks reveal defects, record them rather than repairing them.

## Compatibility and limitations

- Local execution on macOS cannot prove Windows or Linux native packaging/runtime recognition.
- The deterministic review entry uses mocked Tauri IPC and cannot prove real Skills CLI, filesystem, reveal, or translation-provider integration.
- Translation checks must not depend on live external network calls; existing unit/integration tests and bounded fixture behavior are the default evidence.
- Real runtime/Inventory verification may perform normal `npx skills@latest` network resolution and npm cache writes, but is otherwise read-only. It records only readiness, sanitized version/count evidence, and outcome. Skill names, paths, and content are not retained. Real lifecycle mutation is prohibited.
- Visual review judges operability, state clarity, localization, focus/keyboard behavior, and responsive behavior; it does not score aesthetic taste.
- No remote CI state is assumed available. Existing documentation and repository history are evidence only when their scope and date are explicit.
- Current HEAD is not release-ready when required native evidence is stale or absent. This is an open gate, not a claim that untested platform behavior is broken.

## Rollback

No product rollback is needed because product files are not modified. Stop local servers after visual review. If generated artifacts obscure repository state, remove only known task-generated outputs after verifying their exact paths; otherwise leave them and report them.

A severe finding stops only the affected or potentially mutating check. Other safe, read-only checks continue so the final prioritization is based on a complete gap inventory.
