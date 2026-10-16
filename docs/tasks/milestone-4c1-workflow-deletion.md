# Milestone 4c-1 task contract: deletions in the Q5 workflow, with a frameshift warning

Status: **Implemented 2026-10-15** by the local worker; accepted on planner review (§8).
Date: 2026-10-15
Planner: Claude. Parent: `docs/tasks/milestone-4-indels.md`. The design-layer
half, `q5_design.rs`, is 4c-2.

## 1. Goal

A Q5 request can ask for a deletion. The workflow applies and verifies it like
a nucleotide substitution. When the deletion shifts the resolved CDS's reading
frame, the run carries a **warning** and the edit still goes ahead.

## 2. Requirements, inline

- **Owner decision, 2026-10-15 (recorded under D031):** a deletion inside a
  CDS whose width is not a multiple of 3 shifts the reading frame. luria
  **warns and still applies it**, because a frameshift is sometimes the
  intent, as in a knockout.
- **Planner's rule for "shifts the frame"**, stated here so it is not
  invented in code:
  - Count the deleted positions that lie inside any interval of the resolved
    CDS. Positions in an intron or outside the CDS do not count.
  - If that count is 0, there is no warning. The deletion is outside the CDS,
    which `docs/q5-sdm.md` §10 decision 1 says proceeds.
  - If the count is not a multiple of 3, warn.
  - The warning records whether the deletion lay wholly inside the CDS.
- **`docs/q5-sdm.md` §10 decision 1:** a DNA edit that names no CDS proceeds
  even outside the CDS. A request that names a scope is checked against it;
  `apply_deletion` already does this with `TargetOutsideFeature`.
- **Milestone 4a API** (`mutation.rs`): `NucleotideDeletionIntent`,
  `apply_deletion(&Sequence, Option<Cds>, Identifier, NucleotideDeletionIntent)`,
  `MutationResult::is_deletion()`, and `target_coordinates()` (the removed
  source coordinates).

## 3. Required changes, all in `crates/core/src/q5.rs`

1. `Q5MutationIntent` gains `Deletion(NucleotideDeletionIntent)`. Update its
   doc comment: there are now two DNA-level branches.
2. Exhaustive matches:
   - `named_cds`: `None`, like `Nucleotide`.
   - `source_reference`: `Some(intent.source())`, like `Nucleotide`.
   - `missing_claim`: `"name a CDS"`.
   - `run_resolved`, operation: `apply_deletion(&template, dna_scope.clone(),
     result.clone(), intent)?`.
   - `run_resolved`, `verify_scope`: `dna_scope.clone()`, like `Nucleotide`.
3. A new public type:

   ```rust
   #[derive(Clone, Debug, PartialEq)]
   pub enum Q5EditWarning {
       Frameshift {
           cds: Identifier,
           coding_bases_removed: u64,
           wholly_inside: bool,
       },
   }
   impl Q5EditWarning {
       pub fn describe(&self) -> String;
   }
   ```

   `describe` returns `"deleting <n> coding bases from <cds> shifts its
   reading frame"`. When `!wholly_inside`, it appends `"; the deletion also
   extends outside the CDS"`.
4. A private function `fn edit_warnings(mutation: &MutationResult, target:
   &Cds) -> Vec<Q5EditWarning>` that implements §2's rule. It returns an
   empty vector for any result that is not a deletion.
5. `Q5WorkflowExecution` gains a private field `warnings: Vec<Q5EditWarning>`
   and `pub fn warnings(&self) -> Vec<Q5EditWarning>`. In `execute` it is empty
   for a clarification. Otherwise it is
   `edit_warnings(&verified, &target)`.
6. `Q5VariantResult` gains the same field and accessor. In `execute_set`,
   each member gets `edit_warnings(&mutation, &target)`.
7. `Q5_WORKFLOW_VERSION` becomes `"0.2.0"`.

No new `DomainError` variants and no edits to `error.rs`.

## 4. Required tests, in `q5.rs`'s test module

Use the existing `template()` (`"ATGGCTTGGAATCAGCAGCAGCAGCAGCAG"`, 30 nt),
`decoy()`, `cds_of(name, length)` (one interval `[0, length)`) and
`cds_name`. Add a helper `deletion_intent(start, end, bases) ->
Q5MutationIntent` on `template()`.

