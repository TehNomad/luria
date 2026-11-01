---
type: Task Contract
title: "Milestone 3c task contract: wiring the Q5 workflow to the design layer"
description: "Two layers exist and do not meet: crates/core/src/q5.rs runs the Q5 workflow and produces no primers, and crates/core/src/primer_design.rs designs a primer pair."
status: deprecated
tags: [milestone-3]
---
# Milestone 3c task contract: wiring the Q5 workflow to the design layer

Status: Implemented 2026-10-14. See §10.
Date: 2026-10-14
Planner: Claude. Owner decisions recorded in §3.

## 1. What this task is

Two layers exist and do not meet:

- `crates/core/src/q5.rs` runs the Q5 workflow: it resolves a target
  annotation, applies the edit, and verifies it (`execute`, `execute_set`). It
  produces no primers.
- `crates/core/src/primer_design.rs` designs a primer pair from a verified
  `MutationResult` under `luriaNative` (D029).

This task adds the module that composes them, so a Q5 request can reach a
primer pair. It completes step 8 of D015's slice for the substitution path:
a structured result that a caller can act on at the bench.

## 2. Scope

**In scope:** one new file, `crates/core/src/q5_design.rs`, and its
registration in `crates/core/src/lib.rs`.

**Out of scope, and not to be edited:** `q5.rs`, `primer.rs`,
`primer_design.rs`, `mutation.rs`, `sequence.rs`, `cds.rs`, `error.rs`, and
every existing test and fixture. This task adds no error variant: every
failure it reports is one the design layer or the workflow layer already
produces. If a new variant appears to be needed, stop and report it.

No Tm, no candidate ranking, no `nebBaseChangerCompatible` policy, no insert
or delete. Those are unwritten or deliberately deferred (D014, D028).

## 3. Owner decisions, 2026-10-14

1. **A separate module.** The wiring is a third module that composes the other
   two. `Q5SdmRequest` does not gain a design field, and `execute` keeps its
   signature, so no existing caller or test changes. This also keeps
   [docs/specs/q5-sdm.md](/specs/q5-sdm.md) true as written: the workflow contract states that it does
   not describe primer design.
2. **Atomic set failure.** *Reversed by the owner on 2026-10-14, after D030
   made design failures more common: each member reports its own design or
   design failure. Implemented; see §11.* As originally decided: In a variant set, a member whose design fails
   returns that `DomainError` and the set returns no designs, matching
   `execute_set`'s existing rule that no partial results escape as a
   successful set. The alternative — a per-member carrier that reports the
   designs that do exist alongside the ones that cannot be made — is recorded
   as a known gap in §7, not implemented.

## 4. Required API

```rust
/// A Q5 run carried whole, plus the primer design for its edit.
pub struct Q5DesignedRun { /* private fields */ }

impl Q5DesignedRun {
    pub fn run(&self) -> Q5WorkflowExecution;
    pub fn design(&self) -> Option<PrimerDesign>;
    pub fn needs_clarification(&self) -> bool;
    pub fn template_reference(&self) -> SequenceReference;
    pub fn describe(&self) -> String;
}

pub struct Q5DesignedSet { /* private fields */ }

impl Q5DesignedSet {
    pub fn run(&self) -> Q5VariantSetExecution;
    /// One design per member, in request order. Empty for a clarification.
    pub fn designs(&self) -> Vec<Q5MemberDesign>;
    pub fn design_count(&self) -> usize;
    pub fn needs_clarification(&self) -> bool;
    pub fn template_reference(&self) -> SequenceReference;
    pub fn describe(&self) -> String;
}

/// A member's result name paired with its design, so a design in a set of
/// several is traceable to the artifact it was designed for.
pub struct Q5MemberDesign { /* private fields */ }

impl Q5MemberDesign {
    pub fn result_name(&self) -> Identifier;
    pub fn design(&self) -> PrimerDesign;
}

pub fn design_run(
    run: Q5WorkflowExecution,
    request: PrimerDesignRequest,
) -> Result<Q5DesignedRun, DomainError>;

pub fn design_set(
    run: Q5VariantSetExecution,
    request: PrimerDesignRequest,
) -> Result<Q5DesignedSet, DomainError>;
```

Fields stay private with no public constructor, as `q5.rs` does, so the
invariants in §5 hold module-locally by construction discipline. State that
in the module comment rather than claiming the types make an invalid value
unrepresentable.

