# Contract 8m13 — cli: go-to and `w`/`b` land on a codon's first base (M8-35)

Planner: Claude. Status: **Implemented 2026-10-28** by worker `w-8m13` in
two rounds, reviewed and accepted. Findings from review:

* A gap in §3: after `fP`'s new steps (`b` → 61, `w` → 2) the cursor no
  longer ended on 0, so `fM` was selected inside its codon 4 and the
  status-line asserts failed. The worker reported it without changing the
  test. Round 2 added two `h` presses back to 0, as the planner allowed.
* Accepted: one extra `b` press in `codon_moves_step_codon_firsts`; the new
  tests select `bla` with `select_cds`, because a pick would jump to the
  start codon first.
* Wording left as is: in the `fM` part, `w` → 1 is still codon 3 (its first
  base), not "the codon below the join", and `b` → 60 is codon 4, not
  "back". The values are right.
* G1–G3 each failed the tests §8 names.

Tier: light
(D051): cursor landing only. Owner's answer: `docs/tasks/milestone-8m-polish.md`
§1, M8-35.

## 1. What this contract builds

A residue typed at go-to (`g M1`), and every `w` and `b`, put the cursor on
the **first base of the codon in reading direction**, where today they put
it on the middle base. On the minus strand that is the codon's highest base.
The residue letter stays drawn over the middle base (D055). Nothing else
moves.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `step_codon`, `CODON_SEARCH` | `crates/cli/src/sequence_view/state.rs:1189`, `:106` | `w` and `b` |
| `residue_middle`, its callers `goto_residue` and `enter_cds` | `state.rs:1828`, `:1726`, `:1515` | the residue's base |
| `CdsMap::base_residue` (`place` 1–3 in reading order), `residue_bases` (reading order) | `crates/core/src/cds_map.rs:67`, `:175` | which base is first |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/state.rs`: the rules below, and the doc
   comments that say "middle" for these moves.
2. Existing tests, **only** the expectations listed here, with their
   messages, comments and test names updated to say "first base":
   - `crates/cli/tests/sequence_view.rs`, `codon_moves_step_codon_middles`
     (rename `codon_moves_step_codon_firsts`): `synA` (starts at zero-based
     42): from 42, `w` → 45, `w` → 48, `b` → 45, `b` → 42, `b` → 42 (no
     first base left of the first codon). The no-CDS part is unchanged
     (stays at 100). `bla`: walk to **4152** (not 4151); `w` → 4152 (the CDS
     ends above it), `b` → 4149.
   - `sequence_view.rs`, `a_codon_across_the_origin_spans_it`: `fP` is
     `join(56..62,1..11)`, codons (55,56,57) (58,59,60) (61,0,1) (2,3,4)…
     From 0, `b` → **61** (codon 3's first base, across the origin), then
     `w` → **2**. `fM` is `complement(join(53..62,1..8))`, codons in reading
     order (7,6,5) (4,3,2) (1,0,61) (60,59,58)… From 0, `w` → **1**, then
     `b` → **60**. The status-line asserts stay as they are.
   - `crates/cli/tests/sequence_view_goto.rs`, `a_residue_goes_to_its_codon_middle`
     (rename `a_residue_goes_to_its_codon_first_base`): `A42` → 165, `W87`
     → 300, `M1` → 42; `bla` `M1` → 4152, `S2` → 4149.
   - `crates/cli/tests/sequence_view_jumps.rs:310–314`: `4151` → `4152`,
     residue 1 of `complement(3293..4153)` is on its codon's first base.
   - The `jumps.rs:353–357` assert (4152) keeps its value; it still passes.
     Leave it.
3. `crates/cli/tests/sequence_view_codon_first.rs`: new.

## 4. Rules

1. **The residue's base.** Rename `residue_middle` to `residue_first`. On
   success it returns `residue_bases(number)[0]`, the codon's first base in
   reading order. Its refusals do not change. Both callers use it. [1, 2]
2. **`w` and `b`.** `step_codon` moves to the nearest base 1 to
   `CODON_SEARCH` (3) bases away whose `place` is 1, in the same direction
   and with the same wrapping and edge rules as today. A codon is three
   bases, so the next first base is never more than 3 away. [3, 4]
3. **Unchanged.** The residue letter's column (D055), the cursor-codon
   highlight (8m8), the status line, and the start-codon jump on a pick
   (M8-21) do not change.

## 5. Hand values (the planner's, from the files)

`lin` is `tests/fixtures/genbank/synthetic-linear.gb`: `synA` starts at
zero-based 42 on the plus strand, so residue n's first base is 42 + 3(n − 1).
`pbr` is `J01749.1.gb`: `bla` is `complement(3293..4153)`, so residue 1 reads
4152, 4151, 4150 and residue n's first base is 4152 − 3(n − 1). `f62` is
`tests/fixtures/view/frames-62.gb`, codons as in §3.

## 6. Tests (`tests/sequence_view_codon_first.rs`)

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `go_to_lands_on_the_first_base` | `lin`, `synA` selected: `g A42` → 165. `pbr`, `bla` selected: `g M1` → 4152 and `g S2` → 4149 |
| 2 | `w_and_b_step_first_bases_on_the_plus_strand` | `lin`, `synA` selected, cursor 43 (a middle base): `w` → 45, `w` → 48, `b` → 45 |
| 3 | `w_and_b_step_first_bases_on_the_minus_strand` | `pbr`, `bla` selected, cursor 4150 (residue 1's third base): `b` → 4149, `b` → 4146, `w` → 4149, `w` → 4152 |
| 4 | `the_letter_stays_over_the_middle_base` | `pbr` at 80 × 24, `bla` selected with its CDS lane on (`LaneMode::Cds`), `g M1`: the cursor is 4152, and the CDS residue line holds `M` in the column of base 4151, not 4152 |

## 7. Checks

`scripts/check`, `python3 -I tests/fixtures/verify.py`, and every
`sequence_view*` target.

## 8. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | `residue_first` returns `codon[1]` | 1 |
| G2 | `step_codon` looks for `place == 2` | 2 and 3 |
| G3 | `step_codon` looks for `place == 3` | 2 and 3 |

## 9. Do not

- No new dependency. No change to `crates/core`, `layout.rs`, `screen.rs`,
  any fixture, or any existing test except as §3 item 2 lists.
- If any other existing test fails, finish everything else, then stop and
  report it with its assertion, what it expected and what it got. Don't
  change it.
- Don't edit this contract.

## 10. Report

Every guard with its failure line and md5 lines before and after, every
existing-test change made under §3 item 2, the check results, and any
choice §4 left open.
