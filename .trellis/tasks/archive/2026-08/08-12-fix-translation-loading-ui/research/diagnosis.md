# Diagnosis

## Reproduction evidence

- Current running build has `http://127.0.0.1:7890` applied.
- Current Brand `SKILL.md` translated successfully twice; the measured repeat
  completed in about 1.5 seconds. The user's timeout is therefore intermittent,
  not a stable missing-proxy failure.
- Code inspection confirms `chooseSkill` invalidates the request but leaves the
  global `translationOn` true, so the next translatable Preview automatically
  enters translated layout and starts a request.
- CSS inspection confirms `.runtime-screen` and the hidden `.workspace` share
  rows without an explicit app-shell column, allowing Grid auto-placement to
  create the second column visible in the startup screenshot.

## Ranked hypotheses

1. Cross-Skill UI leak is caused by `translationOn` surviving `chooseSkill`.
   Prediction: resetting it at that boundary prevents both the double pane and
   automatic request while generation checks still reject late Skill A results.
2. Startup Header narrowing is caused by implicit Grid columns. Prediction: one
   explicit app-shell column makes Header/runtime/workspace span the window.
3. Translation timeout is a transient connect/request stall. Prediction: one
   retry with a per-attempt cap succeeds when the first attempt stalls, without
   exceeding the existing shared deadline.
4. Proxy propagation is missing. Falsified by the persisted proxy value and two
   successful current-build translations through the same setting.

## Feedback loops

- Frontend: a focused Vitest case covering two Installed Skills, pending stale
  translation, and the exact pane/button/request behavior.
- Backend: a Rust unit test backed by a local stdlib HTTP server that delays the
  first response past the per-attempt cap and succeeds on the second.
- Layout: current-build desktop startup smoke comparing Header/runtime width;
  jsdom remains the wrong seam for pixel Grid placement.
