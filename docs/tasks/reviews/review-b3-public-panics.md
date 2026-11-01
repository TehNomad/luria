# Review fix B3: public functions that panic or check too little

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review.
Planner: Claude. Parent: `docs/tasks/reviews/review-post-m4.md` finding B3.

## 1. The defects

1. `mutation::codon_positions` is `pub` and computes
   `3 * (number - 1) + offset` and `total_length - 1 - read` unchecked:
   - `number == 0` underflows;
   - a minus-strand codon past the CDS end underflows (e.g.
     `codon_positions(intervals, Strand::Minus, 20, 30)`);
   - a huge `number` overflows the multiplication.

   Its only caller is `apply_residue` in the same file.
2. `Sequence::letter_at(u64::MAX)` (`sequence.rs`, around line 375) panics:
   it computes `position + 1` while building the
   `CoordinateOutOfBounds` error.
3. `mutation::require_within_feature` is `pub` and checks only the
   **start** of its `position` interval, so a multi-base interval whose start
   lies in the feature is accepted. Its only caller is `apply_nucleotide`
   (single base). The private `require_region_within_feature` already checks
   the whole interval.

## 2. Required behaviour

**2.1 `codon_positions`**
- Make it private (`fn`, no `pub`).
- Compute `read` and `biological` with checked arithmetic
  (`checked_sub`, `checked_mul`, `checked_add`). On any failure, return the
  same error the function already returns when a nucleotide is not found:
  `DomainError::InvalidMutationRequest` with `request: "codon coordinates"`
  and the same `reason` text.
- When `number` is in range, the behaviour is unchanged.

**2.2 `letter_at`**
- Build the error with `end: position.saturating_add(1)`. Nothing else
  changes.

**2.3 `require_within_feature`**
- Make it private.
- Make it check the whole interval: some feature interval has
  `start <= position.start()` and `position.end() <= interval end`. This is
  exactly what `require_region_within_feature` does.
- Remove `require_region_within_feature` and point its two callers at
  `require_within_feature`. The error variant, payload and fields stay the
  same.
- Update the doc comments of `require_within_feature` and
  `require_point_within_feature` that contrast the two functions.
- For a single base the check is identical to today's, so the behaviour of
  `apply_nucleotide`, `apply_deletion` and `apply_replacement` does not
  change.

## 3. Required tests

Write these first. Confirm that 1–3 fail (panic) before the fix and that 4
fails or does not compile before it. Put tests 1, 2 and 4 in `mutation.rs`'s
test module; tests there can call private functions.

1. `a_residue_number_of_zero_selects_no_codon`:
   `codon_positions(<one interval [0,30)>, Strand::Plus, 0, 30)` returns
   `Err(DomainError::InvalidMutationRequest(_))`.
2. `a_minus_strand_codon_past_the_cds_end_selects_no_codon`: the same
   interval with `Strand::Minus`, number `20`, total length `30` returns
   `Err(DomainError::InvalidMutationRequest(_))`. Also check a huge number,
   e.g. `u64::MAX / 2`, on `Strand::Plus`.
3. In `sequence.rs`'s test module,
   `letter_at_the_largest_position_is_out_of_bounds`: `letter_at(u64::MAX)`
   on any sequence returns `Err(DomainError::CoordinateOutOfBounds(p))` with
   `p.start == u64::MAX` and `p.end == u64::MAX`.
4. `a_multi_base_interval_starting_inside_a_feature_but_ending_outside_is_refused`:
   - feature intervals `[[0,10)]`, position `[8,12)`;
   - `require_within_feature` returns
     `Err(DomainError::TargetOutsideFeature(_))`;
   - `[8,10)` against the same intervals returns `Ok(())`.

All existing tests must keep passing unchanged.

## 4. Do not

- Edit files other than `crates/core/src/mutation.rs` and
  `crates/core/src/sequence.rs`.
- Add dependencies, public items or error variants.
- Change existing tests or fixtures.
- If an integration test under `crates/core/tests/` calls `codon_positions`
  or `require_within_feature`, stop and report instead of making them
  private.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

## 6. Handoff

Report the changed files, the new tests and their pre-fix failures, the exact
check results, and any deviation.
