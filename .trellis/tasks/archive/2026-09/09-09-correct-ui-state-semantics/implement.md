# UI state semantics execution plan

1. Rebase on the completed Update All child before editing `src/App.tsx`.
2. Add focused failing tests for empty Inventory plus filter, Preview empty guidance, startup status semantics, translation status semantics, and narrow translation failure visibility.
3. Correct empty-state derivation without adding duplicate state.
4. Correct shared status precedence so startup and translation are not Ready while preserving mutation/runtime error behavior.
5. Keep egress disclosure visible at 720px and make the narrow translation failure path immediately discoverable.
6. Update only the deterministic scenarios needed to exercise the affected states.
7. Run focused frontend tests, format, lint, typecheck, and production build.
8. Launch the review entry and inspect affected scenarios at 1180x800 and 720x520.
