# Implementation plan: macOS installer smoke and recognition

1. Record the current commit and host/Agent versions; download the matching CI
   universal DMG. A local universal build is diagnostic only and does not pass
   the roadmap item.
2. Verify the artifact SHA-256 and executable architectures, then mount it,
   re-check that `/Applications/Skill Deck.app` is absent, copy the bundle there,
   and launch the installed application.
3. Capture safe pre-smoke state and create one inert, uniquely named temporary
   Skill fixture.
4. Drive the packaged UI to import/install the fixture for Codex and Claude Code;
   verify each fresh runtime exposes it through native Skill discovery UI, not a
   model-generated claim.
5. Uninstall the fixture from both Agent Targets; verify both fresh runtimes no
   longer expose it; Remove from Library, clean up test-owned artifacts, and
   compare post-smoke state.
6. Write `docs/release-smoke.md` and task-local `evidence.md`; update
   `docs/roadmap.md` with an honest per-platform status while leaving R1 open.
7. Run focused documentation and repository checks, review the final diff, and
   archive the task only if every macOS and cleanup acceptance criterion passes.

## Validation

```bash
shasum -a 256 <dmg>
lipo -archs <mounted-app-executable>
npm run format:check
git diff --check
```

Runtime checks are observations in fresh Codex and Claude Code sessions. They do
not invoke the Skill or call a model for effectiveness evidence.

## Risky operations and rollback points

- Installing to `/Applications`, launching GUI applications, mounting disk
  images, and downloading CI artifacts may require explicit host approval.
- Do not overwrite a pre-existing `Skill Deck.app`; none exists at planning time,
  but re-check immediately before installation.
- Before product mutation, stop if the unique fixture name already exists under
  either Agent root.
- Cleanup only the selected mounted image, installed app bundle, temporary
  fixture, and managed entries proven to use that unique name.
- If any product behavior fails, stop and capture evidence. Fixes belong to a
  separate task and require a newly packaged CI artifact before this smoke
  resumes.

## Start gate

- Planning artifacts are complete and require the user's explicit approval of
  this latest summary before `task.py start`.
