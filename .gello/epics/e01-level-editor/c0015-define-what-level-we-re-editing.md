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
