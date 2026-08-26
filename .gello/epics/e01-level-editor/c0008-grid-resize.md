---
id: c0008
title: Grid resize
status: done
epic: e01
depends: [c0007]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T21:55:20
usage-tokens: 87771
usage-cost: 8.914777
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

## Review

### 2026-08-26T01:47:30 — pass

Checked: the six acceptance criteria against `src/level/layout/mod.rs` and
`src/editor/mod.rs`, the diff of `7dd5600`, `cargo test`, `cargo build`,
`cargo clippy --all-targets`.

- Rows and columns at all four edges: `grow`/`shrink` in the layout module,
  reached from `resize_asked_for` (arrow / `Shift`+arrow) in `editor_resize`.
  Covered pure by `a_row_can_be_added_at_the_top_and_at_the_bottom`,
  `a_column_can_be_added_at_the_left_and_at_the_right` and their two shrink
  twins, and through the app by
  `an_arrow_key_adds_a_row_or_a_column_at_the_edge_it_points_at` and
  `shift_and_an_arrow_key_takes_that_edge_away_again`.
- `Edge::Top` really is the top of the screen, so the arrows point where the
  card says they do: `interpret_grid` maps layout line 0 to `cell_to_world` row
  0, which is the most negative z, and the editor camera at
  `(0, EDITOR_CAMERA_HEIGHT, 0.00001)` with `Vec3::Y` up puts -z at the top of
  the frame.
- "Warns first or is undoable" is met by the warning half: `take_edge_away`
  refuses the first press on an edge with blocks on it, spawns the orange
  `EditorWarning` text and outlines the doomed cells, and acts on the second.
  `an_edge_with_blocks_on_it_is_called_out_before_it_is_taken_away`,
  `an_empty_edge_needs_no_warning`,
  `a_warning_does_not_survive_the_author_doing_something_else`,
  `growing_the_grid_drops_a_warning_rather_than_confirming_it` and
  `leaving_the_editor_forgets_the_warning` cover it. `blocks_on_edge`'s count is
  checked against the blocks that actually disappear by
  `the_count_warned_about_is_the_blocks_that_are_lost`, and
  `slot_holds_a_block` agrees with `make_block`'s reading of `.` in either of
  the first two characters.
- Retained cells keep their tokens: `the_cells_that_are_kept_hold_what_they_held`
  and `growing_an_edge_and_taking_it_away_again_is_the_grid_it_was` on the
  layout, `the_blocks_that_are_kept_keep_their_cells` on the blocks on screen.
  The whole grid moving a cell towards the paddle when a row is added at the top
  follows from row 0's fixed world position and is disclosed in the Notes; the
  criterion is about the cells, and it holds.
- Padding: `slot_grid` squares every row up to the widest and `write_grid` is
  the only way back out, so `grow`, `shrink` and `set_cell` cannot write a
  ragged grid. Every slot it writes is >= 2 characters, which is what
  `interpret_grid`'s unfiltered first-line `split(" ")` count needs.
  `a_resized_grid_is_padded_out_to_one_width` covers it.
- Round trip: `a_resized_level_saves_and_reloads_and_plays` writes the resized
  level with `level_to_ron`, reads it back with `load_level`, and compares
  `interpret_grid`'s blocks against the editor's own `EditorBlock` transforms -
  three blocks in a 4x3 grid, so the comparison is not vacuous.
- Checks green: `cargo test` 132 passed, 0 failed - the claimed count, and the
  26 new tests are all present and none `#[ignore]`d. `cargo build` clean at 19
  warnings, none of them in the two files this card touched. `cargo clippy
  --all-targets` exits 0; its only new lint is `needless_lifetimes` on the test
  helper `slot<'a>` at `src/level/layout/mod.rs:1026`, and clippy is not a check
  the repo documents.
- Diff stays inside the What: two files, no test removed or weakened - the only
  deletion in an existing test is `editor::tests::every_shipped_level_fits...`
  reworded onto the new `grid_fits_the_view`, which asserts the same thing.
  Working tree note: many other `src/*` files are modified and `src/diagnostics/`
  is untracked, but none of that is this card's - `git diff 7dd5600 -- src/editor
  src/level` is empty, so the checks above ran on exactly the committed code.

Two things for later, neither blocking:

- The warning's block count can go stale mid-drag. `editor_paint` clears
  `PendingRemoval` only where the stroke *starts*, so an author holding the
  mouse down, pressing `Shift+Up`, painting two more cells into the top row
  while still holding, and pressing `Shift+Up` again removes four blocks having
  been told two. The criterion is still met - the press did warn first - and
  `c0011`'s undo makes it moot, but the Notes claim painting always drops a
  warning and in this one path it does not.
- Not verified here: that the orange text and the doomed-edge outline actually
  render. The tests assert the entities and their strings, and screenshots were
  black in the implementer's session too.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-26 status → in-progress (agent)
- 2026-08-26 `Edge` / `grow` / `shrink` / `blocks_on_edge` in the layout module
  with `set_cell` refactored onto the shared `slot_grid`, arrow-key resizing in
  the editor with a warning before an edge with blocks on it goes; 26 new tests,
  132 green, build clean
- 2026-08-26 status → review (agent)
- 2026-08-26 status → done (app)
