---
id: c0011
title: Undo and redo
status: done
epic: e01
depends: [c0007, c0010]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T21:55:23
usage-tokens: 93403
usage-cost: 8.508495
---

## What

An undo/redo stack over editor operations. Every edit becomes a reversible
command rather than a direct mutation of the level.

## Acceptance criteria

- [x] Cell paints, erases, grid resizes and settings changes are all reversible commands.
- [x] Undo and redo are bound to keys and reachable from the UI.
- [x] A drag-paint is a single undo step, not one per cell.
- [x] Making a new edit after undoing discards the redo stack.
- [x] History is cleared if the level file changes on disk underneath the editor.

## Notes

- This is a design constraint on `c0007`, `c0008` and `c0010` rather than a
  bolt-on: their edits have to be expressed as commands. If those land first as
  direct mutations, this card includes converting them.
- The on-disk-change case matters because `c0004` hot-reloads files: an external
  edit can invalidate the history's assumptions entirely.

### The shape it took

- **An edit is the two levels either side of it**, not a hand-written inverse.
  A `LevelDefinition` is a token grid plus ten small fields, so a snapshot pair
  is cheap - and it is the only form that is *provably* reversible: the inverse
  of "take the top row away" has to carry the blocks that stood on it anyway,
  which is a snapshot of that row by another name. One `Edit` type covers
  paints, erases, resizes and settings alike.
- **A drag is one entry** because `c0007` already recorded the stroke as a
  whole (`PaintStroke`); its `before` is widened from the layout to the level
  and the entry goes in when the button comes up. Anything else the author does
  mid-drag - a resize, a setting, an undo - ends the stroke first, so no edit
  ever swallows another.
- **The file under edit is now a handle.** `EditorLevel::source` was the asset
  *path* it was opened from; it becomes the `Handle<LevelAsset>`, which is what
  an `AssetEvent::Modified` can be matched against. The path is still there as
  `source_path()`, which is what `c0012` saves through.
- **Reachable from the UI** as a two-row "History" bar under the settings
  panel, hit-tested against its own rectangles exactly as `c0010`'s panel is,
  and excluded from cell picking the same way. The panel chrome `c0010` wrote
  is shared rather than copied.

### Keys

`Ctrl+Z` undoes, `Ctrl+Y` and `Ctrl+Shift+Z` both redo, and the Mac's `Cmd`
counts as `Ctrl` throughout. The bar's title says `Ctrl+Z / Ctrl+Y` - the short
pair, because a button wide enough to hold `Ctrl+Shift+Z` leaves no room for
what it would take back.

### Two things that fell out of it

- `c0008` left "warns first *or* is undoable" half-done: an edge with blocks on
  it was only ever called out. It is now both, and undoing the removal brings
  the blocks back with the row.
- A stroke the author was in the middle of when they left the editor used to be
  dropped. What it painted was already in the level, so it was the one edit that
  could not be taken back; it now goes into the history on the way out.

### Not verified by eye

The bar's geometry is arithmetic, not a screenshot: this session can drive
neither the keyboard nor `screencapture`, so nothing in the editor is reachable
by hand from here. Every claim above is covered by a test that goes through the
real window, camera and mouse messages - but how the bar *looks* is worth one
glance from a human.

## Review

### 2026-08-26T02:36:25 — pass

Checked: the five acceptance criteria against the code, the diff of `b917494`,
`cargo test`, `cargo clippy --all-targets`.

- "Paints, erases, resizes and settings changes are all reversible": all three
  edit paths record before they mutate — `editor_paint` via `finish_stroke`,
  `editor_resize` and `editor_settings_click` via `remember` around their
  `bypass_change_detection` blocks (`src/editor/mod.rs:764`, `:924`, `:1100`).
  Grepping every write to `EditorLevel` found no fourth path that edits the
  level outside the history. Covered by `a_painted_cell_is_undone_and_redone`,
  `an_erase_is_undone_and_redone`, `a_resize_is_undone_and_redone`,
  `a_settings_change_is_undone_and_redone`.
