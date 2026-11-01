---
type: Task Contract
title: "Contract 8r3 — cli: the view builds each block once, and reuses the CDS it holds"
description: "Three changes to crates/cli/src/sequence_view/."
status: deprecated
tags: [milestone-8]
---
# Contract 8r3 — cli: the view builds each block once, and reuses the CDS it holds

Planner: Claude. Status: **Implemented 2026-10-29** by worker `w-8r3` in
two rounds, reviewed and accepted. Round 2: the count takes the held CDS
too (`block_line_count_with`), since round 1's count still translated the
CDS per block. pBR322 in `--release`: 50 `j` 1.9 ms to 0.78 ms, 10 `PgDn`
0.38 ms to 0.11 ms with no CDS selected, and about 45 times faster with
one. Not pinned by a test, and accepted: the `translate` guard in the
count (no fixture CDS passes `usable_cds` and fails `translate`), and
the state handing its held CDS over (only speed depends on it). Tier: presentation (D051), but nothing on
screen may change. Plan row: none. The owner asked, on 2026-10-28, for
the speed findings of the milestone 8 code review to be fixed before
milestone 9. Runs after 8r2.

## 1. What this contract builds

Three changes to `crates/cli/src/sequence_view/`. **Nothing drawn may
change.**

1. **Counting a block's rows without drawing it.** `ViewState::line_count`
   (`state.rs:1131`) builds every styled line of a block only to count
   them. `visible`, `run_from` and `follow` call it for every block on
   screen, and the draw then builds the same lines again, so each key
   press draws each block about three times. A new pure function in
   `layout.rs` counts them.
2. **Reusing the selected CDS's translation.** In `LaneMode::Cds`,
   `block_lines` (`layout.rs:1146`, the `LaneMode::Cds` arm near `:1266`)
   translates the whole CDS and builds a `CdsMap` again for every block,
   although `ViewState` already holds both in `Chosen` (`state.rs:216`).
3. **One move for `page` and `half_page`** (`state.rs:1249`, `:1283`),
   which repeat the same twenty lines.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `block_lines`, `Drawing` | `layout.rs:1146`, `:141` | Items 1 and 2 |
| `lanes(record, start, end, level)` | `layout.rs` | What the lane rows come from; the count must use the same call |
| `drawn_lines`, `line_count` | `state.rs:1142`, `:1131` | Item 1 |
| `Chosen { map, letters, strand }` | `state.rs:216`, built in `choose` | Item 2 |
| `layout_fixtures.rs`, every `sequence_view_*.rs` test, the unit tests in `layout.rs` and `state.rs` | `crates/cli/tests/`, `src/sequence_view/` | Must pass unchanged |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/layout.rs` — a new `block_line_count`, a
   new `block_lines_with`, `block_lines` becoming a call to it, and new
   unit tests. `Drawing` does not change, so no existing test changes.
2. `crates/cli/src/sequence_view/state.rs` — `line_count`, `drawn_lines`,
   `Chosen`, `choose`, `page`, `half_page`, and a new private helper.

## 4. The changes, exactly

**Item 1.**

```rust
/// The number of lines `block_lines` returns for these arguments, without
/// building them: the same lanes, `+N more` line, ORF lines and mode lines.
pub fn block_line_count(
    record: &ViewRecord,
    layout: &Layout,
    mode: &LaneMode,
    block: usize,
    drawing: &Drawing,
) -> Result<usize, LayoutProblem>;
```

It must return `Ok(n)` exactly when `block_lines` returns `Ok` of `n`
lines, and `Err` exactly when `block_lines` does. `line_count` calls it
(an `Err` counts as 0, as today's `unwrap_or_default` gives).

**Item 2.** A new `pub fn block_lines_with(record, layout, mode, block,
drawing, cds: Option<(&CdsMap, &[char])>)` holds today's `block_lines`
body; `cds` is the selected CDS's map and its residue letters (no
terminal stop, as `residues_text` gives them). In the
`LaneMode::Cds(index)` arm it uses `cds` when it is `Some`, and
translates as today when it is `None`. `block_lines` keeps its signature
and calls `block_lines_with(.., None)`. `Chosen` keeps its
letters as a `Vec<char>` as well, or instead of the `String` if nothing
else needs the `String`. `drawn_lines` passes `Some` only when the mode
is `LaneMode::Cds(i)` and `Chosen` was built for CDS `i`; otherwise
`None`.

**Item 3.** A private `fn move_by_blocks(&mut self, step: usize, direction: i64) -> Option<usize>`
holds what `page` and `half_page` share: find the cursor's block, clamp
the target block at either end, and move the cursor with its offset
clamped to the target's last base. It returns the target block, or
`None` where today's code returns early. `page` then sets `top` to the
target, and `half_page` moves `top` by `step` and calls `follow`, each as
today.

## 5. Tests the worker writes

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `layout.rs::the_count_is_the_number_of_lines` | For pBR322 (`J01749.1.gb`) and `frames-62.gb` at widths 80 and 120; for every mode (`Off`, `ThreeFrames`, `SixFrames`, and `Cds(i)` for every usable CDS); every `FeatureLevel`; ORFs from `find_orfs` and none; and every block: `block_line_count` equals `block_lines(...).map(len)`, `Err` with `Err` |
| 2 | `layout.rs::a_held_cds_draws_what_a_fresh_one_draws` | For each usable CDS of pBR322 and `frames-62.gb` and every block in `Cds(i)` mode: `block_lines_with` with `Some(..)`, built from `translate` and `CdsMap::new`, equals `block_lines` |
| 3 | `state.rs` timing, `#[ignore]` | Not an assertion: print the time to open pBR322 and to handle 50 `j` and 10 `PgDn` key presses, in `--release`. Run it before your changes (put them aside, then restore) and after, and report both |

## 6. Checks

`scripts/check`, plus `cargo test -p luria-cli --test layout_fixtures`
and every `--test sequence_view_*` passing unchanged.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | In `block_line_count`, forget the `+N more` line | test 1 |
| G2 | In `block_lines_with`, use `cds`'s letters shifted by one residue | test 2 |
| G3 | In `move_by_blocks`, drop the clamp at the last block | an existing page or half-page test (name it) |

## 8. Do not

- Change anything drawn, any key, `layout()`, or the layout tables.
- Add a cache of drawn lines, a `RefCell`, a dependency, or threads.
- Touch `overview.rs`, `screen.rs`, core, or `luria find`.

## 9. Report

Test 3's timings before and after, and G1 to G3 with their failure lines.
