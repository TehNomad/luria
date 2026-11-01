# Milestone 4d-1 task contract: represent, apply and verify an insertion

Status: **Implemented 2026-10-15** by the local worker; accepted on planner review (§9).
Date: 2026-10-15
Planner: Claude. Parent: [docs/tasks/contracts/milestone-4-indels.md](/tasks/contracts/milestone-4-indels.md), pass 4. Mirrors
`milestone-4a-deletion.md`, which is implemented. Use its code in
`mutation.rs` (`NucleotideDeletionIntent`, `apply_deletion`, `verify_deletion`)
as the pattern.

## 1. Goal

A DNA-level insertion can be requested, applied and independently verified.
Primer design for it is **not** part of this task: the design layer must
refuse an insertion explicitly (§3.6).

## 2. Requirements, inline (D032, approved 2026-10-15)

- An `InsertionPoint` is one 0-based coordinate `p`, meaning "between base
  `p − 1` and base `p`". The inserted bases occupy `[p, p + k)` of the edited
  artifact.
- Inserted bases are `A`, `C`, `G`, `T` only, and there is at least one.
- **What an insertion asserts (D011):** the base on each side of the point.
  The request states the bases at `p − 1` and `p`, and a mismatch refuses the
  edit. So `1 ≤ p ≤ n − 1`.
- **Verification:** the edited artifact has exactly `n + k` bases and is
  `source[..p] + inserted + source[p..]`.
- **Scope:** an insertion is inside a CDS interval when `start < p < end`,
  i.e. both flanking bases are in that interval. An insertion exactly at a
  CDS's edge is outside it. (Planner's rule.)
- `EmptyInterval` and `NucleotideInterval` stay exactly as they are
  (owner decision 4.1).

## 3. Required behaviour

### 3.1 `crates/core/src/interval.rs`: a new, additive type

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct InsertionPoint {
    sequence: SequenceReference,
    position: u64,
}

pub fn create_point(sequence: &SequenceReference, position: i64)
    -> Result<InsertionPoint, DomainError>;

impl InsertionPoint {
    pub fn reference(&self) -> SequenceReference;
    pub fn position(&self) -> u64;
    pub fn within(&self, sequence: &Sequence) -> Result<(), DomainError>;
}
```

- `create_point` refuses a negative position with `NegativeCoordinate`
  (`start` and `end` both set to the position).
- `within` refuses `position > sequence.length()` with
  `CoordinateOutOfBounds` (`start` and `end` both set to the position,
  `length` the sequence length). `position == length` is within; `apply`
  refuses it for having no base after it.
- Change nothing else in `interval.rs`.

### 3.2 `mutation.rs`: the intent

```rust
pub struct NucleotideInsertionIntent {
    source: SequenceReference,
    point: InsertionPoint,
    expected_before: char,
    expected_after: char,
    inserted: String,
}
impl NucleotideInsertionIntent {
    pub fn create(source: SequenceReference, point: InsertionPoint,
        expected_before: String, expected_after: String, inserted: String)
        -> Result<NucleotideInsertionIntent, DomainError>;
    // accessors: source(), point(), expected_before(), expected_after(), inserted()
}
```

`create` refuses with `InvalidMutationRequest` and `request: "insertion"`:

1. `point.reference() != source`. Same reason text as the deletion's.
2. `expected_before` or `expected_after` not exactly one of `A`, `C`, `G`,
   `T`. Use `single_dna_base` with field names `"expected before"` and
   `"expected after"`.
3. `inserted` empty, with a reason containing `"at least one"`.
4. Any letter of `inserted` not in `A`, `C`, `G`, `T`. Use `single_dna_base`
   per letter with field `"inserted"`, as the deletion does.

### 3.3 The record and the result variant

```rust
pub struct NucleotideInsertion {
    source: SequenceReference,
    edited: Sequence,
    point: InsertionPoint,
    expected_before: char,
    expected_after: char,
    actual_before: char,
    actual_after: char,
    inserted: String,
    state: MutationState,
}
// accessors for every field except source/edited, which MutationResult exposes

