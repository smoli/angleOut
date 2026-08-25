---
id: c0009
title: Palette UI and shortcuts
status: backlog
epic: e01
depends: [c0007]
created: 2026-08-25
updated: 2026-08-25
status-changed: 2026-08-25T22:58:50
---

## What

A way to choose what you are painting, without memorising the format.

Block types are shown as swatches in the block's own colour carrying its format
letter; behaviours are text labels, since behaviour does not change a block's
colour. Everything is clickable, and every entry is also bound to the letter key
it corresponds to in the file format.

## Acceptance criteria

- [ ] Every block type appears as a swatch in that block's actual colour with its format letter on it (`A` orange, `B` gray, `C` dark gray, `D`, `Z` white).
- [ ] Behaviours `A`–`I` are listed as text labels with their letters.
- [ ] Trigger type and trigger group (`0`–`9`) are selectable.
- [ ] Clicking any entry sets the corresponding part of the brush.
- [ ] Each entry has a keyboard shortcut matching its letter in the file format.
- [ ] The current brush is visible at all times.

## Notes

- Block colours are assigned in `block_spawn` in `src/block/mod.rs` — orange for
  Simple, gray for Hardling, dark gray for Concrete, white for Obstacle, and
  orange/white for SimpleTop's split. Reuse those constants rather than
  re-picking colours.
- Built from the `Node` / `Text` UI already used elsewhere; no widget toolkit is
  in the project.

## Log

- 2026-08-25 created from the e01 epic breakdown
