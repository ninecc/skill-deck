# Backend Quality Guidelines

## Trust boundaries

Every Skill Source is untrusted. Validate before content enters Managed
Library storage. The reference implementation is `validate_skill_dir` in
`src-tauri/src/skill.rs`:

- inspect with `symlink_metadata` and never follow package links;
- reject special files;
- enforce count and byte limits while walking, before reading content;
- parse frontmatter once and return typed metadata plus deterministic
  structural disclosure;
- keep fixed production limits in one policy and inject smaller policies only
  inside tests.

Do not simplify away validation, atomicity, ownership checks, or error paths.
Do not execute Skill scripts or assign safety labels.

## Tests and verification

Non-trivial branches need a focused unit or integration test. Boundary tests
must cover exact-limit acceptance and one-over rejection without allocating
production-sized fixtures; `reports_exact_resource_limit_and_observation` is
the pattern.

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
