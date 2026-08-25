# CLAUDE.md

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
