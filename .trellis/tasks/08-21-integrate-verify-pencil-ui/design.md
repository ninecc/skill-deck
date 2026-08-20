# Technical design

This child adds no new architecture. It exercises the integrated result through
the same entry points users receive: production Vite build and Tauri desktop
window. Fixes should occur at the owning shared seam (token, Icon wrapper,
layout component, modal shell or state renderer), not as proof-specific CSS.

Record a matrix of theme × size × state × locale. Automated tests establish
behavior; screenshots/native smoke establish visual acceptance. Keep each fix
small and rerun the affected child evidence plus the full quality gate.

Rollback integration fixes individually. If a requested fix changes product
scope, return to parent planning rather than absorbing it here.
