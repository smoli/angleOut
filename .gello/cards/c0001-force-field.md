---
id: c0001
title: Force field
status: discuss
created: 2026-08-25
updated: 2026-08-25
status-changed: 2026-08-25T22:15:18
---

## What

Replace the current back-wall force field shader with a blue energy shield that
reacts properly to ball impacts.

At rest the panel is a **smooth energy sheet** — a soft, near-transparent blue
plane with slow movement, and no hex lattice visible. On impact a **radial
ripple** expands outward from the contact point and fades, and the hex lattice
lights up only where that ripple passes. `hexagon2.png` is demoted from
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
- [ ] `ForceFieldMaterial` holds a fixed pool of concurrent hits (position + start time per slot); a new hit claims a free or the oldest slot instead of overwriting a live one.
- [ ] Two balls striking the same panel within one ripple lifetime produce two independently visible, overlapping ripples.
- [ ] Impact position is derived from the panel's own transform and size, so a hit on a rotated `LevelObstacle::ForceField` ripples at the actual contact point rather than an arena-width approximation.
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

- Pool size: 4 or 8? Depends on the highest `simultaneous_balls` worth supporting.
- Does a ripple reflect off the panel edges, or simply fade at them?
- Flare colour — white-hot, or a brighter blue than the sheet?
- Should impact speed modulate ripple strength?
- Rotated panels are only used by the commented-out level in `main.rs`. Re-enable
  it to verify the mapping fix, or verify some other way? (No test level was
  wanted.)

**Origin**

> There’s a shader right now trying to mimic a force field for the back wall. It looks like ass because I suck at shaders.
>
> Let’s turn it into a blue shimmering force field that has proper ripple effects when hit by the balls

## Log

- 2026-08-25 status → discuss (app)
