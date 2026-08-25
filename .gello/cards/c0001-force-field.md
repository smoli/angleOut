---
id: c0001
title: Force field
status: ready
created: 2026-08-25
updated: 2026-08-25
status-changed: 2026-08-25T22:46:24
order: 10
---

## What

Replace the current back-wall force field shader with a blue energy shield that
reacts properly to ball impacts.

At rest the panel is a **smooth energy sheet** — a soft, near-transparent blue
plane with slow movement, and no hex lattice visible. On impact a **radial
ripple** expands outward from the contact point and fades, and the hex lattice
lights up only where that ripple passes — the wavefront runs white-hot and
grades back to blue behind it, and ripples simply fade as they travel rather
than reflecting off the panel edges. `hexagon2.png` is demoted from
permanent base texture to an impact-only flare mask: the shield's structure is
invisible until it is doing work, and a hit briefly exposes the honeycomb it is
made of.

Several impacts must be able to be alive at once, and this covers the rotated
`LevelObstacle::ForceField` panels as well as the top barrier — which means
fixing the impact-position mapping, not just the visuals.

## Acceptance criteria

- [ ] At rest the panel shows a smooth blue energy sheet with no hex lattice anywhere on it.
- [ ] A ball impact spawns a radial ripple that expands outward from the contact point and fades out.
- [ ] The hex lattice is visible only where a ripple is currently passing, and fades out with it.
- [ ] `ForceFieldMaterial` holds a pool of **eight** concurrent hits (position + start time per slot); a new hit claims a free or the oldest slot instead of overwriting a live one.
- [ ] Two balls striking the same panel within one ripple lifetime produce two independently visible, overlapping ripples.
- [ ] Impact position is derived from the panel's own transform and size, so a hit on a rotated `LevelObstacle::ForceField` ripples at the actual contact point rather than an arena-width approximation.
- [ ] The wavefront reads white-hot and grades back to blue behind it.
- [ ] Ripples fade as they travel and do not reflect off the panel edges.
- [ ] Every impact produces an identical ripple, regardless of ball speed or angle.
- [ ] The rotated-panel mapping fix has been verified by temporarily re-enabling the commented-out `LevelObstacle::ForceField` level in `main.rs`.
- [ ] Ripple speed, width, decay time, flare intensity and colour are fields on `ForceFieldMaterial`, not literals in the WGSL.
- [ ] Frame time with several ripples active shows no regression against the current build (FPS readout in the stats UI).
- [ ] `cargo build` is clean and `cargo test` still passes.
- [ ] Final look signed off in-game.

## Discussion

**Decisions**

- Idle look is a smooth energy sheet, not the current hex lattice. The lattice
  survives only as the impact flare.
- Impact is a travelling radial ripple *and* a hex flare, not one or the other —
  the ripple lights the cells it passes through.
- Multi-hit via a small fixed pool (~4–8 slots) in the uniform; the fragment
  shader sums each live slot's contribution.
- Scope includes the rotated obstacle panels, which pulls the impact-position
  bug into this card deliberately.
- Ripple parameters are exposed as material fields so they can be tuned from one
  place (and later driven by gameplay or an inspector).
- Done = mechanical guardrails above, plus a visual sign-off.
- Pool holds **eight** hits. `MoreBalls` pickups stack, so the real ceiling is not
  bounded by a level's `simultaneous_balls`; eight `vec4`s and a fixed loop is
  cheap enough to size once and forget.
- Ripples fade as they travel — no edge reflection. The panel's existing bright
  edge falloff already gives the wave somewhere to wash into.
- The flare is white-hot at the wavefront, grading back to blue behind it, for
  maximum contrast against the smooth blue sheet.
- Every hit ripples identically. Ball speed is clamped to a constant
  (`MIN_BALL_SPEED == MAX_BALL_SPEED == 130`), so speed-scaling would be
  invisible; angle-scaling was considered and dropped to keep the read consistent.
- The rotated-panel fix is verified by temporarily re-enabling the commented-out
  level in `main.rs` and then commenting it back out.

**Rejected**

- Keeping the hex lattice as the permanent base texture — that is the look being
  replaced.
- A "barely there until touched" shield — would weaken how readable the arena
  bounds are during play.
- Local bloom only, or a hex flare with no travelling wave — neither reads as a
  shield hit.
- An unbounded hit list in a storage buffer — bind-group churn not worth it over
  a fixed pool.
- Keeping the single hit slot — visibly wrong as soon as a `MoreBalls` pickup
  puts two or more balls in play.
- Tuning purely through WGSL constants (hot-reload) — rejected in favour of
  material fields.
