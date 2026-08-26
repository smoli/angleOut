---
id: c0011
title: Undo and redo
status: in-progress
epic: e01
depends: [c0007, c0010]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T02:14:52
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

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-26 status → in-progress (agent)
- 2026-08-26 undo/redo landed: `editor/history.rs` (the stack, the keys, the
  bar), with painting, resizing and settings recording through it. 22 new tests;
  `cargo test` 179 pass, `cargo build` clean.
