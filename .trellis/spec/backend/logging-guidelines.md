# Backend Logging and Diagnostics

The application currently has no logging dependency. Keep it that way until a
diagnostic export flow exists; do not add `println!`, analytics, crash upload,
or a logger only for development convenience.

When diagnostics are implemented, they must follow `design.md` for the active
task: remain local, require explicit export, preview included fields, and omit
credentials, Skill bodies, and unnecessary full home paths. Structured error
DTOs are the user-facing diagnostic mechanism today.
