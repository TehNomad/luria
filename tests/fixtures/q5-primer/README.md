# Fixture: Q5 primer-design contract cases

## Purpose

These cases pin the **contract** in `docs/q5-primer-design.md`. `cases.tsv`
does not test a primer-design algorithm: every case states inputs and the
invariants that must hold of *any* design of those inputs, so the contract is
testable independently of an algorithm (§14 of the contract document).
`native.tsv` is the exception: it holds expected designs under luria's own
policy, `luriaNative` (D029), described in "Native designs" below.

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
| `native.tsv` | 8 expected `luriaNative` outcomes (D029): four designs and four failures. Generated, not transcribed. |
| `reference/native-design.py` | The script that generates `native.tsv` from D029's closed form. Shares no code with `crates/core`. |
| `native-deletion.tsv` | 13 expected `luriaNative` deletion outcomes (D031): seven designs and six failures. Generated, not transcribed. |
| `reference/native-deletion.py` | The script that generates `native-deletion.tsv`. Reuses only the Tm and reverse-complement helpers of `native-design.py`. Shares no code with `crates/core`. |
| `native-insertion.tsv` | 15 expected `luriaNative` insertion outcomes (D032): 11 designs and 4 failures. Generated, not transcribed. |
| `reference/native-insertion.py` | The script that generates `native-insertion.tsv`. Same independence as the deletion script. |
| `native-replacement.tsv` | 15 expected `luriaNative` replacement outcomes (D033): 11 designs and 4 failures. Generated, not transcribed. |
| `reference/native-replacement.py` | The script that generates `native-replacement.tsv`. Same independence as the other two. |
| `reference/unit-expectations.py` | Prints the expectations the unit tests in `crates/core/src/primer_design.rs` and `crates/core/src/q5_design.rs` hard-code for the templates they build there, which `native-design.py`'s `--synthetic` mode does not cover. Loads `native-design.py` and `native-replacement.py` from its own directory; writes nothing. |

No sequence file is duplicated here. `sequence_file` is a path relative to
this directory (`tests/fixtures/q5-primer/`), e.g.
`../example-construct/input/sid-1_cDNA.fa`, so the biological input is the artifact another fixture
already validated.

## Rerunning the generators

Every generator in `reference/` has a `--check` mode (D039): it builds its
output text exactly as it does today, compares it byte for byte with the
tracked file, prints `OK <name>` or `DIFFERS <name>` followed by the first
differing line, exits 1 on a difference, and writes nothing. Without `--check`
it rewrites its tracked file. Run them from the repository root with `python -I`;
the four `native-*.py` scripts need **Biopython 1.88**, pinned in
`tests/fixtures/requirements.txt`, and `neb-q5-sdm-reference.py` needs none, as
it inlines the reference Tm model.

```
python -I tests/fixtures/q5-primer/reference/native-design.py                 # rewrite ../native.tsv
python -I tests/fixtures/q5-primer/reference/native-design.py --check         # compare, write nothing
python -I tests/fixtures/q5-primer/reference/native-deletion.py               # rewrite ../native-deletion.tsv
python -I tests/fixtures/q5-primer/reference/native-deletion.py --check       # compare, write nothing
python -I tests/fixtures/q5-primer/reference/native-insertion.py              # rewrite ../native-insertion.tsv
python -I tests/fixtures/q5-primer/reference/native-insertion.py --check      # compare, write nothing
python -I tests/fixtures/q5-primer/reference/native-replacement.py            # rewrite ../native-replacement.tsv
python -I tests/fixtures/q5-primer/reference/native-replacement.py --check    # compare, write nothing
python -I tests/fixtures/q5-primer/reference/neb-q5-sdm-reference.py          # rewrite reference/designs.json
python -I tests/fixtures/q5-primer/reference/neb-q5-sdm-reference.py --check  # compare, write nothing
python -I tests/fixtures/q5-primer/reference/unit-expectations.py             # print unit-test expectations, write nothing
```

