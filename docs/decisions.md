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

**Amended by D030 (2026-10-14):** primer lengths are now set by Tm; D029's fixed, centred lengths no longer apply. Its scope, failure outcomes and abutting pair stand.

Status: **approved** by the project owner on 2026-10-14, as amended below.
Amended in place on 2026-10-13 by the split-policy pass (the
availability-aware split is superseded), and on 2026-10-14 by four owner
decisions: a minimum of 10 unchanged bases on each side of the replacement,
the extra base of an odd split on the 3′ side, a scope of any contiguous
replacement region of at most 3 bases, and a reverse primer of at least 15
bases. The owner's 2026-10-14 answers on failure reporting and on
`maxAnnealingOverlap` are applied as well.

Decision:

Under `luriaNative`, for an edit whose replacement region is the contiguous
template span `[s, e)`, `L = e − s`, `1 ≤ L ≤ 3`:

* the pair abuts at a template boundary `s − u`
* the forward primer covers `[s − u, s + L + v)` and carries the replacement
  inline
* the reverse primer covers `[s − u − w, s − u)` and is the reverse complement
  of 5′ context
* the annealing totals are `a_f = u + v` and `a_r = w`, where `u` and `v` count
  the unchanged template-derived bases either side of the replacement region,
  so `a_f` excludes the replacement itself
* each forward flank has at least `MIN_FLANK = 10` unchanged bases, so
  `a_f ≥ max(minAnnealingOverlap, 20)`
* the reverse primer has at least `MIN_REVERSE = 15` bases, so
  `a_r ≥ max(minAnnealingOverlap, 15)`
* the split `(u, v)` is mutation-centered: `u = floor(a_f / 2)`,
  `v = ceil(a_f / 2)`, so `|u − v| ≤ 1` and the extra base of an odd `a_f`
  goes to the 3′ side. `(u, v)` is a function of `a_f` alone; availability is a
  feasibility test on the centered candidate, not a rule that moves the split
* the bounds are the annealing bounds, the per-side minimum, and availability;
  there is no maximum primer length, no `maxAnnealingOverlap`, and no Tm bound
* candidates are ordered by `a_f + a_r` ascending then `a_f` ascending, and the
  order is independent of sequence content
* when no candidate fits, the policy emits no pair and reports one of the named
  outcomes below

Failure outcomes:

* **no design possible** — the template is shorter than the smallest pair,
  `L + max(minAnnealingOverlap, 20) + max(minAnnealingOverlap, 15) >
  template.length`,
  so no placement of any abutting pair fits. `BiologicalValidation`.
  `q5p-09` (bound 31, 30-base template) is this outcome
* **no centered design fits** — a pair of that size fits the template, but not
  centered on this edit. The report names the binding side, the bases available
  on it, and the flank asked for. `Unsupported`: a different geometry might fit,
  and this policy does not shift
* **not searched** — the request is refused before enumeration:
  `minAnnealingOverlap < 1` is `MalformedInput`; a replacement region longer
  than 3, a non-contiguous changed set, a circular template, or a composed
  variant request is `Unsupported`

The error carriers for the first two outcomes are `NoPrimerDesignPossible` and
`NoPrimerDesignFits` (`crates/core/src/primer_design.rs`, 2026-10-14).

Reason:

Primer lengths are `a_f + L` and `a_r`, so minimising `a_f + a_r` is minimising
total primer length; `L` is fixed by the edit. That is the objective, stated
rather than implied.

