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

- [x] A single `cell_to_world(col, row, cols, gap)` is the only place a block's world position is derived from its grid coordinates.
- [x] A matching `world_to_cell(pos, cols, gap)` inverts it and returns `None` outside the grid.
- [x] `generate_block_grid` and `interpret_grid` both call it; neither computes an origin, a step or a centring offset of its own.
- [x] Round-trip unit test: `world_to_cell(cell_to_world(c, r)) == Some((c, r))` for every cell of both an odd-column and an even-column grid.
- [x] Block positions for the existing levels are unchanged.
- [x] `cargo test` passes and there is no visible change in game.

## Notes

- The odd/even centring differs and must be preserved exactly:
  `if cols % 2 == 1 { x -= cols_h * x_step } else { x -= cols_h * x_step - gap / 2.0 - BLOCK_WIDTH_H }`,
  where `cols_h = (cols / 2) as f32` is integer division before the cast.
- `interpret_grid` derives its column count from the first line only. That quirk
  stays for now; step 7 (`c0008`) makes the writer pad rows so it stays correct.
- Deliberately no behaviour change — this card exists to make step 5 (`c0006`) safe.
- `cell_to_world` steps the origin along instead of multiplying: `y0 + row * y_step`
  differs from the accumulation the old loops did by an ULP or two from row 2 up,
  so multiplying would have moved every shipped block by ~1e-5. Two tests pin the
  output against copies of the pre-refactor loops (a mutation check confirmed both
  fail if the stepping is replaced by a multiply).
- `world_to_cell` keeps the signature the criteria name, so it bounds columns by
  `cols` and rows below by row 0, but is open upwards — the caller owns the row
  count. `c0006` has `rows` to hand and can reject rows above the grid itself.
- A position is assigned to the cell whose centre is nearest, so the gap between
  two cells belongs to the nearer of them rather than being dead space.
- `world_to_cell` is dead code until `c0006` picks it up, so the build carries one
  more `never used` warning alongside the ~20 already there.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-25 status → in-progress (agent)
- 2026-08-25 added `cell_to_world` / `world_to_cell`; `generate_block_grid` and
  `interpret_grid` now derive every position from them and compute no origin,
  step or centring of their own
- 2026-08-25 tests: round trip over every cell of an 11- and a 10-column grid and
  over every column count 1..=12, nearest-centre snapping, out-of-grid rejection,
  and byte-identity against the old maths for filled grids (1..=12 cols × 0..=8
  rows) and for shipped-shaped layouts. `cargo test` 33 passed, `cargo build`
  clean, game starts without panic (reaching an actual match needs input this
  session cannot send, so unchanged positions rest on the identity tests)
