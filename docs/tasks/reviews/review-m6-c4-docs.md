---
type: Task Contract
title: "Review M6 C-4, C-5, C-9: exit codes and a fixture note"
description: "C-4: docs/specs/cli.md §5 says exit 0 is \"verified, and every member designed\" and exit 1 is a failed write of stdout or --write-edited; §10.7 says interpret's codes are \"those of §5, plus 8\"."
status: deprecated
tags: [milestone-6]
---
# Review M6 C-4, C-5, C-9: exit codes and a fixture note

Status: **Implemented 2026-10-21** by the local worker; accepted on planner
review, with one more phrase in §5's prose (`--write-request`), which the
worker flagged. Planner: Claude. Parent:
[docs/tasks/reviews/review-post-m6.md](/tasks/reviews/review-post-m6.md) C-4, C-5, C-9. Owner, 2026-10-20: "Go
ahead and fix them". Docs and one fixture note; no code.

Files (no others):
- [docs/specs/cli.md](/specs/cli.md)
- `tests/fixtures/nl/cases.tsv` (the `note` column of one row)

## 1. Defects

- C-4: [docs/specs/cli.md](/specs/cli.md) §5 says exit 0 is "`verified`, and every member
  designed" and exit 1 is a failed write of stdout or `--write-edited`;
  §10.7 says interpret's codes are "those of §5, plus 8". Nothing says that
  for `luria q5 interpret` exit 0 is `proposed`, that exit 1 also covers
  the backend and `--write-request`, or what a failed `--write-request`
  write prints.
- C-5: §10.7's `model_output_invalid` row says "over 64 KiB"; a whole
  response over 1 MiB is `backend_response`, exit 1.
- C-9: `cases.tsv`'s `two-combined` note says "a combined construct is
  never assumed from separate variants"; the case is M6-16: stated in the
  text, proposed as asked, refused by core at design.

## 2. Required change

### 2.1 [docs/specs/cli.md](/specs/cli.md) §5, the exit-code table

Replace the rows for codes 0 and 1, exactly:

```
| 0 | `verified`, and every member designed; for `luria q5 interpret`, `proposed` |
| 1 | An internal error, or a failure to write the output: stdout, a `--write-edited` file or a `--write-request` file; for `luria q5 interpret`, also a model server that could not be reached or answered badly (§10.7) |
```

Nothing else in §5 changes.

### 2.2 [docs/specs/cli.md](/specs/cli.md) §10.7

1. Replace the sentence `Exit codes are those of §5, plus **8**: the
   model's reply was refused.` with exactly:
   `Exit codes are those of §5: **0** is \`proposed\`, **3** is
   \`needs_clarification\`, and **8** is a model reply that was refused.`
   (written on as many lines as the paragraph needs, with real backticks).
2. In the `model_output_invalid` row, replace `or over 64 KiB.` with
   `or over 64 KiB. A whole response over 1 MiB is \`backend_response\`.`
   (real backticks).
3. After the paragraph that starts `Every local input is checked before
   anything is sent`, add a paragraph, exactly:

   ```
   When the `--write-request` file cannot be written, the `proposed`
   document still goes to stdout, the reason goes to stderr as `luria:
   could not write <name>: <reason>`, and the exit is 1.
   ```

Check the rows touched by the earlier review-m6 contracts are still
there (`request_location_unusable` and the question table are not this
contract's); change nothing else in §10.

### 2.3 `tests/fixtures/nl/cases.tsv`

In the `two-combined` row, replace only the last (tab-separated) field,
`a combined construct is never assumed from separate variants`, with
`combined stated in the text: proposed as asked, and core refuses it at design (M6-16)`.
Keep the tabs and every other field exactly as they are; the file ends
with a newline as now.

## 3. Tests

No new tests. Every existing test must pass: `documented_example.rs` and
`documented_interpret.rs` read [docs/specs/cli.md](/specs/cli.md), and the corpus tests read
`cases.tsv` (none reads the `note` column; if one does, stop and report).

## 4. Do not

- Change any code, any other fixture, any other row or column, or any
  other section of [docs/specs/cli.md](/specs/cli.md).

## 5. Checks

From the repository root, after `export TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 6. Handoff

Report the changed lines, the check results, and any deviation.
