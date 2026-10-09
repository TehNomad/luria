# Research: NEBaseChanger SDM algorithm and the `polymerase-tm` reimplementation

Status: research-only report. **No Milestone 3 implementation is proposed by this
document as already agreed; Section 22 is a recommendation, not a contract.**

Date: 2026 research pass for Luria.

Scope: determine what the `polymerase-tm` project actually reproduces of NEB's
NEBaseChanger site-directed-mutagenesis (SDM) primer design, compare it with NEB
documentation and with the NEBaseChanger front-end source we were able to
retrieve, and identify what Luria can safely encode as its own deterministic
domain logic.

Evidence tags used throughout:

| Tag | Meaning |
| --- | --- |
| `[NEB-DOC]` | Explicitly documented by NEB (site, help text, manuals, FAQ, tutorials). |
| `[NEBC-JS-v2.5.0]` | Behaviour read directly from the NEBaseChanger front-end JavaScript bundle `index.fc8d957d.js` (version 2.5.0), archived 2024-06-11. |
| `[NEBC-OBS]` | Behaviour observed from captured NEBaseChanger tool output (the SID-1 D404N fixture). |
| `[PTM-CODE]` | Behaviour implemented in `polymerase-tm` source. |
| `[INFERENCE]` | Author's interpretation / our interpretation, not directly evidenced. |
| `[UNKNOWN]` | Evidence insufficient. |

Repository studied: `https://github.com/Dioskurides/polymerase-tm` (HEAD on
2026-03-17, package version 2.3.2). Source read directly (not just the README).

---

## 1. Executive summary

1. `polymerase-tm` is a large, broad, rapidly produced Python package (first
   commit 2026-03-02, ~67 commits in ~2 weeks). Its **Tm/Ta calculator** is the
   mature part and claims verification against the NEB Tm Calculator. Its
   **SDM `BaseChanger`** was added in v2.0.0 and is based on a **single
   extraction** from an NEBaseChanger JavaScript bundle (`index.dfcf8292.js`,
   v2.7.2, which we could not retrieve) plus one round of iteration.
   `[PTM-CODE]` `[INFERENCE]`

2. The README/release-note claim escalated from *"Recreated from the NEB Base
   Changer tool (v2.7.2)"* (v2.0.0 source docstring) to *"Exact reproduction of
   the NEB Base Changer tool (v2.7.2)"* (current) and *"full reimplementation"*
   (release notes) **without any added verification evidence**. `[PTM-CODE]`

3. There is **no test in `polymerase-tm` that compares its SDM output to any
   NEBaseChanger output**. `tests/test_mutagenesis.py` checks only self-
   consistency invariants (non-empty primers, `Ta == min(Tm)+1`, template
   unchanged). `[PTM-CODE]`

4. We recovered an archived NEBaseChanger front-end bundle, `index.fc8d957d.js`
   (v2.5.0, 2024-06-11), from Arquivo.pt, and extracted the actual algorithm:
   - the exact **E. coli K-12 codon-usage table** (fractions);
   - maximum-parsimony selection as plain **Hamming distance**, ties undefined;
   - the exact **Tm model** (SantaLucia 1998 NN, initiation `dS=-5.7`,
     `dH=200` cal/mol, terminal A/T `+6.9`/`+2200`, Owczarzy salt correction at
     **150 mM equivalent monovalent**, **500 nM** primer, method 4);
   - the primer **growth + Tm-balancing** loop (`calcSDMPrimers`). `[NEBC-JS-v2.5.0]`

5. We reproduced the captured SID-1 **D404N** NEBaseChanger result *exactly*
   from that extracted algorithm with an independent Python reimplementation
   (forward 23 nt Tm 74, reverse 23 nt Tm 70, Ta 71). This validates the model
   against at least one real tool output. `[NEBC-OBS]` `[INFERENCE]`

6. Running the actual `polymerase-tm` `BaseChanger` on the same input gives a
   **forward primer identical in sequence but a wrong reported Tm** (62 vs 74),
   a **shorter reverse primer** (19 vs 23 nt, Tm 64 vs 70) and a **different Ta**
   (63 vs 71). So `polymerase-tm` does **not** reproduce NEBaseChanger for this
   example. The forward-sequence match is partly coincidental. `[PTM-CODE]`

7. `polymerase-tm` is therefore **not a suitable independent oracle for exact
   NEBaseChanger behaviour**. It is useful as (a) a second opinion on
   *structure*, (b) a source of extracted constants (codon table, NN parameters,
   salt model) that we then verify independently, and (c) a documented example
   of how easy it is to get this wrong. It must not be used as an authority or a
   test oracle. `[INFERENCE]`

8. We found one other public "knock-off", `mcgawc/base-changer` (2017, 16 KB
   script). It is a hard-coded Perl/Python-2-style helper, not a faithful
   algorithm, and it **disagrees with NEB's own codon table** on Arg and His. It
   is not a usable oracle. No consensus public implementation exists; there is
   essentially no credible independent reverse-engineering beyond the one we
   recovered ourselves. `[INFERENCE]`

9. The live NEBaseChanger site is behind Cloudflare and returned HTTP 403 to
   every fetch attempt from this environment. We could not run interactive
   controlled experiments. This is a documented limitation (Section 17), partly
   mitigated by the archived JS and the captured D404N output. `[UNKNOWN]`

10. Recommended direction: implement Q5 SDM as a Luria-owned deterministic
    workflow from the **documented + JS-verified** rules in Sections 21–23, using
    the D404N example and a small controlled matrix as fixtures, **not** by
    porting `polymerase-tm`.

---

## 2. `polymerase-tm` architecture relevant to `BaseChanger`

Package layout (`src/polymerase_tm/`):

| File | Relevant content |
| --- | --- |
| `core.py` | `calc_nn_raw`, `owczarzy_correction` (2004 mono), `owczarzy_bivariate` (claimed 2008), `calc_sdm_tm`, `tm`, `ta`. |
| `constants.py` | `NN_PARAMS` (SantaLucia 1998), `TERMINAL`, `BUFFERS`, `POLYMERASES`, `MISMATCH_NN`, `CODON_TABLE`, `AA_TO_CODONS`, `ECOLI_CODON_USAGE`, `GENETIC_CODES`, `Q5_SDM_*` constants. |
| `mutagenesis.py` | `SDMPrimer`, `MutagenesisResult`, `select_codon`, `parse_aa_mutation`, `_hamming`, `_build_primer`, `_balance_primers`, `BaseChanger`. |
| `batch.py` | `reverse_complement`, CSV/`from_csv` SDM entry points. |
| `cli.py` | `--sdm` handler. |

