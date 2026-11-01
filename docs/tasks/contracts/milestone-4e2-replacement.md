# Milestone 4e-2 task contract: represent, apply and verify a replacement

Status: **Implemented 2026-10-15** by the local worker; accepted on planner review (§9).
Date: 2026-10-15
Planner: Claude. Parent: `docs/tasks/contracts/milestone-4-indels.md`, pass 5. Mirrors
`milestone-4a-deletion.md` and `milestone-4d1-insertion.md`, both implemented.
Use their code in `mutation.rs` as the pattern. The design layer is 4e-3.

## 1. Goal

A DNA-level replacement of a region by a text of any length can be requested,
applied and independently verified. Primer design for it is **not** part of
this task: the design layer must refuse one explicitly (§3.6).

## 2. Requirements, inline (D033, approved 2026-10-15)

- A replacement names a target region `[s, e)` of width `w >= 1` and a
  replacement text of length `m >= 1`. `m` need not equal `w`.
- It asserts the exact bases it replaces, as a deletion does: a mismatch
  refuses the edit (D011).
- The replacement text is `A`, `C`, `G`, `T` only. Ambiguity codes are
  expanded at the request layer, before any edit, and never reach here
  (D033). `expand_degenerate` already exists in `sequence.rs` for that; do
  **not** call it from this layer.
- Verification: the edited artifact has exactly `n − w + m` bases and is
  `source[..s] + replacement + source[e..]`.
- A replacement whose text equals the bases it replaces is refused: it edits
  nothing.

## 3. Required behaviour

### 3.1 The intent, in `crates/core/src/mutation.rs`

```rust
pub struct NucleotideReplacementIntent {
    source: SequenceReference,
    region: NucleotideInterval,
    expected_original: String,
    replacement: String,
}
impl NucleotideReplacementIntent {
    pub fn create(source: SequenceReference, region: NucleotideInterval,
        expected_original: String, replacement: String)
        -> Result<NucleotideReplacementIntent, DomainError>;
    // accessors: source(), region(), expected_original(), replacement()
}
```

`create` refuses with `InvalidMutationRequest`, `request: "replacement"`:

1. `region.reference() != source`, with the deletion's reason text.
2. any letter of either text outside `A`, `C`, `G`, `T`, via
   `single_dna_base` with field names `"expected original"` and
   `"replacement"`, as the deletion does per letter.
3. `expected_original.chars().count() != region.length()`, with a reason
   stating both numbers.
4. `replacement` empty, with a reason containing `"at least one"`.
5. `replacement == expected_original`, with a reason containing
   `"already there"`, mirroring the nucleotide substitution's wording.

### 3.2 The record and the result variant

```rust
pub struct NucleotideReplacement {
    source: SequenceReference,
    edited: Sequence,
    region: NucleotideInterval,
    expected_original: String,
    actual_original: String,
    replacement: String,
    state: MutationState,
}
pub enum MutationResult { …, Replacement(NucleotideReplacement) }
```

Accessors for every field except `source`/`edited`. Every existing `match` on
`MutationResult` in `mutation.rs` gains a `Replacement` arm:

- `source()`, `edited()`, `state()`: the record's fields.
- `target_coordinates()`: the **source** coordinates of the region, ascending
  — `coordinates_of(region)`, as the deletion does. Those are the coordinates
  the edit replaced.
- `is_nucleotide()`, `is_deletion()`, `is_insertion()`: all false.
- Add `pub fn is_replacement(&self) -> bool`.
- `accepted()`: sets the state to `Verified`.
- The second match inside `verify`: `unreachable!`, as the other two are.

### 3.3 Applying

```rust
pub fn apply_replacement(sequence: &Sequence, scope: Option<Cds>,
    result: Identifier, intent: NucleotideReplacementIntent)
    -> Result<MutationResult, DomainError>;
```

In this order, writing nothing until all checks pass:

1. `require_sequence` for `intent.source()` and `intent.region().reference()`;
   `intent.region().within(sequence)?`.
2. If the region is the whole sequence: `InvalidMutationRequest`,
   `request: "replacement"`, reason containing `"the whole sequence"`.
