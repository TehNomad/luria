---
type: Task Contract
title: "Milestone 4e-4a task contract: replacements in the Q5 workflow"
description: "A Q5 request can ask for a replacement."
status: deprecated
tags: [milestone-4]
---
# Milestone 4e-4a task contract: replacements in the Q5 workflow

Status: **Implemented 2026-10-15** by the local worker; accepted on planner review (§8).
Date: 2026-10-15
Planner: Claude. Parent: [docs/tasks/contracts/milestone-4-indels.md](/tasks/contracts/milestone-4-indels.md), pass 5. Mirrors
`milestone-4d4-workflow-insertion.md` (implemented): follow its code in
`q5.rs` and `q5_design.rs`. Degenerate expansion is 4e-4b, not this task.

## 1. Goal

A Q5 request can ask for a replacement. The workflow applies and verifies it,
warns when it shifts the resolved CDS's reading frame, and a run reaches a
D033 primer pair through `design_run` and `design_set`.

## 2. Requirements, inline

- **Owner decision, 2026-10-15 (D033 addenda):** a replacement inside a CDS
  whose length change `m − w` is not a multiple of 3 carries a non-blocking
  frameshift warning and **still applies**, as for deletions and insertions.
- **Planner rule, following the approved deletion precedent** (`edit_warnings`'
  deletion branch): count `coding`, the region positions that lie inside some
  interval of the resolved CDS. No warning when `coding == 0` or when
  `m − w` is a multiple of 3 (including 0, and negative multiples). Otherwise
  warn, with `wholly_inside = (coding == w)`.
- 4e-2 API: `NucleotideReplacementIntent`, `apply_replacement`,
  `MutationResult::is_replacement()`, `MutationResult::Replacement(record)`
  with `record.region()` and `record.replacement()`.

## 3. Required changes

### 3.1 `crates/core/src/q5.rs`

1. `Q5MutationIntent` gains `Replacement(NucleotideReplacementIntent)`. Every
   exhaustive match gains an arm mirroring `Deletion`'s exactly (`named_cds`,
   `source_reference`, `missing_claim`, and `run_resolved`'s operation and
   `verify_scope`), calling `apply_replacement`.
2. `Q5EditWarning` gains
   `ReplacementFrameshift { cds: Identifier, bases_replaced: u64, bases_written: u64, wholly_inside: bool }`.
   Its `describe()` returns `"replacing <w> bases with <m> in <cds> shifts its
   reading frame"`, followed by `"; the replacement also extends outside the
   CDS"` when `wholly_inside` is false.
3. `edit_warnings` dispatches a replacement to a new private
   `replacement_warnings(mutation, target)` implementing §2's rule, written in
   the style of `insertion_warnings`. The deletion and insertion branches are
   unchanged. `insertion_warnings`' `Replacement` arm stays `Vec::new()`.
4. `Q5_WORKFLOW_VERSION` becomes `"0.4.0"`. If a test pins `"0.3.0"`, update
   that one string; that is a permitted change.

### 3.2 `crates/core/src/q5_design.rs`

Tests only. No production change should be needed. If one is, stop and
report.

## 4. Required tests

**In `q5.rs`'s test module**, on `template()`
(`"ATGGCTTGGAATCAGCAGCAGCAGCAGCAG"`). Add helpers
`replacement_intent_on(artifact, start, end, expected, replacement)` and
`replacement_intent(start, end, expected, replacement)` beside the insertion
ones. Values derived by plain string slicing (planner, 2026-10-15):
`[9,12)` is `AAT`, `[10,14)` is `ATCA`, `[15,18)` is `CAG`; on `decoy()`,
`[9,12)` is `CAT`.

