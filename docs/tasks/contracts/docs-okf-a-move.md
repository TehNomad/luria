---
type: Task Contract
title: "Contract okf-a — docs: move files into the OKF layout and split the decisions"
description: "docs/ becomes the bundle root."
status: deprecated
tags: [okf]
---
# Contract okf-a — docs: move files into the OKF layout and split the decisions

Planner: Claude. Status: **Implemented 2026-10-31**. Tier: presentation (no biology changes).
Plan: reorganize `docs/` as an Open Knowledge Format (OKF) v0.2 bundle.
This contract is step 1 of 3 (okf-a moves, okf-b links, okf-c frontmatter
and index files). Do not do the work of okf-b or okf-c here.

## 1. What this contract builds

`docs/` becomes the bundle root. Files move to new paths, `docs/decisions.md`
splits into one file per decision, every reference to an old path changes to
the new path, and `scripts/decision` reads the new directory with the same
output as before. No text changes except paths.

## 2. Moves (use `git mv` for each)

| From | To |
| --- | --- |
| `docs/product-brief.md` | `docs/product/brief.md` |
| `docs/cli.md` | `docs/specs/cli.md` |
| `docs/q5-sdm.md` | `docs/specs/q5-sdm.md` |
| `docs/q5-primer-design.md` | `docs/specs/q5-primer-design.md` |
| `docs/research/nbc_help_v284.txt` | `docs/references/nbc_help_v284.txt` |
| `docs/tasks/milestone-*.md` | `docs/tasks/contracts/` (same file name) |
| `docs/tasks/review-*.md` | `docs/tasks/reviews/` (same file name) |
| `docs/tasks/proposal-*.md`, `docs/tasks/later-*.md` | `docs/tasks/proposals/` (same file name) |

`docs/tasks/TEMPLATE.md`, `docs/agents/worker.md`, `docs/research/*.md` and
`docs/tasks/contracts/docs-okf-*.md` stay where they are.

## 3. Split `docs/decisions.md`

- Lines 1–7 (from `# Decisions` up to the line before `## D001:`) go to
  `docs/decisions/policy.md`.
- Each entry, from its `## Dnnn: ...` line up to the line before the next
  `## D` line (or end of file), goes to `docs/decisions/Dnnn.md`. Keep the
  `## Dnnn: ...` heading line as the first line. Strip trailing blank lines;
  end the file with one newline. Change no other byte.
- `git rm docs/decisions.md` after the split (use a script in
  `target/tmp/`, not hand copying; 62 entries, D001–D062).

## 4. Path rewrite

Write `target/tmp/okf-rewrite.sh` (sed over `git ls-files`, text files only)
and run it. Replace these strings, in this order, in every tracked text file
except the ones in §8:

| Old string | New string |
| --- | --- |
| `docs/product-brief.md` | `docs/product/brief.md` |
| `docs/cli.md` | `docs/specs/cli.md` |
| `docs/q5-sdm.md` | `docs/specs/q5-sdm.md` |
| `docs/q5-primer-design.md` | `docs/specs/q5-primer-design.md` |
| `docs/research/nbc_help_v284.txt` | `docs/references/nbc_help_v284.txt` |
| `docs/decisions.md` | `docs/decisions/` |
| `docs/tasks/milestone-` | `docs/tasks/contracts/milestone-` |
| `docs/tasks/review-` | `docs/tasks/reviews/review-` |
| `docs/tasks/proposal-` | `docs/tasks/proposals/proposal-` |
| `docs/tasks/later-` | `docs/tasks/proposals/later-` |

This applies inside the new `docs/decisions/*.md` files too.

Special case, `crates/cli/tests/documented_interpret.rs`, test
`d044_is_recorded`: read `"../../docs/decisions/D044.md"` (not the
directory). Its assertion stays the same.

## 5. `scripts/decision`

Change it to read `docs/decisions/D*.md` (sorted by number) instead of
`docs/decisions.md`. Same arguments, same output, same exit codes, same
error messages. Update the header comment to name the new directory.

## 6. Checks

`scripts/check`, plus these, each with the required result:

```bash
# 1. Split is lossless (ignoring blank lines): no output.
diff <(git show HEAD:docs/decisions.md | grep -v '^$') \
     <(cat docs/decisions/policy.md $(ls docs/decisions/D*.md | sort) | grep -v '^$')
# 2. 62 decision files.
ls docs/decisions/D*.md | wc -l
# 3. No old path left outside the excluded files: no output.
git grep -nE 'docs/(decisions\.md|cli\.md|q5-sdm\.md|q5-primer-design\.md|product-brief\.md|research/nbc_help|tasks/(milestone|review|proposal|later)-)' \
  -- . ':!AGENTS.md' ':!CLAUDE.md' ':!docs/agents/worker.md' ':!docs/tasks/contracts/docs-okf-*'
# 4. Moves recorded as renames.
git add -N docs && git diff --stat -M | tail -3
```

Note: check 4 uses `git add -N` only so that new files show in the diff.
Do not stage anything else.

## 7. Proving the change bites

| # | Change | Must fail |
| --- | --- | --- |
| 1 | In `d044_is_recorded`, point back at `../../docs/decisions.md` | `cargo test -p luria-cli d044_is_recorded` |
| 2 | Point `scripts/decision` back at `docs/decisions.md` | `scripts/decision D044` exits non-zero |

Restore each change byte-identically and report the failure lines.

## 8. Do not

- Edit `AGENTS.md`, `CLAUDE.md` or `docs/agents/worker.md` (the planner
  edits them after owner approval).
- Edit `docs/tasks/contracts/docs-okf-*.md`.
- Touch `tests/fixtures/genbank/raw/J01749.1_copy.gb` (untracked, not ours).
- Change any wording other than the paths in §4, add frontmatter, add
  `index.md`/`log.md` files, or turn paths into markdown links (okf-b, okf-c).
- Rename files other than in §2, or change any test other than
  `d044_is_recorded`'s path. If another test fails, stop and report.

## 9. Report

The standard handoff, plus: output of the four checks in §6, the list of
files the rewrite changed (`git diff --stat | tail -1` and the non-`docs/`
files by name), and both guard experiments.
