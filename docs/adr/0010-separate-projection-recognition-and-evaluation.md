---
status: superseded by ADR-0011
---

# Separate projection, runtime recognition, and effectiveness evidence

Skill Deck treats three verification levels as distinct: Agent Projection Contract tests deterministically validate official roots, configuration shapes, and native link/junction behavior without launching an Agent; installer smoke tests prove Agent Runtime Recognition after launching a packaged application and installed Agent; Revision-level Eval later measures model-dependent triggering and task success. This separation keeps native CI deterministic and prevents a passing filesystem test from being presented as evidence that a Skill was loaded, invoked, or effective.
