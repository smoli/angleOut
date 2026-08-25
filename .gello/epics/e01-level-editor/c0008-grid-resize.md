---
id: c0008
title: Grid resize
status: in-progress
epic: e01
depends: [c0007]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T01:24:12
---

## What

Let the grid grow and shrink. The shipped levels run from 3 to 11 columns wide,
so a fixed extent could not express them — resizing is not optional.

Rows can be added or removed at top and bottom, columns at left and right.

## Acceptance criteria

- [x] Rows can be added and removed at the top and at the bottom.
- [x] Columns can be added and removed at the left and at the right.
- [x] Removing a row or column containing blocks either warns first or is undoable.
- [x] Existing blocks keep their position relative to the cells that are retained.
- [x] All rows are padded to equal width when written, so `interpret_grid`'s first-line column count stays correct for ragged edits.
- [x] A resized level saves, reloads and plays correctly.

## Notes

- `interpret_grid` takes its column count from the first line alone, so a ragged
  grid silently misaligns every row after it. The padding requirement above is
  what prevents that.
- Grid extent is not stored explicitly today — it is implied by the token string.
  Resizing therefore only has to produce a well-formed string.

**As built** - `src/level/layout/mod.rs` (the four sides of a grid),
`src/editor/mod.rs` (the keys, and the warning before a row goes).

- An arrow key adds a row or a column at the edge it points at; `Shift` and an
  arrow takes that edge away again. Arrow keys were free, they name the four
  edges without a legend, and the editor has no UI yet to click - `c0009` and
  `c0010` build that, and the shortcuts are what they will be labelled with. The
  key map is logged on the way into the editor, since there is nowhere on screen
  to put it yet.
- `Edge::Top` is row 0 - the first line of the layout - because that is what an
  author sees at the top of the screen, under the editor's straight-down camera
  and the game's tilted one alike. `cell_to_world` calls row 0 the *bottom* row,
  which is the same row named from the other side: it works in world
  coordinates, where z grows towards the player. The enum says so out loud,
  because getting it backwards would put every new row at the wrong end.
- Row 0's world position is fixed - `-30 - 4 * (BLOCK_DEPTH + gap)` - so the grid
  always grows *towards the paddle*, whichever end a row is added at. Adding one
  at the top pushes the level down the screen by a cell; adding one at the bottom
  extends it. What the card asks for is what holds either way: every cell that is
  kept keeps its token, and the block that was in it is in it still.
- Every resize writes the whole grid out, which is where the padding comes from.
  `slot_grid` reads a layout as rows of slots squared up to the widest one and
  `write_grid` writes them back; `grow`, `shrink` and `set_cell` are all three
  lines on top of that pair, so there is one place a ragged grid is squared up
  rather than one per operation.
- **The warning is this card's half of "warns first or is undoable"** - `c0011`
  has the undo, and depends on this card rather than the other way round. An edge
  with nothing standing on it costs nothing and goes on the first press. An edge
  with blocks on it does not: the first press names the row and counts what is on
  it, in orange at the bottom of the screen and in the log, and outlines the
  doomed cells on the grid so the warning points at something rather than
  describing it. The same press again means it.
- A warning is about the level as it stood when it was given, so anything else
  the author does drops it - painting, growing, asking after a different edge,
  leaving the editor. Otherwise a press made a minute and an edit later would be
  taken as confirming a count that is no longer true.
- A grid keeps its last row and its last column: shrunk away to nothing there
  would be nothing left on screen to aim at and no cell to grow back from. The
  other end is the frame the editor promises to keep the grid inside, which for
  the shipped gap is 13 x 16 - the widest shipped level is 11. Whether every cell
  of a grid that wide is somewhere a ball can actually reach is a different
  question, and `c0012`'s to warn about; the editor only refuses what it cannot
  draw.
- Growing or shrinking a `FilledGrid` spreads it into tokens for the same reason
  painting one does: a grid with an empty row on it is no longer "the same block
  everywhere". A resize that is refused spreads nothing - the fit and the floor
  are both checked before the level is touched.
- The resize goes in through `bypass_change_detection`, as painting does, so a
  press that is refused - or one that only warns - does not respawn every block
  on screen. `editor_resize` sits in the same chain as the paint, so a grown grid
  is on screen, hoverable and paintable in the frame the key was pressed.
- Tests: 26 new, 132 total green, build clean at 19 warnings (unchanged). Eleven
  are pure - each edge grown and shrunk, a grow and a shrink of the same edge
  coming back to the grid it was, the cells that are kept holding what they held,
  a ragged layout squared up by a resize, the last row and column staying, a grid
  with no cells growing into one, and the block count an edge is warned about
  checked against the blocks that actually disappear. Fifteen run a real app: the
  four edges through the real key path, the pointer finding a cell that did not
  exist a moment ago, blocks keeping their cells on screen, the warning appearing
  with its count and the second press acting on it, an empty edge needing none, a
  warning dropped by a paint / by a grow / by a different edge / by leaving, the
  growth limit, the `FilledGrid`, a refused resize changing nothing, a `Custom`
  level having nothing to resize, and the round trip through disk.
- The last one is the card's "saves, reloads and plays": saving is `c0012`'s, so
  the test makes the trip a save will make - the resized level written as RON with
  `level_to_ron`, read back with `load_level`, and read out again through
  `interpret_grid` as the blocks a match would spawn, compared against the blocks
  the editor had on screen.
- Checked in the running game too, driven by a temporary system that pressed the
  keys through the real `ButtonInput` and then removed itself: the editor opens
  on `level0` at 9x6, `ArrowRight` makes it 10x6, `Shift+Up` leaves it at 10x6 and
  puts *The top row holds 3 blocks - press Shift+Up again to remove it* on screen
  - `level0`'s top row is `ZIR1`, `CA` and `ZAA1`, which is three - and the same
  press again makes it 10x5 with the warning gone. **Not** visually confirmed:
  screenshots come back black from this session, so the orange warning text and
  the outline on the doomed cells rest on the entities being there with the
  game's own font and on `c0006`'s screenshot of the same camera.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-26 status → in-progress (agent)
- 2026-08-26 `Edge` / `grow` / `shrink` / `blocks_on_edge` in the layout module
  with `set_cell` refactored onto the shared `slot_grid`, arrow-key resizing in
  the editor with a warning before an edge with blocks on it goes; 26 new tests,
  132 green, build clean
