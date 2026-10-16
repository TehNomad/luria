# Review fix C5, C13: minus-strand and origin-wrapping coverage

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review. Every expectation held on first run; no non-test code changed.
Planner: Claude. Parent: `docs/tasks/review-post-m4.md` findings C5, C13.

## 1. The gaps

- **C5.** `docs/q5-primer-design.md` §6 says: "A residue edit whose semantics
  are on a reverse CDS is the same DNA edit and produces the same pair". No
  test checks this. There are no minus-strand tests in the test code of
  `q5.rs`, `q5_design.rs`, `primer.rs` or `q5_expand.rs`. There are no
  minus-strand deletion, insertion or replacement tests.
- **C13.** Nothing tests an insertion at point 0 or `n` on a circular
  sequence, or an indel scoped to a CDS that wraps the origin.

## 2. Rules being pinned (inline)

- **D032.** "The request states the bases at `p − 1` and `p` … So
  `1 ≤ p ≤ n − 1`: an insertion needs a base on both sides." This holds for
  every topology.
- **Scoped containment** (`mutation.rs` `require_within_feature`, after B3).
  A scoped deletion or replacement region must lie wholly inside **one**
  interval of the CDS. A scoped insertion point must satisfy
  `start < p < end` for one interval.
- **Frameshift warnings** (`q5.rs` `edit_warnings`):
  - a deletion warns with `Frameshift { coding_bases_removed, wholly_inside }`
    when the number of removed coding positions is not 0 and not a multiple
    of 3;
  - an insertion warns with `InsertionFrameshift` when the point is inside
    and `k` is not a multiple of 3;
  - a replacement warns with `ReplacementFrameshift` when it has coding
    positions and `m − w` is not a multiple of 3.

  None of these reads the CDS strand.

Do **not** test a region or point that crosses the join of two CDS intervals.
That is open owner question A4.

## 3. Hand-derived facts for the expectations

- **mut-a**, linear, 30 bases: `ATGGCTTGGAATCAGCAGCAGCAGCAGCAG`.
  - Index 9 = `A`, 10 = `A`, 11 = `T`, 12 = `C`.
  - A minus-strand complete CDS over `[0,30)` translates
    `L L L L L L I P S H` (`tests/fixtures/mutation/README.md`), so it is a
    valid annotation.
- **mut-c**, circular, 24 bases: `GCTTGGAACGAACGAACGAACATG`.
  - Index 0 = `G`, 1 = `C`, 2 = `T`, 3 = `T`, 4 = `G`, 22 = `T`, 23 = `G`.
  - The CDS `cds-1`, plus strand, complete, over `[22,24)+[0,7)`, reads
    `TGG CTT GGA` (`W L G`). It is the mutation fixture's `res-c-origin`
    annotation.
- Deleting `[2,5)` (`TTG`) from mut-c leaves
  `GCGAACGAACGAACGAACATG` (21 bases, still circular).
- Inserting `A` at point 23 of mut-c (before = `T`, after = `G`) gives
  `GCTTGGAACGAACGAACGAACATAG` (25 bases).

## 4. Required tests

Use each module's existing helpers. Add a small helper only where none fits.

### 4.1 `crates/core/src/primer_design.rs` (C5, the §6 claim)

`a_minus_strand_residue_edit_designs_the_same_pair_as_its_dna_edit`. On the
module's `template()`:
- **(a)** `codon_edit(&template(), 30, Strand::Minus, "P", "W", "TGG")`. The
  existing test `adjacent_changes_fall_back_to_the_matched_forward_tm`
  documents that this writes `CCA` at `[30,33)`;
- **(b)** `codon_edit(&template(), 30, Strand::Plus, "G", "P", "CCA")`. The
  plus-strand reading of `[30,33)` is `GGG` (G), and the supplied codon `CCA`
  writes the same bases;
- **(c)** the module's `replacement` helper over `[30,33)` writing `CCA`, if
  its signature allows that. Otherwise leave (c) out and say so.

Design all of them with `request(15)`. Assert that all designs are equal in:
- forward and reverse primer sequences;
- forward and reverse spans;
- `edit_span()`;
- forward Tm, reverse Tm and annealing temperature (`celsius()`), and the
  forward Tm basis.

If any of these differ, **stop and report**. Do not adjust the test: a
difference would contradict §6.

### 4.2 `crates/core/src/q5.rs` (C5, warnings on a minus-strand CDS)

On mut-a with a single minus-strand CDS over `[0,30)`, each request scoped to
it and run through `execute`:

1. `a_frameshift_deletion_in_a_minus_strand_cds_warns`: delete `[9,10)`
   (`A`). Expect exactly
   `[Frameshift { cds, coding_bases_removed: 1, wholly_inside: true }]`.
2. `an_in_frame_deletion_in_a_minus_strand_cds_does_not_warn`: delete
   `[9,12)` (`AAT`). Expect no warnings.
3. `a_frameshift_insertion_in_a_minus_strand_cds_warns`: insert `A` at point
   12 (before `T`, after `C`). Expect
   `[InsertionFrameshift { cds, bases_inserted: 1 }]`.
4. `a_frameshift_replacement_in_a_minus_strand_cds_warns`: replace `[9,11)`
   (`AA`) by `G`. Expect
   `[ReplacementFrameshift { cds, bases_replaced: 2, bases_written: 1,
   wholly_inside: true }]`.

Each run must also be verified (`is_verified()`).

### 4.3 `crates/core/src/mutation.rs` (C13)

On mut-c (circular) with the origin-wrapping `cds-1`:

1. `an_insertion_at_either_end_of_a_circular_sequence_is_refused`:
   `apply_insertion` at point 0 and at point 24 is refused. Assert the
   variant that `apply_insertion` returns for the D032 end rule. Read it
   from the code around the reason "an insertion needs a base on each side of
   the point", and confirm it matches D032 before asserting.
2. `a_deletion_inside_the_post_origin_interval_of_a_wrapping_cds_is_applied`:
   - delete `[2,5)` scoped to `cds-1`;
   - the edited canonical text is `GCGAACGAACGAACGAACATG`, its topology is
     circular, and `verify` accepts it.
3. `a_deletion_straddling_the_start_of_the_pre_origin_interval_is_refused`:
   - delete `[21,23)` scoped to `cds-1`;
   - expect `Err(DomainError::TargetOutsideFeature(_))`. `[21,23)` lies
     wholly in neither `[22,24)` nor `[0,7)`.
4. `an_insertion_inside_the_pre_origin_interval_of_a_wrapping_cds_is_applied`:
   - insert `A` at point 23 scoped to `cds-1`;
   - the edited text is `GCTTGGAACGAACGAACGAACATAG` and `verify` accepts it.

### 4.4 `crates/core/src/q5.rs` (C13, warning on a wrapping CDS)

`a_frameshift_deletion_in_an_origin_wrapping_cds_warns`:
- on mut-c with `cds-1` as the only annotation, scoped, delete `[2,3)`
  (`T`) through `execute`;
- expect
  `[Frameshift { cds: cds-1, coding_bases_removed: 1, wholly_inside: true }]`.

## 5. Do not

- Edit files other than `primer_design.rs`, `q5.rs` and `mutation.rs` under
  `crates/core/src/`.
- Change non-test code. These are coverage tests: if one fails, stop and
  report. Do not fix the code or adjust the expectation.
- Change existing tests or fixtures, or add dependencies.
- Test across an interval join (A4).

## 6. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

## 7. Handoff

Report the changed files, the new tests, the exact check results, anything
that failed on first run, and any deviation.