## 5. Required behaviour

**R1 — a clarification designs nothing.** When the run needs clarification it
holds no mutation, so `design_run` returns `Ok` with `design() == None`, and
`design_set` returns `Ok` with no designs. This is not a failure: the
clarification is the answer, and it is carried through intact (D005: no
sequence calculation performed). No `PrimerDesignRequest` field is read on
this path.

**R2 — one design request per call.** A set is designed under one request, so
every member shares the bound, the style, the policy and the placement. A
per-member design request is out of scope.

**R3 — the design is built against the run's own template.** The template
passed to `design_primers` is the one the run's request names
(`run.scope().request().template()` for a run; the set's gate request for a
set). Nothing re-loads or re-normalises a sequence, and no caller-supplied
template is accepted, so `PrimerDesign::built_against` is necessarily the
artifact the edit was applied to (I-6, D007).

**R4 — the verified result is passed through unchanged.** The
`MutationResult` handed to `design_primers` is the one the run produced. This
layer does not re-verify, re-derive a coordinate, or rebuild an intent;
`design_primers` already refuses an unverified result, and that refusal is
the guard.

**R5 — failures propagate unchanged.** Every `DomainError` from
`design_primers` is returned as it stands, with its own variant and category.
This layer adds no message and no variant.

**R6 — atomic sets.** Members are designed in request order. The first
failure returns that error; no partial set escapes (§3 decision 2).

**R7 — determinism.** The same run and the same request produce an equal
result (D012, D022). The module has no clock, no randomness, and no I/O.

**R8 — invariants.** `design()` is `Some` exactly when the run carried a
verified mutation. `designs()` has one entry per member exactly when the set
resolved a target, and is empty otherwise. Each `Q5MemberDesign::result_name`
equals the name of the member artifact its design was built for.

## 6. Required tests

Unit tests in `q5_design.rs`, reusing the synthetic Milestone 2 and 3a test
templates already used in `q5.rs`'s own tests. Every expected primer span and
text must be derived from D029's closed form by hand and the derivation
written into the test, as `primer_design.rs` does. Do not copy a value out of
a test run.

Required cases:

1. a residue run reaches a primer pair; the pair abuts, and its forward span
   and text are the ones D029's form gives for that edit;
2. a nucleotide run reaches a primer pair;
3. an unbound-residue run (`ResidueWithoutCds`) reaches a primer pair, so the
   gate-resolved CDS path is covered;
4. a clarification run returns `Ok` with no design, and the clarification text
   survives;
5. the design's `built_against` equals the run's `template_reference()`;
6. a variant set of two members yields two designs, in request order, each
   named by its own member result name, and the two differ;
7. a set whose member cannot be designed returns that member's error and no
   designs (R6). Use an edit too close to a template end, which gives
   `NoCenteredPrimerDesign`;
8. a set that needs clarification returns `Ok` with no designs;
9. the same run and request twice give equal results (R7);
10. a template too short for any pair gives `NoPrimerDesignPossible`
    unchanged, demonstrating R5.

If a required case cannot be built without touching an out-of-scope file,
stop and report it rather than editing that file.

## 7. Known gaps to record on completion

- **No per-member design failure.** An undesignable member discards the whole
  set (§3 decision 2). A caller that wants the designs that do exist cannot
  get them.
- **One request per set**, so a panel cannot mix bounds (R2).
- **No Tm anywhere**, so `minTm`, `tmFunction` and `saltModel` are reported
  unresolved, exactly as `primer_design.rs` reports them (§11 of the design
  contract).
- **`nebBaseChangerCompatible` is unrepresentable**, so a compatibility run
  cannot be requested at all rather than being refused at run time (D028).
- **The insert and delete paths remain absent** at the mutation layer, so the
  workflow cannot reach a design for them (D014).

## 8. Documentation to update on completion

- [docs/specs/q5-sdm.md](/specs/q5-sdm.md): a short section recording that primer design is reached by
  composition in `q5_design.rs`, that the workflow layer itself still designs
  nothing, and the two owner decisions of §3. Do not restate D029.
- [docs/tasks/contracts/milestone-3-q5-workflow.md](/tasks/contracts/milestone-3-q5-workflow.md): the "Known gaps" entry saying no
  primer logic exists is now partly closed; point it at this contract.
- This file: what was implemented, deviations, and the §7 gaps as found.

