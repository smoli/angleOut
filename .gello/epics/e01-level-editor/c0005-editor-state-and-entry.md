---
id: c0005
title: Editor state, menu entry and cursor
status: in-progress
epic: e01
depends: [c0004]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T00:19:15
---

## What

Give the editor somewhere to live: a `GameState::Editor`, an "Editor" entry in
the main menu next to New Game and the unused Settings slot, and a visible mouse
cursor while in it.

The level being edited is held in a resource, not in entities, so it survives
the state transitions that `c0013`'s playtest round trip will put it through.

## Acceptance criteria

- [x] `GameState::Editor` exists and is entered from an "Editor" item in the main menu.
- [x] The mouse cursor is visible in the editor and hidden again on leaving.
- [x] The editor camera frames the play field with the whole grid area visible.
- [x] The level under edit lives in a resource and survives a state transition away and back.
- [x] Entering the editor either opens an existing level from `assets/levels/` or starts an empty one.
- [x] Leaving the editor returns to the menu leaving no editor entities behind.

## Notes

- The cursor is hidden globally at startup via `primary_cursor_options` in
  `main.rs`; the editor flips `CursorOptions.visible` on the primary window on
  enter and exit.
- The menu lives in `src/ui/game/mod.rs`, whose `OptionValues` enum and its
  `TryFrom<u8>` both need the new entry.

**As built** - `src/editor/mod.rs`, plus the menu entry, `GameFlowEvent::OpenEditor`
and `GameState::Editor`.

- `Escape` is the way back to the menu, so `close_on_esc` in `main.rs` now stands
  down while the editor is up. Two things on one key would otherwise quit the
  game on the way out of the editor.
- The camera is orthographic and straight down (`ScalingMode::AutoMin` over the
  arena plus a block of margin), so at least the whole play field is on screen
  whatever the window's shape, and a cell is the same size wherever it sits -
  which is what `c0006`'s picking wants. `editor_view()` is that frame, and it
  is drawn, so a grid growing out of it (`c0008`) is visible rather than silently
  clipped.
- The cell grid is drawn with gizmos - the level's blocks themselves are not
  rendered yet, that comes with painting in `c0007`.
- Grid maths shared with the game went into `level::layout` next to `c0002`'s
  conversion: `grid_bounds`, `grid_dimensions` and `empty_grid`.
- `EditorLevel` is inserted on the first entry and never re-read, which is what
  makes the round trip non-destructive. The flip side: if the level file has not
  arrived from the asset server at that moment, the editor opens a blank grid and
  stays on it. Opening a *named* level is `c0012`'s file handling.
- Verified in the running game as well as in tests: the menu reads New
  Game / Editor / Settings, and the editor comes up on `level0`'s 9x6 grid inside
  its frame.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-26 status → in-progress (agent)
- 2026-08-26 editor state, menu entry, cursor, camera and level resource; 69 tests green
