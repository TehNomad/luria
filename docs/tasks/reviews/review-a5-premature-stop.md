# Review A5: warn when an in-frame DNA edit creates a premature stop (D037)

Status: **Implemented 2026-10-16** by the local worker; accepted after one review round (§7).
Planner: Claude. Parent: `docs/tasks/reviews/review-post-m4.md` A5. Decision: D037.
Depends on A4 (D036): a circular CDS is split only at the origin.

## 1. Requirement (D037, inline)

> When a substitution, deletion, insertion or replacement lies wholly inside
> the resolved CDS and keeps its reading frame, the Q5 workflow re-translates
> that CDS on the edited artifact. If a stop codon now appears before the
> CDS's last codon, the run carries a non-blocking `PrematureStop` warning
> naming the CDS and the codon number on the edited CDS. The edit is still
> applied. […] A residue intent states its replacement residue explicitly and
> gets no such warning.

`verify` does not change.

## 2. Required changes, `crates/core/src/q5.rs` only

**2.1 The warning.** Add this variant to `Q5EditWarning`:

```rust
PrematureStop { cds: Identifier, codon_number: u64 },
```

Its `describe()` text is `"the edit puts a stop codon at codon {n} of {cds},
before its last codon"`. Update the enum's doc comment, citing D037.

**2.2 Computing it.** `edit_warnings` becomes
`fn edit_warnings(template: &Sequence, mutation: &MutationResult, target: &Cds) -> Result<Vec<Q5EditWarning>, DomainError>`.
Update its two call sites in `execute` and the set path, passing the
request's template and using `?`.

The existing frameshift logic stays exactly as it is. A frameshift warning
and a `PrematureStop` can't both fire: frameshift needs a frame change, and
`PrematureStop` needs none.

Add a private helper that returns `Result<Option<Q5EditWarning>, DomainError>`
and runs these steps in order:

1. Work out the edit and the length change `delta`:

   | Kind | Edit | `delta` | Lies "inside" when |
   | --- | --- | --- | --- |
   | `Nucleotide` | `[p, p+1)` | 0 | `holds_position(intervals, p)` |
   | `Deletion` | `[s, e)` | `−w` | `holds_region(intervals, s, e)` |
   | `Insertion` | point `p` | `+k` | `holds_point(intervals, p)` |
   | `Replacement` | `[s, e)` | `m − w` | `holds_region(intervals, s, e)` |
   | `Residue` | — | — | always `None` |

   The `holds_*` functions are in `interval.rs`.
2. Return `None` if the edit isn't inside, or if `delta` isn't a multiple of
   3. Use `i64` for `delta`.
3. Return `None` if `translation::translate(template, target)` is an `Err`:
   the CDS didn't translate before the edit, so the rule doesn't apply.
4. Build the CDS's intervals on the edited artifact (`mutation.edited()`), in
   the original order. The edit's "end" is `e` for a region, `p` for a point,
   and `p + 1` for a substitution.
   - the interval that holds the edit becomes `[start, end + delta)`;
   - an interval with `start >= ` the edit's end becomes
     `[start + delta, end + delta)`;
   - any other interval is unchanged.

   Build each with `interval::create(&edited.reference(), …)`, and the CDS
   with `Cds::create(&edited, target.id(), intervals, target.strand(),
   target.completeness(), target.genetic_code().name(), target.numbering())`,
   the same arguments `mutation::rebuild_cds` uses. Propagate errors with `?`.
5. Call `translate(&edited, &shifted)`:
   - `Ok` returns `None`;
   - `Err(DomainError::InternalStopCodon(p))` returns
     `Some(PrematureStop { cds: target.id(), codon_number: p.codon_number as u64 })`;
   - any other error is returned as it is. It can't occur for an in-frame
     ACGT edit of a CDS that translated.

**2.3 Version.** Change `Q5_WORKFLOW_VERSION` to `"0.5.0"`. The workflow's
output changes (D021). If a test pins `"0.4.0"`, stop and report.

## 3. Required tests, in `q5.rs`'s test module

