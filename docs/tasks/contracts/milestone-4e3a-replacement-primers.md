# Milestone 4e-3a task contract: replacement primers and pairs in `primer.rs`

Status: **Implemented 2026-10-15** by the local worker; accepted on planner review (§9).
Date: 2026-10-15
Planner: Claude. Parent: `docs/tasks/contracts/milestone-4-indels.md`, pass 5. Follows
4e-2 (replacement results exist). The design layer is 4e-3b.

## 1. Goal

`primer.rs` accepts a replacement result: untailed primers on the inline
path, tailed primers and the pair invariant on the tail path. Nothing designs
a replacement yet; the design layer keeps refusing one until 4e-3b.

## 2. Requirements, inline (D033, approved 2026-10-15)

A replacement names a region `[s, e)` of width `w >= 1` replaced by `m >= 1`
bases. With `t = max(w, m)`:

- `t <= 6`, **inline**: one forward primer whose text comes from the edited
  artifact and spans the junction, exactly as a substitution's does. The
  reverse primer is pure template.
- `t > 6`, **tails**: the forward primer anneals to template `[e, e + f)` and
  the reverse to the reverse complement of `[s − r, s)`. The forward tail is
  the last `ceil(m/2)` replacement bases, the reverse tail the reverse
  complement of the first `floor(m/2)`.
- **Pair invariant**, both paths: reverse span end + `w` = forward span start
  on the tail path; on the inline path the single forward primer covers the
  region, as today.

From 4e-2: `MutationResult::Replacement(record)` with `record.region()`,
`record.replacement()`, and `target_coordinates()` giving the region's source
coordinates.

## 3. Required changes, all in `crates/core/src/primer.rs`

### 3.1 `construct_primer` accepts a replacement

Today it exempts deletions and insertions from the equal-length and "stray"
checks, and compares a deletion's text against the template. A replacement
needs its own treatment, because the **forward** primer's text comes from the
edited artifact while the **reverse** primer's comes from the template:

1. Skip the equal-length check and the "stray" containment check for a
   replacement, as for the other two.
2. Compare the primer's text against the **edited** artifact for a forward
   primer and the **template** for a reverse primer. For the forward primer
   the span is a template interval but the text is edited; its edited
   coordinates are `[span.start(), span.end() + m − w)`, because every
   template position at or past `e` shifts by `m − w`. Read the edited text
   over that range. Refuse a mismatch with `PrimerTextAtSpanMismatch` as now.
   - The text-width check must therefore compare the **text's** length with
     the span width **plus** `m − w` for a forward primer, and with the plain
     span width for a reverse primer.
3. The changed positions inside a forward primer are the region's coordinates
   that the span covers, mapped into primer indices, exactly as a
   substitution's are. For a reverse primer they are empty: a replacement's
   reverse primer carries no changed base on either path.
4. Refuse a **reverse** primer whose span overlaps `[s, e)`
   (`span.start() < e && span.end() > s`) with `InvalidPrimerCandidate`,
   reason containing `"replaced"`. A forward primer is allowed to cover the
   region: inline, that is its job.
5. `edited_span` is `None` for a reverse primer, and for a forward primer the
   interval `[span.start(), span.end() + m − w)` against the edited
   artifact's reference.

### 3.2 `construct_tailed_primer` accepts a replacement

It currently requires `result.is_insertion()`. Allow a replacement too, with
the same refusal message when the result is neither. Everything else is
unchanged: the tail carries no template, the annealing portion must be the
template at its span, and the parts are `Tail` then `Context`.

### 3.3 `assemble_pair` for a replacement

Add a branch beside the insertion one, used when `result.is_replacement()`
**and both primers' spans lie outside the region** (the tail path):

- skip H2's coverage check, as for a deletion;
- require `reverse.span().end() == s` and `forward.span().start() == e`;
- require that the reverse tail's reverse complement followed by the forward
  tail equals `record.replacement()`, with a reason containing
  `"rebuild the replacement"`;
- set `deleted_width = w`, so the existing `abuts()` means "the annealing
  portions sit at the two edges of the replaced region".

On the **inline** path a replacement pair goes through the ordinary
substitution branch: H2's coverage check applies, and `deleted_width` is 0.
Decide which branch by whether the forward primer's span starts at or after
`e`; add a short comment saying so.

### 3.4 Errors

No new `DomainError` variants. Do not edit `error.rs`.

## 4. Required tests, in `primer.rs`'s test module

Add a helper `replacement(start, end, expected, text) -> MutationResult`
using `apply_replacement` on `seq_a()`
(`"ATGGCTTGGAATCAGCAGCAGCAGCAGCAG"`, 30 nt), no scope, `result_name()`.
Values derived by plain string slicing (planner, 2026-10-15). For
`[9, 12)` `AAT` → `GG`, the edited artifact is
`ATGGCTTGGGGCAGCAGCAGCAGCAGCAG` (29 nt), so `m − w = −1`.

