---
id: c0009
title: Palette UI and shortcuts
status: in-progress
epic: e01
depends: [c0007]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T01:48:18
usage-tokens: 13256
usage-cost: 1.826775
---

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

- [x] **A — one modifier per axis.** Bare `A`–`D`/`Z` set the block type, `Shift`+`A`–`I` set the behaviour, `Alt`+`A`/`B`/`C`/`R`/`S` set the trigger type, `0`–`9` set the group. Non-modal: every entry is one press, always, and the block type — the thing you change most — keeps the bare letter. (`Alt` rather than `Ctrl` so that `c0011`'s `Ctrl+Z` undo and `c0012`'s `Ctrl+S` save do not later collide with trigger `Z`… `S`.)
- [ ] **B — type the token.** Press `A` `A` and you have typed `AA`; a third and fourth press add the trigger and its group. Matches the file format exactly, and nothing needs a modifier — but it is modal, and changing *just* the behaviour of the current brush means retyping the type first.
- [ ] **C — bare letters, one axis wins each.** `A`–`D` and `Z` go to the block type, `E`–`I` to the behaviour, `R`/`S` to the trigger receivers, `0`–`9` to the group; the colliding behaviours (SittingDuck…Repulsor) and triggers (Start/Stop/StartStop) are then click-only, with no shortcut. Simplest to use, but leaves the most common behaviour of all — `A`, SittingDuck — unreachable from the keyboard, so criterion 5 is only partly met.
- [ ] Something else (say what)

My recommendation is **A**: it is the only one that gives every entry its own
one-press shortcut without a mode, and the modifier still "matches the letter"
the format uses.

## What

A way to choose what you are painting, without memorising the format.

Block types are shown as swatches in the block's own colour carrying its format
letter; behaviours are text labels, since behaviour does not change a block's
colour. Everything is clickable, and every entry is also bound to the letter key
it corresponds to in the file format.

## Acceptance criteria

- [x] Every block type appears as a swatch in that block's actual colour with its format letter on it (`A` orange, `B` gray, `C` dark gray, `D`, `Z` white).
- [x] Behaviours `A`–`I` are listed as text labels with their letters.
- [x] Trigger type and trigger group (`0`–`9`) are selectable.
- [x] Clicking any entry sets the corresponding part of the brush.
- [x] Each entry has a keyboard shortcut matching its letter in the file format.
- [x] The current brush is visible at all times.

## Notes

- Block colours are assigned in `block_spawn` in `src/block/mod.rs` — orange for
  Simple, gray for Hardling, dark gray for Concrete, white for Obstacle, and
  orange/white for SimpleTop's split. Reuse those constants rather than
  re-picking colours.
- Built from the `Node` / `Text` UI already used elsewhere; no widget toolkit is
  in the project.

- **Groundwork established before asking the question above** (so it is not
  re-done, and before the rest of the epic landed): Bevy 0.19's `Interaction` is driven by `ui_focus_system`, which needs
  the whole UI stack up - `UiPlugin`, layout, `UiStack`, visibility propagation
  and a camera with a `RenderTarget`. Added to the editor's headless test app it
  panics in two nameless systems on a missing resource, so a click on a palette
  entry could not be tested the way `c0006`/`c0007` test a click on a cell -
  through the real window and the real pointer. The palette will therefore hit-test
  itself against rectangles it computes, which is the same trade the editor already
  makes for the grid (`world_to_cell`, not mesh picking): one pure layout function
  gives every entry its rect, the spawner draws that rect and the click reads it,
  so what is on screen and what is clickable cannot drift.

**As built** - `src/editor/palette.rs` (the whole of it), plus the four lines it
needed elsewhere: `block_colours` out of `block_material` in `src/block/mod.rs`,
and the module, the chain and the pointer in `src/editor/mod.rs`.

- The palette went in **after the rest of the epic had landed**, so it fits the
  chrome `c0010` established rather than inventing its own: `panel_node`,
  `panel_text`, `PANEL_PADDING`, `ROW_HEIGHT`, `ROW_WIDTH`, `PANEL_Z` and the
  two colours are the settings panel's, and an entry is tagged with a
  `PaletteChoice` the way a settings button is tagged with `SettingButton`.
  Nothing here is a second opinion about what a panel looks like.
- **It sits down the right**, which is the one thing it does differently. The
  left column is four panels deep - settings, history, save, playtest - and
  reaches y=572 before the save report starts growing, where the palette is 598
  tall on its own. So `palette_rect` takes the window's width, which no other
  panel needs: it is the only one anchored to the far side.
