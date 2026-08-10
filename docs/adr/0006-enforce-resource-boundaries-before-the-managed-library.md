# Enforce Resource Boundaries Before the Managed Library

Every Skill Source is untrusted, so Skill Deck applies fixed, versioned, explainable and testable Resource Boundaries before any content enters the Managed Library: 250 MiB Git transfer, 500 MiB checked-out repository, 100 MiB selected Skill Package, 10,000 files, 50 MiB per file and 1 MiB `SKILL.md`. Exceeding any boundary aborts and cleans up the entire import transaction; these limits prevent resource exhaustion but are not malicious-code detection or a safety judgment.
