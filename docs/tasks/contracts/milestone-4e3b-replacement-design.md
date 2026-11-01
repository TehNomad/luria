# Milestone 4e-3b task contract: design a replacement (D033)

Status: **Implemented 2026-10-15** by two local workers; accepted on planner review (§10).
Date: 2026-10-15
Planner: Claude. Parent: [docs/tasks/contracts/milestone-4-indels.md](/tasks/contracts/milestone-4-indels.md), pass 5. Follows
4e-2 (replacement results) and 4e-3a (replacement primers and pairs).

## 1. Goal

`design_primers` designs a verified replacement instead of refusing it, and
matches all 15 rows of `tests/fixtures/q5-primer/native-replacement.tsv`.
Almost all of it is **moving existing code into functions with parameters**,
not new sizing logic, exactly as `size_pair` was extracted in 4b-2.

## 2. Requirements, inline (D033, approved 2026-10-15)

A replacement names a region `[s, e)` of width `w` replaced by `m` bases, with
`t = max(w, m)`.

- **Inline, `t <= 6`:** D030's substitution geometry, unchanged except that
  the forward primer reads the edited artifact, so its text is `m − w` longer
  or shorter than its template span `[s − 10, e + v)`. The forward Tm uses
  D030's mismatch-aware basis when `m == w` and the **matched** basis when
  `m != w` (no position-by-position alignment exists then).
- **Tails, `t > 6`:** D032's insertion geometry, with the point split into
  two edges: the forward primer anneals `[e, e + f)` and the reverse
  `[s − r, s)`. The replacement is **always split**: forward tail
  `text[m/2..]`, reverse tail `rc(text[..m/2])`, integer division. When
  `m == 1` the reverse tail is empty and the reverse primer is untailed, as
  an insertion's is. Sizing on annealing portions only; whole-primer Tm
  reported.
- Both paths: 55 °C floor, G/C end below 63 °C, 5 °C balance,
  Ta = min(72, lower Tm + 3).

## 3. Required changes, `crates/core/src/primer_design.rs`

Do the steps in order and run `cargo test --workspace` after each of 1 and 2.
**Steps 1 and 2 change no behaviour: all 433 tests must still pass after
each.** If one does not, stop and report.

### Step 1: extract the inline path

Move the body of `design_primers` that follows the replacement refusal (from
`let length: u64 = template.length();` to the final `Ok(PrimerDesign {…})`)
into

```rust
fn design_inline(template: &Sequence, result: MutationResult,
    request: PrimerDesignRequest, start: u64, end: u64, written: u64)
    -> Result<PrimerDesign, DomainError>
```

`design_primers` then calls `replacement_region` as now and returns
`design_inline(template, result, request, start, end, end - start)`.

Inside, change exactly two `Sizer` fields:

- `forward_anchor: start + written` (was `end`; equal when `written == end − start`);
- `forward_mismatched: written == end - start` (was `true`).

Everything else — `smallest_pair`, availability, ambiguity checks, spans,
`edit_start`/`edit_end` — stays in **template** coordinates and is unchanged.
Add a comment above each changed field citing D033.

### Step 2: generalise the insertion path

Rename `design_insertion`'s body into

```rust
fn design_tailed(template: &Sequence, result: MutationResult,
    request: PrimerDesignRequest, start: u64, end: u64, text: String,
    split: bool) -> Result<PrimerDesign, DomainError>
```

and make `design_insertion` read `(point, inserted)` from the record as now and
return `design_tailed(…, point, point, inserted, width > MAX_WHOLE_TAIL)`
where `width` is the inserted length. In `design_tailed`, every use of
`point` becomes `start` on the reverse side and `end` on the forward side:

| Today | Becomes |
|---|---|
| `smallest_pair = 2 * first` | `(end - start) + 2 * first` |
| `edit_start: point, edit_end: point` (both in `fits` and `PrimerDesign`) | `start`, `end` |
| `point < first` → 5′ obstacle `available: point` | `start < first`, `available: start` |
| `length - point < first` → 3′ obstacle | `length - end < first`, `available: length - end` |
| ambiguity loops `point - first..point` then `point..point + first` | `start - first..start` then `end..end + first` |
| `forward_available` counted from `point` | counted from `end` |
| `reverse_available` counted back from `point` | counted back from `start` |
| `Sizer { forward_start: point, forward_anchor: point, reverse_end: point, … }` | `end`, `end`, `start` |
| spans `[point, point + forward)`, `[point − reverse, point)` | `[end, end + forward)`, `[start − reverse, start)` |
| `if width <= MAX_WHOLE_TAIL` (whole tail on the forward primer) | `if !split` |

With `start == end == point` every row of that table is today's code, which
is why step 2 changes no behaviour.

### Step 3: dispatch a replacement

- Add `pub const MAX_INLINE_REPLACEMENT: u64 = 6;` beside `MAX_WHOLE_TAIL`,
  doc-commented as D033's inline limit.
