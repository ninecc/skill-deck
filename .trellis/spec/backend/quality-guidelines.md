# Backend Quality Guidelines

## Trust boundaries

The upstream CLI owns installation validation and mutation. Skill Deck must not
recreate its ownership or package model. Its own boundaries remain strict:

- invoke `node`/`npx` without a shell and pass every input as one argument;
- structurally decode JSON and keep Agent values open strings;
- derive preview roots only from current CLI Inventory;
- walk with `symlink_metadata`, list but never follow links, reject special-file
  reads and absolute/parent paths, and enforce byte limits while reading;
- allow translation only through the bounded preview reader and never write
  translated content.

Do not execute Skill scripts, parse human CLI tables, or add a fallback manager.

## Tests and verification

Non-trivial branches need a focused unit or integration test. Boundary tests
cover exact-limit acceptance and one-over rejection with small injected limits.

Required commands:

```bash
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
```

## Scenario: Desktop bundle icons

### 1. Scope / Trigger

Changing the application icon or Tauri bundle configuration affects native
artifacts on Windows, macOS, and Linux even when the Rust build succeeds.

### 2. Signatures

`src-tauri/tauri.conf.json` must declare `bundle.icon` with the generated PNG,
ICNS, and ICO resources under `src-tauri/icons/`.

### 3. Contracts

- `icons/icon.svg` is the editable source.
- `npm run tauri icon src-tauri/icons/icon.svg` regenerates platform assets.
- macOS bundles must contain `Contents/Resources/icon.icns` and set
  `CFBundleIconFile=icon.icns`; Windows consumes `icon.ico`; Linux consumes PNG.

### 4. Validation & Error Matrix

- Missing `bundle.icon` -> reject release: Tauri may build an iconless bundle.
- Missing generated resource -> reject release before packaging.
- Source and bundled ICNS hashes differ -> reject as a stale artifact.
- The 32 px preview is illegible or off-center -> correct the SVG and regenerate.

### 5. Good / Base / Bad Cases

- Good: all formats are declared, generated, legible at 32 px, and packaged.
- Base: the SVG changes and every generated asset is refreshed in one command.
- Bad: only `icon.svg` or `icon.png` changes while native bundle assets stay old.

### 6. Tests Required

Build a native artifact, assert its platform icon exists, and on macOS assert
the source and bundled ICNS hashes match. Native CI remains responsible for
the equivalent Windows and Linux packaging checks.

### 7. Wrong vs Correct

```json
// Wrong: generated files exist but the bundle does not reference them.
{ "bundle": { "active": true } }

// Correct: every desktop format is explicit.
{ "bundle": { "icon": ["icons/32x32.png", "icons/icon.icns", "icons/icon.ico"] } }
```
