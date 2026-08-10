# Backend Error Handling

External input failures are data, not panics. `src-tauri/src/skill.rs` models
them with `SkillErrorCode` plus a serializable `SkillError` containing a stable
code, message, optional path, and optional limit/observed values.

## Rules

- Map I/O errors at the path where they occur; preserve the source error in the
  human message without exposing Skill file contents.
- Resource failures must report both the configured limit and the observed
  value. The UI must not infer these from message text.
- Use `expect` only for internal invariants already established in the same
  function. `strip_prefix` in the package walk is the current example.
- Do not use catch-all string errors, `unwrap` in runtime code, or silent
  fallbacks at ownership and filesystem boundaries.
- Tauri startup may use `expect` because failure to create the application is
  unrecoverable before a UI exists.

Tests should assert stable codes and structured fields; assert full messages
only when wording is itself part of the contract.
