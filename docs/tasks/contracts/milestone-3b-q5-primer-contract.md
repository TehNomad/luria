# Milestone 3b — Specify the Q5 primer-design contract

**Task branch:** `feat/milestone-3b` (work continues on `feat/milestone-3`)
**Status:** specification and reference fixtures delivered; a representation,
decomposition and validation layer (`crates/core/src/primer.rs`) implemented;
no primer-design algorithm, and no Tm function, implemented
**Depends on:** Milestone 3 ([docs/tasks/contracts/milestone-3-q5-workflow.md](/tasks/contracts/milestone-3-q5-workflow.md)),
Milestone 3a (separate-variant composition)

---

## 1. Scope

**In scope**

- the contract for a future deterministic Q5 primer-design layer: its typed
  inputs, its typed outputs, the rules that are hard validity requirements,
  the rules that are preferences, and the rules that remain unknown;
- what a design must report about strand, coordinates, and overlap;
- a fixture set that makes those claims checkable before an algorithm exists;
- an honest record of what the saved NEBaseChanger material does and does not
  establish.

**Out of scope**

- implementing a primer-design algorithm in `crates/core`;
- choosing the project's Tm function;
- defining a Tm range, a `Ta`, or a salt model;
- an LLM adapter, a Python worker, a database, a TUI, a web interface;
- changing any biological assumption already recorded.

**Why this milestone exists before the algorithm.** A primer pair is a
biological product. Without a contract, an algorithm would be chosen against
unstated rules, and its behaviour could not be distinguished from its
author's choices. The contract fixes what must be true of any design, so the
algorithm can be chosen later on its own merits and tested against invariants
that do not depend on it.

---

## 2. What was established before editing

The actual artifacts were read, not inferred from names.

| Artifact | What it establishes |
| --- | --- |
| [docs/specs/q5-sdm.md](/specs/q5-sdm.md) | The input-scope gate, single-request workflow, separate-variant composition, one-time target resolution, atomic failure. §10 records six unsettled items. |
| `crates/core/src/mutation.rs` | `apply_nucleotide` and `apply_residue` are the only mutation operations. A nucleotide substitution selects exactly one nucleotide. There is **no insert and no delete operation**. |
| `crates/core/src/q5.rs` | `Q5MutationIntent` has three variants: `Nucleotide`, `Residue`, `ResidueWithoutCds`. The module header states it "does not add a mutation kind". |
| `crates/core/src/strand.rs` | `Strand` is an explicit two-value enum with a symbol; `is_plus`. |
| `crates/core/src/sequence.rs` | `canonical_text`, `normalize`, `content_hash` over the canonical text; identity is name plus digest. |
| `crates/core/src/interval.rs`, `identity.rs` | Zero-based half-open internally; `SequenceReference` pairs a name and a digest. |
| `tests/fixtures/example-construct/` | SID-1 cDNA, 3153 nt; edit at 1-based 1210–1212, `GAT → AAC`; `raw/sdmsummary.txt` records a primer pair of length 23 and 23 with Tm 74 and 70 and `Ta` 71; `input/mutation_spec.json` records `minimum_overlap_bp: 15`. |
| `tests/fixtures/mutation/` | Synthetic sequences `mut-a`…`mut-e`; `cases.tsv` has 44 cases including three on a reverse-strand CDS, one of which (`res-a-minus-7`) edits `mut-a` at reference coordinates 9–12. |
| `tests/fixtures/nebasechanger-research/` and `target/research/` | The saved HAR, the interface text, and the readable JavaScript: `calcSDMPrimers`, its callsite `buildPrimers`, `calculatePrimers`, `t.Primer`, and the Tm routine. |

Two findings changed the shape of the task and are reported rather than
absorbed:

1. **Insertions and deletions are not representable in the mutation layer.**
   The task asks for them as inputs to the contract; the layer cannot produce
   them. The fixture set carries those cases as contract-level cases, marked,
   and states that their edit is not a `MutationResult`.