- Replace the replacement refusal in `design_primers` with: read
  `(record.region().start(), record.region().end(), record.replacement())`
  from `MutationResult::Replacement(record)` (a `match` in the style of
  `design_insertion`'s), then with `w = e − s` and `m` the text's length:
  - `max(w, m) <= MAX_INLINE_REPLACEMENT`: `design_inline(template, result,
    request, s, e, m)`;
  - otherwise `design_tailed(template, result, request, s, e, text, true)`.
  It must stay **before** the `replacement_region` call, as the refusal is.
- `NATIVE_DESIGN_ALGORITHM_VERSION` becomes `"0.5.0"`.
- Update the module's doc comments that say a replacement is refused.

If `assemble_pair` or `require_design_constraints` refuses an inline
replacement pair, **stop and report** the error and the case; do not loosen
either.

## 4. Required tests

### In `primer_design.rs`

| # | Test | Expect |
|---|---|---|
| U1 | replaces R20 (`a_replacement_is_refused_as_out_of_the_native_design_scope`, delete it): `a_one_base_replacement_on_the_tail_path_has_no_reverse_tail`. On `balanced()` (`request(15)`), `[50, 57)` `ATCAGCA` → `T` | design `Ok`; forward primer `TGCAGCCGAGCGGCTC`, forward tail `T`; reverse primer `TCCAAGCCATGCGTCCG` with **no** tail; forward span `[57, 72)`, reverse span `[33, 50)`; forward Tm 70.90, reverse 68.76, forward whole Tm 72.45, Ta 71.76, each within 0.006; `tm_balance_met()` |

U1's values come from `reference/native-replacement.py`'s `design()` run by the
planner on 2026-10-15 (Biopython 1.88), not from luria.

### In `primer.rs` (4e-3a's untested refusal)

| # | Test | Expect |
|---|---|---|
| V9 | `a_replacement_forward_primer_must_cover_the_whole_region`: replacement `[9, 12)` `AAT` → `GG` on `seq_a()`; forward primer at span `[10, 20)`, text `GCAGCAGCA` (edited `[10, 19)`) | `InvalidPrimerCandidate`, reason contains `covers the replaced region` |

### In `crates/core/tests/q5_primer_fixtures.rs`

Mirror the insertion section (`InsertionRow`, `insertion_rows`,
`insertion_template`, `insertion_edit`, `insertion_design`, and the three
insertion tests), reusing `read`, `template_from`, `obstacle_text`,
`span_text` and the `0.006` Tm tolerance. The header is:

`case_id template region replacement min_overlap expected path forward_span reverse_span forward_tail reverse_tail forward reverse forward_tm forward_tm_basis reverse_tm forward_whole_tm reverse_whole_tm annealing_temp balance_met failure note`

`region` is `start:end:replaced bases`; build the edit with
`apply_replacement` and verify it. A `template` starting with `seq:` is
literal sequence text, as in the insertion TSV; otherwise it is a FASTA path
relative to `tests/fixtures/q5-primer/`.

| # | Test | Checks |
|---|---|---|
| X1 | `native_replacement_tsv_has_every_case_once` | 15 rows, 11 with `expected == design`, ids unique |
| X2 | `each_native_replacement_case_gets_the_design_or_failure_d033_predicts` | for designs: both span texts, both whole primer texts, both tail texts (`none` = no tail), both Tms, the forward Tm's **basis** (`TmBasis` name, as the D030 test compares it), both whole Tms (`none` ⇔ `None`), Ta, `balance_met`. For failures: the same two error formats as the insertion test |
| X3 | `every_native_replacement_design_meets_the_d033_invariants` | `path == inline`: forward span contains `[s, e)`, reverse span end == forward span start, neither primer has a tail. `path == tails`: reverse span end == `s`, forward span start == `e`, `rc(reverse tail) + forward tail == replacement`, `pair.deleted_width() == e − s` |

## 5. Permitted changes to existing tests

- The version pin `design.algorithm_version() == "0.4.0"` becomes `"0.5.0"`.
- R20 is deleted and replaced by U1.
Nothing else. If another existing test breaks, stop and report.

## 6. Do not

- Do not touch `mutation.rs`, `q5.rs`, `q5_design.rs`, `tm.rs`, `error.rs`,
  any fixture or [docs/](/). In `primer.rs`, add V9 only.
- Do not change `size_pair`, `Sizer`'s methods, `replacement_region`,
  `MAX_REPLACEMENT_REGION` or any threshold.
- Do not change a fixture expectation. If a row fails, report luria's value
  next to the fixture's.

## 7. Checks

The five in `AGENTS.md`, with `export TMPDIR=$PWD/target/tmp`. Expected: 437
tests (433 − R20 + U1 + V9 + X1 + X2 + X3), none failing.

## 8. Handoff

Changed files, exact check results, added test names, anything unclear. If the
contract is ambiguous or self-contradictory, stop and report.

## 9. Addendum, 2026-10-15: pin the Tm basis choice (U2, X2a)

The planner's mutation check found one D033 rule the tests do not bind:
replacing `forward_mismatched: written == end - start` with `true` fails
nothing. With `m != w` the forward primer is compared against a template site
shifted by `m − w`. On every fixture row that misalignment produces adjacent
mismatches, which have no parameters, so luria falls back to the matched
basis and reaches the same number, recording only a `substituted_because`
note. In a repetitive region, though, the shifted site can match the primer
exactly, and the mutant would then report a mismatch-aware Tm against a
meaningless alignment. Two additions close it:

| # | Where | Test | Expect |
|---|---|---|---|
| U2 | `primer_design.rs` | `an_unequal_inline_replacement_is_sized_on_the_matched_basis`. Template (110 nt) `ATGGCTTGGAATCAGCAGCAGCCGAGCGGCTCGCGGACGCATGGCTTGGAGGGGGGGGGGGGGGGGGGGGGCCGAGCGGCTCGCGGACGCATGGCTTGGAATCAGCAGCA`, i.e. `[MIX, GC, "ATGGCTTGGA", "G" × 20, GC, MIX].concat()`; `request(15)`; replacement `[50, 52)` `GG` → `G` | design `Ok`; forward primer `ATGGCTTGGAGGGGGGGGGGG`, span `[40, 62)`; reverse primer `GCGTCCGCGAGCCGCTC`, span `[23, 40)`; forward Tm 80.53 and reverse 76.28 within 0.006; Ta 72.00; **`forward_tm().basis().name() == "matched_complement"` and `forward_tm().substituted_because().is_none()`** |
| X2a | `q5_primer_fixtures.rs`, inside X2 | for every design row whose replacement length differs from its region width | `design.forward_tm().substituted_because().is_none()`, with a message naming the case: the matched basis was chosen, not fallen back to |

U2's values come from `reference/native-replacement.py`'s `design()`, run by
the planner on 2026-10-15 with Biopython 1.88. The planner confirmed that on
this template the forward primer and the shifted template site are the same
21 letters, so the mutant would report the mismatch-aware basis with no note.

Expected after the addendum: 438 tests (437 + U2), none failing.

### 9.1 Correction, 2026-10-15: U2 as first written did not bind

The worker ran the mutation proof and reported that U2 passed under the
mutant; only X2a failed (on `rep-02`). The worker was right and stopped
rather than changing the test. The planner's §9 reasoning missed that
`tm.rs` reports the **matched** basis whenever primer and site are equal
(`tm.rs`, `if primer == site`), so on a site identical to the primer the
mutant gives the correct answer. X2a alone closes the gap; U2 is kept as a
second, fixture-independent pin, with a replacement chosen so it binds.

**U2 now replaces `[50, 52)` `GG` with `C`** (not `G`), on the same template.
The forward primer is `ATGGCTTGGACGGGGGGGGGG`; the template site shifted by
`m − w` is `ATGGCTTGGAGGGGGGGGGGG`, differing at one interior index (10), a
single mismatch with published parameters. The mutant would therefore report
a mismatch-aware 71.71 °C; D033 requires the matched basis.

Expectations, from `reference/native-replacement.py`'s `design()` (planner,
2026-10-15, Biopython 1.88): forward primer `ATGGCTTGGACGGGGGGGGGG`, span
`[40, 62)`; reverse `GCGTCCGCGAGCCGCTC`, span `[23, 40)`; forward Tm **80.68**,
reverse 76.28, Ta 72.00, each within 0.006; basis `matched_complement`;
`substituted_because()` is `None`. Rename U2 to
`an_unequal_inline_replacement_is_sized_on_the_matched_basis` (unchanged).