| # | Test | Input | Expect |
|---|---|---|---|
| V1 | an inline forward primer takes its text from the edited artifact | forward, span `[0, 20)`, text `edited[0..19]` = `ATGGCTTGGGGCAGCAGCA` (19 chars) | ok; `length() == 19`; `changed_positions()` is `[9, 10]`; `edited_span()` is `Some` with start 0 and end 19 |
| V2 | the text must match the edited artifact | forward, span `[0, 20)`, text `ATGGCTTGGGGCAGCAGCT` | `PrimerTextAtSpanMismatch` |
| V3 | an inline reverse primer is pure template | reverse, span `[0, 9)`, text `CCAAGCCAT` | ok; `changed_positions()` empty; `edited_span()` is `None` |
| V4 | a reverse primer may not cover a replaced base | reverse, span `[0, 10)`, text `rc(template[0..10])` = `TCCAAGCCAT` | `InvalidPrimerCandidate`, reason contains `replaced` |
| V5 | a tailed forward primer on the tail path | `[9, 12)` `AAT` → `GATTACAG` (8 bases, so `floor(8/2) = 4`: forward tail `ACAG`, reverse tail `rc("GATT")` = `AATC`); forward tailed, tail `ACAG`, annealing `CAGCAGCAGC` at `[12, 22)` | ok; `sequence() == "ACAGCAGCAGCAGC"` (14); `tail()` is `Some`, `primer_indices() == (0, 4)` |
| V6 | a tail pair rebuilds the replacement | V5's forward + reverse tailed with tail `AATC`, annealing `rc(template[0..9])` = `CCAAGCCAT` at `[0, 9)` | pair ok; `deleted_width() == 3`; `abuts()` |
| V7 | tails that do not rebuild it are refused | as V6 but the forward tail is `ACAC` | `InvalidPrimerCandidate`, reason contains `rebuild the replacement` |
| V8 | a tail pair must sit at the region's edges | V5's forward + a reverse tailed primer at `[0, 8)` | `InvalidPrimerCandidate` |

For V5–V7 the replacement is 8 bases, so `t = 8 > 6` and the tail path
applies, and `rc("AATC") + "ACAG" == "GATTACAG"` rebuilds it. (The planner
first wrote this split as 5/3; it is 4/4, corrected before dispatch.) Build V5's reverse primer with `construct_tailed_primer`.

## 5. Permitted changes to existing tests

None expected. If one breaks, stop and report it rather than changing it.

## 6. Do not

- Do not touch `mutation.rs`, `primer_design.rs`, `q5.rs`, `q5_design.rs`,
  any fixture or `docs/`. No new error variants.
- Do not change the substitution, deletion or insertion behaviour or
  messages. All 425 existing tests must pass.

## 7. Checks

The five in `AGENTS.md`, with `export TMPDIR=$PWD/target/tmp`. Expected: 433
tests (425 + 8), none failing.

## 8. Handoff

Changed files, exact check results, added test names, anything unclear. If
the contract is ambiguous or self-contradictory, stop and report rather than
choosing — that has happened three times this milestone and reporting it was
the right call each time.

## 9. Outcome, 2026-10-15

**The worker run was aborted by the harness** after 82 tool calls and 44
minutes, so there is **no worker handoff**. The planner reconstructed and
verified the state directly instead:

- **Scope.** Only `crates/core/src/primer.rs` changed after this contract was
  written; `mutation.rs` is byte-identical to the 4e-2 review copy. Against
  the planner's 4d-2 review copy of `primer.rs`, all 24 removed or changed
  lines are inside `construct_primer` and `construct_tailed_primer`, the two
  functions §3 names. No existing test line was removed or altered.
- **Checks**, planner-run: fmt, clippy `-D warnings` and `git diff --check`
  pass; **433 tests pass (425 + 8), 0 failures**. All eight of V1–V8 are
  present with this contract's values.
- **Mutation check.** Disabling the reverse-overlap rule, the edges rule or the
  rebuild rule each fails exactly the test written for it (V4, V8, V7).
  Reversing the sign of `m − w` fails V1 and V2.

**One addition beyond the contract, accepted.** `construct_primer` also
refuses a replacement's forward primer whose span covers only part of the
region, because that primer's text length would not follow from `m − w`.
D033's inline forward primer always spans `[s − 10, e + v)`, so the design
layer never builds one; the refusal guards an otherwise undefined case and
its comment says so. **It has no test.** 4e-3b adds one.
