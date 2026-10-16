# Fixture: nearest-neighbour thermodynamic parameters

Status: Phase 1 of `docs/tasks/milestone-3d-tm.md`, complete. The
tandem-mismatch gap below is permanent, and is handled by a documented
fallback rather than by a missing value.
Date: 2026-10-14

These are the published parameters a melting-temperature calculation needs.
No value here was produced by running luria, and no value was written from
memory: each was read from a source and then checked against a second,
independent one, as the task contract requires.

## Files

| File | Contents |
| --- | --- |
| `nn-matched.tsv` | The 10 Watson–Crick doublets, plus initiation, the terminal A·T penalty and the symmetry correction. Complete. |
| `nn-mismatch.tsv` | The 48 doublets containing one internal mismatch. Complete for single mismatches. |

## Where the numbers come from

**Primary source, both files:** SantaLucia, J. Jr & Hicks, D. (2004), "The
Thermodynamics of DNA Structural Motifs", *Annu. Rev. Biophys. Biomol.
Struct.* **33**:415–440. Table 1 is the Watson–Crick set; Table 2 is the
single-mismatch set. Open access, so every value here is checkable without a
subscription.

The 2004 review prints ΔH° and ΔS° for the Watson–Crick doublets, but **only
ΔG°37 for the mismatches** — its footnote directs the reader to the original
papers for ΔH° and ΔS°:

- Allawi, H.T. & SantaLucia, J. Jr (1997), *Biochemistry* **36**:10581–10594
  (G·T);
- Allawi & SantaLucia (1998), *Biochemistry* **37**:9435–9444 (A·C);
- Allawi & SantaLucia (1998), *Biochemistry* **37**:2170–2179 (G·A);
- Allawi & SantaLucia (1998), *Nucleic Acids Res.* **26**:2694–2701 (C·T);
- Peyret, N. *et al.* (1999), *Biochemistry* **38**:3468–3477 (A·A, C·C, G·G,
  T·T).

Four of those five are behind a subscription. So the mismatch ΔH°/ΔS° values
were taken from **Biopython**'s `Bio.SeqUtils.MeltingTemp.DNA_IMM1`, an
independent transcription of exactly those papers, and then verified against
the review's open-access ΔG°37 table by the identity

```
ΔG°37 = ΔH° − 310.15 × ΔS° / 1000
```

This is a real check, not a restatement: ΔH° and ΔS° are independent numbers,
and a transcription error in either shows up as a ΔG°37 that does not match
the printed one. The printed values are rounded to 0.1, which propagates to
about ±0.13 kcal/mol in ΔG°37, so that is the tolerance.

**Result: all 48 single-mismatch motifs verified**, worst deviation 0.063
kcal/mol, well inside the rounding tolerance. The Watson–Crick table verifies
the same way against the review's own ΔG°37 column, worst deviation 0.015
kcal/mol. The verification script is `.tmp/tm/verify_imm.py`; it is a working
script, not part of the fixture, and the `dG37_published` column in
`nn-mismatch.tsv` carries the verifier so the check can be rerun.

## Which Watson–Crick set, and why

Two published sets exist from the same group and they differ:

| | Allawi & SantaLucia 1997 | SantaLucia & Hicks 2004 |
| --- | --- | --- |
| `AA/TT` | −7.9 / −22.2 | −7.6 / −21.3 |
| initiation | per terminal pair: G·C 0.1/−2.8, A·T 2.3/4.1 | 0.2/−5.7 once, plus 2.2/+6.9 per terminal A·T |

`nn-matched.tsv` uses the **2004** set, for three reasons: it is the authors'
consolidated set; the 2004 review presents it in the same table pair as the
mismatch parameters, so the two are meant to be used together; and it is the
set the reference tool uses, which makes the planned comparison against the
captured reference Tm values a comparison of *method*, not of parameter
vintage. The research notes record the reference tool's constants as
initiation ΔS −5.7 / ΔH 200 cal and a terminal A/T term of ΔS +6.9 / ΔH +2200
cal, which is the 2004 set exactly
(`docs/research/nebasechanger-client-investigation.md` §4.2).

Note that Biopython's own default pairs the **1997** table with this mismatch
table. That is a defensible choice too; it is not the one made here, and the
difference is recorded rather than hidden.

## Validation of the implementation

`crates/core/src/tm.rs` was checked two ways, and both are permanent tests.