Key defaults, `BaseChanger.__init__`
(`src/polymerase_tm/mutagenesis.py`):

```text
circular=True, min_length=15, min_tm=55, max_tm=72 (Q5_SDM_MAX_TM),
confine_to_tails=False, preserve_degenerate=False,
merge_mutations=False (stored but NEVER referenced again),
codon_mode="usage", orf_start=1 (1-based), genetic_code=1
```

`Q5_SDM_*` constants (`constants.py`): `MONO_MM=50`, `DIVALENT_MM=2`,
`PRIMER_NM=500`, `MIN_TM=55.0`, `MAX_TM=72.0`, `MIN_LEN=15`,
`MIN_ANNEAL=10`.

Call graph for an amino-acid point mutation:

```text
BaseChanger.point_mutation(str)
  -> parse_aa_mutation(str)
  -> for each: _design_point_mutation(mut)
       -> _codon_at(pos)                       # original codon
       -> select_codon(new_aa, original, mode) # usage / parsimony
       -> mutated = template[:i] + new_codon + template[i+3:]
       -> _design_substitution_primers(mutated, i, 3, new_codon)
            -> (confine_to_tails or mut_len>6) ? _design_tail_primers
                                              : inline back-to-back builder
                 -> _build_primer(...) for the reverse primer
       -> ta = min(ceil Tm) + 1, cap 72
```

`substitution`, `deletion`, `insertion` are separate entry points that wrap the
same `_design_substitution_primers` / `_build_primer` helpers.

---

## 3. Exact SDM algorithm flow in NEBaseChanger (recovered)

This is the authoritative description of NEBaseChanger behaviour for an
**inline point substitution**, from `index.fc8d957d.js` (v2.5.0)
`[NEBC-JS-v2.5.0]`, cross-checked against the captured D404N output
`[NEBC-OBS]`.

NEBaseChanger linearises the circular plasmid so that the mutation's **5'
flank begins at index 0**; the mutation/replacement region starts after a fixed
**pad of 10 bases** (`pad = 10`). Let `rot` be that rotated template, `orig` the
wild-type codon, `rep` the replacement.

1. **Forward primer** = `rot[0:10] + rep + rot[10+len(rep) : 10+len(rep)+O]`.
   `O` starts at `pad = 10` (i.e. the right flank initially has 10 bases).
2. Grow the forward primer (`O += 1`) while its Tm `< minTm`.
3. If its Tm `< 63`, keep growing until the 3' base is `G` or `C` (a "GC
   clamp"); if a GC terminus cannot be reached, revert to the shortest
   Tm-satisfying forward primer.
4. **Reverse primer** = `revcomp(rot)[0:ce]` = reverse complement of the `ce`
   bases immediately preceding the forward primer's 5' end (back-to-back).
   `ce` starts at `minLen` (default 15).
5. Grow the reverse primer (`ce += 1`) while its Tm `< minTm`; then apply the
   same GC-clamp / revert rule (`< 63 °C`).
6. **Balance the Tm difference.** If `|Tm_fwd − Tm_rev| > 5 °C`, extend the
   lower-Tm primer until the difference is `≤ 5 °C`; while doing so, prefer a
   GC-clamped 3' end but revert to the last conforming length if extending
   would overshoot the other primer's Tm.
7. **Ta = `min(Tm_fwd, Tm_rev) + 1`, capped at 72 °C.** `[NEBC-JS-v2.5.0]`
8. The displayed Tm values are the **full-primer** Tm (not a sub-region). In the
   D404N case the method-4 full-primer Tm is 74.18 °C and 69.94 °C, displayed as
   74 and 70. `[NEBC-OBS]` `[INFERENCE]`

The mutation sits **inside** the forward annealing region (10 bases of WT on
each side of a 3-nt codon), not in a 5' non-annealing tail, unless the user
selects "Confine mutations to primer 5′ tails" or the replacement exceeds 6 nt.
`[NEB-DOC]` `[NEBC-JS-v2.5.0]`

---

## 4. Codon-selection algorithm

### 4.1 NEBaseChanger (authoritative, recovered)

The bundle contains the exact table `t.cut.Ecoli_K12` as `[three-letter AA,
codon, fraction]`. Full list for the standard amino acids and stops:

```text
Ala GCG .36  GCC .27  GCA .21  GCT .16
Arg CGC .40  CGT .38  CGG .10  CGA .06  AGA .04  AGG .02
Asn AAC .55  AAT .45
Asp GAT .63  GAC .37
Cys TGC .55  TGT .45
Gln CAG .65  CAA .35
Glu GAA .69  GAG .31
Gly GGC .40  GGT .34  GGG .15  GGA .11
His CAT .57  CAC .43
Ile ATT .51  ATC .42  ATA .07
Leu CTG .50  TTG .13  TTA .13  CTT .10  CTC .10  CTA .04
Lys AAA .77  AAG .23
Met ATG 1
Phe TTT .57  TTC .43
Pro CCG .52  CCA .19  CCT .16  CCC .12
Ser AGC .28  AGT .15  TCG .15  TCT .15  TCC .15  TCA .12
Thr ACC .44  ACG .27  ACT .17  ACA .13
Trp TGG 1
Tyr TAT .57  TAC .43
Val GTG .37  GTT .26  GTC .22  GTA .15
Ter TAA .64  TGA .29  TAG .07
All/X NNK
```

- **Maximum codon usage** (`radio value "maxcut"`, `Ee`): filter `Ecoli_K12`
  rows whose three-letter AA matches the requested residue, sort **descending by
  fraction**, take the first codon. Tie behaviour is explicitly **undefined**
  ("could be random" in NEB help). `[NEBC-JS-v2.5.0]` `[NEB-DOC]`
- **Maximum parsimony** (`radio value "mindist"`, `Le`): filter to the
  requested residue's codons, sort **ascending by Hamming/base-change distance**
  to the original codon, take the first. Tie behaviour explicitly **undefined**.
  `[NEBC-JS-v2.5.0]` `[NEB-DOC]`
  - NEB documents this: *"For maximum parsimony, a simple sequence distance is
    calculated as the required number of base changes to go from the original
    codon to the replacement codon."* `[NEB-DOC]`
