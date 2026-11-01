---
type: Task Contract
title: "Review A7b: --check for the q5-primer generators; track the unit-test driver (D039, C1)"
description: "The five generators in tests/fixtures/q5-primer/reference/ overwrite their tracked output, and none has a check-only mode."
status: deprecated
---
# Review A7b: `--check` for the q5-primer generators; track the unit-test driver (D039, C1)

Status: **Implemented 2026-10-16** by the local worker; the worker was stopped while matching values (§2 item 3), and the planner finished that check and the deliberate-failure demonstration during review.
Planner: Claude. Parent: [docs/tasks/reviews/review-post-m4.md](/tasks/reviews/review-post-m4.md) A7 and C1. Decision:
D039. Environment as in [docs/tasks/reviews/review-a7a-tm-verification.md](/tasks/reviews/review-a7a-tm-verification.md) §3: run
`.tmp/tm/venv/bin/python -I`, and keep scratch files under `target/`.

## 1. Gaps

1. The five generators in `tests/fixtures/q5-primer/reference/` overwrite
   their tracked output, and none has a check-only mode:
   - `native-design.py` writes `native.tsv`;
   - `native-deletion.py` writes `native-deletion.tsv`;
   - `native-insertion.py` writes `native-insertion.tsv`;
   - `native-replacement.py` writes `native-replacement.tsv`;
   - `neb-q5-sdm-reference.py` writes `reference/designs.json`.
2. **C1.** Several test modules credit an untracked script or a by-hand call
   for their expected values:
   - `primer_design.rs` (test module comment, around line 1291) says every
     expectation there "was produced by `native-design.py`". Its
     `--synthetic` mode does not cover `TEMPLATE`. The real driver is the
     untracked `.tmp/tm/rederive.py`, which loads `native-design.py`'s
     `design()` and prints the expectations for `TEMPLATE` (T60) and for
     `q5_design.rs`'s template (T90).
   - The figures in `primer_design.rs`'s test near line 2097 were produced
     by "`native-replacement.py`'s `design()` run by the planner".
   - `q5_design.rs` (test module comment, around line 322) also credits
     `native-design.py`.

## 2. Required changes

1. **`--check` in all five generators.**
   - Each one builds its output text exactly as today. Without `--check`,
     write it as today.
   - With `--check`, compare it byte for byte with the tracked file, print
     `OK <name>` or `DIFFERS <name>` with the first differing line, exit 1
     on a difference, and write nothing.
   - The three indel scripts already load `native-design.py`'s helpers, so
     they may reuse one `emit(path, text)` helper from it.
     `neb-q5-sdm-reference.py` gets its own.
   - Keep all diagnostic printouts and every algorithm unchanged.
   - `native-design.py`'s existing `--synthetic` mode stays as it is.
2. **`tests/fixtures/q5-primer/reference/unit-expectations.py`**:
   - copy it from `.tmp/tm/rederive.py`;
   - load `native-design.py` from the script's own directory, instead of
     taking it from `argv`;
   - add the `native-replacement.py` `design()` call or calls that produce
     the figures cited near `primer_design.rs` line 2097. Read the test to
     find its inputs;
   - it prints and writes nothing, so it has no `--check`.
3. **Confirm the expectations, by hand.** Run `unit-expectations.py`. For
   every case it prints, find the Rust test or tests that assert those
   values, in `primer_design.rs` and `q5_design.rs`, and confirm that the
   asserted spans, primer texts and Tms (to the two decimals printed) match.
   List each case with the tests that use it. If any value disagrees, **stop
   and report it**. Do not change a test or the script to make them agree.
4. **The C1 comments.** In the test-module comments named in §1 item 2, name
   `unit-expectations.py` and the command that prints the values. These are
   comment changes only.
5. **`tests/fixtures/q5-primer/README.md`.** Document `--check`, and list
   `unit-expectations.py` in the Files table.

## 3. Required verification

- All five generators pass `--check` against the tracked files. If any
  reports a difference, stop and report the diff; do not regenerate.
- Show one deliberate failure: corrupt a scratch copy under `target/`, then
  restore.

## 4. Do not

- Change any fixture output, any non-comment Rust line, or any test
  assertion.
- Track `.tmp/review/native_design.py`. It is a superseded D029 derivation.

## 5. Checks

The Python commands, a comment-only guard on `primer_design.rs` and
`q5_design.rs` (the script in [docs/tasks/reviews/review-d1-q5-comments.md](/tasks/reviews/review-d1-q5-comments.md) §5), and
the usual cargo checks with `TMPDIR=$PWD/target/tmp`.

## 6. Handoff

Report:
- the files changed and added;
- each `--check` output;
- the case-to-test table from §2 item 3;
- the guard and check results;
- any deviation.
