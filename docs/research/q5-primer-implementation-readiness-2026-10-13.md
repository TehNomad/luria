# Q5 primer split policy revised, and the implementation-readiness audit

**What this is.** Two things, in one document. First, the record of the pass
that changed the `luriaNative` primer **split rule** from availability-aware
splitting to a **mutation-centered** split. Second, an audit of the Q5 primer
workflow's implementation readiness: what is specified, what is implemented,
what is still unresolved, and the prioritized blockers that follow.

**Date.** 2026-10-13.

**What this pass did not do.** It wrote no Rust. `PrimerDesign` is still not
implemented, no candidate is generated anywhere in `crates/`, no Tm function
exists, and no runtime behaviour changed. The pass is documentation and audit.
D029 remains **proposed and unapproved**.

**Supersedes / is superseded by.** This pass supersedes the split rule recorded
by [docs/research/q5-primer-policy-review-2026-10-12.md](/research/q5-primer-policy-review-2026-10-12.md) (A8, Part D item 4,
Part E's D029 bullet, D-5's "the split of `a_f` among ties" row). That report
carries a dated supersession note at its head and is otherwise unchanged.
Everything that report established about evidence, enumeration, test plans and
the compatibility claims C1–C25 still stands; only the split rule changed.

---

## Part 1 — The revised rule

### The rule

For an edit at template span `[s, e)`, replacement length `L = e − s`, edited
span `[s, s + L)`, and a candidate forward annealing total `a_f`:

```
u = ceil(a_f / 2)      v = floor(a_f / 2)      |u − v| ≤ 1
```

`u` and `v` count the **unchanged template-derived bases** of the forward primer
on the 5′ and the 3′ side of the replacement region. The replacement region sits
in the middle of the forward primer's unchanged annealing flanks. When `a_f` is
odd the extra base goes to the **5′** side; that assignment is a stated
convention, recorded so a second run cannot choose the other one, and the
alternative is equally arbitrary.

`(u, v)` is a function of `a_f` **alone**. Availability does not enter it.
Availability is a feasibility test on the centered candidate:

```
s − u − a_r ≥ 0            s + L + v ≤ template.length
```

(a linear template, D005/D019; a circular template has no availability bound,
only a length bound). A centered candidate that fails either test is **not
shifted** to one that fits. The policy reports that no design is available under
it, and names the side that bound it.

The pair geometry is otherwise unchanged from D029: the pair abuts at the
template boundary `s − u`; the forward primer covers `[s − u, s + L + v)` with
text `template[s − u, s] + replacement + template[s + L, s + L + v]`; the
reverse primer covers `[s − u − a_r, s − u)` with text
`revcomp(template[s − u − a_r, s − u])`; `a_f = u + v` excludes the replacement;
primer lengths are `a_f + L` and `a_r`.

### Notation corrected in the same pass

The pass that wrote the availability-aware split used `w` for the reverse
annealing total and `hi5 = s − w`, `hi3 = template.length − (s + L)` for the
availabilities. Those names are kept where they appear, but the notation is
stated once, cleanly: `a_r = w`, and the two availability tests above are the
same inequalities as §18.2's `s − u − w ≥ 0` and `s + L + v ≤ n`. The reason to
restate them is that the old text defined `hi5` and `hi3` only inside the split
formula, so the availability constraints read as a different pair of tests from
the ones the split was checked against.

### The multi-base statement

The single-nucleotide case (`L = 1`, the initial native scope) makes `|u − v| ≤
1` the whole rule. For a longer **contiguous** replacement region the same
definition applies unchanged: the region's `L` bases sit between the two flanks,
so "center the region in the primer" and "equalise the flanks" are one statement.

A changed set that is **not** one contiguous interval — a residue edit on a CDS
that wraps the origin of a circular reference, where the changed positions are
discontinuous in template coordinates — has **no centered definition here**. It
is left undefined and out of the initial native scope, not centered by
generalisation. This is recorded rather than resolved: generalising the equation
would be choosing a rule for a case the scope has not admitted.

### What the revised rule does and does not claim

The reference tool's short-substitution path is described, in the saved in-app
help text, as anchoring the center of the primer to the nucleotide region and
extending in both the 5′ and 3′ directions "so that there are at least 10
nucleotides that are 100% complementary on both sides of the mutation region"
([docs/references/nbc_help_v284.txt](/references/nbc_help_v284.txt) line 200; quoted at
[docs/research/nebasechanger-live-build-4330b0c7-2026-10-10.md](/research/nebasechanger-live-build-4330b0c7-2026-10-10.md) §15.4,
`@19261`). That is source evidence about that path.

