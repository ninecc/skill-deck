# Skill Deck

Skill Deck manages user-owned Agent Skills across supported AI coding agents while preserving the user's existing files.

## Language

**Skill Consumer**:
An individual developer who installs and maintains Skill Packages for personal use across one or more Agent Targets.
_Avoid_: Skill author, administrator, publisher

**Skill Package**:
A directory whose root contains a valid `SKILL.md` and whose remaining files belong to that Skill as one portable package.
_Avoid_: Plugin, extension, script

**Agent Target**:
A supported AI coding agent and its user-level Skill scope, currently Codex or Claude Code.
_Avoid_: Platform, provider, runtime

**Installation**:
A Skill Package present in one Agent Target's official user-level Skill directory.
_Avoid_: Skill, copy, deployment

**Deployment Mode**:
The recorded mechanism by which an Installation exposes its Managed Skill Package content to an Agent Target.
_Avoid_: Installation type, source type

**Linked Installation**:
An Installation whose Agent Target entry is a Skill Deck-owned symlink or junction resolving into the Managed Library.
_Avoid_: Linked Skill, external link

**Copied Installation**:
An Installation whose Agent Target entry is a Skill Deck-owned standalone copy of the Installed Revision.
_Avoid_: Local Skill, unmanaged copy

**Copy Fallback**:
The user's explicit choice to create a Copied Installation after a Linked Installation cannot be created.
_Avoid_: Automatic fallback, degraded mode

**Managed Skill Package**:
A stable, user-visible Skill Package entity whose lifecycle Skill Deck is authorized to manage; changing its Installed Revision does not change its identity.
_Avoid_: Managed Skill, owned Skill, registered Skill

**External Installation**:
An Installation discovered in an Agent Target but not authorized for mutation by Skill Deck.
_Avoid_: External Skill, unmanaged Skill, unknown Skill

**Legacy External Installation**:
An External Installation discovered in an Agent Target's deprecated compatibility Skill root.
_Avoid_: External Installation, Managed Skill Package

**Legacy Migration**:
The import of a Legacy External Installation into the Managed Library without creating a current-root Installation until the legacy entry is removed externally.
_Avoid_: Adoption, move, automatic migration

**Broken External Installation**:
An external link entry whose target is missing, cyclic, invalid, or otherwise unsafe to resolve as an installable Skill Package.
_Avoid_: External Installation, Content Drift

**Adoption**:
The user's explicit grant of lifecycle control over an External Installation to Skill Deck, creating or attaching it to a Managed Skill Package.
_Avoid_: Import, claim, register

**Skill Source**:
The stable provenance from which a Managed Skill Package obtains revisions; it does not include a particular content revision.
_Avoid_: Source revision, origin

**Local Skill Source**:
A local directory captured as an immutable import snapshot; later edits to the original directory are not synchronized.
_Avoid_: Watched folder, linked source

**Git Skill Source**:
A public HTTPS repository URL, Skill subpath, and tracked branch that identify stable Git provenance.
_Avoid_: Git URL, repository, Git revision

**Installed Revision**:
The exact content snapshot currently held by a Managed Skill Package; a Git-backed revision includes its commit OID.
_Avoid_: Skill version, Source, release

**Previous Revision**:
The single Installed Revision immediately preceding the current one and retained for one explicit rollback.
_Avoid_: Revision history, backup

**Roll Back Revision**:
The explicit replacement of the current Installed Revision and all its Installations with the Previous Revision.
_Avoid_: Restore Installation, downgrade, undo

**Managed Library**:
Skill Deck's app-owned storage for Managed Skill Packages and retained revisions, before content is installed into an Agent Target.
_Avoid_: Agent directory, repository, cache

**Add to Library**:
The import of a Managed Skill Package with zero Agent Target Installations.
_Avoid_: Install, Adoption

**Resource Boundary**:
An explainable and testable limit applied to an untrusted Skill Source before content may enter the Managed Library.
_Avoid_: Security scan, risk score, quota

**Configuration Provenance**:
The recorded creator of an Agent Target configuration state, distinguishing Skill Deck changes from pre-existing user or third-party changes.
_Avoid_: Configuration ownership, config history

**Externally Controlled Configuration**:
An Installation configuration state created by the user or a third party that Skill Deck may report but must not modify.
_Avoid_: Configuration Drift, unmanaged configuration

**Configuration Drift**:
An external change to configuration previously created and controlled by Skill Deck.
_Avoid_: Externally Controlled Configuration, Content Drift

**Structural Validation**:
A deterministic check that a directory satisfies the installable Skill Package contract.
_Avoid_: Security scan, safety check

**Capability Disclosure**:
A factual inventory of scripts, declared tools, references, and unknown fields contained in a Skill Package.
_Avoid_: Risk score, safety result

**Change Disclosure**:
A factual comparison of capabilities between an installed Git revision and an available update.
_Avoid_: Changelog, security report

**Content Drift**:
A difference between an Installation's current content and its Managed Skill Package's Installed Revision.
_Avoid_: Dirty state, local changes, corruption

**Source Diverged**:
The state in which a Git Skill Source's remote tracked branch cannot fast-forward from the Installed Revision.
_Avoid_: Update available, conflict

**Source Unreachable**:
A temporary inability to contact or read a Git Skill Source.
_Avoid_: Source Missing, Source Diverged

**Source Missing**:
The state in which a Git Skill Source is reachable but its tracked branch or Skill subpath no longer exists.
_Avoid_: Source Unreachable, Source Diverged

**Read-only Recovery**:
An application mode that prevents lifecycle changes because Skill Deck cannot establish reliable ownership from its persisted state.
_Avoid_: Safe mode, automatic recovery

**Orphaned Package**:
App-owned Skill Package content found during Read-only Recovery without a trustworthy Managed Skill Package record.
_Avoid_: Managed Skill Package, External Installation

**Restore Installation**:
The explicit replacement of a drifted Installation with its Managed Skill Package's Installed Revision.
_Avoid_: Update, reset

**Detach Installation**:
The removal of an Installation from a Managed Skill Package while preserving its content as a standalone External Installation; a linked entry is converted to a copy first.
_Avoid_: Uninstall, delete, abandon

**Uninstall**:
The removal of one Installation from an Agent Target without removing its Managed Skill Package from Skill Deck.
_Avoid_: Delete Skill, remove from library

**Remove from Library**:
The deletion of a zero-Installation Managed Skill Package, including its app-owned content and Skill Source record.
_Avoid_: Uninstall, delete Installation
