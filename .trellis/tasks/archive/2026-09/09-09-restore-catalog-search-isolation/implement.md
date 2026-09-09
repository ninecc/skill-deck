# Catalog search isolation execution plan

1. Add a failing focused test proving valid catalog search does not require session discovery.
2. Add focused ranking and stable network/shape error tests if the seam does not already cover them.
3. Remove `ensure_session()` from the search path and introduce only the minimal deterministic request seam required by the tests.
4. Preserve validation, DTO mapping, sorting, endpoint, and error codes.
5. Run focused Rust tests, `cargo fmt --check`, and Clippy for the crate.
6. Confirm no install/update/remove path changed.
