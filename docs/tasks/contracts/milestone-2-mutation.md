---
type: Task Contract
title: "Milestone 2 task record: deterministic mutation domain"
description: "This file records the concrete semantic choices made while implementing Milestone 2, in the same style as milestone-1-core-foundation.md."
status: deprecated
tags: [milestone-2]
---
# Milestone 2 task record: deterministic mutation domain

Status: Implemented in `crates/core`
Date: 2026-10-06

This file records the concrete semantic choices made while implementing
Milestone 2, in the same style as `milestone-1-core-foundation.md`. It does
not restate [docs/decisions/](/decisions/); each entry names the decision it implements.
If an entry contradicts [docs/decisions/](/decisions/), that document wins and this entry
is wrong.

## Scope actually implemented

Only the mutation domain: mutation intent, codon-selection policy, target
validation, deterministic sequence editing, the mutation result and its
provenance, and an independent verification step. No LLM adapter, no natural
language, no CLI, no persistence, no Python worker, no primer or Q5 design
logic, no workflow layer.

The mutation engine is a library: a future workflow layer calls the same
typed operations. Nothing here depends on a workflow existing.

## The contract

### What constitutes a mutation request (D011, D016)

A mutation request is a typed intent, built only through a validating
constructor. There are two intents, and they are different types, because
they assert different things about different artifacts:

* `NucleotideMutationIntent` — a DNA-level substitution. It names
  the source sequence artifact (`SequenceReference`), the target nucleotide
  coordinate (a `NucleotideInterval` of length 1, which itself names its
  reference sequence), the expected original nucleotide, and the replacement
  nucleotide.
* `ResidueMutationIntent` — an amino-acid substitution as in D011. It names
  the target residue (`ResiduePosition`, which carries its CDS identifier and
  a one-based number), the expected original residue, the replacement residue,
  and a `CodonSelectionPolicy`.

A codon change is not a third intent. It is a `ResidueMutationIntent` whose
policy is `Supplied(codon)`: the DNA edit is stated exactly, the residue
assertion is still checked against it. This keeps the three levels of TODO 5
distinct without inventing a third request type: a nucleotide substitution
asserts a base, a codon change asserts a base-triplet and the residue it
encodes, a residue substitution asserts a residue and lets a policy choose
the triplet.

Separate variants and combined mutations (D011) are **not** modelled here.
Only a single intent is applied at a time. Composing intents is a workflow
concern and needs overlap rules that this milestone does not define.

### What sequence artifact a mutation applies to (D007)

A `Sequence` artifact. The intent names its source by `SequenceReference`
(display identifier plus content digest); the operation is given the artifact
itself and rejects anything that is not that artifact. A same-display-name,
different-content artifact is rejected with `SequenceReferenceMismatch`, the
same rule Milestone 1 established in `interval::require_sequence`.

A residue mutation is additionally given the `Cds` it targets. The CDS must
reference the same artifact.

### How the target position is represented (D006)

* nucleotide target: a `NucleotideInterval`. A single nucleotide is the
  length-1 interval `[p, p+1)`. There is no raw `usize` or `u64` position in
  any mutation API, and no new coordinate type: the Milestone 1 interval is
  the coordinate abstraction, and it already carries its reference sequence.
* residue target: `ResiduePosition`, one-based, naming its CDS.

`NucleotideMutationIntent::create` rejects an interval whose length is not 1:
a DNA substitution replaces one nucleotide, and a wider interval is a
contradictory request, not something to interpret. It also rejects an
interval that names a different artifact from the source the request claims:
the request would be about two sequences at once, which cannot be
represented. That check is the intent's own, and it is `InvalidMutationRequest`
(`MalformedInput`); the artifact-level check inside the operation is
Milestone 1's `SequenceReferenceMismatch` (`BiologicalValidation`), and both
appear in the fixture table.

### How the expected original is represented, and what a mismatch does (D011, D005)

The expected original is a required field of both intents, typed: a DNA
letter for a nucleotide substitution, a `Residue` for a residue substitution.