- A dedicated test level or debug toggle for firing hits at known positions.
- Edge reflection, and a rim flare as the wave reaches the panel bounds.
- Keeping the flare in the blue family, or shifting it to cyan.
- Scaling ripple strength by impact speed or incidence angle.
- A four-slot pool (could silently drop hits once pickups stack) or sixteen.
- Permanently re-enabling the commented-out level, or adding rotated panels to a
  level that already ships.

**Implementation notes**

- `arena_update_force_field_material` currently pushes `time` into every material
  every frame. `bevy_pbr::mesh_view_bindings::globals` already exposes
  `globals.time` at `@group(0) @binding(11)`, so that system can be deleted
  outright.
- If the shader moves to `globals.time`, hit times must be recorded with
  `Time::elapsed_secs_wrapped()`, **not** `elapsed_secs()`. `globals.time` is the
  wrapped clock (wraps hourly); mixing the two silently breaks every ripple after
  an hour of uptime.
- The mapping to replace lives in `arena_handle_collisions`:
  `(collision.other_pos.x + ARENA_WIDTH_H) / ARENA_WIDTH`.
- The impact's vertical position is currently discarded — `hit_position` uses only
  `.x`. Centring a ripple on the true contact height needs it.

**Open questions**

None — all resolved in discussion.

**Origin**

> There’s a shader right now trying to mimic a force field for the back wall. It looks like ass because I suck at shaders.
>
> Let’s turn it into a blue shimmering force field that has proper ripple effects when hit by the balls

## Notes

**How it is built**

- `ForceFieldMaterial` carries a fixed pool of `FORCE_FIELD_HIT_SLOTS = 8` hits
  as `[Vec4; 8]` — `xy` is the panel uv, `z` the start time, `w` marks the slot
  used. `register_hit` takes the first slot that is unused or already decayed and
  only evicts the oldest live ripple when all eight are busy.
- `panel_uv` inverts the panel's own `GlobalTransform` and divides by its size,
  so rotated obstacle panels map correctly and the contact height is kept. The
  `ForceField` component now carries `size` for this.
- The shader sums a gaussian ring per live slot in world units (`uv * panel_size`),
  so ripples stay round on a 200x20 panel and overlap additively. The hex texture
  is multiplied by that sum, which is what makes the lattice exist only where a
  ripple is passing.
- `arena_update_force_field_material` is gone; the shader reads `globals.time` and
  hits are stamped with `Time::elapsed_secs_wrapped()` to match. `wrap_period_matches_bevy`
  fails loudly if Bevy ever changes the hourly wrap period out from under us.
- `hexagon2.png` is loaded with a repeating sampler now that it is tiled across
  the panel in world units rather than sampled in uv with a manual `% 1.0`.

**Open questions, answered while building** — all five are cheap to change, and
four of them are now material fields, so treat these as starting points for the
in-game tuning pass rather than settled:

- *Pool size*: 8. Levels top out at four balls (`simultaneous_balls: 1` plus at
  most three `MoreBalls` pickups), and eight slots leave room for a ball that
  rattles along the same panel twice.
- *Edges*: ripples fade at the panel edge, they do not reflect.
- *Flare colour*: a hot pale blue (`flare_color`, default `srgb(0.65, 0.9, 1.0)`)
  rather than white-hot, so the shield still reads as blue when it is hit.
- *Impact speed modulating ripple strength*: not done — it is not in the
  acceptance criteria, and it would need a per-slot strength in the uniform.
- *Verifying the rotated panels*: done with unit tests over `panel_uv` rather
  than re-enabling the commented-out level, since no test level was wanted.
  `a_rotated_panel_maps_along_its_own_axis` pins the exact case the old
  arena-width mapping got wrong.

**Left for the human**

- The two visual criteria: the look itself, and the FPS readout with several
  ripples alive. The shader should be cheaper than the one it replaces (two
  `noise()` calls instead of the five the voronoi did, against eight cheap loop
  iterations), but that wants confirming on the readout.
- Tuning lives on `ForceFieldMaterial`: `ripple_speed` 70.0, `ripple_width` 6.0,
  `ripple_decay` 1.2, `flare_intensity` 2.5, `hex_tile_size` 10.0, plus
  `sheet_color` / `flare_color`.

## Log

- 2026-08-25 status → discuss (app)
- 2026-08-25 status → ready (app)
- 2026-08-25 status → in-progress (agent)
- 2026-08-25 hit pool, panel-relative impact mapping and the new sheet/ripple/flare shader landed; 26 tests green (agent)
- 2026-08-25 status → ready (app)
