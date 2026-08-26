---
id: c0010
title: Level settings panel
status: review
epic: e01
depends: [c0005]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T02:10:55
usage-tokens: 67853
usage-cost: 7.05599
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

## Review

### 2026-08-26T02:14:04 — pass

Checked: the four acceptance criteria against `src/editor/settings.rs` and
`src/editor/mod.rs`, the diff of `d8d84af`, `cargo test`, `cargo check --tests`.

- Criterion 1 is met and then some: `SETTINGS` covers all nine fields the card
  names plus `time_limit`, which is everything in `LevelDefinition` bar
  `targets` and `obstacles`. `PickupType` has exactly the two variants
  `MoreBalls` and `Grabber`, so the two pickup rows cover the whole of
  `global_pickups`; `Setting::Grabbers` is the first constructor of
  `PickupType::Grabber` in the tree, as the notes asked.
  `every_setting_is_on_screen_saying_what_the_level_holds` and
  `a_setting_says_what_it_holds_the_way_an_author_reads_it` cover it.
- Criterion 2 is met. `editor_settings_click` writes straight into
  `EditorLevel` and `.chain()` in `EditorPlugin::build` puts it ahead of
  `editor_show_settings`, so the panel is redrawn in the same frame;
  `clicking_a_button_steps_its_setting_and_the_panel_says_so` asserts both the
  level and the on-screen text. "Included when it is saved" is verified as far
  as it can be — `c0012` has not built a save path yet, so the test writes
  `editor_level(&app).level` through `campaign::level_to_ron`, which is the
  serializer a save will use.
- Criterion 3 is met. `every_edited_value_round_trips_through_ron` steps all
  eight panel-owned fields off their defaults (asserting all eight moved) and
  round-trips; `every_rung_of_every_ladder_round_trips_through_ron` walks every
  rung of the three non-trivial ladders (scroll velocity, win percentage, time
  limit) through RON, which is the case where a float could quietly change.
  `a_level_edited_through_the_panel_round_trips_through_ron` does the same
  through real clicks.
- Criterion 4 is met. `stepped` snaps a non-finite or out-of-range value back
  onto the ladder before stepping, `step_pickups` clamps to `MAX_PICKUPS`, and
  `simultaneous_balls` uses `saturating_add` then `clamp`.
  `values_no_step_would_ever_have_made_are_shown_and_stepped_back_into_range`
  drives `NaN`, `-5` balls, a 500% criterion and a 7-second limit through
  `value()` and `step()` without a panic, and
  `stepping_a_setting_stops_at_the_ends_of_its_range` walks every setting 200
  rungs each way.
- Diff is in scope: `d8d84af` touches only `src/editor/mod.rs` and
  `src/editor/settings.rs`. The only edits to existing code are the
  `panel_rect()` guard in `cell_under_cursor`, two systems added to the chain,
  the `editor_setup` log line, and two test helpers generalised from the
  `VIEWPORT` constant to `window_size(app)` so `resize_the_window` works. No
  assertion removed, nothing ignored or skipped, no debug leftovers.
- `cargo test`: 157 passed, 0 failed — 19 of them under `editor::settings` and
  `editor::tests::the_settings_panel_is_up_...`, 25 new in total, which matches
  the log. `cargo check --tests`: clean; all 17 warnings are pre-existing
  (`src/block/mod.rs`, `src/materials`, `src/powerups`), none from the new
  files. There is no lint or typecheck step in this repo beyond `cargo check`.
- Not verified: how the panel looks on screen. I did not run the game, so the
  card's "not verified on screen" stands. Nothing in the criteria turns on it.
- One thing for the record rather than a fault: a press that *starts* on a
  panel button and is then dragged off onto the field does start a paint
  stroke, because `editor_paint` opens the stroke from `brush_in_hand` (the
  button being held) and only the cells under the panel are filtered out by
  `hovered`. A stepper is clicked, not dragged, so this is harmless — but the
  panel does not swallow the whole gesture, only the part of it over the panel.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-26 status → in-progress (agent)
- 2026-08-26 settings panel built: `src/editor/settings.rs`, wired into the
  editor's schedule; 25 new tests, `cargo test` 157 passing, build clean
- 2026-08-26 status → review (agent)
