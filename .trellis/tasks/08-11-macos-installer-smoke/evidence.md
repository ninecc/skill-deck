# macOS installer smoke evidence

## Result

macOS universal DMG: **pass** for packaged application launch, Linked
Installation, Codex/Claude Code native runtime recognition, negative recognition
after Uninstall, and cleanup. Windows NSIS and Linux AppImage remain unverified
on native hosts.

## Environment and immutable artifact

- Date: 2026-08-11 (Asia/Shanghai)
- Host: Apple Silicon Mac, macOS 26.5.2
- Commit: `b9373b373c6e5bade8da4df84c633e654da086fe`
- GitHub Actions run: [31466973404](https://github.com/ninecc/skill-deck/actions/runs/31466973404)
- Artifact ID/name: `9092097234`, `skill-deck-macos-universal`
- Artifact ZIP SHA-256: `59815339775d19a165676efcdea2471f5d66dcfea568ffcebd31c71c2375a190`
  (matched the GitHub Actions artifact digest)
- DMG: `Skill Deck_0.1.0_universal.dmg`
- DMG SHA-256: `78bca5f9f9a8d49731c032955158e33cd6d34ec4abe48a048a50977b00372fe0`
- Bundle version/identifier: `0.1.0`, `com.skilldeck.desktop`
- Executable architectures: `x86_64 arm64`
- Agent runtimes: Codex CLI `0.147.0`, Claude Code `2.1.220`

The overall workflow run was red because the Windows packaging job failed its
projection contract gate. The macOS packaging job and Linux packaging job were
green; that CI status is not treated as native Windows or Linux smoke evidence.

## Packaged application observations

- Mounted the selected DMG read-only and copied its app to
  `/Applications/Skill Deck.app` after confirming that path was absent.
- The installed packaged app launched and loaded Inventory. Initial counts were
  `0 Managed Library`, `59` discovered external Installations, and `100` items
  needing attention; unrelated entries were not changed.
- The executable is universal, but the bundle has no distribution identity or
  Team ID (`codesign --verify --deep --strict` reported that the bundle was not
  signed). `spctl` reported acceptance with `override=security disabled` because
  Gatekeeper was already disabled on this host. The test did not change security
  settings or remove quarantine. Signing/notarization remains a separate
  distribution-security gap, not a native launch claim.
- macOS launched the app through App Translocation. All product actions below
  were performed in that packaged application, not a development server or local
  build.

## Runtime recognition

Fixture: `skill-deck-runtime-smoke-b9373b3`, containing only one inert 217-byte
`SKILL.md` with no scripts, references, network use, or secret requests.

1. Imported the fixture to Managed Library and used Linked Installation for
   Codex and Claude Code. Inventory showed both Installations as Healthy and
   resolved both symlinks to the same managed `current` revision.
2. In a fresh Codex CLI session, typing
   `$skill-deck-runtime-smoke-b9373b3` without submitting displayed the native
   `[Skill]` completion and its fixture description.
3. In a fresh Claude Code session, typing
   `/skill-deck-runtime-smoke-b9373b3` without submitting displayed the native
   command completion and its fixture description.
4. No model request was submitted and the Skill was never invoked.
5. Used Skill Deck's Uninstall flow for Codex and Claude Code. Fresh sessions
   then showed `no matches` in Codex and
   `No commands match "/skill-deck-runtime-smoke-b9373b3"` in Claude Code.
6. Used Remove from Library; Inventory returned to `0 Managed Library`.

## Ownership and cleanup

- Codex config SHA-256 before/after:
  `56cf4564a1e92e6955fe653e03744ea663f63890c24fc18dac196b79c12f17de`
- Claude settings SHA-256 before/after:
  `cd4b4a8626b54eb486254b6909355eecb112d29be070f3e8c8ec0ae379da6a9a`
- Both Agent Target symlinks and the managed package path are absent.
- The packaged app was quit normally. The installed app, downloaded artifact,
  temporary fixture/artifact directory, and generated Skill Deck application
  state were moved to uniquely named Trash entries, so cleanup is recoverable.
  The pre-existing empty `com.skilldeck.desktop` application-data directory was
  recreated empty.
- The DMG was detached. `/Applications/Skill Deck.app`, the selected mount,
  downloaded ZIP, temporary directory, Agent Target paths, managed package, and
  active application state contain no fixture residual.
- Fresh Codex/Claude sessions created ordinary local session-history records;
  these were not rewritten or deleted because doing so would cross runtime-owned
  state boundaries. They contain no Skill invocation or model response.

## Remaining platform work

- Windows NSIS: **unverified**. The matching run did not produce a usable
  artifact because its projection contract gate failed.
- Linux AppImage: **unverified**. CI packaging success is not a native-host
  install, launch, recognition, removal, or cleanup result.
- Follow `docs/release-smoke.md` independently on native Windows and Linux hosts
  before checking either roadmap item or declaring R1 complete.
