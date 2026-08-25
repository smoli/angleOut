---
id: c0002
title: Interactive level editor
status: discuss
created: 2026-08-25
updated: 2026-08-25
status-changed: 2026-08-25T22:37:48
epic: e01
---

## What

An in-game, mouse-driven level editor, with levels persisted as text files.

A level file is **RON covering the whole `LevelDefinition`, with the block
layout kept as the existing multi-line token grid embedded inside it** — so the
ASCII map stays readable and hand-editable while the other ten fields become
expressible on disk for the first time.

Campaign order lives in an index file, `assets/levels/campaign.ron`, so levels
can exist off-campaign as scratch and the editor can append a new one.

The editor is reachable from the main menu, paints the block grid by clicking
cells (including growing and shrinking the grid), and has a settings panel for
the non-grid fields (background, scroll
velocity, simultaneous balls, win criteria, global pickups, side walls).
Free-floating `LevelObstacle`s are **out of scope** — they need drag handles and
are a separate problem. Undo/redo is in scope. You can playtest the level you are
editing and come back with unsaved edits intact.

This is epic-sized, not a single card — see the suggested breakdown below.

## Acceptance criteria

- [ ] `LevelDefinition` and its field types round-trip through RON, with the block layout stored as the existing multi-line token string.
- [ ] Shipped levels live in `assets/levels/*.ron`; `main.rs` no longer contains level literals.
- [ ] Campaign order is read from `assets/levels/campaign.ron`, and the editor can append a newly created level to it.
- [ ] Hand-editing a level file while the game is running hot-reloads it.
- [ ] An "Editor" entry in the main menu enters a `GameState::Editor`; the mouse cursor becomes visible there and is hidden again on leaving.
- [ ] Clicking a grid cell places the current brush and the erase brush clears it — including on empty cells, so placement uses a ray/ground-plane hit rather than mesh picking.
- [ ] A palette shows block type, behaviour, trigger type and trigger group, is fully clickable, and each entry has a keyboard shortcut matching its letter in the file format.
- [ ] Palette entries show the block's actual colour with its format letter on it; behaviour entries carry a text label.
- [ ] `Z` / `BlockType::Obstacle` is available as a brush.
- [ ] The editor can add and remove grid rows and columns.
- [ ] Saving warns — but never refuses — on trigger receivers with no matching trigger in their group, portals with no trigger, and levels with no breakable blocks.
- [ ] A settings panel edits background asset, scroll velocity, simultaneous balls, win criteria, global pickups and both side walls.
- [ ] Undo and redo cover both cell edits and settings changes.
- [ ] Saving writes a RON file that reloads to an identical level.
- [ ] Playtesting from the editor enters a match and returns to the editor with unsaved edits intact.
- [ ] An existing hand-written level (e.g. `LEVEL4`) can be rebuilt from scratch in the editor, saved, and played.
- [ ] `cargo build` is clean and `cargo test` still passes.

## Discussion

**Decisions**

- Scope is the block grid plus the scalar level settings. `LevelObstacle`
  placement is explicitly excluded.
- Format is RON for the whole `LevelDefinition` with the token grid embedded as a
  multi-line string. `serde` and `ron` are already in the lock file.
- Entry is a main menu item and a new `GameState::Editor` — the unused "Settings"
  slot already sits next to it.
- Files live in `assets/levels/`, loaded through the asset server so hot reload
  picks up hand edits.
- Playtest is a full round trip: edit → play → back, edits preserved.
- Palette is clickable *and* keyboard-shortcutted on the format's own letters.
- Undo/redo is required.
- Done means authoring one of the existing levels end to end, which implies the
  migration off `main.rs` is part of the epic rather than a follow-up.
- The grid is resizable by rows and columns. The shipped levels run from 3 to 11
  columns wide, so a fixed extent could not express them all.
- `Z` / `BlockType::Obstacle` is a normal brush. Despite the name it is a grid
  cell that happens to be unbreakable and excluded from the win count — unrelated
  to `LevelObstacle`, so excluding those from the epic does not exclude this.
  `LEVEL4` needs it.
- Saving validates structurally and warns without ever blocking: receivers with
  no matching trigger in their group, portals with no trigger, levels with no
  breakable blocks.
- Campaign order comes from `assets/levels/campaign.ron`. Levels not listed are
  scratch, and the editor can append to it.
- Palette entries are a colour swatch carrying the format letter; behaviour needs
  a text label since it does not affect colour.

**Rejected**

- Grid-only editing with settings left in Rust — too little to be worth the
  editor.
- Plain `.txt` token grids — cannot express the other ten `LevelDefinition`
  fields.
- Fully structured RON, or JSON — both lose the at-a-glance ASCII map.
- Dev-only launch flag, or a cargo-feature-gated menu entry.
- Save-and-restart, or a one-way jump into play that discards unsaved edits.
- A separate user data directory for authored levels.
- Keyboard-only brush selection, or a palette with no shortcuts.
- No undo.
- A fixed grid extent, whether chosen at creation or global to all levels.
- Blocking saves on validation failure, and skipping validation entirely.
- Filename-prefix ordering, and a per-level `order` field.
- Text-only palette entries, and rendered 3D block previews.

**Suggested breakdown**

1. RON level format + serde derives; campaign index; migrate the hardcoded levels out of `main.rs`.
2. `AssetLoader` + hot reload; `Levels` becomes handle-driven.
3. `GameState::Editor`, menu entry, cursor visibility, grid picking.
4. Grid painting + brush model + grid resize.
5. Palette UI + shortcuts.
6. Settings panel.
7. Undo/redo command stack.
8. Save to disk + structural validation warnings.
9. Playtest round trip.

**Implementation notes**

- `distributed_global_pickups` is *derived* at runtime by
  `distribute_global_pickups`, not authored — it must be `#[serde(skip)]`.
- `Levels` currently owns `Vec<LevelDefinition>`. Hot reload means it has to
  become handle-driven (`Vec<Handle<...>>`), which touches every
  `get_current_level()` caller.
- Saving cannot go through the asset server (read-only) — it needs `std::fs`.
  Writing into `assets/levels/` will then trigger our own hot-reload watcher, so
  a self-inflicted reload must not clobber in-editor state.
- The cursor is hidden globally via `primary_cursor_options` in `main.rs`; the
  editor has to flip `CursorOptions.visible` on the primary window on enter/exit.
- Cell↔world maths is currently **duplicated** between `generate_block_grid` and
  `interpret_grid` (same origin `-30.0 - 4.0 * (BLOCK_DEPTH + gap)`, same
  odd/even column centring). The editor needs one shared conversion or the three
  will drift apart.
- `interpret_grid` takes its column count from the first line only, so a ragged
  grid silently misaligns. The writer should always pad rows to equal width.
- The playtest round trip has to survive `match_despawn` and the
  `OnExit(PostMatch)` teardown, so the edited level must live in a resource, not
  in entities.
- Undo history should be invalidated if the file changes on disk underneath the
  editor.

**Open questions**

None — all resolved in discussion.

**Origin**

> I want to be able to create levels in game using the mouse.
>
> Levels files should be saved as text files.

## Log

- 2026-08-25 status → discuss (app)
