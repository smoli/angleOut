---
id: c0005
title: Editor state, menu entry and cursor
status: done
epic: e01
depends: [c0004]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T21:55:16
usage-tokens: 77131
usage-cost: 9.866317
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

## Review

### 2026-08-26T00:40:23 — pass

Checked: all six acceptance criteria against the code, the full diff of
`60e61e3`, `cargo test` and `cargo build`.

- `cargo test`: 69 passed, 0 failed. `cargo build`: clean, 20 warnings, all
  pre-existing (`world_to_cell` unused since `c0002`, `triggerStates` naming);
  none from `src/editor/`, `src/level/layout/` or `src/ui/game/`. The repo has
  no lint or typecheck step beyond these.
- "`GameState::Editor` exists and is entered from an Editor item": the variant
  is in `src/state/mod.rs`, the item is spawned between New Game and Settings
  in `ui_spawn`, and `ui_update` maps it to `GameFlowEvent::OpenEditor`, which
  `game_flow_handler` (`src/events/mod.rs:220`) turns into the state. Covered by
  `picking_the_editor_entry_asks_for_the_editor`, plus
  `every_menu_entry_is_reachable_by_walking_the_menu`, which pins the `u8`
  discriminants against `TryFrom` - the failure mode that would otherwise make
  the new entry unreachable without breaking the build.
- "Cursor visible in the editor, hidden again on leaving": `editor_show_cursor`
  / `editor_hide_cursor` on `OnEnter`/`OnExit`, tested end to end from the
  hidden state `WindowPlugin` starts in.
- "Camera frames the play field, whole grid area visible": orthographic,
  `ScalingMode::AutoMin` over `editor_view()` (arena + one `BLOCK_WIDTH` of
  margin, so x ±115, z ±85), asserted by `the_editor_camera_frames_the_view`
  and, against real data, by `every_shipped_level_fits_in_the_editor_view` over
  all 11 level files. Checked the two cases that test does not cover: the blank
  9x6 grid spans x -75.5..75.5, z -71.8..-16.7, and the widest shipped grid
  (11 columns) x -92.5..92.5 - both inside the frame. `OnExit(GameState::InGame)`
  tears down `Environment3d`, so the editor camera is the only one rendering.
- "Level under edit lives in a resource and survives a state transition":
  `EditorLevel` is inserted only when absent (`editor_open`) and never removed;
  `the_level_under_edit_survives_a_trip_out_of_the_editor_and_back` edits it,
  leaves, returns and asserts the edit is still there.
- "Opens an existing level or starts an empty one": `open_current_level` falls
  back to `EditorLevel::blank()`; both branches tested.
- "Leaving leaves no editor entities behind": everything spawned carries
  `EditorEntity`, `editor_teardown` despawns it, asserted by
  `leaving_the_editor_takes_everything_it_spawned_with_it`. `Escape` reaching
  both `editor_leave` and `close_on_esc` in the same frame is ruled out by the
  `not(in_state(Editor))` run condition, which still sees `Editor` that frame.
- Diff scope: `src/ui/game/mod.rs` and `src/state/mod.rs` also carry their bevy
  0.19 migration. Out of the card's What in the strict sense, but the tree does
  not compile without it and `c0004` set the same precedent; the commit message
  discloses it. The dropped `ActionState::consume` calls are forced - the method
  no longer exists in leafwing-input-manager 0.21. No debug leftovers, no test
  weakened or skipped.
- Not verified by me: the "verified in the running game" note - I checked
  statically and through the suite, not by launching the window.
- Heads-up for `c0006`/`c0007`, not a defect in this card: `grid_dimensions`
  drops blank lines anywhere and counts only slots of length >= 2, while
  `interpret_grid` enumerates every line and takes its column count from a raw
  `split(" ")` of the first one. They agree on all shipped levels (and the tests
  pin that), but a layout with a blank interior row, or a first line with a
  leading space, would put the editor's grid one row or one column off from
  where the game spawns the blocks.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-26 status → in-progress (agent)
- 2026-08-26 editor state, menu entry, cursor, camera and level resource; 69 tests green
- 2026-08-26 status → review (agent)
- 2026-08-26 status → done (app)
