---
id: c0006
title: Grid picking
status: done
epic: e01
depends: [c0002, c0005]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T21:55:17
usage-tokens: 57942
usage-cost: 6.292097
---

## What

Work out which grid cell the mouse is over, and show it.

A ray is cast from the cursor through the camera and intersected with the y=0
ground plane, then quantised to a cell with `c0002`'s `world_to_cell`. Deliberately
not mesh picking: the editor has to be able to hover an *empty* cell, and there is
no geometry there to hit.

## Acceptance criteria

- [x] The cursor position is converted to a world ray and intersected with the y=0 plane.
- [x] The hit point is quantised to a grid cell using `world_to_cell`.
- [x] Positions outside the grid produce no hovered cell.
- [x] The hovered cell is visibly highlighted.
- [x] Hovering works over empty cells, not just cells containing a block.
- [x] Picking is correct with the tilted camera (`TILTED_CAMERA`, `CAMERA_TILT`).

## Notes

- `Camera::viewport_to_world` gives the ray; `Ray3d::intersect_plane` with an
  `InfinitePlane3d::new(Vec3::Y)` gives the hit distance.
- Blocks sit at y=0, so the ground plane is the right intersection target.

**As built** - all in `src/editor/mod.rs`.

- The chain is split in two. `cell_under_ray(ray, cols, rows, gap)` is pure: plane
  hit, `world_to_cell`, row bound. `editor_pick_cell` is the plumbing that gets a
  ray - primary window's `cursor_position`, camera marked `EditorCamera`,
  `viewport_to_world` - and writes `HoveredCell`. Every step of the plumbing can
  legitimately come up empty (pointer outside the window, camera a frame short of
  a viewport, a `Custom` level that is not a grid at all), so it is all `Option`
  and no cell is a normal answer rather than an error.
- `world_to_cell` is open upwards by `c0002`'s design - the caller owns the row
  count. `cell_under_ray` is that caller, so the `row < rows` bound lives there.
- `HoveredCell` is a resource, written only when the cell actually changes, so
  change detection on it means "the pointer entered a different cell" - the signal
  `c0007` wants. It is cleared in `editor_teardown`: the camera it was picked
  through leaves with the editor, and a stale hover would be a paint on the wrong
  cell on the way back in.
- The camera picking reads is marked `EditorCamera` rather than taken as "the"
  camera. `c0013` will have the game's camera and the editor's alive in the same
  run, so which one the pointer is read through has to be said out loud.

**On the tilted-camera criterion.** The criterion was written before `c0005`
settled on an orthographic straight-down editor camera, so `TILTED_CAMERA` does
not apply to the camera the editor actually picks through today. It is still a
real constraint on the code, and the code meets it: `cell_under_ray` knows
nothing about where its ray came from. Two tests pin that down - one feeding it
rays from the exact position `setup_3d_environment` computes
(`Quat::from_rotation_x(CAMERA_TILT) * Vec3::new(0.0, 200.0, 0.00001)` when
`TILTED_CAMERA`), and one swapping that camera - perspective and all - into a
running editor and round-tripping every cell of a 9x6 grid through a real
viewport.

- Tests: 11 new, 80 total green. Four are pure (`cell_under_ray` over every cell
  of odd- and even-column grids, anywhere-inside-a-cell, off-the-grid on all four
  sides including the row bound, and rays that never meet the ground), five go end
  to end through a real `Camera` and `Window` (every cell of a 9x6 grid, empty
  cells specifically, off-grid and off-window, the tilted camera, and the hover
  cleared on the way out), and two cover the highlight's placement.
- The headless tests do by hand what `camera_system` does in `PostUpdate`: set
  `computed.target_info` and `computed.clip_from_view` from the projection.
  Without it `viewport_to_world` has no viewport to read. `TransformPlugin` had
  to join the test app too - the camera's `GlobalTransform` is the identity until
  propagation runs.
- Verified in the running game as well as in tests: a screenshot with the pointer
  parked on the pixel cell (4, 3) shows up at has the yellow outline on the middle
  column, fourth row down - the cell that was aimed at. The highlight is the
  block's own footprint (`BLOCK_WIDTH` x `BLOCK_DEPTH`), so it sits inside the
  cell's gridline box, inset by the gap: it shows the block that would go there.

