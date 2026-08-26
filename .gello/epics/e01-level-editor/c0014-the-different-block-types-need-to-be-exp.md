---
id: c0014
title: The different Block types need to be explained
status: in-progress
ref: c0009
epic: e01
created: 2026-08-26
updated: 2026-08-26
status-changed: 2026-08-26T21:57:28
---

![image](../../assets/c0014/image.png)

It could say next to „Block“ what it is, regular, …

Also we do not need the „.“ Block. It was in the text based level format to denote an empty position

## Notes

**As built** - `src/editor/palette.rs`, plus the four doc comments in
`src/editor/mod.rs` that said the right mouse button was a stopgap.

- **The block row's heading says which block is chosen, in words.** "BLOCK" gives
  up its right-hand end to `block_name` of whatever the brush is set to -
  "Regular, 1 hit", "Tough, 2 hits", "Concrete, 3 hits", "Regular, top only",
  "Obstacle, never breaks". The names are what tells the five apart in play,
  which is how much it takes to break them: `block_spawn` gives `Simple` one hit
  point, `Hardling` two, `Concrete` three, `SimpleTop` one but only from above,
  and an `Obstacle` is not `Hittable` at all.
- **One name at a time, and not under the swatches.** The five swatches share
  324 pixels, so each is 61 wide and "Obstacle, never breaks" is 195 - the words
  go where there is room for them. The heading was already there, already on the
  block row, and already had 260 pixels of nothing to its right.
- **It is a `ChosenBlock` item, not an `Entry`.** `palette_items` stays a pure
  layout function that knows nothing about the brush - the drawing is what reads
  it, as it already did for the `Brush   AA` row at the top - and a click on the
  name chooses nothing, because it says what is chosen rather than offering
  something to choose.
- **The `.` swatch is gone**, and with it `PaletteEntry::Block`'s `Option`. `.`
  in the token's *first* character is how a file says a cell is empty, not a
  block anybody paints, so the type no longer has a way to say it; the trigger's
  `Option` stays, because `.` in the *third* character is a real thing to paint -
  a block in no trigger at all. `Brush::erase` stays too: it is what
  `Brush::erasing()` is, and the right mouse button is now the only thing that
  picks one up, which it already was in practice.
- Five new tests, 278 green, build unchanged at 17 warnings, all pre-existing.
  Three pure - every block type having a name and no two sharing one, the
  heading keeping room beside it for one that a click does not land on, and
  nothing in the whole palette painting an empty cell. Two drive a real app: the
  name on screen following the chosen block through all five (and saying only
  that one), and the block row being five swatches with the right button still
  clearing a cell.
- Three mutations checked to fail: a block type with no name, the heading naming
  a fixed block rather than the chosen one, and choosing a block putting the
  erase brush down. One test each, plus two more for the third.
- **Checked in the running game**, because the one thing a headless test cannot
  answer is whether the name fits: with the brush on `ZA`, "Obstacle, never
  breaks" lays out at 194x18 inside its 236x26 node and "BLOCK" at 61x18 inside
  84x26 - one line each, 42 pixels of slack on the longest name there is. The
  row also drew five swatches and no sixth. (Read off `TextLayoutInfo` through a
  temporary system, since `screencapture` cannot reach the display from this
  session; how it *looks* rests on those numbers.)


## Log

- 2026-08-26 status → in-progress (agent)
