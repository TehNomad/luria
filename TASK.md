# TASK: Implement Luria First Milestone — Core Sequence Foundation

## Objective

Implement the first Luria milestone in `crates/core`: a small, well-tested Rust
foundation for canonical DNA sequence artifacts, coordinates, features, CDS
representations, strand/topology semantics, and translation.

The goal is a trustworthy domain foundation, not a broad framework.

At the end of this task, Luria should be able to represent and validate the
supported sequence/feature/CDS cases and translate supported CDSs
correctly, with deterministic behavior and structured errors.

## Scope

### In scope

- Rust workspace/bootstrap required to support `crates/core`.
- Canonical sequence artifact model.
- Sequence normalization rules and validation.
- Explicit sequence alphabet and topology.
- Content hashing / sequence identity.
- Zero-based, half-open nucleotide coordinates: `[start, end)`.
- Explicit reference-sequence association for coordinates/features.
- Explicit strand semantics.
- Feature representation.
- Explicit representation for supported origin-spanning features on circular
  sequences.
- CDS representation.
- CDS extraction in the supported cases.
- Translation under the initially supported genetic code.
- Structured domain errors for malformed, invalid, or unsupported inputs.
- Unit and integration tests.
- Shared scientific fixtures under `tests/fixtures/` plus fixture documentation.
- Required Rust formatting, test, lint, and diff checks.

### Out of scope

Do **not** implement any of the following in this task:

- LLM integration or prompt handling.
- Natural-language parsing.
- Mutation intent or amino-acid substitution editing.
- Codon-selection policy.
- Primer design.
- Q5 / NEBaseChanger logic.
- Python workers or subprocess protocols.
- CLI/TUI/web UI.
- Database or persistence layer.
- MCP integration.
- Job queues, worker pools, distributed execution, or orchestration
  frameworks.
- Speculative abstractions for future workflows.

If an implementation detail appears necessary for one of the excluded areas,
do not build it preemptively. Record the issue and stop at the milestone
boundary.

## Read before coding

Treat these repository documents as requirements:

- `AGENTS.md`
- `docs/decisions.md`
- `docs/product-brief.md`

Pay particular attention to decisions covering:

- canonical Rust ownership of biological semantics
- fail-closed behavior
- coordinate conventions
- sequence artifact identity
- topology and strand
- CDS/translation scope
- fixture provenance
- reproducibility
- minimal architecture

Do not silently change a biological assumption from those documents.
If a requirement is ambiguous or contradictory, report it rather than inventing
a convention.

## Required design decisions to settle in this task

Before implementation, resolve the remaining concrete choices needed by the
core model and record them in code/docs/tests as appropriate.

### 1. Sequence alphabet

Initial support should be deliberately narrow.

Recommended baseline:

- DNA only.
- Accept `A`, `C`, `G`, `T` plus explicitly supported IUPAC ambiguity codes
  if the implementation can represent them without guessing biological
  meaning.
- Do not silently convert invalid characters into valid bases.
- Case normalization may be performed at the input boundary and must be
  documented.

If ambiguity handling is not fully defined, reject ambiguous bases for
translation rather than guessing an amino acid.

### 2. Topology

Support exactly:

- linear
- circular

Do not encode circularity using `start > end`.

### 3. Coordinates

Internal nucleotide coordinates are:

`[start, end)`

with zero-based indexing.

Every coordinate-bearing structure must identify its reference sequence.

Validate bounds against the referenced sequence length.

### 4. Strand

Use an explicit strand type rather than a boolean.

At minimum support:

- forward / plus
- reverse / minus

Do not silently interpret missing strand information for CDS extraction.

### 5. Feature intervals

Use a representation that can express an ordered collection of intervals.

This is necessary for origin-spanning circular features and leaves room for
future compound/spliced features without using undocumented numeric tricks.

For this milestone, only support the feature forms explicitly required by the
tests. Unsupported compound/spliced semantics must fail explicitly.

### 6. CDS translation

Initial translation scope:

- complete CDSs only
- standard genetic code only
- residue numbering starts at 1 for the first translated codon
- terminal stop codon is not an amino-acid residue

Translation must respect feature strand and interval order.

Malformed sequences, impossible CDS lengths, unsupported translation metadata,
and internal-stop cases must produce distinct, structured outcomes where
practical.

Do not infer a missing genetic code or repair an invalid CDS silently.

## Recommended core model

Use domain types rather than passing raw strings and integers throughout the
code.

The exact names are up to the implementation, but the conceptual model should
contain equivalents of:

```text
SequenceArtifact
  - id / stable identity
  - canonical bases
  - alphabet
  - topology
  - length
  - content hash
  - source/import metadata as supported

Coordinate
  - reference sequence identity
  - start
  - end

Strand

Feature
  - reference sequence identity
  - ordered intervals
  - strand
  - feature type
  - relevant annotation metadata

Cds
  - feature/reference identity
  - intervals
  - strand
  - completeness
  - genetic code metadata

Translation
  - amino-acid sequence
  - CDS/reference identity as appropriate
  - explicit result/warning metadata as appropriate

DomainError
  - structured categories for invalid/unsupported inputs
```

Do not over-generalize the model. A compact, strongly typed design is
preferred over a framework for every imaginable annotation format.

## Sequence identity and hashing

A sequence artifact must have stable identity independent of display name.

Use a documented canonical representation for hashing.

Recommended baseline:

- normalize only explicitly allowed representation differences
- hash the canonical sequence bytes with SHA-256
- do not hash arbitrary serialized structs whose field ordering may change

The code should make it clear what content the hash represents.

Do not pretend source metadata is part of sequence-content identity unless the
hash contract explicitly says so.

## Normalization

Define normalization at the input boundary.

Allowed normalization should be minimal and deterministic, for example:

