---
status: accepted
---

# Delegate Skill lifecycle to the upstream CLI

Skill Deck delegates global Skill discovery, installation, removal and update state to one exact Skills CLI version resolved from `skills@latest` per app session, instead of maintaining a Managed Library, ownership model, revisions or reconciliation. This deliberately trades offline fallback and richer update proof for one lifecycle source of truth; Preview and session-only translation remain bounded read-only additions. This supersedes ADRs 0001–0010, whose lifecycle and local-first assumptions belong to the removed manager.
