---
id: c0013
title: Playtest round trip
status: done
epic: e01
depends: [c0007, c0010]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T21:55:25
usage-tokens: 104951
usage-cost: 14.25368
---

## What

Play the level you are editing and come back to it, with unsaved edits intact.
This is what makes the editor worth using rather than a thing you save from and
restart around.

## Acceptance criteria

- [x] A key or button starts a match on the level currently being edited.
- [x] The match uses the in-memory level including unsaved edits, not the file on disk.
- [x] Returning goes back to the editor with every unsaved edit still present.
- [x] The edited level survives `match_despawn` and the `OnExit(GameState::PostMatch)` teardown.
- [x] Returning leaves no match entities behind (ship, balls, blocks, arena, pickups, points).
- [x] Playtesting does not advance `Levels::current_level` or otherwise disturb campaign progress.

## Notes

- The teardown is aggressive: `OnExit(PostMatch)` despawns arena, ship, blocks and
  pickups, and `match_despawn` runs on exiting `InMatch`. The edited level must
  therefore live in a resource (`c0005`), never in entities.
- Returning from a match currently routes through `GameFlowEvent` into
  `PostMatch`; the editor needs its own return path that does not fall into the
  normal win/lose flow.

### The shape it took

- **What a match plays is one question, asked in one place.** `Levels` grows a
  `playtest: Option<Handle<LevelAsset>>`, and `current_handle` prefers it over
  the campaign. That is the whole mechanism: the arena, the blocks, the win
  criteria and the readiness check all go through `current_handle`, so pointing
  it somewhere else points all of them at once. The campaign underneath -
  `handles` and `current_level` - is never written to, which is the card's last
  criterion by construction rather than by care.
- **The level a playtest plays is a copy.** `Assets::add` of a clone of the
  level under edit, dropped again when the playtest ends. Unsaved edits are
  played because nothing on the way reads the file - and an author who paints
  while a ball is in the air is editing the *next* playtest, not this one, which
  is the only version of that that can be reasoned about.
- **Winning and losing have two ends now.** A playtest that ends, however it
  ends, goes back to `GameState::Editor`; a campaign match still goes to
  `PostMatch` and banks its points. `PlayerState` is set either way, and that is
  how the editor knows whether the level was won, lost or walked out of - which
  it says in the panel, because "your level cannot be won" is worth an author's
  time.
- **The editor clears the stage on the way in.** The rest of the game takes a
  match apart at `OnExit(PostMatch)` - deliberately, since the results screen is
  shown over the match that produced it - and a playtest never goes there. So
  `playtest_teardown` names everything a match leaves on the table in one place
  and despawns it as the editor comes up. The two exceptions are the `Match`
  marker and the particle effects, which go at `OnExit(InMatch)`, which a
  playtest does pass through. The cost of naming them in one place is that a new
  kind of match entity has to be added to that list; the test enumerates the
  kinds, so it fails rather than rots quietly.
- **`Escape` is the way out, one level down.** It already means "I am done here"
  in the editor, so it means the same in a playtest - which needs `close_on_esc`
  to stand down for the length of one, exactly as `c0005` had it stand down for
  the editor. `F5` and a `Play` button in a fourth panel are the way in.
- **A playtest deals a fresh hand.** Three balls and no points, every time, the
  way the menu deals them - otherwise the second playtest starts on the empty
  hand the first one ended with and is lost before it begins. Nothing about the
  campaign is disturbed by that: the editor is only reachable from the menu, and
  the menu already resets the player and the level.
- **The menu is a clean slate.** `game_start` now stops any playtest it finds.
  Nothing should be able to get from a playtest to the menu without passing
  through the editor, and that is exactly why the one line is there: a playtest
  handle left behind would shadow the campaign for the rest of the session.

### Worth knowing

**The report from the last save does not survive a playtest**, because
`editor_teardown` clears it every time the editor is left and a playtest leaves
the editor. That is right - it is a report about a file, and a whole match has
happened since - but it means the file panel is back to its shortest on the way
home, and the playtest panel under it moves back up with it. How the *playtest*
went is kept, and is what the panel says instead.

## Review

### 2026-08-26T03:33:03 — pass

Checked: the six acceptance criteria against the code, the commit's diff
(`b180f57`), `cargo test` and `cargo build`. `cargo clippy` run as an extra,
though the repo declares no lint check.

- "A key or button starts a match" — `editor_playtest_shortcut` (F5) and
  `editor_playtest_click` (the `Play` row of the new panel) both go through
  `playtest::start_playtest`, and both are registered in the editor's `Update`
  set (`src/editor/mod.rs:551`). Covered by
  `playtesting_plays_the_level_as_it_stands_and_not_the_file` and
  `the_play_button_starts_a_playtest_too`.
