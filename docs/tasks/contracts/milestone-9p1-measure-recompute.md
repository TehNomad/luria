---
type: Task Contract
title: "Contract 9p1 — cli: measure where an edit's time goes"
description: "One edit on a 10 000 000-base record takes about 1.65 s in a release build (9e §10)."
status: deprecated
tags: [milestone-9]
---
# Contract 9p1 — cli: measure where an edit's time goes

Planner: Claude. Status: implemented. Tier: light.
Plan row: [docs/tasks/contracts/milestone-9-editor.md](/tasks/contracts/milestone-9-editor.md) §7, row 9p (first half).

## 1. What this contract builds

One edit on a 10 000 000-base record takes about 1.65 s in a release
build (9e §10). The owner wants it faster (2026-10-31). Before any change,
this contract measures each step of one edit, so the next contract
(9p2) speeds up the steps that cost the time. **This contract changes
no behaviour.** It adds one ignored test that prints a table of times.

## 2. The steps to time

These are the steps `apply_edit` and `rebuild_view` in
`crates/cli/src/sequence_view/state.rs` take, in order. Find each one in
the code and confirm the list. If a step costs time and is missing here,
add it to the table and say so.

| # | Step | Where |
| --- | --- | --- |
| 1 | `buffer.clone()` of the edit buffer | `apply_edit` |
| 2 | `apply_undoable` on the clone | `apply_edit`, `crates/core/src/edit_buffer.rs` |
| 3 | `rebuild(&buffer, &self.opened)`, and inside it `buffer.to_sequence()` and the feature, CDS and ceiling checks | `rebuild_view`, `crates/core/src/view_record.rs` |
| 4 | `choose()` | `rebuild_view` |
| 5 | `gc_line` | `rebuild_view`, `overview.rs` |
| 6 | `compute_orfs`, when ORFs are shown | `rebuild_view` |
| 7 | The search run again, when there is a last search | `rebuild_view`, `search.rs` |
| 8 | The relayout and `follow` | `rebuild_view` |
| 9 | Anything the undo group keeps per edit (`current_group.push`) | `apply_edit` |
| 10 | The next draw: `screen::draw` into a `TestBackend` of 80 x 24 | `screen.rs` |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/state.rs`: one new test,
   `#[ignore]`d, in the existing `tests` module.

## 4. The test

`recompute_profile`, marked `#[ignore = "timing; run with --release
--ignored --nocapture"]`. It builds three views:
- **A:** a FASTA of 10 000 000 bases (`ACGT` repeated);
- **B:** the same with ORFs shown (`O`);
- **C:** pBR322 with tet selected, ORFs shown, and the last search
  `CTGCAG`.

For each view, it does one one-base `Insert` at the middle, the way
`apply_edit` does, but calls each step of §2 itself, timing each with
`Instant`, and prints one row per step: the view, the step, the
milliseconds. A step that does not apply prints `-`. It runs the whole
sequence three times and prints the median of each step. It asserts
nothing about the times, but it does assert that the view after the
timed steps equals the view `apply_edit` gives for the same edit (same
sequence and the same cursor), so the timed path is the real one.

Make private items `pub(super)` only if the test cannot reach them
otherwise, and list each one you change.

## 5. Checks

`scripts/check`, then
`cargo test -p luria-cli --release --lib recompute_profile -- --ignored --nocapture`.

## 6. Report

The table from the release run, as printed. Then, for each step over
50 ms on view A or B, one line naming what in that step costs the time
(for example a whole-sequence copy, a second validation pass, a pass
over every base), with `file:line`. Do not optimise anything.

## 7. Do not

- Change any behaviour, or any existing test.
- Add dependencies, benchmarks or binaries.

## 8. Result (planner)

Release, median of three, one one-base insert at the middle:
- **A**, 10 000 000-base FASTA: 2044 ms in all. 1929 ms is step 3: `to_sequence` builds a String and `normalize` checks and copies it again (about 7 whole-sequence copies and a SHA-256). Clone 18 ms, `gc_line` 28 ms, the rest under 2 ms.
- **B**, A with ORFs shown: 5438 ms. Step 3 is 1798 ms, and `compute_orfs` is 3669 ms: a String per codon position, and `lookup` rebuilds the 64-letter table on every call.
- **C**, pBR322 with tet, ORFs and a search: 6.4 ms.

9p2 takes the two large steps.
