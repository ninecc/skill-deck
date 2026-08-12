# Backend Development Guidelines

The backend is the Rust crate under `src-tauri/`. It owns safe CLI execution,
the session-pinned upstream version, Inventory refresh, preview containment and
translation provider access. React receives only serializable command DTOs.

## Pre-Development Checklist

Read the guides that match the change:

- [Directory Structure](./directory-structure.md) for modules and command seams.
- [Error Handling](./error-handling.md) for user-visible failures.
- [Quality](./quality-guidelines.md) for trust-boundary and test requirements.
- [Logging](./logging-guidelines.md) before adding diagnostics.
- [Desktop Command Contracts](./command-contracts.md) for Tauri DTOs,
  CLI mutations, preview/translation and cross-layer error behavior.

Then run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` with the
manifest at `src-tauri/Cargo.toml`.
