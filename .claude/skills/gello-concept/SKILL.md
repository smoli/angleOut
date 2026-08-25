---
name: gello-concept
description: Turn a project concept into a gello board's epics. Reads .gello/concept.md (or interviews the human and writes one), proposes the epic breakdown, and only on approval creates each epics/eNN-slug/epic.md. Use when starting a new project on gello, or to turn a concept into epics.
---

# Turn a concept into gello epics

gello is a Markdown-native Kanban board: every card is one `.md` file under
`.gello/` with YAML frontmatter. This skill is the first step of the greenfield
flow — **concept → epics → `gello-plan` → cards**. It stops at epics: one
`epics/eNN-<slug>/epic.md` per epic, no cards.

**Two phases, human-gated: propose → approve → create.** Nothing is written
under `epics/` before the human approves the breakdown.

## 1. Ensure a board

If `.gello/` is missing, scaffold it — the same tree the gello app creates:

```
.gello/
  board.yaml
  concept.md          # placeholder, filled in step 2
  assets/.gitkeep
  epics/.gitkeep
  cards/.gitkeep      # epic-less standalone cards
```

`board.yaml` is exactly:

```yaml
columns: [inbox, discuss, backlog, ready, in-progress, review, done]
types: [task, issue]
wip_limits:
  in-progress: 2
```

Then add the convention block to the project's agent instructions: append it to
`CLAUDE.md` (create the file with a `# CLAUDE.md` heading if absent), and to
`AGENTS.md` only if that file already exists. Skip a file that already contains
the `<!-- gello-convention -->` marker.

````markdown
<!-- gello-convention -->
## Working the gello board

This project uses **gello** — a Markdown-native Kanban board in `.gello/`.
The files are the single source of truth; cards are `.md` files with YAML
frontmatter. Read `.gello/concept.md` for the product spec.

- **Query the board** (never read all cards to find one):
  ```bash
  grep -rl "^status: ready" .gello/cards .gello/epics --include="[ci][0-9]*.md"
  grep -rh "^status:" .gello/cards .gello/epics --include="[ci][0-9]*.md" | sort | uniq -c
  ```
- **Pick up work**: re-query the board from disk first, then take the
  top `ready` card whose `depends` are all `done`; set
  `status: in-progress` before starting.
- **Finish**: set `status: review` (only a human moves cards to `done`).
- **New ideas**: capture a card in `.gello/cards/` with `status: inbox` — a
  heading and a sentence. (Inbox is a status, the first column — not a folder.)
- **Triage**: move a card into an epic (`epics/eNN-name/`) or leave it
  standalone in `.gello/cards/`; `tags:` are the separate cross-cutting axis.
- **Archive**: long-done cards can be archived into an `archive/` folder in
  their own home; they keep their id and epic. Add `--exclude-dir=archive` to
  a board query to leave them out.
- Valid statuses come from `board.yaml`; frontmatter must be valid YAML.
````

Skip this whole step when `.gello/board.yaml` is already there.

## 2. Get the concept

`.gello/concept.md` is the spec every epic traces back to.

- **It exists with real content** → read it, and work from it. Never overwrite
  or rewrite it. Ask about gaps instead; only edit it if the human asks.
- **It is missing, empty, or still the scaffold placeholder** → interview the
  human, one topic at a time: what the thing is, who it is for, the problem it
  solves, the shape of the product, the constraints, and what is explicitly out
  of scope. Then write `.gello/concept.md` from the answers and have the human
  confirm it before decomposing.

## 3. Propose the epics — write nothing

Draft the breakdown and present it in chat:

- a handful of epics (4–8 is usual), each a large effort worth many cards
- one line of goal per epic, in the concept's own words
- which concept.md sections each epic covers
- the order to work them in, and what depends on what
- a coverage check: every part of the concept lands in an epic, or is named as
  out of scope

Ask for approval and adjust until the human agrees. Create no files here.

Epics carry no order or dependency field — sequencing lives in this proposal
and in each Goal; card `depends` do the wiring later, when `gello-plan` runs.

## 4. Create the epics — only on approval

```bash
grep -rh "^id: " .gello/epics/*/epic.md    # ids already taken
```

Allocate the next free `eNN` (`e` + 2 digits, sequential). **Never reuse or
renumber an existing epic id**, even one whose folder is gone.

Write one `.gello/epics/eNN-<slug>/epic.md` per approved epic, with a short
kebab-case `<slug>` from the title:

```markdown
---
id: e03
title: Board rendering
status: backlog
---

## Goal

What this epic delivers and why, in the concept's terms.

## Definition of done

- [ ] checkable outcomes, each traceable to a concept.md section
```

Every new epic is `status: backlog`. Touch no existing epic, and create no
card files.

## 5. Hand off

Tell the human which epics now exist, then stop. `gello-plan` is what breaks
one epic into cards — run it per epic, in the proposed order.
<!-- gello-managed v5 r823hb -->