**Against an independent implementation.** `crates/core/tests/tm_crosscheck.rs`
compares luria with **Biopython 1.88** (`Tm_NN`, `DNA_NN4`, `DNA_IMM1`,
`saltcorr=6`, 150 mM Na, 500 nM primer) on six synthetic sequences: matched,
GC-rich, AT-rich, self-complementary, one mismatch, and two mismatches
separated by a match. **Agreement is exact to three decimal places on all
six.** Two implementations reading the same tables should agree, so this
catches a sign error, a misread doublet, a wrong concentration convention or a
missing correction.

Two conventions had to be settled to get there, and both are recorded in
`tm.rs`:

- **The concentration term.** SantaLucia & Hicks give
  Tm = ΔH°/(ΔS° + R ln C_T), dividing C_T by 4 only for a
  non-self-complementary duplex whose two strands are at equal concentration.
  That case does not arise for a PCR primer, which is in vast excess over the
  template, so C_T is the primer concentration undivided. An early draft had
  this inverted and ran 3.4 °C low on self-complementary sequences; the
  cross-check found it.
- **Self-complementarity.** luria detects it from the sequence and applies the
  symmetry term. Biopython does not detect it — `selfcomp` is a caller flag
  defaulting to false — so the flag must be passed to compare. Biopython's own
  docstring notes that MELTING and Primer3Plus default to C_T/4, which puts
  their values about 1 °C below both of these. A Tm is only comparable
  alongside the convention that produced it, which is why every
  `MeltingTemperature` carries its buffer and basis.

**Against the reference tool's recorded values.**
`crates/core/tests/q5_primer_fixtures.rs` compares all twelve primers of the
six reference designs. On the **matched** basis luria agrees to within
**0.25 °C**, the worst case being 0.17 °C. That validates the whole path at
once: parameters, doublet walk, terminal and symmetry terms, concentration
term and salt correction.

The mismatch-aware basis is **not** comparable to those values, and is not
asserted against them. The fixture's recorded `tm` is a matched-duplex figure:
its generator used no mismatch parameters, and the research notes record the
same number as a hand reconstruction over the pure Watson-Crick table (74.18,
against 74.2397 for the tool's own no-mismatch path). Measured on these
primers, luria's mismatch-aware Tm is **6–7 °C below** its matched Tm.

**That disagrees with the reference tool, and the disagreement is recorded
rather than resolved.** The research notes report the tool's own
mismatch-aware figure as 73.7841 against a matched 74.3490 — a drop of
0.57 °C where the published parameters give 6–7 °C for two mismatches, and
10.6 °C on the synthetic 23-mer. Biopython reproduces luria's figure exactly,
so luria is computing what the published papers specify. The reference tool's
mismatch handling therefore differs from the published model; what it does
instead has not been established, and no agreement with it is claimed on this
basis (D014: a comparison against external software is development
validation, not proof).

## The tandem-mismatch gap

**The published single-mismatch set cannot compute a mismatch-aware Tm for an
edit that changes two adjacent bases.**

A nearest-neighbour calculation walks overlapping doublets. A single mismatch
puts exactly one mismatched pair in each of the two doublets spanning it, and
both are in `nn-mismatch.tsv`. Two *adjacent* mismatches put a doublet with
**two** mismatched pairs in the middle, and no published set covers those in
general. Biopython's table carries three such entries — `GG/TT`, `GT/TG`,
`TG/GT`, all G·T tandems from the 1997 paper — and they cannot be verified
against the review, whose Table 2 is single mismatches only. They are
deliberately **not** included in `nn-mismatch.tsv`.

How much this matters, counted rather than guessed:

| Edit | Mismatch-aware Tm computable from published data? |
| --- | --- |
| single-base substitution | **always** — one mismatch can never be tandem |
| codon substitution, changed bases not adjacent | **yes** — 890 of 3486 residue-changing sense-codon pairs, 26 % |
| codon substitution, changed bases adjacent | **no** — 2596 of 3486, 74 % |

It reaches our own fixtures. `q5p-01` is `GAT`→`AAC`, which differs at
positions 1 and 3, so it is computable. `q5p-07` variant-b is `CTG`→`GAG`,
which differs at positions 1 and 2, so it is not. Those are the two designs
the native fixture cross-checks against the reference tool.

The reference tool does not have this gap: the research notes record its
nearest-neighbour table as 176 match-and-mismatch entries with `nnloop` and
`nnbulge` methods alongside. What it does for a tandem mismatch has not been
established, and adopting its table is ruled out by the task contract's
sourcing rule (§2 decision 4) and by D028.

**Resolved by the owner, 2026-10-14: a hybrid.** A primer gets the
mismatch-aware Tm where this table covers it, and the matched-duplex Tm where
it does not. Every Tm names its basis, and a fallback carries the reason, so
the two quantities are never reported under one name. No tandem-mismatch value
is guessed, interpolated, or taken from the vendor table.