Three things stay separate, and the pass keeps them separate:

* the reference path's behaviour, which the evidence above supports;
* the `luriaNative` centered rule, which is a **stated Luria choice**, proposed
  and unapproved;
* everything unverified.

Adopting a similar shape is **not** evidence that `luriaNative` matches the
reference build. The reference's per-side ≥ 10 bound is **not** adopted — the
2026-10-12 pass's A9 decision keeps H1 a bound on the annealing total — and
neither its growth order (grow both flanks by 5 until Tm reaches the target) nor
its Tm engine follows. D028's rule that matching a numeric default or adopting a
similar geometry is not proof of compatibility applies unchanged.

What the centered rule **is** evidence for is a shape observation, checked
against the fixture corpus: the reference forward primer of `q5p-01` has
annealing spans `[[1199, 1209], [1212, 1222]]` around an edit at `[1209, 1212)`
(`tests/fixtures/q5-primer/reference/designs.json`), i.e. `u = v = 10`, `L = 3`,
`a_f = 20`, primer length 23. Its flanks are equal, so its forward geometry is
centered in the same direction as the revised rule. The difference between the
two policies is the **total** and the **growth rule**, not the shape: with
`minAnnealingOverlap = 15`, `luriaNative` sets `a_f = 15` (`u = 8`, `v = 7`,
primer length 16), while the reference design carries `a_f = 20` (primer length
23) because it grew past the bound toward a Tm target.

---

## Part 2 — Documentation changed by the pass

| File | Where | Change |
| --- | --- | --- |
| [docs/specs/q5-primer-design.md](/specs/q5-primer-design.md) | §1 rule table | added the centered-split rule (it was unlisted, so a reader of §1 could not see it) |
| | §18.1 | split replaced: availability-aware → mutation-centered; `u`, `v` restated as unchanged template-derived flanks with `a_f = u + v` excluding the replacement; availability restated as a feasibility test on the centered candidate; multi-base statement added; the 2026-10-12 amendment kept and labelled superseded |
| | §18.2 | added that the centered split fixes the candidate set, and that availability and alphabet remain the hard feasibility tests — hard constraints and candidate-generation rules kept visibly separate |
| | §18.3 | third ordering key `u ≥ v` still removed, reason restated (it is now true by construction, so as a key it orders nothing); monotonicity consequence added: availability is monotone along the order, so the first candidate is the only one that can fit under the initial policy |
| | §18.4 | closed form restated for the centered split, with the feasibility condition; the "arbitrary among ties" rule replaced by the centered rule, with the reference's ≥ 10 wording kept as source evidence about its own path |
| | §18.5 | outcome 1 renamed to *no centered candidate fits* and required to name the binding side; outcome 2's "one may exist above them" clause dropped (unsound under a monotone availability test); the centered-geometry distinction stated; the error-model gap recorded |
| | §21 change table | three rows marked superseded, kept rather than rewritten |
| | §22 (new) | index to this pass, mirroring §21's form |
| [docs/decisions/](/decisions/) | D029 | amended in place: status line added (proposed, not approved); split bullet replaced; `a_f` notation clarified; Reason rewritten (centered rationale, superseded rule named, reference wording kept as evidence about the reference path); Evidence gained the help-text citation; Consequences gained the closed form, the `maxAnnealingOverlap` monotonicity, and the error-carrier gap; Reversibility gained that reverting is a policy change needing approval |
| [docs/tasks/contracts/milestone-3b-q5-primer-contract.md](/tasks/contracts/milestone-3b-q5-primer-contract.md) | §8 "Still open" | amended: a rule for the annealing boundary is now *proposed* under D029, not settled; D029's unapproved status stated; nothing in the implementation pass changed |
| [docs/research/q5-primer-policy-review-2026-10-12.md](/research/q5-primer-policy-review-2026-10-12.md) | head | dated supersession note naming the four passages that changed meaning; body otherwise intact |
| this file | — | the record and the audit |

[docs/specs/q5-sdm.md](/specs/q5-sdm.md), [docs/tasks/contracts/milestone-3-q5-workflow.md](/tasks/contracts/milestone-3-q5-workflow.md),
`tests/fixtures/q5-primer/README.md` and `tests/fixtures/q5-primer/cases.tsv`
were inspected and **not** changed: they carry no split rule. Their
`split`/`boundary` vocabulary is about primer decomposition and the reference
tool's tail/annealing columns, not about choosing `(u, v)`. `q5p-09`'s expected
outcome is unchanged under the revised rule (see Part 5).

