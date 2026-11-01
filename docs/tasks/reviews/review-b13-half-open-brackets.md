# Review B13: print half-open intervals as `[start, end)`

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review.
Planner: Claude. Parent: `docs/tasks/reviews/review-post-m4.md` B13. Decision:
D006, under which internal nucleotide coordinates are zero-based and
half-open, `[start, end)`.
File: `crates/core/src/primer.rs` only.

## 1. Defect

Several messages in `primer.rs` print a half-open interval with a closing
`]`. For example, `[20, 40]` is printed for `[20, 40)`. The places are these
(line numbers approximate):

- `PrimerPair::relationship` (around 524): `"reverse span ["`, `describe(back)`,
  `"] and forward span ["`, `describe(front)`, `"]: "`;
- around 639: `" overlaps the deleted region ["` … `"]: a primer of a deletion …"`;
- around 684: `" overlaps the replaced region ["` … `"]: a reverse primer of a replacement …"`;
- around 1068: `"the replacement is ["` … `"], so a tailed primer's …"`;
- around 1351: `"the deletion is ["` … `"], so D031 requires …"`;
- around 1427: `"the replacement is ["` … `"], so the reverse primer's …"`.

The message around 833 already closes with `)`, and is correct.

## 2. Required change

At each place listed in §1, change the closing `]` that ends an interval to
`)`. Change nothing else in the message text.

Before you finish:
- grep `primer.rs` for any other message that prints an interval and closes
  it with `]`, and list what you find;
- fix only intervals, i.e. a start and an end coordinate.

## 3. Tests

1. Update the two existing tests that pin the relationship text. Around
   line 2402, `"reverse span [0, 12] and forward span [12, 30]: abutting"`
   becomes `"reverse span [0, 12) and forward span [12, 30): abutting"`.
   Around 2420, `[12, 30]` and `[0, 20]` become `[12, 30)` and `[0, 20)`.
   These are the only existing test changes allowed: their expected text
   encoded the defect.
2. For each of the five error messages in §1:
   - find an existing test that triggers it;
   - add an assertion that the reason contains the half-open form, e.g.
     `"the deletion is [50, 53)"`, using that test's own coordinates.
   - Where no existing test triggers a message, add a minimal test if an
     existing test's setup can easily be adapted. Otherwise, report it.
3. Write the new assertions first, and confirm they fail before the change.

## 4. Do not

- Edit any other file.
- Change wording beyond the bracket.
- Change `PRIMER_DESIGN_VERSION` (B12 does that).

## 5. Checks

The usual cargo checks with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 6. Handoff

Report:
- the changed lines;
- the tests that cover each message, or the ones not covered;
- the grep result;
- the check results;
- any deviation.
