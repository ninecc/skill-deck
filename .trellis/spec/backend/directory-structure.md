# Backend Directory Structure

- `src-tauri/src/lib.rs` is the thin Tauri command boundary.
- `src-tauri/src/cli.rs` owns runtime discovery, the pinned CLI session,
  machine-readable Inventory, search, argv construction, mutation serialization
  and post-command refresh.
- `src-tauri/src/preview.rs` owns installed-root lookup, containment, file-tree
  walking, bounded reads, viewer classification and native file-manager reveal.
- `src-tauri/src/translation.rs` owns the provider request/response and Markdown
  segmentation. Keep its exported boundary provider-neutral without adding a
  one-implementation trait, factory or plugin registry.
- `src-tauri/src/main.rs` only starts the library entry point.

Rust files use `snake_case`; serializable DTOs use `UpperCamelCase` and
`#[serde(rename_all = "camelCase")]`. Do not recreate state, ownership,
configuration, Git, revision or reconciliation modules beside the CLI.