Run each case through `execute(Q5SdmRequest::create(template(), scope,
intent).unwrap(), annotations, Identifier::create("del-run")…)`. All values
were derived by plain string slicing (planner, 2026-10-15).

| # | Test | Setup | Expect |
|---|---|---|---|
| W1 | an in-frame deletion has no warning | `cds_of("cds-1", 30)`, delete `[9,12)` `"AAT"`, no scope | verified; `mutation().is_deletion()`; edited text `ATGGCTTGGCAGCAGCAGCAGCAGCAG`; `warnings()` empty |
| W2 | a one-base deletion warns of a frameshift | `cds_of("cds-1", 30)`, delete `[9,10)` `"A"` | verified; `warnings() == [Frameshift { cds: cds-1, coding_bases_removed: 1, wholly_inside: true }]` |
| W3 | a deletion across the CDS end counts only its coding bases | `cds_of("cds-1", 12)`, delete `[10,14)` `"ATCA"` | verified; `[Frameshift { cds-1, 2, false }]`; `describe()` contains `"outside the CDS"` |
| W4 | a deletion outside the CDS proceeds without warning | `cds_of("cds-1", 12)`, delete `[12,15)` `"CAG"`, no scope | verified; `warnings()` empty |
| W5 | a scoped deletion outside its CDS is refused | as W4, scope `Some(cds_name("cds-1"))` | `TargetOutsideFeature` |
| W6 | only bases inside the CDS's intervals count | a **circular** copy of `template()`'s text; a CDS `cds-1` on it with intervals `[21,30)` then `[0,6)` (crosses the origin; `Strand::Plus`, `Complete`, `"standard"`, `FromCdsStart`); delete `[5,10)` `"TTGGA"` from the circular artifact (`deletion_intent_on`) | verified; `[Frameshift { cds-1, 1, false }]` |
| W7 | a deletion intent on another artifact is refused | `Q5SdmRequest::create(template(), None, Deletion(intent on decoy(), [9,12), "CAT"))` | `SequenceReferenceMismatch` |
| W8 | each set member carries its own warnings | `execute_set` with two `Q5Variant`s, result names `m1` (W2's deletion) and `m2` (W1's deletion), `SeparateVariants`, annotations `[cds_of("cds-1", 30)]` | `results()[0].warnings()` is W2's frameshift; `results()[1].warnings()` empty |

For W6, only position 5 is coding; 6–9 lie outside both intervals.

**Corrected 2026-10-15.** W6 first specified a CDS with an intron, `[0,6)` and
`[9,15)` on the linear template. Milestone 1 refuses that: a linear feature is
one interval, and a circular feature's intervals must be contiguous across the
origin (`interval.rs`, `validate_structure` and `first_gap`). luria therefore
cannot represent an intron, so "intron bases do not count" cannot arise. The
worker found this and stopped instead of weakening the test. W6 now tests the
multi-interval case luria can represent.

## 5. Do not

- Do not add dependencies or error variants, or edit `error.rs`.
- Do not touch `mutation.rs`, `primer.rs`, `primer_design.rs` or
  `q5_design.rs`.
- Do not change any existing test or its expectations. All 337 existing tests
  must pass.
- Do not edit `docs/` or any fixture.

## 6. Checks

```bash
export TMPDIR=$PWD/target/tmp
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

Expected: 345 tests (337 + 8), none failing.

## 7. Handoff

Report changed files, each check's exact result, the added test names, and
anything unclear. If something is ambiguous, stop and report rather than
choosing.

## 8. Outcome, 2026-10-15

This took three worker runs. The first was cut off by the harness's time limit
with six type errors and a moved value outstanding. The second fixed those
and stopped on W6, correctly reporting that the contract's intron case cannot
be represented (see the correction under §4). The third implemented the
corrected W6.

The planner re-ran every check: fmt, clippy `-D warnings` and `git diff
--check` pass, and **345 tests pass (337 + 8), 0 failures**. `edit_warnings`
was read against §2: it counts deleted positions inside any of the CDS's
intervals, returns nothing for a count of 0 or a multiple of 3, and sets
`wholly_inside` when every deleted position was coding. Only `q5.rs` changed.

Worker deviations, all accepted: a second helper `deletion_intent_on` for W7
and W6; three clippy-driven rewrites (`is_multiple_of(3)`, `&str`
comparisons, field shorthand); W5 asserts the variant but not its category.
