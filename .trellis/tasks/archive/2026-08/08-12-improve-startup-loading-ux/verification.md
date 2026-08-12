# Verification Notes

## Current-build GUI

- Built the current debug `.app`; the macOS app bundle succeeded. The later DMG
  packaging script failed independently and is not required for this UI check.
- Confirmed the new startup loading state, compact Inventory, Find & install
  entry, proxy Settings field, and persisted proxy draft in the current build.
- Applied `http://127.0.0.1:7890` in Settings.

## Translation diagnosis

1. The exact Google Translate endpoint returned `你好` for `hello` through the
   proxy in 0.33 seconds, proving the proxy override can work.
2. The current app translated the 147-byte `grill-me` Skill through that proxy,
   proving the applied Preference reaches `translate_preview` and the Rust HTTP
   client.
3. The long `ask-matt` Skill reproduced the 15-second timeout. Inspection found
   one serial provider call per Markdown prose fragment. The implementation now
   HTML-escapes and batches protected prose into strictly indexed inert spans,
   uses at most four batch workers, preserves the shared 15-second deadline,
   validates every returned marker/entity, and publishes atomically.
4. After rebuilding that fix, the local proxy became unhealthy: synthetic large
   requests and two final `hello` probes failed during TLS connect after about
   five seconds. The final GUI green run is therefore blocked by proxy state,
   not evidence of a remaining application timeout defect.

## Automated gates

- Frontend: format, lint, typecheck, 21 tests, production build — pass.
- Rust: fmt, clippy with warnings denied, 25 tests — pass.
- `git diff --check` — pass.