- The original codon is considered **only** in parsimony mode; usage mode ignores
  it. The selected codon is always one that encodes the target residue.
- Stop codons are selectable (`Ter` rows; `*` is a valid replacement). `X`/`All`
  maps to `NNK`. `[NEBC-JS-v2.5.0]`
- Genetic code: NEBaseChanger uses a codon table (the about page and UI show the
  standard table; the bundle contains `CODON_TABLE_DATA` for tables 1–6, 9–15).
  `[NEBC-JS-v2.5.0]`

### 4.2 `polymerase-tm`

- `ECOLI_CODON_USAGE` (per-thousand, Kazusa-like) is **numerically different**
  from NEBC's fraction table but selects the **same preferred codon for every
  amino acid**. Verified by enumeration of both tables; the two exceptions found
  in the older `mcgawc` script (Arg, His) do **not** apply to `polymerase-tm`.
  `[PTM-CODE]` `[INFERENCE]`
- Usage: `max(codons, key=lambda c: ECOLI_CODON_USAGE.get(c, 0))`, ties resolved
  by Python's stable `max` → **first codon in `AA_TO_CODONS` insertion order**,
  which is generated from `CODON_TABLE` iteration. NEBC leaves ties undefined;
  `polymerase-tm` therefore imposes a deterministic tie-break that is **not**
  NEBC. `[PTM-CODE]`
- Parsimony: min Hamming distance, then **tie-break by `ECOLI_CODON_USAGE`**.
  Again not NEBC (NEBC: undefined/random). `[PTM-CODE]`
- Both use `get_aa_to_codons(genetic_code)`; unsupported residues raise
  `ValueError("No codons found")`. `[PTM-CODE]`
- `X` is **not** supported by `polymerase-tm` (`select_codon("X")` raises);
  NEBC's `X` = `NNK`. `[PTM-CODE]` `[NEBC-JS-v2.5.0]`

**Conclusion:** maximum-usage codon choice agrees; parsimony agrees in the
no-tie case; tie-breaking is a `polymerase-tm`-specific choice and must not be
treated as NEBaseChanger behaviour.

---

## 5. Primer-generation algorithm

### 5.1 NEBaseChanger `[NEBC-JS-v2.5.0]` `[NEBC-OBS]`

Parameters: `pad = 10` (fixed annealing bases adjacent to the mutation on each
side, for inline substitutions), `minLen` default 15 (clamped `<15 → 15`),
`minTm` default 55 (clamped `<55 → 55`, `>72 → 72`), GC-extension threshold
63 °C, Tm-balance tolerance 5 °C, Ta cap 72 °C.

- Forward primer is built **with the replacement in the middle**, then grown
  outward on the 3' side only (`O`), i.e. the 5' flank stays at 10 bases.
- Reverse primer is grown from `minLen`; its 3' end is anchored at the forward
  primer's 5' end (back-to-back).
- Tm is computed on the **whole primer**, mismatch-aware against the wild-type
  context (`calcSDMPrimers` passes a paired wild-type complement to the Tm
  calculator).
- Balancing ensures the two primers end up within 5 °C.
- For replacements > 6 nt, or when "Confine mutations to primer 5′ tails" is
  on, the replacement is split between the two 5' tails: with `len(rep) ≤ 6` the
  whole replacement is a 5' tail on the forward primer; for longer inserts, half
  is added to each primer's 5' end (`rep.slice(half)` forward,
  `revcomp(rep.slice(0,half))` reverse). `[NEBC-JS-v2.5.0]` `[NEB-DOC]`
  - NEB FAQ: for insertions > 6 nt, *"Half of the insert should be added to the
    5' end of the forward primer and the other half to the 5' end of the reverse
    primer."* `[NEB-DOC]`

### 5.2 `polymerase-tm` `[PTM-CODE]`

`_design_substitution_primers` (inline path):

- `min_flank = Q5_SDM_MIN_ANNEAL = 10`.
- `left_start = mut_start − 10`; forward = `circ[left_start:end]` and it grows
  `end` from `mut_start + mut_len + 10`.
- **Forward Tm is computed on the right-hand sub-region only**
  (`circ[mut_start:end]`), and the code comment claims *"NEB computes FWD Tm on
  the right-side annealing region ONLY"*. This is **contradicted** by the
  captured D404N output, where NEBC's forward Tm (74) is the full-primer Tm and
  the right-only region gives 62. `[PTM-CODE]` vs `[NEBC-OBS]`
- Reverse primer: `_build_primer(circ, left_start, "reverse")`.
- `_build_primer` grows from `min_len` and stops at the **first** length whose
  Tm `≥ min_tm` **and whose 3' base is G/C**; if not G/C it keeps extending
  until G/C or `Tm > 72`. This "grow to GC clamp" rule is an author invention
  (NEBC only extends to a GC clamp when Tm < 63 °C, and reverts if it cannot).
- `_balance_primers` exists and is correct in spirit, but is **called only by
  `deletion`**, never by `_design_substitution_primers`, `_design_tail_primers`,
  `insertion`, or point mutation. So point-mutation primers are **not balanced**.
- `fwd_tm_val` fallback indexes `circ[circ_mut_start:left_start+len(...)]`,
  which mixes two coordinate systems and looks like a bug. `[PTM-CODE]`
- Reported primer length is the full primer; reported Tm is `math.ceil` of the
  region/whole-primer Tm. NEBC displays rounded full-primer Tm. `[PTM-CODE]`

### 5.3 Worked comparison — SID-1 D404N (`GAT`→`AAC`, NEB default)

Captured NEBaseChanger output (`tests/fixtures/example-construct/raw/sdmsummary.txt`):

```text
FWD  CGTTGAGCAAaacTTCGGGGCGC   len 23  Tm 74
REV  AGCGGATTGAGCAGTTTTCCGTC   len 23  Tm 70
Ta   71
```

Independent Python reimplementation of the extracted NEBC algorithm (Section 3)
reproduces exactly:

```text
FWD  CGTTGAGCAAAACTTCGGGGCGC    len 23  Tm 74.18 -> 74
REV  AGCGGATTGAGCAGTTTTCCGTC    len 23  Tm 69.94 -> 70
Ta   min(74,70)+1 = 71
```

Actual `polymerase-tm` `BaseChanger(wt, orf_start=1).point_mutation("D404N")`:

