---
name: gello-discuss
description: Interview the human about a gello board card flagged `status: discuss` and write the refined outcome back into the card. Use when asked to discuss a gello card, or when picking up work and only discuss-flagged cards remain.
---

# Discuss a gello card

gello is a Markdown-native Kanban board: every card is one `.md` file under
`.gello/` with YAML frontmatter (`id`, `title`, `status`, `epic`, …).
The `discuss` status means the human wants to think a card through with you
*before* it becomes implementable — usually a raw inbox idea.

## Find discuss cards

```bash
grep -rl "^status: discuss" .gello/inbox .gello/epics .gello/cards --include="[ci][0-9]*.md"
sed -n '/^---$/,/^---$/p' <card-file>   # one card's frontmatter
```

## Flow

1. **Pick the card** — the passed card ID, else list discuss cards and ask.
2. **Read it** — the whole file: What, any existing notes, its origin.
3. **Interview the human** — one focused topic at a time: goal, scope,
   constraints, edge cases, what "done" looks like, rejected alternatives.
   Do not write anything yet.
4. **Write the outcome back into the card**, preserving untouched lines
   byte-for-byte (surgical edits, valid YAML):
   - a refined `## What`
   - a drafted `## Acceptance criteria` (each a checkable, testable line)
   - a compact `## Discussion` — key decisions, rejected alternatives, open
     questions. No verbatim transcript.
5. **Offer triage** — assign to an epic / `cards/` / `backlog` / `ready`. Only
   the human decides the exit; perform any move only on explicit confirmation.

## Triage move (only on the human's say-so)

Moving a card between folders is a file move plus a status edit:
- inbox → epic: move `.gello/inbox/<card>.md` to
  `.gello/epics/<eNN-slug>/<card>.md`, set `epic:`, and rewrite relative
  asset links (`](../assets/` → `](../../assets/`).
- inbox → standalone: move to `.gello/cards/<card>.md` (same depth, no link
  rewrite); leave `epic` unset.
- Set `status` and `status-changed` (local ISO datetime) on any status change,
  **replacing** the existing lines — never add a second `status-changed:` (a
  duplicate frontmatter key makes the card unparseable). One line per key.
- Never reuse or renumber existing card IDs.
<!-- gello-managed v5 79i187 -->
