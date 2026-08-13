# Skill Deck desktop app comprehensive redesign

## Goal

Redesign Skill Deck as a quiet, precise, trustworthy, compact, recognizably
native cross-platform desktop utility for inventory, installation, Preview and
updates, while preserving the accepted CLI-backed lifecycle, single-window
command architecture, accessibility, localization and preference contracts.

Direction B is the Approved Visual Direction under ADR-0016. Implementation
still requires review and explicit approval of the final task-local planning
summary before this task may leave `planning`.

## Background and authority

- Accepted ADRs and current implementation take precedence over conflicting
  roadmap history.
- The session-pinned upstream Skills CLI Inventory is lifecycle truth. Managed
  Library, reconciliation, ownership, rollback and app-owned marketplace models
  are historical and must not return.
- The current production appearance and all archived prototypes are Historical
  Visual Direction, not a visual starting point or approval.
- The persisted theme values remain `system | light | dark | sand | plum`.
  Theme Accent remains separate from native System Accent; success, warning and
  danger remain independent semantic roles.

## Requirements

### Product and behavior contracts

- Preserve the single-window utility model; do not add New Window, New
  Document, custom traffic lights or web-style window chrome.
- Preserve the Application Command registry and dispatcher as the single
  behavior authority. Toolbar, native menu, keyboard and context menu remain
  adapters over the same command IDs and availability reasons.
- Preserve macOS native menus and Windows/Linux application-menu conventions
  plus the shared WebView toolbar.
- Preserve existing shortcuts, Esc behavior, arrow-key navigation, focus trap,
  focus restoration, disabled reasons and modal exclusivity.
- Preserve startup runtime probe and Retry; empty Inventory; no-match filter;
  selected Skill and file tree; Markdown, text, code, image and unsupported
  Preview; Reveal; Update; Remove confirmation; Find & Install; direct-source
  installation; translation and Google egress disclosure; diagnostics; and
  neutral, success, partial and error feedback.
- Preserve current preference behavior and storage. Most settings save
  immediately; Translation Proxy remains a dialog-local draft until Apply.
  Close is not Cancel.
- Preserve Skill Deck naming, domain terms, English/Chinese UI and localization.

### Visual and layout outcomes

- Cover the default 1180×800 window and the minimum 720×520 window as desktop
  layouts, not mobile layouts.
- Keep Preview as the primary content area with readable Markdown measure,
  legible code and original/translation comparison.
- Reorganize Settings so Appearance, UI/translation language, Translation
  Proxy, about 70 Agent targets, install method and CLI version do not form one
  undifferentiated scrolling sheet.
- Make immediate-save versus Apply-Proxy semantics explicit without changing
  persistence behavior.
- Make keyboard focus visible on the installed filter and theme radios.
- Distinguish Refresh Inventory, Update All and Update Skill by placement,
  labels and icon semantics.
- Handle long Skill names, sources, root paths and file paths without hiding the
  identity needed to choose or reveal content. A resizable sidebar may be
  proposed but is not approved by this planning task.
- Give loading, runtime failure, empty Inventory, no match, Preview failure,
  translation failure, unresolved install and success/partial outcomes a stable
  visual location, clear severity and a next action where one exists.
- Use a primitive → semantic → component token architecture for color,
  typography, spacing, density, radii, border/elevation, focus and motion.
- Keep desktop controls approximately 28–32 px high. Avoid touch-first sizing,
  cards for every region, marketing composition, decorative gradients, glass
  stacks, Bento layouts, floating CTAs and scroll storytelling.
- Use only system/local fonts and resources. Do not add Tailwind, shadcn, a new
  UI framework, remote fonts, stock imagery or CDN assets without a separate
  rationale and approval.
- Motion must communicate feedback, hierarchy or state, finish within 300 ms,
  be immediate for keyboard navigation and fully respect
  `prefers-reduced-motion`.
- Visual approval evidence must be high fidelity enough to judge typography,
  spacing rhythm, icon semantics, control states, path treatment, dialog
  composition and platform-native feel. A structurally correct wireframe or
  coarse interactive mockup is insufficient for approval.

### Platform scope

- The proposal defaults to the shared WebView UI for macOS, Windows and Linux,
  with native differences in menu presentation, modifier labels, system
  controls, font resolution and platform control rendering.
- Platform guidance may check behavior and accessibility but cannot override
  ADR-0012 or establish visual authority.

## Acceptance criteria

- [x] A task-local audit records current code, test and running-window evidence,
      including state coverage and observed usability defects.
- [x] Two materially different desktop-utility directions are documented with
      a domain-specific proposition and 1180×800 and 720×520 ASCII wireframes.
- [x] Each direction covers the main window, Settings, Find & Install, Remove
      confirmation, file tree, status bar and runtime/loading/empty/error states.
- [x] Each direction defines color, typography, spacing, density, radius,
      border/elevation, focus and motion tokens.
