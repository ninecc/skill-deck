# Require Update All confirmation

## Goal

Prevent accidental bulk updates by requiring explicit confirmation before Update All invokes the backend command.

## Background

`.trellis/spec/backend/command-contracts.md` requires frontend confirmation for Update All. `src/App.tsx` currently invokes `updateSkill(null)` immediately, while localized `confirmUpdateAll` copy already exists.

## Requirements

- R1: Clicking Update All opens a confirmation dialog and does not invoke `update_skill`.
- R2: The safe cancel action receives initial focus; Escape and Cancel close without invocation.
- R3: Explicit confirmation invokes `updateSkill(null)` exactly once through the existing operation/feedback path.
- R4: Reuse `ModalShell`, existing confirmation patterns, and existing localized copy; do not create a second command path.
- R5: Preserve command availability and mutation serialization behavior.

## Acceptance Criteria

- [x] Update All does not invoke the backend before confirmation.
- [x] Cancel and Escape invoke nothing and restore focus to a stable owning control.
- [x] Confirm invokes `update_skill` once with the existing all-Skills payload and retains progress/refresh feedback.
- [x] English and zh-CN confirmation copy render through the existing catalog.
- [x] Focused frontend tests, format, lint, and typecheck pass.

## Out of Scope

- Confirmation for single-Skill Update.
- Install confirmation.
- Backend mutation behavior changes.
- Visual redesign of confirmation dialogs.
