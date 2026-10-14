# Milestone 3 task record: Q5 SDM workflow contract

Status: Contract written; the accepted input scope, target resolution, and
the deterministic vertical slice are implemented in `crates/core/src/q5.rs`,
and Milestone 3a's separate-variant composition is implemented there too.
Candidate generation, primer design, and thermodynamic calculation are not.
Date: 2026-10-11

This file records the concrete choices made while implementing the Q5 slice,
in the style of the Milestone 1 and Milestone 2 records. It does not restate
`docs/decisions.md`; each entry names the decision it implements. If an entry
contradicts `docs/decisions.md`, that document wins and this entry is wrong.
The workflow specification itself is `docs/q5-sdm.md`; this record points at
it rather than repeating it.

## Scope actually implemented

Only the Q5 workflow layer: the accepted input scope (D014's first boundary
entry), target resolution, the typed request that carries an intent
separately from a resolved target, the deterministic vertical slice of D015
composed over Milestone 1 and Milestone 2's public APIs, and the workflow
result and run records. No LLM adapter, no natural language, no CLI, no
persistence, no Python worker, no primer or candidate logic, no
thermodynamic calculation, no warnings.

The mutation domain and the sequence domain were not touched. Every
biological rule this layer needs is reached by calling Milestone 1's or
Milestone 2's public API.

## The contract

### What a Q5 request carries (D016, D014, D011)