## Review

### 2026-08-26T00:55:44 — pass

Checked: all six acceptance criteria against `src/editor/mod.rs` and
`src/level/layout/mod.rs`, the commit diff (`e00a2dc`), `cargo build` and
`cargo test`.

- "Cursor to world ray, intersected with y=0": `cell_under_ray` uses
  `Ray3d::intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))`, and
  `cell_under_cursor` builds the ray from the primary window's
  `cursor_position()` through the `EditorCamera`'s `viewport_to_world`. Rays
  that never meet the plane come back as no cell
  (`a_ray_that_misses_the_ground_finds_no_cell`).
- "Quantised with `world_to_cell`": done on `Vec2::new(hit.x, hit.z)`, which is
  the pair `cell_to_world` returns - no second copy of the centring maths.
  `a_ray_aimed_at_a_cell_finds_that_cell` round-trips every cell of 9x6, 10x4,
  1x1 and 11x8 grids, so both the odd- and even-column centring are covered,
  and `anywhere_inside_a_cell_is_that_cell` pins the off-centre case.
- "Outside the grid, no cell": `world_to_cell` bounds col and row 0 from below,
  and the `row < rows` bound is added in `cell_under_ray` - correct, since
  `c0002` deliberately left the row count to the caller.
  `a_ray_that_lands_off_the_grid_finds_no_cell` covers all four sides including
  the open top, and `the_pointer_hovers_nothing_when_it_is_not_over_a_cell`
  covers it end to end plus the pointer leaving the window.
- "Visibly highlighted": `editor_draw_hover` outlines
  `BLOCK_WIDTH` x `BLOCK_DEPTH` at the cell centre in `YELLOW`, chained after
  `editor_pick_cell` so it cannot trail a frame, and lifted `HOVER_LIFT` off the
  grid gizmo. `the_highlight_sits_on_the_hovered_cell` pins the placement for
  every cell. The gizmo actually reaching the screen is not test-covered (no
  renderer headless, same as `editor_draw_grid` from `c0005`); I did not re-run
  the game, so that half rests on code inspection and the screenshot recorded in
  the notes.
- "Empty cells hover too": no mesh or `bevy_picking` involvement anywhere in the
  chain, and `an_empty_cell_hovers_just_like_a_full_one` hovers the six empty
  cells of a grid that has blocks in the other three.
- "Correct with the tilted camera": the reasoning in the notes holds -
  `cell_under_ray` takes a `Ray3d` and knows nothing about its origin. Both
  tests are real rather than nominal: `picking_is_correct_from_the_tilted_camera`
  rebuilds `setup_3d_environment`'s position exactly (`src/match/mod.rs:99-103`)
  and asserts `camera.z > 1.0`, so it fails loudly rather than passing vacuously
  if `TILTED_CAMERA` is ever turned off; `picking_is_correct_through_the_tilted_camera`
  swaps the perspective tilted camera into a running editor and asserts all 54
  cells of a 9x6 grid were checked.
- Diff is `src/editor/mod.rs` and this card, nothing else. Additive only: no
  existing test changed except an import moved to the module's `use` block, and
  nothing is ignored or weakened.
- `cargo test`: 80 passed, 0 failed. `cargo build`: 19 warnings, none from
  `src/editor/mod.rs` - matches the log. No lint or typecheck step exists in this
  repo (README lists `cargo build` only, no clippy config, no CI workflow), so
  there was none to run.
- Nit, no action needed: the notes' test breakdown ("four are pure ... two cover
  the highlight's placement") splits 11 tests wrongly - it is five pure, five end
  to end and one highlight test.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-26 status → in-progress (agent)
- 2026-08-26 `cell_under_ray` + `editor_pick_cell` + `HoveredCell`, highlight drawn
  with a gizmo, camera marked `EditorCamera`; 11 new tests, 80 green, build clean
  (19 warnings, one fewer than before - `world_to_cell` is no longer unused)
- 2026-08-26 status → review (agent)
- 2026-08-26 status → done (app)