2. **The reference tool's insert and delete primer paths are a different shape
   from its replace path** — they design a whole fragment pair, not a local
   pair. So no reference design is offered for an insertion or a deletion.

---

## 3. The contract

Written to [docs/specs/q5-primer-design.md](/specs/q5-primer-design.md). Its shape:

- **§2 Inputs.** A validated `MutationResult` is the only biological input; the
  design layer must not re-derive coordinates from a bare sequence. Every
  design-request field is carried into the output, including the absent ones.
- **§3 Outputs.** A `PrimerDesign` carries the pair, each primer's text in
  5′→3′ form with an explicit orientation, its span against the template, its
  overlap count, the positions that changed, the Tm under a named function, and
  the constraint set it was evaluated against.
- **§4 Reference sequence.** `sequenceBefore` — the pristine template — is the
  reference for every span. Primer *text* comes from the edited sequence. The
  contract says which, because the two sequences differ in length for an
  insertion or a deletion.
- **§5 Edit geometry.** Substitution, insertion, and deletion are each stated.
  Overlap is counted **position by position** against the template, not as
  "span width minus changed width"; the SID-1 case separates the two.
- **§6 Strand and 5′→3′.** Both primers are reported 5′→3′ with an explicit
  orientation. The reverse primer's text is the reverse complement of its span
  in the template, stated as a checkable invariant. Case carries no
  orientation.
- **§7 Hard requirements H1–H8.** Checked on the output, not assumed of the
  algorithm.
- **§8 Preferences P1–P4.** Never override hard requirements. The search is
  deterministic and the tie-break order is part of the contract.
- **§9 `minOverlap` is a lower bound on every primer**, not a target. The
  fixture set contains a case at the bound and cases above it.
- **§10 Failure and unresolved inputs.** A failed design emits nothing (D010).
  A constraint the caller did not supply is reported as unresolved, with the
  effect of leaving it unset.
- **§11 What the contract cannot know.** The project's Tm function, its salt
  assumption, an acceptable Tm range, a `Ta`, and a minimum `minOverlap` are
  all recorded as **not established**.
- **§13 Reference behaviour, marked as observation.** Six numbered observations
  with their source in the saved material, and §13.3 states why reproducing the
  reference Tm routine is not evidence about the project's Tm.
- **§14 How behaviour is testable before an algorithm is chosen.** Thirteen
  implementation-independent invariants, and the coverage the fixture set must
  reach.
- **§15 Open questions** for the next milestone, with the three that the
  current material cannot settle marked as such.

---

## 4. Fixtures

Written to `tests/fixtures/q5-primer/`: `cases.tsv` (10 cases, 14 columns),
`reference/designs.json`, `reference/neb-q5-sdm-reference.py`, `README.md`.
See that README for the column meanings and the reason each case is in the set.

Every case is derived from an artifact another fixture already validated: the
SID-1 construct from `example-construct/`, and `mut-a.txt` from
`mutation/`. No sequence is duplicated. `q5p-08` shares `mut-a.txt` with the
mutation fixture's `res-a-minus-7` case, but not its edit: that case acts on
residue 7 (`ATT → L`), which under the `Default` policy writes DNA
`AAT → CAG` at 9–12 and the codon `ATT → CTG` at reference coordinates
9,10,11 (before the 2026-10-14 review, `AAT → TAA` and `TTA`), while `q5p-08`
declares DNA `AAT → AAC` at 9–12.

The reference-design cases are reproducible: running
`reference/neb-q5-sdm-reference.py` regenerates `designs.json` byte-for-byte,
and the SID-1
D404N pair it produces matches `raw/sdmsummary.txt` in the existing fixture.
That is the check that the reference values were not transcribed wrongly.

**No fixture asserts expected primer text for `luria`.** D008 states that
NEBaseChanger behaviour is not the source of truth for luria's algorithms, so a
fixture that asserted a design before an algorithm exists would be asserting a
property of a future choice.

