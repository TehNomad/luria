---
type: Task Contract
title: "Milestone 4a task contract: represent, apply and verify a deletion"
description: "A DNA-level deletion of a contiguous region can be requested, applied to a sequence, and independently verified."
status: deprecated
tags: [milestone-4]
---
# Milestone 4a task contract: represent, apply and verify a deletion

Status: **Implemented 2026-10-15** by the local worker; reviewed by the planner (§9).
Date: 2026-10-14
Planner: Claude. Parent: [docs/tasks/contracts/milestone-4-indels.md](/tasks/contracts/milestone-4-indels.md) (passes 1 and 2
of its §5, for deletions only, per owner decision 4.6).

## 1. Goal

A DNA-level deletion of a contiguous region can be requested, applied to a
sequence, and independently verified. Primer design for it is **not** part of
this task: the design layer must refuse a deletion explicitly (§3.6).

## 2. Requirements this task depends on, inline

- **D011, expected original.** Every edit states what it expects to find, and
  a mismatch refuses the edit. For a deletion that is the exact text of the
  deleted bases.
- **D013, independent verification.** `verify` re-derives the edit from the
  source and the edited artifact; it does not trust the generator's record.
  A result is `Calculated` until `verify` returns it `Verified`.
- **Owner decision 4.2, 2026-10-14.** Verification keeps a length rule, stated
  per kind: a deletion of `w` bases must shorten the artifact by exactly `w`.
- **Existing rule, kept.** Asserted bases are `A`, `C`, `G`, `T` only: an
  ambiguity code is not a base that can be asserted
  (`single_dna_base` in `mutation.rs`).

## 3. Required behaviour

All new code goes in `crates/core/src/mutation.rs` unless stated.

### 3.1 The intent

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct NucleotideDeletionIntent {
    source: SequenceReference,
    region: NucleotideInterval,
    expected_deleted: String,
}

impl NucleotideDeletionIntent {
    pub fn create(
        source: SequenceReference,
        region: NucleotideInterval,
        expected_deleted: String,
    ) -> Result<NucleotideDeletionIntent, DomainError>;
    pub fn source(&self) -> SequenceReference;
    pub fn region(&self) -> NucleotideInterval;
    pub fn expected_deleted(&self) -> String;
}
```

`create` refuses, each with `DomainError::InvalidMutationRequest` and
`request: "deletion"`:

1. `region.reference() != source` — reason: "the target coordinate names a
   sequence other than the source" (same text as the substitution).
2. any letter of `expected_deleted` not in `A`, `C`, `G`, `T` — reason must
   contain "A, C, G, T".
3. `expected_deleted` length (in chars) `!= region.length()` — reason must
   state both numbers.

An empty region cannot occur: `interval::create` already refuses it.
Do not lowercase-fold `expected_deleted`; the substitution path does not.

### 3.2 The record and the result variant

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct NucleotideDeletion {
    source: SequenceReference,
    edited: Sequence,
    region: NucleotideInterval,
    expected_deleted: String,
    actual_deleted: String,
    state: MutationState,
}
// accessors: region(), expected_deleted(), actual_deleted(), state()

pub enum MutationResult {
    Nucleotide(NucleotideMutation),
    Residue(ResidueMutation),
    Deletion(NucleotideDeletion),
}
```

Every existing `match` on `MutationResult` in `mutation.rs` gains a
`Deletion` arm:

- `source()`, `edited()`, `state()`: the record's fields.
- `target_coordinates()`: the **source** coordinates removed, ascending, i.e.
  `coordinates_of(region)`. Update its doc comment to "Reference coordinates
  the mutation wrote or removed, ascending."
- `is_nucleotide()`: returns **false** for a deletion. Add a doc comment:
  true only for a single-nucleotide substitution.
- `accepted()`: sets the deletion's state to `Verified`.
- Add `pub fn is_deletion(&self) -> bool`.

### 3.3 Applying

```rust
pub fn apply_deletion(
    sequence: &Sequence,
    scope: Option<Cds>,
    result: Identifier,
    intent: NucleotideDeletionIntent,
) -> Result<MutationResult, DomainError>;
```

In this order, writing nothing until all checks pass, never modifying the
source:

1. `require_sequence` for `intent.source()` and `intent.region().reference()`;
   `intent.region().within(sequence)?`.
2. If `region.length() == sequence.length()`: `InvalidMutationRequest`,
   `request: "deletion"`, reason "a deletion may not remove the whole
   sequence".
