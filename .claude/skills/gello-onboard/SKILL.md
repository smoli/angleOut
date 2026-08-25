---
name: gello-onboard
description: Migrate an existing project's task organisation (TODO.md, plan files, docs, issue lists) onto a gello board, preserving history. Use to onboard a legacy project to gello.
---

# Onboard a project to gello

gello is a Markdown-native Kanban board: each card is one `.md` file under
`.gello/` with YAML frontmatter. Your job is to migrate whatever planning the
project already has onto the board — **safely, completely, and only after the
human approves the plan**.

## Board format (target)

```
.gello/
  board.yaml                 # columns: [backlog, ready, in-progress, review, done]
  concept.md                 # long-form product concept (optional)
  inbox/                     # unassigned ideas
  epics/e01-<slug>/
    epic.md                  # id, title, status
    c001-<slug>.md           # cards, flat within their epic
  cards/                     # epic-less standalone cards (bugs, small changes)
    c002-<slug>.md
```

Card frontmatter: `id` (per-board sequential, `c`+4 digits, never reused or
duplicated), `title`, `status` (only values from board.yaml), `epic` (the
`eNN` id; omit for a standalone card in `cards/`), `created`, `updated`,
optional `tags`. Card bodies use `## What`, `## Acceptance criteria`
(`- [ ]`), `## Notes`, `## Log`.

## Pre-flight (do this first, always)

1. **Require a clean git working tree.** Run `git status --porcelain`; if it
   is non-empty, warn the human and **stop** — write nothing, create nothing.
   A clean tree makes the whole migration one reviewable, revertable diff.

## Flow: inventory → propose → confirm → write

2. **Inventory** the project's planning artifacts, format-agnostically: plan
   / TODO / roadmap markdown, task lists, `docs/` folders, exported issue
   lists, anything that encodes work. Do not assume a specific format.
3. **Propose a mapping** and present it for approval — this is the reviewable
   artifact, and it must be **complete** (never sampled or truncated, however
   large):
   - source structure / phases → **epics** (a small change with no phase can be
     a standalone card in `cards/`)
   - completed items → `done` cards (keeps throughput history on the board)
   - active / next work → `ready` at most (**never** `in-progress` — WIP is
     claimed by whoever actually works it)
   - ambiguous items → `inbox` as `backlog`
   List every item with its target status. For a huge backlog, write the
   proposal to a file rather than chat, but keep it complete.
4. **Confirm** — make no board writes until the human approves or adjusts the
   mapping at this single checkpoint.
5. **Write** the board: create epic folders + cards with sequential,
   unique IDs and valid frontmatter.

## History & provenance

- Recover `created` / `updated` from the git history of the source files
  where possible (`git log --follow --format=%ad --date=short <file>`); else
  use today.
- Every migrated card gets a `## Log` line citing its origin (source file /
  item / issue number).
- **Never edit, move, or delete the legacy source files.** They are read-only.

## Concept folding

- If a legacy vision / spec / design doc exists, offer to synthesize
  `.gello/concept.md` from it — leave the original in place.

## Finish

- Write `.gello/migration.md`: a list of every legacy file made obsolete by
  the migration, stating explicitly that **removing them is the human's
  decision, not yours**.
- Validate: every card parses, statuses are all from board.yaml, and no two
  cards share an id.
<!-- gello-managed v5 1pvcao2 -->