```text
FWD  CGTTGAGCAAaacTTCGGGGCGC   len 23  reported Tm 62  (right-13-mer Tm)
REV  AGCGGATTGAGCAGTTTTC       len 19  reported Tm 64
Ta   63
```

Classification: forward **sequence** exact match, forward **Tm** clearly
different (wrong region); reverse primer clearly different (length, sequence,
Tm); Ta clearly different.

---

## 6. Primer-selection / ranking algorithm

- NEBaseChanger does not enumerate and rank candidates in the common case. It
  deterministically constructs one primer pair by the growth + balancing loop.
  Ambiguous replacement sequences are the exception: they are expanded
  combinatorially (Section 10), producing one primer pair per concrete
  replacement; there is no published scoring/ranking among them.
  `[NEBC-JS-v2.5.0]` `[NEB-DOC]`
- `polymerase-tm` likewise constructs one pair; there is no candidate ranking.
  `[PTM-CODE]`
- The only "selection" in either tool is the codon-selection policy
  (Section 4). Ties are undefined in NEBC and deterministically broken in
  `polymerase-tm`. `[INFERENCE]`

---

## 7. Tm formulas

Three separate things must not be conflated (`D014`): general primer Tm, SDM
primer Tm, and the Q5 annealing-temperature recommendation.

### 7.1 General NEB Tm Calculator (`polymerase-tm` `tm()`)

`[PTM-CODE]`, claimed verified against the NEB Tm Calculator with 0 °C
deviation. Uses `calc_nn_raw` (SantaLucia 1998 `NN_PARAMS` + `TERMINAL`
initiation corrections) and either:
- `owczarzy_correction` (Owczarzy 2004 monovalent), the default; or
- `schildkraut_correction` (`+16.6·log10([Na+]/1M)`), used for Phusion; or
- primer3, if `method="primer3"`.

Buffer salt table (`BUFFERS`) is explicitly noted as extracted from the NEB Tm
Calculator JS (`main-90540d67e2.js`, v1.16.10). Q5 = 150 mM, Taq = 51 mM, etc.
Primer concentrations: Q5/Taq 500/200 nM. `[PTM-CODE]`

### 7.2 NEBaseChanger SDM Tm (recovered, authoritative)

Method 4 = **SantaLucia 1998 nearest-neighbour** with:
- initiation `ΔS = −5.7 cal/(mol·K)`, `ΔH = 200 cal/mol`;
- `+6.9 / +2200` per terminal A or T;
- gas constant `R = 1.987`;
- self-complementarity penalty `ΔS = −1.4` if applicable;
- primer concentration **ct = 0.5 µM (500 nM)** (`primerc = ct·10⁻⁶ M`);
- salt = **150 mM equivalent monovalent** for Q5, **divalent = 0**;
- correction `sc_ow`: if divalent/`[Na⁺]` ratio `< 0.22`, the monovalent
  Owczarzy form `(4.29·f_GC − 3.95)·10⁻⁵·ln[Na⁺] + 9.40·10⁻⁶·(ln[Na⁺])²`;
  otherwise the full bivariate form with salt-dependent coefficients.
- `Tm = 1 / (1/Tm_1M + sc_ow) − 273.15`, then `− 0.6·DMSO%`.
- Reported as the full-primer Tm. `[NEBC-JS-v2.5.0]`

NEB lists the SantaLucia/Allawi/SantaLucia-Hicks/Bommarito/Owczarzy-2004
references in the tool's references panel and states Tm is *"corrected for
buffer composition … assuming the recommended buffer composition for Hot Start
High-Fidelity DNA Polymerase, and the preferred primer concentration
(500 nM)"*. `[NEB-DOC]`

**Independent verification:** our reimplementation gives FWD 74.18 °C and REV
69.94 °C for the D404N primers, matching the captured 74/70.

### 7.3 SDM Tm in `polymerase-tm` (`calc_sdm_tm`)

`[PTM-CODE]`

```text
calc_sdm_tm(seq, mono_mM=50, divalent_mM=2, primer_conc_nM=500, dmso_pct=0)
  = owczarzy_bivariate(calc_nn_raw(seq, 500), seq, 50, 2) - 0.6*dmso
```

- `calc_nn_raw` uses the **same SantaLucia 1998 NN table** but the wrong
  initiation convention: it adds `TERMINAL[first]` and `TERMINAL[last]`
  (A/T `+2.3 kcal, +4.1 cal/K`; G/C `+0.1, −2.8`) instead of NEBC's
  `−5.7/200` init plus `+6.9/+2200` terminal A/T. `[PTM-CODE]`
- `owczarzy_bivariate` (in `core.py`) claims Owczarzy 2008 and uses
  **static coefficients** `a=3.92e-5, b=−9.11e-6, c=6.26e-5, d=1.42e-5,
  e=−4.82e-4, f=5.25e-4, g=8.31e-5`, and **regimes 2 and 3 are identical**
  (dead branch). This is the IDT simplified form, not NEBC's salt-dependent
  coefficient set. `constants.py::OWCZARZY_BIVARIATE` contains *different*
  coefficients that are not the ones used. `[PTM-CODE]`
- Conditions used (50 mM Na⁺ + 2 mM Mg²⁺) differ from NEBC's effective
  Q5 condition (150 mM monovalent, no explicit Mg²⁺).
- Net effect on D404N: `calc_sdm_tm` gives FWD23 = 73.18 (vs NEBC 74.18) and
  the right-13-mer = 61.98.

**Conclusion:** the claim *"SDM Tm with Owczarzy (2008) bivariate salt
correction (Na⁺ + Mg²⁺)"* does not match the NEBaseChanger bundle we recovered
(v2.5.0). It may describe a later NEBaseChanger version, but we could not verify
v2.7.2 and must treat it as unconfirmed. `[UNKNOWN]`

---

## 8. Ta formulas

- **NEBaseChanger SDM:** `Ta = min(Tm_fwd, Tm_rev) + 1`, capped at 72 °C.
  `[NEBC-JS-v2.5.0]` `[NEBC-OBS]` (D404N Ta 71).
- **`polymerase-tm` SDM:** `ta = min(fwd_raw_tm, rev_raw_tm) + 1`, capped 72,
  computed from **raw (unrounded) floats**, while the displayed primer `tm` is
  `ceil`. This can make `Ta != min(displayed_tm) + 1`; the unit test
  `test_ta_calculation` asserts equality on the displayed values and is not
  guaranteed by the implementation. `[PTM-CODE]`
