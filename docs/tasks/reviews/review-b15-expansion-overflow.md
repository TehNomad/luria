# Review fix B15: degenerate expansion overflows before it can warn

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review.
Planner: Claude. Parent: `docs/tasks/reviews/review-post-m4.md` finding B15 and
owner answer A3.

## 1. The defect

`sequence::expand_degenerate` (`crates/core/src/sequence.rs`, around line
251) multiplies the per-position set sizes with an unchecked
`stride *= sets[set_index].len()`. A text of 32 `N` denotes 4^32 = 2^64
sequences:

- debug builds panic with "attempt to multiply with overflow";
- release builds wrap the count to 0, return an empty list, and
  `q5_expand::expand_replacement` then refuses the request as a no-op, which
  is the wrong error.

## 2. Owner decision (A3, 2026-10-16), inline

D033 stays as written: **no cap** on the expansion count, and a warning past
64 emitted designs. This fix adds no policy limit. It refuses only a count
that cannot be represented in a `usize`, which no machine could produce.
Counts that fit but are very large are out of scope; leave them alone.

## 3. Required behaviour

**3.1 New error variant, `crates/core/src/error.rs`.** Follow the pattern of
the existing variants exactly: payload struct, enum arm, `category()`, the
message in the `Display`/describe match, and any exhaustive tests inside
`error.rs`.

```rust
/// Payload: a degenerate text denotes more concrete sequences than can be
/// counted (D033 sets no cap; this is representability, not policy).
#[derive(Clone, Debug, PartialEq)]
pub struct DegenerateExpansionTooLargeProblem {
    pub replacement: String,
}
// enum arm:
DegenerateExpansionTooLarge(DegenerateExpansionTooLargeProblem),
```

- Category: `ErrorCategory::Unsupported`.
- Message, for reporting only: `"'{replacement}' denotes more concrete
  sequences than can be counted"`.

**3.2 `expand_degenerate`.**
- Compute the strides and the count with `checked_mul`.
- On overflow, return
  `DomainError::DegenerateExpansionTooLarge(DegenerateExpansionTooLargeProblem
  { replacement: text.to_string() })` **before** any sequence is built or any
  output vector is allocated.
- The existing letter check runs first, so an invalid letter is still
  reported as `InvalidSequenceCharacters`, even in a long text.
- Update the doc comment, whose line "it is not capped here" stays true, to
  mention this error.

**3.3 `q5_expand::expand_replacement`.** It already propagates
`expand_degenerate`'s errors with `?`. Add the new error to step 2 of its doc
comment. No code change is expected there.

## 4. Required tests

Write these first. Confirm 1 and 2 fail (panic) before the fix.

**Never** write a test whose text denotes between 2^20 and 2^64 sequences:
it would try to allocate them and hang or exhaust memory. 32 `N`s is the
smallest overflowing case on a 64-bit target.

1. In `sequence.rs`'s test module,
   `an_expansion_too_large_to_count_is_refused`: `expand_degenerate` on
   `"N".repeat(32)` and on `"N".repeat(40)` returns
   `Err(DomainError::DegenerateExpansionTooLarge(p))` with `p.replacement`
   equal to the input, and `category() == ErrorCategory::Unsupported`.
2. In `q5_expand.rs`'s test module,
   `a_degenerate_replacement_too_large_to_count_is_refused`: same as test 1,
   called through `expand_replacement` with a valid template, region and
   expected original from the module's existing helpers. Assert the variant,
   **not** `InvalidMutationRequest`.
3. In `sequence.rs`, `an_invalid_letter_is_reported_before_the_count`:
   `"N".repeat(32) + "X"` returns `InvalidSequenceCharacters` with
   `position == 32`.
4. Regression: the existing `NNN` → 64 test (or an equivalent you add) still
   passes.

**Exhaustive name maps in integration tests.** Three tests list every
`DomainError` variant by name:

- `crates/core/tests/core_foundation.rs` (around line 258);
- `crates/core/tests/mutation_fixtures.rs` (around line 230);
- `crates/core/tests/mutation_invariants.rs` (around line 235).

Add exactly one arm for the new variant to each, following the existing
lines (`DomainError::DegenerateExpansionTooLarge(_) =>
"DegenerateExpansionTooLarge",`). If any of them also lists variants
elsewhere, for example a category table, add the new variant there in the
same style. This is the only permitted change to existing test files.

## 5. Do not

- Edit files other than `error.rs`, `sequence.rs`, `q5_expand.rs`, and the
  one-arm additions in the three integration test files named above.
- Add a cap, a threshold, or any limit below `usize` overflow.
- Add dependencies or change existing assertions or fixtures.

## 6. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

## 7. Handoff

Report the changed files, the new tests and their pre-fix failures, the exact
check results, and any deviation.
