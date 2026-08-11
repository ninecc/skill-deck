# Cross-Platform Thinking Guide

Use this guide when code touches native paths, links, commands, packaging, or
`#[cfg(...)]` branches.

## Native path boundaries

- Build logical paths from individual components instead of embedding `/` or
  `\` in one child string.
- A Rust `Path` may still contain an alternate platform separator. Before
  passing it to a native shell command, reconstruct it from `components()` so
  the command receives the platform's native separator without lossy Unicode
  conversion.
- Keep normalization at the shared command boundary so production and every
  caller receive the same protection.
- Do not convert paths with `to_string_lossy()` for filesystem mutation.
- `symlink_metadata()` describes the link object rather than its target. On
  Windows, identify directory junctions from the directory and reparse-point
  attributes, then canonicalize separately to validate the resolved target.

## Conditional compilation

- Check that helpers, fields, imports, and their callers have compatible
  `#[cfg(...)]` scopes.
- A Unix-only helper must not be referenced by a cross-platform test.
- Remove platform-only dead-code warnings structurally with matching `#[cfg]`
  attributes rather than suppressing warnings.

## Validation

- Local tests validate the current host only. Never claim a Windows or Linux
  native path/link fix from a macOS test run.
- Keep a focused native CI test at the real boundary. For Windows junctions,
  `projection_contract_smoke` must create, inspect, retarget, restore, and remove
  a real junction before packaging starts.
- Distinguish compile success from runtime success: a newly compiling native
  test can still expose the next platform-specific failure.