The domain reads the actual value out of the artifact before writing anything.
If they differ the operation fails with `ExpectedNucleotideMismatch` or
`ExpectedResidueMismatch`, both `BiologicalValidation`: the request is
well-formed, the world simply disagrees with it. No sequence is produced, and
the source artifact is untouched. There is no mode that applies a mutation
without asserting the original.

### How the replacement is represented

* nucleotide: one of `A`, `C`, `G`, `T`, validated at construction. The same
  rule applies to the expected original. IUPAC ambiguity codes are valid
  sequence letters but not substitution targets or replacements: writing `N`
  is not a defined edit, and asserting that a position holds `N` does not
  identify a base. Either is `InvalidMutationRequest` (`MalformedInput`), so a
  position that holds an ambiguity code cannot be mutated by this operation.
* residue: a `Residue` enum value. A residue letter that is not one of the
  twenty standard residues — including `*`, the stop — is rejected as an
  `InvalidMutationRequest`. A mutation to a stop residue is not in this
  domain's scope.
* codon: a `Codon`, constructible only when `genetic_code::lookup` returns
  the `Residue` branch. An ambiguous codon, an out-of-alphabet codon, and a
  stop codon therefore cannot exist as a `Codon`, so no downstream step can
  be handed one.

### Strand (D009, D010)

There is no strand parameter on any mutation call. Strand is read from the
`Cds`, exactly as Milestone 1 required of extraction and translation.

* Residue numbering runs over the extracted, strand-applied CDS text, which
  is what `translate` already produces. Residue 1 of a minus-strand CDS is
  therefore the first residue of the reverse complement, not of the reference
  text.
* The codon's reference coordinates are computed by walking the CDS intervals
  in biological order and, on a minus strand, reading them from the far end.
  The replacement triplet is complemented when written back, so the
  reference text is unambiguous and re-extracting the CDS from the edited
  artifact yields the requested triplet.
* `start > end` is never a reverse-strand encoding (unchanged from D009).
* A codon that straddles the origin of a circular reference is recorded as
  two intervals. The recorded coordinates are a set of reference positions in
  ascending order, `[0,1)` then `[22,24)` for example, not the biological
  wrap order used for feature intervals. Nothing is wrapped silently. The
  intervals name the edited artifact, whose coordinates are the source's
  because a substitution preserves length.

### Nucleotide coordinates, CDS residues, or both

Both, and they are never interchangeable. A nucleotide mutation addresses the
reference sequence; a residue mutation addresses a CDS. A nucleotide
mutation may optionally be scoped to a CDS, in which case the coordinate must
lie inside that CDS's intervals or the operation fails with
`TargetOutsideFeature`. Scoping is a parameter of the operation, not a field
of the intent: the DNA edit is a DNA edit, and the scope says which
annotation it must fall inside.

### Codon selection (D011)

`CodonSelectionPolicy` has exactly three inhabitants:

* `Unspecified` — the request did not say how to choose. Applying it fails
  with `UnspecifiedCodonPolicy` (`MalformedInput`: a required field is
  missing, the user can supply one). Nothing is guessed.
* `Default` — an explicitly named deterministic rule: the codon of the
  replacement residue with the highest count in the *E. coli* K-12 codon usage
  table in `crates/core/src/codon_usage.rs`. Equal counts would be broken by
  standard table order (base order `T, C, A, G`); the current table has no
  tie among any residue's top codons. The table is the Kazusa Codon Usage
  Database entry for *E. coli* W3110, a K-12 substrain (species 316407, 4332
  CDSs, 1372057 codons, retrieved 2026-10-14). The database's entry named
  "Escherichia coli K12" (species 83333) is built from only 14 CDSs; it gives
  the same top codon for all twenty residues. The chosen codons are
  `A GCG, C TGC, D GAT, E GAA, F TTT, G GGC, H CAT, I ATT, K AAA, L CTG,
  M ATG, N AAC, P CCG, Q CAG, R CGC, S AGC, T ACC, V GTG, W TGG, Y TAT`.
