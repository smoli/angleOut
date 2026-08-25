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

1. **Shared cell↔world conversion** — one function pair (cell→world, world→cell)
   replacing the origin and odd/even-centring maths currently duplicated between
   `generate_block_grid` and `interpret_grid`. Pure refactor, no behaviour
   change, unit-tested. (no deps)
2. **RON level format + campaign index + migrate levels off `main.rs`** — serde
   derives across `LevelDefinition` and its field types, block layout kept as the
   embedded token string, `distributed_global_pickups` skipped; the 8 levels move
   to `assets/levels/*.ron` with order in `campaign.ron`; read once at startup.
   (no deps)
3. **Asset-driven loading + hot reload** — custom `AssetLoader` for level files;
   `Levels` becomes handle-driven, touching every `get_current_level()` caller;
   hand edits hot-reload into a running game. (← step 2)
4. **Editor state, menu entry, cursor** — `GameState::Editor`, an "Editor" item
   in the main menu, cursor made visible on enter and hidden on exit, editor
   level held in a resource so it survives state changes. (← step 3)
5. **Grid picking** — ray from cursor to the y=0 ground plane, quantised to a
   cell via step 1's conversion, with the hovered cell highlighted. Works on
   empty cells, so no mesh picking. (← step 1, step 4)
6. **Grid painting** — brush model (type × behaviour × trigger × group, plus
   erase), click to place and clear, `Z`/`BlockType::Obstacle` included.
   (← step 5)
7. **Grid resize** — add and remove rows and columns from the edges; the writer
   pads rows to equal width so `interpret_grid`'s first-line column count stays
   correct. (← step 6)
8. **Palette UI + shortcuts** — clickable entries showing each block's colour
   with its format letter, behaviours as text labels, every entry bound to the
   matching letter key. (← step 6)
9. **Settings panel** — background asset, scroll velocity, simultaneous balls,
   win criteria, global pickups, both side walls. (← step 4)
10. **Undo/redo** — command stack covering cell edits and settings changes;
    history invalidated if the file changes on disk underneath. (← step 6,
    step 9)
11. **Save + validation warnings** — write RON with `std::fs`, guard against the
    resulting hot-reload clobbering in-editor state, and warn (never block) on
    orphaned trigger receivers, triggerless portals and levels with no breakable
    blocks. (← step 7, step 9)
12. **Playtest round trip** — jump from editor into a match and back with unsaved
    edits intact, surviving `match_despawn` and the `OnExit(PostMatch)` teardown.
    (← step 6, step 9)

Linear working order is 1 → 12 as numbered.
