---
name: gello-plan
description: Break a gello epic into dependent child cards. Interview the human about the epic, draft a stepwise plan + dependency graph into epic.md, and only on approval create the wired child cards. Use when asked to plan or break down an epic.
---

# Plan a gello epic into cards

gello is a Markdown-native Kanban board: every card is one `.md` file under
`.gello/` with YAML frontmatter. An **epic** (`epics/eNN-<slug>/`) is a large
effort broken into dependent child cards. This skill turns an epic's goal into
that breakdown — **two phases, human-gated: plan → approve → create**. Nothing
is created before the human approves.

## Board format you're writing into

```
epics/eNN-<slug>/
  epic.md            # id: eNN, title, status; ## Goal, ## Definition of done
  c001-<slug>.md     # child cards, flat within the epic
```

Child-card frontmatter: `id` (per-board sequential — `c`+4 digits for tasks,
`i`+4 for issues; never reused, renumbered, or duplicated), `title`,
`status: backlog`, `epic: eNN`, `depends: [<ids>]`, `created`, `updated`.
Bodies use `## What`, `## Acceptance criteria` (`- [ ]`, each testable),
`## Notes`, `## Log`.

## Find the epic and the next free ids

```bash
grep -rh "^id: " .gello/epics/*/epic.md                     # existing epics
sed -n '/^---$/,/^---$/p' .gello/epics/<eNN-slug>/epic.md    # the epic's goal
grep -rhoE "^id: [ci][0-9]+" .gello/inbox .gello/epics .gello/cards | sort -u  # taken ids
```

## Phase 1 — interview + plan (write only epic.md)

1. Read `epic.md` — its Goal and Definition of done — and any related cards.
2. Interview the human, one topic at a time: scope boundaries, constraints,
   the smallest shippable slice, what is explicitly out.
3. Draft a `## Plan (steps + dependencies)` section into `epic.md` with a
   surgical edit (preserve untouched lines byte-for-byte, valid YAML): an
   ordered list of proposed child cards, each a one-line scope plus its
   dependencies (e.g. `3. Card — … (← step 1, step 2)`). Number steps so the
   dependency graph reads top-down (root steps first). **Create no card files.**

## Phase 2 — create the child cards (only on explicit approval)

4. Present the plan and ask for approval. Change nothing until the human
   approves or adjusts it — this is the single checkpoint.
5. On approval, create one card file per step in `epics/eNN-<slug>/`:
   - allocate fresh sequential ids (never reuse or renumber existing ones),
   - set `epic: eNN`, `status: backlog`, and `depends:` wired to the ids of
     the steps it follows,
   - write a `## What`, a drafted `## Acceptance criteria`, and a `## Log`
     line citing "created from the eNN epic breakdown".
6. Leave every card in `backlog`; the human moves roots to `ready` when it's
   time to work. **Never** set `in-progress`.

## Rules

- Two-phase always: nothing is created before the human approves the plan.
- Ids are per-board and unique; existing ids never change.
- Surgical frontmatter edits; only statuses from `board.yaml`.
<!-- gello-managed v5 qlhfza -->