`native-design.py`'s `--synthetic` mode is unchanged: it prints expectations for
the synthetic templates in its `SYNTHETIC`, the ones `primer_design.rs`'s
branch tests build. `unit-expectations.py` prints, and writes nothing, so it has
no `--check`: it covers the other templates those two Rust modules build inside
their tests — `primer_design.rs`'s 60-base `TEMPLATE` and `q5_design.rs`'s 90-base
template, and the two `native-replacement.py` designs near `primer_design.rs`'s
tail-path test — and it names each case, so a test comment can point at the case
that produced its figures. None of this is part of the required cargo checks
(D039, D026).

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
  change the DNA edit, and it does not change which primer is `forward`:
  that is a property of the construct's orientation (§6).
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
| `q5p-08-reverse-strand` | Which primer is `forward` when the edit's semantics are on the minus strand. The DNA edit at 9–12 is the same either way (§6). `mut-a.txt` is the artifact the mutation fixture's `res-a-minus-7` case uses, **not** the same edit: that case acts on residue 7 (`ATT → L`), which writes DNA `AAT → CAG` at 9–12 and the codon `ATT → CTG` at reference coordinates 9,10,11, while this case declares DNA `AAT → AAC` at 9–12. (The `note` column of `cases.tsv` still gives the pre-2026-10-14 `TAA`/`TTA`.) |
| `q5p-09-bound-exceeds-template` | A bound no design can meet: 31 required overlap against a 30-base template. Unsatisfiable under every geometry, so the failure is not a property of one search. |
| `q5p-10-no-tm-target` | An unsupplied constraint. `target_tm = absent` must appear as absent in the output, never as a substituted default (§10). |

## Scope and known limitations

- **`q5p-05` and `q5p-06` predate indel support.** When they were written the
  mutation layer had substitution only, so they state their edit directly
  rather than taking it from a `MutationResult`, and they remain
  contract-level cases. Insertions, deletions and replacements are now
  implemented (`apply_insertion`, `apply_deletion`, `apply_replacement`,
  D031–D033), and their expected designs are in `native-insertion.tsv`,
  `native-deletion.tsv` and `native-replacement.tsv`.
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
  under the `Default` policy writes the DNA `AAT → CAG` at 9–12 and the codon
  `ATT → CTG` at reference coordinates 9,10,11 (before the 2026-10-14 review,
  `AAT → TAA` and `TTA`). This case declares the DNA edit `AAT → AAC` at 9–12. At
  30 bases it is too short for any reference design, so the case is
  contract-only.
- `cases.tsv` and `designs.json` assert no Tm value for `luria`; the Tm
  values in `designs.json` are labelled with the reference function that
  produced them. luria's own Tm (`tm.rs`, D030) and annealing temperature are
  asserted by the `native*.tsv` files (`forward_tm`, `reverse_tm`,
  `annealing_temp`), not by these cases.
- Test coverage of `cases.tsv` is no longer partial: `q5_primer_fixtures.rs`
  reads every column except `strand`, `cds` and `note`, which are descriptive
  and checked by no test. Every row is checked, including the contract-only
  rows q5p-05, -06, -08, -09 and -10. Which check reads which column:

| Column | Checked by |
| --- | --- |
| `case_id` | every test that names a row; `every_reference_design_case_has_a_record_and_no_record_lacks_a_case` against the `designs.json` keys |
| `kind` | `every_case_kind_agrees_with_its_edits` |
| `sequence_file` | `template_of` for every row, and `every_case_names_the_digest_of_its_sequence_file` |
| `sequence_digest_16` | `every_case_names_the_digest_of_its_sequence_file` (all 10 rows), and `each_reference_design_is_built_against_the_template_its_case_names` against `builtAgainst` |
| `edits` | `every_case_edit_asserts_the_bases_its_template_holds` against the template, `every_case_kind_agrees_with_its_edits`, and `every_reference_case_agrees_with_its_design_record` against the record's edit |
| `min_overlap` | `every_reference_case_agrees_with_its_design_record` against `request.minOverlapSupplied`, and the bound `the_no_design_case_gets_no_design_from_luria` and `the_no_tm_target_case_reports_the_tm_input_as_unresolved` build their requests from |
| `target_tm` | `every_reference_case_agrees_with_its_design_record` against `request.targetTm` (`55` against `55.0`), and `the_no_tm_target_case_reports_the_tm_input_as_unresolved` for the `absent` row |
| `reference_design` | `every_reference_design_case_has_a_record_and_no_record_lacks_a_case`, and which rows `every_reference_case_agrees_with_its_design_record` and `every_reference_check_matches_its_record` cover |
| `expected` | `the_no_design_case_gets_no_design_from_luria` (`no-design`) |
| `invariants` | `every_case_names_only_defined_invariants` |
| `reference_check` | `every_reference_check_matches_its_record` |

