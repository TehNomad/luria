# Contract 8m1 — cli: where the cursor lands after a pick (M8-21, M8-22)

Planner: Claude. Status: delegated to `w-8m1` 2026-10-27. Tier: light
(D051): tests assert behaviour, not wording. Owner's answers:
`docs/tasks/milestone-8m-polish.md` §1, M8-21 and M8-22.

## 1. What this contract builds

Three changes to where the cursor goes:

- A CDS picked from the `c` list moves the cursor to its start codon.
- `f` + `enter` on a minus-strand feature goes to its 5′ end.
- `g M1` with no CDS selected opens the CDS picker, then goes to residue 1
  of the CDS you pick.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `enter_cds`, `enter_feature`, `select_cds`, `follow` | `crates/cli/src/sequence_view/state.rs` | the pick and the jump |
| `goto_residue`, `go_to`, `prompt_key` | `state.rs` (8k2) | the residue rule: range, letter check, middle base |
| `open_list(ListKind::Cdss, "")` | `state.rs` (8k3) | the picker `g` opens |
| `CdsMap::residue_bases` | `crates/core/src/cds_map.rs:125` | codon positions in reading order, so `[0]` is the start codon's first base |
| `DisplayLocation::{open_before, open_after}` | `crates/core/src/display_location.rs:54` | which written end is open |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/state.rs`.
2. `crates/cli/tests/sequence_view_jumps.rs`: new.
3. The existing expectations §8 lists, and nothing else in them.

## 4. Rules

1. **CDS pick.** `enter` on a usable CDS row selects it, closes the list,
   and moves the cursor to `residue_bases(1)[0]` of the selected CDS (its
   start codon's first base in reading direction). Then the scroll follows.
   `select_cds` called directly (M8-11 at open, the tests, `enter_feature`)
   does not move the cursor. An unusable row behaves as today. [1]
2. **Feature 5′ end.** `enter_feature`'s target is the first base of the
   first interval on the plus strand, and the last base of the last interval
   (`end − 1`, D006) on the minus strand. Intervals are in D036's reference
   order. The refusal "the start is not stated" applies to `open_before` on
   the plus strand and to `open_after` on the minus strand. A usable CDS is
   selected exactly as today. Go-to by name (8k2's `goto_name`, which calls
   `enter_feature`) inherits this. [2, 3]
3. **Pending residue.** In go-to, a residue (`M1`) typed with no CDS
   selected opens the CDS picker with no filter, and the view remembers the
   residue. `enter` on a usable row selects the CDS and closes the list.
   The cursor then goes where 8k2's residue rule would put it in that CDS
   (the codon's middle base). If that rule refuses (number out of range, or
   the letter is wrong), the cursor goes to the start codon as in rule 1,
   and the status line shows the refusal 8k2 gives. `enter` on an unusable
   row behaves as today and keeps the residue. `esc` closes the list and
   forgets the residue, so a later `c` pick follows rule 1. [4, 5]
4. **No CDS at all.** If the record has no CDS, a residue in go-to opens
   nothing, moves nothing, and the status line says the record has no CDS.
   [6]
5. **The scroll.** Every path above moves the cursor after the list is
   closed, then calls `follow`, so the cursor's block is drawn. [1, 4]

## 5. Tests the worker writes (`tests/sequence_view_jumps.rs`)

`pbr` is `tests/fixtures/genbank/raw/J01749.1.gb`, opened at 80 × 24 with
no CDS selected (two usable CDSs, M8-11). Its picker rows are `[0, 1, 2]`:
`tet`, ROP (refused, `no_id`) and `bla`. `f62` is
`tests/fixtures/view/frames-62.gb`. Values are zero-based. The planner
derived them from the files' own locations and `/translation` qualifiers,
not from luria's output. Move the highlight with `Home` and `Down`, not by
relying on where the list opens.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `a_cds_pick_goes_to_its_start_codon` | `pbr`, `c`, `Home`, `Down` ×2, `enter`: `selected() == Some(2)`, cursor 4152 (`bla` `complement(3293..4153)`), its block in `visible()`. Then `c`, `Home`, `enter`: `selected() == Some(0)`, cursor 85 (`tet` `86..1276`), its block in `visible()` |
| 2 | `a_minus_strand_feature_goes_to_its_five_prime_end` | `pbr`, `f`, type `bla`, `Down`, `enter` (feature 38, the `bla` CDS): cursor 4152. `f62`, `f`, type `fM`, `enter`: cursor 7 (`complement(join(53..62,1..8))`); `f`, type `fP`, `enter`: cursor 55 (`join(56..62,1..11)`) |
| 3 | `an_open_five_prime_end_is_not_jumped_to` | A GenBank record written in the test: 30 bp, linear, with `misc_feature complement(<5..10)` named `a` and `misc_feature complement(5..>20)` named `b`. `f`, `a`, `enter`: cursor 9. `f`, `b`, `enter`: cursor unchanged, list still open, and a message |
| 4 | `a_residue_with_no_cds_opens_the_picker` | `pbr`, `g`, `M1`, `enter`: the list is open with `kind() == ListKind::Cdss`, cursor 0, nothing selected. `Home`, `Down` ×2, `enter`: `selected() == Some(2)`, cursor 4151 (the middle base of `bla`'s codon 1), list closed, its block in `visible()` |
| 5 | `a_refused_residue_still_picks` | `pbr`, `g`, `S1`, `enter`, `Home`, `enter`: `selected() == Some(0)`, cursor 85, and the message contains `M` (tet's residue 1 is M). Fresh `pbr`: `g`, `M1`, `enter`, `esc`: list closed, nothing selected, cursor 0. Then `c`, `Home`, `Down` ×2, `enter`: cursor 4152 (the residue was forgotten) |
| 6 | `a_residue_in_a_record_with_no_cds` | The FASTA `>w\nTTCAAAAAAAAAAGAA\n`: `g`, `M1`, `enter`: `list()` is `None`, cursor 0, and a message |

## 6. Checks

`scripts/check`, the six `sequence_view*` test targets, and
`python3 -I tests/fixtures/verify.py`.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | `enter_cds` does not move the cursor | 1 |
| G2 | `enter_feature` uses the first interval's first base on both strands | 2 |
| G3 | the open-end check uses `open_before` on both strands | 3 |
| G4 | with no CDS selected a residue says "no CDS is selected" as before | 4 |
| G5 | a refused pending residue leaves the cursor where it was | 5 |
| G6 | `esc` keeps the pending residue | 5 |

## 8. Do not

- No new dependency. No change to `crates/core`, `layout.rs`, `list.rs`,
  `goto.rs`, `search.rs`, `overview.rs`, `screen.rs`, or any fixture.
- No change to an existing test except these three expectations:
  - `tests/sequence_view_list.rs`, `enter_jumps_to_a_feature`: the `bla`
    CDS feature's cursor and block go from 3292 to 4152 (M8-21).
  - `tests/sequence_view_goto.rs`, `a_residue_needs_a_cds_and_a_number_in_range`:
    the first `M1` now opens the picker (assert that, then `esc`). The rest
    is unchanged.
  - `tests/sequence_view_overview.rs`, `picking_a_cds_keeps_the_cursor_on_screen`:
    after the pick the cursor is 85, and its block is in `visible()`. Fix
    that test's doc comment, which says "base 303" for cursor 301.
- `c_selects_only_a_usable_cds` keeps its value (p1 is `1..12`, so its
  start codon is base 0). Only its message wording may change.

## 9. Report

Every guard with its failure line and md5 lines before and after, the check
results, and every choice §4 left open.
