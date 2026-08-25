---
name: gello-review
description: Review a gello card in `review` — verify its acceptance criteria against the code, run the repo's tests, lint and typecheck, read the diff, and record a pass/fail verdict on the card. Use when asked to review a card or to check an agent's finished work before signing it off.
---

# Review a gello card

gello is a Markdown-native Kanban board: every card is one `.md` file under
`.gello/` with YAML frontmatter. A card in `review` is a claim, not a fact —
the agent that implemented it also moved it there. Decide whether the claim
holds and record the decision on the card.

You verify, you do not implement. Do not edit, change or fix the code, do not
tick the card's criteria, and do not commit — a failed card goes back to its
implementer with your notes.

## Pick the card

The card you were pointed at. With none named, list the candidates and ask:

```bash
grep -rl "^status: review" .gello/cards .gello/epics --include="[ci][0-9]*.md"
```

## The checks

1. **Read the card** — `## What`, `## Acceptance criteria`, `## Notes`,
   `## Log`. The What is the scope; the criteria are the spec.
2. **Read the diff** — the changes made for this card (`git log`, `git show`,
   `git diff` on the commits naming the card id). It has to match the What: no
   unrelated scope, no leftover debug code, no test weakened, skipped or
   `.only`'d to reach green.
3. **Verify every acceptance criterion** against the code, not against its
   checkbox — a ticked box with nothing behind it is an unmet criterion. Where
   the repo works test-first, a criterion no test covers is unmet.
4. **Run the repo's checks** — tests, lint, typecheck. Take the commands from
   the repo itself (CLAUDE.md, README, the package manifest); do not assume
   them. A check you could not run is not a pass; record that you could not run
   it.

## The verdict

It is a fail if any criterion is unmet, any check is red, the diff reaches
beyond the card's What, or a test was weakened to pass. Otherwise a pass.

Record it in a `## Review` section on the card (create it above `## Log` if
there is none), one entry per round, newest last:

```markdown
## Review

### 2026-08-08T21:04:11 — fail

Checked: acceptance criteria, tests, lint, typecheck, diff.

- Criterion "a parked run frees its WIP slot" is unmet: `planDispatch` still
  counts a parked run.
- Tests red: 2 failures in `companion/runner.test.ts`.
```

The heading is `### <local ISO datetime> — <pass|fail>`; the verdict word ends
that line and is what the board and the companion read. Under it, one line
naming what you checked, then the reasons — each concrete enough to act on,
naming the criterion, file or command. On a pass, the reasons are what you
verified.

Write the entry and stop. Do not move the card — the verdict is the whole
product of a review, and it is what everything else reads:

- **Pass** — the card stays in `review`, vetted. That verdict is the
  go-signal: a human takes the card to `done`, and the companion lets the
  cards depending on it start.
- **Fail** — the card stays in `review` too, and the notes are for the
  implementer, who gets it back to fix.
<!-- gello-managed v5 xwd7ob -->
