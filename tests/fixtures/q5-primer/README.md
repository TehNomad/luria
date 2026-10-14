# Fixture: Q5 primer-design contract cases

## Purpose

These cases pin the **contract** in `docs/q5-primer-design.md`. They do not
test a primer-design algorithm: `crates/core` has none, and none is claimed
here. Every case states inputs and the invariants that must hold of *any*
design of those inputs, so the contract is testable before an algorithm is
chosen (§14 of the contract document).

Two kinds of case appear, and they are marked differently:

- **reference-design cases** (`reference_design = yes`) carry a design produced
  by a reference tool. That design is external reference data, not an
  expectation on `luria` (D008, D011, D016). It is here to make the contract's
  numeric claims checkable — the bound-versus-target distinction, the overlap
  count — and because it is reproducible, not transcribed.
- **contract-only cases** (`reference_design = no`) carry inputs and the
  invariants alone. Nothing is asserted about what a design would look like.

## Files

| Path | Role |
| --- | --- |
| `cases.tsv` | 10 cases, one per line, tab-separated. Column meanings below. |
| `reference/designs.json` | The reference designs for the reference-design cases, keyed by `case_id`. Generated, not transcribed. |
| `reference/neb-q5-sdm-reference.py` | The script that generates `designs.json`. **Not** luria's algorithm; a transcription of the reference tool's Q5 SDM `replace` path, kept so the reference values are reproducible. |

No sequence file is duplicated here. `sequence_file` is a path relative to
`tests/fixtures/`, so the biological input is the artifact another fixture
already validated.

## Column meanings

`case_id`, `kind`, `sequence_file`, `sequence_digest_16`, `edits`,
`min_overlap`, `target_tm`, `strand`, `cds`, `reference_design`,
`expected`, `invariants`, `note`

- `kind` is `replace`, `insert`, `delete`, or `replace-set`.
- `sequence_digest_16` is the first 16 hex digits of the SHA-256 of the
  artifact's canonical text — FASTA record text, header excluded, whitespace
  removed, upper-cased (D002, D016). `mut-a.txt`'s digest matches the one the
  mutation fixture records for the same artifact.
- `edits` is `start:end:original:replacement`, zero-based half-open (D004),
  with `;` between edits of a `replace-set`. An empty `original` means an
  insertion; an empty `replacement` means a deletion.
- `min_overlap` is an integer. It is the value the contract's `minAnnealingOverlap`
  (§2.2) is given. The reference script feeds it into the reference tool's own
  *minimum primer length* slot. The help text restricts that parameter to the
  annealing portion **only when modifications are constrained to the 5′ tails**
  (§13.1 item 5); these reference designs are inline, so the restriction does
  not apply to them. The two quantities are **not** established to be the same
  (§15 question 4), so the reference record states both measured quantities and
  the comparison separately rather than equating them.
- `target_tm` is a number or `absent`; `absent` means the caller did not supply
  it, which is not the same as supplying a value.
- `strand` is the strand the edit's *semantics* are stated on. It does not
  change the DNA edit; it changes which primer is `forward` (§6).
- `expected` is `design`, `design-per-variant`, `contract-only`, or
  `no-design`. `no-design` is the failure contract: the case has no design
  that satisfies the hard requirements, and a layer must say so (§10).
- `invariants` names the requirements from `docs/q5-primer-design.md` §7
  (`H1`–`H8`) and §14.1 (`I-1`–`I-13`) that this case exercises. These are
  requirements on a **luria** design; a reference design is not claimed to
  satisfy them.
- `reference_check` states, for a reference-design case, the outcome of
  evaluating that case's recorded fields, with the exact measured quantity:
  `H1=pass(fwd annealing 20 > 15; rev annealing 23 > 15)` or
  `H1=VIOLATED(fwd annealing 20 < 24; rev annealing 25 > 24)`. For a case with
  no design record it is `unverified(...)` naming the invariants that no
  recorded field permits checking. `invariants` then lists the requirements the
  case **would** exercise if a design were recorded, and asserts nothing about
  them having been checked. No case records `none`.

## Why each case is in the set

