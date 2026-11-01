# Contract 8m2 — cli: a hit is marked on its strand (M8-24)

Planner: Claude. Status: **Implemented 2026-10-27** by worker `w-8m2` in two
rounds, reviewed and accepted. Tier: light (D051): tests assert behaviour,
not wording. Owner's answer: `docs/tasks/contracts/milestone-8m-polish.md` §1, M8-24.
Findings from review:

* Round 1: tests 1–3 and G1–G4 behaved as §7 says. The worker stopped on
  a conflict the planner's §8 missed: `a_hit_across_the_origin_is_marked`
  searches `GAATTC`, a `both` hit, and its helper required every mark on
  one row. In round 2 that test expects columns 44–46 on two adjacent rows
  and 6–8 on rows 8–9, and the helper went, as nothing else used it.
  Under G1 the rewritten test also fails, so the palindrome rule is guarded
  twice.
* Choices accepted: a private `Rows { top, bottom }` keeps `mark_bases`
  under clippy's argument limit, and a block with no bottom row marks
  nothing there (`layout` always draws one).

## 1. What this contract builds

Today a search marks every hit's bases yellow on the top strand only. After
this contract, a hit found on the minus strand is marked on the bottom
strand, a palindrome found on both strands is marked on both, and a plus
hit stays on the top strand.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `HitStrand` (`Plus`, `Minus`, `Both`) | `crates/core/src/search.rs:64` | a DNA hit's strand, as `DnaHit::strand` gives it |
| `ProteinHit::strand` (`Strand`) | `crates/core/src/protein_search.rs:116` | a protein hit's strand: its CDS's or its frame's |
| `Hit`, `search_record` | `crates/cli/src/sequence_view/search.rs:143`, `:277`, `:293` | where the view's hits are built |
| `mark_bases` and its caller | `crates/cli/src/sequence_view/screen.rs:226-232`, `:301` | the yellow marks; the caller finds the `LineRole::Top` row |
| `LineRole::Bottom` | `crates/cli/src/sequence_view/layout.rs:115` | the bottom strand's row in a block |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/search.rs` (including its `mod tests`).
2. `crates/cli/src/sequence_view/screen.rs`.
3. `crates/cli/tests/sequence_view_strands.rs`: new.
4. The one existing expectation §8 names.

## 4. Rules

1. **The hit's strand.** `Hit` gains `pub strand: HitStrand`. A DNA hit
   takes `DnaHit::strand`. A protein hit takes `HitStrand::Plus` for
   `Strand::Plus` and `HitStrand::Minus` for `Strand::Minus`. The hit's
   `label` is unchanged. [3]
2. **Where it is marked.** In each drawn block, a base covered by a shown
   hit goes `bg = Yellow` on the `LineRole::Top` row when the hit is `Plus`
   or `Both`, and on the `LineRole::Bottom` row when it is `Minus` or
   `Both`. Find the bottom row the way the caller finds the top row: by its
   role in the block's lines, not by assuming it is next to the top. [1, 2,
   and `every_shown_hit_is_marked`]
3. **Unchanged.** The cursor's reversal and the codon underline stay on the
   top row exactly as 8k1 draws them. So do the hit's span, the status line,
   `n`/`N`, and the overview's dots (8l2). Nothing else on the bottom row
   changes style.

## 5. Tests the worker writes (`tests/sequence_view_strands.rs`)

`pbr` is `tests/fixtures/genbank/raw/J01749.1.gb` at 80 × 24. It opens with
the overview strip on, so block 0's top-strand row is screen row 8 and its
bottom-strand row is row 9. A base `p` of block 0 sits at column `6 + p`
(as 8l1's `every_shown_hit_is_marked` reads them). The planner derived these
values from the file's sequence with an independent script. Zero-based
bases `[10, 18)` are `TGACAGCT` on the top strand. That pattern occurs
nowhere else on either strand, and its reverse complement `AGCTGTCA` occurs
on the top strand nowhere.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `a_minus_hit_is_marked_on_the_bottom_strand` | `pbr`, `/`, type `AGCTGTCA`, `enter`: cursor 10. The cells with `bg == Yellow` are exactly row 9, columns 16–23 |
| 2 | `a_plus_hit_is_marked_on_the_top_strand` | `pbr`, `/`, type `TGACAGCT`, `enter`: cursor 10. The yellow cells are exactly row 8, columns 16–23 |
| 3 | `a_protein_hit_takes_its_cds_strand` | `pbr`, `c`, `Home`, `Down` ×2, `enter` (selects `bla`, minus strand), `/`, type `MSIQ`, `enter`: one hit shown, with `start == 4141`, `length == 12`, `strand == HitStrand::Minus` (`bla`'s residues 1–4, from its `/translation`). Fresh `pbr`: `c`, `Home`, `enter` (`tet`), `/`, type `MKSNNAL`, `enter`: one hit, `start == 85`, `length == 21`, `strand == HitStrand::Plus` |

Both patterns in test 3 contain a letter that is not a nucleotide letter,
so the prompt's automatic mode is AA. Assert that rather than pressing
`tab`.

## 6. Checks

`scripts/check`, the seven `sequence_view*` test targets, and
`python3 -I tests/fixtures/verify.py`.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | every hit marked on the top row only (today's rule) | 1 and `every_shown_hit_is_marked` |
| G2 | every hit marked on the bottom row only | 2 and `every_shown_hit_is_marked` |
| G3 | a `Both` hit marked on the top row only | `every_shown_hit_is_marked` |
| G4 | every protein hit given `HitStrand::Plus` | 3 |

## 8. Do not

- No new dependency. No change to `crates/core`, `layout.rs`, `list.rs`,
  `goto.rs`, `overview.rs`, `state.rs`, or any fixture.
- No change to an existing test except this one expectation: in
  `tests/sequence_view_search.rs`, `every_shown_hit_is_marked`. `RATY`'s
  hits are `both` in `search-dna-hits.tsv`, so its yellow cells are the ten
  it lists on rows 8 and 15 plus the same columns on rows 9 and 16: twenty
  cells. Update its doc comment to say why.

## 9. Report

Every guard with its failure line and md5 lines before and after, the check
results, and every choice §4 left open.