---

## Part 3 — Implementation-readiness audit

Three states per element: **specified** (a contract states it), **implemented**
(Rust enforces it), **unresolved** (no contract, or a contract with no
implementation). Evidence is file:line as it stands in this worktree.

### Implemented — validation only

| Element | Evidence |
| --- | --- |
| What a `Primer` is: role, orientation, placement, built-against reference, span, parts, changed positions, annealing portion, annealing overlap, whole-primer match count | `crates/core/src/primer.rs` types at 96 (`PrimerRole`), 134 (`PrimerOrientation`), 161 (`PrimerPlacement`), 188 (`PrimerPartRole`), 213 (`PrimerPart`), 260 (`Primer`); `construct_primer` at 482 derives the decomposition and re-checks the structural invariant |
| H2 (pair covers the changed region), H4 (opposite orientations), H5 (relationship derived from the two spans) | `assemble_pair` at 713; `PrimerPair::separation`/`boundary`/`abuts` at 408/421/433 |
| H1 (annealing overlap meets the bound), and that a bound < 1 is a request defect | `require_design_constraints` at 806; bound < 1 → `InvalidPrimerCandidate`; below-bound → `AnnealingOverlapBelowBound` at 820/832 |
| H6 alphabet, H8 mutated-position partition | `construct_primer` rejects a bad partition as `PrimerPartition` (654); alphabet checked there |
| `annealing_overlap` counts annealing positions only, not the whole primer | 659–699: `overlap_positions` pushed only where `!is_changed(...)`; `whole_primer_match_count` kept apart (I-13) |
| Placement: `Inline` supported, `ConfineToTail` refused | `PrimerPlacement` 161; `is_supported` false at 178; `UnsupportedPrimerPlacement` returned at 511 |
| Error categories for primer-level failures | `crates/core/src/error.rs`: `InvalidPrimerCandidate` → `MalformedInput` (412), `PrimerPartition`/`AnnealingOverlapBelowBound` → `BiologicalValidation` (427–428), `UnsupportedPrimerPlacement` → `Unsupported` (437) |
| Q5 input-scope gate: which supported CDS a request targets; clarification vs failure | `crates/core/src/q5.rs` (`Q5SdmRequest` 320, `Q5WorkflowState` 413, `Q5WorkflowResult` 447, composition/variants 904–1200) |

The module's own doc says this state, and says it correctly:
`primer.rs:3-14` — "this module takes a proposed primer and decides whether it
is one. It never proposes a span, a length, or a candidate."

### Specified, not implemented

| Element | Specified at | Implemented |
| --- | --- | --- |
| The centered split, `a_f`, `a_r`, the abutting pair | §18.1, D029 | **no** — no type, no function |
| Candidate set `(a_f, a_r)` and its ordering | §18.3 | **no** |
| Hard feasibility tests as a search | §18.2 | **partly** — `require_design_constraints` checks H1 on a primer the caller already supplied; availability is checked nowhere |
| The three failure outcomes | §18.5 | **no** — no carrier (below) |
| `designPolicy`, `compatibilityTarget` | D028, §19 | **no** — `PrimerDesign` does not exist; `rg "PrimerDesign\|designPolicy\|DesignPolicy\|compatibilityTarget\|CompatibilityTarget" crates/` over `*.rs` returns nothing |
| The request's design parameters (`minAnnealingOverlap`, `maxAnnealingOverlap`, `targetTm`) | §2.2, §18 | **no** — `Q5SdmRequest` is `{ template, scope, intent }` (q5.rs:320–323); none of the three fields exists |
| Tm | §16, §18 | **no** — no Tm function anywhere |
| Ranking/preference beyond the two order keys | §18.3 | **no**, and none is needed under the initial policy |

### Unresolved — no contract, or a contract that cannot be closed yet

1. **No design-level type or entry point.** There is no `PrimerDesign`, no
   request field carrying the bounds, and no function from a `MutationResult`
   to a `PrimerPair`. The whole of §18 sits between `q5.rs` (which stops at the
   CDS gate) and `primer.rs` (which starts at a caller-supplied primer).
