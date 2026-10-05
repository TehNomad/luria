Product brief

Status: Initial draft
Date: 2026-10-05
Working name: Molecular Biology Toolbox

Purpose

Build a standalone molecular-biology toolbox that combines reliable,
reproducible sequence-based tools with a natural-language interface.

Users should be able to describe an operation in ordinary language, inspect
how Luria interpreted it, resolve ambiguity when necessary, and receive
results calculated by validated software.

The long-term goal is a broad toolbox.

Development begins with one narrow, well-defined workflow and expands only
through validated capabilities.

Product thesis

Luria is not an autonomous molecular-biology agent.

Its core product promise is:

AI interprets intent; deterministic software performs the biology.

The system should remain useful without an LLM.

Natural language is an interface to the toolbox, not the source of scientific
truth.

Intended users
Molecular biologists and protein scientists working with constructs,
plasmids, coding sequences, and mutations.
Users who want a convenient interface without writing scripts.
Advanced users who want reproducible inputs, machine-readable outputs,
inspectable assumptions, and clear provenance.

The interface should support expert review without requiring users to
understand the internal software architecture.

Initial reference workflow

Given a DNA construct with a defined target CDS, interpret a request to design
primers for a series of independent amino-acid substitutions using the
supported Q5 site-directed mutagenesis workflow.

Example request:

"Using this plasmid, design Q5 SDM primers for A42G and W87A as separate
variants. Number residues from the beginning of the annotated CDS."

The application should:

Identify the input sequence artifact.
Identify the target CDS.
Interpret the requested mutations.
Preserve whether mutations are independent variants or a combined construct.
Validate the requested original residues against the translated CDS.
Identify missing or ambiguous information.
Require clarification before calculation when required information is missing.
Record the resulting DNA edits and replacement codons.
Generate candidates using the explicitly supported design method.
Independently verify the intended edited construct.
Present sequence context, primer orientation, calculated properties,
warnings, assumptions, and provenance.
Allow export of the reviewed result.

Combined multi-mutant constructs are a distinct request.

They must not be silently treated as independent variants or assumed to use the
same supported workflow.

Scientific trust model

The LLM interprets language and explains structured results.

It is not the source of truth for sequence calculations.

Deterministic software is responsible for:

sequence identity
coordinate conversion
strand handling
reverse complementation
CDS extraction
translation
mutation validation
sequence editing
primer calculations
supported workflow constraints
result verification

The application must distinguish:

requested operation
interpreted operation
unresolved ambiguity
calculated result
computational validation
experimental validation
warning
assumption

A calculated design must not be presented as experimentally validated unless
supporting experimental evidence has actually been supplied.

Fail-closed behavior

A central product behavior is knowing when not to act.

When Luria cannot establish a safe, supported interpretation, it should stop
and explain what needs to be resolved.

Example:

User:

"Mutate residue 42 to glycine."

Luria:

Status: needs_clarification
Reason: multiple supported CDSs could contain residue 42
No sequence calculation performed

The application should prefer an explicit clarification request over a
plausible guess.

This behavior is part of the product, not merely an error-handling detail.

Required inputs

Each operation declares its required inputs.

The initial Q5 workflow requires, at minimum:

the DNA template required by the supported workflow
an identified target CDS
a numbering convention
explicit mutation intent

A protein sequence alone is insufficient for the DNA-editing workflow because
it does not establish the actual template DNA sequence.

Missing or contradictory information must produce a clarification state or
structured error rather than an invented result.

Sequence artifacts

A sequence is treated as a versioned scientific artifact.

The artifact model records enough information to identify exactly which
sequence was used.

The initial implementation should capture:

canonical sequence content
sequence hash
topology
alphabet
length
annotation metadata
source/import metadata where available

Display names are not sufficient identity.

Results must reference the actual sequence artifact used for computation.

Architecture direction

Luria is organized around three conceptual layers:

Domain

Rust owns:

canonical sequence representation
coordinates
topology
strand
features
CDS definitions
mutation intent
validation primitives
artifact identity
Workflow

Operations own:

domain validation
scientific algorithms
candidate generation
independent verification
structured warnings and assumptions

A workflow may use a Python worker when a concrete scientific computation
benefits from Python libraries.

Presentation

The CLI, TUI, web interface, machine-readable exports, and natural-language
interface all call the same application capabilities.

No presentation layer owns its own scientific calculations.

AI architecture

Natural language becomes a proposed typed request.

The LLM is permitted to:

identify an intended operation
extract explicit parameters
identify candidate entities
explain structured results
formulate clarification questions

The LLM is not permitted to become the scientific execution engine.

The same typed operation must be callable without an LLM.

The removal of the LLM must not alter the scientific calculation.

Deterministic vertical slice