The per-side minimum exists because the annealing total alone allowed primers
the reference tool's documented rule would never produce: at a bound of 15 the
forward primer's 3′ flank was 7 bases. That flank is what the polymerase
extends from, and with no Tm function nothing else rejects a weak primer. The
reference tool's manual states "at least 10 nucleotides that are 100%
complementary on both sides of the mutation region" for short substitutions;
the minimum adopts that number as a length floor. It adopts the number, not the
reference algorithm: there is no Tm growth, no G/C clamp, and no Tm balance.
H1 is unchanged and remains a bound on the annealing total (the 2026-10-12
pass's A9); the per-side minimum is a rule of this policy, not a contract
requirement of every policy.

The extra base of an odd split goes 3′ because the 3′ flank primes extension,
and because the 3′ flank is the side the reference tool grows.

The split is centered because the centered geometry is the proposed native
default: the shape follows from the request, and availability only decides
whether that shape fits. The 2026-10-12 availability-aware split, which let one
request emit an off-center primer on a template whose 3′ flank was exhausted,
stays superseded.

Centering is a Luria choice. It is not a universal biological law and it is
not a compatibility claim. The reference tool's code pins the forward primer's
5′ flank at its pad of 10 and grows only the 3′ flank for Tm; D028's rule that
a similar geometry is not proof of compatibility applies unchanged.

The scope covers codon edits because a residue substitution, the most common
SDM request, claims a 3-base region; the centered definition applies unchanged
to any contiguous region.

Evidence:

* `crates/core/src/mutation.rs` — a nucleotide substitution claims one base and
  a residue substitution claims its codon's three, so `1 ≤ L ≤ 3` covers every
  edit the mutation layer produces on a linear template
* `crates/core/src/primer.rs` — `construct_primer`, `assemble_pair` and
  `require_design_constraints` already express the spans, the parts and the
  annealing flag this geometry is built on
* `docs/research/nbc_help_v284.txt` line 200, quoted at
  `docs/research/nebasechanger-live-build-4330b0c7-2026-10-10.md` §15.4
  (`@19261`) — the source of the 10-bases-per-side number
* `docs/research/q5-primer-policy-review-2026-10-12.md` C8, and
  `tests/fixtures/nebasechanger-research/replay_dispatcher_geometry.py` (on
  the `research/nebasechanger_reverse_engineer` branch) — the reference's 5′
  flank is its pad and never moves; its 3′ flank is seeded at 10 and grown
  for Tm; its minimum-length setting applies to the reverse primer only
* `tests/fixtures/q5-primer/reference/designs.json` — every reference design
  has forward flanks of exactly 10 and 10, because the Tm target was already
  met at the seed. No saved design distinguishes a centered rule from a pinned
  5′ flank. Under this policy the forward primer covers the same span as the
  reference forward primer for the four designs at bound 15 (`q5p-01`,
  `q5p-04`, both `q5p-07` variants). For `q5p-02` and `q5p-03` (bounds 24, 25)
  this policy uses `a_f` of 24 and 25, where the reference keeps 20 and fails
  H1. Reverse primers differ throughout (this policy: the bound; the
  reference: grown for Tm). An observation, not a compatibility claim

Consequences:

* with Tm inert the policy has a closed form. With `b = minAnnealingOverlap`
  `A = max(b, 20)` and `R = max(b, 15)`, the one candidate is `a_f = A`,
  `a_r = R`, `u = floor(A/2)`, `v = ceil(A/2)`, and a design exists if and
  only if `s ≥ R + floor(A/2)` and `template.length − s − L ≥ ceil(A/2)`. At
  `b = 15` that is `s ≥ 25` and at least 10 bases after the replacement
* the shortest forward primer is `L + 20` and the shortest reverse primer is
  15. The reverse floor matches the reference tool's minimum-length clamp
  (`minLen < 15 ? 15 : minLen`), adopted as a length floor only
* availability is monotone along the candidate order, so the first candidate
  is the only one that can fit
* the centered-geometry failure stays distinguishable from an H1 bound failure
  and from an unsupported-placement refusal
* the native policy and the compatibility target are different geometries, and
  neither is adjusted to fit the other
* Tm is inert in the initial native policy; the enumeration is stated so that
  enabling Tm later extends the same order

Reversibility:

Changing the per-side minimum, the reverse-primer minimum, the side of the
extra base, or the scope is a policy change and needs approval. Removing the per-side minimum would again
allow primers below the reference tool's documented floor.

## D030: Tm-driven primer length for `luriaNative`

Status: approved by the owner, 2026-10-14. Amends D029: D029's scope, its
failure outcomes and its abutting pair stand; its fixed, centred lengths do
not. Amended by the owner, 2026-10-18: the annealing temperature is the lower
Tm rounded to a whole degree, plus 1 °C (was the lower Tm plus 3 °C).

Decision:

A `luriaNative` design sizes each primer by melting temperature. For a
replacement region `[s, e)` on a linear template of length `n`, with bound
`b = minAnnealingOverlap`:

* **Junction.** The primers abut at `B = s − 10`. The junction never moves:
  the forward primer keeps exactly 10 unchanged bases on its 5′ side, and both
  primers grow only at their 3′ ends.
* **Starting lengths.** The forward primer's 3′ flank starts at
  `v = max(10, b − 10)`, so its annealing overlap `10 + v` meets the bound
  (H1). The reverse primer starts at `r = max(b, 15)`.
* **Tm floor, 55 °C.** Each primer grows one base at a time until its Tm is at
  least 55 °C. A primer that runs out of template first is a failure of the
  "no design fits" kind, naming the side and the Tm reached.
* **G/C 3′ end.** After the floor, a primer keeps growing toward a G or C last
  base while its Tm is below 63 °C; if it does not reach one it returns to the
  shortest length that met the floor. A preference, not a requirement.
* **Order.** Forward first, then reverse.
* **Balance, 5 °C.** If the two Tms differ by more than 5 °C, the colder primer
  grows until they are within 5 °C, then toward a G/C last base while its Tm
  stays at or below the other's, returning to the balanced length otherwise. A
  preference: if template runs out first, the design is emitted and records
  that balance was not met.
* **Tm basis.** The forward primer uses the mismatch-aware Tm against the
  wild-type site where published parameters cover the mismatch pattern, and
  the matched Tm where they do not; the reverse primer, which carries no
  mismatch, uses the matched Tm. Each Tm records its basis.
* **Buffer.** Named by the caller in the request, never defaulted.
* **Annealing temperature.** Reported as the lower of the two Tms, rounded
  half up to a whole degree, plus 1 °C, capped at 72 °C, using the same Tms
  the sizing used. It is a whole number of degrees because a thermocycler
  is set to that precision (owner, 2026-10-18).
* **Ambiguity codes.** An ambiguity code in the template ends availability on
  that side: a primer cannot grow across it.

Sources:

* 55 °C floor: owner, 2026-10-14, as the practical minimum Tm for a PCR primer;
  also the reference tool's documented minimum.
* 5 °C balance: NEB, *Guidelines for PCR Optimization with Thermophilic DNA
  Polymerases*: "Primer pairs should have Tms within 5 °C of each other."
* Annealing temperature: the NEB Tm calculator's changelog, v1.9.1, 27 July
  2016, as quoted by the owner on 2026-10-18: "The calculations in the NEB Tm
  calculator were modified to correct an error that was decreasing the
  effective primer concentration to one quarter of its input value. This
  correction raises Tm values by roughly 2°C for all polymerase reaction
  buffers. The suggested value for annealing temperature using Q5 is now
  Tm + 1 (was Tm + 3) to account for the revised Tm values." luria's Tm uses
  the primer concentration undivided (`tm.rs`), which is the corrected form,
  so the +1 offset is the one that matches it. NEBaseChanger computes the
  same rule, `min(min(tmf, tmr) + 1, 72)`, and rounds it for display
  (`docs/research/nebasechanger-client-investigation.md`). Two NEB pages
  still give +3 — Q5 *PCR Optimization* (E0555), "3 °C above the Tm of the
  lower Tm primer", and the Q5 Site-Directed Mutagenesis Kit FAQ, "a Tm+3
  rule" — which D030 cited until 2026-10-18; the changelog indicates they
  describe the pre-2016 Tm. The planner could not open the calculator's page
  itself (HTTP 403), so the quotation rests on the owner's reading.
  NEBaseChanger's own SID-1 D404N result, captured in
  `tests/fixtures/example-construct/raw/sdmsummary.txt`, shows the same rule
  independently of the changelog: Tms of 74 and 70 °C and a Ta of 71 °C, the
  lower Tm plus 1 (plus 3 would give 73). The 72 °C cap is the top of the Q5
  SDM protocol's 50–72 °C annealing range.
* G/C 3′ end and the 63 °C ceiling on G/C extension: owner, 2026-10-14, as
  general PCR practice.
* Tm model: `crates/core/src/tm.rs` and `tests/fixtures/tm/README.md`.

Reason:

The fixed lengths of D029 emit primers that cannot work: at a bound of 15 the
`native-04` reverse primer is 15 bases and melts at 45.4 °C. Growing only at
3′ ends keeps the junction, and so the abutment, fixed while each primer is
sized, and puts any extra length on the side the polymerase extends from.

Deviation recorded:

NEB's Q5 SDM primer-design FAQ says "Substitutions are created by designing a
mismatch in the center of the mutagenic primer." Under this decision the
mismatch is centred only while the forward 3′ flank is 10 bases; a colder site
grows the 3′ flank and moves the mismatch toward the 5′ side. This matches
NEB's own design tool, whose forward primer keeps a fixed 10-base 5′ flank
(`docs/research/nebasechanger-live-build-4330b0c7-2026-10-10.md` §6.1), and was
chosen by the owner over centred growth on 2026-10-14.

Not adopted from the reference tool: its mismatch-aware Tm, which differs
from the published model by several degrees (`tests/fixtures/tm/README.md`).
Its 1 °C annealing offset, first not adopted, was adopted on 2026-10-18 with
the NEB calculator changelog as the source.

Reversibility:

Each threshold — 55, 63, 5, the 10-base 5′ flank, and the +1 annealing offset with its whole-degree rounding — is
a policy value and changing one needs approval. Centred growth was considered
and rejected; adopting it later would change every design with a 3′ flank
above 10.

## D031: Deletion primer geometry for `luriaNative`

Status: approved by the owner, 2026-10-15. Extends D030 to deletions; D030
is unchanged for substitutions.

Decision:

For a deletion of `[s, e)`, width `w = e − s`, on a linear template of length
`n`, with bound `b = minAnnealingOverlap`:

* **Geometry.** The forward primer's 5′ end is at `e` and it reads rightward,
  `[e, e + f)`. The reverse primer's 5′ end is at `s` and it reads leftward,
  `[s − r, s)`. Neither primer carries a non-template base.
* **Pair invariant.** Reverse end + `w` = forward start. In the product the
  two 5′ ends meet at `s`. Template abutment, the substitution rule, is the
  case `w = 0` (owner, 2026-10-14, `docs/tasks/milestone-4-indels.md` 4.3).
* **Starting lengths.** Both primers start at `max(b, 15)`.
* **Everything else is D030.** The 55 °C floor, the G/C 3′ end below 63 °C,
  forward first, the 5 °C balance, annealing temperature as the lower Tm
  rounded to a whole degree plus 1 °C, capped at 72 °C, ambiguity codes
  ending availability, the buffer named
  by the caller, and D030's two failure kinds. Both Tms are on the matched
  basis.
* **Scope.** Linear templates only. No cap on `w`: both primers only have to
  fit.

Sources:

* Geometry: the reference tool's help text, "For deletions, the 5′ ends of the
  primers align with the edges of the deletion"
  (`docs/research/nbc_help_v284.txt`).
* Starting length: owner, 2026-10-15. D030's forward start of 20 exists to put
  10 unchanged bases on each side of a mismatch. A deletion primer has no
  mismatch, so both primers use the reverse primer's existing floor.

Not adopted from the reference tool: its refusal when fewer than 62 bases
remain after a deletion
(`docs/research/nebasechanger-live-build-4330b0c7-2026-10-10.md` §17.8). The
number has no stated source.

Related owner decision, 2026-10-15: a deletion inside a CDS whose width is not
a multiple of 3 shifts the reading frame. luria **warns and still applies it**,
because a frameshift is sometimes the intent, as in a knockout. Implemented
2026-10-15 as `Q5EditWarning::Frameshift` (`crates/core/src/q5.rs`).

Reversibility:

The starting length and the absence of a width cap are policy values;
changing either needs approval.

## D032: Insertion representation and primer geometry for `luriaNative`

Status: approved by the owner, 2026-10-15. Builds on owner decisions 4.1,
4.2, 4.4 and 4.5 in `docs/tasks/milestone-4-indels.md`.

Decision:

* **Representation.** An `InsertionPoint` is one 0-based coordinate `p`,
  meaning "between base `p − 1` and base `p`". The inserted bases occupy
  `[p, p + k)` of the edited artifact. Inserted bases are `A`, `C`, `G`, `T`
  only; degenerate insertions are a later pass.
* **What an insertion asserts (D011).** The base on each side of the point:
  the request states the bases at `p − 1` and `p`, and a mismatch refuses the
  edit. So `1 ≤ p ≤ n − 1`: an insertion needs a base on both sides.
* **Verification.** The edited artifact has exactly `n + k` bases and is
  `source[..p] + inserted + source[p..]`.
* **Geometry**, on a linear template. Both primers' annealing portions abut at
  `p`: the forward primer anneals to `[p, p + f)`, the reverse to
  `[p − r, p)`.
  * `k ≤ 6`: the whole insertion is the forward primer's 5′ tail.
  * `k > 6`: the forward tail is the last `ceil(k/2)` inserted bases; the
    reverse tail is the reverse complement of the first `floor(k/2)`.
* **Starting lengths.** Both annealing portions start at `max(b, 15)`.
* **Tm.** Sizing, balance and annealing temperature use each annealing
  portion's Tm on the matched basis. Each primer's whole-primer Tm is also
  reported, labelled.
* **Everything else is D030**: 55 °C floor, G/C end below 63 °C, forward
  first, 5 °C balance, ambiguity codes ending availability, the same failure
  kinds. The "confine to 5′ tails" option does not change an insertion.
* **No cap on insertion length.** Any primer longer than **100 nt** gets a
  quality warning, not a refusal.

Sources:

* Tail placement and the 6-nt threshold: the reference tool's help text,
  "If 6 or fewer nucleotides are to be inserted, they will be placed at the 5′
  end of one primer. If more than 6 … they will be split between the two
  primers to keep the primer lengths and Tm's similar"
  (`docs/research/nbc_help_v284.txt`).
* The split point `ceil(k/2)` forward: **luria-native**, owner, 2026-10-15.
  The reference tool's own split was never substantiated: a recorded run put an
  8-nt replacement wholly on the forward primer.
* Annealing-only minimum length: the same help text, "When modifications are
  constrained to the 5′ tails, the minimum primer length only applies to the
  annealing portion of the primer."
* 100 nt: owner, 2026-10-15, as the practical limit for ordering a standard
  oligo from IDT. A vendor limit, so a warning.

Related owner decision, 2026-10-15: an insertion of `k` bases inside a CDS
with `k` not a multiple of 3 shifts the reading frame. As for deletions,
luria **warns and still applies it**. "Inside" means both bases beside the
point lie in the same interval of the CDS (`start < p < end`). An insertion
exactly at a CDS's edge does not warn.

Reversibility:

The 6-nt threshold, the split point, the flanking assertion and the 100-nt
warning are policy values; changing one needs approval.

## D033: Unequal replacements and degenerate expansion for `luriaNative`

Status: approved by the owner, 2026-10-15. Generalises D029/D030's
substitution scope; D031 and D032 are unchanged.

Decision:

A replacement names a target region `[s, e)` of width `w >= 1` and a
replacement text of length `m >= 1`. `m` need not equal `w`, so one request
may carry several changes at once — that is how two codons are altered with a
single primer pair — and a replacement with `m != w` changes the artifact's
length. `w = 0` is an insertion (D032) and `m = 0` a deletion (D031); neither
is reached through this path.

* **Geometry.** With `t = max(w, m)`:
  * `t <= 6`: **inline**, exactly D030's geometry with the replacement region
    in place of the substituted bases. The junction stays at `s - 10`, the
    forward primer keeps its fixed 10-base 5′ flank, and its 3′ flank starts
    at `max(10, b - 10)`. `MAX_REPLACEMENT_REGION` rises from 3 to 6.
  * `t > 6`: **split across both 5′ tails**, exactly D032's rule with the
    replacement text in place of the inserted text — forward tail the last
    `ceil(m/2)` bases, reverse tail the reverse complement of the first
    `floor(m/2)`. Both annealing portions' 5′ ends sit at the edges of the
    target region, so the replaced bases are carried by the tails and
    annealed over by neither primer. Both start at `max(b, 15)`.
* **Sizing** is D030's throughout: the 55 °C floor, the G/C end below 63 °C,
  forward first, the 5 °C balance, and `Ta = min(72, round(lower Tm) + 1)`. On the
  tail path sizing uses the annealing portions only, with the whole-primer Tm
  reported, as D032 does.
* **Pair invariant**, generalising D031's: reverse end + `w` = forward start.
* **Tm basis.** On the inline path with `m == w` the forward primer uses
  D030's hybrid basis, unchanged. With `m != w` it uses the **matched** basis:
  the primer and the template site are different lengths, so there is no
  position-by-position alignment for a mismatch-aware Tm to read, and the
  model offers no other answer. This is forced by the thermodynamic model
  rather than chosen (planner, 2026-10-15). On the tail path both primers are
  pure template, so both are matched, as in D031 and D032.
* **Degenerate expansion.** The replacement may contain IUPAC ambiguity
  codes. Each expands over the bases it stands for, giving **one design per
  concrete sequence**. Each position's bases are taken in `A`, `C`, `G`, `T`
  order, and the **leftmost position varies fastest**, so `NS` gives
  `AC, CC, GC, TC, AG, CG, GG, TG` — the order the reference tool's help text
  lists, checked against it on 2026-10-15. (The planner first wrote this
  order backwards, as leftmost-slowest, which gives
  `AC, AG, CC, CG, …`; the quoted list settles it.) Expansion happens at the **request** layer, before any
  edit: each concrete sequence is its own mutation, independently verified,
  with its own pair. `mutation.rs` does not loosen — an applied edit still
  writes only `A`, `C`, `G`, `T`.
* **Expansion count.** No cap. Past **64** designs (three fully ambiguous
  positions) the request carries a warning and every design is still emitted.
* **Naming.** An expanded design's result name is the caller's base name, a
  hyphen, and its concrete replacement sequence: `mut-1-AC`, `mut-1-CC`, and
  so on. Names are distinct by construction, which satisfies the variant
  set's existing duplicate-name rule.

Sources:

* The 6-nucleotide threshold, splitting across both tails, combinatorial
  expansion, and several changes in one non-degenerate replacement: the
  reference tool's help text (`docs/research/nbc_help_v284.txt`), confirmed by
  the owner on 2026-10-15 as the behaviour luria should match.
* The split point `ceil(m/2)` forward is luria-native, as in D032: the
  reference tool's own split was never substantiated.
* The 64-design warning threshold and the naming scheme: owner, 2026-10-15.
  Warning rather than refusing follows D032's treatment of the 100-nt limit.

Reversibility:

The inline limit of 6, the split point, the 64-design threshold and the
naming scheme are policy values; changing one needs approval.

Addenda, 2026-10-15:

* **Expansion and variant sets** (owner). A request whose replacement holds
  ambiguity codes expands into a **variant set**, one member per concrete
  sequence, named as above. The set's per-member reporting applies: a member
  that cannot be designed is reported and the others are still designed. A
  degenerate replacement **inside** an existing variant set is refused; sets
  do not nest.
* **Frameshift** (owner). A replacement inside a CDS whose length change
  `m − w` is not a multiple of 3 carries a non-blocking frameshift warning and
  still applies, the policy already chosen for deletions and insertions.
* **Implementation note** (planner). The inline limit of 6 is the constant
  `MAX_INLINE_REPLACEMENT` in `primer_design.rs`, used only for replacements.
  `MAX_REPLACEMENT_REGION` stays at 3: it bounds only single-base and codon
  substitutions, whose regions never exceed 3, so the "rises from 3 to 6"
  above is met by the new constant with no change in behaviour.
* **Wild-type members** (owner). When an expansion includes the bases the
  region already holds, that member would be a no-op edit. It is **skipped
  with a warning** naming it, and the other members are designed. An
  expansion with no member left after skipping is refused as the no-op it is.
  The 64-design threshold counts the members actually emitted, after
  skipping (planner reading of "past 64 designs").

## D034: Milestone order after Milestone 4

Status: approved by the owner, 2026-10-16. Replaces the milestone list in
`docs/product-brief.md` from Milestone 4 on.

Decision:

* **Milestone 4** is insertions, deletions, unequal replacements and
  degenerate expansion (D031–D033), as implemented under
  `docs/tasks/milestone-4-*.md`.
* **Milestone 5** is the machine-readable and CLI workflow: a minimal CLI and
  structured output, so complete scientific workflows can be exercised
  without a TUI, web frontend, or LLM.
* **Milestone 6** is natural-language interpretation: an LLM adapter that
  turns user requests into typed operations and clarification requests.
* **Milestone 7** is the review-oriented user interface (TUI).
* **Milestone 8** is the web interface.

Reason:

Milestone 4 became indel support during implementation, and no scope decision
recorded the change. This records it. The CLI comes before natural language,
as the original order had it, so the LLM adapter targets a typed,
machine-readable surface that already exists.

Reversibility:

Reordering again needs an explicit scope decision.

## D035: Primer design is for linear templates only

Status: approved by the owner, 2026-10-16. Amends `docs/q5-primer-design.md`
§18.2. Corrected the same day: the first version said a circular template is
designed as though it were linear, which is not what the code does.

Decision:

`luriaNative` primer design refuses a circular template. `design_primers`
returns `UnsupportedPrimerDesignScope` for it before any geometry is computed,
for every edit kind. The mutation layer's circular semantics (D009) are
unchanged: circular sequences can still be edited and verified.

Reason:

The reference tool, NEBaseChanger, designs on linear templates only, and no
source for origin-spanning primer geometry has been reviewed. The earlier
§18.2 sentence, "A circular template has no availability bound, only a length
bound", was never implemented. The code already refused circular templates
(`a_circular_template_is_out_of_scope` and its deletion and insertion
counterparts in `primer_design.rs`); this decision records that as policy.

Reversibility:

Supporting origin-spanning primers needs a sourced geometry and approval.

## D036: A circular feature is split only at the origin

Status: approved by the owner, 2026-10-16 (`docs/tasks/review-post-m4.md`
A4). Narrows D009.

Decision:

A feature or CDS on a circular sequence is one interval, or exactly two
intervals that cross the origin: `[a, n)` followed by `[0, b)`. Abutting
intervals split anywhere else, such as `[0, 6)` followed by `[6, 24)`, are
refused at construction with `UnsupportedFeatureStructure`. A linear feature
stays a single interval.

Reason:

An arbitrary split adds no information, and it hid a join that the
containment checks and frameshift warnings treat as an edge: an edit at the
join got no frameshift warning, or was refused as outside the CDS. With one
canonical form, every region that is coding lies wholly inside one interval,
because no single region can cross the origin and an insertion at point 0 is
already refused (D032).

Reversibility:

Accepting arbitrary splits again needs containment and warning checks that
merge abutting intervals, and approval.

## D037: An in-frame DNA edit that creates a premature stop is warned

Status: approved by the owner, 2026-10-16 (`docs/tasks/review-post-m4.md`
A5). Settles the open limitation in `docs/q5-sdm.md` §8.

Decision:

`verify` stays a DNA-level check for DNA-level edits: it confirms that the
edit happened as recorded, and does not re-translate. D013's "the resulting
translation where applicable" is met at the workflow layer instead. When a
substitution, deletion, insertion or replacement lies wholly inside the
resolved CDS and keeps its reading frame, the Q5 workflow re-translates that
CDS on the edited artifact. If a stop codon now appears before the CDS's last
codon, the run carries a non-blocking `PrematureStop` warning naming the CDS
and the codon number on the edited CDS. The edit is still applied, as for a
frameshift (D031–D033), because a nonsense mutation is sometimes the intent.

Not covered, and recorded as open: edits only partly inside the CDS, a CDS
that did not translate before the edit, an edit that removes the terminal
stop (stop loss), and a stop written into the last codon of a CDS that had no
terminal stop. A residue intent states its replacement residue explicitly and
gets no such warning.

Reversibility:

The warning is policy; turning it into a refusal, or extending it, needs
approval.

## D038: `ConfineToTail` is accepted where it changes nothing

Status: approved by the owner, 2026-10-16 (`docs/tasks/review-post-m4.md`
A6). Implements D032's sentence "The 'confine to 5′ tails' option does not
change an insertion" and extends it to D033's tail path.

Decision:

A `luriaNative` request with placement `ConfineToTail` is designed exactly as
the same request with `Inline` when the edit is an insertion (D032), or a
replacement with `max(w, m) > 6`, which already puts every written base in the
5′ tails (D033). The design records the placement the caller asked for. For
every other edit (substitutions, inline replacements and deletions) the
placement is still refused with `UnsupportedPrimerPlacement`: there it would
need a geometry no source defines.

Reversibility:

Supporting `ConfineToTail` for the other edits needs a sourced geometry and
approval.

## D039: Fixture verification is reproducible from tracked files

Status: approved by the owner, 2026-10-16 (`docs/tasks/review-post-m4.md`
A7). Applies D018 and the fixture rule in `AGENTS.md`.

Decision:

* Every script that produced or verified a fixture expectation is tracked
  next to the fixture it serves, under that fixture's `reference/` directory.
* The Python dependencies are pinned in `tests/fixtures/requirements.txt`.
* Every generator has a `--check` mode. It regenerates in memory, compares
  with the tracked file, and exits non-zero on any difference, without
  writing anything.
* One documented command runs every check: `tests/fixtures/verify.py`.
* None of this is part of the required cargo checks (D026). The Rust tests
  stay independent of Python. The Python checks are run when a fixture or its
  source changes.

Reversibility:

Adding the Python checks to the required checks needs approval.

## D040: A codon's recorded coordinates follow the feature interval order

Status: approved by the owner, 2026-10-16 (`docs/tasks/review-post-m4.md`
B10). Applies D006, D009 and D036.

Decision:

* The intervals a residue result records for its codon are listed the way
  D036 writes a feature: in reference order, except that a codon crossing the
  origin of a circular sequence lists the part ending at the sequence length
  first, then the part starting at zero. A codon at bases 22, 23 and 0 of a
  24-base sequence is recorded as `[22, 24)` then `[0, 1)`, and
  `target_coordinates()` gives `22, 23, 0`.
* The order is not the strand's reading order. A minus-strand codon is
  listed in reference order, as a minus-strand CDS's intervals are.
* The mutation fixtures' `coords` field follows the same order.

Reversibility:

Changing the order changes recorded results and fixture expectations, and
needs approval.

## D041: The Milestone 5 CLI, its inputs and its output

Status: approved by the owner, 2026-10-16 (`docs/tasks/milestone-5-cli.md`
§4). Applies D003, D006, D007, D009, D021 and D034.

Decision:

* **Dependencies.** The new `luria-cli` crate may depend on `serde` and
  `serde_json`, and on nothing else. It parses its arguments by hand.
  `luria-core` stays dependency-free.
* **Coordinates.** Every nucleotide coordinate in the request and result JSON
  is zero-based half-open, written as a `[start, end]` pair. The request
  declares this in a required field, `"coordinates":
  "zero-based-half-open"`, and any other value is refused. Residue numbers
  are 1-based, as in the library.
* **Output.** The CLI writes one JSON document to stdout, for success,
  clarification and failure alike. There is no human-readable text output in
  this milestone.
* **Sequence input.** Both FASTA and GenBank are read in this milestone,
  one record per file. The readers live in `luria-core`. A FASTA template's
  topology and CDSs are declared in the request.
* **GenBank CDSs.** The request names the CDS features to import, by id.
  Only those become annotations. If a named CDS can't be represented, the run
  fails with that CDS's own error. Every other feature is listed in the
  result as not used, and never fails the run.
* **GenBank CDS identity.** A CDS's id is the first of `/locus_tag`, `/gene`
  and `/label`. The result records which qualifier it came from. A name that
  matches no CDS, or more than one, is refused.
* **`/translation`.** When a named CDS has a `/translation` qualifier, it
  must equal luria's translation without the terminal stop. A mismatch
  refuses the CDS before any edit.
* **GenBank topology.** It comes from the LOCUS line. A LOCUS line that
  states neither `linear` nor `circular` is refused.
* **`/transl_table`** (added 2026-10-17, owner answer G5). Absent or `1` is
  the standard code. `11`, the bacterial code, assigns the same amino acid to
  every codon as `1` and differs only in allowing alternative start codons,
  so a named CDS with `/transl_table=11` is accepted only when its first
  codon is `ATG`. It is then imported as the standard code, and the result
  records that the record said 11. Table 11 with any other start codon, and
  every other table, is refused.
* **Buffer** (added 2026-10-17, owner answer G6). A request states its
  primer-design buffer, which is never defaulted (D028): either the three
  numbers, monovalent salt in mM, primer in nM and DMSO in percent, or the
  named preset `q5_reaction` (150 mM, 500 nM, 0 %), which is core's
  `BufferModel::q5_reaction`. The result echoes the numbers, and the preset's
  name when one was used.
* **A single edit's design failure** (added 2026-10-17, owner answer G7). An
  edit that is applied and verified but cannot be designed is reported like a
  one-member set: the state stays `verified`, the member carries its mutation
  and its `designFailure`, and the run exits 7. The verified edit is not
  discarded because no primer pair fits.
* **`--write-edited DIR`** (added 2026-10-17, owner answer G8). A directory
  that does not exist is created, parents included. An existing file is never
  overwritten: that is `output_exists` and exit 4, with nothing written.
* **FASTA against GenBank** (added 2026-10-17, owner answer G9, amended the
  same day). The claim is that the same template in either format gives the
  same **result**, proven on pBR322 (R1), a real record with real CDS
  features. It is not proven on a primer design: pBR322 is circular, D029
  scopes `luriaNative` to a linear template, and both GenBank fixtures in
  the repository are circular, so both runs report a verified edit whose
  design was refused. The SID-1 cDNA stays a FASTA fixture only: writing a
  3153-base GenBank record with a `/translation` qualifier by hand is not
  practical, and generating one would make the fixture a product of a script
  rather than of independent verification.
* **Plasmids cannot be designed** (added 2026-10-17, owner answer G10).
  D029's native design scope is a linear template, so every circular
  template — every plasmid, which is where most site-directed mutagenesis is
  done — gives a verified edit with an `unsupported_primer_design_scope`
  design failure and exit 7. The CLI does not refuse such a request when it
  is read: the edit and its construct are still correct and still reported.
  The limit is stated in `docs/cli.md` and the README. Lifting it means a
  design policy for circular templates, which is not in this milestone.

Reversibility:

Adding a dependency, a coordinate convention, a text output or another input
format needs approval. So does relaxing any of the GenBank refusals.

## D042: Every imported GenBank CDS starts with ATG

Status: approved by the owner, 2026-10-18 (`docs/tasks/review-post-m5.md`
A-9). Amends D041's `/transl_table` rule.

Decision:

* A CDS a request names from a GenBank record is imported only when its
  first codon is `ATG`, whatever its `/transl_table` (absent, 1 or 11). Any
  other first codon is refused as `alternative_start`, with the codon as the
  detail.
* Reason: luria translates codon 1 literally. A cell reads every start codon
  as methionine, so a GTG, TTG or CTG start would give a wrong residue 1, and
  a residue edit numbered against it would assert the wrong original. Table
  11 was already limited to ATG starts (G5); tables absent and 1 now follow
  the same rule.
* FASTA CDSs declared in a request are unchanged.

Reversibility:

Accepting alternative starts needs approval and a rule for residue 1.

## D043: Upper limits on the primer-design buffer

Status: approved by the owner, 2026-10-18 (`docs/tasks/review-post-m5.md`
C-4). Amends D041's buffer rule.

Decision:

* A buffer is refused (`invalid_buffer_model`) when its monovalent salt is
  above 1000 mM, its primer concentration above 10000 nM (10 µM), or its
  DMSO above 20 %. Each limit is inclusive. The existing lower limits stay:
  salt and primer above zero, DMSO at least zero.
* Reason: the Tm model accepted any finite positive value, so an absurd
  buffer gave a verified result with meaningless Tms and primers. 1000 mM is
  about the top of the Owczarzy salt correction's validated range; the other
  two are well above any real PCR condition.

Reversibility:

Changing a limit changes which requests are accepted and needs approval.

## D044: Natural-language interpretation (Milestone 6)

Status: approved by the owner, 2026-10-19 (`docs/tasks/milestone-6-nl.md`
§4, answers M6-1 to M6-18). Applies D004, D005, D016, D021, D023, D028 and
D034. Amends D041's dependency rule.

Decision:

* **Two steps** (M6-3). `luria q5 interpret` turns a request in words into
  a proposed `luria.q5-request/1` request, or into questions, and writes
  it as one `luria.q5-interpretation/1` document. The user reviews it and
  runs `luria q5 design` on it, unchanged. Removing the model changes no
  scientific calculation (D004).
* **The model extracts; Rust decides** (M6-5, M6-15). The model returns a
  narrow extraction: the operation, the CDS name, each substitution as a
  canonical token with a verbatim quote of the user's words, the
  composition and the user's words about numbering. Rust grounds every
  string against the user's text; checks each token against its quote (the
  number written there, both residues named there, the original before the
  target); reads the notation; checks the CDS against the record; and
  builds the request itself. The model writes no request field, no
  coordinate, no sequence and no primer. A string the text does not
  contain fails the run (`ungrounded_extraction`); a quote that does not
  support its token is a question.
