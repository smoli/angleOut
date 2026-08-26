---
id: c0010
title: Level settings panel
status: in-progress
epic: e01
depends: [c0005]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T01:52:27
---

## What

Edit the parts of a level that are not the grid: everything in `LevelDefinition`
except `targets` and the out-of-scope `obstacles`.

## Acceptance criteria

- [x] The panel edits background asset, background scroll velocity, simultaneous balls, win criteria, global pickups, `default_wall_l` and `default_wall_r`.
- [x] Changes apply to the in-memory level immediately and are included when it is saved.
- [x] Every edited value round-trips through RON unchanged.
- [x] Invalid or out-of-range input is rejected without crashing.

## Notes

- `time_limit` is an `Option<Duration>` that nothing currently reads; include it
  only if it is cheap.
- `global_pickups` is a `Vec<PickupType>` where `PickupType::Grabber` is never
  constructed today — the panel is the natural place to make it reachable.
- Free-floating `LevelObstacle`s are explicitly out of scope for this epic.

## How it came out

`src/editor/settings.rs` - nine rows in a panel top left, each a label, the
value, and a `<` and a `>` that walk it. `src/editor/mod.rs` gained the two
systems that drive it (`editor_settings_click`, `editor_show_settings`) and one
guard in `cell_under_cursor`.

**Every setting is a stepper.** There is no text entry anywhere in this game to
type a number into, and a stepper needs none - it also means no setting ever
holds an invalid value, because the step is what keeps it in range. `time_limit`
was cheap on those terms, so it is in: below the shortest limit is `None`.

**Ranges, and why they are where they are.** Scroll velocity stops at 0 because
`arena_update_background` only wraps a segment that has run off the near end, so
a negative velocity sends the backdrop away and never brings it back. Balls at
once starts at 1 because `MatchEvent::BallSpawn` only adds one while
`balls_in_play` is under it, so 0 is a level that can never be launched. The win
percentage covers the whole 0-100%: it is the fraction of blocks hit rather than
lost, so 0% is "clear the level, however badly" and is meaningful.

**Backgrounds are a named list, not free text.** `BACKGROUNDS` holds the four
scenes the levels use. Offering every scene in `ship3_003.glb` would offer the
ball and the paddle as backdrops; free text would need somewhere to type it. A
level naming a background the panel does not know is still *shown* - stepping it
joins the list at the end it was stepped from.

**The pickup rows count pickups, not amounts.** `Extra balls` and `Grabbers` are
the number of each kind in `global_pickups`; the panel adds `MoreBalls(1)` and
`Grabber(1)` and takes back the last of that kind. A level that hand-authored a
`MoreBalls(3)` keeps it and gains and loses whole pickups either side of it. The
amounts are read nowhere in the game today. This is where `Grabber` becomes
reachable, as the card asked.

**The panel hit-tests its own rectangles** rather than going through
`Interaction`, for the reason `c0009` worked out: Bevy 0.19's `ui_focus_system`
needs the whole UI stack, which a headless test app cannot stand up. One
`settings_rows()` is both what is drawn and what a click is read against, so the
two cannot drift - the same trade the grid already makes with `world_to_cell`.
A click on the panel is also *not* a click on the play field: `cell_under_cursor`
hands back no cell for a pointer inside `panel_rect()`, so the highlight goes
too rather than a covered cell sitting there looking armed.

**Not verified on screen.** The panel is covered by 25 tests, including through
a real window and a real pointer, but every screenshot taken from this session
comes back black - in the editor and in the menu alike - so how it *looks* is
unconfirmed. Worth a glance when you next run it.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-26 status → in-progress (agent)
- 2026-08-26 settings panel built: `src/editor/settings.rs`, wired into the
  editor's schedule; 25 new tests, `cargo test` 157 passing, build clean
