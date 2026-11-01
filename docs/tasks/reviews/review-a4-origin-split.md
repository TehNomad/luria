---
type: Task Contract
title: "Review A4: refuse circular features split away from the origin (D036)"
description: "A feature or CDS on a circular sequence is one interval, or exactly two intervals that cross the origin: [a, n) followed by [0, b)."
status: deprecated
---
# Review A4: refuse circular features split away from the origin (D036)

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review.
Planner: Claude. Parent: [docs/tasks/reviews/review-post-m4.md](/tasks/reviews/review-post-m4.md) A4. Decision: D036.

## 1. Requirement (D036, inline)

> A feature or CDS on a circular sequence is one interval, or exactly two
> intervals that cross the origin: `[a, n)` followed by `[0, b)`. Abutting
> intervals split anywhere else, such as `[0, 6)` followed by `[6, 24)`, are
> refused at construction with `UnsupportedFeatureStructure`. A linear feature
> stays a single interval.

## 2. Today

`interval::validate_structure` (`crates/core/src/interval.rs`, around line
332) treats consecutive intervals as contiguous when they are abutting
(`previous.end() == current.start()`) or crossing
(`previous.end() == length && current.start() == 0`). Both `Cds::create` and
the feature constructors call it.

## 3. Required change, `crates/core/src/interval.rs` only

- In `validate_structure`'s circular branch, add one check **after** the
  existing crossing-count and gap checks, so existing failures keep their
  current errors. If any consecutive pair is abutting and not crossing,
  return `DomainError::UnsupportedFeatureStructure`, with the same `feature`,
  `sequence` and `interval_count` fields as the neighbouring refusals and the
  reason `"a circular feature is split only where it crosses the origin"`.
- Update `validate_structure`'s doc comment to state D036's rule and cite
  D036. If the module doc says a circular feature may be "several intervals
  that abut", correct it the same way.

## 4. Required tests, in `interval.rs`'s test module

Write these first, and confirm that tests 1 and 2 fail before the change.
Build a 24-base circular sequence with the module's existing helpers.

1. `a_circular_feature_split_away_from_the_origin_is_refused`: `[0,6)+[6,24)`
   gives `UnsupportedFeatureStructure`, with category `Unsupported` (check the
   category in `error.rs`) and the reason above.
2. `a_circular_feature_split_at_the_origin_and_elsewhere_is_refused`:
   `[20,24)+[0,3)+[3,6)` gives `UnsupportedFeatureStructure`.
3. `a_circular_feature_split_only_at_the_origin_is_accepted`: `[21,24)+[0,6)`
   is `Ok`. Add this only if no existing test already asserts it.

## 5. Do not

- Edit any other file. If `cds.rs` or `feature.rs` documents the old rule,
  report it; don't change it.
- Change existing tests or fixtures. If an existing test or fixture case uses
  a non-origin split and now fails, **stop and report it**.

## 6. Checks

Run cargo with `TMPDIR=$PWD/target/tmp`, after `mkdir -p target/tmp`.

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

## 7. Handoff

Report the changed file, the new tests and their failures before the change,
the exact check results, and any deviation.