- "Bound to keys and reachable from the UI": `step_asked_for`
  (`src/editor/history.rs`) takes Ctrl/Cmd + Z, Ctrl+Shift+Z and Ctrl+Y; the
  bar's own buttons go through `editor_history_click`/`step_at`. Verified by
  `the_bars_buttons_take_the_step_they_name` and
  `holding_a_bar_button_down_takes_one_step` (press, not hold). The bar is
  excluded from cell picking in `cell_under_cursor`, and
  `a_click_on_the_history_bar_paints_nothing` plus the widened
  `the_pointer_over_the_editors_own_panels_is_not_over_a_cell` hold that down.
- "A drag-paint is a single undo step": `Stroke::before` widened from the layout
  to the whole `LevelDefinition`, recorded once in `finish_stroke`. Verified by
  `a_drag_paint_is_a_single_undo_step`, and the harder half —
  `an_edit_made_mid_drag_is_an_entry_of_its_own` — shows a resize made with the
  button still down does not get swallowed.
- "A new edit after undoing discards the redo stack": `EditHistory::record`
  clears `undone` before pushing; `a_new_edit_after_an_undo_discards_the_redo_stack`
  checks both the depth and that the redo press then does nothing.
- "History cleared if the level file changes on disk": `editor_watch_the_file`
  matches `AssetEvent::Modified` against `EditorLevel::source`, now the
  `Handle<LevelAsset>`. `filter(..).count()` rather than `any`, so no message is
  left behind to re-fire. Verified by
  `the_history_is_dropped_when_the_level_file_changes_on_disk`, with
  `a_change_to_another_level_file_leaves_the_history_alone` and
  `a_level_that_was_never_opened_from_a_file_keeps_its_history` for the
  negatives.
- Diff stays inside the What: three files, all under `src/editor/`. The
  `source: String` → `Handle<LevelAsset>` change is what an `AssetEvent` can be
  matched against, and `source_path()` keeps what `c0012` needs; nothing outside
  `src/editor/` read that field. The `settings.rs` change is `const`/`fn`
  visibility only, no behaviour. No test weakened: the one edited test
  (`the_pointer_over_the_editors_own_panels_is_not_over_a_cell`) got a stricter
  assertion — it now insists cells end up behind *both* panels.
- `cargo test`: 179 passed, 0 failed, 0 ignored. `cargo clippy --all-targets`:
  exit 0, and no warning points at `src/editor/` — the 108 it reports are
  pre-existing, in `src/level/layout` and elsewhere. Run against the working
  tree, which carries unrelated uncommitted changes to ~30 non-editor files;
  nothing under `src/editor/` is uncommitted.

Two things worth knowing, neither a failed criterion:

- `Ctrl+Y` and the Mac's `Cmd` are branches of `step_asked_for` no test
  exercises — every keyboard test presses `Ctrl+Z` or `Ctrl+Shift+Z`.
- `editor_watch_the_file` only runs `in_state(GameState::Editor)`, and Bevy
  drops unread messages after two frames. Since the history deliberately
  survives a trip out to the menu (`the_history_survives_a_trip_out_of_the_editor_and_back`),
  a hand edit to the file made *while the author is out of the editor* leaves a
  stale history behind on the way back in. The criterion says "underneath the
  editor", so this is outside it — but it is the same hole the criterion exists
  to close.

The card's own "not verified by eye" still stands: I checked the bar's geometry
as arithmetic (it sits at y 300–394, x 16–356, clear of the 9-row settings panel
above it) and not as pixels. The human glance it asks for is still outstanding.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-26 status → in-progress (agent)
- 2026-08-26 undo/redo landed: `editor/history.rs` (the stack, the keys, the
  bar), with painting, resizing and settings recording through it. 22 new tests;
  `cargo test` 179 pass, `cargo build` clean.
- 2026-08-26 status → review (agent)
- 2026-08-26 status → done (app)
