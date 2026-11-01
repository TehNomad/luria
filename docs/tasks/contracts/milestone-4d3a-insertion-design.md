# Milestone 4d-3a task contract: insertion primer design (D032)

Status: **Implemented 2026-10-15** by the local worker; accepted on planner review (§9).
Date: 2026-10-15
Planner: Claude. Parent: `docs/tasks/contracts/milestone-4-indels.md`, pass 4. Depends on
4d-1 (insertion results) and 4d-2 (`construct_tailed_primer`, insertion pairs).
Its fixture tests are 4d-3b.

## 1. Goal

`design_primers` designs a verified insertion under D032, reusing D030's
sizing through the existing `size_pair`, exactly as `design_deletion` does.

## 2. D032, inline (approved 2026-10-15)

For an insertion of `k` bases at point `p` on a **linear** template of length
`n`, with bound `b`:

- Both annealing portions start at `first = max(b, MIN_REVERSE)` (15). The
  forward primer anneals to template `[p, p + f)`, the reverse to the reverse
  complement of `[p − r, p)`.
- Tails: if `k ≤ 6`, the forward tail is the whole insertion and the reverse
  primer has none. If `k > 6`, the forward tail is `inserted[k/2 ..]` (the
  last `ceil(k/2)` bases) and the reverse tail is
  `reverse_complement(inserted[.. k/2])`, using integer division.
- **No design possible** if `2 * first > n`: `NoPrimerDesignPossible
  { template_length: n, smallest_pair: 2 * first }`.
- **No design fits** (`NoPrimerDesignFits`, `edit_start: p`, `edit_end: p`):
  - `p < first` → `FivePrimeAvailability { available: p, needed: first }`;
  - `n − p < first` → `ThreePrimeAvailability { available: n − p, needed: first }`;
  - both pushed if both apply, 5′ first, then return;
  - then every non-`ACGT` base in `[p − first, p)` and then `[p, p + first)`,
    ascending, as `AmbiguousTemplateBase`, then return.
- Availability counts outward from `p`, as `design_deletion` counts from `s`
  and `e`.
- **Sizing uses the annealing portions only**, on the matched basis. This is
  exactly `design_deletion`'s sizing with `s = e = p`.
- **Whole-primer Tm** (tail plus annealing portion, matched basis) is also
  computed for each primer and reported. It is never used for sizing, balance
  or annealing temperature.
- The 55 °C floor, the G/C end below 63 °C, the 5 °C balance and
  Ta = min(72, lower Tm + 3) are D030's, unchanged.

## 3. Required changes, all in `crates/core/src/primer_design.rs`

1. `pub const MAX_WHOLE_TAIL: u64 = 6;`, with a doc comment citing D032 and
   the reference tool's help text.