## Provenance of the reference designs

`reference/designs.json` is generated by `reference/neb-q5-sdm-reference.py`,
which transcribes the Q5 SDM `replace` path from the saved NEBaseChanger
material. Running it reproduces the SID-1 D404N pair that
`tests/fixtures/example-construct/raw/sdmsummary.txt` records;
`the_reference_script_reproduces_the_nebasechanger_pair` in
`crates/core/tests/q5_primer_fixtures.rs` is the test that compares them, which
is why the reference values here are reproducible rather than copied. The script is
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

## Native designs

`native.tsv` holds the outcome D029 and D030 predict for eight edits. Its
expectations are **independently derived**: `reference/native-design.py`
restates the policy in plain Python, reads the template, computes every Tm with
**Biopython 1.88** (an independent implementation of the published model; see
`tests/fixtures/tm/README.md`), and writes the spans, lengths, Tms and failure
detail. It does not import or run `crates/core`, and was not written from
luria's output. Rerunning it reproduces `native.tsv` byte for byte (sha256
checked twice on 2026-10-14, after D030). It needs Biopython; the project's
working copy lives in `.tmp/tm/venv`, which is not committed.

Columns: `edit` is `start:end:original:replacement` (0-based, half-open);
`expected` is `design`, `error:NoPrimerDesignPossible` or
`error:NoPrimerDesignFits`; spans are `start:end`; `flanks` is `10:v`, the
forward primer's unchanged bases 5′ (fixed under D030) and 3′ of the edit;
`forward_tm`, `reverse_tm` and `annealing_temp` are °C to two decimals at the
Q5 buffer condition (150 mM monovalent, 500 nM primer); `forward_tm_basis` is
`mismatched_wild_type_site` or, where two changed bases are adjacent,
`matched_complement`; `balance_met` says whether the two Tms ended within
5 °C. `failure` is `pair:<smallest pair>><template length>` for no design
possible, one `<side>:<available><<needed>` per binding side when the starting
pair does not fit, or `tm-floor <side>:<available>:<Tm reached>` when a primer
runs out of template below 55 °C.

The rows cover: the `q5p-01` and `q5p-07` variant-b edits at bound 15
(`native-01`, `native-04`); `q5p-01`'s edit at bounds 24 and 25, where the
forward 3′ flank starts at 14 and 15 (`native-02`, `-03`); the `q5p-08` and
`q5p-09` edits on the 30-base `mut-a`, which no pair fits (`native-05`, `-06`);
and single-base edits 20 bases from SID-1's 5′ end and 4 from its 3′ end
(`native-07`, `-08`). The Tm-floor failures, the G/C revert, both directions
of balancing, an unmet balance and the annealing cap are covered by the unit
tests in `crates/core/src/primer_design.rs` on synthetic templates, with
expectations from the same script's `--synthetic` mode.

**Cross-checks against the reference tool.** For `native-01` and `native-04`
luria's forward primer equals the reference's — same span and text — because
neither needed to grow past 10 + 10. For `native-04` the **reverse primer
matches too**, `[1469, 1493)`, 24 bases: the forward Tm there is on the matched
basis (two adjacent changes), where luria and the reference agree to 0.1 °C,
so both balance to the same length. For `native-01` the reverse primers differ,
19 bases against 23: luria balances against its published-model mismatch-aware
forward Tm of 67.72 °C, the reference against its own, several degrees higher
(`tests/fixtures/tm/README.md`, "Validation of the implementation").

