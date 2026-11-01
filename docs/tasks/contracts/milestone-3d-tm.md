# Milestone 3d task contract: melting temperature, and Tm-driven primer length

Status: Implemented 2026-10-14. Phases 1–2 in §8, Phase 3 (D030) in §9.
Date: 2026-10-14
Planner: Claude. Owner decisions in §2.

## 1. Why this milestone exists

luria emits primer pairs whose geometry satisfies D029 and whose melting
temperature nobody has computed. At a bound of 15 the reverse primer is 15
bases; the reference tool emits 23 and 24 on the two cases we can compare,
because it extends until the primer melts at 55 °C or above. A 15-mer on an
AT-rich site may melt well below the annealing temperature the Q5 protocol
recommends, so luria can currently emit a pair that is contract-valid and
would not work at the bench.

The design contract already names Tm as unavailable and reports `minTm`,
`tmFunction` and `saltModel` as unresolved inputs ([docs/specs/q5-primer-design.md](/specs/q5-primer-design.md)
§10, §11). This milestone supplies the function and lets it determine primer
length.

## 2. Owner decisions, 2026-10-14

1. **Mismatch-aware Tm.** luria reports the Tm of a primer against its
   **wild-type binding site**, not against its own complement. This matches
   the reference tool's choice and describes the first PCR cycle, where the
   forward primer anneals over a mismatched bubble at the edit. It requires
   internal-mismatch nearest-neighbour parameters, not only the ten
   Watson–Crick pairs.
2. **The tandem-mismatch fallback is a hybrid** (decided 2026-10-14 after
   Phase 1 found the gap; see §8 and `tests/fixtures/tm/README.md`). A primer
   gets the mismatch-aware Tm where published parameters cover it, and the
   matched-duplex Tm where they do not, which is any edit changing two
   adjacent bases. Every Tm names the basis it was computed on, and a
   fallback also carries the reason. The two quantities are never averaged,
   compared to each other, or reported under one name.
3. **Tm drives primer length in this milestone.** D029's fixed centred
   geometry becomes a starting point that extends until a Tm rule is
   satisfied. This amends D029 and needs a new decision entry; the amendment
   is part of this milestone, not a follow-up.
4. **Parameters come from the published literature** (planner, from D014 and
   D028, not a new choice): each value names its paper, is cross-checked
   against an independent open implementation, and the captured vendor table
   is a cross-check only. A disagreement between the literature and the
   captured table is recorded, never silently resolved in the vendor's favour.
5. **Computed in Rust** (planner, from D025): deterministic table lookup and
   arithmetic. No Python worker is introduced for it.
6. **Buffer conditions are named, never defaulted** (planner, from
   [docs/specs/q5-primer-design.md](/specs/q5-primer-design.md) §2.2's rule that an absent input is reported
   absent): a named model carries the monovalent salt and primer
   concentration, and the caller names it. The Q5 condition is 150 mM
   monovalent, 0 mM divalent, 500 nM primer, 0 % DMSO, per the research
   notes; the same tool uses 50 mM / 200 nM at a different call site, which is
   why a default is refused.

## 3. Phase 1: parameter sourcing (no Rust)

Deliver `tests/fixtures/tm/` with:

- `nn-matched.tsv` — the ten Watson–Crick nearest-neighbour ΔH/ΔS pairs,
  initiation, the terminal A/T penalty, and the self-complementarity term;
- `nn-mismatch.tsv` — internal-mismatch ΔH/ΔS for every mismatch type in
  every neighbour context;
- `dangling.tsv` — only if the chosen Tm path needs terminal dangling-end or
  terminal-mismatch terms; omit it if not, and say so;
- `README.md` recording, **per value**, the paper it comes from and the
  independent implementation it was checked against, plus every disagreement
  found against the captured vendor table, with the vendor's value and ours
  side by side.

Rules for this phase:

- No value is written from memory. Every number is read from a source.
- Two independent sources must agree before a value is accepted. The vendor
  table is not one of the two.
- A value that cannot be sourced twice is recorded as unsourced and the
  milestone stops rather than guessing.
- No Rust is written in this phase.

## 4. Phase 2: the Tm function (Rust, geometry unchanged)

`crates/core/src/tm.rs`:

- a named buffer model carrying monovalent salt, divalent salt, primer
  concentration and DMSO percent, with the Q5 condition as a named
  constructor rather than a default;
- a named Tm function identifying its model and parameter source (D014: a
  calculated thermodynamic value identifies its method);
- `tm_matched(primer)` and `tm_mismatched(primer, binding_site)`, the second
  being what a design reports per §2 decision 1. The reverse primer carries
  no mismatch, so the two agree on it; the forward primer is where they
  differ;
- the Owczarzy 2004 monovalent salt correction, with the divalent branch
  refused rather than approximated while divalent is 0 in scope;
- `PrimerDesign` reports each primer's Tm with the function and buffer model
  that produced it, and `unresolved` loses the entries this milestone
  supplies.

Geometry does not change in this phase. D029 still fixes the lengths.

