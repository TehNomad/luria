# Q5 site-directed mutagenesis: the workflow contract

Status: Contract for the implemented slice; open decisions are marked
Date: 2026-10-11; current-state note added 2026-10-16
Implements: [docs/decisions/](/decisions/) D005, D006, D007, D011, D013, D014, D016,
D020, D021; the Q5 workflow boundary in [docs/product/brief.md](/product/brief.md); the
mutation-domain contract in [docs/tasks/contracts/milestone-2-mutation.md](/tasks/contracts/milestone-2-mutation.md).
Code: `crates/core/src/q5.rs`.

**Current state (2026-10-16).** This document was written before primer
design existed and is kept as the workflow contract. Since then: D029–D033
define primer design (`primer_design.rs`, `primer.rs`), `tm.rs` implements
the Tm rules (D030), `q5_design.rs` wires the workflow to them (§12), and
Milestone 4 added deletions, insertions, unequal replacements and degenerate
expansion (D031–D033) to the intents `q5.rs` carries, with frameshift
warnings. Read statements below that "no design rule is written" or that
indels are out of scope as superseded. `q5.rs` itself still designs no
primer. Where this document and [docs/decisions/](/decisions/) disagree, the decisions
win.

This document is the specification of the part of the Q5 workflow that
exists. It does not describe primer design: the workflow boundary in the
product brief and D014 require every design rule — candidate generation,
primer orientation, sequence constraints, thermodynamic calculation method,
rejection criteria, warnings — to identify a documented source, and this
repository has written none of those down. Nothing in `q5.rs` designs a
primer, generates a candidate, or calculates a thermodynamic value, and this
document does not invent a rule to make it able to.

Milestone 3b wrote the contract that bounds such a design layer, and marked
which of those rules are settled, which are preferences, and which remain
unknown: [docs/specs/q5-primer-design.md](/specs/q5-primer-design.md), with its fixture set in
`tests/fixtures/q5-primer/` and its task contract in
[docs/tasks/contracts/milestone-3b-q5-primer-contract.md](/tasks/contracts/milestone-3b-q5-primer-contract.md). That document specifies a
layer `q5.rs` does not contain. It fixes what any design result must report —
strand, 5′→3′ orientation, spans against the pristine template, an overlap
that is counted position by position — and it records the thermodynamic rules
as **not established**, so the layer still cannot be implemented from it alone.
Nothing in `q5.rs` reads it.

**Amended 2026-10-14 (Milestone 3c).** A design layer now exists for the
substitution path — `crates/core/src/primer_design.rs`, implementing D029's
`luriaNative` policy — and the workflow reaches it by composition in
`crates/core/src/q5_design.rs`. Everything above still holds of `q5.rs`
itself: it designs nothing, reads no design document, and gained no field, no
function and no dependency. The wiring is specified in
[docs/tasks/contracts/milestone-3c-q5-design-wiring.md](/tasks/contracts/milestone-3c-q5-design-wiring.md) and summarised in §13; the Tm
rules remain unwritten, so no thermodynamic value is calculated anywhere.

Source labels used below and in the traceability table:

* **brief** — [docs/product/brief.md](/product/brief.md).
* **D0xx** — [docs/decisions/](/decisions/).
* **M2** — [docs/tasks/contracts/milestone-2-mutation.md](/tasks/contracts/milestone-2-mutation.md), the mutation-domain contract.
* **M1** — [docs/tasks/contracts/milestone-1-core-foundation.md](/tasks/contracts/milestone-1-core-foundation.md).
* **impl** — current behavior of `crates/core/src/q5.rs`, recorded here so a
  caller can tell implementation apart from requirement.
* **proposed** — a decision this task made, not settled by the sources above.
* **open** — a question the sources do not settle; no behavior is claimed.

Where two sources say different things, the difference is recorded as a
difference, not resolved into agreement. [docs/decisions/](/decisions/) wins over a
task record when they contradict (both records say so).

## 1. Inputs, and what the user has and has not identified

A Q5 SDM request is a typed capability invocation (D016, D014). The
product brief lists four required inputs for this workflow: the DNA
template, an identified target CDS, a numbering convention, and an
explicit mutation intent.