Before LLM integration, Luria should be able to execute one complete workflow
without natural language.

Reference path:

DNA artifact
→ validated CDS
→ translation
→ typed mutation request
→ mutation validation
→ deterministic edit
→ independent verification
→ structured result

This is the foundational product slice.

The natural-language interface is added afterward as another way to construct
the same typed request.

Initial implementation milestone

Implement the sequence, feature, CDS, coordinate, and artifact foundation.

Required outcomes:

explicit linear and circular topology
explicit strand and coordinate semantics
explicit sequence identity
documented normalization rules
validated extraction of supported CDS representations
translation under explicitly supported genetic-code rules
deterministic behavior
structured errors for invalid or unsupported input
known-answer tests
negative tests
pathological edge-case fixtures
a minimal deterministic mutation/edit path where practical

This milestone does not include:

LLM integration
Python workers
primer generation
persistence infrastructure
TUI
web interface
Fixture-first development

Scientific fixtures are part of the specification.

The initial corpus should include ordinary and pathological examples,
including:

forward-strand CDS
reverse-strand CDS
circular sequences
origin-spanning features
invalid coordinates
wrong reference sequence
incomplete CDS
internal stops
terminal stops
ambiguous input
incorrect expected residues
separate mutation requests
combined mutation requests
unsupported annotations

Expected results should come from independent references, published
examples, hand-verified calculations, or other documented sources.

The implementation must not define correctness by generating its own expected
fixtures.

Subsequent milestones
Milestone 2: Mutation semantics

Implement explicit mutation intent, residue validation, deterministic sequence
editing, and independent verification.

Milestone 3: Supported Q5 workflow

Implement the narrow Q5 workflow with documented design rules, candidate
generation, warnings, and independent verification.

Milestone 4: Machine-readable and CLI workflow

Provide a minimal CLI and structured output so complete scientific workflows
can be exercised without a TUI, web frontend, or LLM.

Milestone 5: Natural-language interpretation

Add an LLM adapter that converts user requests into typed operations and
clarification requests.

Milestone 6: Review-oriented user interface

Build the TUI around the same application capabilities and typed operations.

Milestone 7: Web interface

Expose the same capabilities through a web interface without duplicating
scientific logic.

Reorder these milestones only through an explicit scope decision.

Q5 workflow boundary

Q5 is an explicit supported workflow.

Luria should document exactly what it supports and what it does not.

The workflow must identify:

accepted input scope
mutation semantics
primer-generation method
sequence constraints
thermodynamic calculation method
warnings
candidate rejection criteria
independent verification

Comparison with external tools is development validation, not proof of
experimental success.

A design result should clearly distinguish calculated properties from
experimentally observed outcomes.

Result lifecycle

Results should expose meaningful state, including:

proposed
needs_clarification
validated
calculated
verified
failed

Computational verification is not experimental validation.

The interface should make these distinctions visible rather than burying them
in logs.

Provenance and reproducibility

A result should record enough information to reproduce the calculation.

At minimum:

input artifact identifiers
input hashes
normalized operation parameters
operation type
algorithm version
relevant dependency versions
warnings
assumptions
computational validation status

LLM interpretation provenance is separate from scientific calculation
provenance.

Persistence technology is deferred until the product requires it.

The provenance model is not deferred.

Privacy

Scientific operations should run locally where practical.

Cloud use must be explicit.

Private constructs, sequences, credentials, or experimental records must not
be sent to a cloud model by default.

Initial cloud-assisted development should use synthetic or approved public
examples.

Non-goals

Luria is not:

an autonomous wet-lab scientist
a generic chat interface that emits unverified sequence outputs
a system that guarantees experimental success
an automatic oligo-ordering service
an external-action execution agent
a Pi runtime dependency
a distributed job platform at launch
a multi-agent orchestration platform at launch
Definition of success

The first complete scientific workflow succeeds when:

the operation can be executed without an LLM
the interpreted request is visible and reviewable
invalid and ambiguous requests fail clearly
the exact input sequence artifact is identifiable
the intended edit is verified independently
known-answer and negative tests pass
pathological fixtures are covered
assumptions and warnings are explicit
computational and experimental validation are clearly distinguished
provenance is sufficient to reproduce the calculation
the reviewed result can be exported

The first AI milestone succeeds when natural language can produce the same
typed operation and deterministic scientific result without changing the
underlying calculation.

Long-term direction

The architecture should support additional validated capabilities such as:

sequence conversion
translation
construct analysis
restriction analysis
primer design
assembly planning
other sequence-based molecular-biology operations

These remain roadmap candidates.

Each new capability should enter the product as an explicit typed workflow
with tests, fixtures, provenance, validation, and a defined supported scope.

The goal is not to make Luria capable of everything.

The goal is to make every capability it does support trustworthy.
