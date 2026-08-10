# Prefer App-owned Linked Installations

Skill Deck keeps each Installed Revision in a private app-data Managed Library and prefers Agent entries that link to it: relative directory symlinks on macOS/Linux and directory junctions on Windows. It never writes third-party lock files. Link failure requires explicit Copy Fallback, every Installation records its actual Deployment Mode and provenance, and Adoption of third-party links migrates content into the Managed Library rather than claiming or mutating unknown targets.