| Brief input | Where it lives | Authority |
| --- | --- | --- |
| DNA template | `Q5SdmRequest::template`, a `Sequence` artifact | brief, D007 |
| Mutation intent | `Q5SdmRequest::intent`, a `Q5MutationIntent` | brief, D011, D016 |
| Numbering convention | not a request field: it belongs to the annotation, carried as `Cds::numbering` (M1's `Numbering`) | brief, D006, M1 |
| Identified target CDS | either half of the request can supply it: `Q5SdmRequest::scope` (an `Identifier`) or the residue intent's own position | brief, D006, D011; **open** as to whether "identified" must be supplied by the caller or may be resolved by the workflow — see §5 and §10 |

What the caller has **not** identified, and how that is represented:

* A nucleotide request with no `scope`: the DNA edit addresses the reference
  sequence, so it states no CDS at all. **impl**, and the reason the field is
  optional: refusing to build a record without a named CDS would make the
  ambiguity case unrepresentable rather than clarifiable (D020 gives
  ambiguity a lifecycle state).
* A residue request whose CDS the caller did not name: represented by
  `Q5MutationIntent::ResidueWithoutCds(ResidueIntentWithoutCds)`. This is
  the case the product brief's fail-closed example needs ("Mutate residue 42
  to glycine"), and before this task it had **no representation** — see §3.
* A residue request whose CDS the caller did name:
  `Q5MutationIntent::Residue(ResidueMutationIntent)`, M2's intent unchanged,
  whose `ResiduePosition` carries a CDS `Identifier`.

## 2. Intent representation versus resolved target representation

These are two things and the model keeps them apart (D016, D003).

* **Intent** — what the caller asked for, before anything on the artifact
  was consulted. `Q5MutationIntent`: a DNA substitution (M2's
  `NucleotideMutationIntent`), a residue substitution against a named CDS
  (M2's `ResidueMutationIntent`), or a residue substitution with no CDS
  named (`ResidueIntentWithoutCds`).
* **Resolved target** — the one `Cds` the request is about, decided by
  target resolution against the template's annotations, and carried as
  `Q5WorkflowResult::target`. It is not a field of the request and the
  request is never rewritten to hold it.

Rules:

* A residue position that has been resolved **must** name a CDS. D006 says a
  residue position must identify its CDS and numbering origin, so
  `ResiduePosition::cds` stays total and Milestone 1's type is not relaxed.
  The "no CDS yet" state exists only one level up, in the workflow's intent
  carrier. **impl** for M1/M2, **proposed** for where the new carrier sits.
* Binding is one step, and it composes: `ResidueIntentWithoutCds::bind(cds)`
  calls M1's `ResiduePosition::create` and M2's
  `ResidueMutationIntent::create`. It validates nothing of its own, so no
  residue-number or residue-letter rule is restated at the workflow layer.
* A residue intent names no artifact of its own (`source_reference()` is
  `None`): its CDS half is an `Identifier`, and D007 keeps artifacts apart by
  name *and* digest, which a bare identifier carries neither half of. Its
  artifact binding comes from the annotation the request resolves to, which
  the gate checks against the template. **impl**, unchanged.
* A nucleotide intent does claim an artifact, and `Q5SdmRequest::create`
  binds that claim to the template with M1's `interval::require_sequence`.
  **impl**, M1, D007.

## 3. The residue-target gap, and what closed it

The product brief states, under "Fail-closed behavior", the exchange:

> User: "Mutate residue 42 to glycine."
> Luria: Status: needs_clarification / Reason: multiple supported CDSs could
> contain residue 42 / No sequence calculation performed

and says this "is part of the product, not merely an error-handling detail".
D005 lists "multiple possible target CDSs" among the ambiguities that must
prefer clarification over silent inference. **brief, D005: the behavior is
required.**

It was not representable. `ResiduePosition::create` requires a CDS
`Identifier`, so `ResidueMutationIntent` cannot be built without a CDS, and
`Q5MutationIntent::named_cds()` returned a CDS claim for every residue
intent. The request could therefore only ever express a residue target that
was already identified, and the brief's example had no input form
(**impl**, recorded before this task).

Closed by adding the intent carrier `ResidueIntentWithoutCds` and the
variant `Q5MutationIntent::ResidueWithoutCds`, which carries a residue
number, an expected residue, a replacement residue, and a codon policy, and
carries no CDS. `named_cds()` now returns `None` for it, so target
resolution is the step that decides whether the residue target is
identified. **proposed representation; required behavior.**

The fixture in which the brief's reason is true of the annotations is the one
over `long_template` in the tests: a 129-nucleotide artifact annotated with CDSs
of 129 and 126 nucleotides, i.e. 43 and 42 residues, so both could hold residue
42. The fixture in `an_unbound_residue_request_against_several_cdss_needs_clarification`
and `a_residue_request_without_a_cds_never_selects_one_and_never_edits` is the
30-nucleotide template annotated with CDSs of 30 and 12 nucleotides, i.e. 10 and
4 residues, so **no** annotation there could hold residue 42: those two tests
exercise the same resolution rule without the brief's premise, and the tests
named `..._over_two_cdss_that_could_hold_it_...` and
`..._scoped_to_one_of_two_cdss_that_could_hold_it_...` exercise it with it.

Not closed, and deliberately not closed:

* M1's `ResiduePosition` was not made optional-CDS. D006 requires a residue
  position to identify its CDS, and relaxing it would change M1 and M2
  semantics and every residue call site in both prior milestones.
* The candidate set is **not** narrowed by asking which annotated CDS could
  actually hold residue 42. The gate reads annotation metadata and artifact
  identity, never sequence content (**impl**). The brief's wording "could
  contain residue 42" suggests such narrowing; no source states the rule, so
  the gate over-reports candidates and can answer a resolvable case with a
  clarification. That is fail-closed in D005's direction, but see §10.

## 4. Nucleotide scope and containment semantics

M2 establishes: a nucleotide mutation may optionally be scoped to a CDS, the
coordinate must then lie inside that CDS's intervals or the operation fails
with `TargetOutsideFeature`, and scoping is a parameter of the operation,
not a field of the intent — "the DNA edit is a DNA edit, and the scope says
which annotation it must fall inside". M2's fixture `nuc-a-unscoped` pins
that an unscoped nucleotide edit is a legal mutation-domain operation.
**M2.**

The Q5 workflow resolves a target CDS for every request that proceeds.
Whether that resolution becomes a containment constraint on a DNA edit the
request left unscoped is **not settled by the sources**, and the
alternatives are not equivalent.

**Alternative 1 — explicit-scope semantics (current behavior).**
`nucleotide_scope` mirrors the request: a request that scoped a CDS is
checked against the annotation it named; a request that scoped nothing
gains no containment rule from the gate's resolution.

* User-visible: an unscoped DNA edit is applied wherever it is valid on the
  artifact, including outside the resolved CDS. The run still reports which
  CDS it resolved and the coordinates it wrote, so a reviewer can see an
  edit that does not touch the target — but nothing refuses it.
* Compatibility: exactly M2's operation, with the scope the caller stated.
  No rule is added or removed at the workflow layer.
* Safety: the workflow can produce a verified DNA edit that has no effect on
  the target CDS's translation. M2's verification of a nucleotide result
  checks DNA claims only, so such an edit verifies.

**Alternative 2 — resolved-target semantics.** Once the workflow resolves a
target CDS, an unscoped DNA edit must lie inside it.

* User-visible: an unscoped DNA edit outside the resolved CDS is refused with
  M2's `TargetOutsideFeature` (`BiologicalValidation`); an unscoped edit
  inside it proceeds. A caller who wants a DNA edit outside the CDS has no
  Q5 request for it.
* Compatibility: the workflow adds a containment rule M2 does not have. That
  is allowed by D014 ("the workflow defines ... sequence constraints"), but
  D014 also requires each design rule to name a documented source, and none
  is written for this constraint.
* Safety: every DNA edit a Q5 run performs touches the CDS the run names, so
  a verified edit is at least relevant to the target.

**Alternative 3 — warn rather than reject.** An unscoped DNA edit outside the
resolved CDS proceeds and carries a warning. D014 and the brief list warnings
as part of the workflow, but no source says an out-of-CDS DNA edit is one, so
this is recorded as a candidate refinement, not as an interpretation the
sources support.

Recommendation, labeled as a recommendation and not as a requirement:
**alternative 1**, because (a) it is the request's own claim, kept rather
than chosen, so the workflow adds no assumption the caller did not state,
which D005's "execution is permitted only when all required inputs and
assumptions are explicit" reads against; (b) alternative 2's benefit is a
sequence constraint that D014 requires a documented source for, and no such
source exists; (c) alternative 1 leaves M2's operation reachable exactly as
M2 specified it. The residual risk of alternative 1 — a verified edit that
does not touch the resolved CDS — is the reason this stays an open item
rather than a settled one.

Authority needed to settle it: a product decision on whether the Q5 SDM
workflow accepts DNA edits that fall outside its target CDS. Until that
decision exists, this document records alternative 1 as **impl**. No test
asserts alternative 2's behavior, and one test —
`an_unscoped_nucleotide_edit_outside_the_resolved_cds_proceeds`
— asserts alternative 1's as expected behavior, so the test suite currently
encodes the open answer rather than leaving it uncommitted: adopting
alternative 2 or 3 would require changing that test, not only adding one.
§10 lists this as an open item together with the test that pins it.

## 5. Target resolution: zero, one, or multiple candidate CDSs

Target resolution is `validate_input_scope(request, annotations)`. It reads
artifact identity and annotation metadata only; it never reads sequence
content (**impl**). It checks three things, in this order:

1. `annotations` is empty → `DomainError::MissingCdsAttribute`
   (`MalformedInput`). M1's rule: "a record with no CDS is a missing
   attribute, not an empty CDS". **M1, impl.**
2. Every annotation must name the template by identity (name *and* digest),
   or `DomainError::SequenceReferenceMismatch` (`BiologicalValidation`). An
   annotation of a different artifact is refused, not dropped from the
   candidate list. **M1, D007, impl.**
3. The request must identify exactly one annotation. The identifier it
   claims is `target_cds_name()`: the intent's claim if it has one, else
   `scope`.

| Candidates | Request claims | Result | Authority |
| --- | --- | --- | --- |
| exactly one | any, or matching it | `Validated`, `target = that Cds` | impl |
| several | none | `NeedsClarification`, no target, `candidates()` lists every annotation id | brief, D005, D020, impl |
| several | one identifier, matching one | `Validated`, `target = that Cds` | impl |
| several | one identifier, matching none | `NeedsClarification`, no target, clarification names what is available | impl; brief permits "clarification state or structured error" |
| several sharing one identifier | that identifier | `NeedsClarification` — duplicates are not deduplicated | impl |
| zero annotations | any | `MissingCdsAttribute` error, not a clarification | M1, impl |

An unbound residue request against exactly one annotation resolves to it:
the artifact identifies the target, which is how the brief's "an identified
target CDS" is satisfied without the caller naming it. **impl** (the same
rule already applies to nucleotide requests), **proposed** for the residue
case, since no residue-without-CDS request existed before.

Ordering: validation happens at three stages, and a request is refused at the
earliest one it reaches.

1. **Request construction** — `ResidueIntentWithoutCds::create` (the residue
   letters, through M2's `parse_residue`) and then `Q5SdmRequest::create` (the
   intent's artifact claim against the template, and a scope that disagrees
   with a named residue position). A request refused here never reaches the
   gate, so it cannot be reported as a clarification even when it would also
   be ambiguous.
2. **Target resolution** — `validate_input_scope`: empty annotations
   (`MissingCdsAttribute`), an annotation of another artifact
   (`SequenceReferenceMismatch`), then the candidate count. Sequence content
   is not read.
3. **Operation time** — `execute`, after the gate resolved one annotation, in
   the order the operation takes its checks. A residue intent: `bind` puts the
   resolved CDS into the residue position through M1's and M2's own
   constructors, so a residue number M1 rejects is `InvalidResiduePosition`
   (`MalformedInput`); artifact identity; the CDS translates (an incomplete
   CDS, an unsupported genetic code or numbering scheme and a non-triplet
   length all fail here); the residue number is checked against that
   translation, and a number beyond it is `ResiduePositionOutsideCds`
   (`BiologicalValidation`) — the feasibility question, asked only here and
   never at the gate; the expected-original residue is checked against the
   translation (`ExpectedResidueMismatch`); the codon policy is resolved
   (`CodonSelectionUnavailable` for `Unspecified`); the codon the policy
   selected is compared with the codon already at that residue, and equal is
   M2's `InvalidMutationRequest`; only then are the codon's reference
   coordinates calculated (`CodonPositionOutsideSequence`). A nucleotide
   intent, in the order M2's `apply_nucleotide` takes: artifact identity, the
   coordinate in range, containment against the resolved CDS but only when
   the request scoped one, then the expected-original nucleotide
   (`ExpectedNucleotideMismatch`) — a nucleotide no-op was already refused at
   construction. M2's verification then checks whatever the edit wrote.

So a request that is ambiguous *and* malformed in stage 3 reports the
clarification, not the malformed-input error, while a request malformed at
stage 1 is refused before the gate can call it ambiguous. **impl.**

## 6. Invalid input, target-not-found, and needs-clarification

The three outcomes are kept distinct (D020, D005, and error.rs's three
categories):

* **Invalid input** — a `DomainError`. The request could not be represented,
  or the artifact is missing an attribute, or two halves of the request
  contradict each other: no CDS annotation at all (`MissingCdsAttribute`,
  `MalformedInput`); an annotation of another artifact
  (`SequenceReferenceMismatch`, `BiologicalValidation`); a scope and a
  residue position naming different CDSs (`ResiduePositionWrongCds`,
  refused by `Q5SdmRequest::create`); an intent claiming another artifact
  (same, at `create`); a residue letter that is not a standard residue
  (`InvalidMutationRequest`, at `ResidueIntentWithoutCds::create` through
  M2's `parse_residue`).
* **Target-not-found** — a named identifier matching no annotation. The
  current contract reports it as `NeedsClarification` with `candidates()`
  listing what the artifact *is* annotated with, because the caller can
  resolve it by naming one. **impl**; the brief permits a structured error
  here too, so this is an adopted choice, not a derived one.
* **Needs-clarification** — ambiguity the caller can resolve by choosing:
  several annotations and no claim, or a claim matching several. D005
  forbids choosing; D020 gives it a lifecycle state rather than an error.

Clarification text is for human reporting only; classification is by state
and by `DomainError` variant and `category()` (error.rs).

## 7. Workflow states and their invariants

`Q5WorkflowState` is the workflow-level subset of D020's lifecycle reachable
today: `NeedsClarification`, `Validated`. `proposed` is the state of a
`luria q5 interpret` document (Milestone 6, D044);
`calculated` and `verified` are M2's `MutationState`; `failed` is carried by
a `DomainError`. **impl.**

Invariants of `Q5WorkflowResult`:

* `Validated` ⇒ `target` is exactly one `Cds`, and that CDS's `reference()`
  equals the request's `template_reference()`.
* `NeedsClarification` ⇒ `target` is `None`, and `clarifications` is
  non-empty.
* `candidates()` lists every annotation identifier considered, in both
  states.
* `request` is carried whole, so what the application understood stays
  inspectable alongside what it decided (D004, D016).

These hold **module-locally, by construction discipline**: `Q5WorkflowResult`
is a public struct whose fields are private and which has no public
constructor, so no caller outside `q5.rs` can build one at all, invalid or
valid; inside the module, `validate_input_scope` is the only producer. The
type does not make a `Validated` result with no target unrepresentable —
anything inside the module can set the fields. That is stated rather than
assumed, and `execute` no longer relies on it by unwrapping: a `Validated`
gate result with no target is returned as `DomainError::MissingCdsAttribute`
("a resolved target CDS") instead of panicking, because error.rs's contract
is that every failure this model produces is a `DomainError`. **impl, plus
this task's guard.**

`Q5WorkflowExecution` invariants:

* `mutation` is present exactly when the gate resolved a target. A
  clarification run performs no edit, produces no artifact, and holds no
  mutation claim (D005: "No sequence calculation performed").
* `is_verified()` is true only when the carried `MutationResult` is in
  M2's `MutationState::Verified`, which only M2's `verify` produces. A run
  that performed no edit is not verified; a run that only calculated one is
  not verified either (D020).
* `template_reference()` reports the artifact the run was about, by identity,
  in every state (D007).

## 8. Verification postconditions and error behavior

`execute` composes M2's vertical slice (D015): gate → `apply_nucleotide` or
`apply_residue` → `verify` → structured result. It re-derives nothing.

Guarantees already enforced by lower layers, reused rather than restated:

* artifact identity by name, digest, and topology: M1's `interval::require_sequence`,
  applied by `Q5SdmRequest::create`, by `validate_input_scope` to every
  annotation, and inside both M2 operations;
* interval bounds, ordering, and structure: M1's `interval::create`,
  `validate_against`, `validate_structure`, reached when a `Cds` and a
  `NucleotideInterval` are built;
* CDS completeness, genetic code, numbering, length-divisible-by-three,
  ambiguous codons, internal stops: M1's `translate`, reached by
  `apply_residue` and by `verify`;
* expected-original checks, codon-policy resolution, and the written-back
  codon at reference coordinates: M2's `apply_residue` / `apply_nucleotide`.

Independent checks `verify` performs (D013), restated here as the workflow's
trust boundary rather than re-implemented:

* sequence identity: the result's source matches the artifact handed to it,
  by name, digest, and topology;
* length, by kind: a substitution preserves it; a deletion of `w` bases
  shortens the artifact by exactly `w`; an insertion of `k` bases lengthens it
  by exactly `k`; a replacement of `w` bases by `m` leaves exactly
  `n − w + m`;
* deletion, insertion and replacement results: the edited artifact is rebuilt
  from the source and the record's own fields (region, point, inserted or
  replacement text) and must equal the recorded edited artifact; the source
  must hold the recorded expected and actual bases (the deleted or replaced
  text, or the bases on each side of an insertion point);
* nucleotide results: the set of positions where source and edited differ is
  **exactly** the recorded target coordinates; the source holds the recorded
  expected nucleotide there and the edited artifact holds the recorded
  replacement;
* residue results: that set is **non-empty** and a subset of the recorded
  codon coordinates (a replacement triplet may share letters with the
  triplet it replaces), so an edit that changed nothing is refused here, not
  at the edit step;
* residue results: the CDS handed to `verify` has the identifier the result
  names, so the residue claims are checked against the annotation the result
  was actually applied to (through `execute` that agreement is the gate's own
  doing, so this check is meaningful for a result that arrives from elsewhere);
* residue results: source and edited are re-translated over the same CDS
  coordinates rebuilt against the edited artifact; the residue sequences
  agree at every residue except the requested one; the source holds the
  recorded expected residue there and the edited artifact holds the recorded
  replacement; the codon read from the edited CDS at that residue is the
  recorded codon.

One further limitation of that boundary, recorded rather than changed: `verify`'s
`Option<Cds>` parameter is consulted on the residue branch only. For a nucleotide
result the branch reads the two artifacts and the recorded coordinate, so the
value `execute` passes — `dna_scope`, the request's own scope — changes no check.
It is passed rather than `None` so that the value stays the request's claim
rather than a decision made at this layer, and no verification behavior was
changed to make the inertness obvious. The consequence is that a nucleotide
result carries no CDS claim for verification to check against, which is what
§4's open containment decision is about: alternative 2 would constrain the edit
at the operation, not give `verify` a nucleotide CDS claim to check.

A second limitation, recorded 2026-10-16 and open: `verify` re-translates
only residue results. A DNA-level substitution, deletion, insertion or
replacement inside a CDS is verified as a DNA edit, and its effect on the
translation is not checked. A scoped substitution that writes a premature
stop (`TGG` → `TAG`) comes back `Verified` with no warning. D013 asks for
"the resulting translation where applicable"; whether this applies to
DNA-level edits is an owner decision
([docs/tasks/reviews/review-post-m4.md](/tasks/reviews/review-post-m4.md) A5).

Verification does not re-derive the residue-to-coordinate mapping: that is
the generator's responsibility, checked by the residue and codon claims
(**M2**, deliberate).

Error behavior: a `DomainError` from the edit or the verification pass is
returned as it stands, with M2's variants and categories.

Reachability: **no verification failure is reachable through `execute`
today.** `execute` hands the same template, the same resolved annotation,
and the result the edit just produced to `verify`, and every claim that
result records follows from that edit. The failure paths M2 exercises
(a result claiming a coordinate it did not write, a residue result verified
without the CDS it names, an empty difference) require a `MutationResult`
that `execute` does not produce, and M2 reaches them by constructing such a
result directly. This task does not manufacture a reachable failure to
raise coverage, and does not weaken `verify` because none is easy to reach.

The contract stays meaningful because D013's requirement is about
separation, not about a currently-reachable failure: verification re-derives
claims from the artifacts instead of repeating the generator's, so it
becomes load-bearing when a result arrives from somewhere other than the
call that produced it — a candidate-generation step, a worker response
(D017), a replayed or deserialized result, or a multi-step edit. Keeping the
step is what makes those later paths checkable at all.

## 9. Provenance and naming of the edited sequence

* The edited artifact's display name is the caller's: `execute` takes
  `result: Identifier`, exactly as M2's fixture table supplies a `result_id`
  per case. The workflow names nothing of its own. **impl, M2.**
* Identity is name *and* content digest (D007), so no naming rule is needed
  and none is invented; a substitution preserves length, so the source keeps
  its digest and the edited artifact is a different artifact by name and
  digest.
* The result carries M2's provenance unchanged: source `SequenceReference`,
  the edited artifact and its reference, the target, expected and actual
  originals, the replacement, and for residue results the CDS identifier,
  original codon, selected codon, policy, and codon reference coordinates
  (D021). This layer adds nothing to that record.
* `Q5_WORKFLOW_NAME` / `Q5_WORKFLOW_VERSION` identify the workflow code path
  (D021); M2's `MUTATION_ALGORITHM_NAME` / `MUTATION_ALGORITHM_VERSION`
  identify the edit path.
* Computational validation is never experimental validation (D020): no
  result reports experimental status, and nothing here supplies experimental
  evidence.

## 10. Known limitations, and decisions still required

**Decided by the owner, 2026-10-14.** All six decisions below are now settled.
Items 1–5 keep the behaviour already implemented, so no code changes; the
tests listed under each are now the specification of an owner decision rather
than of an open question. Item 6 changed behaviour, and is implemented.

| # | Decision | Owner's answer |
| --- | --- | --- |
| 1 | An unscoped DNA edit outside the resolved CDS | **Proceeds** (alternative 1). Editing a promoter or UTR base is legitimate. |
| 2 | Narrowing candidate CDSs by whether they can hold the residue | **No narrowing.** Several annotations still ask the caller; choosing by length would be a guess (D005). |
| 3 | A named CDS that matches no annotation | **Clarification**, listing the annotations that exist. Most likely a typo. |
| 4 | One annotation, and the caller names none | **Used.** There is nothing ambiguous to resolve. |
| 5 | A residue request whose expected original equals its replacement | **Legal** when the codon changes: this is how a silent mutation is made, e.g. to remove a restriction site. A DNA-level no-op is still refused. |
| 6 | Two members of one variant set sharing a result identifier | **Refused** at set construction (`InvalidMutationRequest`), compared exactly. |

The text of each item is kept below as the record of the options that were
weighed. Where an item says "unsettled" or "open", read it as superseded by the
table above. The containment test was renamed on 2026-10-14 to
`an_unscoped_nucleotide_edit_outside_the_resolved_cds_proceeds`, so its name no
longer calls the decision open. Item 6 was implemented the same day:
`two_variants_sharing_a_result_name_are_refused`.

The decisions as they stood before 2026-10-14, each with the tests that encode
one answer:

1. **Nucleotide containment (§4)** — alternative 1 vs 2 vs 3. Unsettled;
   behavior is **impl**. Encoded by
   `an_unscoped_nucleotide_edit_outside_the_resolved_cds_proceeds`,
   which asserts alternative 1 as expected behavior: an unscoped DNA edit
   outside the resolved CDS proceeds and verifies.
2. **Candidate narrowing by residue feasibility (§3, §5)** — whether
   resolution excludes CDSs too short to hold the requested residue.
   Unsettled; the gate reads no sequence content, so it reports every
   annotation as a candidate today. Asserted: an unbound residue request
   against one annotation resolves to it whether or not that annotation
   could hold the requested residue — residue 2 over a 10-residue CDS
   (`an_unbound_residue_request_resolves_when_the_artifact_has_one_cds`) and
   residue 42 over a 4-residue one
   (`an_infeasible_residue_against_one_annotation_resolves_and_is_refused_downstream`,
   which resolves and is then refused downstream by
   `ResiduePositionOutsideCds`); and against several annotations it
   clarifies whether none of them could hold the residue
   (`an_unbound_residue_request_against_several_cdss_needs_clarification`) or
   all of them could
   (`an_unbound_residue_42_request_over_two_cdss_that_could_hold_it_needs_clarification`).
   Not asserted: no test offers a mixed set, several annotations of which
   only one could hold the residue, so "resolution does not narrow
   candidates" is an inference from §5's impl note that the gate never reads
   sequence content, not a result any test observes.
3. **Target-not-found as clarification vs structured error (§6)** — the
   brief permits both; the current contract chooses clarification. Encoded by
   `a_request_naming_an_unannotated_cds_needs_clarification` (nucleotide form),
   `an_unbound_residue_request_naming_an_unannotated_cds_needs_clarification`,
   and `a_named_residue_intent_naming_an_unannotated_cds_needs_clarification`.
4. **Whether the brief's "an identified target CDS" must be supplied by the
   caller** — today a single annotation identifies it without the caller
   naming it (§5). Encoded by the unbound-residue requests that carry no
   claim at all — no CDS in the intent and no `scope` — against an artifact
   with one annotation:
   `an_unbound_residue_request_resolves_when_the_artifact_has_one_cds` (asserts
   `Validated` with that annotation as the target) and
   `an_unbound_residue_request_binds_to_the_cds_the_gate_resolved` (asserts
   the operation is bound to that annotation). Tests whose residue intent
   names a CDS are not cited here: in them the caller supplied the target,
   so they say nothing about the no-claim case.
5. **Whether a residue request whose expected original equals its replacement
   is a legal mutation** — not settled by the sources, and not settled here.
   Today it is legal: nothing rejects `expected_original == replacement` at
   construction, and the run proceeds whenever the codon the policy selects
   differs from the codon already at that residue, so a verified residue result
   can leave the residue sequence unchanged. Only a DNA-level no-op — the
   selected codon equal to the codon already there — is refused, with M2's
   `InvalidMutationRequest`. This is M2's behavior reached unchanged through
   the workflow, not a rule this slice added; a nucleotide intent, by contrast,
   refuses `expected_original == replacement` at construction. Authority needed:
   a product decision on whether D011's "expected original and replacement" must
   name two different residues.
6. **Result-identifier uniqueness within a variant set (§11)** — whether two
   members of one set may name the same result identifier. Unsettled, and
   deliberately not settled here: no uniqueness rule is enforced, no
   duplicate-identifier error exists, and no identifier length or case-folding
   rule was invented, because D007 makes artifact identity name *and* digest
   and no source states an identifier rule for a set. Encoded only as
   representability: `two_identical_variants_with_one_result_name_are_representable`
   asserts that the set API rejects the pair neither at construction nor at
   run time, and asserts nothing about the two results' artifact identity
   beyond the fact that Milestone 1's identity semantics make identical
   intents produce references that compare equal. Neither duplicate results
   nor distinct results are claimed as product policy. Authority needed: a
   product decision on whether a set's result identifiers must be distinct.

Out of scope when this section was written, and now specified elsewhere:
candidate generation, primer orientation, thermodynamic calculation method,
sequence constraints and rejection criteria (D029–D033,
[docs/specs/q5-primer-design.md](/specs/q5-primer-design.md)), and edit warnings (frameshift, D031–D033).
Comparison with external tools (NEBaseChanger fixtures under
`tests/fixtures/nebasechanger-research/`) is development validation, not
proof of experimental success (D014), and no comparison is claimed here.

Also out of scope: combined-construct composition (§11 keeps it
representable and refuses it for exactly this reason — the overlap rules
composing mutations into one construct needs are unwritten, and D014
requires every design rule to name a source). Separate-variant composition
is no longer out of scope: §11 implements it. Insertions, deletions and
unequal replacements are no longer out of scope either (D031–D033). Still out
of scope: RNA alphabets, compound features, incomplete CDSs, non-standard genetic
codes, non-`FromCdsStart` numbering, natural language (Milestone 6), CLI
(Milestone 5), persistence (D034 sets the milestone order).

## 11. Separate-variant composition (Milestone 3a)

Separate variants are one typed request, not a caller's loop over §5's
single-request API: the set-level invariants below are what a loop cannot
establish, and the composition mode is what a loop would silently drop. The
set layer adds representation and a run; it adds no biological rule, and
every residue, codon, coordinate, numbering and verification check is the
single-request path's, reached once per member.

**The types.** `Q5Composition` is `SeparateVariants` or `CombinedConstruct`,
an explicit field and never inferred from the member count, so a one-member
set still names a mode. `Q5Variant` pairs a `Q5SdmRequest` (§2's request,
unchanged) with the caller's result `Identifier`; that intent-to-name pairing
lives only there — the set keeps no parallel list of names, generates none,
and derives none from mutation text. `Q5VariantSetRequest` is the composition
plus an ordered `Vec<Q5Variant>`, built by
`Q5VariantSetRequest::create(composition, variants)`; its accessors expose the
mode, the members in request order, the one template artifact every member
addresses by identity, and the set's agreed CDS claim. `Q5VariantSetExecution`
carries the set request, the one `Q5WorkflowResult` the gate produced, and one
`Q5VariantResult` per member in request order. `execute_set(set, annotations)`
is the set-level entry point; `execute(request, annotations, result)` is
unchanged and remains the single-request path.

**Construction refuses, before any member resolves a target or edits
anything**, in `create`'s order:

| Condition | Result | Authority |
| --- | --- | --- |
| no members | `InvalidMutationRequest` (`MalformedInput`) | task contract, impl |
| `CombinedConstruct` | `UnsupportedComposition` (`Unsupported`) | task contract, D014, impl |
| members address different template artifacts, by name *or* digest | `SequenceReferenceMismatch` (`BiologicalValidation`) | task contract, M1, D007 |
| members make different explicit CDS claims | `ResiduePositionWrongCds` (`BiologicalValidation`) | task contract, M2's rule `Q5SdmRequest::create` already uses |

`CombinedConstruct` is representable so a caller can ask for it and be refused
rather than be quietly reinterpreted: composing mutations into one construct
needs overlap rules no milestone has defined, and executing one as separate
variants would answer a different question than the one the caller asked. It
is never executed as separate variants, and no overlap rule was written.
A member that claims no CDS is not a disagreement with the agreed claim; it
takes that claim when the set runs.

**Execution.** The target resolves once, through `validate_input_scope` as it
stands, asked with the set's agreed CDS claim rather than once per member, so
one annotation is applied to every member. A set that needs clarification
returns `NeedsClarification` with no results and no edited artifact for any
member (D005, D020). For `SeparateVariants`, each member runs the same
`run_resolved` the single `execute` runs — M2's `apply_nucleotide` /
`apply_residue` and `verify` — against the template its own request names,
never a previous member's edited artifact, so variant 2 is not variant 1 plus
another edit and the variants are independent. Members run in request order
and a member that fails returns that original `DomainError`, Milestone 1's or
Milestone 2's variant and category unchanged, so no earlier member's results
escape as a successful partial set and no per-variant failure carrier exists.
Results are returned in request order only when every member succeeded.

**What a set does not decide.** No candidate narrowing by residue
feasibility (§10 item 2 stays open); no containment rule for a member's DNA
edit, so §4's open decision reads the same for a set member as for a single
request; result identifiers must be distinct (§10 item 6, decided 2026-10-14).

**Tests** (`crates/core/src/q5.rs`, each named for the rule it pins):
`an_empty_variant_set_is_refused_at_construction`,
`a_combined_construct_is_refused_before_any_member_resolves_or_edits`,
`a_one_member_combined_construct_is_refused_so_the_mode_is_not_counted`,
`separate_variants_are_independent_and_reported_in_request_order`,
`a_one_member_separate_variant_set_reports_one_result`,
`reversing_the_members_reverses_the_results_without_changing_an_edit`,
`members_over_different_template_artifacts_are_refused_at_construction`,
`a_member_over_a_same_named_different_artifact_is_refused_by_digest`,
`members_that_claim_different_cdss_are_refused_at_construction`,
`a_member_s_claim_from_its_scope_agrees_with_another_member_s_intent_claim`,
`an_unbound_member_takes_the_target_the_set_resolved`,
`a_set_that_needs_clarification_returns_no_results_and_edits_nothing`,
`a_failing_member_ends_the_set_and_no_earlier_result_escapes`,
`the_set_request_and_its_scope_decision_are_carried_into_the_execution`,
`two_separate_variants_over_the_long_fixture_each_edit_only_their_own_codon`,
`a_member_over_a_cds_milestone_one_refuses_ends_the_set_before_any_edit`,
`a_later_member_with_a_wrong_expected_residue_fails_the_whole_set`,
`a_one_member_set_produces_the_same_result_as_the_single_request_path`,
`two_variants_sharing_a_result_name_are_refused`,
`result_names_are_compared_exactly`,
`a_member_with_residue_zero_is_refused_by_milestone_one_at_bind_time`,
`a_member_with_a_negative_residue_is_refused_by_milestone_one_at_bind_time`,
`a_member_that_asks_for_an_unspecified_codon_policy_is_refused_by_milestone_two`,
`a_member_whose_dna_edit_writes_no_change_is_refused_before_the_set_exists`.

## 12. Reaching the design layer (Milestone 3c)

`q5_design.rs` composes `q5.rs` and `primer_design.rs`:
`design_run(execution, request)` designs the pair for one run, and
`design_set(set_execution, request)` designs one pair per set member. It holds
no biological rule: the template it passes to the design layer is the one the
run's own request names, the `MutationResult` is the one the run produced, and
every design-layer `DomainError` is returned with its own variant and
category. No error variant was added.

Two behaviours are owner decisions of 2026-10-14, not deductions from this
contract:

- **A clarification designs nothing, and that is an `Ok` outcome.** A
  clarification run holds no mutation, so it gets no design and no field of
  the design request is read. The clarification survives intact (D005).
- **A set's design failure is atomic.** Members are designed in request
  order, and the first member that cannot be designed returns its error, so
  no partial set escapes as a successful one — the rule `execute_set` already
  applies to the edits. One undesignable member therefore discards the
  designs of the others; a per-member design-failure carrier was considered
  and not implemented.

  **Reversed by the owner, 2026-10-14, and implemented.** Each member carries
  either its design or its own design failure, so the designable members are
  returned: `Q5DesignedSet::members` is one `Q5MemberDesign` per member in
  request order, each holding a `MemberDesignOutcome` of `Designed` or
  `NotDesigned`, and `designs`, `failures`, `designed_count` and
  `all_designed` read across them. D030's 55 °C floor makes a design failure
  on an AT-rich site ordinary rather than exceptional, which is why the 3c
  choice was revisited. A failure of the *edits* still fails the whole set, as
  `execute_set` does. In the same decision, a set whose members share a result
  identifier is refused at construction (§10 item 6).

A set is designed under one request, so all members share the bound, style,
policy and placement. Per-member design requests are not supported.

## 13. Traceability

| Rule | Authority |
| --- | --- |
| Q5 is an explicit supported workflow with a documented accepted input scope | brief, D014 |
| A request is a typed invocation; the LLM never becomes the execution engine | D016, D004 |
| DNA template, intent, numbering convention, identified target CDS are the required inputs | brief |
| Numbering belongs to the annotation, not to the request | D006, M1 |
| A residue position identifies its CDS and numbering origin | D006 |
| A residue substitution identifies a target CDS, residue, expected original, replacement | D011 |
| Scoping a DNA edit is an operation parameter, not a property of the edit | M2 |
| An unscoped nucleotide edit is a legal mutation-domain operation | M2, fixture `nuc-a-unscoped` |
| "Mutate residue 42 to glycine" against several CDSs → needs_clarification, no calculation | brief, D005 |
| That residue request had no representation before this task | impl |
| `ResidueIntentWithoutCds` + `ResidueWithoutCds` variant closes it | proposed representation |
| M1's `ResiduePosition` is not relaxed to carry the gap | D006, M1, proposed |
| Binding a residue intent uses M1's and M2's constructors, and validates nothing of its own | proposed, D003 |
| The residue number is validated where a CDS identifier exists (bind time) | proposed, M1 |
| The gate reads annotation metadata and identity, never sequence content | impl |
| Candidates are not narrowed by residue feasibility | impl; owner 2026-10-14 |
| The brief's residue-42 reason is true of the long-template fixture, not of the 30-nucleotide one | impl, fixture record |
| `verify`'s scope parameter is read on the residue branch only | impl, limitation |
| A residue request whose expected original equals its replacement proceeds | M2, impl; owner 2026-10-14 |
| No CDS annotation at all is a missing attribute, not an empty CDS | M1 |
| An annotation of another artifact is refused, not dropped | M1, D007 |
| Duplicate annotation identifiers are not deduplicated | impl |
| Several annotations and no claim → clarification, no target | brief, D005, D020, impl |
| Named identifier matching no annotation → clarification | impl; owner 2026-10-14 |
| `Validated` ⇒ exactly one resolved target; clarification ⇒ none | impl, module-local |
| `execute` returns a structured error rather than unwrapping that invariant | impl, this task; error.rs |
| A clarification run performs no edit and produces no artifact | D005, impl |
| `calculated` is never presented as `verified` | D020, M2 |
| Verification is separate from generation and re-derives claims | D013, M2 |
| Verification postconditions (§8 list) | M2, D013 |
| No verification failure is reachable through `execute` today | impl, stated narrowly |
| The edited artifact's name is the caller's | impl, M2, D007 |
| Result provenance is M2's, unchanged, plus the workflow code-path name/version | D021, M2 |
| Computational validation is never experimental validation | D020 |
| Primer design, thermodynamics, warnings, rejection criteria are unwritten | D014, brief |
| Nucleotide containment policy: an unscoped DNA edit proceeds | impl; owner 2026-10-14 |
| A set is one typed request with N ordered members, not a caller loop | task contract, impl |
| The workflow reaches primer design by composition, not inside `q5.rs` | 3c contract, owner 2026-10-14 |
| A design is built against the template the run's own request names | 3c contract, D007, I-6 |
| A clarification run designs nothing, and that is not a failure | D005, 3c contract |
| A set's design failure is atomic; no per-member design carrier exists | owner 2026-10-14; **gap** |
| One design request per set, so members cannot mix bounds | 3c contract; **gap** |
| The composition mode is explicit and never inferred from the member count | task contract, impl |
| `CombinedConstruct` is representable and refused, never run as separate variants | task contract, D014, impl |
| `UnsupportedComposition` is the refusal's error variant, categorized `Unsupported` | impl, error.rs |
| An empty variant list is refused at construction with `InvalidMutationRequest` | task contract, impl |
| Set members must address one template artifact by name, digest, and topology | task contract, M1, D007 |
| Set members' explicit CDS claims must agree, via `ResiduePositionWrongCds` | task contract, M2 |
| An unbound member takes the set's agreed claim | task contract, impl |
| The set resolves its target once through `validate_input_scope` unchanged | task contract, impl |
| A clarification set performs no edit and produces no artifact for any member | D005, D020, impl |
| Each member is applied to the pristine template its own request names | task contract, impl |
| A failing member returns the original `DomainError`; no partial set escapes | task contract, impl |
| Results are per-variant, in request order, each carrying its caller's name | task contract, D007 |
| A one-member set gives the same result as the single-request `execute` | impl, test |
| Set result identifiers must be distinct, compared exactly | owner 2026-10-14, §10 item 6 |