**No reference design is claimed to satisfy H1–H8 either.** The reference tool's
inline `replace` path applies its minimum-length parameter to one primer, so the
`q5p-02` and `q5p-03` reference designs leave the forward primer's **annealing
overlap** at 20, below the supplied bounds of 24 and 25, while their reverse
primer reaches 25. H1 is evaluated on that annealing overlap — not on primer
length, and not on the whole-primer match count, which is 21 and 25 for those
two designs. The two quantities are recorded under different names
(`annealingOverlap`, `wholePrimerMatchCount`) so that neither can be read as the
other. Every outcome is recorded in each record's `h1` block and in `cases.tsv`'
`reference_check` column. The generator asserts the derived fields against the
primer text and the template for every design it writes, so a self-contradicting
record fails the run.

---

## 5. Checks run

| Command | Result |
| --- | --- |
| `python3 tests/fixtures/q5-primer/reference/neb-q5-sdm-reference.py` | ran; asserted the derived decomposition for every design; wrote `reference/designs.json`; printed the H1 outcome of each reference design with its measured annealing quantity |
| second run, `sha256sum` before/after | `d0f54eb7…` both times — regeneration is byte-for-byte identical |
| reproduction check against `raw/sdmsummary.txt` | forward `CGTTGAGCAAAACTTCGGGGCGC` len 23 Tm 74.18; reverse `AGCGGATTGAGCAGTTTTCCGTC` len 23 Tm 69.94; `Ta` 71 — matches the fixture's 23/23, 74/70, 71 |
| H1 outcome from `designs.json`, evaluated on `annealingOverlap` | `q5p-01` satisfied (fwd 20 above 15, rev 23 above 15); `q5p-02` VIOLATED (fwd 20 below 24, rev 25 above 24); `q5p-03` VIOLATED (fwd 20 below 25, rev 25 **at** 25); `q5p-04` satisfied (fwd 20 above 15, rev 23 above 15); `q5p-07` variant-a satisfied (20, 23 above 15), variant-b satisfied (20, 24 above 15) |
| digest check | `sid-1` → `54704740a9f04b79`, `mut-a.txt` → `a83f6d82…`, the value `mutation/cases.tsv` already records for that artifact |
| `awk -F'\t' '{print NF}' cases.tsv` | 14 fields on all 11 lines |
| cross-check of `cases.tsv` `reference_check` against `designs.json` | every recorded outcome and quantity matched |
| `cargo fmt --all -- --check` | not run — no Rust changed in this task |
| `cargo test --workspace` | re-run after the fixture changes: **190 passed, 0 failed** |
| `git diff --check` | run; clean |

---

## 6. Deliberately left open

Recorded in [docs/specs/q5-primer-design.md](/specs/q5-primer-design.md) §11 and §15. They are listed so that a
later milestone does not answer one by accident:

1. Which Tm function `luria` owns, and which salt assumption it states.
2. Whether Tm is a hard constraint or a preference when `tmTarget` is absent.
3. Whether a Q5 pair must abut, or only cover the edit. The reference pair
   abuts; one case does not establish a rule.
4. Whether a maximum primer length exists, and whether it is a bound or a
   preference.
5. How a composed separate-variant request is represented at the design layer.
6. What the layer does when two variants' primer spans overlap on the template.
7. Whether the layer emits a `Ta` at all.

---

## 8. The implementation layer delivered against this contract

A second pass implemented the part of the contract that is fully specified. It
is recorded here because it changes what this milestone delivered.

**Added:** `crates/core/src/primer.rs` — `PrimerRole`, `PrimerOrientation`,
`PrimerPlacement`, `PrimerPartRole`, `PrimerPart`, `Primer`, `PrimerPair`, and
the functions `construct_primer`, `assemble_pair`, `require_design_constraints`.