| Case | What it separates |
| --- | --- |
| `q5p-01-sub-above-bound` | A bound and a target. `min_overlap = 15`; the reference pair's **annealing** overlap is 20 and 23 — both above the bound, neither equal to it. A design that read 15 as a target would emit a different pair. |
| `q5p-02-sub-above-bound-longer` | Raising the bound to 24 changed **only the reverse primer** (annealing 23 → 25, above 24). The forward primer stays at annealing 20, **below** 24, so this reference design violates H1. The reference tool extends one primer, not both — a reference-observed behaviour, not a luria rule. |
| `q5p-03-sub-at-bound` | The same 25-long reverse primer against a bound of 25: the reverse primer's annealing overlap **equals** the bound. This separates `>=` from `>` **for the reverse primer only**. The forward primer (annealing 20) is below the bound, so the case is not a pair-level witness and the reference design violates H1. |
| `q5p-04-shared-base-count` | The three quantities that the same primer exhibits. Its span is 23 wide, its whole-primer position-match count is 21, and its **annealing** overlap is 20: `GAT → AAC` shares the middle base, and that base is a mutated position, so it is not annealing context (H8, I-13). This is the case that separates the whole-primer count from the contract's overlap. |
| `q5p-05-insertion` | The degenerate changed region. The inserted text occupies nothing in the template, so all overlap must come from context. |
| `q5p-06-deletion` | The empty replacement. No primer position can carry a deletion; the pair is defined by the context it preserves. |
| `q5p-07-separate-variants` | One pristine template, two independent pairs (D020). The reference spans 1176–1222 and 1469–1516 are disjoint; nothing is merged. |
| `q5p-08-reverse-strand` | Which primer is `forward` when the edit's semantics are on the minus strand. The DNA edit at 9–12 is the same either way (§6). `mut-a.txt` is the artifact the mutation fixture's `res-a-minus-7` case uses, **not** the same edit: that case acts on residue 7 (`ATT → L`), which writes DNA `AAT → TAA` at 9–12 and the codon `ATT → TTA` at reference coordinates 9,10,11, while this case declares DNA `AAT → AAC` at 9–12. |
| `q5p-09-bound-exceeds-template` | A bound no design can meet: 31 required overlap against a 30-base template. Unsatisfiable under every geometry, so the failure is not a property of one search. |
| `q5p-10-no-tm-target` | An unsupplied constraint. `target_tm = absent` must appear as absent in the output, never as a substituted default (§10). |

## Scope and known limitations

- **Insertions and deletions are not representable in `crates/core` today.**
  The mutation layer implements nucleotide and residue *substitution* only
  (`apply_nucleotide`, `apply_residue`); there is no insert or delete
  operation. `q5p-05` and `q5p-06` therefore state their edit directly rather
  than taking it from a `MutationResult`, and are contract-level cases, not
  validated mutation outputs.
- **The reference designs cover the reference tool's `replace` path only.**
  Its insert and delete paths take a different shape — they design a whole
  fragment pair rather than a local pair — which is an observation about the
  reference tool, not a requirement on `luria`. No reference design is offered
  for `q5p-05` or `q5p-06` for that reason.
- **Five cases carry no design record** (`q5p-05`, `q5p-06`, `q5p-08`, `q5p-09`,
  `q5p-10`). Their `invariants` column names the requirements the case would
  exercise, and their `reference_check` column says `unverified(...)` because no
  recorded field permits checking them. They pin inputs, not behaviour.
- **H1 is evaluated on the annealing overlap, never on primer length.** The
  reference forward primer is 23 long, its whole-primer match count is 21, and
  its annealing overlap is 20. All three are recorded, under different names;
  only the annealing overlap is the quantity H1 bounds, and only that one is
  compared in `reference_check`.
- **No reference design here is claimed to satisfy H1–H8.** The reference tool's
  inline `replace` path leaves the forward primer's annealing overlap at 20 while
  the supplied bound rises to 24 and 25, so the `q5p-02` and `q5p-03` reference
  designs violate H1. They are kept, and the violation is recorded in each
  record's `h1` block and in `cases.tsv`'s `reference_check` column.
- `q5p-08` uses `mut-a.txt` with the reverse CDS `cds-2` from the mutation
  fixture — the same artifact that fixture's `res-a-minus-7` case mutates, but
  **not the same edit**. `res-a-minus-7` acts on residue 7 (`ATT` → `L`), which
  writes the DNA `AAT → TAA` at 9–12 and the codon `ATT → TTA` at reference
  coordinates 9,10,11. This case declares the DNA edit `AAT → AAC` at 9–12. At
  30 bases it is too short for any reference design, so the case is
  contract-only.
- No case here asserts a Tm value for `luria`. The Tm values in
  `designs.json` are labelled with the reference function that produced them;
  the project's own Tm function is not yet defined (§11, §13.3).
- No `Ta` rule is asserted. The reference tool prints one and nothing in this
  workspace defines it (§13.1 item 8, §15 question 7).

