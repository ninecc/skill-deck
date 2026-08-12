# Technical Design

## Boundaries

Keep the existing Tauri command surface and React state model. Move blocking
work off Tauri's command/event-loop thread with `async` commands plus
`tauri::async_runtime::spawn_blocking`. Do not convert the CLI adapter or
translation parser to a second async architecture.

```text
React renders startup shell immediately
  -> async Tauri command
     -> spawn_blocking
        -> existing CliManager / preview / blocking reqwest
```

## Startup

React already has a `runtime === null` state. Render the full app header plus a
stable startup skeleton/status in that state. Settings and UI locale remain
usable while runtime-dependent and Inventory-dependent controls are disabled.
React must paint before the async backend command starts blocking work. Do not
add a splash window, cached Inventory, preload process, or second startup state
machine.

On macOS, resolve the Node toolchain once per CLI session: first use the
inherited PATH, then check `/opt/homebrew/bin` and `/usr/local/bin`. Select a
`node` only when an executable sibling `npx` exists, and run both by absolute
path for the rest of the session. Do not invoke a login shell or source user
profile scripts. Other platforms retain PATH resolution.

`RuntimeStatus` exposes a stable optional error code while keeping its bounded
message for compatibility. React maps known runtime codes to localized title
and recovery copy; raw process-spawn errors remain backend-only.

## Translation network client

Extend preferences with `translationProxy: string`. Empty means reqwest's
default automatic environment proxy behavior. A non-empty override is sent with
`translate_preview` and validated in Rust:

- maximum 2,048 bytes;
- `http` or `https` URL with a host;
- no username/password, fragment, path, or query.

Build a request-scoped blocking client with a 5-second connect timeout. Give the
whole translation operation, including every provider chunk, one shared
15-second deadline; each request receives only the remaining duration. Add
`reqwest::Proxy::all` only for a non-empty override. Do not retry automatically.
Accumulate chunks privately and publish `TranslationResult` only after every
chunk succeeds; timeout or any provider failure discards the partial buffer.

Map every provider-originated failure—connection, timeout, HTTP status, decode,
or incompatible response—to a short stable command error and localized UI
copy. Never serialize reqwest's URL-bearing error string to React.

## Frontend UX

Settings adds one optional `Translation proxy` input near target language, with
example `http://127.0.0.1:7890`; blank uses environment/default networking. The
field edits dialog-local draft state. Apply validates it in the frontend before
updating persisted Preferences; Rust independently validates every command
input.

Translation error state includes Retry. A monotonically increasing request
generation enforces latest-request-wins when translation is closed, a file,
language, or proxy changes, or Retry starts another request.

## Verification and Rollback

- Test proxy validation/client configuration without contacting Google.
- Test command futures yield while blocking work is pending.
- Test startup shell and translation Retry/error behavior in React.
- macOS smoke delayed startup and an unreachable/local test proxy.

Rollback is one commit; older builds ignore the optional localStorage field.
