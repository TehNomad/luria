# Decisions

Status: Proposed baseline; review before implementation
Date: 2026-10-02

Changes to these decisions require explicit human approval.
Task-specific implementation details belong in docs/tasks/.

## D001: Pi is development tooling

Decision:
Pi assists with planning, implementation, and review.
The shipped application does not require Pi.

Reason:
Development tooling should not constrain the application's interface,
deployment, or scientific API.

## D002: Rust owns canonical domain semantics

Decision:
Rust owns canonical sequence representation, coordinates, features,
CDS definitions, mutation intent, validation, and application state.

Python workers consume explicit requests and return structured results.
They do not independently redefine coordinate or mutation conventions.

Scientific computations may use Python libraries, but ownership of each
operation must be documented to avoid divergent implementations.

## D003: Natural language becomes a proposed typed request

Decision:
The LLM converts user language into a structured proposed operation.

The application validates that operation before execution.
Missing, ambiguous, or unsupported fields remain explicit.

The LLM does not supply authoritative primer sequences, calculated
temperatures, edited sequences, or validation claims.

An interpreted request must be available for user inspection.

## D004: Coordinate conventions

Decision:
Internal nucleotide coordinates are zero-based and half-open:
[start, end).

External formats retain their own conventions at the boundary.
Import and export perform explicit, tested conversions.

A coordinate must identify its reference sequence.
A residue position must identify its CDS and numbering origin.

Raw integers must not be treated as interchangeable between nucleotide,
codon, amino-acid, and user-facing numbering systems.

## D005: Circular features and strand

Decision:
Topology is explicit: linear or circular.

Feature intervals use reference-sequence coordinates.
Strand determines biological orientation.

Origin-spanning features must have an explicit representation.
Their extraction order and reverse-strand behavior must be documented
and tested.

Open:
Choose the concrete Rust representation before implementing it.
A validated ordered collection of intervals is a candidate.

Do not use start > end as an undocumented wraparound convention.
Do not silently accept unsupported compound or spliced features.

## D006: CDS and translation

Decision:
A CDS identifies its reference sequence, intervals, strand, and supported
translation metadata.

For the initial milestone, accept only explicitly complete CDSs under
an explicitly declared supported genetic code.

Protein residue numbering begins at 1 for the first translated codon
of the selected CDS unless an explicit numbering mapping is provided.

Unsupported partial CDSs, alternative numbering, or translation rules
must produce explicit errors rather than inferred corrections.

The terminal stop codon must not be treated as an amino-acid residue.
Malformed inputs and unsupported cases must have separate error paths.

Open:
Define the supported genetic codes and the behavior of ambiguous codons
in the first task contract.

## D007: Mutation intent is explicit

Decision:
An amino-acid substitution identifies a CDS, position, expected original
residue, and replacement residue.

The expected original residue must match the translated template.

Separate variants and combined mutations are different operations.
The parser must preserve this distinction.

Replacement codon selection uses an explicit policy and records the
chosen codon.

Open:
Choose the initial codon-selection policy before implementing primer
design. Do not silently equate a requested amino-acid substitution with
a unique DNA change.

## D008: Q5 is an explicit supported workflow

Decision:
Q5 design is implemented as a distinct workflow, not as a generic primer
algorithm with a different label.

Design rules must have documented sources and a supported scope.

Comparison with NEBaseChanger is part of development validation where
applicable, not proof of experimental success.

Identical primer sequences are not required unless exact reproduction
is an explicit goal; multiple valid candidates may exist.

Calculated annealing recommendations must identify their method.
Do not label an independently calculated value as a NEBaseChanger result.

## D009: Generation and verification are separate

Decision:
Candidate generation and result verification are separate responsibilities.

The validator checks the intended edited sequence and translation, not
only candidate oligo properties.

Shared utilities are acceptable, but verification must not merely repeat
the generator's claim that an edit is correct.

## D010: Python worker boundary

Decision:
Start with a subprocess that accepts one JSON request and returns one
JSON response.

Requests and responses include an explicit schema version.
Successful results and structured errors are distinguishable.

Reserve stdout for protocol output and stderr for diagnostic logging.
Define timeout, exit-code, and malformed-response behavior before use.

Rust owns job orchestration and project state.
Workers do not execute arbitrary commands supplied by an LLM.

Defer persistent pools, network workers, and job queues until needed.

## D011: Tests and shared fixtures

Decision:
Shared scientific test data lives in tests/fixtures/.
Rust integration-test code lives in each crate's tests/ directory.
Python test code lives with the worker package when added.

Fixture documentation records:

- Input sequence and annotations.
- Requested operation.
- Expected output or error.
- How the expectation was verified.
- Scope and known limitations.
- Whether the data is safe to share with cloud services.

Expected outputs must not be accepted solely because the implementation
under test generated them.

Small unit tests may use inline data.
Cross-language contract tests must be added when workers exist.

## D012: Cloud data handling

Decision:
Use synthetic or approved public examples for initial cloud-assisted work.

Cloud planning and review do not imply permission to disclose private
sequence data.

Before cloud use, inspect task files, source code, fixtures, and logs for
sensitive material. Remove private fixtures from the cloud review context.

## D013: Development and approval workflow

Decision:
Cloud models plan and review.
Local Qwen performs most implementation.
A human approves scope and creates every commit.

Use one bounded task per task branch.
Agents do not stage, commit, rewrite history, merge, or push.

Review the actual changed files and staged diff, including new files.
Run applicable tests and checks before acceptance.

Agent instructions are behavioral guidance, not a security sandbox.

## D014: Minimal architecture first

Decision:
Start with a Rust workspace and a core library.
Add application, frontend, and worker components when a milestone needs
them.

Defer selection of the LLM framework, database, frontend frameworks,
MCP integration, and orchestration system.

Do not build placeholder infrastructure for hypothetical capabilities.

## D015: Validation commands

Decision:
Once the Rust workspace exists, run:

cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings

Also inspect Git status and run git diff --check.

Add Python formatting, linting, test, and cross-language checks when the
Python worker is introduced.

Report commands and actual outcomes. Never report unrun checks as passed.

## D016: Result provenance

Decision:
Scientific results identify the input sequence, operation parameters,
algorithm version, relevant dependency versions, warnings, and explicit
assumptions.

LLM interpretation provenance is separate from scientific calculation
provenance.

The persistence format is deferred; the provenance requirements are not.
