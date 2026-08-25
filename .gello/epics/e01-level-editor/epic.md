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
