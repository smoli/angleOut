---
id: c0015
title: Define what level we’re editing
status: in-progress
created: 2026-08-26
updated: 2026-08-26
status-changed: 2026-08-26T22:22:18
epic: e01
usage-tokens: 7740
usage-cost: 0.950367
---

## What

The editor opens whatever the campaign is pointing at and can never be told
otherwise: `editor_open` inserts `EditorLevel` once, from
`levels.get_current_level()`, and nothing after that can put a different level in
front of it. An author who wants to edit `level4.ron` has to make the campaign
play it first, and there is no way at all to start a level from nothing.

A fifth panel at the foot of the editor's left-hand column, under the playtest
panel: the level files on disk named one at a time with a stepper either side,
an `Open` that puts the named one in front of the editor, and a `New` that
starts a blank grid. Opening replaces what is being edited outright - unsaved
edits and all - and the panel says which file arrived.

## Acceptance criteria

- [x] The panel names a level file on disk, and the steppers walk the whole
      directory - every `*.ron` in it except the campaign index, in a stable
      order, wrapping at both ends.
- [x] Entering the editor points the chooser at the level under edit, so the
      panel opens naming what the author is already working on.
- [x] `Open` puts the named file's level in front of the editor: its blocks, its
      settings, and its path as what the next save writes back to.
- [x] `New` starts a blank grid that belongs to no file, exactly as an editor
      that found no level to open does.
- [x] Both discard whatever was being edited, without asking, and the panel says
      what arrived - including when the file could not be read, which leaves the
      level under edit alone.
- [x] Opening or starting a level drops the undo history: every entry in it is a
      level that belonged to a different file.
- [x] `Ctrl+O` and `Ctrl+N` do the same as the two buttons.
- [x] A click anywhere on the panel is aimed at the panel, never at the grid cell
      behind it.
- [x] `cargo build` is clean and `cargo test` passes.

## How should the author choose which level to edit?

Today `editor_open` (`src/editor/mod.rs:644`) takes `levels.get_current_level()` — whatever the campaign happens to be pointing at — and there is no way to open a different file or start a blank one. The save panel names the file it would write to, so the editor *says* what it is editing; it just cannot be told.

Constraint on where a chooser can go: the left-hand column is already four panels deep (settings 16–292, history 300–394, save 402–548, playtest 556–624 of an 800px window), and the palette owns the right. A list of all 11 level files is ~354px tall, so it does not fit down the left.

**Which shape?**

- [x] **A — a compact "Level" panel at the foot of the left column.** `◀ level4.ron ▶` steps through the files on disk without loading anything, `Open` loads the one named, `New` starts a blank grid. Three rows; reuses the settings panel's own ◀ ▶ idiom and fits the room that is left. *(my recommendation)*
- [ ] **B — a full list of every level file**, one clickable row each. Needs somewhere new to live: a second column under the palette on the right, or a paged/scrolling list down the left.
- [ ] **C — a chooser screen before the editor.** "Editor" in the main menu lists the levels plus "New", and picking one enters the editor on it.
- [ ] **D — something else** (say what).

**And what happens to unsaved edits when another level is opened?**

- [ ] **1 — it replaces what is being edited outright**, and the report line says which file arrived. Matches the save panel's never-block spirit. *(my recommendation)*
- [ ] **2 — Open asks first** when the level has unsaved edits: the row says "unsaved edits — press again to discard", and a second press does it.

**Out of scope unless you say otherwise:** *naming* the file. `Save` still invents `levelN.ron` for a level that has never been on disk, and there is still nowhere in this game to type a name. Tick here if renaming belongs in this card too:

- [ ] Choosing a level should also let me name/rename the file.

Cannot define what level we’e editing right now

## Notes

**As built** - `src/editor/choose.rs` (new), wired into `EditorPlugin` in
`src/editor/mod.rs`.

- **A fifth panel at the foot of the left-hand column**, under the playtest
  panel: the name of one level file across the whole row, then `<` `>` `Open`
  `New` and "2 of 11", then a line saying what is being edited. Laid out and
  hit-tested against its own rectangles, exactly as the four panels above it are.