2. **No error carrier for the three outcomes.** `error.rs` has no variant for
   "no design available under this policy". The nearest existing variants mean
   other things: `AnnealingOverlapBelowBound` asserts a bound was *missed*,
   which §18.5 forbids for a centered-geometry failure; `InvalidPrimerCandidate`
   is `MalformedInput`, about the request; `UnsupportedPrimerPlacement` is about
   placement. Which `ErrorCategory` a design-level absence belongs to is a
   contract choice, not a routine one.
3. **`maxAnnealingOverlap` has no defined role under the initial policy.**
   Under the centered split availability is monotone along the order (§18.3), so
   a cap cannot rescue an infeasible edit; the closed form (§18.4) does not use
   it. Whether it is accepted-and-ignored, rejected, or reserved for the Tm
   policy is not stated. §18.5 outcome 2 now reports the same absence as
   outcome 1, which is the honest reading, but the request-level behaviour is
   open.
4. **Abutment is reported, not required.** `assemble_pair` deliberately leaves
   §15 question 3 open (comment at 710–712). §18.1 states the native pair abuts.
   Nothing enforces it, and whether it is a requirement or a property of the
   native construction is still §15's open question — unchanged by this pass.
5. **No fixture pins a `luriaNative` expectation.** All ten cases in
   `tests/fixtures/q5-primer/cases.tsv` are reference-design (5: q5p-01, 02,
   03, 04, 07), contract-only (4: q5p-05, 06, 08, 10) or no-design (1: q5p-09).
   None states what `luriaNative` must emit. The policy change is therefore not
   testable against a pinned expectation — see Part 6.
6. **No Rust test reads the Q5 primer fixtures.** `rg "q5-primer\|q5p-"` over
   `crates/` and `tests/` matches only the reference Python scripts and
   `primer.rs`'s doc comment. The fixture corpus is not wired into the test
   suite.
7. **The native scope is not exercised at the primer layer.** The 17
   tests in `primer.rs` build primers for a **residue** edit (three changed
   positions, `L = 3`, DNA `AAT`→`ATG` at 9–11, residue 4 N→M), not for a nucleotide
   substitution; `rg "apply_nucleotide|NucleotideMutationIntent" crates/core/src/primer.rs`
   returns nothing. §18's initial scope (`L = 1`, one inline nucleotide
   substitution) therefore has no primer-layer test. The mutation layer does
   exercise nucleotide substitution (`mutation.rs`, `q5.rs`,
   `tests/mutation_invariants.rs`, `tests/mutation_fixtures.rs`), so the gap is
   specific to primers and pairs.

---

## Part 4 — Prioritized blockers

P0 blocks writing the policy at all; P1 blocks testing it; P2 blocks claiming
anything about the reference.

| # | Blocker | Blocks | Smallest next step | Who decides |
| --- | --- | --- | --- | --- |
| **P0-1** | No `PrimerDesign` type, no design-parameter fields on `Q5SdmRequest`, no entry point from `MutationResult` to `PrimerPair` | any implementation of §18 | add the typed design record and the request fields, mirroring §2.2's names, before any search code | contract (mechanical once §2.2's names are fixed) |
| **P0-2** | No error carrier for §18.5 outcomes 1 and 2, and no stated `ErrorCategory` for them | reporting a centered-geometry absence at all; the distinction §18.5 requires | propose the variant and its category in [docs/decisions/](/decisions/) as a decision, then implement | **human** — the category is a contract choice (D005's fail-closed rule does not pick it) |
| **P0-3** | D029 is **proposed and unapproved**; §18 is a contract, not a decision | implementing the centered split as *the* policy rather than a candidate | human approval of D029 as amended | **human** |
| **P1-1** | No fixture states a `luriaNative` expectation, and no case distinguishes the centered rule from the superseded availability-aware rule | any test that would notice the policy changed | add one case: an edit near a template end where the availability-aware rule would have designed and the centered rule reports no design (worked example, Part 5). Its expected outcome under the centered policy is a no-design report naming the binding side | expectation is **human**-gated by D029; the case itself is mechanical |
| **P1-2** | No Rust test reads `tests/fixtures/q5-primer/` | turning the existing corpus into a check | a reader for `cases.tsv` + `designs.json` asserting the invariants I-1…I-13 against the reference designs, without asserting algorithm-specific text | mechanical |
| **P1-3** | No primer-layer test exercises the native scope (`L = 1` nucleotide substitution) | confidence that `construct_primer`/`assemble_pair` handle the scope §18 actually admits | one test per scope case: substitution inline, `L = 1`, both flanks named, `a_f` at the bound | mechanical |
| **P2-1** | Availability is checked nowhere in Rust | the feasibility half of §18.2; the "names the binding side" requirement of §18.5 | a function returning the two availabilities and the failing side, used by the search | mechanical once P0-1 exists |
| **P2-2** | `maxAnnealingOverlap`'s role under the initial policy is undefined (see Part 3 item 3) | request-level behaviour, and an honest outcome-2 report | state one of: rejected, accepted-and-ignored, or reserved for the Tm policy | **human** |
| **P2-3** | §15 question 3 (must a pair abut?) is still open, and `assemble_pair` only reports abutment | whether §18.1's abutting pair is a requirement or a construction property | settle it by a source, or by the caller declining to need it (the 2026-10-12 report's Part E item) | **human** / source |
| **P2-4** | The compatibility claims C1–C25 of the 2026-10-12 report are unverified against a live build for the native policy | any statement that `luriaNative` matches NebaseChanger | none — do not claim it. D028's rule already forbids the claim; the gap is recorded, not closed | n/a |