**Validation gate.** Phase 2 is not complete until luria's Tm is compared
against the Tm values recorded for all six reference designs in
`tests/fixtures/q5-primer/reference/designs.json`, and the differences are
reported per design. The research notes record one independently executed
comparison for the q5p-01 forward primer: 73.7841 °C mismatch-aware against
74.3490 °C matched. A difference of that order is expected and is not a
failure; an unexplained difference of several degrees is, and stops the
milestone for review.

## 5. Phase 3: Tm-driven length (amends D029)

Blocked until Phase 2's validation gate is reported to the owner, because
the rules below need numbers to choose sensibly. The owner decides, with the
Phase 2 comparison in hand:

- the Tm target, and whether it is a floor, a band, or a preference;
- whether extension is 3′-only, as the reference does, or symmetric, which is
  what D029's centring currently requires — these conflict, and one must give;
- whether a 3′ terminal G or C is required, preferred, or ignored;
- the maximum Tm, if any, and what happens above it;
- the permitted Tm difference between the two primers of a pair;
- the recommended annealing temperature, if luria reports one at all, and its
  source;
- `maxAnnealingOverlap`, which becomes meaningful again and was deferred on
  2026-10-14 only because nothing could use it.

Each answer needs a documented source under D014. The reference tool's rules
are evidence of what one tool does, not a source for a luria rule.

## 6. Out of scope

Hairpins, self-dimers, hetero-dimers, GC-content rules, insertions,
deletions, degenerate replacement expansion, the `nebBaseChangerCompatible`
policy, and any Python worker.

## 7. Required checks

The five in `AGENTS.md`, plus: no fixture value generated by running luria,
and every Tm fixture value traceable to its two sources in the README.

---

## 8. Phases 1 and 2 as built, 2026-10-14

**Phase 1.** `tests/fixtures/tm/` holds `nn-matched.tsv` (13 values) and
`nn-mismatch.tsv` (48 motifs, the complete single-mismatch set), each value
sourced and verified as that directory's README records. `dangling.tsv` was
not needed and is absent: the chosen path has no dangling-end or
terminal-mismatch term.

Phase 1 found the **tandem-mismatch gap** — no published parameters cover a
doublet with two mismatches, which is 74 % of residue-changing codon
substitutions — and the owner resolved it as the hybrid in §2 decision 2.

**Phase 2.** `crates/core/src/tm.rs`:

- `BufferModel`, with `q5_reaction()` naming the 150 mM / 500 nM / 0 % DMSO
  condition, and `create` refusing a concentration whose logarithm the
  equation cannot take;
- `TmBasis` (`MatchedComplement`, `MismatchedWildTypeSite`),
  `MeltingTemperature` carrying the value, basis, buffer, ΔH°, ΔS° and any
  substitution reason, plus the function name and version;
- `melting_temperature(primer, site, buffer)`, and
  `melting_temperature_or_matched` implementing the hybrid fallback;
- two new error variants: `InvalidBufferModel` (`MalformedInput`) and
  `TmParametersUnavailable` (`Unsupported`).

Tests: 15 unit tests in `tm.rs`; `tests/tm_tables.rs` (4) checks the embedded
tables still equal the fixture and reruns the fixture's ΔG°37 verification;
`tests/tm_crosscheck.rs` (2) pins exact agreement with Biopython 1.88;
`tests/q5_primer_fixtures.rs` gains the matched-basis comparison against all
twelve reference primers. Workspace total 284 tests, 0 failures, all five
required checks clean.

**The validation gate is passed, with one documented disagreement.** On the
matched basis luria is within 0.17 °C of every recorded reference value. On
the mismatch-aware basis luria is 6–7 °C below its own matched figure, where
the reference tool's own mismatch-aware figure is 0.57 °C below its matched
one. Biopython reproduces luria's number exactly, so luria computes what the
published papers specify and the reference tool does something else. Recorded,
not resolved; see `tests/fixtures/tm/README.md`.

**A bug the cross-check caught.** The first draft inverted the concentration
convention, applying C_T/4 to self-complementary duplexes instead of to
non-self-complementary ones, and ran 3.4 °C low on self-complementary
sequences. The twelve reference primers would never have caught it — none is
self-complementary.

### Measurements for the Phase 3 decisions

What D029's primers melt at today, at the Q5 buffer condition
(`what_the_native_designs_melt_at`):

| case | bound | fwd len | fwd matched | fwd aware | rev len | rev Tm |
| --- | --- | --- | --- | --- | --- | --- |
| native-01 | 15 | 23 | 74.35 | 67.72 | 15 | **60.03** |
| native-02 | 24 | 27 | 76.62 | 71.84 | 24 | 69.55 |
| native-03 | 25 | 28 | 77.39 | 73.02 | 25 | 68.89 |
| native-04 | 15 | 23 | 63.36 | tandem | 15 | **45.40** |

Two things follow, and both are evidence for Phase 3 rather than decisions:

- **A 15-base reverse primer can be far too cold.** `native-04`'s melts at
  45.40 °C, about 10 °C below the 55 °C the reference tool documents as its
  minimum. This is the concrete form of the concern that opened this
  milestone.
- **A pair-matching rule would reproduce the reference's lengths.** For
  `native-01` luria's pair is 67.72 / 60.03, a 7.7 °C spread; the reference's
  pair for the same edit is 74.18 / 69.94, a 4.2 °C spread with a 23-base
  reverse primer. The reference's reverse primer is long because of its
  documented "within 5 °C of the forward primer" rule, not because of its
  minimum length.

---

## 9. Phase 3 as built, 2026-10-14: D030

**Owner decisions** (recorded in full as D030 in [docs/decisions/](/decisions/)):

1. 55 °C floor, as the practical minimum Tm for a PCR primer.
2. Abutment stays the pair rule; Tm balance is a separate sizing rule on top
   of it, sourced to NEB's PCR guidelines ("within 5 °C of each other").
3. Growth at 3′ ends only, with the forward primer's 5′ flank fixed at 10, so
   the junction never moves — as NEBaseChanger does. Chosen over centred
   growth after NEB's Q5 SDM FAQ ("a mismatch in the center of the mutagenic
   primer") was put to the owner; recorded in D030 as a deviation.
4. The Tm used for sizing is the hybrid basis of §2 decision 2, and the
   annealing temperature uses the same Tms.
5. G/C last base with a 63 °C ceiling, as general PCR practice; annealing
   temperature as the Q5 documentation's "Tm+3", not NEBaseChanger's +1.

**Code.** `crates/core/src/primer_design.rs` rewritten for D030: a `Sizer`
over the fixed junction, the floor / G/C / balance loops, the buffer as a
required request field, and `PrimerDesign` gaining `forward_tm`, `reverse_tm`,
`annealing_temperature` and `tm_balance_met`. Algorithm version 0.2.0.
`NoCenteredPrimerDesign` is renamed `NoPrimerDesignFits`, and
`DesignObstacle` gains `FivePrimeTmFloorUnreachable` and
`ThreePrimeTmFloorUnreachable`. `q5_design.rs` passes the buffer through.

**Expectations.** `tests/fixtures/q5-primer/reference/native-design.py` was
rewritten to restate D030 in plain Python with Biopython's Tm; it regenerates
`native.tsv` byte for byte and, with `--synthetic`, scans four synthetic
templates for cases that exercise every branch. The unit tests use those
cases: every branch (forward and reverse floor growth, G/C extension and
revert, balance in both directions, balance ending on a G/C base, unmet
balance, the tandem fallback, both Tm-floor failures, the 72 °C cap) has at
least one test.

**Results against the reference tool.** For `native-01` and `native-04` the
forward primer still equals the reference's. For `native-04` the reverse
primer now matches as well, 24 bases. For `native-01` luria's reverse primer
is 19 bases against the reference's 23, because luria balances against the
published-model mismatch-aware Tm (67.72 °C), which is several degrees below
the reference tool's own.

**Checks.** `cargo fmt --all -- --check`, `cargo test --workspace` (292
tests, 0 failures), `cargo clippy --workspace --all-targets -- -D warnings`
and `git diff --check` all pass. Changing `TM_BALANCE` to 4, `TA_OFFSET` to 1
or `GC_CEILING` to 60 each made tests fail (2, 9 and 4 failures), and the
restored file passes.

**Not done, and not in this milestone's scope:** hairpin, dimer and
GC-content rules; `maxAnnealingOverlap` (still absent — D030 adds no upper
length bound, so it would still change nothing); a reported Ta that matches
NEB's calculator, which this milestone does not claim.

## 10. Next, as ordered by the owner, 2026-10-14

1. **Variant sets.** Per-member design results, so an undesignable member no
   longer discards the others; refuse a set whose members share a result
   identifier ([docs/specs/q5-sdm.md](/specs/q5-sdm.md) §10 item 6, §12).
2. **Old workflow decisions.** Settled 2026-10-14, all as implemented
   ([docs/specs/q5-sdm.md](/specs/q5-sdm.md) §10). Only a stale test name remains.
3. **Primer quality checks.** Warnings on an emitted design, not rejections.
   Sequence-only checks first — GC content, runs of one base, 3′ end
   composition — each threshold citing a published guideline. Hairpin and
   dimer checks follow later, after loop, bulge and dimer parameters are
   sourced and verified as the Tm tables were.
4. **Insertions and deletions.** A milestone of its own:
   [docs/tasks/contracts/milestone-4-indels.md](/tasks/contracts/milestone-4-indels.md), written 2026-10-14 and **blocked on six
   owner decisions** in its §4 — how an insertion point is represented,
   whether `verify` keeps a length rule, the pair invariant for a deletion,
   which primer carries an insertion, whether a 5′ tail counts toward Tm, and
   whether deletion comes before insertion. Nothing is implemented: each
   decision changes a Milestone 1 or 2 rule, and the reference tool's indel
   path was never substantiated, so there is no external design to check
   against.
