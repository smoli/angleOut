---
id: c0009
title: Palette UI and shortcuts
status: review
epic: e01
depends: [c0007]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T07:18:38
usage-tokens: 178660
usage-cost: 23.96981
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

**After the first review** - the palette was drawn at the width it was spawned
at and clicked at the width the window is now, so a resize pulled the two apart.
The review was right, and the tests were blind to it because they asked
`palette_items` where an entry is and then clicked there, which only ever proved
the layout function agrees with itself.

- **`PaletteWidth` is the width the palette on screen was laid out for.** Written
  by `editor_show_palette` and read by the click and by `cell_under_cursor`, so
  the rectangles a press is tested against are the rectangles the author is
  looking at. The redraw alone would not have been enough: a click is read before
  the panel is drawn again - it has to be, or choosing a brush would lag the press
  by a frame - so on the one frame a window actually changes, the entries are
  still where they were put and only this remembers where that was.
- **The redraw asks a state, not an event.** `the_palette_is_out_of_date` compares
  the window's width with the width on screen rather than listening for
  `WindowResized`. A state cannot be missed, it answers correctly however the
  window came to be that wide, and it is testable - a headless app that assigns
  `Window::resolution` emits no resize message at all, so an event-driven redraw
  would have been untestable with the helper every other test in this file uses.
- **The palette can no longer climb onto the column down the left.**
  `palette_left` is clamped to the right of `panel_rect()`, so on a window too
  narrow for two columns the palette's right-hand end runs off the edge instead
  of lying over the settings panel. That is the better of two bad options:
  overlapping panels would mean a click landing on an entry *and* a setting at
  once - neither system consumes the press - where an entry off the edge is still
  reachable by its shortcut. That is what the modifiers are for, and the narrow
  window test asks for exactly it.
- **Every click test now reads the rectangle off the node the panel spawned**, via
  `drawn_rect`, rather than off `palette_items`. That is the change that makes
  this class of bug visible at all, and it applies to the tests that were already
  there as well as the new ones.
- Six new tests, 274 green: the panel following the window both ways, all 31
  entries clicked at their drawn rectangles across four window widths, the ground
  the palette has left going back to being play field, a press in the very frame
  the window changes, the clamp across eight widths from 0 upwards, and a 600px
  window where the swatches and most of the digits are clickable and the rest is
  keyboard-only - plus 400px, where nothing is clickable and everything still has
  to be typeable. Build unchanged at 17 warnings, none in the new code.
- Three mutations checked to fail: the click reading the live width again (caught
  by the same-frame test alone, which is why that test exists), the redraw
  dropped back to `resource_changed::<Brush>` (two tests), and the clamp removed
  (two tests).
- **Checked in the running game again**, since the draw path changed: the 1512px
  window drew the palette at x=1156..1496 with the `Obstacle` swatch at 1382.7;
  resized to 1100 it redrew at 744..1084 with the swatch at 970.7 - moved by
  exactly the 412 the window lost - and a click at the swatch's new centre set
  the brush to `ZA`. That is the review's own scenario, run.

## Review

### 2026-08-26T06:58:24 — fail

Checked: the six acceptance criteria against `src/editor/palette.rs` and
`src/editor/mod.rs`, the diff of `5eec2e6`, `cargo test`, `cargo build`
warnings. No lint or typecheck step exists in this repo beyond `cargo build`,
so none was run.

- Criteria 4 and 6 are unmet once the window is resized. `editor_show_palette`
  spawns the panel from `palette_rect(window.width())` into absolute
  `Val::Px` nodes, and it only runs `OnEnter(Editor)` and under
  `resource_changed::<Brush>.or_else(resource_changed::<BrushGroup>)`
  (`src/editor/mod.rs:522`, `:605`). Nothing in the repo listens for
  `WindowResized`, and `main.rs` leaves Bevy's `resizable: true` default. So
  the drawn palette keeps the width it was spawned at while
  `editor_palette_click` and `cell_under_cursor` read `window.width()` live —
  the one thing the module's own doc comment promises cannot happen ("what is
  on screen and what is clickable cannot come apart"). Concretely: open the
  editor in the game's 1512px window, where the entries are drawn across
  x=1156..1480, then drag the window to 1200px. `palette_entry_at` now answers
  for x=844..1184 only, so a click on the visibly-drawn `Obstacle` swatch at
  x≈1400 chooses nothing, falls past `palette_rect(1200.0)` in
  `cell_under_cursor`, and paints the cell behind the panel; a click at x≈900,
  where nothing is drawn, is swallowed as a palette click. The current-brush
  row is drawn 300px off the right edge, so it is no longer visible either.
  No test covers this: `choosing_a_brush_does_not_paint_the_cell_behind_the_palette`
  and `the_pointer_over_the_editors_own_panels_is_not_over_a_cell` both resize
  the window but then read `palette_items`/`palette_rect` directly rather than
  the nodes `editor_show_palette` spawned, so the drift is invisible to them.
