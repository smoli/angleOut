---
id: c0012
title: Save to disk with validation warnings
status: backlog
epic: e01
depends: [c0008, c0010]
created: 2026-08-25
updated: 2026-08-25
status-changed: 2026-08-25T22:58:50
---

## What

Write the edited level back out as RON, and warn about structural mistakes on the
way — without ever refusing to save.

## Acceptance criteria

- [ ] Saving writes the level as RON with `std::fs` (the asset server is read-only).
- [ ] Saving does not trigger a hot-reload that discards in-editor state.
- [ ] The saved file reloads to an identical level.
- [ ] A newly created level can be appended to `assets/levels/campaign.ron` from the editor.
- [ ] Saving warns, but never blocks, on: a trigger receiver with no matching trigger in its group; a portal with no trigger; a level with no breakable blocks.
- [ ] Warnings are visible in the editor rather than only in the log.

## Notes

- The self-inflicted hot-reload is the real hazard: `c0004` watches
  `assets/levels/`, so the editor's own write comes straight back as an asset
  change. Ignore the reload for the file we just wrote, or suppress the watcher
  across the write.
- Trigger semantics live in `src/block/trigger.rs`: receivers (`R`, `S`) and
  triggers (`A`, `B`, `C`) pair up by `TriggerGroup`. A receiver whose group has
  no trigger can never activate.

## Log

- 2026-08-25 created from the e01 epic breakdown
