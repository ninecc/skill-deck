# Backend Logging and Diagnostics

The application currently has no logging dependency. Keep it that way until a
diagnostic export flow exists; do not add `println!`, analytics, crash upload,
or a logger only for development convenience.

CLI stdout/stderr may be returned only as bounded, control-character-cleaned
diagnostic text beside a structured error or refreshed mutation result. Never
log credentials or Skill bodies. `CommandError` remains the user-facing
diagnostic mechanism; do not add an export subsystem without a product need.
