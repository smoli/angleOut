---
id: c0009
title: Palette UI and shortcuts
status: in-progress
epic: e01
depends: [c0007]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T01:48:18
awaiting: input
usage-tokens: 13256
usage-cost: 1.826775
---

```gelloquestion
## How should the palette's keyboard shortcuts resolve the format's letter collisions?

Criterion 5 asks for "a keyboard shortcut matching its letter in the file
format". The format reuses its letters across the three axes, so a bare letter
key cannot name one entry:

| letter | block type | behaviour | trigger type |
|---|---|---|---|
| `A` | Simple | SittingDuck | Start |
| `B` | Hardling | Spinner | Stop |
| `C` | Concrete | Vanisher | StartStop |
| `D` | SimpleTop | Repulsor | — |
| `E`–`I` | — | Evaders, Portal | — |
| `R`, `S` | — | — | the two receivers |
| `Z` | Obstacle | — | — |

The token resolves this by *position* (1st char is the type, 2nd the behaviour,
3rd the trigger); a keyboard has no positions. Trigger group is `0`–`9` and
collides with nothing either way.

Which do you want?

- [ ] **A — one modifier per axis.** Bare `A`–`D`/`Z` set the block type, `Shift`+`A`–`I` set the behaviour, `Alt`+`A`/`B`/`C`/`R`/`S` set the trigger type, `0`–`9` set the group. Non-modal: every entry is one press, always, and the block type — the thing you change most — keeps the bare letter. (`Alt` rather than `Ctrl` so that `c0011`'s `Ctrl+Z` undo and `c0012`'s `Ctrl+S` save do not later collide with trigger `Z`… `S`.)
- [ ] **B — type the token.** Press `A` `A` and you have typed `AA`; a third and fourth press add the trigger and its group. Matches the file format exactly, and nothing needs a modifier — but it is modal, and changing *just* the behaviour of the current brush means retyping the type first.
- [ ] **C — bare letters, one axis wins each.** `A`–`D` and `Z` go to the block type, `E`–`I` to the behaviour, `R`/`S` to the trigger receivers, `0`–`9` to the group; the colliding behaviours (SittingDuck…Repulsor) and triggers (Start/Stop/StartStop) are then click-only, with no shortcut. Simplest to use, but leaves the most common behaviour of all — `A`, SittingDuck — unreachable from the keyboard, so criterion 5 is only partly met.
- [ ] Something else (say what)

My recommendation is **A**: it is the only one that gives every entry its own
one-press shortcut without a mode, and the modifier still "matches the letter"
the format uses.
```

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

- **Groundwork established before asking the question below** (so it is not
  re-done): Bevy 0.19's `Interaction` is driven by `ui_focus_system`, which needs
  the whole UI stack up - `UiPlugin`, layout, `UiStack`, visibility propagation
  and a camera with a `RenderTarget`. Added to the editor's headless test app it
  panics in two nameless systems on a missing resource, so a click on a palette
  entry could not be tested the way `c0006`/`c0007` test a click on a cell -
  through the real window and the real pointer. The palette will therefore hit-test
  itself against rectangles it computes, which is the same trade the editor already
  makes for the grid (`world_to_cell`, not mesh picking): one pure layout function
  gives every entry its rect, the spawner draws that rect and the click reads it,
  so what is on screen and what is clickable cannot drift.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-26 status → in-progress (agent)
