---
id: c0002
title: Shared cell/world conversion
status: in-progress
epic: e01
depends: []
created: 2026-08-25
updated: 2026-08-25
status-changed: 2026-08-25T23:22:07
---

## What

The maths that turns a grid cell into a world position is implemented twice, both
in `src/level/layout/mod.rs`: once in `generate_block_grid` (for `FilledGrid`) and
again in `interpret_grid` (for `SparseGrid`). Both compute the same origin
(`-30.0 - 4.0 * (BLOCK_DEPTH + gap)`), the same steps (`BLOCK_WIDTH + gap`,
`BLOCK_DEPTH + gap`) and the same odd/even column centring.

The editor needs a third use — turning a mouse ray hit back into a cell — and a
third copy is exactly how the editor ends up disagreeing with the game about
where a block is.

Replace both with one shared pair, `cell_to_world` and its inverse
`world_to_cell`, and make the two existing callers use it. This is a pure
refactor: every shipped level must produce byte-identical block positions.

## Acceptance criteria

- [ ] A single `cell_to_world(col, row, cols, gap)` is the only place a block's world position is derived from its grid coordinates.
- [ ] A matching `world_to_cell(pos, cols, gap)` inverts it and returns `None` outside the grid.
- [ ] `generate_block_grid` and `interpret_grid` both call it; neither computes an origin, a step or a centring offset of its own.
- [ ] Round-trip unit test: `world_to_cell(cell_to_world(c, r)) == Some((c, r))` for every cell of both an odd-column and an even-column grid.
- [ ] Block positions for the existing levels are unchanged.
- [ ] `cargo test` passes and there is no visible change in game.

## Notes

- The odd/even centring differs and must be preserved exactly:
  `if cols % 2 == 1 { x -= cols_h * x_step } else { x -= cols_h * x_step - gap / 2.0 - BLOCK_WIDTH_H }`,
  where `cols_h = (cols / 2) as f32` is integer division before the cast.
- `interpret_grid` derives its column count from the first line only. That quirk
  stays for now; step 7 (`c0008`) makes the writer pad rows so it stays correct.
- Deliberately no behaviour change — this card exists to make step 5 (`c0006`) safe.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-25 status → in-progress (agent)
