# Milestone 4e-4b task contract: degenerate replacement requests

Status: **Implemented 2026-10-15** by the local worker; accepted on planner review (§8).
Date: 2026-10-15
Planner: Claude. Parent: [docs/tasks/contracts/milestone-4-indels.md](/tasks/contracts/milestone-4-indels.md), pass 5. Uses
4e-1's `expand_degenerate` and 4e-4a's replacement intent. The last contract
of pass 5.

## 1. Goal

A caller can ask for a replacement containing IUPAC ambiguity codes. luria
expands it into a variant set, one member per concrete sequence, which then
runs through the existing `q5::execute_set` and `q5_design::design_set`
unchanged.

## 2. Requirements, inline (D033 and its addenda, owner, 2026-10-15)

- **Expansion.** Each ambiguity code expands over the bases it stands for, in
  `A, C, G, T` order, leftmost position varying fastest: `NS` gives
  `AC, CC, GC, TC, AG, CG, GG, TG`. `sequence::expand_degenerate` already does
  exactly this; call it, do not reimplement it.
- **One member per concrete sequence**, in expansion order, composition
  `SeparateVariants`. Each member is an ordinary, non-degenerate replacement
  request; `mutation.rs` never sees an ambiguity code.
- **Naming.** Member result name = base name, `-`, concrete sequence:
  `mut-1-AC`.
- **Wild-type members.** A concrete sequence equal to the region's current
  bases (`expected_original`) is **skipped with a warning** naming it. If no
  member is left, the request is refused.
- **Count warning.** When the number of members actually emitted (after
  skipping) is **greater than 64**, the expansion carries a warning. Every
  member is still emitted; there is no cap.
- **No nesting.** A degenerate replacement inside an existing variant set is
  refused. This already holds: `NucleotideReplacementIntent::create` refuses
  ambiguity codes. Nothing to add; test X9 pins it.

## 3. Required changes

### 3.1 New module `crates/core/src/q5_expand.rs`

Register it in `crates/core/src/lib.rs` as `pub mod q5_expand;`
(alphabetical, after `q5_design`).

```rust
/// D033: past this many emitted members an expansion carries a warning.
pub const EXPANSION_WARNING_THRESHOLD: usize = 64;

pub enum Q5ExpansionWarning {
    WildTypeSkipped { sequence: String },
    ManyDesigns { count: usize, threshold: usize },
}
impl Q5ExpansionWarning {
    pub fn describe(&self) -> String;
}

pub struct Q5Expansion {
    set: Q5VariantSetRequest,
    warnings: Vec<Q5ExpansionWarning>,
}
impl Q5Expansion {
    pub fn set(&self) -> Q5VariantSetRequest;
    pub fn warnings(&self) -> Vec<Q5ExpansionWarning>;
}

pub fn expand_replacement(
    template: Sequence,
    scope: Option<Identifier>,
    region: NucleotideInterval,
    expected_original: String,
    degenerate: String,
    base_name: Identifier,
) -> Result<Q5Expansion, DomainError>;
```

Derive `Clone, Debug, PartialEq` as the neighbouring `q5.rs` types do.

`describe()`:
- `WildTypeSkipped`: `"<sequence> equals the bases already in the region and was skipped"`
- `ManyDesigns`: `"<count> designs, more than <threshold>; every one is still emitted"`

`expand_replacement`, in order:

1. `degenerate` empty → `InvalidMutationRequest`, `request:
   "q5-degenerate-replacement"`, reason containing `"at least one"`.
2. `expand_degenerate(&degenerate)?` (an invalid letter is its
   `InvalidSequenceCharacters`, propagated unchanged).
3. For each concrete sequence, in order:
   - if it equals `expected_original`, push
     `WildTypeSkipped { sequence }` and continue;
   - otherwise build `NucleotideReplacementIntent::create(template.reference(),
     region.clone(), expected_original.clone(), concrete.clone())?`, then
     `Q5SdmRequest::create(template.clone(), scope.clone(),
     Q5MutationIntent::Replacement(intent))?`, then the name
     `Identifier::create(format!("{}-{}", base_name.value(), concrete))?`, and
     push `Q5Variant::create(request, name)`.
4. No members → `InvalidMutationRequest`, same `request`, reason containing
   `"already there"`.
5. More than `EXPANSION_WARNING_THRESHOLD` members → push
   `ManyDesigns { count, threshold: EXPANSION_WARNING_THRESHOLD }` **after** any
   `WildTypeSkipped` warnings.
6. `Q5VariantSetRequest::create(Q5Composition::SeparateVariants, variants)?`.

No new `DomainError` variants. Do not edit `error.rs`.

