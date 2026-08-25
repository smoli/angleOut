---
id: e01
title: Level Editor
status: backlog
---

## Goal

Author levels for Angle Out inside the game with the mouse, instead of
hand-writing Rust literals in `main.rs`.

Levels become data: one RON file per level under `assets/levels/`, covering the
whole `LevelDefinition`, with the block layout kept as the existing multi-line
ASCII token grid embedded inside it — so a level stays readable and diffable by
hand while also being editable by mouse.

The editor is reachable from the main menu, paints the block grid cell by cell,
exposes the non-grid level settings in a panel, supports undo/redo, and can
playtest the level being edited and return to it with unsaved edits intact.

**Not in this epic:** free-floating `LevelObstacle`s (`Box`, `ForceField`,
`DirectionalDeathTrigger`). They are positioned by raw `Vec3` rather than grid
cells and need drag handles, which is a separate problem.

## Definition of done

- [ ] No level data remains in `main.rs`; every shipped level lives in `assets/levels/*.ron`, campaign order comes from `assets/levels/campaign.ron`, and the game plays them all exactly as it does today.
- [ ] A level round-trips through disk: load → edit → save → reload reproduces the same level, and hand-editing a file hot-reloads it into a running game.
- [ ] An existing hand-written level (e.g. `LEVEL4`) can be rebuilt from scratch in the editor using the mouse, saved, and played.
- [ ] The editor covers block type, behaviour, trigger type and trigger group on the grid, plus background asset, scroll velocity, simultaneous balls, win criteria, global pickups and both side walls.
- [ ] Undo and redo cover both cell edits and settings changes.
- [ ] Playtesting from the editor enters a match and returns to the editor with unsaved edits intact.
- [ ] Cell↔world conversion has a single implementation shared by the editor, `generate_block_grid` and `interpret_grid`.
- [ ] `cargo build` is clean and `cargo test` still passes.

## Plan (steps + dependencies)

Foundation (steps 1–3) is split so each lands as its own reviewable change.
Steps 1 and 2 are both roots and share no code. Everything after is a chain.

1. **Shared cell↔world conversion** `c0002` — one function pair (cell→world, world→cell)
   replacing the origin and odd/even-centring maths currently duplicated between
   `generate_block_grid` and `interpret_grid`. Pure refactor, no behaviour
   change, unit-tested. (no deps)
2. **RON level format + campaign index + migrate levels off `main.rs`** `c0003` — serde
   derives across `LevelDefinition` and its field types, block layout kept as the
   embedded token string, `distributed_global_pickups` skipped; the 8 levels move
   to `assets/levels/*.ron` with order in `campaign.ron`; read once at startup.
   (no deps)
3. **Asset-driven loading + hot reload** `c0004` — custom `AssetLoader` for level files;
   `Levels` becomes handle-driven, touching every `get_current_level()` caller;
   hand edits hot-reload into a running game. (← step 2)
4. **Editor state, menu entry, cursor** `c0005` — `GameState::Editor`, an "Editor" item
   in the main menu, cursor made visible on enter and hidden on exit, editor
   level held in a resource so it survives state changes. (← step 3)
5. **Grid picking** `c0006` — ray from cursor to the y=0 ground plane, quantised to a
   cell via step 1's conversion, with the hovered cell highlighted. Works on
   empty cells, so no mesh picking. (← step 1, step 4)
6. **Grid painting** `c0007` — brush model (type × behaviour × trigger × group, plus
   erase), click to place and clear, `Z`/`BlockType::Obstacle` included.
   (← step 5)
7. **Grid resize** `c0008` — add and remove rows and columns from the edges; the writer
   pads rows to equal width so `interpret_grid`'s first-line column count stays
   correct. (← step 6)
8. **Palette UI + shortcuts** `c0009` — clickable entries showing each block's colour
   with its format letter, behaviours as text labels, every entry bound to the
   matching letter key. (← step 6)
9. **Settings panel** `c0010` — background asset, scroll velocity, simultaneous balls,
   win criteria, global pickups, both side walls. (← step 4)
10. **Undo/redo** `c0011` — command stack covering cell edits and settings changes;
    history invalidated if the file changes on disk underneath. (← step 6,
    step 9)
11. **Save + validation warnings** `c0012` — write RON with `std::fs`, guard against the
    resulting hot-reload clobbering in-editor state, and warn (never block) on
    orphaned trigger receivers, triggerless portals and levels with no breakable
    blocks. (← step 7, step 9)
12. **Playtest round trip** `c0013` — jump from editor into a match and back with unsaved
    edits intact, surviving `match_despawn` and the `OnExit(PostMatch)` teardown.
    (← step 6, step 9)

Linear working order is 1 → 12 as numbered.

## Design decisions

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

**Superseded breakdown** (see the Plan below)

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

_Carried over from card c0002, which was rewritten as step 1 of the plan._