3. If `scope` is `Some(cds)`: `require_sequence(cds.reference(), sequence)?`,
   then the whole region must lie inside one interval of `cds.intervals()`.
   Reuse the deletion's existing private helper rather than writing a second
   one. Otherwise `TargetOutsideFeature`.
4. Read the actual bases `[s, e)`. If they differ from `expected_original`,
   return `ExpectedNucleotideMismatch` with `position: s`, the expected text
   and the actual text, as the deletion does.
5. Build `letters[..s] ++ replacement ++ letters[e..]` and `normalize(result,
   …, sequence.topology())`.
6. Return `MutationResult::Replacement`, state `Calculated`.

### 3.4 Verifying

Handle `Replacement` in its own branch in `verify`, next to the other two,
before the equal-length check. Refuse with `MutationVerificationFailure`
when:

1. `edited.length() != source.length() - w + m`, with a reason containing
   `"replacing <w> bases with <m> gives"`. Compute it without `u64`
   underflow, as `verify_deletion` does.
2. the edited letters are not `source[..s] ++ replacement ++ source[e..]`,
   with a reason naming the region.
3. `source[s..e]` differs from the record's `expected_original` or
   `actual_original`, with a reason containing
   `"the replacement claims do not match the source"`.

### 3.5 Errors

No new `DomainError` variants. Do not edit `error.rs`.

### 3.6 The design layer refuses a replacement

In `primer_design.rs`, `design_primers`, beside the existing insertion and
deletion refusals: if `result.is_replacement()`, return
`out_of_scope(&policy, "luriaNative replacement primer design is not
implemented yet")`.

## 4. Required tests, in `mutation.rs`'s test module

On `seq_a()` (`"ATGGCTTGGAATCAGCAGCAGCAGCAGCAG"`, 30 nt). `[9, 12)` holds
`AAT`. Values derived by plain string slicing (planner, 2026-10-15).

| # | Test | Input | Expect |
|---|---|---|---|
| R1 | create refuses another source | region on another sequence | `InvalidMutationRequest` |
| R2 | create refuses a wrong expected length | `[9,12)`, expected `"AA"`, replacement `"GG"` | `InvalidMutationRequest`, reason contains `2` and `3` |
| R3 | create refuses an ambiguity code | `[9,12)`, `"AAT"` → `"GNG"` | `InvalidMutationRequest`, reason contains `A, C, G, T` |
| R4 | create refuses an empty replacement | `[9,12)`, `"AAT"` → `""` | `InvalidMutationRequest`, reason contains `at least one` |
| R5 | create refuses a no-op | `[9,12)`, `"AAT"` → `"AAT"` | `InvalidMutationRequest`, reason contains `already there` |
| R6 | a shorter replacement | `[9,12)` `"AAT"` → `"GG"` | edited `ATGGCTTGGGGCAGCAGCAGCAGCAGCAG` (29), `Calculated`, `is_replacement()`, the other three kind tests false, `target_coordinates() == [9,10,11]`, `actual_original() == "AAT"`, source unchanged at 30 |
| R7 | a longer replacement | `[9,12)` `"AAT"` → `"GGGGG"` | edited `ATGGCTTGGGGGGGCAGCAGCAGCAGCAGCAG` (32) |
| R8 | an equal-length replacement | `[9,12)` `"AAT"` → `"CCC"` | edited `ATGGCTTGGCCCCAGCAGCAGCAGCAGCAG` (30) |
| R9 | one base to two | `[9,10)` `"A"` → `"GG"` | edited `ATGGCTTGGGGATCAGCAGCAGCAGCAGCAG` (31) |
| R10 | at either end | `[0,3)` `"ATG"` → `"C"`, and `[27,30)` `"CAG"` → `"TT"` | `CGCTTGGAATCAGCAGCAGCAGCAGCAG` and `ATGGCTTGGAATCAGCAGCAGCAGCAGTT` |
| R11 | the whole sequence | `[0,30)`, its full text → `"AAAA"` | `InvalidMutationRequest`, reason contains `the whole sequence` |
| R12 | wrong expected bases | `[9,12)` `"AAC"` → `"GG"` | `ExpectedNucleotideMismatch`, `expected == "AAC"`, `actual == "AAT"` |
| R13 | circular stays circular | circular `seq_a` text, R6's edit | topology `Circular`, text as R6 |
| R14 | scope: inside | `cds_of(12)`, R6's edit | succeeds |
| R15 | scope: straddling | `cds_of(12)`, `[10,14)` `"ATCA"` → `"G"` | `TargetOutsideFeature` |
| R16 | verify accepts | R6's result | `Verified` |
| R17 | verify: wrong length | R6's record with `edited` set to R8's text (30 nt) | `MutationVerificationFailure`, reason contains `replacing 3 bases with 2` |
| R18 | verify: wrong content | R6's record with `edited` set to `ATGGCTTGGTTCAGCAGCAGCAGCAGCAG` (29 nt) | `MutationVerificationFailure` |
| R19 | verify: false claim | R6's record with `expected_original` and `actual_original` set to `"AAC"` | `MutationVerificationFailure`, reason contains `replacement claims` |