**In scope:** the representation (§3, as `Primer`/`PrimerPart`/`PrimerPair`),
the decomposition (H8, I-13), the overlap quantity (§5, I-5/I-13), the
reference-sequence rule (§4, I-2/I-6), the strand and orientation rule (§6, H4/I-7),
the pair-relationship rule (H5, I-8 — derived from the two spans, not stored), the
text-versus-coordinates checks (H3, `PrimerTextAtSpanMismatch` and
`PrimerAnnealingNotTemplate`), the coverage rule (H2, I-4) and H1 as a constraint
check (I-3).

**Out of scope, and left unimplemented:** candidate generation, ranking, Tm,
every preference, and the contract's §3 output type `PrimerDesign` itself. The
layer delivers `PrimerPair` and the construction functions; nothing yet carries
the design-request fields into an output record, so §2.2 and §10 reporting is
not implemented either. The contract's §11 and §15 say these are not specified;
the reference material describes them but is not the project's rule.

**Two contract gaps were found while implementing, and both were closed by
naming the missing quantity in the record rather than by choosing a design rule:**

1. H1 as written ("every primer's annealing overlap meets the bound") is only
   evaluable once a record names which positions are annealing. It does: every
   `PrimerPart` carries an `annealing` flag, and every reference record carries
   an `annealing` block with its sequence, its template spans and its length. H1
   is therefore evaluated for all six reference designs, and the two that fail
   it are recorded as failing rather than dropped or repaired.
2. The decomposition needs the boundary of the annealing portion. It is stated
   as the runs of unchanged positions within the primer, derived from the parts,
   so a design that cannot name its parts cannot name its annealing portion
   either — and `construct_primer` rejects that partition as
   `DomainError::PrimerPartition`.

**Checks run for the layer** are in the handoff for this pass: `cargo fmt --all
-- --check`, `cargo build`, `cargo test --workspace`, `git diff --check`.

**Still open:** the annealing portion named here is a description of what the
reference construction produced, not a project rule. A `luria` design will have
to name its own boundary, and the rule that chooses it is still unsettled.

**Amended (2026-10-13, split-policy pass).** A rule for that boundary is now
*proposed*, not settled: D029 states a mutation-centered split —
`u = ceil(a_f / 2)`, `v = floor(a_f / 2)`, extra base 5′ — under which the
forward primer's annealing portion is the two unchanged flanks and the reverse
primer's is its whole span. D029 is **proposed and unapproved**, so the rule is
not a contract the implementation pass above worked to; nothing in that pass
changed, and no Rust was written by this amendment. The split that the 2026-10-12
review pass wrote into §18.1 is superseded; see §22 of
[docs/specs/q5-primer-design.md](/specs/q5-primer-design.md) and
[docs/research/q5-primer-implementation-readiness-2026-10-13.md](/research/q5-primer-implementation-readiness-2026-10-13.md) for what the
centered rule does and does not settle.

Questions 3, 6 and 7 are the ones the current material cannot settle. They
should be settled by a source, or by the caller declining to need them.

---

## 9. Merge of `main` after the 2026-10-14 core review

