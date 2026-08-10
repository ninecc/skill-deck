# Never Infer Lifecycle Ownership

Skill Deck only mutates files and Agent configuration whose ownership and Configuration Provenance it can prove from valid persisted state. Adoption preserves pre-existing configuration provenance, and failed state recovery enters Read-only Recovery rather than inferring ownership from matching paths or content; this favors user-data safety over automatic repair.
