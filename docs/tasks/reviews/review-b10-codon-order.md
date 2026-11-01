# Review B10: a codon crossing the origin is recorded in feature order (D040)

Status: **Implemented 2026-10-16** by the local worker; accepted after the §9 amendment (the worker stopped on a test my contract missed). The planner fixed one wrong reverse complement in a test comment.
Planner: Claude. Parent: `docs/tasks/reviews/review-post-m4.md` B10. Decision: D040.
Files:
- `crates/core/src/mutation.rs`
- `tests/fixtures/mutation/cases.tsv` (two rows)
- `tests/fixtures/mutation/README.md` (one line)

## 1. Requirement (D040, inline)

> The intervals a residue result records for its codon are listed the way
> D036 writes a feature: in reference order, except that a codon crossing
> the origin of a circular sequence lists the part ending at the sequence
> length first, then the part starting at zero. A codon at bases 22, 23 and
> 0 of a 24-base sequence is recorded as `[22, 24)` then `[0, 1)`, and
> `target_coordinates()` gives `22, 23, 0`. The order is not the strand's
> reading order. A minus-strand codon is listed in reference order, as a
> minus-strand CDS's intervals are. The mutation fixtures' `coords` field
> follows the same order.

## 2. Today

`intervals_for` (`mutation.rs`, near line 2124) sorts the positions in
ascending order and groups consecutive runs. A codon at 22, 23, 0 comes out
as `[0, 1)` then `[22, 24)`.

## 3. Required changes

1. **`intervals_for`:**
   - add a parameter, the sequence length (`length: u64`);
   - after grouping, if there are exactly two intervals, the first starts at
     0 and the second ends at `length`, swap them;
   - a codon is three contiguous CDS positions, and a CDS crosses the origin
     at most once (D036), so that is the only reordering needed;
   - the call site in `apply_residue` (near line 1170) passes
     `edited.length()`; a residue edit doesn't change the length;
   - update `intervals_for`'s comment, citing D040.
2. **`ResidueMutation::coordinates()`:** say in its doc comment that the
   order is D040's.
3. **`tests/fixtures/mutation/cases.tsv`:** in rows `res-c-origin` and
   `res-c-origin-write`, change `coords:0,22,23` to `coords:22,23,0`. Change
   nothing else in those rows or in any other row.
4. **`tests/fixtures/mutation/README.md`:**
   - line 62 now says "`coords` are the reference coordinates the operation
     recorded, ascending";
   - make it say they're in the recorded intervals' order: ascending, except
     that a codon crossing the origin lists the part ending at the sequence
     length first (D040).

## 4. Tests: write them first, with the fixture edit, and confirm they fail before the change

1. In `mutation.rs`'s test module,
   `a_codon_across_the_origin_is_recorded_in_feature_order`:
   - use `mut-c` (circular, 24 bases): reuse the existing helper that the
     origin-crossing residue tests use, or build the sequence the same way;
   - use a plus-strand complete CDS `[22, 24)` + `[0, 7)`, as fixture row
     `res-c-origin-write` does, with residue 1 W→F and the default policy;
   - assert `coordinates()` is `[22, 24)` then `[0, 1)`;
   - assert `target_coordinates()` is `vec![22, 23, 0]`;
   - assert the result verifies.
2. `a_minus_strand_codon_across_the_origin_is_listed_in_reference_order`:
   - same sequence, with a minus-strand CDS `[22, 24)` + `[0, 7)`;
   - residue 3 reads, 5′→3′ on the minus strand, reference positions 0, 23
     and 22. Derive this yourself from `codon_positions` and confirm it;
   - assert `target_coordinates()` is `vec![22, 23, 0]`;
   - choose the expected original from the bases you read: complement them
     and read them in strand order, and write the derivation in a comment;
   - use any replacement residue and the default policy.
   - If `Cds::create` refuses this CDS (because of completeness, for
     example), try `completeness` values that the existing minus-strand
     wrap tests use. If none work, leave this test out and report it.
3. With the fixture edit in place, `mutation_fixtures` must fail on the two
   rows before the code change.

## 5. Stop and report if

- any existing test besides the two fixture rows needs a change. Grep
  `mutation.rs` and `crates/core/tests/` for tests asserting the coordinates
  of an origin-crossing codon;
- verification of an origin-crossing residue result fails after the change.

## 6. Do not

- Change the order for any codon that doesn't cross the origin.
- Change `target_coordinates()` for the other result kinds.
- Change `MUTATION_ALGORITHM_VERSION` (B12 does that).
- Edit other files.

## 7. Checks

The usual cargo checks with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 8. Handoff

Report:
- the changed files;
- the derivations for test 2;
- which tests failed first;
- the check results;
- any deviation.

## 9. Amendment, 2026-10-16

My search missed one test and two doc lines that state the old order. All
three follow from D040 and are approved:
- `crates/core/tests/mutation_invariants.rs`,
  `an_origin_crossing_codon_is_recorded_as_two_intervals`: the expected
  `vec![0, 22, 23]` becomes `vec![22, 23, 0]`. The message says the
  coordinates are in D040's order, the part ending at the sequence length
  first.
- `MutationResult::target_coordinates()` doc comment: "ascending" becomes
  D040's order.
- `tests/fixtures/mutation/README.md` near line 158: "Both record coordinates
  0, 22, 23." becomes "22, 23, 0".
