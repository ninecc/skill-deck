# Catalog search isolation design

## Decision

Separate catalog HTTP access from CLI session resolution. `CliManager::search` validates the query and delegates to a small injectable catalog client/request seam; it never reads or initializes `session` or `inventory`.

## Data Flow

```text
Tauri search command -> CliManager::search(query)
  -> validate query
  -> catalog request seam
  -> decode SearchResponse
  -> map SearchResult
  -> sort installs descending
  -> return stable DTO/error
```

## Test Boundary

Use an in-process deterministic HTTP or client seam consistent with existing Rust test patterns. Tests must fail if session discovery is reintroduced. Do not add a broad provider abstraction: one narrow seam sufficient to inject catalog responses/errors is preferred.

## Compatibility

No Tauri command name, DTO, endpoint, result ordering, or stable error code changes. Install continues to require the pinned CLI session.

## Risks

- A seam placed around all CLI behavior would be needless abstraction; keep it catalog-specific.
- Live network tests would be flaky and privacy-sensitive; focused tests stay hermetic.
