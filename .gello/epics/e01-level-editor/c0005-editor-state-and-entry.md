---
id: c0005
title: Editor state, menu entry and cursor
status: ready
epic: e01
depends: [c0004]
created: 2026-08-25
updated: 2026-08-25
status-changed: 2026-08-25T22:59:42
order: 50
---

## What

Give the editor somewhere to live: a `GameState::Editor`, an "Editor" entry in
the main menu next to New Game and the unused Settings slot, and a visible mouse
cursor while in it.

The level being edited is held in a resource, not in entities, so it survives
the state transitions that `c0013`'s playtest round trip will put it through.

## Acceptance criteria

- [ ] `GameState::Editor` exists and is entered from an "Editor" item in the main menu.
- [ ] The mouse cursor is visible in the editor and hidden again on leaving.
- [ ] The editor camera frames the play field with the whole grid area visible.
- [ ] The level under edit lives in a resource and survives a state transition away and back.
- [ ] Entering the editor either opens an existing level from `assets/levels/` or starts an empty one.
- [ ] Leaving the editor returns to the menu leaving no editor entities behind.

## Notes

- The cursor is hidden globally at startup via `primary_cursor_options` in
  `main.rs`; the editor flips `CursorOptions.visible` on the primary window on
  enter and exit.
- The menu lives in `src/ui/game/mod.rs`, whose `OptionValues` enum and its
  `TryFrom<u8>` both need the new entry.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