- "The match uses the in-memory level" — `start_playtest` does
  `level_assets.add(LevelAsset(editor_level.level.clone()))` and points
  `Levels::playtest` at it; `current_handle` prefers it over the campaign
  (`src/level/mod.rs:137`). Nothing on that path touches the file. The test
  paints first and asserts the level played is the edited one, not the one on
  disk. `readiness` returns `Ready` for a handle from `Assets::add` because it
  checks `levels.contains(handle)` before asking the asset server, so the
  match really does start — `a_playtest_starts_a_match_on_the_level_it_was_handed`.
- "Returning goes back to the editor with every unsaved edit" — `playtest_leave`
  (Escape, gated `in_state(InMatch).and_then(playtesting)`) sets
  `GameState::Editor`; `editor_open` returns early when `EditorLevel` already
  exists, and `editor_teardown` does not remove it. Covered by
  `coming_back_from_a_playtest_finds_every_unsaved_edit` and
  `the_level_is_back_on_screen_after_a_playtest`, and again for a second trip by
  `a_second_playtest_plays_what_was_painted_after_the_first`.
- "Survives `match_despawn` and the `OnExit(PostMatch)` teardown" — met by
  construction and read rather than exercised end to end: `game_flow_handler`
  routes a won or lost playtest to `Editor`, so `PostMatch` is never entered;
  `match_despawn` (`src/match/mod.rs:75`) only despawns `With<Match>` entities,
  and `EditorLevel` is a resource. Worth knowing that the editor's test app
  carries `EditorPlugin` + `EventsPlugin` only, so no test runs a real
  `match_despawn` around a playtest — the argument here is from the code, not
  from a red-to-green test.
- "Leaves no match entities behind" — `playtest_teardown` runs in the
  `OnEnter(Editor)` chain before the editor spawns anything, and its query is
  the union of exactly what the game's own teardowns despawn: `arena_despawn`
  `With<Arena>`, `ship_despawn` `With<Ship>`, `ball_despawn` `With<Ball>`,
  `block_despawn` `With<Block>`, `pickup_despawn_all` `With<Pickup>`, plus
  `PointsDisplay`, `MatchStatsUI` and `Environment3d`. The `Match` marker and the
  particles are correctly left to `OnExit(InMatch)`, which the trip does pass
  through. `coming_back_from_a_playtest_leaves_no_match_behind` and
  `clearing_the_stage_leaves_the_editors_own_things_alone` cover it — though the
  first one's assertion query is the same list as the system's, so it will not
  catch a *new* kind of match entity going unnamed, only a regression in the
  ones listed.
- "Does not advance `current_level` or disturb campaign progress" —
  `start_playtest`/`stop_playtest` only write the `playtest` field, `next_level`
  refuses while playtesting, and `game_start` clears any stray playtest.
  `a_playtest_leaves_the_campaign_exactly_where_it_was`,
  `a_playtest_does_not_walk_the_campaign_on`,
  `stopping_a_playtest_hands_the_campaign_back_where_it_was` and
  `a_new_game_starts_from_the_top_of_the_campaign`.
- `cargo test`: 243 passed, 0 failed. `cargo build`: clean; its 17 warnings are
  pre-existing dead code and `non_snake_case` in `block/`, none from this card's
  files. `cargo clippy` flags two idiom warnings in `src/editor/playtest.rs`
  (`editor_playtest_click` has 8 params, the `Or<..>` teardown query is a
  "very complex type") — noise for Bevy systems, and clippy is not a check the
  repo declares.
- Diff scope: the bevy 0.19 migration of `src/ui/mod.rs`, `src/ui/stats/mod.rs`,
  `src/game/mod.rs` and `src/player/mod.rs` rides along. It is load-bearing here
  — `playtest_teardown` names `MatchStatsUI` and `Environment3d`, `game_start`
  calls `stop_playtest`, and those modules were still on the 0.9 API — and it
  follows what `c0004` and `c0005` did on this branch. Read through: the stats
  rewrite is a faithful `TextBundle`→`Text`/`TextSpan` port with `spawn_stat`
  factored out, no readout dropped.
- No test was weakened. The existing edits are `..default()` for the new `Levels`
  field, and `the_pointer_over_the_editors_own_panels_is_not_over_a_cell` was
  strengthened rather than relaxed — the chrome list grew from two rects to four
  and both original `covered[..] > 0` assertions stayed.
- Not run: the interactive round trip in the running game. The card claims it was
  verified there; it needs a controller and a window, so I checked the code and
  the suite instead.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-26 status → in-progress (agent)
- 2026-08-26 playtest round trip landed; 243 tests pass, `cargo build` clean, verified in the running game (agent)
- 2026-08-26 status → review (agent)
- 2026-08-26 status → done (app)
