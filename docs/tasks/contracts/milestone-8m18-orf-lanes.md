# Contract 8m18 — cli: ORF lanes, shown and hidden with `O` (M8-40)

Planner: Claude. Status: **Implemented 2026-10-28** by worker `w-8m18` in
two rounds, reviewed and accepted. Findings from review:

* `O` computes the ORFs on its first press and keeps them; `T` drops them
  and recomputes at once if the lanes are shown. Feature lanes and ORF
  lines share the column rule `g + first - start`.
* Tests 1–4 assert the full texts of §5's runs; the planner rechecked the
  cell counts (block 9: 66, 38, 48, 66 cells, heads on 641 and 622).
* `sequence_view_minus_bright.rs` has no `Drawing` literal, so §3 item 2
  touched five test files. `sequence_view_keys.rs`' comment grew to name
  the key, as §3 item 3 allows.
* Round 2 replaced a garbled doc comment on `ORF_COLOUR`.
* G1–G5 each failed the test §8 names (G3 failed tests 1–4).

Tier:
light (D051): the ORFs themselves are pinned by 8m17's table; this contract
only draws them, and the look can change at the hand test. Owner's answer:
[docs/tasks/contracts/milestone-8m-polish.md](/tasks/contracts/milestone-8m-polish.md) §1, M8-40.

## 1. What this contract builds

`O` shows and hides lanes for the open reading frames 8m17's `find_orfs`
finds (at least `ORF_MIN_RESIDUES` residues). They are hidden at open. Each
ORF overlapping a block gets one line below the feature lanes, drawn as a
feature lane is (a bar with a head at the 3′ end, 8m11) but labelled
`ORF {residues} aa` and in its own colour. `t`'s frames, `F`'s levels and the
overview strip do not change.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `find_orfs`, `Orf`, `ORF_MIN_RESIDUES` | `crates/core/src/orf.rs` (8m17) | the ORFs |
| `Drawing`, `LineRole`, `block_lines`, `lane_text`, `LANE_LIMIT`, the `+N more` line | `crates/cli/src/sequence_view/layout.rs:137`, `:147`, `:1040`, `:462`, `:102` | the new lines |
| `ViewState::new`, `lines`, `flip`, `help_rows`, `handle` | `crates/cli/src/sequence_view/state.rs` | the toggle and the kept ORFs |
| `describe` (layout unit tests), `role_for` | `layout.rs:1296`, `crates/cli/tests/layout_fixtures.rs:218` | the role names |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/layout.rs`, `state.rs`, `screen.rs` (only
   if a match needs the new roles).
2. Every `Drawing { … }` literal in existing tests gains the new field with
   the empty value (`orfs: &[]`), and nothing else: `layout.rs`'s unit tests,
   `crates/cli/tests/layout_fixtures.rs`, `sequence_view_levels.rs`,
   `sequence_view_lane_colours.rs`, `sequence_view_ruler.rs`,
   `sequence_view_arrowheads.rs`, `sequence_view_minus_bright.rs` and any
   other the compiler names.
3. `crates/cli/tests/sequence_view_keys.rs`: the `KEYS` array only, `"O"`
   inserted after `"o"` (18 keys), with its length and comment.
4. `crates/cli/tests/sequence_view_orf_lanes.rs`: new.

## 4. Rules

1. **What `block_lines` is given.** `Drawing` gains `pub orfs: &'a [Orf]`
   (so `Drawing<'a>`): the ORFs to draw, empty when they are hidden. [1–4]
2. **Which ORFs, in what order.** The ORFs with a base in `[start, end)`,
   each clipped to the block as a feature's runs are (an origin-crossing ORF
   may have two runs in one block on a short record). Order: longest span
   first, then `intervals[0].0`, then plus before minus. At most
   `LANE_LIMIT` (4) ORF lines; when more overlap the block, one more line
   reads `+N more ORFs`, as the feature `+N more` line does. [2, 3]
3. **Where.** Role `LineRole::Orf(n)`, numbered from 1, and
   `LineRole::MoreOrfs`; `describe` names them `orf{n}` and `more_orfs`.
   They come after the feature lanes and their `+N more` line and before
   every other line, in every mode. With `O` off a block has no ORF line. [2]
