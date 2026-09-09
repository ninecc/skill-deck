# Restore catalog search isolation

## Goal

Allow users to search the upstream catalog even when Node, Skills CLI session resolution, or Inventory startup is unavailable.

## Background

`.trellis/spec/backend/command-contracts.md` requires catalog search to remain independent of runtime and Inventory. `CliManager::search` currently calls `ensure_session()` before issuing the HTTP request, while the frontend deliberately leaves Find & Install available when runtime readiness fails.

## Requirements

- R1: `CliManager::search` must not resolve or require a CLI session.
- R2: Preserve query validation, install-count descending order, result DTO shape, and stable `search_unavailable` / `incompatible_response` errors.
- R3: Add a deterministic backend seam that proves search behavior without contacting the live catalog.
- R4: Prove runtime/session unavailability does not prevent a valid search request.
- R5: Do not change install behavior or permit real Skill mutation.

## Acceptance Criteria

- [x] A search test fails if `search()` calls `ensure_session()` or requires Node/Skills CLI.
- [x] Catalog results remain ordered by installs descending.
- [x] HTTP and response-shape failures retain stable sanitized error codes.
- [x] The existing frontend runtime-failure path can open and use discovery search without Node-install guidance replacing the search error.
- [x] Focused Rust tests, format, and Clippy pass.

## Out of Scope

- Installing a search result.
- Changing catalog endpoint or result DTOs.
- Live external endpoint smoke.
