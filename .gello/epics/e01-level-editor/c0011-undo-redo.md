---
id: c0011
title: Undo and redo
status: ready
epic: e01
depends: [c0007, c0010]
created: 2026-08-25
updated: 2026-08-25
status-changed: 2026-08-25T23:00:03
order: 110
---

## What

An undo/redo stack over editor operations. Every edit becomes a reversible
command rather than a direct mutation of the level.

## Acceptance criteria

- [ ] Cell paints, erases, grid resizes and settings changes are all reversible commands.
- [ ] Undo and redo are bound to keys and reachable from the UI.
- [ ] A drag-paint is a single undo step, not one per cell.
- [ ] Making a new edit after undoing discards the redo stack.
- [ ] History is cleared if the level file changes on disk underneath the editor.

## Notes

- This is a design constraint on `c0007`, `c0008` and `c0010` rather than a
  bolt-on: their edits have to be expressed as commands. If those land first as
  direct mutations, this card includes converting them.
- The on-disk-change case matters because `c0004` hot-reloads files: an external
  edit can invalidate the history's assumptions entirely.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