| # | Test | Setup | Expect |
|---|---|---|---|
| Z1 | an equal-length replacement has no warning | `cds_of("cds-1", 30)`, `[9,12)` `AAT` → `GGC` | verified; `mutation().is_replacement()`; edited `ATGGCTTGGGGCCAGCAGCAGCAGCAGCAG`; `warnings()` empty |
| Z2 | a shorter replacement warns of a frameshift | `[9,12)` `AAT` → `GG` | edited `ATGGCTTGGGGCAGCAGCAGCAGCAGCAG`; `warnings() == [ReplacementFrameshift { cds-1, 3, 2, true }]`; its `describe()` is `replacing 3 bases with 2 in cds-1 shifts its reading frame` |
| Z3 | a change of 3 does not warn | `[9,12)` `AAT` → `GGGGGG` | edited `ATGGCTTGGGGGGGGCAGCAGCAGCAGCAGCAG`; `warnings()` empty |
| Z4 | a longer replacement warns too | `[9,12)` `AAT` → `GGGG` | edited `ATGGCTTGGGGGGCAGCAGCAGCAGCAGCAG`; `[ReplacementFrameshift { cds-1, 3, 4, true }]` |
| Z5 | a replacement across the CDS edge warns and says so | `cds_of("cds-1", 12)`, **no scope**, `[10,14)` `ATCA` → `GG` | edited `ATGGCTTGGAGGGCAGCAGCAGCAGCAG`; `[ReplacementFrameshift { cds-1, 4, 2, false }]`; `describe()` ends `; the replacement also extends outside the CDS` |
| Z6 | a scoped replacement across the CDS edge is refused | as Z5 with scope `Some(cds_name("cds-1"))` | `TargetOutsideFeature` |
| Z7 | a replacement outside the CDS does not warn | `cds_of("cds-1", 12)`, no scope, `[15,18)` `CAG` → `G` | edited `ATGGCTTGGAATCAGGCAGCAGCAGCAG`; `warnings()` empty |
| Z8 | a replacement intent on another artifact is refused | intent on `decoy()`, `[9,12)` `CAT` → `GG` | `SequenceReferenceMismatch` |

Set up scope and CDS exactly as the insertion tests X1–X5 do.

**In `q5_design.rs`'s test module**, on its 90-base template (`ATG` ×14,
`GCT`, `ATG` ×15), `cds_of(&template(), "cds-1")`, `design_request(15)`.
`[45,48)` is `ATG`. Values from `native-replacement.py`'s `design()` with
Biopython 1.88, run by the planner before this code existed. Tms within 0.006.

| # | Test | Expect |
|---|---|---|
| W1 | an inline replacement run reaches a D033 pair | `[45,48)` `ATG` → `TGGTGG`: forward span `35:58`, reverse `5:35`; forward primer `GATGATGGCTTGGTGGATGATGATGA`, reverse `ATCATCATCATCATCATCATCATCATCATC`; no tails; Tms 68.70 / 64.00; forward basis `matched_complement`; Ta 67.00; `run().warnings()` empty |
| W2 | a frameshift replacement is still designed, with its warning | `[45,48)` `ATG` → `TG`: forward span `35:60`, reverse `11:35`; forward primer `GATGATGGCTTGATGATGATGATG`, reverse `ATCATCATCATCATCATCATCATC`; Tms 63.36 / 59.48; Ta 62.48; `run().warnings() == [ReplacementFrameshift { cds-1, 3, 2, true }]` |
| W3 | a long replacement run reaches a tail pair | `[45,48)` `ATG` → `TGGTGGTGG`: forward annealing span `48:69`, reverse `26:45`; forward primer `GGTGGATGATGATGATGATGATGATG` with tail `GGTGG`, reverse `ACCAAGCCATCATCATCATCATC` with tail `ACCA`; Tms 56.52 / 58.66; whole Tms 65.03 / 64.69; Ta 59.52; `run().warnings()` empty |
| W4 | a set of replacements designs every member | W1's and W3's edits as members `r1` and `r2` (`SeparateVariants`): `designed_count() == 2`, `all_designed()` |

## 5. Do not

- Do not touch `mutation.rs`, `primer.rs`, `primer_design.rs`,
  `primer_quality.rs`, any fixture or [docs/](/). No new error variants.
- Do not change the deletion or insertion warning logic or any existing test,
  except the version string in §3.1 item 4.
- If adding the `Q5MutationIntent` variant forces an arm in a match in a file
  not named here, add one arm mirroring `Insertion`'s and report it. (That
  has happened in three earlier contracts.)

## 6. Checks

The five in `AGENTS.md`, with `export TMPDIR=$PWD/target/tmp`. Expected: 450
tests (438 + 8 + 4), none failing.

## 7. Handoff

Changed files, exact check results, added test names, anything unclear. If
the contract is ambiguous or self-contradictory, stop and report.

## 8. Outcome, 2026-10-15

One worker run. Planner-run checks: fmt, clippy `-D warnings` and
`git diff --check` pass; **450 tests pass (438 + 12), 0 failures**. Only
`q5.rs` and `q5_design.rs` changed; no match outside `q5.rs` needed an arm,
and no test pinned the old workflow version.

**Mutation check, planner-run.** Each part of §2's warning rule fails a test
when removed: dropping the `coding == 0` exemption fails Z7; dropping the
multiple-of-3 exemption fails Z1, Z3, W1 and W3; forcing `wholly_inside` to
true fails Z5.