* `Supplied(codon)` — the caller states the triplet; it must encode the
  replacement residue or the request is contradictory (`InvalidMutationRequest`).

`parse_codon_policy` is the boundary parser: `default`, `unspecified`,
`supplied:XXX`, and anything else is `UnsupportedMutationPolicy`
(`Unsupported`). Near matches are not aliased.

The chosen codon is always recorded in the result (D011).

`Default` is permitted only because the request must name it explicitly
(D011 forbids inferring a policy silently); a request that omits the policy is
`Unspecified` and fails. Choosing the highest-usage *E. coli* K-12 codon was
approved by the project owner on 2026-10-14, replacing the earlier
first-in-table-order rule.

### Comparison with NEBaseChanger codon selection

NEBaseChanger is an external reference, not a specification (D014). This
comparison records where luria's codon selection agrees with it and where it
deliberately does not, so a future change can be made on purpose.

Sources: the reverse-engineering notes on the
`research/nebasechanger_reverse_engineer` branch
([docs/research/nebasechanger-client-investigation.md](/research/nebasechanger-client-investigation.md) §4.1,
[docs/research/nebasechanger-live-build-4330b0c7-2026-10-10.md](/research/nebasechanger-live-build-4330b0c7-2026-10-10.md) §10.4, and the
in-app help [docs/references/nbc_help_v284.txt](/references/nbc_help_v284.txt)), and a comparison on
2026-10-14 of the `Ecoli_K12` table extracted from the archived v2.5.3 and
v2.8.4 bundles and the live build `4330b0c7` on that branch, against the
Kazusa pages cited above.

**How NEBaseChanger selects a codon.** Its bundled table `cut.Ecoli_K12` has
one `[three-letter residue, codon, fraction]` row per codon, fractions rounded
to two decimals, ordered by descending fraction within each residue, plus a
65th row `["All","NNK",1]` for saturation mutagenesis (`X`). The table is
identical in all three builds. NEB's help says only that the usage is "obtained
from E. coli K12"; no source is named.

* Maximum codon usage, the default when no triplet is given: filter the table
  to the residue, stable-sort by descending fraction, take the first codon.
* Maximum parsimony, an option: filter to the residue, stable-sort by
  ascending Hamming distance to the original codon, take the first. Because
  the table is already in descending usage, a distance tie goes to the more
  used codon; equal fractions go to bundle order.
* An explicit triplet (`A34F:TTC`) overrides both.

**Where luria agrees.**

* `Default` is NEBaseChanger's maximum codon usage, and the two pick the same
  codon for all twenty residues.
* `Supplied(codon)` corresponds to an explicit triplet.
* The table is not byte-identical but tracks the same data: 57 of NEBC's 64
  fractions equal Kazusa W3110's, and the other 7 (`TGC`, `TGT`, `GGC`, `AAA`,
  `AAG`, `CCG`, `ACT`) differ by exactly 0.01. Against Kazusa's 14-CDS
  "Escherichia coli K12" entry (species 83333), 55 of 64 differ.

**Where luria differs.**

| Behaviour | NEBaseChanger | luria |
| --- | --- | --- |
| Ranking data | two-decimal fractions | raw Kazusa W3110 counts; same top codon for every residue, but lower ranks can differ where NEBC fractions tie (e.g. Leu `TTG`/`TTA` 0.13, four Ser codons at 0.15) |
| Maximum parsimony (fewest base changes) | available | **not implemented** |
| Mutation to a stop codon | allowed (`*` → `TAA`) | refused, `InvalidMutationRequest` |
| Saturation mutagenesis (`X` → `NNK`) | allowed | not representable; a substitution writes only `A`, `C`, `G`, `T` |
| Original residue does not match | help text: "processing is not halted"; codon number wins | refused, `ExpectedResidueMismatch` (D011, D005) |