- **NEB Tm Calculator general Ta** (`polymerase-tm` `ta()`): polymerase-family
  rule (`min(Tm)+offset`, `0.93·min(Tm)+7.5` for Phusion), cap, `−0.6·DMSO%`,
  with `min_len` gating. Q5: `min(Tm)+1`, cap 72. This is a separate component
  and should not be reused blindly as the SDM Ta. `[PTM-CODE]`
- NEB also documents a manual fallback: *"For predesigned back-to-back primer
  sets, a Tm+3 rule can be applied but optimization may be necessary."*
  `[NEB-DOC]` This is a lab rule, not the tool's rule.

---

## 9. Q5-specific rules

Documented/observed `[NEB-DOC]` `[NEBC-JS-v2.5.0]`:

- Back-to-back primers whose 5' ends abut, exponential amplification, KLD
  cleanup; mutations are not phosphorylated and do not need purification (unless
  long).
- Substitutions: mismatch inside the primer, ≥10 complementary bases at the 3'
  end.
- Insertions > 6 nt: split between the two 5' tails.
- Q5 annealing temperature must account for mismatches; the manufacturer Tm of
  the oligo is not the Q5 Tm. NEBC's Ta is Q5-specific.
- Default minimum primer length 15 nt; default minimum Tm 55 °C.
- **Legacy mode** (emulating v1): minimum primer length reset to **18 nt**, and
  insertions always in the 5' tails. `[NEB-DOC]`
- "Confine mutations to primer 5′ tails": replacement always attached to the 5'
  end; the minimum length applies to the annealing portion only; useful to keep
  Tm uniform across many mutations. `[NEB-DOC]`
- Primers > ~60 nt: HPLC/PAGE purification recommended. `[NEB-DOC]`

`polymerase-tm` encodes `Q5_SDM_MIN_TM=55`, `MIN_LEN=15`, `MIN_ANNEAL=10`,
`MAX_TM=72`, `Ta=min+1 cap 72`, tail-splitting at half the insert. Those match
NEB documentation/conventions, but the **implementation of them does not match
NEBC output** (Sections 5–8). `[PTM-CODE]`

---

## 10. Ambiguous / degenerate base behaviour

- **NEBaseChanger:** accepts IUPAC ambiguity codes in replacement sequences and
  **expands them combinatorially**, generating one primer pair per concrete
  sequence (e.g. `NS` → 8 replacements). Since v2.4.2 an option exists to
  preserve degenerate codes in primers. `[NEB-DOC]` `[NEBC-JS-v2.5.0]`
- **`polymerase-tm`:** `preserve_degenerate=True` only **skips the invalid-
  character check** on the template. It does **not** expand degenerate bases and
  does **not** teach the Tm calculator to accept them. Any candidate containing a
  non-`ACGT` base raises inside `calc_sdm_tm`, is swallowed by
  `except ValueError: continue`, and the growth loop falls through to the
  fallback path. The `BaseChanger` docstring claim *"IUPAC ambiguity codes are
  kept as-is (not expanded)"* describes intent, not a working expansion.
  `[PTM-CODE]`
- Luria decision `D010`/`D005` already requires explicit ambiguity semantics or a
  structured failure; NEBC's combinatorial expansion is a behaviour we could
  adopt deliberately (with an explicit cap) but must not guess. `[INFERENCE]`

---

## 11. Indel behaviour

- **NEBaseChanger:** general indel/substitution uses 5'-flank start and 3'-flank
  end positions; the replaced region is **between** start and end. Empty
  replacement = deletion; consecutive start/end = insertion. For small inline
  replacements (≤ 6 nt) primers are built inline; insertions and replacements
  > 6 nt use 5' tails, splitting inserts between the two primers.
  `[NEB-DOC]` `[NEBC-JS-v2.5.0]`
- **`polymerase-tm`:** `deletion(start, length)` builds forward after the
  deletion and reverse before it, then calls `_balance_primers`.
  `insertion(position, insert_seq)` puts the whole insert on the forward 5' tail
  (lowercase) and does **not** split, and does not balance Tm.
  `substitution(start, replacement, length)` routes through the same builder.
  Coordinate conventions are documented inconsistently (comments say "1-based
  position after which to insert" but `idx = position` is used as a 0-based
  slice index). `[PTM-CODE]`

---

## 12. Multiple-mutation behaviour

- **NEBaseChanger:** point-mutation mode accepts a list of `[old][pos][new]`
  mutations and (in NEBuilder mode) can build multi-site primer pairs that
  produce assemblable fragments. `[NEB-DOC]` `[NEBC-JS-v2.5.0]`
- **`polymerase-tm`:** `point_mutation` / `batch` simply iterate and design an
  independent primer pair per mutation. `merge_mutations` is a documented
  parameter that is **never used** anywhere in the code. There is no combined/
  multi-site design. `[PTM-CODE]`

---

## 13. Circular / origin-spanning behaviour

- **NEBaseChanger:** circular by default; a "Partial" checkbox tells it not to
  assume a complete circular sequence. Legacy mode emulates v1. For circular
  substrates the mutation is placed after a fixed pad in a rotated representation
  so that the reverse primer wraps around the origin. `[NEB-DOC]`
  `[NEBC-JS-v2.5.0]`
- **`polymerase-tm`:** `circular=True` by default. The inline builder doubles the
  template (`circ = mutated + mutated`) and places the mutation in the second
  copy, so forward primers can wrap. The reverse primer is built directly on the
  doubled template. This is a plausible strategy but it is **not** the same as
  NEB's rotated-template construction and it is not validated against NEBC.
  `[PTM-CODE]` `[INFERENCE]`
- A dedicated Luria representation already exists (`D009`): ordered intervals for
  origin-spanning features. SDM should reuse that rather than double the
  sequence.

---

## 14. Minimum-length / minimum-Tm behaviour

- Defaults: min length 15 nt, min Tm 55 °C (both user-adjustable in current
  NEBC). Legacy mode min length 18 nt (and NEB FAQ/history mention 15–18 nt).
  `[NEB-DOC]` `[NEBC-JS-v2.5.0]`
- Upper Tm clamp in the builder: 72 °C; GC-extension threshold: 63 °C; Tm
  balance tolerance: 5 °C; Ta cap: 72 °C. `[NEBC-JS-v2.5.0]`
