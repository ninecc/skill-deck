# Manual release smoke

Run this checklist separately on native Windows, macOS, and Linux hosts. A CI
Agent Projection Contract test or a result from another operating system does
not satisfy it.

## Before each platform run

- [ ] Use the NSIS (`skill-deck-windows-x64`), universal DMG
      (`skill-deck-macos-universal`), or AppImage
      (`skill-deck-linux-x86_64`) produced by CI from the commit under test. A
      local build is diagnostic only.
- [ ] Confirm that platform's packaging job passed
      `projection_contract_smoke` before producing the artifact; this does not
      prove Agent Runtime Recognition.
- [ ] Record the commit, CI run, artifact filename, SHA-256, host architecture
      and version, and installed Codex and Claude Code versions.
- [ ] Record the architecture declared by CI and verify the executable with
      `lipo -archs` on macOS or `file` on Linux. On Windows, record the expected
      x64 packaging target and `Get-AuthenticodeSignature` result. Record signing,
      notarization, and security prompts without bypassing host protections.
- [ ] Use a clean test host or stop if Skill Deck is already installed. Capture
      the pre-smoke existence and entry names of the Codex and Claude Skill
      roots, hashes of relevant configuration files, and relevant Skill Deck
      state without recording configuration contents, tokens, or unrelated
      Skill contents.
- [ ] Choose a collision-resistant Skill name and stop if it already exists in
      Skill Deck or either Agent Target. Create a temporary Skill Package that
      contains only a valid `SKILL.md` with inert instructions: no commands,
      network access, or secrets.

## Install and launch

### Windows — NSIS

- [ ] Verify the SHA-256 with `Get-FileHash -Algorithm SHA256`, run the installer
      normally, and launch Skill Deck from the installed application.
- [ ] Confirm the packaged UI opens and loads Inventory. Do not substitute a
      development server or unpackaged executable.

### macOS — universal DMG

- [ ] Verify the SHA-256 with `shasum -a 256`, mount the DMG, and confirm
      `/Applications/Skill Deck.app` is absent immediately before copying the
      bundle there.
- [ ] Record both executable architectures from `lipo -archs`, then launch the
      installed bundle and confirm Inventory loads. Do not disable Gatekeeper or
      remove quarantine attributes.

### Linux — AppImage

- [ ] Verify the SHA-256 with `sha256sum`, make only the downloaded AppImage
      executable, and launch that file normally.
- [ ] Confirm the packaged UI opens and loads Inventory. Record any required
      desktop integration or FUSE prompt; do not replace the AppImage with an
      unpackaged executable.

## Projection and runtime recognition

- [ ] In the packaged Skill Deck UI, import the temporary Skill Package and
      install it for Codex and Claude Code using Linked Installation. Do not use
      Copy Fallback for this smoke.
- [ ] Confirm Inventory reports both Installations as Healthy.
- [ ] Start fresh Codex and Claude Code sessions. In each runtime's native Skill
      listing or completion UI, observe the unique name. A model response,
      invocation, or task result is not recognition evidence.
- [ ] In Skill Deck, Uninstall the fixture from both Agent Targets.
- [ ] Start fresh Codex and Claude Code sessions and confirm their native Skill
      discovery UI no longer exposes the unique name.
- [ ] Remove from Library and confirm the Managed Skill Package is absent.

## Cleanup and evidence

- [ ] Exit Skill Deck and remove it through the platform-owned path: NSIS
      uninstaller on Windows; remove the copied application bundle, unmount the
      selected disk image, and remove the test-owned DMG on macOS; or remove the
      downloaded AppImage on Linux. Remove the temporary fixture directory.
- [ ] Compare Agent root entry names, configuration hashes, and relevant Skill
      Deck state with the pre-smoke capture. Preserve unrelated files and report
      any test-owned residual path instead of performing an ambiguous deletion
      or restoring a whole configuration file.
- [ ] Record the application launch and Inventory result, positive and negative
      native recognition observations for both Agent Targets, cleanup result,
      failures, and security prompts. Retain screenshots only when concise text
      cannot capture the observation.
- [ ] Mark this platform passed only when its packaged application launches and
      loads Inventory, both Agent runtimes recognize the installed Skill and no
      longer recognize it after Uninstall, and cleanup is proven. Track Windows,
      macOS, and Linux independently.