* **What leaves the machine** (M6-4). The user's text, and per CDS its id,
  `/gene`, `/product`, residue count, strand and whether luria can use it.
  Never the sequence, never a coordinate. The document records the
  messages sent. An endpoint that is not loopback needs `--allow-remote`
  (D023), checked before anything is read or sent.
* **Backend** (M6-1, M6-2, M6-17, M6-18). One backend, an OpenAI-compatible
  `/chat/completions` endpoint over plain HTTP, through `ureq` with its
  default features off: temperature 0, a fixed seed, the extraction schema
  as `response_format`, no thinking or token-limit setting. HTTPS and the
  Anthropic API are deferred, because the TLS stack's C build needs paths
  the development sandbox denies (`docs/tasks/later-tls-and-anthropic.md`).
* **Templates** (M6-7): GenBank only, so CDSs come from the record. FASTA
  is refused (`interpret_fasta_unsupported`).
* **Notation** (M6-8): residue substitutions only, one-letter (`A42G`) and
  three-letter (`Ala42Gly`), optionally `p.`-prefixed; `?` marks a part the
  user did not state, and is asked about.
* **No silent choices** (D005). A CDS the text does not name is always
  asked about, whatever the record holds (M6-9). Two or more substitutions
  with no stated composition are asked about. Numbering is taken from the
  text, from a fixed list of phrasings, or from the profile; otherwise
  asked about (M6-11).