- `polymerase-tm` exposes the same defaults but its growth loop semantics differ
  (first GC-clamp-terminated length above min Tm; falls back to the longest
  60-nt window if min Tm is unreachable). `[PTM-CODE]`

---

## 15. 5' non-annealing tail behaviour

- **NEBaseChanger:** for ≥ 7-nt replacements or when confined, the replacement is
  a 5' tail; long inserts are split half-and-half between forward and reverse.
  For inline substitutions, the mutation is inside the annealing region (10 bases
  of WT on each side of a codon). `[NEB-DOC]` `[NEBC-JS-v2.5.0]`
- **`polymerase-tm`:** triggers the tail path when `confine_to_tails` or
  `mut_len > 6`; splits at `half = mut_len // 2`, puts `mut_bases[:half]` (forward)
  and `revcomp(mut_bases[half:])` (reverse) as lowercase 5' tails. Direction of
  the split matches NEB's description (first half forward, second half reverse),
  within an off-by-one for odd lengths. Tm is computed on the annealing region
  only, which is reasonable for tails. `[PTM-CODE]`

---

## 16. Version differences

- NEBaseChanger **v1** was retired **2025-10-01** and is no longer available.
  `[NEB-DOC]`
- We recovered **v2.5.0** (bundle `index.fc8d957d.js`, archived 2024-06-11) and a
  **v2.4.x** bundle (`index.5fcbb0b3.js`, archived 2023-11-21). Their public
  changelog lists v2.0.0 (2023-05-17) through v2.5.0 (2024-04-16), including:
  v2.3.1 raised minimum primer length to 15; v2.4.0 changed `X` to `NNK`; v2.4.2
  added preserve-degenerate; v2.5.0 added adjustable minimum Tm/length.
  `[NEBC-JS-v2.5.0]` `[NEB-DOC]`
- `polymerase-tm` targets **v2.7.2**, bundle `index.dfcf8292.js`, which is **not
  archived** at Arquivo.pt (only v2.4.x and v2.5.0 are). We could not retrieve
  v2.7.2 or the current live bundle (Cloudflare 403). `[UNKNOWN]`
- The captured D404N output is reproduced exactly by the **v2.5.0** algorithm, so
  the core SDM logic was stable across the versions we can see. Whether v2.7.2
  changed the Tm/salt model (e.g. to bivariate 2008) is **unverified**.
  `[INFERENCE]` `[UNKNOWN]`
- The current live tool is at least v2.5.0 and has continued to evolve; the
  built-in changelog is the only version list we could read. Do not assume the
  live site behaves identically to v2.5.0 or v2.7.2. `[UNKNOWN]`

---

## 17. Direct NEBaseChanger observations

Attempts to exercise the live site from this environment:

- `GET https://nebasechanger.neb.com/` → **HTTP 403** (Cloudflare challenge
  page, 108 945 bytes), with browser User-Agent, via `curl` and via the Pi
  fetch tools.
- `GET https://nebasechangerv2.neb.com/`, `https://nebasechangerv1.neb.com/`,
  `https://www.neb.com/...` → **HTTP 403**.
- Wayback Machine (`web.archive.org`) → blocked / temporarily offline; CDX API
  returned an "Internet Archive: Temporarily Offline" page.
- Proxy mirrors (allorigins, corsproxy, codetabs) → 520/403/522.
- **Arquivo.pt succeeded**: it returned (a) the NEBaseChanger HTML shell and (b)
  the JavaScript bundles, which is what made this investigation possible.

Consequently, **the only direct tool output we have is the captured SID-1 D404N
result** already committed under `tests/fixtures/example-construct/`. The
archived JS is the primary independent source, and it is sufficient to
reconstruct and validate that output. We did **not** run interactive controlled
experiments on the live site (GC-rich/AT-rich, boundary, option toggles, etc.).
Those remain to be done by a human with browser access before Milestone 3 is
finalised. `[UNKNOWN]`

---

## 18. Comparison results

Only one controlled case could be compared against real NEBC output. The
following table uses the actual `polymerase-tm` code (run with a stub for the
optional `primer3` risk check) on the same input.

| Field | NEBaseChanger (captured) | Reconstructed NEBC algorithm | `polymerase-tm` | Classification |
| --- | --- | --- | --- | --- |
| Selected codon | `AAC` | `AAC` (usage) | `AAC` (usage) | exact match |
| Parsimony codon | n/a | `AAT` (1 change) | `AAT` (1 change) | exact match (no tie) |
| FWD sequence | `CGTTGAGCAAaacTTCGGGGCGC` | same | same | exact match |
| FWD length | 23 | 23 | 23 | exact match |
| FWD Tm | 74 | 74.18 → 74 | 62 (right-13-mer) | clearly different (wrong region) |
| REV sequence | `AGCGGATTGAGCAGTTTTCCGTC` | same | `AGCGGATTGAGCAGTTTTC` | clearly different (4 nt short) |
| REV length | 23 | 23 | 19 | clearly different |
| REV Tm | 70 | 69.94 → 70 | 64 | clearly different |
| Ta | 71 | 71 | 63 | clearly different |
| DMSO | n/a | `−0.6 °C/%` | `−0.6 °C/%` | numerically equivalent |

Root causes of the differences:

1. **Forward Tm computed on a sub-region** in `polymerase-tm`; NEBC uses the full
   primer. `[PTM-CODE]` vs `[NEBC-JS-v2.5.0]`
2. **No Tm balancing** in the `polymerase-tm` point-mutation path; NEBC extends
   the lower-Tm primer to within 5 °C. `[PTM-CODE]` vs `[NEBC-JS-v2.5.0]`
3. **Different Tm conditions** (50 mM Na⁺ + 2 mM Mg²⁺ bivariate vs 150 mM
   equivalent monovalent) and different initiation constants. `[PTM-CODE]` vs
   `[NEBC-JS-v2.5.0]`

The forward-sequence match is explained by both tools fixing 10 annealing bases
on each side of a 3-nt codon; it is **not** evidence that the algorithm matches.

---

## 19. Known discrepancies / limitations

### `polymerase-tm` (all `[PTM-CODE]`)

- No output-level test against NEBaseChanger or any captured NEB result.
- `merge_mutations` is dead.
- `_balance_primers` is only used by `deletion`.
- Forward Tm region bug / misleading comment.
- `owczarzy_bivariate` regimes 2 and 3 identical; coefficient sets inconsistent
  between `core.py` and `constants.py`.