Every value below was worked out by hand by the planner; re-derive each one
before you assert it. `mut-a` is `ATGGCTTGGAATCAGCAGCAGCAGCAGCAG` (30 bases,
linear). Index 5 is `T`; 6–8 are `TGG`; 9–11 are `AAT`. Unless a test says
otherwise, the CDS is `cds-1`: plus strand, `[0,30)`, complete, standard
code, translating `M A W N Q Q Q Q Q Q` with no terminal stop. Each run must
be verified (`is_verified()`).

1. `a_substitution_that_writes_a_stop_warns`: substitute position 7, `G` to
   `A` (`TGG` to `TAG`). Expect exactly
   `[PrematureStop { cds-1, codon_number: 3 }]`.
2. `a_synonymous_substitution_does_not_warn`: substitute position 5, `T` to
   `C` (`GCT` to `GCC`). Expect no warnings.
3. `an_in_frame_insertion_of_a_stop_warns`: insert `TAA` at point 9
   (before `G`, after `A`). Expect `[PrematureStop { cds-1, 4 }]`.
4. `an_in_frame_insertion_without_a_stop_does_not_warn`: insert `GCT` at
   point 9. Expect no warnings.
5. `an_equal_length_replacement_that_writes_a_stop_warns`: replace `[9,12)`
   (`AAT`) with `TGA`. Expect `[PrematureStop { cds-1, 4 }]`.
6. `an_in_frame_unequal_replacement_that_writes_a_stop_warns`: replace
   `[9,12)` (`AAT`) with `AATTAA` (`m − w = 3`). Expect
   `[PrematureStop { cds-1, 5 }]`.
7. `a_frameshift_does_not_also_warn_of_a_stop`: delete `[9,10)`. Expect
   exactly the existing `[Frameshift { cds-1, coding_bases_removed: 1,
   wholly_inside: true }]`.
8. `a_stop_on_a_minus_strand_cds_warns`:
   - use a minus-strand `[0,30)` CDS on `mut-a`. It translates
     `L L L L L L I P S H`; codon 7 (`ATT`) is reference `[9,12)`, read
     reverse-complemented;
   - replace `[9,12)` (`AAT`) with `TTA`. The minus reading becomes
     `revcomp(TTA) = TAA`;
   - expect `[PrematureStop { cds, 7 }]`.
9. `a_stop_in_an_origin_wrapping_cds_warns`:
   - use `mut-c`, which is `GCTTGGAACGAACGAACGAACATG` (24 bases, circular),
     with `cds-1` at `[22,24)+[0,7)`, plus strand, reading `TGG CTT GGA`;
   - insert `TAA` at point 1 (before `G`, after `C`), scoped to `cds-1`;
   - the CDS text becomes `TGG TAA CTT GGA`;
   - the rebuilt intervals are `[25,27)+[0,10)` on the 27-base edit;
   - expect `[PrematureStop { cds-1, 2 }]`.
10. `an_edit_only_partly_inside_the_cds_does_not_warn_of_a_stop`: with a
    plus-strand CDS `[0,18)` on `mut-a` (`ATG GCT TGG AAT CAG CAG`),
    unscoped, delete `[15,21)`. Three of the removed bases are coding, so no
    frameshift, and the edit isn't wholly inside. Expect no warnings.

Reuse the helpers the existing tests use (`minus_cds_of`, `seq_c`, `cds_c`,
and so on) wherever they exist.

## 4. Do not

- Edit any other file. If a test elsewhere matches `Q5EditWarning`
  exhaustively and stops compiling, stop and report it.
- Change `verify` or `mutation.rs`.
- Change the frameshift rules or any existing test. If an existing test now
  carries an unexpected `PrematureStop`, **stop and report it**, with the
  test and the codon.

## 5. Checks

Same as A4 §6.

## 6. Handoff

Report the changed file, the new tests, the exact check results, and any
deviation.

## 7. Amendment on review, 2026-10-16

§2.2 step 4 was wrong for an in-frame deletion that removes a whole CDS
interval. The rebuilt interval came out empty, and `interval::create`
returned `EmptyInterval`, so a valid edit, such as deleting a whole gene,
failed the run. The amended rule:
- drop any rebuilt interval that is empty;
- if no interval remains, there is no warning.

Two tests pin the fix: `deleting_a_whole_cds_does_not_warn_or_fail` and
`deleting_a_whole_interval_of_a_wrapping_cds_does_not_fail`.