## 10. Outcome, 2026-10-15

Split across two worker runs to stay inside the harness's time limit: part 1
(§3, U1, V9, the version pin) and part 2 (X1–X3, then the §9 addendum).

Planner-run checks: fmt, clippy `-D warnings` and `git diff --check` pass;
**438 tests pass, 0 failures** (433 − R20 + U1 + U2 + V9 + X1 + X2 + X3).

- **Steps 1 and 2 were pure moves.** The worker ran the suite after each: 433
  passing both times. After step 3, the only failure was R20, which D033
  turns from a refusal into a design.
- **All 15 rows of `native-replacement.tsv` agree with luria** on every
  compared field: 11 designs, 4 failures with their exact error text.
- **Mutation check, planner-run.** Each D033 choice fails at least one test
  when perturbed: inline limit 6→5 (2 tests) and 6→7 (3), deciding the path by
  `m` alone (3), forward anchor ignoring `m − w` (2), not splitting a
  replacement's tails (1), and always using the mismatch-aware basis (U2 and
  X2, after §9 and §9.1).
- **The contract's arithmetic was wrong once:** the planner's prompt to part 1
  said 435; the worker pointed out that its own formula gives 434.
- **§9's first U2 did not bind**, because `tm.rs` reports the matched basis for
  a site identical to the primer. The worker found this through the requested
  mutation proof and stopped rather than changing the test. §9.1 chose a
  replacement that leaves one isolated mismatch; the planner confirmed it binds.

What remains for pass 5 is 4e-4: expansion at the request layer, the
64-design warning, naming, and workflow wiring.