- `preserve_degenerate` does not expand ambiguity and breaks Tm growth.
- Point-mutation `warnings` for a mismatched original residue are overwritten
  before being returned.
- SDM Tm conditions differ from the recovered NEBC conditions.
- `test_ta_calculation` can fail because Ta uses raw floats while primer Tm is
  `ceil`ed.
- Insertion `position` off-by-one vs its own docstring.
- Extra out-of-scope features (virtual gels, DMSO reports, Gibson, restriction
  scans) increase surface area and reduce auditability of the SDM core.

### NEBaseChanger reconstruction

- We have only the one captured output for end-to-end validation.
- Mismatch-aware vs no-mismatch Tm path in `calcSDMPrimers` is not fully
  disambiguated; the captured D404N Tm is reproduced by the **no-mismatch**
  method-4 value, even though the code passes a wild-type complement. This should
  be treated as an open item and tested with more outputs.
- v2.7.2 and the live site were not retrievable; the reconstruction is v2.5.0.
- Behaviour for `Partial`/linear sequences, degenerate expansion, batch mode, and
  multi-site NEBuilder mode was not exercised.

### Other public projects

- `mcgawc/base-changer` (GitHub, 2017): a 16 KB Python-2 script with hard-coded
  primers and a `gencode_preferred` table that disagrees with NEB's own table on
  Arg (`cgt` vs NEBC `CGC`) and His (`cac` vs NEBC `CAT`). Not a usable oracle.
- No other public NEBaseChanger implementation was found (GitHub repo search for
  `nebasechanger` returned only `mcgawc/base-changer`; searching `q5 site
  directed mutagenesis primer` returned 0 repositories).

---

## 20. Confidence level for each major finding

| Finding | Confidence | Basis |
| --- | --- | --- |
| NEBC E. coli K-12 codon table and max-usage selection | High | JS table + NEB docs |
| NEBC parsimony = Hamming distance; ties undefined | High | JS + NEB docs |
| NEBC Ta = min(Tm)+1, cap 72 | High | JS + captured output |
| NEBC SDM Tm = SantaLucia 1998 + Owczarzy salt at 150 mM / 500 nM | High | JS + reproduced 74/70 |
| NEBC inline substitution: 10-base flanks, growth + GC clamp + balancing | High | JS + reproduced D404N |
| NEBC indel/tail/insert-splitting rules | Medium-High | JS + NEB FAQ |
| `polymerase-tm` max-usage codons match NEBC | High | enumerated both tables |
| `polymerase-tm` does not reproduce NEBC primers/Tm/Ta | High | executed both |
| `polymerase-tm` SDM Tm model claim (bivariate 2008) | Low/contradicted for v2.5.0 | JS uses monovalent 150 mM path |
| NEBC v2.7.2 ≡ v2.5.0 behaviour | Low | v2.7.2 not retrieved |
| NEBC ambiguous-base expansion semantics | Medium | NEB docs + JS; not executed |
| `polymerase-tm` as an independent oracle | Low (rejected) | no NEBC comparison tests; diverges |
| Live-site controlled observations | None | Cloudflare blocked |

---

## 21. Implications for Luria Milestone 3

1. Luria must implement its own SDM/Q5 workflow. `polymerase-tm` is not an
   oracle and must not be a dependency, a port source, or a runtime component
   (`D002`, `D012`, `D014`, `D015`). `[INFERENCE]`
2. The **documented + JS-verified** rules in Sections 3–9 and 14–15 are strong
   enough to form the basis of a deterministic Luria contract, provided each rule
   is labelled with its source and version. `[NEB-DOC]` `[NEBC-JS-v2.5.0]`
3. Luria's existing coordinate/strand/topology model (`D006`, `D009`) should
   represent the circular substrate explicitly; do **not** copy the
   doubled-sequence trick. `[INFERENCE]`
4. Ambiguous bases must either be expanded with an explicit, bounded algorithm or
   rejected with a structured error; silent partial handling is not acceptable
   (`D005`, `D010`). `[INFERENCE]`
5. Tm, Ta, codon selection, and primer construction should be separate,
   independently testable domain functions (`D003`, `D013`). `[INFERENCE]`