3. If `scope` is `Some(cds)`: `require_sequence(cds.reference(), sequence)?`,
   then the **whole** region must lie inside **one** of `cds.intervals()`
   (`region.start() >= iv.start() && region.end() <= iv.end()`). Otherwise
   `DomainError::TargetOutsideFeature` with the same payload fields
   `require_within_feature` fills. Add a private helper for this; do **not**
   change `require_within_feature`, which only checks a start.
4. Read the actual bases `[start, end)`. If they differ from
   `expected_deleted`: `DomainError::ExpectedNucleotideMismatch` with
   `position: region.start()`, `expected: expected_deleted`,
   `actual: <actual text>`.
5. Build the edited letters as `letters[..start] ++ letters[end..]` and
   `normalize(result, canonical_text(...), sequence.topology())`. Topology is
   kept: a deletion from a circular sequence gives a circular sequence.
6. Return `MutationResult::Deletion` with state `Calculated`.

### 3.4 Verifying

In `verify`, handle `MutationResult::Deletion(record)` **before** the existing
equal-length check, in its own branch, and leave the substitution and residue
paths byte-for-byte unchanged, including the "a substitution preserves length"
message. The deletion branch refuses with `MutationVerificationFailure` when:

1. `edited.length() != source.length() - region.length()` — reason
   "a deletion of N bases shortens the artifact by exactly N" with `N` filled
   in;
2. the edited letters are not `source[..start] ++ source[end..]` — reason
   "the edited artifact is not the source with <start>..<end> removed";
3. `source[start..end] != record.expected_deleted` or
   `record.actual_deleted != source[start..end]` — reason "the deletion claims
   do not match the source".

The `cds` argument is ignored for a deletion, as it is for a nucleotide
substitution. On success return `accepted(result)`.

### 3.5 Errors

**No new `DomainError` variants and no edits to `error.rs`.** Every refusal
above reuses an existing variant.

### 3.6 The design layer refuses a deletion

In `crates/core/src/primer_design.rs`, `design_primers`, immediately after the
existing "takes a verified result" check: if `result.is_deletion()`, return
`out_of_scope(<policy name>, "luriaNative designs substitutions; deletion
primer design is not implemented yet")`, i.e.
`DomainError::UnsupportedPrimerDesignScope`. Use whatever the function already
uses for the policy name in its other `out_of_scope` calls.

Without this, a deletion's `target_coordinates()` would be read as a
replacement region and a wrong substitution pair would be designed.

## 4. Required tests

Unit tests in `mutation.rs`'s existing `#[cfg(test)] mod tests`, reusing
`seq_a()` (`"ATGGCTTGGAATCAGCAGCAGCAGCAGCAG"`, 30 nt, linear), `result_name()`
and `cds_of(len)`. Expectations were derived by plain string slicing
(planner, 2026-10-14), not by running luria:

| # | Test | Input | Expect |
|---|---|---|---|
| T1 | create refuses another source | region on a different sequence | `InvalidMutationRequest` |
| T2 | create refuses length mismatch | `[3,6)`, expected `"GC"` | `InvalidMutationRequest`, reason contains `2` and `3` |
| T3 | create refuses ambiguity code | `[3,6)`, expected `"GCN"` | `InvalidMutationRequest`, reason contains `A, C, G, T` |
| T4 | apply deletes a region | `[3,6)`, `"GCT"` | edited text `ATGTGGAATCAGCAGCAGCAGCAGCAG` (27 nt), linear, `Calculated`, `target_coordinates() == [3,4,5]`, `is_deletion()`, `!is_nucleotide()`, `actual_deleted() == "GCT"`, source still 30 nt and unchanged |
| T5 | ends of the sequence | `[0,3)` `"ATG"` and `[27,30)` `"CAG"` | `GCTTGGAATCAGCAGCAGCAGCAGCAG` and `ATGGCTTGGAATCAGCAGCAGCAGCAG` |
| T6 | circular stays circular | same text as `seq_a`, circular, `[3,6)` `"GCT"` | edited topology `Circular`, text as T4 |
| T7 | wrong expected bases | `[3,6)`, `"GCA"` | `ExpectedNucleotideMismatch`, payload `expected == "GCA"`, `actual == "GCT"` |
| T8 | whole sequence | `[0,30)`, the full text | `InvalidMutationRequest` |
| T9 | scope: inside | `cds_of(12)`, `[3,6)` `"GCT"` | succeeds |
| T10 | scope: straddling | `cds_of(12)`, `[9,15)` `"AATCAG"` | `TargetOutsideFeature` |
| T11 | verify accepts | T4's result | `Verified`, otherwise equal to T4's |
| T12 | verify: wrong length | T4's record with `edited` replaced by `ATGTTGGAATCAGCAGCAGCAGCAGCAG` (28 nt, `[3,5)` removed) | `MutationVerificationFailure`, reason contains `3` |
| T13 | verify: wrong content | T4's record with `edited` replaced by `ATGGGGAATCAGCAGCAGCAGCAGCAG` (27 nt, `[4,7)` removed) | `MutationVerificationFailure` |
| T14 | verify: false claim | T4's record with `expected_deleted` and `actual_deleted` set to `"GCA"` | `MutationVerificationFailure` |

