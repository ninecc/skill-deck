---
status: superseded by ADR-0011
---

# Model Skill Packages Separately from Installations

Skill Deck presents one Skill Package with zero or more Agent-specific Installations, rather than treating every installed copy as a separate Skill. This makes cross-Agent install and update behavior coherent while still keeping same-name, different-content External Installations separate until the user resolves them.
