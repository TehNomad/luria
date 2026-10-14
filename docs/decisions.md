# Decisions

Status: Proposed baseline; review before implementation
Date: 2026-10-05

Changes to these decisions require explicit human approval. Task-specific
implementation details belong in `docs/tasks/`.

## D001: Pi is development tooling

Decision:

Pi assists with planning, implementation, and review.

The shipped application does not require Pi.

Reason:

Development tooling should not constrain the application's interface,
deployment, or scientific API.

## D002: Rust owns canonical domain semantics

Decision:

Rust owns canonical biological domain semantics, including:

* sequence representation
* topology
* coordinates
* strand
* features
* CDS definitions
* mutation intent
* validation
* operation state
* application orchestration

Python workers may perform selected scientific computations, but they consume
explicit typed requests and return structured results.

Python workers do not independently redefine sequence, coordinate, strand,
mutation, or numbering conventions.

Scientific computation may use Python libraries where they provide meaningful
value. Ownership of each computation must be documented.

## D003: Separate domain, workflow, and presentation layers

Decision:

Luria is organized conceptually into three layers:

1. **Domain**
   Canonical biological data structures and deterministic semantics.

2. **Workflow**
   Operation-specific algorithms, validation, candidate generation, and
   verification.

3. **Presentation**
   CLI, TUI, web interface, machine-readable output, and natural-language
   interpretation.

Presentation layers must call the same typed application capabilities.

They must not duplicate biological calculations or establish their own
coordinate or numbering semantics.

## D004: Natural language becomes a proposed typed request

Decision:

The LLM converts user language into a structured proposed operation.

The application validates that operation before execution.

Missing, ambiguous, contradictory, or unsupported fields remain explicit.

The LLM does not provide authoritative:

* sequence calculations
* coordinates
* edited sequences
* primer sequences
* calculated thermodynamic values
* validation claims
* experimental conclusions

The interpreted request must be available for user inspection.

Removing the LLM from a workflow must not change the scientific computation.

## D005: Fail closed on ambiguity

Decision:

Luria must prefer clarification or structured failure over silent inference.

Examples include:

* multiple possible target CDSs
* unspecified numbering origin
* conflicting sequence annotations
* ambiguous strand
* unsupported coordinate format
* incomplete or unsupported CDS
* mutation whose expected original residue does not match
* unsupported genetic code
* unsupported topology or feature structure

The application must not silently select a plausible interpretation merely
because one interpretation is common.

Execution is permitted only when all required inputs and assumptions are
explicit and valid.

## D006: Coordinate conventions

Decision:

Internal nucleotide coordinates are zero-based and half-open:

`[start, end)`

External formats retain their own conventions at the boundary.

Import and export perform explicit, tested conversions.

A coordinate must identify its reference sequence.

A residue position must identify its CDS and numbering origin.

Raw integers must not be treated as interchangeable between nucleotide,
codon, amino-acid, and user-facing numbering systems.

Coordinate conversion functions must be deterministic and independently
tested.

## D007: Sequence artifacts have explicit identity

Decision:

A sequence is an explicit artifact, not merely an in-memory string.

A sequence artifact records sufficient identity and provenance to distinguish
one input from another.

At minimum:

* artifact identifier
* normalized sequence representation
* alphabet
* topology
* length
* source metadata when available
* annotation source or import metadata when available
* content hash

The initial content hash should use SHA-256 over a documented canonical
representation.

Two sequence artifacts with different canonical content must not be treated
as interchangeable merely because they share a display name.

Operations reference sequence artifacts explicitly.

## D008: Sequence normalization is explicit

Decision:

Sequence normalization must occur at a documented boundary.

Normalization may include only operations that are defined by the input
contract, such as case normalization or line-ending removal.

Normalization must never silently:

* repair biological content
* delete unexpected symbols
* invent missing bases
* alter annotations
* change topology
* infer missing sequence

The normalized artifact and its source representation must remain
distinguishable.

## D009: Topology, intervals, and strand are explicit

Decision:

Topology is explicit:

* linear
* circular

Feature intervals use reference-sequence coordinates.

Strand determines biological orientation.

Origin-spanning features require an explicit representation.

