# Design: macOS installer smoke and recognition

## Boundary

This is a release-verification task, not a product feature. It adds reusable
documentation and evidence, then exercises the packaged application against the
real macOS, Codex, and Claude Code environments. It does not add test harnesses
or runtime adapters.

## Artifact selection

Use the current commit's universal DMG from GitHub Actions. This tests the exact
artifact produced by the release pipeline. If unavailable, a local universal
build may diagnose packaging, but the macOS roadmap item remains incomplete until
the matching CI artifact is tested.

For the selected artifact, record:

- source commit and whether CI or local build produced it;
- filename and SHA-256;
- architectures reported for the executable inside the application bundle;
- macOS, Codex, and Claude Code versions.

## Smoke data flow

1. Capture relevant pre-existing Skill/config paths without copying secrets into
   repository evidence.
2. Mount the DMG, install and launch the packaged application, and confirm its
   inventory loads.
3. Create a unique inert fixture in a temporary directory.
4. Use Skill Deck to import it as Linked Installations for Codex and Claude Code.
5. Start fresh runtime sessions and inspect their native Skill listing or
   completion UI for the unique name.
6. Uninstall it from both Agent Targets, start fresh runtime sessions and verify
   the name is absent, then Remove from Library.
7. Remove only test-owned files, unmount the image, and compare relevant paths
   with the pre-smoke state.

## Evidence contract

`.trellis/tasks/08-11-macos-installer-smoke/evidence.md` will contain concise
text evidence: commands, versions, artifact identity, observed UI/runtime states,
and cleanup results. It must not contain tokens, full user configuration, or
unrelated Skill contents. Screenshots are optional and only retained when text
cannot capture the observation.

The reusable checklist belongs in `docs/release-smoke.md`. `docs/roadmap.md`
tracks macOS, Windows, and Linux separately. macOS passes only when the installed
application and both Agent runtimes pass; the R1 stage remains incomplete until
native Windows and Linux results exist.

## Safety and rollback

- Use a collision-resistant fixture name and refuse to overwrite an existing
  Skill or application bundle.
- Install only to `/Applications`; stop if `Skill Deck.app` appears before the
  copy step.
- A standard macOS user confirmation is allowed. Do not disable Gatekeeper,
  strip quarantine attributes, or restore whole configuration files. Record
  missing signing/notarization separately from whether the app launches.
- Prefer product-owned uninstall/remove flows. If those fail, manually remove
  only paths proven to belong to the fixture and document the failure.
- Stop before any ambiguous deletion. A residual test entry is safer than
  deleting unrelated user state.
- Remove the installed application after the test; the smoke must not leave a
  new daily-use installation behind.

## Trade-offs

Native listing/completion proves discovery without spending model calls or
conflating discovery with effectiveness. Keeping the test manual avoids owning a
fragile GUI framework before release frequency demonstrates that need.

The smoke uses only the primary Linked Installation flow. Exercising Copy
Fallback manually would duplicate deterministic projection coverage without
changing what runtime discovery means.

If the smoke exposes a product defect, the tested artifact remains immutable:
record the failure and create a separate defect task. A fixed local rebuild is
not substituted for the CI artifact; rerun the smoke after CI packages the fix.
