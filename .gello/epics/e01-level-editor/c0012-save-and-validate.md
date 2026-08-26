---
id: c0012
title: Save to disk with validation warnings
status: in-progress
epic: e01
depends: [c0008, c0010]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T02:37:09
---

## What

Write the edited level back out as RON, and warn about structural mistakes on the
way — without ever refusing to save.

## Acceptance criteria

- [x] Saving writes the level as RON with `std::fs` (the asset server is read-only).
- [x] Saving does not trigger a hot-reload that discards in-editor state.
- [x] The saved file reloads to an identical level.
- [x] A newly created level can be appended to `assets/levels/campaign.ron` from the editor.
- [x] Saving warns, but never blocks, on: a trigger receiver with no matching trigger in its group; a portal with no trigger; a level with no breakable blocks.
- [x] Warnings are visible in the editor rather than only in the log.

## Notes

- The self-inflicted hot-reload is the real hazard: `c0004` watches
  `assets/levels/`, so the editor's own write comes straight back as an asset
  change. Ignore the reload for the file we just wrote, or suppress the watcher
  across the write.
- Trigger semantics live in `src/block/trigger.rs`: receivers (`R`, `S`) and
  triggers (`A`, `B`, `C`) pair up by `TriggerGroup`. A receiver whose group has
  no trigger can never activate.

### The shape it took

- **The self-inflicted reload is settled by content, not by a flag.** The file
  watcher reports that a file changed, never who changed it, and it reports it a
  debounce later - by which time the author may have painted three more cells. So
  `LastSave` keeps *what the editor last wrote*, and `editor_watch_the_file`
  drops the history only when the file has come back saying something else. A
  hand edit that happens to reproduce the saved level is let through too, which
  is correct: the history is still entirely true of that file.
- **A complaint is a remark, never a veto.** The file is on disk before the level
  is read over at all. An author halfway through wiring a trigger up has a level
  with a receiver and no trigger in it, and a save that refused would punish them
  for stopping for lunch.
- **"A portal with no trigger" is really "a portal that is not a receiver".**
  `block_update_portals` only moves a ball through a portal carrying a
  `BlockTriggerTarget`, which is what the `R` and `S` trigger types put on one -
  so a portal marked `A` is a trigger rather than a receiver and is every bit as
  shut as one with no trigger at all. The warning covers both.
- **A third panel** under the history bar, laid out from the same chrome as
  `c0010`'s settings panel and hit-tested against its own rectangles the same
  way. `Save` writes; `Campaign` enrols. Under them the report: what the last
  save did, and a line per complaint in orange. `Ctrl+S` saves from the keyboard.
- **A level that has never been on disk is given a name rather than refused
  one**: `levelN.ron`, one past the highest the directory holds. There is nowhere
  in this game to type a file name, and the number means nothing beyond "not
  taken" - play order is `campaign.ron`'s to say.
- **Enrolling is a separate press from saving**, and the campaign is never
  touched unasked. A saved level that is not in `campaign.ron` is scratch, which
  is what the epic already decided a level file not listed there is.
- **`campaign.ron`'s header is now code** (`CAMPAIGN_HEADER`), because RON drops
  comments on the way through a parse and the editor rewrites that file every
  time a level is enrolled. `the_campaign_index_is_written_back_exactly_as_it_reads`
  holds the constant to the file.

### Worth knowing

**Saving a level file loses the comments in it.** A save is a rewrite, not a
patch, so the hand-written notes at the top of `level0.ron`, `level5.ron`,
`level6.ron`, `conveyor.ron`, `simple1.ron`, `demo_moving.ron` and
`demo_minimal_win_state_error.ron` go the first time the editor writes one of
them, and the field order becomes the serializer's. The *level* survives whole -
that is what `saving_a_level_keeps_the_level_and_not_the_comments_around_it`
pins down. `campaign.ron` is the exception, because the editor appends to it
routinely rather than once.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-26 status → in-progress (agent)
- 2026-08-26 save + validation landed; 218 tests pass, `cargo build` clean (agent)