A feature must not use `start > end` as an undocumented wraparound
convention.

Unsupported compound or spliced features must produce explicit errors.

Extraction order and reverse-strand behavior must be documented and tested.

## D010: Initial CDS and translation scope is deliberately narrow

Decision:

A CDS identifies:

* its reference sequence
* its intervals
* its strand
* its completeness
* its translation metadata

For the initial milestone, support only explicitly complete CDSs under an
explicitly supported genetic code.

The initial implementation should support the standard genetic code only.

Protein residue numbering begins at 1 for the first translated codon of the
selected CDS unless an explicit numbering mapping is supplied.

The terminal stop codon is not an amino-acid residue.

Unsupported partial CDSs, unsupported numbering schemes, and unsupported
translation rules produce explicit errors.

Ambiguous codons must not be silently converted into a guessed amino acid.
The implementation must either represent the ambiguity explicitly or return
a structured translation error according to the supported input contract.

Internal stop codons must have a distinct validation result from malformed
sequence input.

## D011: Mutation intent is explicit

Decision:

An amino-acid substitution identifies:

* a target CDS
* a residue position
* an expected original residue
* a replacement residue

The expected original residue must match the translated template before a
mutation is applied.

Separate variants and combined mutations are different operations.

The parser and typed request model must preserve that distinction.

A requested amino-acid substitution does not uniquely determine a DNA edit.

Replacement codon selection therefore uses an explicit policy and records the
chosen codon.

No codon-selection policy may be inferred silently.

## D012: Scientific operations are deterministic

Decision:

Every supported scientific operation must have a deterministic execution
path independent of natural-language interpretation.

Given the same:

* canonical input artifacts
* operation parameters
* algorithm version
* dependency versions where relevant

the operation must produce the same structured result, or a documented
deterministic failure.

Randomized algorithms require an explicit seed and provenance record.

## D013: Generation and verification are separate responsibilities

Decision:

Candidate generation and result verification are separate responsibilities.

The validator independently determines whether the requested edit and
resulting construct satisfy the operation's biological contract.

Verification must not merely repeat the generator's claims.

Shared low-level utilities are permitted, but the architecture must preserve
an independent verification step.

For mutation workflows, verification must include the intended edited
sequence and the resulting translation where applicable.

## D014: Q5 is an explicit supported workflow

Decision:

Q5 site-directed mutagenesis is implemented as a distinct workflow.

It is not a generic primer algorithm with a Q5 label.

The workflow defines:

* supported input scope
* mutation representation
* candidate-generation method
* primer orientation
* sequence constraints
* thermodynamic calculations
* rejection criteria
* warnings
* verification requirements

Each scientific design rule must identify its documented source.

Any comparison against external software is development validation, not proof
of experimental success.

Luria must not claim exact reproduction of another tool unless exact
reproduction is an explicit project goal.

Calculated annealing or thermodynamic values must identify the method used.

## D015: Deterministic vertical slice before natural language

Decision:

Before integrating an LLM, Luria must support at least one complete workflow
through deterministic code.

The reference vertical slice is:

1. load a known sequence artifact
2. identify a supported CDS
3. translate it
4. construct a typed mutation request
5. validate the expected residue
6. apply the edit
7. independently verify the edited sequence
8. return a structured result

The same workflow must later be callable through natural language without
changing the underlying scientific path.

## D016: Operations are explicit typed capabilities

Decision:

Runtime operations are explicit application capabilities.

An LLM may propose a typed capability invocation, but it must not generate
arbitrary shell commands or arbitrary program execution.

The application validates operation type, required inputs, and allowed
parameters before execution.

Workers receive only the structured requests required for their declared
operation.

## D017: Python worker boundary

Decision:

The initial Python worker boundary is a subprocess with one versioned JSON
request and one versioned JSON response.

Requests and responses include:

* schema version
* operation identifier
* success or structured error
* provenance metadata as required by the operation

stdout is reserved for protocol output.

stderr is reserved for diagnostic logging.

Rust owns:

* worker lifecycle
* timeouts
* exit-code handling
* malformed-response handling
* operation state
* project state

Workers do not execute commands supplied directly by an LLM.