pub enum MutationResult { …, Insertion(NucleotideInsertion) }
```

Every existing `match` on `MutationResult` in `mutation.rs` gains an
`Insertion` arm:

- `source()`, `edited()`, `state()`: the record's fields.
- `target_coordinates()`: **empty**. An insertion writes and removes no source
  coordinate. Say so in the doc comment.
- `is_nucleotide()`: false. `is_deletion()`: false.
- Add `pub fn is_insertion(&self) -> bool`.
- `accepted()`: sets the state to `Verified`.
- The second match inside `verify`: `unreachable!`, as the deletion arm is.

### 3.4 Applying

```rust
pub fn apply_insertion(sequence: &Sequence, scope: Option<Cds>,
    result: Identifier, intent: NucleotideInsertionIntent)
    -> Result<MutationResult, DomainError>;
```

In this order, writing nothing until all checks pass:

1. `require_sequence` for `intent.source()` and
   `intent.point().reference()`; `intent.point().within(sequence)?`.
2. `p == 0` or `p == sequence.length()`: `InvalidMutationRequest`,
   `request: "insertion"`, with a reason containing `"a base on each side"`.
3. If `scope` is `Some(cds)`: `require_sequence(cds.reference(), sequence)?`.
   Then require `start < p < end` for one of `cds.intervals()`. Otherwise
   return `TargetOutsideFeature` with `start: p`, `end: p`. Use a new private
   helper.
4. Read the actual bases at `p − 1` and `p`. If `p − 1` differs, return
   `ExpectedNucleotideMismatch` with `position: p − 1` and that base's
   expected and actual letters. Check `p` the same way, with `position: p`.
5. Build `letters[..p] ++ inserted ++ letters[p..]`, then
   `normalize(result, …, sequence.topology())`.
6. Return `MutationResult::Insertion`, state `Calculated`.

### 3.5 Verifying

In `verify`, handle `Insertion` in its own branch next to the deletion's,
before the equal-length check. Leave every other path unchanged. Refuse with
`MutationVerificationFailure` when:

1. `edited.length() != source.length() + k`, with a reason containing
   `"an insertion of <k> bases lengthens the artifact by exactly <k>"`;
2. the edited letters are not `source[..p] ++ inserted ++ source[p..]`;
3. the source's bases at `p − 1` and `p` differ from the record's expected or
   actual flanking bases, with a reason containing
   `"the insertion claims do not match the source"`.

### 3.6 The design layer refuses an insertion

In `primer_design.rs`, `design_primers`, after the verified-result check:
`if result.is_insertion()`, return
`out_of_scope(&policy, "luriaNative insertion primer design is not implemented yet")`.

### 3.7 Errors

No new `DomainError` variants. Do not edit `error.rs`.

## 4. Required tests

Unit tests in `mutation.rs`'s test module, on `seq_a()`
(`"ATGGCTTGGAATCAGCAGCAGCAGCAGCAG"`, 30 nt). Values derived by plain string
slicing (planner, 2026-10-15). In this table `ins(p, flanks, bases)` means an
intent at `p` asserting `flanks` (before, after) and inserting `bases`.

| # | Test | Input | Expect |
|---|---|---|---|
| I1 | create refuses another source | point on a different sequence | `InvalidMutationRequest` |
| I2 | create refuses an empty insertion | `ins(9, G/A, "")` | `InvalidMutationRequest`, reason contains `at least one` |
| I3 | create refuses an ambiguity code | `ins(9, G/A, "GNG")` | `InvalidMutationRequest` |
| I4 | create refuses a non-base flank | `ins(9, N/A, "GGG")` | `InvalidMutationRequest` |
| I5 | apply inserts at the point | `ins(9, G/A, "GGG")` | edited `ATGGCTTGGGGGAATCAGCAGCAGCAGCAGCAG` (33), linear, `Calculated`, `is_insertion()`, `!is_deletion()`, `!is_nucleotide()`, `target_coordinates()` empty, source unchanged |
| I6 | near both ends | `ins(1, A/T, "C")` and `ins(29, A/G, "T")` | `ACTGGCTTGGAATCAGCAGCAGCAGCAGCAG` and `ATGGCTTGGAATCAGCAGCAGCAGCAGCATG` |
| I7 | the very ends are refused | `ins(0, …)` and `ins(30, …)` with any valid letters | `InvalidMutationRequest`, reason contains `a base on each side` |
| I8 | past the end is out of bounds | point 31 | `CoordinateOutOfBounds` from `apply_insertion` |
| I9 | a wrong flank names the position | `ins(9, C/A, "GGG")` | `ExpectedNucleotideMismatch`, `position == 8`, `expected == "C"`, `actual == "G"` |
| I10 | circular stays circular | circular `seq_a` text, `ins(9, G/A, "GGG")` | topology `Circular`, text as I5 |
| I11 | scope: inside | `cds_of(12)`, `ins(9, G/A, "GGG")` | succeeds |
| I12 | scope: at the CDS edge | `cds_of(12)`, `ins(12, T/C, "GGG")` | `TargetOutsideFeature` |
| I13 | verify accepts | I5's result | `Verified` |
| I14 | verify: wrong length | I5's record with `edited` = I5's text minus its last base (32 nt) | `MutationVerificationFailure`, reason contains `lengthens the artifact by exactly 3` |
| I15 | verify: wrong place | I5's record with `edited` = `ATGGCTTGGAGGGATCAGCAGCAGCAGCAGCAG` (GGG at 10) | `MutationVerificationFailure` |
| I16 | verify: false flank claim | I5's record with `expected_before` and `actual_before` set to `'C'` | `MutationVerificationFailure`, reason contains `insertion claims` |

In `interval.rs`'s test module:

| # | Test | Expect |
|---|---|---|
| P1 | a negative point is refused | `create_point(&ref, -1)` gives `NegativeCoordinate` |
| P2 | a point may equal the length but not exceed it | on a 30-nt sequence, `within` accepts 30 and refuses 31 with `CoordinateOutOfBounds` |

In `primer_design.rs`'s test module:

| # | Test | Expect |
|---|---|---|
| R1 | a verified insertion is refused | an insertion into `template()`, verified, gives `UnsupportedPrimerDesignScope` whose reason contains `insertion`. Read the flanking bases from the template; do not hard-code them. |

## 5. Permitted changes to existing tests

Adding the variant breaks some exhaustive matches. In each, add one arm that
mirrors the neighbouring `Deletion` arm, with `insertion` in place of
`deletion`, and change nothing else:

- `crates/core/tests/mutation_fixtures.rs` (~line 843);
- `crates/core/tests/mutation_invariants.rs` (~lines 1066 and 1132);
- the unit test `a_result_that_does_not_follow_from_its_source_fails_verification`
  in `mutation.rs`.

## 6. Do not

- Do not change `NucleotideInterval`, `create`, `EmptyInterval` or anything
  else existing in `interval.rs`.
- Do not touch `primer.rs`, `q5.rs` or `q5_design.rs`. Do not add error
  variants or edit `error.rs`.
- Do not change existing tests beyond §5, or any fixture, or [docs/](/).

## 7. Checks

The five in `AGENTS.md`, with `export TMPDIR=$PWD/target/tmp`. Expected: 367 tests
(348 + 19), none failing.

## 8. Handoff

Changed files, exact check results, added test names, anything unclear. If
something is ambiguous, stop and report rather than choosing.

## 9. Outcome, 2026-10-15

Two worker runs: the first was cut off by the harness's time limit with the
whole layer and every test written but the §5 arms missing, so the integration
tests would not compile; a short resumed run added them.

Planner-run checks: fmt, clippy `-D warnings` and `git diff --check` pass, and
**367 tests pass (348 + 19), 0 failures**.

**Planner review.** `apply_insertion` checks identity, bounds, then the ends,
then scope, then both flanking bases, and only then writes, so no index is
taken before its bound is checked. `verify_insertion` re-derives the length,
the placement and both flank claims.

**Mutation check.** Four perturbations, each caught:

| Perturbation | Failures |
| --- | --- |
| insert one base late (off-by-one) | 6 |
| skip verify's length rule | 1 (`verify_refuses_an_insertion_that_lengthens…`) |
| skip verify's placement rule | 1 (`verify_refuses_an_insertion_at_another_place`) |
| skip apply's "base before" check | 1 (`a_wrong_flank_names_the_position…`) |

**Deviations, all accepted:**

1. §5 was incomplete again: two matches in `mutation.rs`'s own test helpers
   (`deletion_record`, `tampered_deletion`) also needed an arm.
2. The §3.6 refusal sits next to the existing deletion refusal rather than
   immediately after the verified-result check, because `policy` is bound
   later. An insertion on a circular template therefore reports the circular
   refusal first. Accepted: both are `UnsupportedPrimerDesignScope`.
3. `verify_insertion` also guards `p == 0` and `p == n`, which `apply` already
   refuses. Reachable only through a tampered record, and untested — recorded
   by the worker as reasoned rather than verified.
