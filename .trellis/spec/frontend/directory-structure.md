# Frontend Directory Structure

- `src/main.tsx` owns only React startup and global stylesheet loading.
- `src/App.tsx` is the current application shell. Split a component when it
  gains an independent interaction or reusable visual contract, not merely to
  shorten a file.
- `src/i18n.ts` is the single static catalog and locale resolver.
- colocate small tests with the source (`src/i18n.test.ts`).
- `src/styles.css` contains the current token-free global visual system. Do not
  add CSS-in-JS, Tailwind, or a component library without a demonstrated need.

Use `PascalCase.tsx` for components and lowercase descriptive names for plain
TypeScript modules. Keep raw filesystem paths, Git payloads, and persisted
state parsing out of React.