In `primer_design.rs`'s test module:

| # | Test | Expect |
|---|---|---|
| R20 | a verified replacement is refused | a replacement on `template()`, verified, gives `UnsupportedPrimerDesignScope` whose reason contains `replacement`. Read the replaced bases from the template rather than hard-coding them. |

## 5. Permitted changes to existing tests

Adding the variant breaks exhaustive matches. In each, add one arm mirroring
the neighbouring `Insertion` arm with `replacement` in place of `insertion`,
and change nothing else: `crates/core/tests/mutation_fixtures.rs`,
`crates/core/tests/mutation_invariants.rs` (two), and any match in
`mutation.rs`'s own test helpers that needs one to compile.

## 6. Do not

- Do not touch `sequence.rs`, `interval.rs` or `q5_design.rs`. No new error
  variants; do not edit `error.rs`.
- **Amended 2026-10-15**, during the run: `primer.rs` and `q5.rs` were also on
  this list, but adding a `MutationResult` variant forces an arm in an
  exhaustive match in each (`insertion_claim`, `insertion_warnings`), so the
  crate cannot compile without them. The worker flagged the contradiction
  mid-run rather than guessing, and added exactly one no-op arm per site,
  mirroring the neighbouring `Insertion` arm. That is now the permitted
  change; nothing else in those two files may be touched.
- Do not call `expand_degenerate` from this layer.
- Do not change existing tests beyond §5, or any fixture, or `docs/`.

## 7. Checks

The five in `AGENTS.md`, with `export TMPDIR=$PWD/target/tmp`. Expected: 425
tests (405 + 20), none failing.

## 8. Handoff

Changed files, exact check results, added test names, anything unclear. If
something is ambiguous, stop and report rather than choosing.

## 9. Outcome, 2026-10-15

One worker run, long enough to need an internal compaction but not cut off.

Planner-run checks: fmt, clippy `-D warnings` and `git diff --check` pass, and
**425 tests pass (405 + 20), 0 failures**.

**The contract was wrong again, and the worker caught it.** §6 forbade editing
`primer.rs` and `q5.rs`, but a new `MutationResult` variant forces an arm in
an exhaustive match in each. The worker reported the conflict while working,
explained that reverting would leave the tree uncompilable, and added one
no-op arm per site. The planner verified both are single lines matching their
neighbours, and amended §6. This is the third contract this milestone where
the planner under-specified the ripple of a new enum variant.

**Mutation check.** Each of `verify_replacement`'s three rules is pinned:
disabling the length rule, the placement rule or the claims rule each fails
exactly one test (R17, R18, R19). An earlier planner run reported the claims
rule as unpinned; that was a scripting error in the planner's patch, which
had not landed. Re-run correctly, it fails R19.

**Independent fixture, written before the design layer exists:**
`tests/fixtures/q5-primer/native-replacement.tsv`, 15 rows. Its row `rep-14`
reproduces `native.tsv`'s `native-01` exactly — the same edit expressed as a
replacement — which is evidence that D033's inline path is D030 generalised
rather than a second geometry. 4e-3 will check luria against it.
