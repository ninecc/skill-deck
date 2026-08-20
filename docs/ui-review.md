# UI review entry

Run `npm run dev`, then open:

`http://localhost:1420/review.html?scenario=shell-ready&theme=dark&locale=en`

The review entry installs deterministic typed Tauri IPC fixtures before mounting
the regular `App`. It never reads the local Skill inventory, filesystem, CLI, or
network. Supported shell scenarios are `shell-ready`, `shell-loading`,
`shell-empty`, `shell-long`, and `shell-zh`. Content review adds
`content-tree`, `content-translation`, `content-translation-loading`, and
`content-translation-error`. Theme can be `system`, `light`, `dark`, `sand`,
or `plum`; locale can be `system`, `en`, or `zh-CN`.

Use WebView viewport sizes 1180×800 and 720×520. Native window decoration is not
part of the React comparison area. The normal production build uses only
`index.html`, so `review.html`, `src/review/`, scenario identifiers, and fixture
payloads must not appear in `dist`.