- Related, same root: `palette_left` has no lower bound, so on a narrow window
  the palette walks left into the settings column and past x=0 (at 400px —
  the width `the_pointer_over_the_editors_own_panels_is_not_over_a_cell` used
  before this card — `palette_left` is 44, and the palette overlaps the left
  column). Widening that test to 800x800 keeps its own assertions honest and is
  not a weakening, but it does mean nothing now exercises a window narrower
  than the palette's own two columns.

Verified and sound otherwise:

- Criterion 1: `every_swatch_wears_the_block_s_own_colour` reads the spawned
  `BackgroundColor` back and compares it with `block_colours`, which is
  `block_material`'s own table extracted rather than re-picked — orange,
  gray, dark gray, the orange/white `SimpleTop` split and white all come from
  the one place, and the split swatch draws its second colour too.
- Criteria 2 and 3: `every_behaviour_and_trigger_is_on_screen_in_words` reads
  every letter and label off the spawned `Text`; `every_letter_the_format_defines_is_in_the_palette`
  pins the rows to `ABCDZ.` / `ABCDEFGHI` / `.ABCRS` / `0123456789`, all taken
  back out of `block_token` rather than restated.
- Criterion 4 (unresized): `clicking_every_palette_entry_sets_that_part_of_the_brush`
  walks all 31 entries cumulatively through the real click path, and
  `choosing_the_entries_of_a_token_paints_that_token` walks all 2295 tokens
  with the group chosen before the trigger.
- Criterion 5: `every_entry_can_be_typed`, `no_two_entries_answer_to_the_same_press`,
  `a_press_belongs_to_exactly_one_row` and `every_palette_shortcut_sets_what_clicking_the_entry_sets`
  cover the one-modifier-per-axis scheme the question settled on, and
  `a_chord_the_editor_owns_is_not_also_a_palette_entry` covers the `commanding`
  guard. Cross-checked against the editor's other bindings: `F5`, `Escape`,
  the arrows and the `Ctrl` chords are all clear of `A`–`I`/`R`/`S`/`Z`/digits.
- Diff scope is the card's What: `src/editor/palette.rs`, the `block_colours`
  extraction in `src/block/mod.rs`, and the module/chain/pointer wiring in
  `src/editor/mod.rs`. No debug code left in, no test skipped or `.only`'d.
- `cargo test`: 268 passed, 0 failed — the count the card claims. `cargo build`
  is clean at 17 warnings, all pre-existing (`config`, `match/state`, `state`,
  `events`, `ship`, `block`, `level/campaign`, `player`, `powerups`), none in
  the new code.
- Not verified: the in-game check the Notes describe (84 nodes, the 2x scale
  factor, the click on the `Concrete` swatch). That rests on the implementer's
  own run.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-26 status → in-progress (agent)
- 2026-08-26 asked how the format's letter collisions should be resolved; human
  chose one modifier per axis
- 2026-08-26 `src/editor/palette.rs`: the four rows, the letters read back out of
  `block_token`, `BrushGroup` beside the brush, `block_colours` shared out of
  `block_material`; 25 new tests, 268 green, checked in the running game
- 2026-08-26 status → review (agent)
- 2026-08-26 status → in-progress (agent)
- 2026-08-26 review → fail: the panel was drawn at its spawn width and clicked at
  the window's live one, so a resize pulled the two apart
- 2026-08-26 `PaletteWidth` records what is on screen and the click reads it, the
  redraw is driven by a width comparison rather than an event, `palette_left` is
  clamped clear of the left column, and every click test now reads the drawn
  node; 6 new tests, 274 green, the review's own resize scenario run in the game
- 2026-08-26 status → review (agent)