## Native deletion designs

`native-deletion.tsv` holds the outcome D031 predicts for 13 deletions. It
was written by the planner on 2026-10-15, before any deletion design code
existed. `reference/native-deletion.py` restates D031 in plain Python. Its
sizing loops are D030's, with the forward 3′ flank replaced by the whole
forward length. It computes both Tms with Biopython 1.88 on the matched basis.
It loads `native-design.py` only for `bio`, `rc`, `ends_gc` and the
thresholds, and does not import or run `crates/core`. Rerunning it reproduced
the file byte for byte (sha256 `9dc30ca1…`, checked twice on 2026-10-15).

Columns differ from `native.tsv`. `template` is either a fixture path or
`seq:` followed by a synthetic template's full text, and the synthetic
templates are the ones `primer_design.rs`'s unit tests build. `deletion` is
`start:end:deleted bases` (0-based, half-open). `forward` and `reverse` are
the primer texts, 5′→3′. There is no `flanks` or Tm-basis column: a deletion
primer has no mismatch, and both Tms are on the matched basis. `failure`
uses `native.tsv`'s forms.

What the rows cover:

- **Ordinary deletions:** a codon (`del-01`), a single base (`del-02`), 40
  bases (`del-03`), and bound 25 (`del-04`).
- **Floor growth:** on the 5′ side, with balance left unmet (`del-05`); and on
  the 3′ side with a G/C revert (`del-06`).
- **Starting pair does not fit:** on the 5′ side (`del-07`) and the 3′ side
  (`del-08`).
- **Failures:** a template too short for any pair (`del-09`), an `N` under the
  starting reverse primer (`del-10`), and a Tm floor unreachable on each side
  (`del-12`, `del-13`).
- **Real sequence:** `q5p-01`'s codon deleted from SID-1 (`del-11`), where
  balance grows the reverse primer and then extends it to a G/C end.

In every design row, reverse end + width = forward start (D031's pair
invariant). The script asserts it.

## Native insertion designs

`native-insertion.tsv` holds the outcome D032 predicts for 15 insertions. It
was written by the planner on 2026-10-15, before any insertion design code
existed. `reference/native-insertion.py` restates D032 in plain Python, reusing
only `native-design.py`'s `bio`, `rc`, `ends_gc` and thresholds. Rerunning it
reproduced the file byte for byte (sha256 `a9d655ba…`, checked twice).

Columns. `template` is a fixture path or `seq:` followed by a synthetic
template's text. `point` is the 0-based insertion point `p`: the inserted bases
go between `p − 1` and `p`. `inserted` is the inserted text.
`forward_anneal_span` and `reverse_anneal_span` are the **annealing** portions
on the template; a tail covers no template, so it has no span. `forward_tail`
and `reverse_tail` are the non-annealing 5′ tails, or `none`. `forward` and
`reverse` are the whole primers, 5′→3′, tail then annealing portion.
`forward_tm` and `reverse_tm` are the **annealing portions'** Tms, which is
what D032 sizes and balances on; `forward_whole_tm` and `reverse_whole_tm` are
the whole primers' Tms, reported but never used for sizing. `failure` uses
`native.tsv`'s forms.

What the rows cover:

- **The tail rule:** 3 bases (`ins-01`), 1 base (`ins-02`), 6 bases, the
  threshold, still one tail (`ins-03`), 7 bases split 3 reverse / 4 forward
  (`ins-04`), and 14 split 7 / 7 (`ins-05`). The script asserts that the
  reverse tail's reverse complement followed by the forward tail rebuilds the
  insertion exactly.
- **Sizing:** bound 25 (`ins-06`), an AT-rich 5′ side where the reverse primer
  grows (`ins-07`), an AT-rich 3′ side where the forward primer grows
  (`ins-08`), and a point with exactly 15 bases of 3′ room, where the forward
  primer just fits, cannot grow, and balance is left unmet (`ins-10`).