Defer persistent worker pools, network workers, queues, and distributed
execution until a concrete workload requires them.

Python should be introduced only when a real scientific operation benefits
from a Python dependency or implementation.

## D018: Fixtures are part of the scientific specification

Decision:

Shared scientific fixtures live in `tests/fixtures/`.

Fixture documentation records:

* input sequence and annotations
* requested operation
* expected output or error
* independent source of the expectation
* scope
* known limitations
* whether the fixture is approved for cloud-assisted review

Fixtures are created before or alongside implementation of the behavior they
specify.

Expected outputs must not be accepted solely because the implementation under
test generated them.

Reference results should, where possible, be checked against an independent
implementation, published example, hand-verified calculation, or other
documented source.

## D019: Test pathological cases explicitly

Decision:

The initial test corpus must include, at minimum:

* ordinary linear forward-strand CDS
* reverse-strand CDS
* circular sequence
* origin-spanning feature
* invalid coordinates
* coordinates referencing the wrong sequence
* incomplete CDS
* internal stop
* terminal stop
* ambiguous input
* wrong expected residue
* separate mutations
* combined mutation requests
* unsupported annotation forms

Edge cases are part of the contract rather than cleanup work after the happy
path.

## D020: Results have explicit lifecycle state

Decision:

An operation result must distinguish at least:

* `proposed`
* `needs_clarification`
* `validated`
* `calculated`
* `verified`
* `failed`

A result that is merely calculated must not be represented as verified.

Experimental validation is a separate status and is never inferred from
computational verification.

## D021: Result provenance is first-class

Decision:

Scientific results identify:

* input artifact identifiers and hashes
* operation type
* normalized operation parameters
* algorithm and algorithm version
* relevant dependency versions
* warnings
* assumptions
* computational validation status

LLM interpretation provenance is stored separately from scientific
calculation provenance.

The persistence format is deferred; the provenance requirements are not.

## D022: Reproducibility precedes persistence infrastructure

Decision:

Luria must define what must be reproducible before choosing a database or
persistence architecture.

The initial system may remain file- or process-oriented.

A database, account system, cloud synchronization layer, or distributed job
system must not be introduced merely to support hypothetical future
capabilities.

## D023: Privacy defaults are conservative

Decision:

Scientific operations should run locally where practical.

Cloud use must be explicit.

Private constructs, sequences, credentials, or experimental records must not
be sent to a cloud model by default.

Initial cloud-assisted development should use synthetic or approved public
fixtures.

Before cloud review, inspect task files, fixtures, source, generated logs,
and other context for sensitive scientific material.

## D024: Development and approval workflow

Decision:

Use one bounded task per task branch.

Agents do not stage, commit, rewrite history, merge, or push.

A human approves scope and creates commits.

Review actual changed files, including new files.

Run applicable tests and checks before acceptance.

Agent instructions are behavioral guidance, not a security sandbox.

## D025: Minimal architecture first

Decision:

Start with a Rust workspace and a core library.

Add application components only when a milestone needs them.

Defer:

* LLM framework selection
* database selection
* frontend framework selection
* MCP integration
* orchestration frameworks
* persistent worker infrastructure

Do not build placeholder infrastructure for hypothetical capabilities.

## D026: Validation commands

Decision:

Once the Rust workspace exists, run:

`cargo fmt --all -- --check`

`cargo test --workspace`

`cargo clippy --workspace --all-targets -- -D warnings`

Also inspect Git status and run:

`git diff --check`

Add Python formatting, linting, tests, and cross-language contract checks
when Python workers are introduced.

Report exact commands and actual outcomes.

Never report an unrun check as passed.

## D027: First milestone acceptance criteria

Decision:

The first milestone is complete only when:

* canonical sequence semantics are implemented
* supported topology and strand behavior are explicit
* supported CDS extraction is validated
* translation is deterministic
* ambiguous and unsupported cases fail explicitly
* sequence identity and hashing are defined
* fixtures cover ordinary and pathological cases
* known-answer and negative tests pass
* the deterministic vertical slice can run without an LLM
* provenance requirements are represented in the result model

The milestone is not complete merely because the Rust types compile.

## D028: A design names a policy, and a compatibility policy names a build

Decision:

