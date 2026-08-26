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

- [ ] The panel names a level file on disk, and the steppers walk the whole
      directory - every `*.ron` in it except the campaign index, in a stable
      order, wrapping at both ends.
- [ ] Entering the editor points the chooser at the level under edit, so the
      panel opens naming what the author is already working on.
- [ ] `Open` puts the named file's level in front of the editor: its blocks, its
      settings, and its path as what the next save writes back to.
- [ ] `New` starts a blank grid that belongs to no file, exactly as an editor
      that found no level to open does.
- [ ] Both discard whatever was being edited, without asking, and the panel says
      what arrived - including when the file could not be read, which leaves the
      level under edit alone.
- [ ] Opening or starting a level drops the undo history: every entry in it is a
      level that belonged to a different file.
- [ ] `Ctrl+O` and `Ctrl+N` do the same as the two buttons.
- [ ] A click anywhere on the panel is aimed at the panel, never at the grid cell
      behind it.
- [ ] `cargo build` is clean and `cargo test` passes.

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

## Log

- 2026-08-26 status → ready (app)
- 2026-08-26 status → in-progress (agent)