* **The design profile** (M6-6). Values the text does not state — the
  design, a codon policy, the numbering origin — come from a user-owned
  `luria.q5-design-profile/1` file, read with the request parser's own
  `design` rules. An absent value is a question; luria defaults nothing
  (D028).
* **The template id** is the record's LOCUS name, reported as an
  assumption (M6-10).
* **A combined construct is proposed as asked** (M6-16), and core refuses
  it at design: the adapter does not duplicate a scope rule core owns
  (D003).
* **Exit 8** is a refused model reply (M6-14). `luria q5 design`'s result
  schema is unchanged (M6-12).
* **Dependencies.** `luria-nl` holds model I/O only and may depend on
  `serde`, `serde_json` and `ureq` (default features off). `luria-cli` may
  depend on `luria-nl`. `luria-core` stays dependency-free.
* **Measured, not assumed.** The prompt shipped is the one measured on a
  small local model (`docs/research/m6-local-model-extraction-2026-10-19.md`)
  and is pinned to that record by test.

Reversibility:

Adding a backend, a TLS stack, a notation, a template format, a default
for any value, or a field the model may fill needs approval. So does
sending the model anything more of the record.

## D045: Tighter grounding of the model's extraction

Status: approved by the owner, 2026-10-20 (`docs/tasks/review-post-m6.md`,
owner decisions). Amends D044. Each rule fails closed: a doubtful case is
asked about, never proposed.