Every primer design carries a `designPolicy`. The two policies are
`luriaNative` and `nebBaseChangerCompatible`.

Neither policy has a silent default. A request that does not name a policy has
not chosen one, and is an error.

`nebBaseChangerCompatible` is always accompanied by a `compatibilityTarget`
naming the build it was substantiated against. The initial target is
`NebaseChangerV2_11_0`. A target supplied alongside `luriaNative` is an error.

Matching numerical defaults is never described as compatibility. The word
*compatible* may appear in output, documentation and UI only for the mode,
qualified by the substantiation record.

Reason:

The reference tool's documented defaults — minimum primer length 15 nt and
minimum primer Tm 55 °C — are numbers a preset would set, and setting them
reproduces none of the reference tool's growth order, terminal-GC rule, balance
loops, revert-to-last-good step, availability limits or Tm engine. The same
quantity has different defaults at different call sites in the live build
(`primerMinLen` is 20 at the `Primer` layer and 15 at the `Fragment.autogen*`
layer; `monosalt`/`ct` are 50/200 in the `Primer` default object literal against
150/500 in the fallbacks). A preset names numbers; it does not name a site, and
it does not name a rule.

Naming a build rather than "the website" means a later build cannot silently
redefine what the policy promised.

D013 and D022 already make an un-chosen decision an error rather than a
fallback; this decision applies that rule to the design policy field.

Evidence:

* `tests/fixtures/nebasechanger-research/manifest.json` identifies the captured
  live build (`4330b0c7`, `NebaseChangerV2_11_0`) and records that the HAR's
  live asset `index.4d8eac54.js` is not in the Internet Archive
* the three default sets are reproduced by
  `tests/fixtures/nebasechanger-research/q6_v284_vs_live.py`

Consequences:

* `PrimerDesign` is not yet implemented, so these are contract additions and
  not API additions; no public API changed
* a consumer that reads only primer text still cannot mistake a compatibility
  design for a Luria design, because `policy`, `policyTarget` and
  `substantiation` are fields of the record

Reversibility:

A fragment/NEBuilder compatibility target would need a second target version or
a second policy; that is open question 11 of `docs/q5-primer-design.md` §20.

Reverting to an availability-aware split, or moving the extra base of an odd
`a_f` to the 3′ side, is a policy change under this file's header and needs
explicit human approval; neither is a routine implementation choice.

## D029: The Luria-native candidate geometry

Status: proposed, not yet approved. Amended in place on 2026-10-13 by the
split-policy pass; the availability-aware split this entry previously stated is
superseded and is recorded as such below.

Decision:

Under `luriaNative`, for an edit at template span `[s, e)` with replacement
length `L`:

* the pair abuts at a template boundary `s − u`, `u ≥ 0`
* the forward primer covers `[s − u, s + L + v)` and carries the replacement
  inline
* the reverse primer covers `[s − u − w, s − u)` and is the reverse complement
  of 5′ context
* the annealing totals are `a_f = u + v` and `a_r = w`, where `u` and `v` count
  the unchanged template-derived bases either side of the replacement region,
  so `a_f` excludes the replacement itself
* the split `(u, v)` is mutation-centered: `u = ceil(a_f / 2)`,
  `v = floor(a_f / 2)`, so `|u − v| ≤ 1` and the extra base of an odd `a_f`
  goes to the 5′ side. `(u, v)` is a function of `a_f` alone; availability is a
  feasibility test on the centered candidate, not a rule that moves the split.
  If the centered geometry cannot satisfy the policy's hard constraints, the
  policy reports that no design is available under it
* the bounds are the annealing bounds and availability; there is no maximum
  primer length and no Tm bound
* candidates are ordered by `a_f + a_r` ascending then `a_f` ascending, and the
  order is independent of sequence content
* an empty candidate set is a failure, reported with one of three named
  outcomes

Reason:

Primer lengths are `a_f + L` and `a_r`, so minimising `a_f + a_r` is minimising
total primer length; `L` is fixed by the edit. That is the objective, stated
rather than implied.

