# Separate Theme Accent from System Accent

Theme Preference controls Skill Deck's application appearance and the Theme Accent used by custom components, while native platform affordances keep their system appearance and System Accent where supported. Success, warning and danger remain independent semantic roles. This separation does not require any permanent theme count, theme names, palette mappings, or visual identity.

[ADR-0016](./0016-govern-visual-direction-authority.md) partially supersedes the earlier requirement to preserve all five complete themes and their then-current visual identity. The Theme Accent/System Accent separation and independent semantic roles remain accepted architecture; changing the current runtime preference schema still requires an explicit product migration.
