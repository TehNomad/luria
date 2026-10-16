# Fixture: primer-quality thresholds

Status: complete for the sequence-only checks. Hairpin, self-dimer and
primer-dimer checks are **not** here; see "Not included".
Date: 2026-10-14

These are the thresholds behind `crates/core/src/primer_quality.rs`. Breaking
one produces a **warning** on a design that is still emitted — not a rejection
(owner, 2026-10-14). They are rules of thumb from vendor and protocol
guidance, not biological requirements, and a primer that breaks one can still
work; D030's Tm floor is the only hard thermodynamic requirement.

`crates/core/tests/primer_quality_thresholds.rs` checks that the Rust
constants still equal `thresholds.tsv`, so this fixture is the authority
(D018).

## Each threshold and its source

**GC content 40–60 %.** NEB, *Guidelines for PCR Optimization with Taq DNA
Polymerase*: "Ideal GC content is 40-60%". Agilent, *PCR Primer Design
Guidelines*: "GC Content 40% to 60%". Two independent sources agree.

**Runs of one base: 4 or more warned about.** Takara, *Designing primers*:
"Avoid long runs of a single base (more than three) as this can cause primer
slippage and contribute to mispriming." Thermo Fisher, *PCR Primer Design
Tips*: "Try to avoid runs of 4 or more of one base". Two sources agree.

**Dinucleotide repeats: 5 or more warned about.** **The sources disagree, and
the disagreement is recorded rather than resolved.** Takara: "The maximum
number of di-nucleotide repeats in a primer is four (e.g., ATATATAT)", so four
is acceptable and five is not. Benchling, *Primer Design*, and Thermo Fisher:
"avoid ... four or more dinucleotide repeats (e.g., ATATATATAT)", so four is
already too many. luria takes the **more permissive** reading — warn at five —
because a warning that fires on a sequence one source calls acceptable is
noise. A caller wanting the stricter reading has the repeat count in the
warning and can apply its own threshold.

**G/C run at the 3′ end: 3 or more warned about.** Agilent: "Avoid polybase
sequences (3 or more) Gs and Cs at the 3' end." Thermo Fisher, on the GC
clamp: "Be mindful not to have [too many] G or C bases [at the 3′ end], as
this can cause primer-dimer formation." Takara, on complementary 3′ ends:
primers "should not have complementary G or C stretches (>4 contiguous
bases)".

This check interacts with D030 and is the reason it is worth having: D030
extends a primer *toward* a G or C last base, so it can produce an ending like
`…GGG`. A single G or C at the 3′ end is the intended GC clamp; three or more
is the thing to avoid. The warning distinguishes them.

**Length 18–30 nt.** Agilent: "Length 18 to 30 nucleotides". NEB,
*Guidelines for PCR Optimization with Taq DNA Polymerase*: "Generally 20-30
nucleotides in length".

Two caveats on length, recorded because this warning will fire on legitimate
designs:

- D030 lets a reverse primer start at 15 bases, below the 18 the guideline
  gives, so a short reverse primer warns. That is informative, not wrong.
- D030 grows a primer until it reaches 55 °C, which on an AT-rich site can
  take well over 30 bases. The warning then says the primer is unusually long;
  the Tm floor is the reason, and the design records both.

These guidelines are written for ordinary two-primer PCR, not for the
back-to-back primers of Q5 site-directed mutagenesis, where NEBaseChanger
itself emits 23–24 nt primers. They are applied here as the only published
numbers available, and the mismatch of context is why they warn rather than
reject.

**Ordering limit: longer than 100 nt warned about.** Owner, 2026-10-15
(D032): 100 nt is the practical limit for ordering a standard oligo from IDT.
It is a vendor limit, not a PCR guideline, so it is a separate warning from
the 18–30 nt length guideline, and a 120-nt primer draws both. It matters for
insertions: D032 puts no cap on insertion length, so a long insertion makes
long tailed primers, and this warning is what tells the user.

## Not included

- **Hairpins, self-dimers and primer-dimers.** These need loop, bulge and
  dimer nearest-neighbour parameters, which have not been sourced or verified.
  Adding them means a sourcing pass like the one in `tests/fixtures/tm/`.
  Deferred by the owner on 2026-10-14; the sequence-only checks come first.
- **"Space GC residues evenly within the primer"** (NEB). No source gives a
  number for "evenly", so no threshold is invented.
- **"Avoid mismatches at the 3′ end"** (Agilent). Under D029 the mutation sits
  at least 10 bases from the forward primer's 3′ end, so this cannot occur in
  a `luriaNative` design; nothing is checked for it.
