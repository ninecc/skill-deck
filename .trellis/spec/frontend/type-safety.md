# Frontend Type Safety

TypeScript strict mode is required by `tsconfig.app.json`. Avoid `any`, unchecked
casts, and local reinterpretations of Tauri payloads.

- Define each command DTO once at the command boundary and consume that type in
  components.
- Normalize `unknown` command failures before rendering them.
- Keep domain enum values separate from localized labels.
- Use exhaustive `switch` statements for operation or status variants.
- `src/i18n.ts` uses `as const` and a `Locale` key union; the catalog alignment
  test prevents one language from silently missing a key.

The locale select cast in `App.tsx` is allowed because every option value is
declared in the same component from the closed `Locale` set. Do not use that
pattern for external data.
