---
type: Task Contract
title: "Milestone 3e task record: primer-quality warnings"
description: "A design that breaks a guideline is still emitted, with the guideline it broke recorded against the primer."
status: deprecated
tags: [milestone-3]
---
# Milestone 3e task record: primer-quality warnings

Status: Implemented 2026-10-14 (sequence-only checks).
Date: 2026-10-14
Planner: Claude. Owner decisions in §1.

## 1. Owner decisions, 2026-10-14

1. **Warnings, not rejections.** A design that breaks a guideline is still
   emitted, with the guideline it broke recorded against the primer. These are
   published rules of thumb, not biological requirements; D030's 55 °C Tm floor
   remains the only hard thermodynamic rule.
2. **Sequence-only checks first.** G+C content, runs of one base, dinucleotide
   repeats, the G/C run at the 3′ end, and length. Hairpin, self-dimer and
   primer-dimer checks are deferred: they need loop, bulge and dimer
   nearest-neighbour parameters that have not been sourced, and sourcing them
   is a pass of its own like `tests/fixtures/tm/`.

## 2. What was implemented

`crates/core/src/primer_quality.rs`:

- `PrimerWarning`, five variants, each carrying the measurement it made rather
  than only the fact of a warning, so a caller applying a stricter or looser
  threshold can;
- `inspect(primer) -> Vec<PrimerWarning>`, in a fixed order — length, G+C, then
  sequence features by position — so two equal primers always report
  identically;
- the seven thresholds as named constants.

`PrimerDesign` gains `forward_warnings()`, `reverse_warnings()` and
`warnings()`, the last labelling each warning with the primer it is about.
The warnings are computed from the primers the design actually emitted.

Nothing in this module reads a template, an edit or a Tm: a warning is a
property of the primer text alone.

## 3. Thresholds and their sources

Recorded per threshold in `tests/fixtures/primer-quality/README.md`, with the
source text quoted. `crates/core/tests/primer_quality_thresholds.rs` keeps the
Rust constants equal to `thresholds.tsv`, so the fixture is the authority
(D018), and checks that the bands are coherent.

| Check | Threshold | Source |
| --- | --- | --- |
| G+C content | outside 40–60 % | NEB Taq guidelines; Agilent |
| run of one base | 4 or more | Takara; Thermo Fisher |
| dinucleotide repeat | 5 or more units | Takara (see below) |
| G/C run at the 3′ end | 3 or more | Agilent; Thermo Fisher |
| length | outside 18–30 nt | Agilent; NEB Taq guidelines |

**One disagreement, recorded rather than resolved.** On dinucleotide repeats
Takara allows four units and Benchling and Thermo Fisher warn at four. luria
takes the permissive reading and warns at five, because a warning that fires
on a sequence one source calls acceptable is noise. The repeat count is in the
warning, so a caller wanting the stricter reading has it.

## 4. Two interactions worth knowing

- **The 3′ G/C check earns its place because of D030.** D030 extends a primer
  *toward* a G or C last base, so it can produce an ending like `…GGG`. One G
  or C is the intended GC clamp; three or more invites primer-dimers. The check
  distinguishes them, so the two rules do not quietly fight.
- **The length check fires on correct designs.** D030 starts a reverse primer
  at 15 bases, under the guideline's 18, and grows either primer past 30 when an
  AT-rich site needs it to reach 55 °C — 45 bases in one test. The warning
  reports an unusual length; the design also records the Tm that required it.
  These guidelines are written for ordinary two-primer PCR, not for
  back-to-back SDM primers, which is part of why they warn rather than reject.

## 5. Not implemented

Hairpins, self-dimers, primer-dimers; "space G/C residues evenly", for which no
source gives a number; and "avoid mismatches at the 3′ end", which D029's
geometry already makes impossible in a `luriaNative` design.

## 6. Checks

```
cargo fmt --all -- --check                             pass
cargo test --workspace                                 pass, 310 tests, 0 failures
cargo clippy --workspace --all-targets -- -D warnings   pass
git diff --check                                        pass
```

Moving `MAX_SINGLE_BASE_RUN` to 4, `GC_LOW_PERCENT` to 20 and
`MAX_THREE_PRIME_GC_RUN` to 3 each made tests fail (2, 1 and 1 failures),
including the fixture-equality test; the restored file passes.

Two defects in this pass were found by its own tests and fixed in the code or
the test, not by weakening an expectation: a wrong bound in the dinucleotide
scan that read past the end of a primer, and a test expectation naming the
wrong repeat unit.
