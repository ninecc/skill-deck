# Refactor Settings UI

## Goal

Implement the completed Pencil Settings contract across General, Appearance,
Translation, Installation and About, including all documented edge states,
theme/localization proofs and desktop keyboard behavior.

## Requirements

- Match Reference 13, the eight-state Dark matrix, Light Appearance proof and
  Chinese Installation stress proof.
- Use stable 720×548 dialog chrome: fixed header, five-section navigation and
  footer with an independently scrolling content viewport.
- Open on General with initial focus on the first section button. Preserve
  logical Tab/Shift+Tab order, radio/list arrow behavior, Escape/header-close/
  footer-close dismissal and trigger focus restoration.
- Persist interface language, theme, translation target, install method and
  Agent targets immediately. Keep Translation Proxy draft-only until Apply and
  show invalid values inline without a false saved state.
- Preserve System (Default), Light, Dark, Sand and Plum theme choices.
- Preserve established localized production copy unless a wording correction is
  separately approved.
- Preserve upstream Agent registration order. Do not implement the Pencil
  recommendation to alphabetize targets in this task.
- Support automatic targets, explicit targets, filtering/no matches, long lists
  and unavailable CLI version.
- Add deterministic dev/review scenarios for the Dark matrix, Light proof and
  Chinese stress proof; keep them absent from production output.

## Dependency

- Requires Children 1–3 to be completed, checked, committed and archived.

## Acceptance Criteria

- [x] All eight Dark matrix states and both isolated proof states match Pencil.
- [x] Header/navigation/footer remain stable while long Installation content
      scrolls independently without clipping at 720×520.
- [x] Immediate-save and proxy-Apply boundaries are behaviorally tested.
- [x] Keyboard navigation, dismissal and focus restoration are tested.
- [x] English and Simplified Chinese layouts remain readable; Agent order is
      unchanged from upstream registration order.
- [x] Full frontend quality gate passes.

## Out of scope

- Alphabetizing Agent targets, new preferences or backend settings behavior.