T12–T14 build the tampered record directly; the test module can see private
fields. Build the tampered edited `Sequence` with `normalize` under the same
result name and topology.

In `primer_design.rs`'s test module:

| # | Test | Expect |
|---|---|---|
| T15 | a verified deletion is refused | `design_primers` on a verified deletion of any template the module's helpers build returns `UnsupportedPrimerDesignScope`, and the payload reason contains `deletion` |

## 5. Permitted changes to existing tests

Adding the variant breaks three exhaustive matches. In each, **add one arm and
change nothing else**:

- `crates/core/tests/mutation_fixtures.rs` ~line 840,
- `crates/core/tests/mutation_invariants.rs` ~lines 1063 and 1126,

```rust
MutationResult::Deletion(_record) => {
    panic!(<same message as the neighbouring Nucleotide arm, with "deletion">);
}
```

## 6. Do not

- Do not add dependencies, new error variants, or edit `error.rs`.
- Do not change `interval.rs`, `require_within_feature`, the substitution or
  residue paths of `apply_*` or `verify`, or any existing test or fixture
  beyond §5.
- Do not touch `q5.rs` or `q5_design.rs`: the workflow cannot request a
  deletion yet, and that is a later pass.
- Do not add insertion, `InsertionPoint`, or any tail logic.
- Do not edit any file under [docs/](/).

## 7. Checks

From the repository root, with `TMPDIR` set (sandbox requirement):

```bash
export TMPDIR=$PWD/target/tmp
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Before the change the workspace has 310 passing tests; after it, 310 plus the
15 above, and none failing.

## 8. Handoff

Report changed files, each check's exact result, the test names added, and
any point where this contract was unclear or could not be followed as
written. If something is ambiguous, stop and report rather than choosing.

## 9. Outcome, 2026-10-15

Implemented by the local worker in two resumed runs (the first ran out of
turns with the library not yet compiling), plus one review round. Planner
review of the diff found it faithful to §3; the planner re-ran every check
independently:

```
cargo fmt --all -- --check                              pass
cargo test --workspace                                  pass, 325 tests (310 + 15), 0 failures
cargo clippy --workspace --all-targets -- -D warnings    pass
git diff --check                                         pass
```

Changes to existing tests were exactly the three §5 arms.

**Deviations the worker reported, all accepted:**

1. §5 was incomplete. Two more exhaustive matches needed a `Deletion` arm to
   compile: the second match inside `verify`, which got `unreachable!` since a
   deletion returns from its own branch earlier, and the existing unit test
   `a_result_that_does_not_follow_from_its_source_fails_verification`, which
   got an arm mirroring its `Residue` arm. No expectation changed.
2. The length rule is written `edited.length() + width != source.length()`
   rather than `edited.length() != source.length() - width`, to avoid `u64`
   underflow on a tampered region longer than the artifact. Equivalent for
   every well-formed region.
3. In `design_primers` the policy name is bound inside the new refusal block,
   because the function's own `let policy` comes after the placement check.

**Planner mutation check.** Disabling each of `verify_deletion`'s three
checks in turn: the content check and the claims check are each caught by a
test (T13, T14). The length check was not caught on its own: an artifact equal
to "source minus region" has the right length by construction, so the length
rule is a fail-fast with a clearer message rather than an independent
guarantee, and T12's assertion (`reason` contains "3") was satisfied by the
content check's message too. T12 now asserts the length rule's own message.

**Still not done, by design:** the Q5 workflow cannot request a deletion, and
the design layer refuses one. Deletion primer design is pass 3 of
[docs/tasks/contracts/milestone-4-indels.md](/tasks/contracts/milestone-4-indels.md) §5 (decision 4.3: reverse end + deleted
width = forward start).
