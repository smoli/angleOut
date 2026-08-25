---
id: c0013
title: Playtest round trip
status: backlog
epic: e01
depends: [c0007, c0010]
created: 2026-08-25
updated: 2026-08-25
status-changed: 2026-08-25T22:58:50
---

## What

Play the level you are editing and come back to it, with unsaved edits intact.
This is what makes the editor worth using rather than a thing you save from and
restart around.

## Acceptance criteria

- [ ] A key or button starts a match on the level currently being edited.
- [ ] The match uses the in-memory level including unsaved edits, not the file on disk.
- [ ] Returning goes back to the editor with every unsaved edit still present.
- [ ] The edited level survives `match_despawn` and the `OnExit(GameState::PostMatch)` teardown.
- [ ] Returning leaves no match entities behind (ship, balls, blocks, arena, pickups, points).
- [ ] Playtesting does not advance `Levels::current_level` or otherwise disturb campaign progress.

## Notes

- The teardown is aggressive: `OnExit(PostMatch)` despawns arena, ship, blocks and
  pickups, and `match_despawn` runs on exiting `InMatch`. The edited level must
  therefore live in a resource (`c0005`), never in entities.
- Returning from a match currently routes through `GameFlowEvent` into
  `PostMatch`; the editor needs its own return path that does not fall into the
  normal win/lose flow.

## Log

- 2026-08-25 created from the e01 epic breakdown
