---
id: c0007
title: Grid painting
status: in-progress
epic: e01
depends: [c0006]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T00:56:22
---

## What

Actually edit the grid: a brush describing what to place, and click or drag to
place and erase it.

The brush covers the full token: block type, behaviour, trigger type and trigger
group, plus an erase mode. `Z` / `BlockType::Obstacle` is a normal brush — despite
the name it is an ordinary grid cell that happens to be unbreakable, unrelated to
`LevelObstacle`.

## Acceptance criteria

- [x] A brush resource holds block type, behaviour, trigger type and trigger group, plus an erase mode.
- [x] Clicking a cell writes the current brush into the in-memory level.
- [x] The erase brush clears a cell back to empty (`..`).
- [x] `Z` / `BlockType::Obstacle` can be painted.
- [x] Painted cells appear immediately as blocks in the editor view.
- [x] Dragging paints a run of cells without a click per cell.
- [x] The edited layout serializes to a token grid that `interpret_grid` parses back to the same blocks.

## Notes

- Trigger group is `0..=9`, encoded as the 4th character and only valid when a
  trigger type is present. A brush with a trigger type but no group is invalid.
- Drag painting needs to be one undo step later (`c0011`), so record a drag as a
  single edit from the start rather than per-cell.

**As built** - `src/level/layout/mod.rs` (the format), `src/block/mod.rs` (how a
block looks), `src/editor/mod.rs` (the brush, the mouse and the blocks on
screen).

- The format gained its inverse. `block_token(block_type, behaviour, trigger)`
  writes the token `make_block` would read the block back out of, and lives next
  to it so the two alphabets are one screen apart. A test walks all 2295 tokens
  the format defines - every type x behaviour x (no trigger, or a type and a
  group) - through `make_block` and back out again, because a single letter out
  of step would change a level's shape the next time an author touched a cell of
  it.
- The invalid brush the card warns about cannot be built. Trigger type and group
  are *one* field, `Option<(TriggerType, TriggerGroup)>`, since half a trigger is
  a token `make_block` reads back as something else. `block_token` is the outer
  wall: handed a group the format has no digit for, it writes no trigger rather
  than half of one.
- Painting is `set_cell(layout, col, row, token)`, which rewrites the whole grid
  rather than patching the one line. That is what pads every row out to the
  widest one - `interpret_grid` takes the column count off the first line only,
  so a ragged grid centres its rows on a width they do not have. Painting a
  ragged layout squares it up and moves its blocks; that is the repair, not the
  damage. A write outside the grid writes nothing: growing it is `c0008`'s.
- `Z` needed no special case, which was the point of the card saying so. It is a
  letter of the format like the other four, so it is a brush like any other; the
  test paints two of them and finds two `Obstacle` blocks.
- **The editor had no blocks in it at all before this.** `c0005` and `c0006` drew
  the grid and the hover, and the level itself was invisible. `editor_show_blocks`
  puts the level's blocks on screen and puts them there again whenever
  `EditorLevel` changes - which, while painting, is the frame the cell was
  painted in. It is one chain with picking and painting, so nothing trails by a
  frame.
- Only what actually changed counts as a change. `paint_cell` says whether it
  wrote anything and the paint goes in through `bypass_change_detection`, so
  dragging the erase brush over empty cells does not respawn every block on
  screen once a frame.
- Blocks are dressed in the game's own mesh and `BlockMaterial`, not in an
  editor-coloured stand-in - `block_material` moved out of `block_spawn` so there
  is one table of what a `Concrete` block looks like rather than two. Dressing is
  a second system, because the level is on screen the instant it is edited where
  the glTF arrives whenever it arrives; it is chained straight after the spawn so
  a repaint never leaves the grid meshless for a frame.
- A `FilledGrid` is written out as the token grid that says the same thing the
  first time a cell of it is painted - a grid with one cell changed is no longer
  "the same block everywhere". This is not a corner: `LevelDefinition::default`
  is `FilledGrid(5, 5, ...)`, so it is every level file that names no layout.
- The right mouse button erases whatever the brush is set to. Without it there is
  no way to clear a cell until `c0009`'s palette can switch the brush's own erase
  mode on - and it is where a level author's hand goes anyway.
- A drag is one edit from the start, as the card asks. `PaintStroke` opens when a
  button goes down, holds the layout as it stood at that moment and the cells the
  pointer has crossed, and closes when the button comes up. The cell list is also
  what stops a resting pointer rewriting its cell every frame; `c0011` has one
  entry to push rather than a hundred identical ones to collapse.
- A run condition is evaluated every frame whether or not the `in_state` beside
  it holds - Bevy runs them all rather than short-circuiting, so change detection
  in a condition cannot miss a change. `resource_changed::<EditorLevel>` therefore
  panicked on the main menu, before the editor had ever been opened. Caught by
  running the game, not by the tests; fixed with
  `resource_exists_and_changed`, and pinned by
  `the_editor_asks_nothing_of_a_game_that_has_not_opened_it`, which fails on the
  old condition.
- Tests: 26 new, 106 total green, build clean at 19 warnings (unchanged). Ten are
  pure - the whole token alphabet, the trigger with no digit, the evader speed the
  format drops, `set_cell` writing / keeping a hand-written shape / squaring up a
  ragged one / refusing to go outside, a `FilledGrid` spelled out as tokens, and
  the block colour and texture. The other sixteen run a real app, and thirteen of
  those drive the mouse end to end through the editor's camera and window: a
  click, a trigger brush, an obstacle, both erase paths, the same-frame
  appearance, a drag, the one-edit stroke, a cell crossed twice in one drag, a
  pointer held down off the grid, the round trip back through `interpret_grid`, a
  `FilledGrid` and a `Custom` level. The last three are the brush's starting
  state, the level's blocks being on screen every time the editor is opened, and
  the run condition above.
- Checked in the running game too, driven by a temporary system that aimed a
  synthetic pointer through the editor camera and then removed itself: the
  editor opens on `level0` with all 8 of its blocks wearing the game's mesh and
  material, and a mouse-driven paint through the real input path adds a ninth,
  dressed the same way, and one `painted N cell(s)` line per stroke rather than
  per cell. **Not** visually
  confirmed: Bevy's own screenshot comes back solid black from this session, as
  `screencapture` does - so how it looks on screen rests on the material being
  the game's own and on `c0006`'s screenshot of the same camera.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-26 status → in-progress (agent)
- 2026-08-26 `block_token` / `set_cell` / `filled_grid` in the layout module,
  `Brush` + `PaintStroke` + `editor_paint` on the mouse, `EditorBlock` putting
  the level on screen for the first time, `block_material` shared out of
  `block_spawn`; 26 new tests, 106 green, build clean