Decision:

* **One number per quote, in order** (B-1, B-2). A mutation's quote
  supports its token only when it holds exactly one run of digits, equal
  to the residue number and joined to no letter except a residue's name,
  with the original residue named before it and the target after it. A
  quote with more than one number is asked about
  (`mutation_quote_ambiguous`).
* **Numbering in context** (B-3). A negation or contrast word in the
  sentence holding the user's numbering words, or another numbering
  origin anywhere in the text, makes the numbering doubtful.
* **A CDS name is a whole name** (B-4). It must occur in the text with no
  letter, digit, `_` or `-` on either side, and not inside a longer CDS id
  the record has.
* **Composition is read by Rust** (B-5), from a fixed list of phrasings in
  the text. None is `composition_unstated`; a model's value that disagrees
  is asked about.
* **A codon belongs to its mutation** (B-7). It must occur, as a whole
  word, in that mutation's own quote.
* **Nothing dropped or repeated** (B-6, B-8). A substitution token in the
  text that no quote covers, and two mutations with the same canonical
  token, are asked about. A substitution written only in prose that the
  model drops is not detected: the user's review of the proposal is the
  control.
* **No key over plain HTTP** (M6-18). An API key with an endpoint that is
  not loopback is refused before anything is sent, until luria has TLS.

Reversibility:

Loosening any of these rules needs approval.

## D046: The review terminal interface (Milestone 7)

Status: approved by the owner, 2026-10-20 and 2026-10-21
(`docs/tasks/milestone-7-tui.md` §4, §5.3a, §5.3b, answers M7-1 to
M7-30); the planner choices of the plan's §8a were confirmed by the
owner on 2026-10-21. Applies D003, D004, D005, D012, D020, D021, D023 and
D034. Amends D041's dependency and text-output rules.

Decision:

* **`luria review`** (M7-1). `luria review REQUEST.json` runs a request as
  `luria q5 design` does and shows its result in the terminal.
  `luria review --template … --text … --request FILE.json` interprets a
  request in words as `luria q5 interpret` does, shows the proposal, and
  only on the user's explicit approval writes the request to
  `--request`, under `--write-request`'s rules, and designs from that
  file. Quitting before approval writes nothing and runs nothing.
  `--request` is required, so the file is named and checked before the
  model is called (§8a).
* **Presentation only** (D003, amended by M7-21). The screens show
  fields of the result and interpretation documents. The variant's
  protein comes from luria-core (`frame::mutant_residues`), read from the
  edited construct core produced, used only when its digest is the
  variant's: in the CDS's frame, on past a frameshift, ending at the
  first stop codon (M7-18, M7-20). The CLI only lays bases and residues
  out in columns, takes complements with core's `complement_letter`,
  reverses a reverse primer's text to read 3'→5', converts positions to
  1-based inclusive for display (M7-24; files stay zero-based, D006),
  and rounds a Tm to one decimal on the variant screen (the full value is
  on the details screen). It computes no Tm, codon or translation.
* **The screens** (M7-16 to M7-30). `enter` on a variant opens its
  sequence with the protein between the strands; `d` opens the details;
  `i` expands an insertion over 30 bases, which starts collapsed; `?`
  opens the legend. The overview lists the template, the target and the
  variants. Warnings are shown in core's own words.
* **State is always on screen** (D020, M7-23). Every result screen's
  frame title holds the state as a coloured badge and the experimental
  validation; a proposal's badge is `PROPOSAL`.
* **Questions are read-only** (M7-3). To change a request, the user
  changes the text or profile and runs again.
* **Export** (M7-4): the result JSON byte for byte as `luria q5 design`
  prints it, the edited constructs as `--write-edited` writes them, and an
  IDT order table, written only on `w` or `o` into the `--export DIR`
  directory, as `<request>.result.json`, `<template id>_<member>.fa`
  (M7-57) and `<request>.order.tsv`; an existing file is replaced only
  on `y` at a prompt that names it (M7-49, D047).
* **The IDT order table** (M7-4, M7-6 to M7-13, M7-15). Tab-separated,
  header `Name Sequence Scale Purification`, one row per primer of each
  designed member, forward then reverse, named `<template id>_<member>_F`
  or `_R`. The scale is fixed by full primer length: 15-60 nt `25nm`,
  61-90 `100nm`, 91-100 `250nm`, 101-200 `4nmU`; purification is always
  `STD`. A primer outside 15-200 nt, a base other than A, C, G or T, or a
  result that is not verified refuses the table; nothing is guessed. The
  codes were copied from IDT's Bulk Input panel by the owner on
  2026-10-20; the ranges are IDT's published ones. luria writes a file and
  never submits an order.
* **Dependencies** (M7-2, M7-5). `luria-cli` may depend on `ratatui` and
  `crossterm`. They are vendored by the owner outside the development
  sandbox (`cargo vendor`, ignored in git, M7-9, M7-14). `luria-nl` is
  unchanged; `luria-core` gains only `frame::mutant_residues` (M7-21),
  with no dependency.
* **Tested without a terminal.** State, text and drawing are pure and
  tested on a test backend; the terminal session is checked by the owner
  by hand. The member-screen layouts and mutant reading frames are fixed by
  `tests/fixtures/review/`, written from independently verified
  `tests/fixtures/q5-primer/` rows by scripts that splice and translate
  on their own.

Reversibility:

Adding an editing screen, a new export format or vendor, a default scale
or purification, or any value the screens compute needs approval.
The variant screen's layout and words (M7-16 to M7-30) are the
owner's, after using the earlier versions.

## D047: Entering a design in the terminal (Milestone 7, `luria new`)

Status: approved by the owner, 2026-10-21 and 2026-10-22
(`docs/tasks/milestone-7k-new.md`, answers M7-31 to M7-44, and
`docs/tasks/milestone-7m-polish.md`, answers M7-45 to M7-65, after the
owner's first hand test). Applies D003, D005, D006, D012, D029, D034,
D044 and D046.

Decision:

* **`luria new`** (M7-31, M7-62, M7-63). A form builds a
  `luria.q5-request/1` request from a template, mutation lines and the
  overlap. `[ Save and run ]` writes it under the form's `Save as` name
  and runs it, then shows it on `luria review`'s screens, where the
  user checks the mutations. A file that exists is replaced only on
  `y`. Quitting before saving writes nothing and runs nothing.
* **The template** (M7-32, M7-37, M7-50). A FASTA or GenBank file, picked
  from a filtered list of the directory being typed or typed in full, or
  a pasted sequence saved as a new FASTA file. For FASTA and a paste the
  user states the CDS (whole sequence, or 1-based first and last base,
  and the strand); for GenBank the user picks a usable CDS from the
  record. Never guessed. The request goes beside a template file, as
  `--write-request` requires; a paste and its request go in the current
  directory.
* **The shorthand** (M7-35, M7-46). One mutation per line, read by
  luria's own code with no model: residues as milestone 6's notation,
  with an optional codon (`D404N:AAC`), else the default codon policy;
  base changes, deletions and insertions by 1-based inclusive template
  positions, converted to the request's zero-based half-open form (D006)
  by one tested function. A one-base change is a substitution; any longer
  range is a replacement. Typed bases must match the template. Several
  lines are separate variants (M7-34).
* **Words** (M7-33, M7-41, M7-42, M7-47, M7-59). A mutation line luria cannot
  read at all makes the whole box words. With `--backend` the box goes
  through `luria q5 interpret`'s path unchanged, with the form's overlap,
  the default codon policy, residues counted from the CDS start, and,
  when the words name no CDS, the form's CDS as an assumption (M7-59;
  M6-9 still holds for `luria q5 interpret`); it
  needs a GenBank template (M6-7). A reading with no questions is
  written and run at once, and the reading itself is at the bottom of
  each variant's Details (M7-62, M7-64); for `luria new` only, this
  replaces D046's rule that nothing is written until the user approves,
  so the result screens are where the reading is checked. A reading
  with questions writes nothing. Without a backend the form names
  `--backend`. A line
  luria reads but refuses is an error and is never sent to the model.
* **Values on screen** (M7-34, M7-39, M7-45). The overlap starts at 15
  nt, shown and editable, or at a `--design` profile's value; placement
  is shown as a fixed `inline`, the only placement `luria q5 design`
  supports (D029). The overlap is shown on the form before anything is
  saved, so it is the user's value.
* **Replacing a file** (M7-49). When a file `luria new` or the review
  export would write exists, a prompt names it and only `y` replaces it.
  The new text is written to a temporary file and renamed over the name;
  a name that is a link or a folder is refused. `luria q5 design
  --write-edited`, `luria q5 interpret --write-request` and `luria review
  --template` still never replace a file.
* **Same code paths.** The request is built as the request parser's own
  types, the paste is written by luria-core's FASTA writer, and the
  design runs as `luria review` runs an approved request.
* **Tested without a terminal.** The shorthand, the request builder, the
  form, its pop-up, its file list and their text and drawing are pure and
  tested against planner fixtures in `tests/fixtures/new/`, written by
  scripts that share no code with `crates/`, and every expected request
  was run through `luria q5 design`. The terminal session is checked by
  the owner by hand.

Reversibility:

Other placements, composition choices, a different default overlap, or
any other default for a scientific value needs approval.

## D048: What the Milestone 8 view selects, shows, and refuses to read

Status: approved by the owner, 2026-10-24
(`docs/tasks/milestone-8-view.md`, answers M8-11 to M8-14). Applies
D003, D005, D006, D018, D041, D046 and D047. M8-15 was answered on
2026-10-24 and is recorded in D049; M8-1, M8-3, M8-6 and M8-20 are in
D049, M8-16 to M8-19 in D050, M8-10 in D051. Still open there: M8-2,
M8-4, M8-5, M8-7, M8-8 and M8-9.

Decision:

* **Which CDS is selected on open** (M8-11). A record with exactly one
  usable CDS opens with it selected. A record with several opens with
  none, and the title bar says so. The view never picks a CDS silently,
  because the selected CDS decides the residue numbering on screen, what
  the codon-step keys do, and what a protein search reads — the case
  D005 lists first among the ambiguities that block.
* **A cap on search hits** (M8-12). The view and `luria find` show at
  most 200 hits and say how many there really are (`showing the first
  200 of 8521 hits`). The cap is presentation: luria-core returns every
  hit, at every position, overlapping included, uncapped. Without it one
  all-ambiguity pattern such as `N` hits at every base of the record —
  8521 hits on an 8521 bp construct — which is no usable hit list in the
  view and 8521 rows on stdout.
