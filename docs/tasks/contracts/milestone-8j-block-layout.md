# Contract 8j — cli: the block layout (§5.1)

Planner: Claude. Status: **Implemented 2026-10-26** by worker `w-8j` in two
rounds, reviewed and accepted. Tier: full (D051). Plan row: **8j** in
[docs/tasks/contracts/milestone-8-view.md](/tasks/contracts/milestone-8-view.md) §7. The view's state, cursor and status line
(8k1) draw with this. Findings from review:

* Round 1: tests 1 and 2 read all 645 block rows and all 845 cell rows (the
  cells table covers 24 blocks); G1–G7 behaved as §7 says, and the worker
  added two of its own (the stop style on every residue; a left-aligned
  frame label), both caught.
* The planner ran nine mutations. Two README rules survived, because no
  tabled block reaches them: numbers that touch with no blank column
  between them, and the inward clamp at a line's right end. Round 2 added
  unit tests 8 and 9 (below) on `number_cells` directly, with guards G8 and
  G9; both are now caught. Those are gaps in the 8a2 tables, not in the code.
* Merging abutting intervals of one location into one run (`merged`) is the
  worker's choice; no rule asks for it and no fixture reaches it. A stop
  drawn without `terminal_stop()` is unreachable for a usable CDS.
* The worker's report said a wrong arrow glyph would pass test 2; it would
  not, because the test's `run_text` builds `▸`/`◂` independently.

## 1. What this contract builds

`crates/cli/src/sequence_view/layout.rs`: pure functions that split a record
into blocks for a width and a translation mode, and turn one block into
ratatui lines. No terminal, no state, no keys. It is tested against the 8a2
column tables: `tests/fixtures/view/layout-blocks.tsv` and
`layout-cells.tsv`. `tests/fixtures/view/README.md`, section "Block layout",
states every rule those tables pin. That section is this contract's rule
text; read it in full.

Rules that apply, already settled:
- Plan §5.1 (`docs/tasks/contracts/milestone-8-view.md:353-384`).
- D050 M8-19 and D055 (`scripts/decision D055`): a shortened line, not a
  shifted grid; residues over middle bases; numbers for the selected CDS only;
  no toggles.
- D053: the display name; only `FeatureShape::Drawn` features have lanes.
- D052: frames do not wrap; a frame's unreadable codon is `?`.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `ViewRecord`, `ViewFeature`, `ViewCds`, `CdsStatus`, `FeatureShape` | `crates/core/src/view_record.rs:110-157` | the input |
| `translate(sequence, cds)` | `crates/core/src/translation.rs:130` | the selected CDS's residues |
| `CdsMap::new`, `base_residue(position)` | `crates/core/src/cds_map.rs:93`, `:151` | which codon a base is in, its place (2 = middle), its residue number; and whether two bases share a codon (a block edge) |
| `frame_residues(sequence, strand, frame, start, end)` | `crates/core/src/frames.rs:119` | the residues whose middle base lies in a block: call it with the block's window |
| `review::view` pattern | `crates/cli/src/review/view.rs:408` (`styled_lines`), `:563` (`Paragraph`) | pure functions return `Line<'static>`, and only a `draw` renders them |
| test helpers | `crates/cli/tests/review_view.rs:101-124` (`rows`, `styles`) | the `TestBackend` idiom: draw, then read `buffer.content` row by row |

Do not redo codon or frame arithmetic: every residue position comes from
`CdsMap` or `frame_residues`.

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view.rs`: new, `pub mod layout;` only.
2. `crates/cli/src/sequence_view/layout.rs`: new, with its `mod tests`.
3. `crates/cli/src/lib.rs`: `pub mod sequence_view;`, in alphabetical place.
4. `crates/cli/tests/layout_fixtures.rs`: new.

## 4. The public surface, exactly

```rust
pub enum LaneMode { Off, Cds(usize), ThreeFrames, SixFrames } // Cds: index into record.cdss

pub enum LineRole { Lane(u8), More, Residues, Numbers, Frame(Strand, ReadingFrame), Top, Bottom }

pub struct Layout { pub gutter: u16, pub line_width: u64, pub blocks: Vec<(u64, u64)> } // zero-based, half-open

pub enum LayoutProblem {
    TooNarrow { width: u16, needed: u16 },
    NotUsable(usize),           // LaneMode::Cds names a CDS that is not CdsStatus::Usable
    Domain(DomainError),        // translate or frame_residues refused
}