Do not edit [docs/decisions/](/decisions/). This task implements D015, D014, D028 and
D029 as written and decides nothing they leave open.

## 9. Required checks

From the repository root:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

All five must pass. Report the exact commands and their results. In addition,
confirm the new tests can fail: break one expected primer span by one base,
observe the failure, restore the file, and report both outcomes.

---

## 10. What was implemented, 2026-10-14

`crates/core/src/q5_design.rs` (new, registered in `lib.rs`) with
`design_run`, `design_set`, `Q5DesignedRun`, `Q5DesignedSet` and
`Q5MemberDesign`, exactly as §4 specifies. No other source file changed
except `lib.rs`'s module list, and no error variant was added.

All of R1–R8 are implemented as written. Two notes on how:

- **R3, per-member template.** `design_set` takes each member's own template
  from its own request rather than the set's first, which is the same artifact
  by construction (`Q5VariantSetRequest::create` requires every member to
  address one artifact by name, digest and topology).
- **R8, one design per member.** `design_set` zips the set's members with its
  results rather than indexing, so a length disagreement could not panic.
  That the two lists have equal length is a `q5.rs` invariant — `execute_set`
  pushes one result per member in request order — relied on here and pinned by
  `a_two_member_set_gets_one_design_per_member_in_request_order`, which
  asserts `design_count() == run().result_count()`.

**Deviation from §4.** `Q5DesignedRun::describe` was specified to delegate to
the design's own label, but `PrimerDesign` has no `describe`, and
`primer_design.rs` is out of scope to edit (§2). It reports the run's label and
the edit span instead: `"<run> designed [42, 45)"`. No classification depends
on it.

Tests: the 10 required cases of §6, all passing, on a 90-nucleotide synthetic
template whose whole length is one 30-codon CDS. Every expected span and
primer text is D029's closed form applied to that text by hand, with the
derivation written into the test module's comment. Breaking one expected
forward span by a single base was confirmed to fail
(`primer_spans(&design) == ((32, 56), (17, 32))`, expected `(32, 55)`), and
the file was restored.

The §7 gaps are as predicted, with nothing new found.

### Checks

```
cargo fmt --all -- --check                                   pass
cargo test --workspace                                       pass, 262 tests, 0 failures
                                                             (223 unit; 8, 6, 14, 11 integration)
cargo clippy --workspace --all-targets -- -D warnings        pass
git diff --check                                             pass
git status --short                                           reviewed
```

---

## 11. Per-member design outcomes, 2026-10-14

The §3 decision 2 reversal, implemented in `crates/core/src/q5_design.rs`:

- `MemberDesignOutcome` is `Designed(Box<PrimerDesign>)` or
  `NotDesigned(DomainError)`. The design is boxed because it is much larger
  than an error, which clippy's `large_enum_variant` requires.
- `Q5MemberDesign` pairs a member's result identifier with its outcome, and
  offers `design()`, `failure()` and `is_designed()`.
- `Q5DesignedSet` reports `members()` (one per member, request order),
  `member_count()`, `designs()` (the successful ones), `designed_count()`,
  `failures()` and `all_designed()`. `all_designed()` is false for a
  clarification, which has no members, rather than vacuously true.
- `design_set` no longer returns `Err` for a member's design failure. Its
  `Result` is kept for the signature's stability; no member failure reaches it.

The §7 gap "an undesignable member discards the whole set" is closed. The
remaining §7 gaps stand: one design request per set, no Tm beyond D030's
rules, `nebBaseChangerCompatible` unrepresentable, and no insert or delete.

Tests: `a_member_that_cannot_be_designed_does_not_discard_the_others` (three
members, the middle one undesignable: the other two are designed, in order,
and the failure carries its own obstacle),
`a_set_no_member_of_which_can_be_designed_reports_every_reason`, and
`a_failing_edit_still_fails_the_whole_set_before_any_design`, which pins that
the edit-level rule is unchanged. The clarification test now also asserts
`!all_designed()`.

Also in this pass, from [docs/specs/q5-sdm.md](/specs/q5-sdm.md) §10 item 6: `Q5VariantSetRequest`
refuses two members sharing a result identifier, with
`InvalidMutationRequest` (`MalformedInput`), names compared exactly
(`two_variants_sharing_a_result_name_are_refused`,
`result_names_are_compared_exactly`).