The stop, `NNK`, and mismatch rows are deliberate fail-closed choices, not
gaps. Parsimony is a gap. The two strategies are not interchangeable: from
`GAT`, usage and parsimony pick different codons for `A C G L N P R S T V`
(investigation notes §4.1). For the D404N example-construct fixture,
NEBaseChanger's captured output used `AAC` (maximum usage); parsimony would
have given `AAT`.

### Domain errors versus biological validation failures (D005)

* `MalformedInput`: the request could not be represented, or a required field
  is missing — a bad nucleotide letter, a non-triplet codon, a length-≠1
  interval, a contradictory intent, residue 0, no codon policy.
* `BiologicalValidation`: a well-formed request that disagrees with the
  artifact — wrong source artifact, coordinate outside the sequence,
  coordinate outside the scoped CDS, residue outside the CDS, expected
  original does not match, a claimed result that does not follow from its
  source (`MutationVerificationFailure`).
* `Unsupported`: well-formed but outside this milestone's scope — an
  unrecognised codon policy, and everything inherited from Milestone 1
  (incomplete CDS, unsupported genetic code or numbering). An ambiguous codon
  is `BiologicalValidation`, as Milestone 1 classifies it.

### Provenance the result carries (D021, D020)

Every result records: the source `SequenceReference` (name plus digest), the
edited `Sequence` artifact and its `SequenceReference`, the target, the
expected original, the actual original read from the source, the replacement,
and for residue mutations the identifier of the CDS it was applied to, the
original codon, the selected codon, the policy,
and the codon's reference coordinates. `MUTATION_ALGORITHM_NAME` and
`MUTATION_ALGORITHM_VERSION` identify the code path, per D021; the version is
`0.2.0` since the `Default` policy changed. `CodonSelectionPolicy::source()`
names the codon usage table a `Default` choice depends on. The general
provenance model stays deferred.

`MutationState` is the mutation-domain subset of the D020 lifecycle:
`Calculated` (produced by the edit) and `Verified` (accepted by the
independent verification pass). A `Calculated` result is never presented as
`Verified`, and the two are unequal values.

## Verification is separate from generation (D013)

`verify` re-derives, without re-running the edit. Any check that does not
hold fails with `MutationVerificationFailure` (`BiologicalValidation`), whose
payload names the result and says which claim failed:

* that the source artifact is the one the result claims (name, digest, and
  topology);
* that length is preserved, which is the substitution invariant;
* for a nucleotide result, that the set of positions where source and edited
  differ is exactly the recorded target coordinates;
* for a residue result, that that set is non-empty and lies inside the
  recorded codon coordinates: a replacement triplet may share letters with
  the triplet it replaces, so the edit can be a proper subset of the codon,
  but an empty difference is not a mutation at all;
* that the source really holds the recorded expected nucleotide, and the
  edited artifact the replacement nucleotide, at a nucleotide target.

`verify` given a residue result also requires the CDS, and adds:

* that the CDS handed to `verify` is the CDS the result names by identifier,
  so the residue claims are checked against the annotation the result was
  actually applied to;
* re-translating the source and the edited artifact over the same CDS
  coordinates, rebuilt against the edited artifact;
* that the two residue sequences agree at every residue except the requested
  one, that the source holds the recorded expected residue there, and that
  the edited artifact holds the recorded replacement residue there; the
  residue letter itself may stay the same, as when a codon is re-expressed,
  because the DNA still changed
* that the codon read back out of the edited CDS text at the requested residue
  is the recorded codon.

A request whose chosen codon is the codon already at that residue is refused
by the operation before anything is written, so no result records a mutation
that changes nothing; `verify` refuses one too, because an empty difference
between the two artifacts is not a mutation.

Verification is parameterised by a scope, and that scope has to agree with the
result's provenance. A residue result names the CDS it was applied to, and
`verify` refuses an annotation with a different identifier: verifying residue
claims against an annotation the result never made would check a different
mutation. A same-identifier annotation given with different coordinates is a
caller error that verification catches through the claims it can check — the
residue must exist, the residue sequences must agree everywhere else, and the
codon must read back — rather than by re-deriving the residue-to-coordinate
mapping, which is deliberately the generator's code path. A nucleotide result
records no scope, only the coordinate it wrote, so verifying it against a
different scope than the one used at apply time is not a claim a result makes
and is not a claim verification can contradict.

