---
type: Task Contract
title: "Milestone 4c-2 task contract: deletion runs reach a primer pair"
description: "Prove, in tests, that a deletion requested through the Q5 workflow reaches a D031 primer pair through design_run and design_set."
status: deprecated
tags: [milestone-4]
---
# Milestone 4c-2 task contract: deletion runs reach a primer pair

Status: **Implemented 2026-10-15** by the local worker; accepted on planner review (§7).
Date: 2026-10-15
Planner: Claude. Parent: [docs/tasks/contracts/milestone-4-indels.md](/tasks/contracts/milestone-4-indels.md).

## 1. Goal

Prove, in tests, that a deletion requested through the Q5 workflow reaches a
D031 primer pair through `design_run` and `design_set`. A frameshift warning
does not stop the design. No production code should need to change: 4b-2
taught `design_primers` deletions and 4c-1 taught the workflow. If a test
cannot pass without a production change, stop and report it.

## 2. Expectations

The 90-base template in `q5_design.rs`'s tests is `ATG` ×14, `GCT`, `ATG` ×15,
with `cds_of(&template(), "cds-1")` over all of it. The values below come from
`tests/fixtures/q5-primer/reference/native-deletion.py`'s `design()` with
Biopython 1.88, run by the planner on 2026-10-15 before this code existed:

| Deletion | Forward span, text | Reverse span, text | Tm fwd / rev / Ta |
|---|---|---|---|
| `[45,48)` `"ATG"` | `48:69` `ATGATGATGATGATGATGATG` | `26:45` `AGCCATCATCATCATCATC` | 56.52 / 58.66 / 59.52 |
| `[46,47)` `"T"` | `47:69` `GATGATGATGATGATGATGATG` | `26:46` `TAGCCATCATCATCATCATC` | 57.83 / 58.37 / 60.83 |

## 3. Required tests, in `crates/core/src/q5_design.rs`'s test module

Use the existing `designed(intent, scope)` helper and `design_request(15)`.
Build intents as `Q5MutationIntent::Deletion(NucleotideDeletionIntent::create(
template().reference(), create(&template().reference(), s, e).unwrap(),
bases).unwrap())`.

| # | Test | Expect |
|---|---|---|
| R1 | an in-frame deletion run reaches a D031 pair | row 1: both spans and texts exactly; Tms and Ta within 0.006; `design.deleted_width() == 3`; `design.pair().abuts()`; `run().warnings()` empty |
| R2 | a frameshift deletion is still designed, with its warning | row 2: both spans and texts; `run().warnings()` has one `Q5EditWarning::Frameshift` with `coding_bases_removed == 1`, `wholly_inside == true` |
| R3 | a set of deletions designs every member | `execute_set` with both deletions as members `d1` and `d2` (`SeparateVariants`), then `design_set(…, design_request(15))`: `designed_count() == 2`, `all_designed()`, and member forward spans `48:69` then `47:69`. Mirror the existing set tests in this module. |

## 4. Do not

- Do not change production code in any file, or any existing test or fixture.
- Do not edit [docs/](/).

## 5. Checks

The five in `AGENTS.md`, with `export TMPDIR=$PWD/target/tmp`. Expected: 348
tests (345 + 3), none failing.

## 6. Handoff

Report changed files, exact check results, the added test names, and anything
unclear.

## 7. Outcome, 2026-10-15

One worker run, accepted. **No production code changed**, which was the point:
`design_primers` already designed deletions (4b-2) and the workflow already
requested them (4c-1), so wiring them together needed only tests.

Planner-run checks: fmt, clippy `-D warnings` and `git diff --check` pass,
**348 tests pass (345 + 3), 0 failures**. Both designs match the spans, primer
texts and Tms the planner derived from `native-deletion.py` before the tests
existed. Only `q5_design.rs`'s test module changed.

Accepted additions beyond §3: the worker also pinned `edit_span()`,
`deleted_width()` and `pair().abuts()` in R2, and that the set's two members
differ in R3.