- **The line under the buttons is the card's own question, answered and kept
  answered**: "editing levels/level0.ron", or "editing a level with no file yet",
  all the time rather than only in the moment something arrives. It gives way to
  "could not read levels/x.ron" for as long as that is the last thing that
  happened.
- **The list is the directory, not the campaign** - every `*.ron` in
  `LevelsOnDisk` except `campaign.ron`, sorted, wrapping at both ends. A level
  that is not in the campaign is scratch, and scratch is what a level halfway
  through being authored is. A directory that cannot be read is a panel with
  nothing to offer rather than a panic.
- **Entering the editor points the chooser at the level under edit**, so the
  panel opens naming what the author is already working on. A save re-reads the
  directory (`editor_relist_levels`, on the report changing), because a save is
  the one thing that can put a file there while the editor is up - and it keeps
  the author's choice where it is rather than letting a file that sorts in front
  steal it.
- **Opening replaces what is being edited outright**, per the answer on the card
  (option 1 was not ticked either way, so the recommendation stands). What goes
  with it: the undo history, whose every entry is a level that belonged to the
  file just closed; the stroke the mouse was in the middle of; the removal
  warning; `LastSave`; and the save report, because "Saved levels/level0.ron"
  over a level that is no longer `level0.ron` has stopped being true. A file that
  will not read changes none of it and says so.
- **Ctrl+O and Ctrl+N**, said in the panel's title as the three panels above say
  theirs. The steppers have no shortcut, as the settings panel's steppers have
  none.
- **The level is read with `std::fs` and the handle loaded from the asset
  server**: the editor needs the level this frame, where the asset arrives
  whenever it arrives - and it is the handle that carries the path a save writes
  back to and that `c0011` watches for a hand edit. A freshly loaded asset raises
  `Added`, not `Modified`, so opening does not trip `editor_watch_the_file`.
- **The name has the whole row, and the steppers sit under it.** Measured in the
  running game: with `<` `>` either side, `demo_minimal_win_state_error` lays out
  at 272 pixels of Orbitron in the 264 the row could spare and overflowed onto
  the stepper. Across the full `ROW_WIDTH` it has 52 pixels to spare. Everything
  else measured on screen too - title 215/324, "2 of 11" 49/88, "editing
  levels/level0.ron" 201/324, `Open` 44/84 - and the longest thing the message
  row can say (`editing levels/demo_minimal_win_state_error.ron`, ~385) is what
  the two reserved rows under the buttons are for.
- **Vertical budget**: the panel is 146 tall and the column reaches 726 of 800 on
  entry, 778 after a save with no complaints. A save carrying a complaint pushes
  the panel's reserved bottom row past the window edge - the buttons and the
  message stay on screen, and the panel above it was already the one running out
  of room first.
- **`editor_choose_click` and `editor_choose_shortcut` share an `Editing`
  `SystemParam`** - the level plus the six resources an arriving one displaces -
  so the two systems and the two functions under them name them once.
- 33 new tests, 311 green, `cargo build` at 16 warnings and `cargo clippy` clean
  for the new file, both all pre-existing elsewhere. 20 pure (the listing, the
  stepping, the wrap, the empty directory, the wording, the panel's geometry) and
  13 driving a real app against real files in a scratch directory.
- Seven mutations checked to fail: the panel not excluded from picking, opening
  keeping the history, entering not pointing the chooser at the level under edit,
  a failed open replacing the level anyway, the list never being read again after
  a save, `New` keeping the old file, and the two shortcuts.
- **Checked in the running game** for the text measurements above, through a
  temporary system reading `TextLayoutInfo` (reverted). As with `c0014`,
  `screencapture` cannot reach the display from this session, so how it *looks*
  rests on those numbers; and the mouse cannot be driven from here, so the
  clicking is covered by the app-driven tests rather than by hand.

## Log

- 2026-08-26 status → ready (app)
- 2026-08-26 status → in-progress (agent)
