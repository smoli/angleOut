---
id: c0006
title: Grid picking
status: review
epic: e01
depends: [c0002, c0005]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T00:53:44
usage-tokens: 47860
usage-cost: 5.054555
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

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-26 status → in-progress (agent)
- 2026-08-26 `cell_under_ray` + `editor_pick_cell` + `HoveredCell`, highlight drawn
  with a gizmo, camera marked `EditorCamera`; 11 new tests, 80 green, build clean
  (19 warnings, one fewer than before - `world_to_cell` is no longer unused)
- 2026-08-26 status → review (agent)
