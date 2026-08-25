---
id: c0008
title: Grid resize
status: backlog
epic: e01
depends: [c0007]
created: 2026-08-25
updated: 2026-08-25
status-changed: 2026-08-25T22:58:50
---

## What

Let the grid grow and shrink. The shipped levels run from 3 to 11 columns wide,
so a fixed extent could not express them — resizing is not optional.

Rows can be added or removed at top and bottom, columns at left and right.

## Acceptance criteria

- [ ] Rows can be added and removed at the top and at the bottom.
- [ ] Columns can be added and removed at the left and at the right.
- [ ] Removing a row or column containing blocks either warns first or is undoable.
- [ ] Existing blocks keep their position relative to the cells that are retained.
- [ ] All rows are padded to equal width when written, so `interpret_grid`'s first-line column count stays correct for ragged edits.
- [ ] A resized level saves, reloads and plays correctly.

## Notes

- `interpret_grid` takes its column count from the first line alone, so a ragged
  grid silently misaligns every row after it. The padding requirement above is
  what prevents that.
- Grid extent is not stored explicitly today — it is implied by the token string.
  Resizing therefore only has to produce a well-formed string.

## Log

- 2026-08-25 created from the e01 epic breakdown
