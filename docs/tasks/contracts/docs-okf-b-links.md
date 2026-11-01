---
type: Task Contract
title: "Contract okf-b — docs: turn path references into OKF links"
description: "Inside the bundle (docs/), a reference to another bundle file becomes a standard markdown link with a bundle-absolute target (OKF v0.2 §6.1."
status: deprecated
tags: [okf]
---
# Contract okf-b — docs: turn path references into OKF links

Planner: Claude. Status: **Implemented 2026-10-31**. Tier: presentation (no biology changes).
Step 2 of 3 of the OKF reorganization; okf-a (`docs-okf-a-move.md`) is done.

## 1. What this contract builds

Inside the bundle (`docs/`), a reference to another bundle file becomes a
standard markdown link with a bundle-absolute target (OKF v0.2 §6.1: a
target that starts with `/` is relative to the bundle root, `docs/`). The
link text keeps the repository path, so a reader who works from the
repository root still sees where the file is:

```
before:  see `docs/specs/cli.md` §10
after:   see [docs/specs/cli.md](/specs/cli.md) §10
```

## 2. Files the worker may touch

1. `target/tmp/okf-links.py` — new: the conversion script.
2. Every `*.md` file under `docs/`, except those in §6 — change: only the
   conversions in §3.

## 3. The conversion rules

Work line by line. Skip lines inside fenced code blocks (opened and closed by
a line starting with ```` ``` ```` or `~~~`). On the other lines:

1. **Code span that is only a path.** `` `docs/P` `` where the whole span is
   `docs/P` → `[docs/P](/P)`.
2. **Bare path in prose** (not inside a code span, not already inside a
   markdown link's text or target): `docs/P` → `[docs/P](/P)`. Trailing
   punctuation (`.`, `,`, `;`, `:`, `)`) is not part of the path.
3. Convert only when `docs/P` names a file or a directory that exists now.
   A directory keeps its trailing slash: `docs/decisions/` →
   `[docs/decisions/](/decisions/)`.
4. Leave everything else unchanged. In particular: code spans that hold more
   than the path (`` `docs/specs/cli.md §10` ``), paths with placeholders
   (`<n>`, `*`, `{}`), paths that do not exist, and paths outside `docs/`
   (`crates/...`, `tests/...`, `scripts/...`).

The script must be idempotent: a second run changes nothing.

## 4. Checks

`scripts/check`, plus:

```bash
# 1. Idempotent: no output after a second run.
python3 target/tmp/okf-links.py && git diff --stat > target/tmp/a.txt \
  && python3 target/tmp/okf-links.py && git diff --stat | diff - target/tmp/a.txt
# 2. Only link syntax changed: with links folded back to code spans, the
#    diff against HEAD is empty for every changed file. Write this check as
#    part of the script (--verify flag) and quote its output.
python3 target/tmp/okf-links.py --verify
# 3. Every link target exists: no output.
grep -rhoE '\]\(/[^)]*\)' docs | sed -E 's/^\]\(\/(.*)\)$/\1/' | sort -u \
  | while read -r p; do [ -e "docs/$p" ] || echo "missing: $p"; done
```

## 5. Proving the change bites

| # | Change | Must fail |
| --- | --- | --- |
| 1 | Make the script skip rule 3's existence test | check 3 lists at least one `missing:` path |

Restore and report the failure lines.

## 6. Do not

- Edit `docs/agents/worker.md`, `docs/tasks/TEMPLATE.md`, or
  `docs/tasks/contracts/docs-okf-*.md`.
- Edit any file outside `docs/` (code comments, tests, `README.md`,
  `AGENTS.md`, `CLAUDE.md` keep plain paths).
- Link D-numbers, section signs (§) or bare file names without `docs/`.
- Add frontmatter or `index.md`/`log.md` files (okf-c).
- If a `crates/` test fails because of a changed `docs/specs/cli.md` line,
  stop and report the test and the line; do not edit the test.

## 7. Report

The standard handoff, plus: the number of links made per rule (1 and 2), the
list of paths left unconverted under rule 4 with `file:line` (group repeats),
the output of the three checks, and the guard experiment.
