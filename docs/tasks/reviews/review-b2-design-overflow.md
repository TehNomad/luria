# Review fix B2: overflow in the primer-design feasibility sums

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review with one amendment (§7).
Planner: Claude. Parent: [docs/tasks/reviews/review-post-m4.md](/tasks/reviews/review-post-m4.md) finding B2.

## 1. The defect

`crates/core/src/primer_design.rs` computes `smallest_pair` with unchecked
`u64` arithmetic in three places:

- `design_inline` (around line 529): `region + MIN_FLANK + first_flank + first_reverse`
- `design_deletion` (around line 696): `width + 2 * first`
- `design_tailed` (around line 912): `(end - start) + 2 * first`

`first`, `first_flank` and `first_reverse` derive from the caller's
`min_annealing_overlap`. With `PrimerDesignRequest::create(…, i64::MAX, …)`
the sum overflows. Debug builds panic ("attempt to add with overflow").
Release builds wrap to a small number, pass the check, and report
`NoPrimerDesignFits` (Unsupported) instead of `NoPrimerDesignPossible`
(BiologicalValidation).

## 2. Required behaviour

In each of the three functions, compute `smallest_pair` with saturating
arithmetic (`saturating_add`, `saturating_mul`), so an overflowing sum
becomes `u64::MAX`. That is always greater than the template length, so the
existing check returns `DomainError::NoPrimerDesignPossible` with
`smallest_pair: u64::MAX`. For any sum that does not overflow, the value and
the behaviour are unchanged.

Code after the `smallest_pair > length` check runs only when the sum fits
the template, so it needs no change. If you find another overflow reachable
from a caller-supplied bound, stop and report it rather than fixing it.

## 3. Required tests, in `primer_design.rs`'s test module

Write these first and confirm they fail (panic) before the fix. Each builds a
`PrimerDesignRequest` with `min_annealing_overlap = i64::MAX`, calls
`design_primers`, and asserts
`Err(DomainError::NoPrimerDesignPossible(p))` with `p.smallest_pair ==
u64::MAX` and `category() == ErrorCategory::BiologicalValidation`. Reuse the
module's existing template and edit helpers.

1. `an_overflowing_bound_on_a_substitution_is_no_design_possible`
2. `an_overflowing_bound_on_a_deletion_is_no_design_possible`: a deletion of
   at least 2 bases.
3. `an_overflowing_bound_on_an_insertion_is_no_design_possible`
4. `an_overflowing_bound_on_a_tailed_replacement_is_no_design_possible`: a
   replacement with `max(w, m) > 6`, so it takes the tail path.

If `PrimerDesignRequest::create` refuses `i64::MAX`, stop and report. Do not
change `create`.

## 4. Do not

- Edit any file other than `crates/core/src/primer_design.rs`.
- Add dependencies, public items, or error variants.
- Change existing tests, fixtures, or any arithmetic other than the three
  `smallest_pair` sums.
- Refactor the three design paths (that is finding E1, a later contract).

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

## 6. Handoff

Report the changed file, the new tests and that they failed before the fix,
the exact check results, and any deviation.

## 7. Amendment on review, 2026-10-16

The expectation in §3 test 3 was wrong. An insertion's region is empty, so
with `bound = i64::MAX` the tailed sum is `2 × i64::MAX = u64::MAX − 1`. That
fits in a `u64`, never overflows, and the test cannot fail before the fix.
The worker kept the test and asserted `smallest_pair == u64::MAX − 1`, with
the same variant and category. Accepted. Test 1 uses a 3-base codon
substitution for the same reason: a 1-base region sums to exactly
`u64::MAX` without overflowing.