`main` gained `fix/core-review` (PR #4): the IUPAC alphabet fix, all-pairs
interval overlap, topology in `SequenceReference`, A/C/G/T-only nucleotide
substitution, and a `Default` codon policy that picks the most used
*E. coli* K-12 codon. Merging it into `feat/milestone-3` needed these changes
to Milestone 3 code; none changes a Milestone 3 rule:

- `error.rs`: the `UnsupportedMutationPolicy` message no longer names a CDS,
  because the payload no longer carries one.
- `q5.rs`: the three `MissingCdsAttributeProblem` values (no target CDS, no
  resolved target) carry `value: None`, meaning the attribute was absent.
- `primer.rs`: `oriented` propagates `reverse_complement`'s `Result`. The
  letters come from validated artifacts, so the error is unreachable in
  practice, but it is no longer a panic path.
- `q5.rs` tests: the composition tests supply their codons (`pinned_policy`:
  G `GGT`, A `GCT`, W `TGG`) instead of relying on `Default`, which now gives
  `GGC` for G and `GCG` for A. They test composition, not codon choice, so the
  hand-derived edits are kept rather than re-derived.
- `primer.rs`: the 16 clippy findings recorded in
  [docs/research/q5-primer-implementation-readiness-2026-10-13.md](/research/q5-primer-implementation-readiness-2026-10-13.md) Part 7 and
  Part 8 item 7 are fixed (`is_empty`, string-literal comparisons, `!` for
  `== false`). `cargo clippy --workspace --all-targets -- -D warnings` is a
  required check (D026), and it now passes.

§2's table records what was true before this merge: identity is now name,
digest, and topology, and `tests/fixtures/mutation/cases.tsv` has 47 cases.
The workspace suite runs 219 tests (191 unit, 8 + 6 + 14 integration), 0
failures.

---

## 10. Owner answers and test coverage, 2026-10-14

Answers to the blockers in
[docs/research/q5-primer-implementation-readiness-2026-10-13.md](/research/q5-primer-implementation-readiness-2026-10-13.md) Part 4:

- **P0-3, D029:** **approved** on 2026-10-14, with four amendments applied to
  D029 and [docs/specs/q5-primer-design.md](/specs/q5-primer-design.md) §18 (§23 lists them): at least 10
  unchanged bases on each side of the replacement, the extra base of an odd
  split on the 3′ side, a scope of any contiguous replacement region of at most
  3 bases, and a reverse primer of at least 15 bases.
- **P0-2, error carrier for "no design":** split by cause. When D029 is
  implemented, a single new variant reports a no-design outcome and names the
  binding side. Its category depends on the cause: `Unsupported` when the
  centered geometry does not fit but another geometry might (§18.5 outcome
  1, e.g. an edit too close to a template end); `BiologicalValidation` when no
  primer of any geometry could satisfy the request (e.g. `q5p-09`, a bound of
  31 on a 30-base template). Not implemented: there is no caller for it until
  the design layer exists (D025).
- **P2-2, `maxAnnealingOverlap`:** left out of the request for now. It cannot
  change any outcome under the centered split, so it is not added until a
  Tm-based length policy exists; nobody can set a field that silently does
  nothing.

Tests added in this pass:

- **P1-3:** six `primer.rs` tests over a single-nucleotide substitution
  (`L = 1`, coordinate 15 of `seq-a`, C -> T): the three-part decomposition,
  equal annealing and whole-primer counts when the claimed letter changes, a
  substitution at a primer's 3′ end, a reverse primer's index mapping, an
  abutting pair at the H1 bound, and rejection of a primer that keeps the
  original base.
- **P1-2:** `crates/core/tests/q5_primer_fixtures.rs` (8 tests) reads
  `tests/fixtures/q5-primer/` with a small built-in JSON reader (no new
  dependency) and rebuilds each of the six reference designs through
  `construct_primer` and `assemble_pair`. Each three-base edit is applied as a
  residue substitution on a one-codon CDS over the edited codon, with the
  replacement codon supplied. It checks the template digest, the reproduced
  edit, each primer's text, length, span, changed positions, annealing
  sequence, spans and length, annealing overlap, whole-primer match count and
  part count, the pair's abutment, boundary and gap, the H1 outcome, and
  q5p-07's `spansDisjoint`. luria and the reference generator agree on every
  field; a deliberately broken comparison was confirmed to fail.

The workspace suite now runs 233 tests (197 unit; 8, 6, 14 and 8
integration), 0 failures.

## 11. Design layer: `luriaNative` (D029), 2026-10-14

Implemented after D029's approval, in `crates/core/src/primer_design.rs`:

- `PrimerDesignRequest` (style `Q5Polymerase`, policy `LuriaNative`, required
  `minAnnealingOverlap ≥ 1`, placement) and `design_primers(template, result,
  request)`, which returns a `PrimerDesign`: the pair, the request, the
  template and edited-artifact references, the replacement region, `u`, `v`,
  `a_r`, the algorithm name and version (D021), and the unresolved inputs
  (`minTm`, `tmFunction`, `saltModel`, `maxAnnealingOverlap`; §10).
- The pair is proposed by D029's closed form and then passed through
  `construct_primer`, `assemble_pair` and `require_design_constraints`, so the
  output is checked by the same code as any other primer.
- Four error variants in `error.rs`: `InvalidPrimerDesignRequest`
  (`MalformedInput`: bound below 1, unverified result),
  `UnsupportedPrimerDesignScope` (`Unsupported`: circular template,
  non-contiguous or over-3-base region), `NoPrimerDesignPossible`
  (`BiologicalValidation`) and `NoCenteredPrimerDesign` (`Unsupported`, with
  one obstacle per binding side). A confine-to-tail request is refused with the
  existing `UnsupportedPrimerPlacement`.

Deviations, confirmed by the owner on 2026-10-14:

- §10 records the no-design answer as "a single new variant" whose category
  depends on the cause. `error.rs` assigns one category per variant, so the
  two causes are two variants. The categories are the ones chosen.
  **Confirmed.**
- An ambiguity code (H6) in the template under the centered pair is not one
  of §18.5's outcomes. It is reported as a `NoCenteredPrimerDesign` obstacle
  (`AmbiguousTemplateBase`), on the reasoning that a shifted pair might avoid
  it. **Confirmed for point-mutation mode** ([docs/specs/q5-primer-design.md](/specs/q5-primer-design.md)
  §18.2, §18.5).

Owner note for future work (2026-10-14): if an indel/substitution mode like
NEBaseChanger's is added, it should accept ambiguity codes in the
**replacement** sequence and expand them combinatorially into one primer pair
per concrete sequence, as NEBaseChanger's help describes ("a replacement
sequence containing NS will be expanded to 8 sequences containing: AC, CC, GC,
TC, AG, CG, GG, and TG, generating 8 primer pairs"). Recorded in
[docs/specs/q5-primer-design.md](/specs/q5-primer-design.md) §19. Not implemented; the mutation layer currently
restricts replacements to A/C/G/T.

Tests: 16 unit tests in `primer_design.rs` on a 60-base synthetic template,
with expectations derived by hand from D029's closed form (the derivation is
written into the test module's comment; no shared code); and 3
integration tests in `q5_primer_fixtures.rs` against the new
`tests/fixtures/q5-primer/native.tsv` (8 cases; see that README), including
an exact forward-primer match with the reference designs for `q5p-01` and
`q5p-07` variant-b. Moving the split point by one base was confirmed to fail
both suites.

---

## 7. Acceptance criteria

This milestone is complete only when:

- [docs/specs/q5-primer-design.md](/specs/q5-primer-design.md) states inputs, outputs, hard requirements,
  preferences, and unknowns as separate categories;
- every rule names the decision that fixes it, or is marked **new**;
- no rule rests on an observed website behaviour without its scope stated;
- the fixture set covers substitution above the bound, substitution at the
  bound, a shared-base overlap, insertion, deletion, separate variants, a
  reverse-strand edit, an unsatisfiable constraint, and an unsupplied
  constraint, and names which primer sits at or above the bound in each;
- every fixture case names the artifact its biology comes from;
- every derived field in the reference designs is asserted against the primer
  text and the template by the generating script, and reported-versus-derived
  provenance is recorded per design;
- the reference designs are reproducible from a script in the fixture set;
- no fixture asserts a design for `luria`;
- the absence of insert and delete operations in `crates/core` is reported,
  not worked around;
- no Rust changed, and the existing 190-test suite still passes.

The last two criteria belong to the contract pass. The implementation pass that
followed it changed the criteria: it added `crates/core/src/primer.rs` and its
17 tests, and the workspace suite then runs 207 tests with 0 failures. The
primer-design algorithm, the Tm function, and every preference remain
unimplemented.
