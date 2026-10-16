# Milestone 4d-4 task contract: insertions in the Q5 workflow

Status: **Implemented 2026-10-15** by the local worker; accepted on planner review (§7).
Date: 2026-10-15
Planner: Claude. Parent: `docs/tasks/milestone-4-indels.md`, pass 4. Mirrors
`milestone-4c1-workflow-deletion.md` and `milestone-4c2-design-deletion-run.md`,
both implemented. Follow their code in `q5.rs` and `q5_design.rs`.

## 1. Goal

A Q5 request can ask for an insertion. The workflow applies and verifies it,
and warns when it shifts the resolved CDS's reading frame. A run reaches a
D032 primer pair through `design_run` and `design_set`.

## 2. Requirements, inline

- **Owner decision, 2026-10-15 (recorded under D032):** an insertion of `k`
  bases inside a CDS, with `k` not a multiple of 3, shifts the reading frame.
  luria **warns and still applies it**. "Inside" means some interval of the
  resolved CDS has `start < p < end`. An insertion exactly at a CDS edge does
  not warn.
- Milestone 4d-1 API: `NucleotideInsertionIntent`, `apply_insertion`,
  `MutationResult::is_insertion()`, `MutationResult::Insertion(record)` with
  `record.point().position()` and `record.inserted()`.

## 3. Required changes

### 3.1 `crates/core/src/q5.rs`

1. `Q5MutationIntent` gains `Insertion(NucleotideInsertionIntent)`. Every
   exhaustive match gains an arm that mirrors `Deletion`'s exactly
   (`named_cds`, `source_reference`, `missing_claim`, and `run_resolved`'s
   operation and `verify_scope`), calling `apply_insertion`.
2. `Q5EditWarning` gains
   `InsertionFrameshift { cds: Identifier, bases_inserted: u64 }`. Its
   `describe()` returns `"inserting <k> bases into <cds> shifts its reading
   frame"`.
3. `edit_warnings` returns that warning for an insertion when §2's rule holds,
   and nothing otherwise. The deletion branch is unchanged.
4. `Q5_WORKFLOW_VERSION` becomes `"0.3.0"`.

### 3.2 `crates/core/src/q5_design.rs`

Tests only. No production change should be needed. If one is, stop and
report.

## 4. Required tests

**In `q5.rs`'s test module**, on `template()` (`"ATGGCTTGGAATCAGCAGCAGCAGCAGCAG"`).
Add a helper `insertion_intent(p, before, after, inserted)` and a variant on
another artifact for X5. Values derived by plain string slicing (planner,
2026-10-15): at `p = 9` the flanks are `G`/`A`; at `p = 12` they are `T`/`C`.

| # | Test | Setup | Expect |
|---|---|---|---|
| X1 | an in-frame insertion has no warning | `cds_of("cds-1", 30)`, insert `"GGG"` at 9 | verified; `mutation().is_insertion()`; edited `ATGGCTTGGGGGAATCAGCAGCAGCAGCAGCAG`; `warnings()` empty |
| X2 | a one-base insertion warns of a frameshift | `cds_of("cds-1", 30)`, `"G"` at 9 | edited `ATGGCTTGGGAATCAGCAGCAGCAGCAGCAG`; `warnings() == [InsertionFrameshift { cds-1, 1 }]` |
| X3 | four bases warn too | `"GGGG"` at 9 | edited `ATGGCTTGGGGGGAATCAGCAGCAGCAGCAGCAG`; `[InsertionFrameshift { cds-1, 4 }]` |
| X4 | an insertion at the CDS edge does not warn | `cds_of("cds-1", 12)`, `"G"` at 12, no scope | verified; `warnings()` empty |
| X5 | a scoped insertion at the CDS edge is refused | as X4 with scope `Some(cds_name("cds-1"))` | `TargetOutsideFeature` |
| X6 | an insertion intent on another artifact is refused | intent on `decoy()` at 9, with `decoy()`'s own bases at 8 and 9 read in the test | `SequenceReferenceMismatch` |

**In `q5_design.rs`'s test module**, on its 90-base template (`ATG` ×14,
`GCT`, `ATG` ×15) with `cds_of(&template(), "cds-1")` and
`design_request(15)`. The flanks at 45 are `T`/`A`. Values from
`native-insertion.py`'s `design()` with Biopython 1.88, run by the planner
before this code existed. Tms within 0.006.

| # | Test | Expect |
|---|---|---|
| Y1 | an in-frame insertion run reaches a D032 pair | `"GCT"` at 45: forward annealing span `45:66`, reverse `26:45`; forward primer `GCTATGATGATGATGATGATGATG`, reverse `AGCCATCATCATCATCATC`; forward tail `GCT`, reverse none; Tms 56.52 / 58.66; whole Tms 60.54 / 58.66; Ta 59.52; `run().warnings()` empty |
| Y2 | a frameshift insertion is still designed, with its warning | `"G"` at 45: same spans; forward primer `GATGATGATGATGATGATGATG`; whole forward Tm 57.83; `run().warnings() == [InsertionFrameshift { cds-1, 1 }]` |
| Y3 | a set of insertions designs every member | both as members `i1` and `i2` (`SeparateVariants`): `designed_count() == 2`, `all_designed()` |

## 5. Do not

- Do not touch `mutation.rs`, `primer.rs`, `primer_design.rs`,
  `primer_quality.rs`, any fixture or `docs/`. No new error variants.
- Do not change the deletion branch of `edit_warnings` or any existing test.

## 6. Checks

The five in `AGENTS.md`, with `export TMPDIR=$PWD/target/tmp`. Expected: 396
tests (387 + 9), none failing.

## 7. Outcome, 2026-10-15

Two worker runs. The first did §3.1 and X1–X6, then was cut off by the
harness's time limit; it left one unused module-level import (`create_point`
at `q5.rs:106`; the test module imports it separately). A fresh worker did
Y1–Y3 in `q5_design.rs`'s test module, found that the import broke
`clippy -D warnings`, and correctly stopped instead of editing a file outside
its scope.

**Planner direct edit:** the planner deleted that one unused import line. That
is the one-line fix AGENTS.md permits the planner; no behaviour changed.

Planner-run checks: fmt, clippy `-D warnings` and `git diff --check` pass, and
**396 tests pass (387 + 9), 0 failures**. Y1 and Y2 match the values the
planner derived from `native-insertion.py` before the code existed.

**Mutation check** on `insertion_warnings`: counting a CDS edge as inside fails
X4; warning on in-frame insertions too fails X1 and Y1.
