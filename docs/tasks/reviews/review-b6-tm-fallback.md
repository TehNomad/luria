# Review fix B6: the matched-Tm fallback is broader than D030 allows

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review.
Planner: Claude. Parent: `docs/tasks/reviews/review-post-m4.md` finding B6.

## 1. The defect

`tm::melting_temperature_or_matched` (`crates/core/src/tm.rs`, around line
510) falls back to the matched duplex on **any**
`DomainError::TmParametersUnavailable`. `melting_temperature` raises that
error with four `aspect` values:

| `aspect` | Raised when |
| --- | --- |
| `"length"` | the primer and the site differ in length |
| `"alphabet"` | a base in the primer or the site is not `A`/`C`/`G`/`T` |
| `"tandem mismatch"` | two adjacent positions are both mismatched |
| `"motif"` | a doublet has no entry in the parameter table |

So a length mismatch, or an ambiguity code in the template site, silently
becomes a matched Tm with a recorded substitution, instead of an error. The
design paths are guarded today (ambiguity codes end availability, and
`m ≠ w` takes the matched basis directly), so the defect is reachable only
through the public function.

## 2. Requirement (D030, Tm basis, inline)

> The forward primer uses the mismatch-aware Tm against the wild-type site
> where published parameters cover the mismatch pattern, and the matched Tm
> where they do not.

The fallback is for a mismatch pattern the published parameters do not
cover: `"tandem mismatch"` and `"motif"`. A wrong length or a non-ACGT base
is not a mismatch pattern.

## 3. Required behaviour, `crates/core/src/tm.rs` only

In `melting_temperature_or_matched`:
- fall back, exactly as today, only when the error is
  `TmParametersUnavailable` with `aspect` equal to `"tandem mismatch"` or
  `"motif"`;
- return every other error unchanged, including `TmParametersUnavailable`
  with `aspect` `"length"` or `"alphabet"`.

Update the function's doc comment to state this. Do not change
`melting_temperature`.

## 4. Required tests, in `tm.rs`'s test module

Write these first. Tests 1 and 2 must fail before the fix.

1. `the_hybrid_does_not_fall_back_on_a_length_mismatch`:
   - a primer of 18 bases and a site of 17;
   - `melting_temperature_or_matched` returns
     `Err(DomainError::TmParametersUnavailable(p))` with
     `p.aspect == "length"`.
2. `the_hybrid_does_not_fall_back_on_an_ambiguous_site`:
   - an 18-base ACGT primer and the same site with one base replaced by `N`;
   - expect `Err(DomainError::TmParametersUnavailable(p))` with
     `p.aspect == "alphabet"`.
3. Regression: the existing test
   `the_hybrid_falls_back_to_the_matched_basis_and_says_so` still passes
   unchanged.

## 5. Do not

- Edit any file other than `crates/core/src/tm.rs`.
- Change `melting_temperature`, the parameter tables, or the terminal-mismatch
  and symmetry handling (B7 and B8 are separate findings).
- Change existing tests or fixtures, or add dependencies or public items.

## 6. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

## 7. Handoff

Report the changed file, the new tests and their pre-fix failures, the exact
check results, and any deviation.