## 4. Required tests, in `q5_expand.rs`'s own test module

On `"ATGGCTTGGAATCAGCAGCAGCAGCAGCAG"` (30 nt, linear, named as `q5.rs`'s
`template()` is), no scope unless stated, base name `mut-1`. Values derived
by plain string slicing and an independent Python expansion (planner,
2026-10-15): `[3,5)` is `GC`, `[9,11)` is `AA`, `[9,12)` is `AAT`,
`[9,13)` is `AATC`.

| # | Test | Input | Expect |
|---|---|---|---|
| X1 | NS expands to eight named members in order | `[9,11)` `AA` → `NS` | 8 members; names exactly `mut-1-AC, mut-1-CC, mut-1-GC, mut-1-TC, mut-1-AG, mut-1-CG, mut-1-GG, mut-1-TG`; composition `SeparateVariants`; `warnings()` empty |
| X2 | a wild-type member is skipped with a warning | `[3,5)` `GC` → `NS` | 7 members, `mut-1-GC` absent, the other seven in order; `warnings() == [WildTypeSkipped { "GC" }]` |
| X3 | 64 members do not warn | `[9,11)` `AA` → `NNN` | 64 members; `warnings()` empty |
| X4 | the count is taken after skipping | `[9,12)` `AAT` → `NNN` | 63 members; `warnings() == [WildTypeSkipped { "AAT" }]` |
| X5 | more than 64 warns, and still emits all | `[9,11)` `AA` → `NNNS` | 128 members; `warnings() == [ManyDesigns { 128, 64 }]` |
| X6 | warnings are ordered | `[9,13)` `AATC` → `NNNN` | 255 members; `warnings() == [WildTypeSkipped { "AATC" }, ManyDesigns { 255, 64 }]` |
| X7 | a concrete replacement is a one-member set | `[9,11)` `AA` → `GG` | 1 member `mut-1-GG`; no warnings |
| X8 | nothing left is refused | `[9,12)` `AAT` → `AAT` | `InvalidMutationRequest`, reason contains `already there` |
| X9 | a degenerate member cannot be built directly | `NucleotideReplacementIntent::create` on `[9,11)` with replacement `NS` | `InvalidMutationRequest` (the no-nesting rule, already true) |
| X10 | empty and invalid texts are refused | `""`; then `"NX"` | `InvalidMutationRequest` with `at least one`; then `InvalidSequenceCharacters` |
| X11 | scope reaches every member | X1 with scope `Some("cds-1")` | every member's `variant.request().scope()` is `Some("cds-1")` |
| X12 | an expansion designs end to end | the 90-base template of `q5_design.rs`'s tests (`ATG` ×14, `GCT`, `ATG` ×15) with its CDS built as those tests do; base name `r`; `[45,48)` `ATG` → `TGS`; then `q5::execute_set(set, vec![cds])` and `q5_design::design_set(run, design_request(15))` built as `q5_design.rs`'s tests do | 2 members named `r-TGC`, `r-TGG`; `designed_count() == 2`; `all_designed()` |

Copy the small helpers X12 needs from `q5_design.rs`'s test module into this
module's tests; do not make them public.

## 5. Do not

- Do not touch any existing file except `lib.rs` (one line). In particular,
  not `q5.rs`, `q5_design.rs`, `mutation.rs`, `sequence.rs`, `error.rs`,
  fixtures or [docs/](/).
- No new dependencies, no new error variants, no new public accessors on
  existing types.

## 6. Checks

The five in `AGENTS.md`, with `export TMPDIR=$PWD/target/tmp`. Expected: 462
tests (450 + 12), none failing.

## 7. Handoff

Changed files, exact check results, added test names, anything unclear. If the
contract is ambiguous or self-contradictory, stop and report.

## 8. Outcome, 2026-10-15

One worker run. Planner-run checks: fmt, clippy `-D warnings` and
`git diff --check` pass; **462 tests pass (450 + 12), 0 failures**. Only the
new `q5_expand.rs` and one line of `lib.rs` changed.

**Mutation check, planner-run** (`cargo test --lib q5_expand`):

- removing the wild-type skip fails X2, X4, X6 and X8;
- warning at 64 instead of past 64 fails X3;
- joining names with `_` instead of `-` fails X1, X2, X7 and X12.

Counting members before skipping instead of after fails nothing, and cannot
be made to: expansion totals are products of set sizes 1–4, so always
`2^a · 3^b`, and at most one member can equal the wild type. The two counts
cross the threshold differently only at a total of exactly 65, which no
expansion produces. The rule is recorded in D033 for clarity, but it has no
observable consequence today.

Pass 5 (D033) is complete with this contract.