4. **The text.** A run's text is `lane_text(&format!("ORF {residues} aa"),
   &strand, length, head)`, with `head` true for the run holding the ORF's
   3′ end: the last base of the last interval on the plus strand, the first
   base of the first interval on the minus strand (8m11 rule 1). [1, 2]
5. **The colour.** Every ORF cell is `fg` `Indexed(151)` with the colours on
   and has no colour with them off. No ORF line is bold. [4]
6. **`O`.** Toggles the lanes. The first `O` computes
   `find_orfs(&record.sequence, self.circular, ORF_MIN_RESIDUES)` and keeps
   the result; later toggles reuse it. A `T` that flips the topology drops
   the kept ORFs, and they are computed again for the new topology when
   they are next shown (at once if they are shown when `T` is pressed).
   The status line says `ORFs shown: {count}` or `ORFs hidden`. The scroll
   follows the cursor, as `F` does. Inside a list or a prompt `O` is text.
   Nothing is computed at open. [1, 5]
7. **The key list.** One row after the `o` row:
   `O  show the open reading frames and hide them`. The key line does not
   change. [5]

## 5. Hand values (the planner's, from 8m17's table and the layout)

`pbr` (`J01749.1.gb`, circular) at 80 columns, lanes off (`LaneMode::Off`),
`F` set to none so no feature lane is drawn: blocks are 66 bases, `g = 5`,
block `k` is `[66k, 66k + 66)`. Its nine ORFs (8m17 §7), zero-based:
`+ [85, 1276)` 396 aa (tet), `+ [258, 642)` 127, `- [322, 565)` 80,
`- [438, 789)` 116, `- [622, 1081)` 152, `+ [1179, 1518)` 112,
`- [1514, 1883)` 122, `- [3292, 4153)` 286 (bla), `+ [3422, 3689)` 88.

- Block 0 `[0, 66)`: no ORF line.
- Block 1 `[66, 132)`: one ORF line, `orf1`, a run at columns 24–70 (47
  cells) reading `ORF 396 aa ` then `═` to the end, no head.
- Block 9 `[594, 660)`: four lines, in this order: tet (1191 bases),
  `- [622, 1081)` (459), `+ [258, 642)` (384), `- [438, 789)` (351). No
  `+N more ORFs` line.
- Block 19 `[1254, 1320)`: tet's run at columns 5–26 (22 cells) holds its 3′
  end (1275) and reads `ORF 396 aa `, ten `═`, then `▸`. The `+ [1179,
  1518)` run reads `ORF 112 aa ═…` with no head.
- Block 49 `[3234, 3300)`: bla's run at columns 63–70 holds its 3′ end
  (3292, column 63) and reads `◂ ORF 28`, cut to its 8 cells.

## 6. Tests (`tests/sequence_view_orf_lanes.rs`)

Use `block_lines` with `Drawing { orfs: &find_orfs(...) , … }` for the
layout values, and a `ViewState` for the keys.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `orf_lanes_hold_their_orfs` | §5's blocks 0, 1 and 19 |
| 2 | `orf_lanes_go_longest_first` | §5's block 9: the four roles `orf1`–`orf4`, in order, right after any feature lines, and their first cells' texts |
| 3 | `more_orfs_than_lanes` | `pbr` block 9 with a hand-built `orfs` slice of six `Orf`s that all cover `[594, 660)` (plus strand, lengths 600, 500, 400, 300, 200, 100 bases): four `orf` lines, then `+2 more ORFs` |
| 4 | `orf_lanes_have_their_own_colour` | block 49: bla's ORF cells `fg` `Indexed(151)` with colours on, `Reset` with them off; no ORF cell is `BOLD` |
| 5 | `o_shows_and_hides_the_orfs` | `pbr` view at 80 × 24: no ORF line at open; `O` → status says `ORFs shown: 9` and the drawn block holding the cursor has its ORF lines; `O` again → `ORFs hidden`, no ORF line. `f` then `O`: the list filter reads `O`. The key list has the `O` row right after `o` |
| 6 | `t_recomputes_for_the_new_topology` | a FASTA record written in the test, `>orfwrap` then `TAAATG` followed by 80 × `GCT` (246 bp, topology assumed, linear at open): `O` → `ORFs shown: 0` (the ATG runs off the end); `T` → `ORFs shown: 1`, the ORF `+ join(4..246,1..3)` of 81 aa, crossing the origin into the `TAA` |

## 7. Checks

`scripts/check`, `python3 -I tests/fixtures/verify.py`, `layout_fixtures`
and every `sequence_view*` target.

## 8. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | ORFs ordered by start instead of longest first | 2 |
| G2 | no `+N more ORFs` line (all ORFs drawn) | 3 |
| G3 | the head at the 5′ end | 1 or 4 |
| G4 | `T` keeps the old ORFs | 6 |
| G5 | ORF cells coloured with the colours off | 4 |

## 9. Do not

- No new dependency. No change to `crates/core`, any fixture or table, or
  any existing test except §3 items 2 and 3.
- No ORF computed at open or while drawing: only on `O` (and after `T`).
- If any other existing test fails, finish everything else, then stop and
  report it with its assertion, what it expected and what it got. Don't
  change it.
- Don't edit this contract.

## 10. Report

Every guard with its failure line and md5 lines before and after, every
change made under §3 items 2 and 3, the check results, and any choice §4
left open.