The order is not a queue: P0-1 and P0-2 are one contract change, and P0-3 gates
all of them. Nothing above was resolved by this pass except by writing the
ambiguity down.

---

## Part 5 — Behavioural delta of the policy change

The centered rule is strictly **less permissive** than the superseded
availability-aware rule: whenever the centered candidate fits, the availability-aware
split chooses the same `(u, v)` (`v = min(floor(a_f/2), hi3) = floor(a_f/2)`
when `hi3 ≥ floor(a_f/2)`). The delta is one-directional — designs the old rule
would have emitted and the new rule refuses.

Worked example, the smallest kind of case the delta turns on:
`template.length = 40`, edit at `s = 35`, `L = 1`,
`minAnnealingOverlap = 15`, `a_r = 15`.

* **Centered:** `u = 8`, `v = 7`. 5′ test `35 − 8 − 15 = 12 ≥ 0` passes. 3′ test
  `35 + 1 + 7 = 43 > 40` fails. → **no design**, reported as outcome 1, naming
  the 3′ side: 4 bases available, 7 asked for.
* **Availability-aware (superseded):** `hi3 = 4`, so `v = 4`, `u = 11`;
  `hi5 = 35 − 15 = 20 ≥ 11` passes. → a design: forward covers `[24, 40)`
  (16 bases, the template's last 16), reverse covers `[9, 24)` (15 bases).
  H1 holds (`a_f = a_r = 15`), so the old rule emitted a pair whose forward
  primer ends at the template's end and whose mutation sits 11 bases from the
  primer's start and 4 from its end.

For the general case (`L = 1`, bound `b`, template length `n`) the centered rule
admits an edit if and only if

```
s ≥ b + ceil(b / 2)        s ≤ n − 1 − floor(b / 2)
```

so on a 40-base template with `b = 15` the admissible window is
`s ∈ [23, 32]`. §18.4 states the same condition in closed form.

`q5p-09` (bound 31 against a 30-base template) is a no-design case under both
rules, and its fixture note — "no primer of any span overlaps 31 bases of a
30-base template: unsatisfiable under every geometry" — remains true, so the
fixture was not touched. What changed is only which side a *report* must name
for cases like the one above, and that no report may promise a design above a
supplied cap.

---

## Part 6 — Coverage of the corpus, as measured

`tests/fixtures/q5-primer/cases.tsv`, 10 cases:

| Case | kind | `min_overlap` | `targetTm` | reference design | expected |
| --- | --- | --- | --- | --- | --- |
| q5p-01 | replace | 15 | 55 | yes | design |
| q5p-02 | replace | 24 | 55 | yes | design (H1 **violated** by the reference: fwd annealing 20 < 24) |
| q5p-03 | replace | 25 | 55 | yes | design (H1 violated; rev annealing 25 = bound) |
| q5p-04 | replace | 15 | 55 | yes | design (shared-base count: annealing 20 ≠ whole-primer match 21 ≠ span 23) |
| q5p-05 | insert | 15 | 55 | no | contract-only |
| q5p-06 | delete | 15 | 55 | no | contract-only |
| q5p-07 | replace-set | 15 | 55 | yes | design-per-variant |
| q5p-08 | replace | 6 | absent | no | contract-only (strand −, cds-2) |
| q5p-09 | replace | 31 | absent | no | **no-design** |
| q5p-10 | replace | 15 | absent | no | contract-only (I-10: absence must be reported, not defaulted) |

No case carries a `luriaNative` expectation; no case sits in the window where
the two split rules differ (Part 5). The corpus pins invariants I-1…I-13 and the
reference designs, which is what it was built for and what it still gives.

---

## Part 7 — Checks run, and their results

Run in this worktree on 2026-10-13, after the documentation edits. No Rust was
edited by this pass.

| Check | Result |
| --- | --- |
| `git status --short --branch` | branch `feat/milestone-3`, tracking `origin/feat/milestone-3`; modified: `crates/core/src/{error,lib,q5}.rs`, `crates/core/tests/{core_foundation,mutation_fixtures,mutation_invariants}.rs`, [docs/decisions/](/decisions/), `tests/fixtures/README.md`; untracked: `crates/core/src/primer.rs`, [docs/specs/q5-primer-design.md](/specs/q5-primer-design.md), [docs/specs/q5-sdm.md](/specs/q5-sdm.md), [docs/research/](/research/), `docs/tasks/contracts/milestone-3{,-b}*.md`, `package.json`, `package-lock.json`, `tests/fixtures/{nebasechanger-research,q5-primer}/`. Nothing staged, committed, discarded or moved |
| `cargo fmt --all -- --check` | clean (exit 0) |
| `cargo build` | `luria-core` built, 0 errors, 0 warnings |
| `cargo test --workspace` | 207 tests, **0 failures** (179 + 8 + 6 + 14, plus 0 doc-tests) |
| `git diff --check` | clean |
| `cargo clippy --all-targets` | **16 warnings, 0 errors**, all in `crates/core/src/primer.rs`: 11 × `len() == 0` (`len_zero`), 4 × owned instance for comparison, 1 × comparison against `false`. Pre-existing: this pass wrote no Rust. Recorded rather than fixed, since fixing them is outside this task |

Documentation checks:

| Check | Result |
| --- | --- |
| `rg -n "availability-aware" docs/` | 4 files, 24 lines: this report (6), `q5-primer-policy-review-2026-10-12.md` (6: head note, A8, D-5, Part D item 4, Part E's D029 bullet, handoff bullet), [docs/decisions/](/decisions/) (4: status line, Reason, Consequences, Reversibility), [docs/specs/q5-primer-design.md](/specs/q5-primer-design.md) (8: §18.1 superseded note ×2, three §21 rows, the note beneath that table, two §22 rows). Every statement of the rule that is **live** is the centered one; every availability-aware statement is labelled superseded or records the earlier pass |
| `rg -n "arbitrary among ties" docs/` | 4 lines, all in changed-or-superseded context: this report (2), the 2026-10-12 report's head note (1), [docs/specs/q5-primer-design.md](/specs/q5-primer-design.md) §22's row saying the rule was replaced (1). No live rule calls the split arbitrary |
| centered rule present where the rule belongs | [docs/specs/q5-primer-design.md](/specs/q5-primer-design.md) §1 table, §18.1, §18.4, §22; [docs/decisions/](/decisions/) D029 |
| D029 status | "proposed, not yet approved" stated in the entry; the file's global approval header unchanged |
| No Rust touched | `git status` shows the same Rust files as before the pass; `crates/core/src/primer.rs` still untracked and byte-identical (not opened for editing) |

---

## Part 8 — Outstanding issues

1. **D029 needs human approval** as amended. Until then the centered split is a
   contract proposal, and Part 4's P0-3 gates the implementation.
2. **The extra base of an odd `a_f` on the 5′ side** is a convention, not a
   finding. If a human prefers the 3′ side, that is a policy change needing
   approval (stated in D029's Reversibility).
3. **`maxAnnealingOverlap`'s role** under the initial native policy is
   undefined (Part 3 item 3, P2-2).
4. **The error carrier and its category** for §18.5 outcomes 1 and 2 (P0-2).
5. **§15 question 3** (must a pair abut?) is still open and still unenforced
   (P2-3).
6. **Non-contiguous changed sets** have no centered definition and are out of
   the initial native scope; if they ever come in, the rule must be stated, not
   extrapolated.
7. **The clippy warnings in `primer.rs`** are pre-existing and unaddressed; the
   2026-10-12 report recorded the same finding.
8. **No compatibility claim was made or retired.** The centered rule's direction
   agrees with the reference's short-substitution path; that agreement is
   evidence about the reference path only, and C1–C25 remain as the 2026-10-12
   report left them.
