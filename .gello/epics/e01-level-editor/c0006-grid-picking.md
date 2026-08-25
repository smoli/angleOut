---
id: c0006
title: Grid picking
status: backlog
epic: e01
depends: [c0002, c0005]
created: 2026-08-25
updated: 2026-08-25
status-changed: 2026-08-25T22:58:50
---

## What

Work out which grid cell the mouse is over, and show it.

A ray is cast from the cursor through the camera and intersected with the y=0
ground plane, then quantised to a cell with `c0002`'s `world_to_cell`. Deliberately
not mesh picking: the editor has to be able to hover an *empty* cell, and there is
no geometry there to hit.

## Acceptance criteria

- [ ] The cursor position is converted to a world ray and intersected with the y=0 plane.
- [ ] The hit point is quantised to a grid cell using `world_to_cell`.
- [ ] Positions outside the grid produce no hovered cell.
- [ ] The hovered cell is visibly highlighted.
- [ ] Hovering works over empty cells, not just cells containing a block.
- [ ] Picking is correct with the tilted camera (`TILTED_CAMERA`, `CAMERA_TILT`).

## Notes

- `Camera::viewport_to_world` gives the ray; `Ray3d::intersect_plane` with an
  `InfinitePlane3d::new(Vec3::Y)` gives the hit distance.
- Blocks sit at y=0, so the ground plane is the right intersection target.

## Log

- 2026-08-25 created from the e01 epic breakdown
