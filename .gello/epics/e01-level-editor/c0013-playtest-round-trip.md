---
id: c0013
title: Playtest round trip
status: in-progress
epic: e01
depends: [c0007, c0010]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T03:03:29
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

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-26 status → in-progress (agent)
- 2026-08-26 playtest round trip landed; 243 tests pass, `cargo build` clean, verified in the running game (agent)
