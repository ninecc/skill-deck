# Backend Error Handling

External runtime, CLI, network and filesystem failures are structured data.
`CommandError` carries a stable code and message plus optional operation, exit
code and bounded sanitized diagnostics.

- Reject empty, oversized, NUL-containing and multiline CLI inputs before
  process launch.
- Preserve the operation and exit status for CLI failures; never interpret
  human stdout as a success protocol.
- Invalid UTF-8 or JSON fails closed. An incompatible `skills@latest` reports
  the actual version and requires a Skill Deck upgrade.
- Map filesystem errors at the inspected path without including Skill bodies.
- Translation/search failures stay local to those features.
- Use `expect` only for invariants established in the same function. Tauri
  startup may use it because no UI exists yet.

Tests assert stable codes and structured fields; full wording is asserted only
when the wording itself is a product contract.