2. Remove the block refusing `result.is_insertion()` ("insertion primer design
   is not implemented yet"). After the circular-template check, next to the
   deletion dispatch, add
   `if result.is_insertion() { return design_insertion(template, result, request); }`.
3. `fn design_insertion(template, result, request) -> Result<PrimerDesign,
   DomainError>` implements §2. Read `p` and the inserted text with a `match`
   on `MutationResult::Insertion(record)` (`record.point().position()`,
   `record.inserted()`). Build the `Sizer` as `design_deletion` does with
   `forward_start = forward_anchor = reverse_end = p`, then call `size_pair`.
   Then build the primers:
   - forward: `construct_tailed_primer(template, result, PrimerRole::Forward,
     forward_tail, annealing text, [p, p + f))`. The forward tail is never
     empty, because an insertion has at least one base.
   - reverse: when the reverse tail is empty,
     `construct_primer(template, result, PrimerRole::Reverse,
     request.placement(), annealing text, [p − r, p))`; otherwise
     `construct_tailed_primer(..., PrimerRole::Reverse, reverse_tail,
     annealing text, [p − r, p))`.
   - then `assemble_pair` and `require_design_constraints`, as for a deletion.
4. `PrimerDesign` gains two private fields,
   `forward_whole_tm: Option<MeltingTemperature>` and
   `reverse_whole_tm: Option<MeltingTemperature>`, with accessors of those
   names. They are `Some(melting_temperature(&whole, &whole, &buffer)?)` for an
   insertion, where `whole` is the primer's full text, and `None` for
   substitutions and deletions. Doc comment: the whole-primer Tm is reported
   because it is what a user's own calculator shows, and is never used for
   sizing (D032).
5. For an insertion: `edit_span() == (p, p)`, `five_prime_flank() == 0`,
   `three_prime_flank()` holds `f`, `reverse_length()` holds `r` (the
   reverse **annealing** length), `forward_tm()` and `reverse_tm()` are the
   annealing portions' Tms. Update those accessors' doc comments to say what
   they hold for an insertion.
6. Warnings: `inspect` is run on each primer's **whole** text, tail included,
   because the guidelines are about the oligo as ordered.
7. `NATIVE_DESIGN_ALGORITHM_VERSION` becomes `"0.4.0"`; its doc comment adds
   that 0.4.0 adds D032 insertion design, with earlier output unchanged.

## 4. Permitted changes to existing tests

- The 4d-1 test `an_insertion_is_out_of_scope` pins behaviour D032 reverses.
  Replace it with U3 below.
- The algorithm-version assertion `"0.3.0"` becomes `"0.4.0"`.

Nothing else.

## 5. Required tests, in `primer_design.rs`'s test module

Use the existing `balanced()` template (100 nt), `request(15)` and the
existing `insertion(sequence, position, inserted)` helper. These values come
from `tests/fixtures/q5-primer/native-insertion.tsv` rows `ins-01` and `ins-04`,
generated by the planner with Biopython before this code existed. Compare Tms
within 0.006.

| # | Test | Expect |
|---|---|---|
| U1 | a short insertion is one forward tail | `insertion(&balanced(), 50, "GAT")`: forward annealing span `50:65`, reverse span `35:50`; forward `tail()` text `"GAT"`, reverse `tail()` is `None`; forward Tm 63.89, reverse 62.17, Ta 65.17; `forward_whole_tm()` 64.53, `reverse_whole_tm()` 62.17; `edit_span() == (50, 50)`; `pair().abuts()`; both Tm bases `MatchedComplement` |
| U2 | a long insertion is split between the primers | `insertion(&balanced(), 50, "GATTACA")`: spans as U1; forward tail `"TACA"`, reverse tail `"ATC"`; Tms 63.89 / 62.17; whole Tms 66.66 / 65.17 |
| U3 | an insertion on a circular template is out of scope | as U1 on a circular copy of `balanced()` gives `UnsupportedPrimerDesignScope` |
| U4 | a substitution reports no whole-primer Tm | any existing substitution design: both `forward_whole_tm()` and `reverse_whole_tm()` are `None` |

## 6. Do not

- Do not change `size_pair`, `Sizer`, `design_deletion` or the substitution
  path's behaviour.
- Do not touch `primer.rs`, `mutation.rs`, `q5.rs`, `q5_design.rs`, any
  fixture or `docs/`. No new error variants.

## 7. Checks

The five in `AGENTS.md`, with `export TMPDIR=$PWD/target/tmp`. Expected: 381
tests (378 + 4 − 1), none failing.

## 8. Handoff

Changed files, exact check results, added test names, anything unclear.

## 9. Outcome, 2026-10-15

Two worker runs. The first was cut off by the harness's time limit, with the
two new accessors taking `self` by value (unlike every other `PrimerDesign`
accessor), which caused five use-after-move errors in the tests. The second
fixed that and finished U1–U4.

Planner-run checks: fmt, clippy `-D warnings` and `git diff --check` pass, and
**381 tests pass (378 + 4 − 1), 0 failures**. U1 and U2 match fixture rows
`ins-01` and `ins-04`, generated before the code existed.

**Mutation check:** moving the whole-tail threshold from 6 to 7, splitting with
ceil instead of floor, and computing the "whole" Tm on the annealing portion
only are each caught (1, 1 and 2 failures). 4d-3b's fixture rows pin these
more thoroughly: `ins-03` sits at the threshold and `ins-04` just above it.

**Noted, accepted:** for a `k ≤ 6` insertion, the forward primer reports
placement `ConfineToTail` (it has a tail) and the reverse primer `Inline` (it
has none). Each primer's placement describes that primer, so the mix is
correct rather than inconsistent.
