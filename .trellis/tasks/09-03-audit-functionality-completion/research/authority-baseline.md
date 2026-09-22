# Authority baseline for the completion audit

## Confirmed hierarchy

1. Accepted ADRs 0011–0016 and active Trellis specs define current architectural and quality contracts.
2. `CONTEXT.md` defines current domain language and lifecycle ownership.
3. Current implementation and tests provide behavior evidence but cannot silently override accepted product intent.
4. The approved 2026-08-13 desktop redesign contract preserves the current single-window inventory, preview, management, settings, translation, localization, command, and cross-platform behavior scope.
5. `docs/roadmap.md` is historical where it conflicts with ADR-0011 or current domain language; its future R2–R4 ambitions are reported separately.

## Evidence supporting the hierarchy

- ADR-0011 explicitly supersedes ADRs 0001–0010 and delegates lifecycle to one session-pinned upstream Skills CLI.
- ADRs 0012–0015 preserve the single-window utility and shared Application Command model across toolbar/menu/shortcut/context-menu adapters.
- ADR-0016 marks prior visual directions as historical unless explicitly approved for a task while preserving accessibility, platform behavior, localization, responsive operability, and the current preference schema.
- Cross-session memory for Codex session `019ff984-379f-7213-a263-d844deb420ff` records the approved redesign requirement that accepted ADRs and current implementation outrank conflicting roadmap language.

## Audit consequence

The headline percentage measures the current accepted MVP. Historical R1–R4 progress is an appendix and cannot reintroduce removed Managed Library, reconciliation, ownership, or rollback requirements into the denominator.