pub fn layout(record: &ViewRecord, width: u16, mode: &LaneMode) -> Result<Layout, LayoutProblem>;
pub fn block_lines(record: &ViewRecord, layout: &Layout, mode: &LaneMode, block: usize)
    -> Result<Vec<(LineRole, Line<'static>)>, LayoutProblem>;
```

Rules:

1. **Gutter, width, blocks, line order, residues, numbers, lanes**: exactly as
   the README's block-layout section states them, with `width` the number of
   columns the lines may use. `layout` refuses with `TooNarrow` when no line
   width of at least one step fits, where `needed = 2 * gutter + 2 + step`.
2. **Columns.** Every line's text starts at column 0. Positions not covered by
   an item are spaces. A line's text never goes past its last item, so no line
   has trailing spaces.
3. **Lane text.** Each run of a lane is `"{name} {arrow} "` followed by `═`,
   cut to the run's length, where `arrow` is `▸` for a plus-strand feature
   and `◂` for a minus-strand one. Every run starts with the name, including
   one clipped at a block edge. The `more` line's text is `+N more`.
4. **Labels.** A frame line's label is `+1` … `-3`, right-aligned in the
   gutter. The top strand's label is its first base's number. Residue and
   number lines carry no label.
5. **Styles.** One rule only: a `*` residue is bold red, as
   `review::view::letter_style` draws protein (`review/view.rs:262-295`).
   Cursor, underline and colours are 8k1's.
6. **`Cds(i)`** whose CDS is `CdsStatus::Unusable` is `NotUsable(i)` from both
   functions. An out-of-range `block` panics: the caller holds the layout.

## 5. Tests the worker writes

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `layout_fixtures::every_case_splits_into_the_tabled_blocks` | for each (record, width, mode) in `layout-blocks.tsv`: `line_width` and every block (table bases 1-based inclusive, `Layout` zero-based half-open), and no block beyond the table's |
| 2 | `layout_fixtures::every_tabled_item_is_where_the_table_says` | for each (record, width, mode, block) in `layout-cells.tsv`: render `block_lines` through `Paragraph` on a `TestBackend` of `width` × line count. Each row's role matches the table's `role` for that `line` (`lane1` … `lane4`, `more`, `residues`, `numbers`, `+1` … `-3`, `top`, `bottom`). For every row of kind `bases`, `complement`, `left_number`, `right_number`, `label`, `residue`, `residue_number` or `more`, the rendered text at `[column, column + cells)` equals `value`. For `feature`, it equals rule 3's run text for the name and strand in `value` (`"tet (+)"`). Every non-space rendered cell lies inside some tabled item. |
| 3 | `the_gutter_is_at_least_two_columns` | FASTA `>t\nATGAA\n` (5 bp) through `open_record`, width 40, `SixFrames`: gutter 2, line width 30, one block `(0, 5)`. Rows: `+3` label at columns 0-1 and `E` at column 6; `+2`, `*` at 5; `+1`, `M` at 4; top `1` at column 1, bases at 3, `5` at 9; `-1`, `F` at 6; `-2`, `S` at 5; `-3`, `H` at 4 |
| 4 | `a_narrow_width_is_refused` | pBR322 in `ThreeFrames`: width 39 → `TooNarrow { width: 39, needed: 40 }` (`2 × 4 + 2 + 30`); width 40 lays out with line width 30 |
| 5 | `a_lane_name_longer_than_its_run_is_cut` | pBR322, width 40, `Cds(0)` (`tet`), block 2 (bases 61-88): lane 1's run at columns 30-32 reads `tet`, lane 3's at 11-14 reads `misc` |
| 6 | `an_unusable_cds_is_refused` | pBR322's ROP (cdss 1) → `NotUsable(1)` from `layout` |
| 7 | `the_stop_is_bold_red` | synthetic `Cds(2)` (`pj`): the `*` cell has `fg == Red` and `BOLD`; an `M` cell has neither |
| 8 | `a_number_touching_a_placed_one_is_dropped` | round 2, unit on `number_cells`: residues 119 at column 10 and 120 at 13, `g = 4` → only `120`, at column 12 |
| 9 | `a_number_is_clamped_inside_the_bases` | round 2: residue 1000 at column 63, `g = 5`, `bases_end = 65` → `1000` at column 61 |

Test 1 and 2's records are read with `open_record` from the files the README
names. `framesamb` and any FASTA is read as `open_record` reads it (linear
assumed). `cds:ID` selects the `record.cdss` entry whose `id` is `ID`.

## 6. Checks

`scripts/check`, plus `cargo test -p luria-cli --test layout_fixtures` and
`python3 -I tests/fixtures/verify.py` (still `all 25 checks passed`).

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | No shortened line (blocks always `[kN, (k+1)N)`) | 1, 2 |
| G2 | Residue over the codon's first base in reading order | 2 |
| G3 | Numbers left-aligned instead of centred | 2 |
| G4 | Lanes in file order instead of longest first | 2 |
| G5 | Gutter without the minimum of 2 | 3 |
| G6 | Minus-strand CDS lines in the plus order (residues, numbers above) | 2 |
| G7 | `source` features given lanes | 2 |
| G8 | Numbers may touch (no blank column needed), round 2 | 8 |
| G9 | No inward clamp at the right end, round 2 | 9 |

## 8. Do not

- No new dependency. No change to `crates/core`, to `review`, or to any
  existing test or fixture.
- No cursor, scrolling, key or terminal code; no title or status line (8k1).
- No glyph or style beyond rules 3 and 5.

## 9. Report

Every guard experiment with its failure line, and any tabled item the rules
above do not answer.