The split is centered because the centered geometry is the proposed native
default: the mutation region sits in the middle of the unchanged annealing
flanks, so a design's shape follows from the request and availability only
decides whether that shape fits. The version of this record written on
2026-10-12 made the split availability-aware, which let one request emit an
off-center primer on a template whose 3′ flank was exhausted; that rule is
superseded. What that pass found — that a fixed split can report no design
where a shifted one would fit — is now a **named outcome** of the centered
policy (§18.5 outcome 1), not a defect to be hidden by moving the split.

Centering is a Luria choice about the candidate geometry. It is not a
universal biological law, and it is not a compatibility claim. The saved help
text describes the reference tool's short-substitution path as anchoring the
center of the primer to the nucleotide region and extending in both the 5′ and
3′ directions with at least 10 fully complementary nucleotides on each side;
that is source evidence about that path. This record adopts neither its
per-side bound nor its growth order, and D028's rule that matching a numeric
default or adopting a similar geometry is not proof of compatibility applies
unchanged to this one.

The three failure outcomes are distinguished because collapsing them loses
information the caller needs: no centered candidate fits, the domain was capped
by a supplied `maxAnnealingOverlap`, and the request was rejected before
enumeration are different reasons, and the fixture `q5p-09` is the first of
them. Under the centered split the first two report the same absence (§18.3's
monotonicity), so what they distinguish is what the request supplied, not two
reasons in the geometry.

The abut rule, the absence of a maximum primer length and the absence of a Tm
bound are Luria choices recorded as such: the reference build documents a
minimum length and a minimum Tm and no maximum, and its back-to-back rule is
reference scope only.

Evidence:

* `crates/core/src/mutation.rs` — `NucleotideMutationIntent` carries one
  `replacement: char` at a one-nucleotide interval, which fixes `L = 1` for the
  native scope
* `crates/core/src/primer.rs` — `construct_primer`, `assemble_pair` and
  `require_design_constraints` already express the spans, the parts and the
  annealing flag that this geometry is built on
* `tests/fixtures/q5-primer/reference/designs.json` — `q5p-09` is an outcome-1
  case: bound 31 against a 30-base template, so no candidate fits under any rule,
  and the failure is not a property of one search
* `tests/fixtures/q5-primer/reference/designs.json` — `q5p-01`'s forward primer
  has annealing spans `[[1199, 1209], [1212, 1222]]` around an edit at
  `[1209, 1212)`, i.e. `u = v = 10`, `L = 3`, `a_f = 20`, primer length 23: its
  flanks are equal, so the reference's forward geometry is centered in the same
  direction as this rule. The difference is the total and the growth rule, not
  the shape — evidence about the reference design, not a compatibility claim
* `docs/research/nbc_help_v284.txt` line 200, quoted at
  `docs/research/nebasechanger-live-build-4330b0c7-2026-10-10.md` §15.4
  (`@19261`) — the reference's short-substitution path is described as center-
  anchored and extended in both directions with at least 10 complementary
  nucleotides per side. Evidence about that path, not about `luriaNative`
* `tests/fixtures/nebasechanger-research/replay_dispatcher_geometry.py` — the
  reference geometry, replayed, is *not* this geometry; the reference's forward
  5′ context is the pad and never shrinks

Consequences:

* under the centered split the initial native policy has a closed form: a
  design exists if and only if `s ≥ bound + ceil(bound / 2)` and
  `template.length − s − L ≥ floor(bound / 2)`, where `bound` is
  `minAnnealingOverlap`. Near a template end, an edit the superseded
  availability-aware rule would have designed now reports no design, which is
  the intended behaviour and must be reported as such
* `maxAnnealingOverlap` cannot make an infeasible edit feasible, because
  availability is monotone along the candidate order under a centered split
* the centered-geometry failure must stay distinguishable from an H1 bound
  failure and from an unsupported-placement refusal; `crates/core/src/error.rs`
  has no design-level failure carrier yet, so the carrier is an open item
* the native policy and the compatibility target are different geometries, and
  neither is adjusted to fit the other
* a native design at `a_f` equal to the bound satisfies H1 and would never be
  emitted by the reference, whose per-side wording implies `a_f ≥ 20`
* Tm is inert in the initial native policy, so the policy has a closed form and
  the enumeration is stated so that enabling Tm later extends the same order

