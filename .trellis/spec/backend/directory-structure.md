# Backend Directory Structure

## Runtime boundary

- `src-tauri/src/lib.rs` is the Tauri boundary. Commands deserialize input,
  call a domain function, and serialize its result. Do not put traversal,
  ownership, or transaction rules in a command handler.
- `src-tauri/src/skill.rs` is the current deep module for Skill package
  validation. It owns the metadata, resource observation, structured error,
  and tests so the trust-boundary rules cannot diverge across callers.
- `src-tauri/src/main.rs` only starts the library entry point.

Add a module only when it owns a real domain seam such as Agent configuration,
persisted state, or transactions. Do not add one-interface/one-implementation
layers or a generic plugin framework.

Rust files and modules use `snake_case`; serializable Rust DTOs use
`UpperCamelCase` and `#[serde(rename_all = "camelCase")]` at the frontend
boundary, as demonstrated by `ValidatedSkill` and `AppInfo`.