The residue checks go through Milestone 1's extraction and translation, which
is a different code path from the mutation's coordinate mapping and write-back.
Verification is not a claim of experimental success (D020): it is
computational only, and no result ever reports experimental validation.

## Tests and fixtures

* Eleven unit tests in `crates/core/src/mutation.rs`, each named for the rule it
  pins: a mismatch naming the value that is actually there, refusal before any
  edit, plain bases only in a substitution, policy classification and source, verification of a residue result, and the fact
  that mutation failures are not collapsed into one variant.
* `crates/core/tests/mutation_invariants.rs`: fourteen invariants — one-based
  bounded residue positions, artifact identity, strand-aware numbering,
  minus-strand write-back at reference coordinates, refusal before any edit,
  mismatch payloads, a failed mutation leaving the source untouched, a residue
  mutation changing exactly the codon it targeted, deterministic policy,
  verification against the CDS the result names, verification, origin-crossing
  codons recorded as two intervals, refusal of a mutation that changes nothing,
  and error classification by meaning.
* `crates/core/tests/mutation_fixtures.rs` drives
  `tests/fixtures/mutation/cases.tsv` (47 cases, 5 sequence files). Expectations
  are re-derived by hand — digests with `hashlib`, residues off the published
  table, default codons off the Kazusa *E. coli* K-12 counts, edited texts
  written out coordinate by coordinate — and are recorded in
  `tests/fixtures/mutation/README.md`. The driver asserts the edited text, the
  residues re-read through Milestone 1's translation, the recorded codon, the
  recorded coordinates, length preservation, the source identity, the state,
  and the verification outcome. It duplicates none of the implementation's
  classification logic: it reads `DomainError::category()`.

## Known gaps at the milestone boundary

* One intent at a time. Separate variants and combined mutations (D011) are
  not represented; a workflow layer must decide how to compose them.
* Insertions and deletions are not modelled. Every supported mutation is
  length-preserving.
* Only the standard genetic code, only `FromCdsStart` numbering, only
  explicitly complete CDSs — all inherited from Milestone 1 and enforced by
  reusing `translate`.
* `Default` codon selection uses one organism's table, *E. coli* K-12. Other
  hosts, avoiding new restriction sites, and any other optimisation are
  separate concerns and are deliberately absent.
* Maximum parsimony (fewest base changes from the original codon), which
  NEBaseChanger offers, is deferred. When it is added it needs its own named
  policy (`parse_codon_policy` currently refuses `minimal-edit` as
  `UnsupportedMutationPolicy`), a documented tie-break (NEBaseChanger's is the
  usage order of its table), and fixtures for the residues where it disagrees
  with `Default`. See "Comparison with NEBaseChanger codon selection".
* RNA alphabets, circular-sequence insertions, compound features, and
  sequence source metadata remain out of scope.
* Milestone 1's `ResiduePosition` had no consumer before this milestone;
  `ResidueMutationIntent` is now that consumer, and no Milestone 1 semantics
  changed to accommodate it.

## Cross-milestone clarification

Two Milestone 1 files were touched, both mechanically, no semantics changed:

* `genetic_code.rs`: `residue_for_symbol` became public so a residue letter in
  a mutation request can be parsed into the existing `Residue` enum rather
  than a second residue representation being invented.
* `crates/core/tests/core_foundation.rs`: `variant_name` matches every
  `DomainError` variant exhaustively, so the new mutation variants were added
  to that match. The Milestone 1 fixture table is unchanged and its cases all
  still pass.
* `tests/fixtures/README.md`: one row added to the fixture index for the new
  `mutation/` set. The Milestone 1 and example-construct fixtures are untouched.