- **Erase is the sixth swatch of the block row**, not a switch beside the
  palette. `.` is the format's own "no block here", so the erase brush is the
  same character of the same token as the other five and choosing a block is
  what puts it down again. The card did not ask for erase at all; leaving it out
  would have meant the only way to clear a cell stayed the right mouse button
  `c0007` added as a stopgap "until `c0009`'s palette can switch the brush's own
  erase mode on".
- **The letters are read back out of `block_token`**, not restated. A palette
  offering a letter the file would not be read back with is the one failure mode
  that would be invisible until a level was saved and reopened, so there is no
  second alphabet to drift from the first.
- **The modifier is the token's position.** The format writes `A` for a Simple
  block, a SittingDuck behaviour and a Start trigger, telling them apart by
  where they sit in the token; a keyboard has no positions, so the axis became
  the modifier - bare, `Shift`, `Alt`, digits - per the answer on the question
  above. `Alt` and not `Ctrl` mattered in practice: `entry_typed` also has to
  refuse anything under `commanding`, or `Ctrl+Z` would undo an edit *and* pick
  up the `Z` block on the way past, and `Ctrl+S` would save while changing the
  brush.
- **`BrushGroup` is a resource beside the brush rather than a field in it.**
  `c0007` fused trigger type and group into one `Option<(TriggerType,
  TriggerGroup)>` so half a trigger cannot be built - which leaves nowhere to
  put a group chosen *before* a type. Without somewhere, the digit row would be
  dead half the time; with it, `3` then `Start` and `Start` then `3` are the
  same brush, which is a test.
- **The palette hit-tests itself**, as the settings panel does and for the same
  reason the grid does: Bevy's `Interaction` needs `UiPlugin`, layout, the UI
  stack, visibility propagation and a camera with a render target, none of which
  survives a headless test - spiked, and it panics in two systems on a missing
  resource. One `palette_items` gives every entry its rectangle, the drawing
  puts it there and the click reads the same one.
- The palette is cut out of `editor_pick_cell` alongside the other four panels,
  so choosing a brush does not also paint with the one being replaced - and the
  hover highlight goes too, rather than a cell sitting under the panel looking
  armed.
- Tests: 25 new, 268 green, build clean at 17 warnings, all pre-existing and
  none in the new code. Seventeen are pure - the format's whole alphabet through
  the palette (all 2295 tokens, chosen entry by entry, with the group picked
  before the trigger every time), no letter twice in a row, no two entries under
  one press, every entry typeable, a press belonging to exactly one row, the
  `Ctrl` chords belonging to the editor, exactly one entry of each row outlined
  for every brush, an evader at a speed the format cannot hold still showing its
  own row, every entry found where it is drawn, and the panel covering
  everything it lays out. Eight drive a real app: every entry clicked in turn
  and checked to have set its own part of the brush and nothing else, every
  shortcut checked to set what its click sets, the swatch colours read back off
  the screen and compared with `block_spawn`'s own table, every label and letter
  read off the screen, the token on screen after each of five choices, the four
  outlines, the palette coming and going with the editor, and a click on an
  entry that is over a cell painting nothing.
- `the_pointer_over_the_editors_own_panels_is_not_over_a_cell` gained the
  palette and had to change window: at 400x800 the two columns between them
  covered every cell of the grid, so it now asks a 800x800 window, where the
  grid runs under both and still shows between them.
- Four mutations were checked to fail: behaviours moved onto bare letters,
  `commanding` dropped from `entry_typed`, the swatches re-picking their own
  colours, and the group forgotten when no trigger is set. Each takes down
  between one and four of the new tests.
- **Checked in the running game**, driven by a temporary system that opened the
  editor, read the palette's laid-out sizes and aimed a synthetic pointer
  through the real window: 84 nodes, none collapsed, 31 entries, every one of
  them the physical size its logical rectangle asks for. That run is what caught
  the thing no headless test can - **the window is 1512x800 at a scale factor of
  2**, where every test runs at 1. The hit test reads `Window::cursor_position`,
  which is logical, against `Val::Px` rectangles, which are logical, so a click
  on the `Concrete` swatch at logical (1299, 94) set the brush to `CA` as it
  should. **Not** visually confirmed: `screencapture` cannot reach the display
  from this session, so how it *looks* rests on the layout numbers above and on
  the colours being `block_spawn`'s own.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-26 status → in-progress (agent)
- 2026-08-26 asked how the format's letter collisions should be resolved; human
  chose one modifier per axis
- 2026-08-26 `src/editor/palette.rs`: the four rows, the letters read back out of
  `block_token`, `BrushGroup` beside the brush, `block_colours` shared out of
  `block_material`; 25 new tests, 268 green, checked in the running game