`Q5SdmRequest` is `template` (a `Sequence` artifact), `scope`
(an optional `Identifier`), and `intent` (a `Q5MutationIntent`). The
numbering convention is not a request field: it belongs to the annotation,
which carries it as `Cds::numbering` (Milestone 1's `Numbering`). The
identified target CDS is not a field of its own either: `scope` and the
intent are the two halves that can each identify it, and `target_cds_name`
prefers the intent's claim and falls back to `scope`.

`Q5MutationIntent` has three inhabitants: `Nucleotide` (Milestone 2's
`NucleotideMutationIntent`, unchanged), `Residue` (Milestone 2's
`ResidueMutationIntent`, unchanged), and `ResidueWithoutCds`
(a `ResidueIntentWithoutCds`). The third is the same biological operation
missing exactly one field: the CDS the residue number counts over. It is
not a fourth mutation kind, and it is not a relaxed residue position:
D006 requires a residue position to identify its CDS, so Milestone 1's
`ResiduePosition` stays total and the "no CDS yet" state exists only in the
workflow's intent carrier.

Why the third inhabitant was added: the product brief's fail-closed example
("Mutate residue 42 to glycine" refused because multiple supported CDSs
could contain residue 42") is a required product behavior, and it had no
input form, because `ResidueMutationIntent` cannot be built without a CDS
and `named_cds()` therefore claimed a CDS for every residue intent.
`named_cds()` now returns `None` for the unbound form, so target resolution
is the step that decides whether a residue target is identified.

### Intent versus resolved target (D016, D003)

Intent representation and resolved target representation are separate. The
request never holds a resolved CDS; the gate's answer is carried as
`Q5WorkflowResult::target`. Binding happens at one step,
`ResidueIntentWithoutCds::bind(cds)`, which calls Milestone 1's
`ResiduePosition::create` and then Milestone 2's
`ResidueMutationIntent::create` and validates nothing of its own. The
residue number is validated where a CDS identifier exists, i.e. at bind
time, by Milestone 1's constructor, so a request for residue 0 is refused
with `InvalidResiduePosition` naming the CDS it was resolved against rather
than at request construction, where no CDS name exists to put in that
payload. The residue letters are validated eagerly at request construction
through Milestone 2's public `parse_residue`, so no residue-letter rule is
restated here.

### Resolution (D005, D020, Milestone 1)

`validate_input_scope` checks three things and nothing else: that the
artifact carries a CDS annotation at all (Milestone 1's
`MissingCdsAttribute`, `MalformedInput` — "a record with no CDS is a missing
attribute, not an empty CDS"); that every annotation names this artifact by
identity (Milestone 1's `SequenceReferenceMismatch`, `BiologicalValidation`,
so an annotation of another artifact is refused rather than dropped from the
candidate list); and that the request identifies exactly one annotation.
Sequence content is never read. Duplicate annotation identifiers are not
deduplicated, so two annotations sharing an identifier are ambiguous rather
than quietly reduced to one.

Zero, one, and several candidates are distinct: zero annotations is an
error, one resolves without the caller naming it, several with no claim or
a claim matching several is `NeedsClarification` with `candidates()`
listing every annotation identifier. A named identifier matching no
annotation is reported as a clarification, not an error, because the caller
can resolve it by naming one; the brief permits either, so this is recorded
as an adopted choice. The gate runs before the operation, so a request that
is both ambiguous and malformed reports the clarification.

### Nucleotide scope and containment (Milestone 2, open decision)

`nucleotide_scope` mirrors the request: a request that scoped a CDS is
checked against the annotation it named; a request that scoped nothing
gains no containment rule from the gate's resolution. This is Milestone 2's
operation with the scope the caller stated, kept rather than chosen.

Whether the workflow *ought* to impose containment once it has resolved a
target CDS is **not settled by the written contracts** and is recorded as an
open product decision in `docs/q5-sdm.md` §4, with the three alternatives and
their consequences. It was not implemented as though it were settled: no
containment rule was added, and no test asserts one of the alternatives that
adds it. What the test suite does encode is alternative 1: the test is named
`an_unscoped_nucleotide_edit_pins_alternative_one_of_the_open_containment_decision`
for exactly that reason, and §10 of the specification lists it among the tests
that currently pin an open answer. A residue mutation is different in kind — its
position names its CDS, so the resolved annotation is always its scope, and
`verify` is handed that annotation.

### Result invariants (D020, D007)

`Validated` implies exactly one resolved target whose `reference()` equals
the request's `template_reference()`; `NeedsClarification` implies no target
and a non-empty clarification. `mutation` is present in a run exactly when
the gate resolved a target. These hold module-locally, by construction
discipline: `Q5WorkflowResult` is a public struct whose fields are private and
which has no public constructor, so nothing outside `q5.rs` can build one at
all, invalid or valid, and `validate_input_scope` is the only producer inside
it. The type does not make an invalid pairing
unrepresentable, and that is stated rather than assumed.

`execute` no longer relies on the invariant by unwrapping: a `Validated`
gate result with no target is returned as `DomainError::MissingCdsAttribute`
("a resolved target CDS"). error.rs's contract is that every failure this
model produces is a `DomainError`, so a panic would break an established
contract; this is the smallest change that removes the panic path, and it
adds no new error variant.

### Verification trust boundary (D013, Milestone 2)

`verify` is called as it stands, with the same template and the same
resolved annotation the edit used. Its postconditions are listed in
`docs/q5-sdm.md` §8, separating what Milestone 1 and Milestone 2 already
enforce (artifact identity by name and digest, interval bounds and
structure, CDS and genetic-code and numbering rules, the expected-original
checks, codon-policy resolution, refusal of an edit that changes nothing)
from the independent checks verification performs (source identity, length
preservation, the exact or non-empty-subset difference between source and
edited artifacts, the claims read back out of both artifacts, and for
residue results the CDS-identifier agreement and the re-translation of both
artifacts over the same coordinates).

**No verification failure is reachable through `execute` today.** Every claim
the produced result records follows from the edit that produced it, and the
failure paths Milestone 2 exercises need a `MutationResult` that `execute`
does not produce. Nothing was manufactured to make one reachable, and
nothing was weakened because none is easy to reach. The step stays because
D013 requires separation, and verification re-derives claims from the
artifacts rather than repeating the generator's, which is what makes later
paths — candidate generation, a worker response (D017), a replayed result,
a multi-step edit — checkable at all.

### Naming and provenance (D007, D021, Milestone 2)

The edited artifact's display name is the caller's: `execute` takes
`result: Identifier`, as Milestone 2's fixture table supplies a `result_id`
per case, and this layer names nothing of its own. Identity is name *and*
digest, so no naming rule is needed. The mutation result is Milestone 2's
record unchanged; this layer adds only the workflow code-path name and
version (`Q5_WORKFLOW_NAME`, `Q5_WORKFLOW_VERSION`).

## Milestone 3a: separate-variant composition

This section records the choices made while adding the set API, in the same
style as above. It names the task-contract decision each entry implements
rather than restating it, and `docs/q5-sdm.md` §11 is the specification of
the behavior itself.

### What was added, and what was not touched

A new set API was added beside the existing one rather than widening it: the
single-request `Q5SdmRequest`, `Q5MutationIntent`, `ResidueIntentWithoutCds`,
`execute`, `validate_input_scope`, `Q5WorkflowResult`, and
`Q5WorkflowExecution` keep their types, signatures, and behavior unchanged,
as the task contract requires. The new types are `Q5Composition`,
`Q5Variant`, `Q5VariantSetRequest`, `Q5VariantResult`, and
`Q5VariantSetExecution`, plus `set_gate_request` and `set_target_claim` as
helpers and `execute_set` as the entry point. `run_resolved` is reused as it
stands, which is what keeps every biological rule Milestone 1's and Milestone
2's: the set layer re-derives no residue, codon, coordinate, or verification
claim.

### Composition mode (D-A, D-G)

`Q5VariantSetRequest` is one typed request carrying an explicit `Q5Composition`
and an ordered `Vec<Q5Variant>`, so a caller's loop over the old API is not
the representation. The mode is a required parameter of `create` and is never
inferred from the member count, so a one-member set still names a mode; the
one-member tests assert both that a `SeparateVariants` set of one runs and
that a `CombinedConstruct` set of one is still refused, so the mode cannot be
recovered from the count.

### Refusing a combined construct (D-B)

`CombinedConstruct` is representable and refused at construction, before any
member resolves a target or edits anything, and is never run as separate
variants. No overlap rule was written (D014 requires each design rule to name
a documented source and none is written for those).

The error variant is new: `DomainError::UnsupportedComposition`
(`UnsupportedCompositionProblem { composition, variant_count, reason }`),
categorized `Unsupported` in error.rs alongside
`UnsupportedMutationPolicy`. The choice and its rationale: no existing
variant fits. `UnsupportedMutationPolicy` carries `{cds, policy}` and its
message reads "requests codon policy {}; only default and supplied are
supported", so putting a composition mode in `policy` would misrepresent a
composition mode as a codon policy on a named CDS, which the task contract
forbids. `InvalidMutationRequest` would call a well-formed request that this
milestone declines to run malformed, which it is not. `UnsupportedComposition`
names the mode the caller asked for and how many members it carried, states
why the mode is out of scope, and its message ends "no variant was executed",
so the refusal says what was refused and that nothing ran. The minimal
changes are the payload struct, the enum member, the `Unsupported` arm of
`category()`, and the `describe()` arm; no other error-model refactoring was
done, and no exhaustive match needed changing beyond those.

### Set-level invariants at construction (D-D, D-E)

`Q5VariantSetRequest::create` checks, in order: nonempty member list
(`InvalidMutationRequest`, `MalformedInput`); the explicit mode; supported
mode (`UnsupportedComposition`); one template artifact for every member, by
Milestone 1's `interval::require_sequence` and its
`SequenceReferenceMismatch` (`BiologicalValidation`, name *and* digest, so a
same-named different artifact is refused rather than merged); and agreement of
explicit CDS claims, reported with Milestone 2's `ResiduePositionWrongCds`,
the same rule `Q5SdmRequest::create` already uses for a scope that
disagrees with a named residue position. A member that claims no CDS is not a
disagreement: it takes the agreed claim when the set runs, which is tested
explicitly rather than assumed.

### Execution (D-C, D-D)

The target resolves once: `set_gate_request` builds the request the gate is
asked with out of the shared template artifact plus the set's agreed CDS
claim, and `validate_input_scope` runs as it stands. The members' own
requests are not rewritten, so a member's own `scope` stays its own claim and
§4's open containment decision reads the same for a set member as for a
single request. A clarification returns `NeedsClarification` with no results
and no edited artifact for any member. Each member is then handed the
template its own request names, never a previous member's edited artifact,
so separate variants are independent; each member's supplied result name is
used; each member's edit and verification are `run_resolved`, i.e. Milestone
2's `apply_nucleotide` / `apply_residue` and `verify` called as they stand.
Members run in request order and a member that fails returns that original
`DomainError` unchanged, so no earlier member's results escape as a
successful partial set and no per-variant failure carrier exists.

### Result-identifier uniqueness (D-F)

Left open, as the contract requires: no uniqueness check, no duplicate-identifier
error, and no identifier length, case-folding, or uniqueness rule invented.
`two_identical_variants_with_one_result_name_are_representable` asserts only
that the pair is accepted and run; the identical artifact identity the two
results carry is Milestone 1's identity semantics for identical content,
not a claim that duplicate result names are legal or illegal as product
policy. The question is recorded as `docs/q5-sdm.md` §10 item 6.

### Deviation from the task contract's worked example

The contract's case 1 names `A42G` "with default codon selection producing
`GTT`". `GTT` is valine, not glycine, so it is not a codon for the residue
that request asks for. Milestone 2's `default_codon` takes the first codon
for the residue in the published table order, whose first glycine is `GGT`,
so the test asserts `GGT` and the single differing coordinate 124, which is
what the contract's own coordinate expectation describes. The fixture pair is
likewise `A42G` and `M1W` rather than the brief's `A42G and W87A`: the long
fixture is 129 nucleotides and 43 residues, so residue 87 does not exist in
it, and the residue-1 member carries the same shape. `M1W` writes `TGG` over
`ATG`, which differs at coordinates 0 and 1 only, because the third letter of
both codons is `G`.

## Tests

`crates/core/src/q5.rs` unit tests, each named for the rule it pins.
Pre-existing tests were kept; the residue helpers gained an unbound-CDS
form so both residue shapes are exercised. New or changed coverage:

* an unbound residue request against one annotation resolves to it, and the
  run edits and verifies against that annotation;
* an unbound residue request against several annotations is
  `NeedsClarification`, names the residue number in the clarification,
  lists every candidate, and performs no edit and produces no artifact —
  the generic resolution rule, on a fixture whose two annotations hold 10 and
  4 residues so that none of them could hold the requested residue 42;
* the product brief's fail-closed exchange over a fixture where its stated
  reason is true rather than only asserted: a 129-nucleotide template annotated
  with CDSs of 129 and 126 nucleotides, i.e. 43 and 42 residues, so both
  annotations could hold residue 42, and the gate still clarifies. The gate
  narrows no candidate because it reads no sequence content (§5's impl note);
  that is the current implementation, not a result any test observes, since
  no test offers several annotations of which only one could hold the residue
  (§10 item 2 records that gap). The same request scoped to one of the
  two resolves, edits residue 42 at the zero-based reference coordinates 123,
  124 and 125 the assertions name — one-based positions 124, 125 and 126 in the
  external format — and verifies;
* an unbound residue request with a scope that names one of several
  annotations resolves through the scope;
* a residue number Milestone 1 rejects (residue 0) is refused at bind time
  with `InvalidResiduePosition` (`MalformedInput`), before any edit;
* residue letters that are not standard residues are refused at request
  construction with `InvalidMutationRequest` (`MalformedInput`);
* `named_cds()` claims a CDS for a named residue intent and claims none for
  an unbound one, and an unbound residue intent names no artifact of its own;
* a `Validated` result carries a target and a clarification result does not,
  asserted for both residue shapes;
* a scoped nucleotide edit inside and outside the resolved CDS, and an
  unscoped nucleotide edit, which pins **Alternative 1** of the open containment
  decision in §4 as expected behavior — the test is named for that, and
  adopting another alternative would require changing it rather than only
  adding one (§10 of the specification lists every test that pins an open
  answer);
* an unbound residue request whose scope names one of several annotations runs
  the edit against the annotation the scope resolved, and produces the same
  edit the single-annotation case produces, so a scope and a resolved single
  annotation bind the same target;
* an unbound and a named residue intent whose CDS identifier matches no
  annotation are reported as the same target-not-found clarification as the
  nucleotide form, with the clarification naming what the artifact *is*
  annotated with;
* an unspecified codon policy on the unbound form is refused by the run with
  `UnspecifiedCodonPolicy` (`MalformedInput`), as it is on the named form, so
  resolving a target does not let the workflow choose a policy on the caller's
  behalf;
* a negative residue number is refused at bind time with `InvalidResiduePosition`
  (`MalformedInput`) naming the resolved CDS, as residue 0 is, so the workflow
  layer adds no residue-number rule of its own;
* caller-provided result naming and provenance, and the verification
  postconditions reached through the run.

Milestone 3a adds set coverage, listed in full in `docs/q5-sdm.md` §11: an
empty set refused at construction; a combined construct refused before any
member resolves or edits, and refused for one member as well as several;
separate variants independent and reported in request order, and the reverse
order reversing the results without changing an edit; members over different
and over same-named-different template artifacts refused by identity; members
claiming different CDSs refused, and a member's scope claim agreeing with
another member's intent claim; an unbound member taking the target the set
resolved; a clarifying set producing no results and no artifacts; a failing
member ending the set with its original error and no earlier result escaping;
the set request and its scope decision carried into the execution; the
flagship `A42G` and `M1W` pair over the long fixture, each editing only its
own codon; a member over a CDS Milestone 1 refuses, which ends the set before
any edit; a later member with a wrong expected residue failing the whole set;
a one-member set giving the same result as the single-request `execute`; two
identical variants with one result name being representable; residue 0 and a
negative residue refused by Milestone 1 at bind time; an unspecified codon
policy refused by Milestone 2; and a DNA-level no-op refused where the
member's request is built, before the set exists.

No test asserts a verification failure through `execute` or through
`execute_set`, because public APIs cannot produce one; Milestone 2's own
tests cover those paths where they are reachable.

## Known gaps at the milestone boundary

* Open product decisions, recorded rather than chosen: nucleotide
  containment semantics; candidate narrowing by residue feasibility; target-not-
  found as clarification versus structured error; whether the
  brief's "an identified target CDS" must be supplied by the caller; and
  whether a residue request whose expected original equals its replacement is
  a legal mutation. The last is Milestone 2's behavior reached unchanged —
  nothing rejects `expected_original == replacement` at construction, and the
  run proceeds when the codon the policy selects differs from the codon already
  at that residue, so a verified residue result can leave the residue sequence
  unchanged while the DNA changes. Only a DNA-level no-op is refused, with
  Milestone 2's `InvalidMutationRequest`. A nucleotide intent, by contrast,
  refuses an equal expected and replacement at construction. This slice
  changed neither; the decision is listed in `docs/q5-sdm.md` §10.
* Several of those open decisions are currently pinned by unit tests, so the
  test suite encodes one answer today; §10 of the specification names the
  tests for each, and changing a decision is a test change as well as a
  product change.
* Unwritten design rules block the rest of the Q5 boundary (D014 requires
  each to name a documented source): candidate generation, primer
  orientation, thermodynamic calculation method, sequence constraints,
  rejection criteria, warnings.
* Separate variants now run in one typed set request (§11 of the
  specification); combined constructs remain out of scope, refused at
  construction because composing mutations into one construct needs overlap
  rules no milestone has defined and D014 requires each design rule to name
  a documented source.
* Verification is not reachable-failure-tested through the workflow API;
  see above.
* The clarification text is human-facing only; classification is by state,
  variant, and `category()`.

## Cross-milestone clarification

No Milestone 1 or Milestone 2 source, test, or fixture was changed. The
mutation domain and the sequence domain are called as they stand. `crates/core`
gained no dependency. The pre-existing untracked research captures under
`tests/fixtures/nebasechanger-research/` were not read, moved, or overwritten;
they remain out of scope, and no comparison against them is claimed (D014).