6. Version behaviour must be explicit. Given the live site cannot be treated as
   stable, the contract should state a target behaviour (e.g. "NEBaseChanger v2
   inline substitution semantics as reconstructed from v2.5.0, validated on the
   D404N reference") and record the algorithm version in provenance (`D021`).
   `[INFERENCE]`
7. Human-run live-site observations are still needed for GC/AT-rich, boundary,
   partial/linear, tail-confinement, degenerate, indel, and multi-site cases
   before freezing fixtures. `[UNKNOWN]`

---

## 22. Recommended deterministic Luria contract (proposal, not implemented)

A minimal, auditable SDM slice:

```text
Inputs
  template: SequenceArtifact (circular or linear; explicit)
  mutation: MutationRequest
            - amino-acid substitution: target CDS, residue position,
              expected original residue, replacement residue
            - OR nucleotide replacement: explicit [start,end) + replacement
  policy: CodonSelectionPolicy = {MaxUsage(EcoliK12) | MaxParsimony(
              tie = Fail | Deterministic(first-in-canonical-order))}
  constraints: min_annealing_len=10, min_primer_len=15, min_tm=55,
               max_tm=72, gc_extension_below=63, tm_balance_tolerance=5,
               ta_cap=72, tail_mode={inline|confine}
  thermochemistry: TmMethod = SantaLucia1998 + OwczarzySalt(model=..., salt_mM=150,
                     primer_nM=500, dmso_pct=0)

Outputs
  chosen codon (with policy + tie outcome recorded)
  edited sequence (independently verified, D013)
  forward + reverse primer: full sequence, annealing region, 5' tail,
     length, Tm (method recorded), orientation
  Ta (min(Tm)+1, capped)
  warnings (e.g. original residue mismatch, Tm imbalance, min-Tm unreachable)
  provenance: algorithm id + version, constants, parameter set
```

Determinism requirements:

- Ties must never be random; the contract picks an explicit rule and records
  when a tie occurred (`D012`).
- Primer growth and balancing must be specified as a deterministic state machine
  with a documented termination guarantee.
- Every numeric constant and thermodynamic parameter must name its source.
- Fail closed when the substrate is too short, ambiguous, or unsupported.

---

## 23. Proposed independent fixture cases

Preliminary fixtures live in `tests/fixtures/nebasechanger-research/`
(this is **research evidence**, not the final Q5 fixture set). Recommended matrix
for Milestone 3 (each with an independent expectation source):

| # | Case | Isolates | Independent expectation |
| --- | --- | --- | --- |
| 1 | SID-1 D404N (GAT→AAC, Q5 defaults) | full inline substitution | captured NEBC output (existing) |
| 2 | As #1 with `max parsimony` | codon selection | hand-computed Hamming (`AAT`) |
| 3 | 1-base-change substitution (e.g. GAT→AAT) | parsimony vs usage divergence | hand computation |
| 4 | 2-base-change substitution (as #1) | parsimony vs usage | hand computation |
| 5 | 3-base-change substitution | parsimony vs usage | hand computation |
| 6 | Synonymous re-expression (new AA = old AA) | zero-change handling | contract decision |
| 7 | GC-rich flanking region | GC-clamp / balance | NEBC output needed |
| 8 | AT-rich flanking region | min-Tm growth | NEBC output needed |
| 9 | Mutation near sequence start (circular origin) | circular wrap | NEBC output needed |
| 10 | `Partial`/linear substrate | linear semantics | NEBC output needed |
| 11 | min primer length 15 vs 18 (legacy) | min-length rule | NEB docs + NEBC output |
| 12 | min Tm 55 vs higher/lower | min-Tm rule | NEBC output |
| 13 | confine-to-tails on/off | tail placement | NEB docs + NEBC output |
| 14 | insertion ≤ 6 nt | single-tail insertion | NEB FAQ |
| 15 | insertion > 6 nt | split tails | NEB FAQ + NEBC output |
| 16 | deletion of N nt | deletion geometry | NEBC output |
| 17 | multiple point mutations | per-mutation vs combined | NEB docs |
| 18 | ambiguous replacement (e.g. `NS`) | combinatorial expansion | NEB docs + NEBC output |
| 19 | degenerate code preserved | preserve option | NEBC output |
| 20 | stop-codon introduction (`Q10*`) | stop selection | NEBC output |
| 21 | unsupported residue / `X` | failure semantics | contract |
| 22 | expected original residue mismatch | warning vs error | NEB docs |
| 23 | Tm model regression vectors | Tm/Ta formula | NEBC JS + captured values |

Cases 1–5 can be handled now from documentation and hand computation. Cases 7–22
need human-run live-site captures to be trustworthy; until then they should be
marked `proposed` / `needs-observation`.

---

## Appendix A — Evidence sources

### Primary (authoritative for our reconstruction)

- NEBaseChanger front-end bundle v2.5.0: `index.fc8d957d.js`, archived
  2024-06-11 by Arquivo.pt.
  `https://arquivo.pt/wayback/20240611100829js_/https://nebasechanger.neb.com/assets/index.fc8d957d.js`
  - `t.cut.Ecoli_K12` codon table
  - `Ee` (max usage), `Le` (max parsimony)
  - `t.TmCalc` / `ge.createTmCalc` method-4 thermodynamics
  - `ge.calcSDMPrimers` and `xe` (`buildPrimers`)
- NEBaseChanger bundle v2.4.x: `index.5fcbb0b3.js`, archived 2023-11-21.
- Captured D404N output: `tests/fixtures/example-construct/raw/{sdmsummary.txt,sdmprimer.txt,sdmfinalseq.fa}`.

### NEB documentation

- NEBaseChanger: https://nebasechanger.neb.com/ (and `/help`; Cloudflare-blocked here)
- NEBaseChanger v2 help mirror referenced by search indexes.
- NEBaseChanger tutorial: https://www.neb.com/en-us/tools-and-resources/video-library/nebasechanger-primer-design-tool-tutorial
- Q5 SDM FAQ: https://www.neb.com/en/faqs/how-do-i-design-primers-to-use-with-the-q5-site-directed-mutagenesis-kit
- Q5 Ta FAQ: https://www.neb.com/en/faqs/what-should-i-use-for-an-annealing-temperature-with-the-q5-site-directed-mutagenesis-kit
- Q5 SDM experimental considerations: https://www.neb.com/en/applications/cloning-and-synthetic-biology/site-directed-mutagenesis/site-directed-mutagenesis-experimental-considerations
- Q5 SDM manual E0554 / E0552 (NEB PDFs).
- NEBaseChanger v1 retirement notice: https://nebasechangerv1.neb.com/

### Secondary / other implementations

- `polymerase-tm` v2.3.2: https://github.com/Dioskurides/polymerase-tm
  - `src/polymerase_tm/mutagenesis.py`, `core.py`, `constants.py`, `tests/test_mutagenesis.py`, `release_notes.md`, `__init__.py`.
- `mcgawc/base-changer`: https://github.com/mcgawc/base-changer (2017).

### Thermodynamics references (as cited by NEB/polymerase-tm)

- SantaLucia J Jr. 1998, PNAS 95:1460–65 (NN parameters).
- Allawi & SantaLucia 1997–1999 (mismatch parameters).
- SantaLucia & Hicks 2004, Annu. Rev. Biophys. Biomol. Struct. 33:415–40.
- Bommarito et al. 2000, Nucleic Acids Res. 28:1929–34.
- Owczarzy et al. 2004, Biochemistry 43:3537–54.
- Owczarzy et al. 2008, Biochemistry 47:5336–53.

## Appendix B — Reproducing the NEBC reconstruction

The reconstruction used to validate D404N is:

```text
Tm_method4(seq):
  w = lower(seq); N = len(w); fgc = (G+C)/N
  dS_init = -5.7; dH_init = 200
  if w[0] in AT: dS_init += 6.9; dH_init += 2200
  if w[-1] in AT: dS_init += 6.9; dH_init += 2200
  (dH, dS) += SantaLucia1998 dinucleotide values
  Tm1M = (dH*1000 + dH_init) / (dS + dS_init + 1.987*ln(500e-9))
  sc_ow = monovalent Owczarzy(2004) at [Na+] = 0.150 M
  return 1/(1/Tm1M + sc_ow) - 273.15
```

with `minLen=15`, `minTm=55`, `pad=10`, GC threshold 63, balance tolerance 5,
`Ta=min(Tm)+1` capped 72. This yields the captured D404N primer pair exactly.

> The script and raw JS used for this research live under
> `target/research/` (git-ignored) and are intentionally **not** part of the
> repository. They are working artifacts, not Luria dependencies.
