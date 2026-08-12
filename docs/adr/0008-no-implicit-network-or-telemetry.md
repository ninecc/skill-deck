---
status: superseded by ADR-0011
---

# No Implicit Network or Telemetry

Skill Deck performs no network activity during startup, inventory or ordinary lifecycle operations and includes no analytics SDK or automatic crash upload. Only an explicit user-triggered Git action may access the network; diagnostics remain local until the user deliberately exports them, preserving the local-first trust contract at the cost of passive usage and crash data.