## Provenance of the reference designs

`reference/designs.json` is generated by `reference/neb-q5-sdm-reference.py`,
which transcribes the Q5 SDM `replace` path from the saved NEBaseChanger
material. Running it reproduces the SID-1 D404N pair that
`tests/fixtures/example-construct/raw/sdmsummary.txt` records, which is why
the reference values here are reproducible rather than copied. The script is
not called by anything in `crates/core`.

### Reported by the raw artifact, versus derived by the script

`raw/sdmsummary.txt` has the columns `ID`, `Description`, `Fwd Primer ID`,
`Fwd Primer Seq`, `Length`, `Tm Fwd`, `Rev Primer ID`, `Rev Primer Seq`,
`Length`, `Tm Rev`, `Ta`. For the D404N row it reports:

| Reported | Value |
| --- | --- |
| `forward.sequence`, `forward.length` | `CGTTGAGCAAaacTTCGGGGCGC`, 23 — the raw row lower-cases the replacement; `designs.json` stores `CGTTGAGCAAAACTTCGGGGCGC`, so the case pattern is presentation (D001) and is **not** a reported value |
| `reverse.sequence`, `reverse.length` | `AGCGGATTGAGCAGTTTTCCGTC`, 23 |
| `Tm Fwd`, `Tm Rev` | 74, 70 (printed rounded; the script computes 74.18, 69.94) |
| `Ta` | 71 |

**It reports no overlap column, no span, no orientation, no tail/annealing
split, and no changed positions.** Every one of those is derived by the script
from its own construction, and each design carries a `provenance` block naming
the reported and derived fields. Only the D404N row corresponds to a design in
`designs.json` — `q5p-01`, `q5p-04` and `q5p-07` variant-a. The `q5p-02`,
`q5p-03` and `q5p-07` variant-b records have no raw artifact behind them: they
are what the transcribed path produces for a different bound, and their
`provenance.reportedBy` is `null`.

### The decomposition each reference primer carries

The reference `replace` path builds the forward primer as
`template[fwd_start:start] + replacement + template[end:end+o]`, so its parts
are known exactly rather than inferred. Each primer record therefore carries:

- `sequence`, `length`, `span`, `oneBasedSpan`, `editedSpan` — the primer and
  where it sits (zero-based half-open, D004; 1-based inclusive alongside, D015);
- `orientation` — `same` or `opposite` against the construct (H4, I-7);
- `parts` — context / mutation / context, each with its template span and its
  primer index range; the parts abut and rebuild `sequence`, which the script
  asserts;
- `tail` — `null` for these designs, with `tailReason` stating why: the mutated
  positions sit at primer indices 10..13, not at the 5′ end, because the
  reference path is `inline` placement (`mutationPlacement: inline`);
- `changedPositions` — primer indices carrying the mutation (forward: 10,11,12;
  reverse: none);
- `annealing` — its sequence, its template spans, and its length; the spans
  reconstruct the sequence from the template, which the script asserts (H3,
  I-1);
- `annealingOverlap` — the contract's quantity: matches counted over annealing
  positions only (H8, I-13);
- `wholePrimerMatchCount` — the position-by-position count over the whole
  primer, mutated positions included. For the forward primer this is 21 while
  `annealingOverlap` is 20; the difference is the shared middle base.

The script asserts the decomposition against the primer text and the template
for every design it writes, so a record that contradicts itself fails the run
rather than being written. The named checks are:

- **H3** — `forward.sequence` equals the edited sequence at `editedSpan` (the
  edited sequence is reconstructed from the template and the recorded `edit`, so
  H3 is checkable from the record plus `sequence_file` alone), and
  `reverse.annealing` is the reverse complement of the template under its spans;
- **H4 / I-7** — both primers state an orientation, and the two are opposite;
- **H8** — the annealing positions and the mutated positions partition each
  primer, with no gap and no overlap;
- **I-1** — each `annealing.sequence` is reconstructible from its `spans`;
- **I-13** — `annealingOverlap` is counted over annealing positions only, and is
  never above `wholePrimerMatchCount`;
- **H2** — the changed region lies inside the pair's covered span;
- **H5** — `gap` and `abut` are derivable from the two spans, not stored apart;
- **I-6** — `builtAgainst` is the digest of the template actually loaded.

## Cloud sharing

The SID-1 cases carry a real construct sequence and captured reference-tool
output. No approval status for public cloud sharing has been recorded, so they
must not be sent to a cloud model without explicit approval (D012). The
`mut-a.txt` cases are synthetic and computational only (D020).
