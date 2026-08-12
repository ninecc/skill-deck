---
status: superseded by ADR-0011
---

# Reconcile Managed Installations without automatic repair

Skill Deck derives a single primary Installation Status, factual expected/observed evidence, and ownership-safe available actions from persisted state plus current filesystem and Agent configuration state whenever inventory is loaded. Reconciliation remains read-only: it does not persist health, infer new ownership, or repair anything automatically; every recovery still requires an explicit mutation whose backend revalidates current state. This keeps the home view truthful without turning stale inventory evidence into mutation authorization or allowing React to duplicate ownership policy.

Full reconciliation evidence remains local to inventory/UI. Shared diagnostics exports continue to omit resolved targets and full fingerprints, and rebuilding a broken Managed Library from a source or Previous Revision remains a separate recovery concern.