- case normalization
- removal of formatting whitespace where the input contract permits it

Do not silently:

- delete unexpected biological characters
- invent bases
- repair malformed annotations
- alter topology
- infer strand
- infer CDS boundaries

Tests must establish normalization behavior.

## Origin-spanning circular features

Represent circular origin-spanning features explicitly rather than with
`start > end`.

For example, an ordered interval list can represent:

```text
[900, 1000), [0, 100)
```

on a 1000-base circular sequence.

The extraction algorithm must concatenate intervals in biological order and
then apply strand semantics.

Test both forward- and reverse-strand behavior where the representation
supports it.

## Translation requirements

For a supported complete CDS:

1. Extract nucleotide sequence in biological orientation.
2. Verify the sequence length is divisible by three.
3. Translate codons under the supported genetic code.
4. Handle the terminal stop according to the documented rule.
5. Do not treat the stop codon as an amino-acid residue.
6. Produce deterministic output.

Do not guess through ambiguous codons unless ambiguity semantics are
explicitly implemented and tested.

An internal stop should not be conflated with malformed input.

## Fixtures

Create or update `tests/fixtures/README.md` and the fixture files needed for
the first milestone.

The fixture documentation must record, for every shared scientific fixture:

- input sequence and annotations
- requested/derived operation relevant to the fixture
- expected output or expected error
- how the expectation was independently verified
- scope and known limitations
- whether the fixture is safe for cloud-assisted review

At minimum include fixtures covering:

1. simple linear forward-strand CDS
2. reverse-strand CDS
3. circular sequence
4. origin-spanning feature/CDS
5. invalid coordinate bounds
6. coordinate referencing the wrong sequence
7. incomplete CDS
8. CDS length not divisible by three
9. valid terminal stop
10. internal stop
11. invalid sequence characters
12. ambiguous bases, if accepted by the sequence model
13. unsupported annotation/feature structure

Do not create expected outputs by running the new implementation and then
copying its output into fixtures without independent verification.

Small unit-test sequences may remain inline where that is clearer.

## Testing requirements

Tests are part of the implementation, not follow-up work.

Cover at least:

### Sequence

- construction of valid sequence artifacts
- normalization behavior
- invalid characters
- empty sequence behavior
- length reporting
- topology
- stable content hash

### Coordinates

- valid ranges
- empty intervals where supported or explicit rejection
- negative/out-of-range equivalents in Rust representation
- end < start rejection
- wrong reference sequence rejection
- exact-end boundary behavior
- half-open semantics

### Strand / extraction

- forward extraction
- reverse-complement extraction
- multi-interval extraction
- origin-spanning circular extraction
- invalid interval/reference combinations

### CDS / translation

- known-answer translation
- reverse-strand translation
- multi-interval CDS translation where supported
- non-triplet CDS rejection
- missing/incomplete CDS rejection
- terminal stop handling
- internal stop handling
- unsupported genetic code rejection
- ambiguous-codon handling according to the chosen contract

### Errors

Errors should be structured enough for callers to distinguish malformed
input from unsupported features and biological validation failures.

Do not rely on string matching alone for core error classification.

## Implementation style

Prefer:

- small focused types
- explicit invariants
- constructors/validation at boundaries
- immutable or otherwise tightly controlled domain state where practical
- deterministic pure functions for extraction and translation
- readable error types
- tests close to behavior

Avoid:

- global mutable state
- generic "utility" modules that hide biological semantics
- excessive trait abstraction
- serialization formats introduced only for future use
- dependencies that solve problems not required by this milestone

## Public API expectations

The core crate should expose a small, coherent API that later workflows can
build on without knowing internal implementation details.

At minimum, callers should be able to:

- create/validate a sequence artifact
- construct/validate coordinates against a sequence
- construct features/CDSs
- extract a feature/CDS sequence
- translate a supported CDS
- receive structured errors/results

The API does not need to be final. It should be semantically sound and easy to
evolve without duplicating biological rules.

## Acceptance criteria

The task is complete only when all of the following are true:

- `crates/core` exists as the canonical owner of first-milestone biology.
- Sequence topology is explicit.
- Strand is explicit.
- Internal coordinates are zero-based half-open.
- Coordinates are tied to a reference sequence.
- Sequence normalization behavior is documented and tested.
- Sequence artifacts have stable content identity/hash.
- Supported CDS extraction is validated.
- Translation is deterministic and uses the explicitly supported genetic code.
- Terminal stop handling is correct and tested.
- Invalid and unsupported inputs produce structured errors.
- Circular/origin-spanning behavior is explicit and tested.
- Reverse-strand behavior is explicit and tested.
- Shared fixtures exist with independently sourced expectations.
- Unit/integration tests cover ordinary and pathological cases.
- No LLM, Python worker, primer, UI, database, or speculative orchestration
  code was added.
- The implementation passes the repository's required checks.

## Required checks

Run these after implementation:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report the exact command results.

Do not claim a check passed unless it was actually run successfully.

## Handoff report

At the end, report:

1. Changed files.
2. Exact checks run and their results.
3. The final concrete domain-model choices made for any previously open
   decisions.
4. Assumptions introduced.
5. Unsupported or unverified behavior that remains.
6. Any deviation from this task contract and why.

Do not stage, commit, amend, reset, rebase, merge, or push.

## Stop conditions

Stop implementation and report back instead of silently deciding when you
encounter:

- ambiguity in biological semantics
- conflicting repository requirements
- a need to change `docs/decisions.md` materially
- a need for mutation/primer/LLM behavior to complete this milestone
- a fixture whose expected result cannot be independently verified
- a representation choice that would impose a major future semantic
  commitment not covered by this task

A small implementation choice within the stated contract is fine; a new
biological assumption is not.
