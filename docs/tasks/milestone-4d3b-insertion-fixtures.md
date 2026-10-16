# Milestone 4d-3b task contract: insertion fixture tests and the 100-nt warning

Status: **Implemented 2026-10-15** by the local worker; accepted on planner review (§6).
Date: 2026-10-15
Planner: Claude. Parent: `docs/tasks/milestone-4-indels.md`, pass 4.

## 1. Goal

1. Check every row of `tests/fixtures/q5-primer/native-insertion.tsv` against
   luria. The planner generated the file with Biopython before any insertion
   design code existed.
2. Add D032's 100-nt warning to `primer_quality.rs`.

## 2. Requirements, inline

- **D032:** there is no cap on insertion length, but any primer longer than
  **100 nt** gets a quality warning and is still emitted. The limit is the
  owner's practical limit for ordering a standard oligo from IDT, 2026-10-15.
- The planner has added the row `MAX_ORDERABLE_LENGTH 100` to
  `tests/fixtures/primer-quality/thresholds.tsv`, and its source to that
  fixture's README. Until §3.1 is done, `every_threshold_equals_the_fixture`
  fails because the fixture has 8 thresholds and the Rust has 7. That failure
  is expected at the start of this task.

## 3. Required changes

### 3.1 `crates/core/src/primer_quality.rs`

- `pub const MAX_ORDERABLE_LENGTH: u64 = 100;` with a doc comment citing D032
  and the owner.
- `PrimerWarning::AboveOrderingLimit { length: u64 }`. `describe()` returns
  `"<n> bases, longer than the <limit> nt a standard oligo can be ordered at"`.
- In `inspect`, push it right after the length-guideline check when
  `length > MAX_ORDERABLE_LENGTH`. A 120-nt primer therefore draws both
  `LengthOutsideGuideline` and `AboveOrderingLimit`, which is intended: one is
  a PCR guideline, the other a vendor limit.
- Update the module doc and `inspect`'s doc about the fixed order: length,
  ordering limit, G+C, then sequence features.
- Unit tests: 100 nt draws no `AboveOrderingLimit`; 101 nt draws
  `AboveOrderingLimit { length: 101 }`. Build each primer by repeating
  `"ATCG"` and truncating.

### 3.2 `crates/core/tests/primer_quality_thresholds.rs`

Add `("MAX_ORDERABLE_LENGTH", MAX_ORDERABLE_LENGTH as f64)` to the expected
list and its import. In `the_fixture_bands_are_coherent`, add
`assert!(value("LENGTH_HIGH") < value("MAX_ORDERABLE_LENGTH"))`.

### 3.3 `crates/core/tests/q5_primer_fixtures.rs`

Mirror the existing deletion-fixture code (`deletion_rows`,
`deletion_template`, `deletion_edit`, `deletion_design` and its three tests).

Columns are documented in `tests/fixtures/q5-primer/README.md`, "Native
insertion designs". Header: `case_id, template, point, inserted, min_overlap,
expected, forward_anneal_span, reverse_anneal_span, forward_tail,
reverse_tail, forward, reverse, forward_tm, reverse_tm, forward_whole_tm,
reverse_whole_tm, annealing_temp, balance_met, failure, note`. 15 rows: 11
designs and 4 errors.

Build each edit with `NucleotideInsertionIntent::create(template.reference(),
create_point(&template.reference(), p), letters[p-1], letters[p], inserted)`,
reading the flanking bases from the template, then `apply_insertion(…, None,
…)` and `verify(…, None, …)`.

| # | Test | Expect |
|---|---|---|
| F1 | `native_insertion_tsv_has_every_case_once` | 15 rows, distinct ids |
| F2 | `each_native_insertion_case_gets_the_design_or_failure_d032_predicts` | design rows: forward and reverse **annealing** spans (`pair.forward().span()`, `pair.reverse().span()`); each tail's text, or `none` when `tail()` is `None`; both whole primer texts exactly; `forward_tm`, `reverse_tm`, `forward_whole_tm`, `reverse_whole_tm` and `annealing_temp` within 0.006; `balance_met`. Error rows: the variant and the `failure` text, reusing `obstacle_text` and `pair:<smallest>><length>` |
| F3 | `every_native_insertion_design_meets_the_d032_invariants` | for the 11 designs: both annealing Tms ≥ `TM_FLOOR`; Ta = min(`TA_CAP`, min annealing Tm + `TA_OFFSET`); `reverse.span().end() == forward.span().start() == p`; `rc(reverse tail) + forward tail == inserted`; for `k ≤ 6` the reverse primer has no tail; both bases `MatchedComplement`. Assert exactly 11 checked |
| F4 | `a_primer_past_the_ordering_limit_is_warned_about_and_still_emitted` | `ins-13`: a design is returned; both primers are 120 nt; `forward_warnings()` and `reverse_warnings()` each contain `AboveOrderingLimit { length: 120 }` |

## 4. Do not

- Do not change production code outside `primer_quality.rs`, or any fixture,
  or `docs/`. If a fixture row disagrees with luria, stop and report the case
  id, the field and both values.

## 5. Checks

The five in `AGENTS.md`, with `export TMPDIR=$PWD/target/tmp`. Expected: 387
tests (381 + 6), none failing.

## 6. Outcome, 2026-10-15

One worker run, cut off by the harness's time limit before it wrote a handoff,
but with the work complete. The planner checked it directly: fmt, clippy
`-D warnings` and `git diff --check` pass, and **387 tests pass (381 + 6), 0
failures**.

**All 15 rows of `native-insertion.tsv` agree with luria**: annealing spans,
both tails, both whole primer texts, annealing and whole-primer Tms, Ta,
balance, and failure detail. The fixture was generated with Biopython before
any insertion design code existed.

**Mutation check.** Moving the whole-tail threshold to 7 or splitting with
ceil each fails both U2 and the fixture test. Moving the ordering limit to 125
fails three tests, including the fixture-equality test, so the fixture still
governs the constant.