- **Failures:** too little room on the 5′ side (`ins-09`) and the 3′ side
  (`ins-15`), a template too short for any pair (`ins-11`), and an `N` under
  the starting reverse annealing portion (`ins-12`).
- **A 210-base insertion** (`ins-13`), which makes both primers 120 nt, past
  the owner's 100-nt practical ordering limit. D032 still emits it; the length
  is a quality warning, not a refusal.
- **A real template:** an insertion into SID-1 (`ins-14`).

Note that `forward_whole_tm` is *below* `forward_tm` in `ins-06`: a long
annealing portion plus a short tail can melt lower than the annealing portion
alone, because the tail is unpaired. That is why D032 sizes on the annealing
portion.

## Native replacement designs

`native-replacement.tsv` holds the outcome D033 predicts for 15 replacements:
a target region of width `w` replaced by a text of length `m`, with `m` free
to differ from `w`. The planner wrote it on 2026-10-15, before any replacement
design code existed. `reference/native-replacement.py` restates D033 in plain
Python, reusing only `native-design.py`'s `bio`, `rc`, `ends_gc`,
`forward_tm` and thresholds. Rerunning it reproduced the file byte for byte
(sha256 `19ef9087…`, checked twice).

**A cross-check worth noting.** Row `rep-14` is the `q5p-01` edit, 1209–1212
`GAT`→`AAC` at bound 15, expressed as a replacement instead of a codon
substitution. It reproduces `native.tsv`'s `native-01` exactly — forward span
`1199:1222`, reverse `1180:1199`, Tms 67.72 (mismatched basis) and 62.90,
Ta 65.90, balance met. Two scripts written weeks apart agree, which is direct
evidence that D033's inline path really is D030 generalised rather than a
second geometry.

Columns extend `native.tsv`'s. `region` is `start:end:replaced bases` and
`replacement` the new text. `path` is `inline` or `tails`, the branch D033
chose. `forward_tail` and `reverse_tail` are `none` on the inline path.
`forward_whole_tm` and `reverse_whole_tm` are `none` on the inline path too,
where there is no tail and the primer's Tm is already the whole primer's.

What the rows cover:

- **The inline path** (`max(w, m) <= 6`): 3 for 3, which is an ordinary
  substitution (`rep-01`); 3 for 2 and 3 for 5, where the lengths differ
  (`rep-02`, `rep-03`); and 6 for 6, the threshold (`rep-04`).
- **The tail path** (`max(w, m) > 6`): 7 for 7 (`rep-05`); 3 for 7, where the
  replacement's length forces it (`rep-06`); 10 for 2, where the region's
  width does (`rep-07`); and a 12-base replacement split 6 and 6 (`rep-08`).
- **The Tm basis.** `rep-01`, `rep-10` and `rep-14` have `m == w` and use
  D030's mismatch-aware basis. Every row with `m != w` uses the matched
  basis, because a primer and a template site of different lengths have no
  position-by-position alignment for a mismatch-aware Tm to read.
- **Sizing and failures:** an AT-rich 5′ side where the reverse primer cannot
  reach the floor (`rep-09`); an AT-rich 3′ side where the forward primer
  grows and balance is left unmet (`rep-10`); too little 5′ room
  (`rep-11`); a template too short for the pair (`rep-12`); an `N` under the
  starting reverse primer (`rep-13`).
- **A real template:** `rep-14` and `rep-15` on SID-1.

The script asserts D033's tail invariant on every tail row: the reverse
tail's reverse complement followed by the forward tail rebuilds the
replacement exactly.

## Cloud sharing

The SID-1 cases carry a real construct sequence and captured reference-tool
output. They are **approved for cloud-assisted review** by the project owner on
2026-10-14 (D023), together with the `example-construct/` fixture they derive
from; `reference/designs.json` is covered by the same approval. The
`mut-a.txt` cases are synthetic and computational only (D020).