- [x] Each direction states shared and platform-specific behavior,
      accessibility/keyboard handling, behavior-contract impact,
      implementation cost, risk and rollback boundary.
- [x] A recommendation compares task speed, status readability, command
      discoverability, information density, cross-platform consistency and
      implementation risk rather than aesthetics alone.
- [x] A task-external, thread-local interactive structural prototype renders both
      directions at 1180×800 and 720×520 with Preview, Translation, Settings,
      Find & Install, Remove, loading, runtime failure, empty Inventory and
      Preview error scenes; it is review evidence only and cannot be promoted
      to production code without explicit approval.
- [x] Direction B is refined to high-fidelity visual approval quality
      and reviewed at actual 1180×800 and 720×520 scale; the current structural
      prototype has been superseded for visual review by the refined B surface.
- [x] The user explicitly approves one direction and identifies its UI scope and
      macOS/Windows/Linux platform scope in an unambiguous reply.
- [x] After approval, the approved direction and scope are distilled into
      task-local `design.md` and `implement.md` for final review.
- [x] A subsequent explicit approval of the final planning summary authorizes
      `task.py start` and production implementation.

## Out of scope

- Any lifecycle, command, preference-schema, localization or security-model
  change.
- A new UI framework, remote asset, custom native chrome, marketplace, Managed
  Library, security score, Agent runtime, silent update or background execution.
- Any layout below the contracted 720×520 desktop minimum.
- A resizable Inventory sidebar in this implementation; it may be evaluated in
  a separate explicitly approved task.

## Approved visual direction and scope

On 2026-08-13 the user gave ADR-0016-compliant approval for Direction B.

- **UI scope:** main window, toolbar and command hierarchy, Inventory/filter,
  floating file tree, Preview, Settings, Find & Install, Remove confirmation,
  status bar, and all loading/runtime/empty/error/success/partial states.
- **Platform scope:** the shared WebView UI on macOS, Windows and Linux, while
  preserving native menu, shortcut, window and pointer differences on each
  platform.
- **Visual character:** an extremely restrained professional desktop utility,
  using low-chroma graphite surfaces and monochrome 1.5 px icons with sparse
  Theme Accent detail for orientation rather than decoration.
- **Viewport contract:** 1180×800 default and 720×520 minimum, both as two-pane
  desktop layouts. Sub-720 and mobile adaptations are excluded.
- **Navigation contract:** Inventory and Preview are the persistent panes. The
  file tree is a trigger-anchored popover, not a persistent third pane.

## Decision history

The user selected Direction B on 2026-08-13 for high-fidelity refinement before
later giving the full approval above.

After reviewing the refined B prototype, the user confirmed the modal/state
switching fix and requested another visual-detail pass, specifically including
palette and icon usage. The user chose an “extremely restrained professional
tool” character: low-chroma graphite surfaces, monochrome 1.5 px icons, no
decorative color, and accent color limited to focus, selection, and small
command/state signals while success, warning, and danger remain independent
semantic roles. Direction B's structure remains the working direction. This
visual preference is planning evidence, not production approval.

The user then clarified that restraint should not mean total visual neutrality.
Small accents are welcome when they improve craft and orientation. The refined
rule is: retain neutral large surfaces and commands, while allowing sparse
Theme Accent details on the brand glyph, install affordance, selected row,
active tabs, code edge, and state symbols. Decorative gradients, broad tinted
panels, multicolor peer icons, and ornamental effects remain excluded.

The user identified the selected-Skill header as the next refinement target and
explicitly chose a two-column workspace with a floating file tree. Inventory
and Preview remain the two persistent panes at both target sizes. A compact
current-file control anchors a 244 px popover tree with clear file/folder
glyphs, disclosure chevrons, a restrained selected-file rail, root identity,
and file count. Source and install path remain separate provenance metadata.
Translate and Reveal form the content/navigation command group, separated from
Update Skill and Remove as lifecycle commands. At 720×520, the same popover is
width-bounded and the commands use a non-wrapping second row. Esc closes the
popover and restores focus to its trigger. No command ID, availability rule,
shortcut, or file-selection behavior changes.

The user reaffirmed that Skill Deck is a desktop application and that viewports
smaller than the contracted minimum do not need product-layout support. The
responsive floor remains 720×520 from the original brief. That size is treated
as a compact desktop window with both Inventory and Preview still visible, not
as a phone-style single-pane breakpoint. No mobile navigation, touch sizing,
safe-area behavior, or sub-720 product breakpoint will be designed.

The user explicitly confirmed this desktop responsive strategy on 2026-08-13,
then included it in the full Direction B approval.

After the first production implementation pass, the user rejected its visual
fidelity: the result did not match even the basic visual effects of the final
approved reference. This is implementation drift, not a new visual-direction
decision. Completion now requires direct production-to-reference comparison and
correction of palette, typography/weight, density, toolbar and selected-Skill
header hierarchy, command treatment, selection states, Preview rhythm, file
popover placement and Settings/dialog composition.
