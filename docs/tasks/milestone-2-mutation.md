# Milestone 2 task record: deterministic mutation domain

Status: Implemented in `crates/core`
Date: 2026-10-06

This file records the concrete semantic choices made while implementing
Milestone 2, in the same style as `milestone-1-core-foundation.md`. It does
not restate `docs/decisions.md`; each entry names the decision it implements.
If an entry contradicts `docs/decisions.md`, that document wins and this entry
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

* nucleotide: a DNA letter, validated against `DNA_ALPHABET` at construction.
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
  two intervals, `[22,24)` and `[0,1)` for example. Nothing is wrapped
  silently.

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
* `Default` — an explicitly named deterministic rule: the first codon of the
  replacement residue in the published standard-code table order, base order
  `T, C, A, G`, lowest table index first. This is a representation rule over
  a published table, not a biological optimisation.
* `Supplied(codon)` — the caller states the triplet; it must encode the
  replacement residue or the request is contradictory (`InvalidMutationRequest`).

`parse_codon_policy` is the boundary parser: `default`, `unspecified`,
`supplied:XXX`, and anything else is `UnsupportedMutationPolicy`
(`Unsupported`). Near matches are not aliased.

The chosen codon is always recorded in the result (D011).

Interpretation flagged for review: D011 forbids inferring a policy silently,
not forbidding a named default policy. `Default` is therefore permitted only
because the request must name it explicitly; a request that omits the policy
is `Unspecified` and fails. If that reading is wrong, `Default` should be
removed and only `Supplied` kept.

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
  (incomplete CDS, unsupported genetic code or numbering, ambiguous codon).

### Provenance the result carries (D021, D020)

Every result records: the source `SequenceReference` (name plus digest), the
edited `Sequence` artifact and its `SequenceReference`, the target, the
expected original, the actual original read from the source, the replacement,
and for residue mutations the identifier of the CDS it was applied to, the
original codon, the selected codon, the policy,
and the codon's reference coordinates. `MUTATION_ALGORITHM_NAME` and
`MUTATION_ALGORITHM_VERSION` identify the code path, per D021; the general
provenance model stays deferred.

`MutationState` is the mutation-domain subset of the D020 lifecycle:
`Calculated` (produced by the edit) and `Verified` (accepted by the
independent verification pass). A `Calculated` result is never presented as
`Verified`, and the two are unequal values.

## Verification is separate from generation (D013)

`verify` re-derives, without re-running the edit. Any check that does not
hold fails with `MutationVerificationFailure` (`BiologicalValidation`), whose
payload names the result and says which claim failed:

* that the source artifact is the one the result claims (name and digest);
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

* Nine unit tests in `crates/core/src/mutation.rs`, each named for the rule it
  pins: a mismatch naming the value that is actually there, refusal before any
  edit, policy classification, verification of a residue result, and the fact
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
  `tests/fixtures/mutation/cases.tsv` (44 cases, 5 sequence files). Expectations
  are re-derived by hand — digests with `hashlib`, residues off the published
  table, edited texts written out coordinate by coordinate — and are recorded in
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
* `Default` codon selection is a table-order rule. Minimal-edit distance,
  organism codon usage, and any other optimisation are separate concerns and
  are deliberately absent.
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