* **A terminal or nothing** (M8-13). `luria FILE` needs a terminal on
  both stdin and stdout, checked by the same code `luria review` and
  `luria new` use, and refusing with the usage error and exit status
  `docs/cli.md` §5 already gives. Redirected output is a usage error, not
  a half-drawn screen; the check runs before the file is opened. `luria
  find` needs no terminal and writes its table to stdout like any other
  report.
* **How big a file luria will read** (M8-14). A record over 10 000 000
  bases is refused, and the limit is named in the message. The
  application refuses an over-long file on byte size before reading it,
  so a gigabase FASTA never reaches the parser. This bounds what the
  application opens; it does not change the sequence model's own
  coordinate limits (D006), and both checks are tested at the boundary
  and one base or byte past it.
* **Shared entry point.** The view and `luria find` build the record
  through one luria-core function, so file-format detection, topology and
  these limits cannot differ between the two.

Reversibility:

Raising or removing either ceiling, a different hit cap, or selecting one
CDS when a record offers several needs approval, since the last one would
put numbers on screen that the user did not choose.

## D049: Reading a record for the view: partials, format, topology, CDS names

Status: approved by the owner, 2026-10-24
(`docs/tasks/milestone-8-view.md`, answers M8-1, M8-3, M8-6, M8-15 and
M8-20). Applies D005, D006, D009, D036, D041 and D042.

Decision:

* **Partial features are drawn, by their own parser** (M8-1). A feature
  location with `<` or `>` — `<1..12`, `1..>900` — is read for display by
  a separate display-only parser, which takes the stated base as the
  bound and records which end is uncertain. The feature list names the
  uncertain end; go-to does not jump to a feature whose start is not
  stated. `parse_genbank_location` and its tests stay exactly as they
  are: no flag is added to the shared parser, because a boolean that
  changes what it accepts is the shape D009 and D036 reject. CDS import
  still refuses a partial location (D041), so a partial feature is never
  selectable for numbering and D042 is unchanged.
* **Format by content, not by name** (M8-6). `LOCUS` first is GenBank,
  `>` first is FASTA, anything else is refused. The view and `luria find`
  read the record through one core entry point, so they cannot disagree
  about what a file is (D048).
* **FASTA topology is assumed, stated, and switchable** (M8-3, M8-20).
  FASTA carries no topology, so the view assumes linear and labels it
  `linear (assumed)`; `T` switches the record to circular for that
  session, and `luria find --circular` does the same for one run. A
  GenBank record's declared topology stays authoritative as D036 reads
  it. This is a bounded exception to D005's "never guess": the
  assumption is named on screen, changed by one visible key, and cannot
  enter a design's arithmetic — no design is built from a view-only
  assumption, and `find` names the mode it searched in. It licenses no
  other inference, and what an assumed-circular FASTA may be *designed*
  on remains milestone 9's question (T1, D035 — unchanged, unapproved).
* **`--cds VALUE` names a CDS by any id the record offers** (M8-15). It
  matches the display name, `/gene`, `/locus_tag` and `/protein_id`, and
  the list form's `name (start..end)` only as the whole string. Matching
  ignores ASCII case and interior whitespace; a substring must start at
  a word boundary; a case-folded prefix is accepted when it matches one
  CDS and rejected when it matches several. When several CDSs match,
  `luria find` **refuses and lists the candidates** with the id source
  and span that matched — it does not pause, because a command a script
  runs must fail fast; choosing among them by eye is the view's picker.
  The flag takes no range in milestone 8; `86..1276` is refused with a
  message saying ranges come later, so the grammar does not change under
  `proposal-commands.md` §3.2.

Reversibility:

Drawing a partial feature without marking the uncertain end, letting the
science parser accept `<` or `>`, making FASTA circular by default, or
letting an ambiguous `--cds` pick a CDS instead of listing the candidates
all need approval.

## D050: What the Milestone 8 search and frame fixtures encode

Status: approved by the owner, 2026-10-24
(`docs/tasks/milestone-8-view.md`, answers M8-16 to M8-19). Applies
D006, D010, D018, D028, D040 and D048.

Decision:

* **Overlapping hits are all reported** (M8-16). `AA` on `AAA` is three
  hits. On a circular sequence an equal-length pattern matches at every
  rotation — a 6 bp pattern has as many start positions per strand as the
  record has bases, including starts that run across the origin, and
  those are not omitted. luria-core returns them uncapped; the 200 cap is
  presentation (D048).
* **Whose residue numbers a protein hit carries** (M8-17). In CDS mode it
  carries that CDS's residue numbering (D006), and a hit ending on the
  terminal stop carries none, because D010 says that codon is not a
  residue — `luria find` prints `stop` there and the view says "to the
  stop codon". In six-frame mode the number counts codons within that
  frame from its first complete codon, and the strand and frame columns
  are never empty.
* **Reference order is not reading order.** `residue_bases` returns
  reading order, as D040 records: descending base ranges on a
  minus-strand CDS, so `bla` residue 1 sits on bases 4153, 4152, 4151.
  D040's `aa_to_base` columns are ascending reference order, because
  design intervals are sorted and merged. They are the same bases in two
  orders, and every Milestone 8 fixture column states which one it holds.
* **A leftover codon in a frame is not drawn** (M8-18). When a frame's
  length is not a multiple of 3, the trailing one or two bases have no
  letter above them. `?` means a *complete* codon that cannot be read —
  an ambiguity code, or a codon D010 refuses to guess at. The Python
  reference script maps Biopython's partial-codon output by hand and
  states what it did, because Rust and Biopython disagree there first.
* **Blocks snap to the translation's phase** (M8-19). In CDS mode a block
  starts on the selected CDS's phase even when the terminal is too narrow
  for a whole codon's worth of columns; the block widens or the view
  scrolls. A residue never moves off its own bases, so single-basing past
  a block edge does not re-seat every letter on the screen. The rejected
  alternative — a split codon drawn over the line holding its middle base
  — would make the residue-to-base relationship depend on scroll position
  and double the layout fixtures.

Reversibility:

Collapsing overlapping hits, numbering frame-mode residues from the
sequence start instead of the frame, drawing the leftover codon, or
splitting a codon across a block edge all need approval, and each changes
fixture rows already written against them.

## D051: The two-tier process, for Milestone 8 only

Status: approved by the owner, 2026-10-24
(`docs/tasks/milestone-8-view.md`, answer M8-10; triage P1 and P2 of
`docs/tasks/proposal-decision-triage.md`, which remains otherwise
unapproved). `AGENTS.md` is **not** edited by this entry: the rules there
stay as they are, and this is the owner's exception for one milestone,
recorded here so the difference is visible rather than inferred.

Decision:

* **Science tier keeps the full process.** Contracts 8a1, 8a2, 8b, 8c,
  8d, 8e, 8f, 8g and 8j are written as the milestones so far were: named
  functions and signatures, the exact tests and commands, a "do not"
  list, planner-written fixtures with independently verified expectations
  recorded under `tests/fixtures/`, and decision entries for what they
  settle. Nothing about that changes.
* **Presentation tier is lighter.** Contracts 8h, 8i, 8k1, 8k2, 8k3, 8l1
  and 8l2 name the behaviour, the keys and the tests, but key bindings,
  colours and on-screen sentences are not numbered as M8-n questions and
  get no decision entry. Their tests assert what a command does — that
  pressing `j` moves the selection to the next base, that the view opens
  a file and refuses one it cannot read — not the exact wording of a
  status line. The planner may change such wording or a key between
  contracts without an approval round.
* **Scope and expiry.** This applies to Milestone 8 and nothing else. It
  expires when Milestone 8 merges: the next milestone returns to
  `AGENTS.md` unless the owner approves triage P1 and P2 as standing
  rules. T1–T6 and P3–P5 stay unapproved, so the biology and the
  design-side rules they would change are untouched.
* **Why.** The milestone exists to find out how luria's sequences read at
  a keyboard. A keystroke set chosen before that test is a set the owner
  will want to change after it, and under the current rules each change
  costs a contract amendment and a decision entry. The part of the
  process that protects the science — fixtures, independent verification,
  no silent inference, no expectation copied from an implementation — is
  kept in full.

Reversibility:

The owner can pull any named screen back to the full process at any point
in the milestone by naming it. A later milestone that approves triage P1
and P2 supersedes this entry; one that does not returns to `AGENTS.md`
by default.

