---
type: Task Contract
title: "Review refactor E3: one copy of each interval-containment predicate"
description: "Three containment tests over a CDS's intervals are written out by hand in several places."
status: deprecated
---
# Review refactor E3: one copy of each interval-containment predicate

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review.
Planner: Claude. Parent: [docs/tasks/reviews/review-post-m4.md](/tasks/reviews/review-post-m4.md) finding E3.

**Behavior-preserving.** Every existing test must pass unchanged. Every
warning and error payload must be byte-identical.

## 1. The duplication

Three containment tests over a CDS's intervals are written out by hand in
several places:

| Predicate | Meaning | Written in |
| --- | --- | --- |
| holds position `p` | some interval has `start <= p < end` | `q5.rs` `edit_warnings` (deletion coding count) and `replacement_warnings` (coding count) |
| holds region `[s, e)` | some interval has `start <= s && e <= end` | `mutation.rs` `require_within_feature` |
| holds point `p` strictly | some interval has `start < p < end` | `mutation.rs` `require_point_within_feature`, `q5.rs` `insertion_warnings` |

`q5.rs` also routes each kind through its own helper. `insertion_warnings`
and `replacement_warnings` each take a `MutationResult` and match on it, with
dead arms for the other four kinds.

A fourth difference is **not** in scope:
- the deletion warning tests whether the coding count is a multiple of 3;
- the replacement warning tests whether `m − w` is a multiple of 3.

That is a behavior question (`review-post-m4.md` E3). Keep both exactly as
they are.

## 2. Required changes

**2.1 `crates/core/src/interval.rs`.** Add three `pub(crate)` functions with
one-line docs:

```rust
pub(crate) fn holds_position(intervals: &[NucleotideInterval], position: u64) -> bool;
pub(crate) fn holds_region(intervals: &[NucleotideInterval], start: u64, end: u64) -> bool;
pub(crate) fn holds_point(intervals: &[NucleotideInterval], point: u64) -> bool;
```

Add unit tests to `interval.rs`'s test module that cover both edges of each:
- `p == start` and `p == end − 1` hold a position, `p == end` does not;
- a region equal to an interval is held, a region one wider is not;
- `p == start` and `p == end` are not held as points, `p == start + 1` is.

These are the only new tests.

**2.2 `crates/core/src/mutation.rs`.** `require_within_feature` uses
`holds_region`, and `require_point_within_feature` uses `holds_point`. Their
error payloads stay unchanged.

**2.3 `crates/core/src/q5.rs`.**
- Restructure `edit_warnings` into one `match` on `&MutationResult` with
  five arms:
  - `Deletion`: the current deletion logic, counting with `holds_position`;
  - `Insertion`: the current `insertion_warnings` logic, using
    `holds_point`;
  - `Replacement`: the current `replacement_warnings` logic, counting with
    `holds_position`;
  - `Nucleotide` and `Residue`: no warnings.
- Remove `insertion_warnings` and `replacement_warnings` once their logic has
  moved. Carry their doc comments, with the sources, into the matching arms or
  into `edit_warnings`'s doc. The D031–D033 citations must survive.
- Every warning value stays identical, including `wholly_inside` and the
  counts.

## 3. Guards

- `python3 -I target/api_guard.py crates/core/src/interval.rs crates/core/src/mutation.rs crates/core/src/q5.rs`
  prints nothing and exits 0. The script is in
  [docs/tasks/reviews/review-e1-design-paths.md](/tasks/reviews/review-e1-design-paths.md) §3.
- The only test-module change is the new `interval.rs` tests. There is no
  change under `crates/core/tests/`.

## 4. Do not

- Edit any other file.
- Change behavior, including the deletion-versus-replacement frame test.
- Change the abutting-interval behavior (owner question A4, deferred).
- Change existing tests.

## 5. Checks

Same as E1 §5, with the three files passed to the API guard. The test count
is 510 plus your new `interval.rs` tests.

## 6. Handoff

Report the changed files, the new tests, the exact check results, and any
deviation.
