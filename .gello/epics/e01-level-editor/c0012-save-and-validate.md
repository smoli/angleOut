---
id: c0012
title: Save to disk with validation warnings
status: review
epic: e01
depends: [c0008, c0010]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T02:58:47
usage-tokens: 88097
usage-cost: 10.7548
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

## Review

### 2026-08-26T03:02:49 — pass

Checked: the six acceptance criteria against the code, `cargo test`, `cargo build`,
`cargo clippy`, and the diff in `97591f7`.

- "Saving writes the level as RON with `std::fs`": `save_level`
  (`src/level/campaign.rs:150`) goes through `fs::write`; the asset server is only
  ever asked to `load`. `editor_save_shortcut`/`editor_save_click`
  (`src/editor/save.rs:290`, `:311`) both reach it through `save`.
- "Saving does not trigger a hot-reload that discards in-editor state": guarded by
  content rather than a flag - `LastSave::is_what_the_file_now_says`
  (`src/editor/save.rs:71`) compares the reloaded asset with what was written, and
  `editor_watch_the_file` (`src/editor/history.rs:334`) returns before clearing the
  history on a match. Both directions are covered:
  `the_editors_own_save_leaves_the_undo_history_where_it_was` and
  `a_hand_edit_after_a_save_still_drops_the_undo_history`. The level under edit is
  never written by an asset event at all, and `level_reload` is registered under
  `in_state(GameState::InMatch)` (`src/level/mod.rs:184`), so it cannot fire in the
  editor.
- "The saved file reloads to an identical level": asserted through the real reader
  in `saving_writes_the_level_under_edit_to_its_own_file` and in
  `a_level_written_to_disk_reads_back_as_the_same_level`.
- "A newly created level can be appended to `assets/levels/campaign.ron`": a level
  with no file is given `levelN.ron` (`next_free_name`, `src/editor/save.rs:437`) -
  `a_level_that_has_never_been_on_disk_is_given_a_file_of_its_own` also pins that a
  second save does not make a second file - and `enrol` appends to the index
  `LevelsOnDisk` names, which is `levels_dir()` = the game's `assets/levels`.
  `a_saved_level_can_be_added_to_the_campaign_from_the_editor`,
  `a_level_already_in_the_campaign_is_not_played_twice` and
  `a_level_that_was_never_saved_has_no_file_to_put_in_the_campaign` cover the three
  outcomes.
- "Warns, but never blocks": in `save` the write happens and returns early on
  failure only; `complaints` is computed after the file is on disk.
  `a_level_worth_complaining_about_is_saved_anyway_and_the_complaints_are_on_screen`
  asserts the file exists and reloads. All three rules match the runtime they claim
  to: `block_update_portals` (`src/block/mod.rs:635`) queries
  `&BlockTriggerTarget`, which only `ReceiverStartingInactive`/`ReceiverStartingActive`
  insert (`src/block/mod.rs:320`, `:325`), so the widened "portal that is not a
  receiver" is right rather than a change of scope; `make_grid_from_string_layout`
  (`src/level/mod.rs:243`) leaves `Obstacle` out of the count; and `block_spawn`
  reads a missing group as 0 (`src/block/mod.rs:253`), which `group_of` matches.
- "Warnings visible in the editor": `SaveReport::lines` is drawn as `ReportLine`
  text nodes by `editor_show_save`, and the tests read them off the screen rather
  than out of the resource.
- `cargo test`: 218 passed, 0 failed, exit 0. `cargo build`: 17 warnings, all
  pre-existing dead-code/naming ones - none in the new code (`load_level` was
  already test-only before this commit).
- `cargo clippy --all-targets` is not a gate this repo defines and was already at
  109 warnings. One is new: `editor_teardown` (`src/editor/mod.rs:1207`) crosses the
  7-argument threshold now that it takes `SaveReport`. Noise, not a failure - worth
  a `#[allow]` or a bundled `SystemParam` next time that function is touched.
- Diff stays inside the What. The two extractions it makes (`blocks_of`,
  `commanding`) are both things `save` needs and neither changes behaviour; no test
  was removed, weakened, skipped or ignored.
- Not run: nothing. The one thing not exercised by a test is the real file watcher -
  `change_the_file` simulates the `AssetEvent::Modified` it would raise, which is
  how `c0011`'s own tests do it.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-26 status → in-progress (agent)
- 2026-08-26 save + validation landed; 218 tests pass, `cargo build` clean (agent)
- 2026-08-26 status → review (agent)