## D052: Frames, ambiguity codes, palindromic sites, and what `find` exits with

Status: approved by the owner, 2026-10-24
(`docs/tasks/milestone-8-view.md`, answers M8-2, M8-4, M8-5, M8-7, M8-8
and M8-9, all as proposed). Applies D006, D010, D018, D028, D041, D042,
D048, D049 and D050. With this entry, Milestone 8's open-question list is
closed.

Decision:

* **Six frames do not run across the origin** (M8-2). Frames are counted
  on the linear coordinates from base 1 and stop at the last base, on a
  circular record as on a linear one: a codon that would wrap is not
  translated in frame mode. A CDS that crosses the origin is still one
  translation in CDS mode, joined as D018 has it. Frame numbering is
  therefore independent of topology, and switching a FASTA to circular
  (D049) changes search and the overview, not the frames.
* **A template ambiguity code matches only what it truly is** (M8-7). At
  an aligned position, the template letter matches a pattern letter only
  when the pattern letter's base set contains **all** the template
  letter's bases: template `A` matches pattern `A`, `R` and `N`; template
  `N` matches pattern `N` alone. This rule is not derived from any
  library. An earlier version of this bullet said Biopython's IUPAC
  matching is intersection-based and answers `N` versus `A` the other way
  round; measured on the pinned `biopython==1.88` that is not what
  `Bio.Restriction` does — it compiles a regular expression out of the
  enzyme's own site and matches the record literally, so `BstEII`
  (`GGTNACC`) matches `GGTXACC`, where `X` is no IUPAC letter at all,
  while `Dpi3090II` (`AAGRAG`) and `SenSARA26III` (`ACRCAG`) refuse a
  record `R` at the position of their own `R`, which this rule accepts.
  Neither containment nor intersection: Biopython says nothing about how
  a record's own ambiguity codes should be read. So the reference script
  hand-codes the containment rule, and Biopython is used in the fixtures
  only where measurement supports it — the letter sets, the complement
  table, and site counts over records whose bases are all `A`, `C`, `G`
  and `T`. The answers are asserted in
  `tests/fixtures/view/reference/search_dna.py`, corrected in
  `docs/tasks/milestone-8-view.md` §6 and §8, and recorded in
  `tests/fixtures/view/README.md` (text amended 2026-10-24 on the
  owner's instruction; the rule itself is unchanged).
* **A palindromic site is one hit** (M8-8). A site equal to its own
  reverse complement is reported once, with strand `both`, in the view and
  in `luria find`; a non-palindromic match still yields separate `+` and
  `−` hits. pBR322 makes the count checkable: its EcoRI site `GAATTC`
  occurs **exactly once** in the whole record, at 4359, only because the
  record is circular (`GAA` at 4359–4361 plus `TTC` at 1–3), and its
  BamHI (375), HindIII (29) and NdeI (2295) sites likewise occur exactly
  once — matching the map every pBR322 prep sheet carries. Searching it
  for `GAATTC` yields one hit, not two, and zero if the search refuses to
  wrap.
* **`luria find` exits 0 when nothing matches** (M8-9). A search that
  finds nothing is a successful run: exit 0, `0 hits` on stdout. Exit 1
  stays what `docs/cli.md` reserves it for — an internal error — and the
  usage, malformed-input and design-failure codes stay 2, 4 and 7.
  `grep`'s habit of exiting 1 on no match would read here as a broken
  program, and a script that chains on `&&` would treat a clean search as
  a failure.
* **ORF finding stays in milestone 9** (M8-4). Milestone 8 gives a FASTA
  file six-frame translation and nothing that numbers an ORF. Triage T2
  is not approved and is not implemented here.
* **The CDS picker shows unusable CDSs** (M8-5). pBR322's ROP CDS appears
  in the picker, greyed, with the reason it cannot be selected, and
  cannot be chosen. The reason comes from the refusal code the core
  returns (D042), not from a second judgement in the ui — the ui renders
  a code, it does not decide biology. Whether `GTG` starts become usable
  is triage T3, unapproved.

Reversibility:

Letting frames wrap, matching a record's ambiguity codes by intersection
instead of containment, reporting a palindromic site twice, exiting 1 on
no matches, or deciding greyness in the ui instead of from a core refusal
code each need approval, and the first three change fixture rows written
against them.

## D053: How the view draws partial features and names refusals

Status: approved by the owner, 2026-10-26, all four as proposed in
contracts 8b1 and 8b2. Applies D036, D041, D042 and D049.

Decision:

* **A partial marker is drawn only on a feature's outer ends.** A `<` is
  drawn when its base is the first interval's start, and a `>` when its
  base is the last interval's end, both in D036 order. A marker on an
  inner boundary (`1..<12`, or `join(55..>60,1..5)`, whose `>` sits on the
  origin boundary) is not drawn and keeps its `fuzzy_end` refusal: one bar
  with an open inner end has no honest drawing, and no fixture holds one.
* **The feature list gives the refusal that stands without the marker.**
  The display parser removes outer markers before calling the shared
  parser, so `order(<1..10,20..30)` is refused there as `order` and
  `join(1..>5,8..10)` as `unsupported_join`, while
  `parse_genbank_location` keeps answering `fuzzy_end` for both.
* **Refusals shown in the view are the existing `DomainError` codes.**
  `docs/tasks/milestone-8-view.md` §4.1's `partial_location`,
  `non_atg_start` and `fuzzy_start` were illustrations; inventing them
  would give one rule two codes. A CDS shows the first rule that refuses
  it, so pBR322's ROP is unusable as `no_id`, though it also has a `GTG`
  start.
* **A feature's display name is not its CDS id.** The name is the first
  of `/label`, `/gene`, `/locus_tag` and `/product` with a value, else the
  feature key; the D041 id is unchanged. synthetic's `m1` CDS is named
  `ignored-gene` in the feature list, and ROP is named `ROP protein` with
  no id.

Reversibility:

Drawing inner markers needs a drawing rule for an open inner boundary.
New refusal codes for the view would change `docs/cli.md`'s stable codes
(5a1). Either needs approval and changes fixture rows written against
these answers.

## D054: Protein-search matching and order, and the overview's columns

Status: approved by the owner, 2026-10-26. Settles four questions that
contracts 8f and 8g put to the owner and that
`docs/tasks/milestone-8-view.md` §4.5 and §4.6 left open. Applies D006,
D010, D050 and D052.

Decision:

* **`X` matches every symbol, the stop included.** In a protein pattern
  `X` matches any residue, an unreadable codon `?`, and `*`. So `MKMAWX`
  finds synthetic's `pj` (`MKMAW*`), and `KX` finds frame +1's `K*` in
  `ATGAAATAG`. `*` still matches only `*`, any other letter only itself,
  and `?` is matched only by `X`. This is a matching rule only: the stop
  codon still has no residue number (D010), so a CDS hit that ends on it
  has no last residue. The planner proposed that `X` skip stops, since a
  stop is not a residue; the owner chose the wider match.
* **Protein hits come left to right by base, in both modes.** Hits sort
  by their zero-based `start`, as DNA hits do (D052), and two six-frame
  hits with the same start come plus strand first. In CDS mode this is not
  reading order: on a minus-strand CDS later residues come first, and in
  `pj`, which crosses the origin, `M` at residue 3 (base 0) comes before
  `M` at residue 1 (base 54). A hit that crosses the origin sorts by its
  start, near the end of the record. `luria find --cds` prints its rows in
  this order. The planner proposed reading order for CDS hits.
* **Overview columns use the floor formula.** With `w` columns over `L`
  bases, column `i` covers `[⌊i·L/w⌋, ⌊(i+1)·L/w⌋)`. Sizes differ by at
  most one, and the larger columns are spread through the strip rather
  than bunched at one end: pBR322 (4361 bases) on 80 columns gets 41
  columns of 55 bases and 39 of 54, interleaved.
* **A record shorter than the overview is drawn narrow.** When there are
  more columns than bases, the overview uses one column per base and the
  strip is that much shorter: a 60 bp record on 80 columns is 60 columns
  wide. No column is empty and no base is stretched over two. Keeping all
  80 columns would leave every fourth one empty under the floor formula,
  and a feature drawn in the columns its bases fall in (§4.6) would show
  gaps.

Reversibility:

Each is local to one function: `find_protein`'s matcher or its sort, or
`overview_columns`. The first two also fix rows in
`tests/fixtures/view/search-protein-hits.tsv` and the order `luria find`
prints, so reversing either rewrites that table and needs approval.
