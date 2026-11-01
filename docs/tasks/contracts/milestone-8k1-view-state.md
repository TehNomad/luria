# Contract 8k1 — cli: the view's state, cursor and status line (§5.1, §5.2)

Planner: Claude. Status: **Implemented 2026-10-26** by worker `w-8k1` in two
rounds, reviewed and accepted. Tier: light (D051): tests assert behaviour, not
wording; the hand test (8m) may change keys and sentences. Plan row: **8k1** in
`docs/tasks/contracts/milestone-8-view.md` §7. 8h opens this view from `luria FILE`.
Findings from review:

* Round 1: tests 1–12 and G1–G7 behaved as §7 says.
* A planner probe on `frames-62.gb` found one defect: the status span of a
  codon across the origin was min..max (`1..62`), not D036 order (`62..2`).
* The planner ran 17 mutations; four survived: `visible()` without the gaps,
  `w`/`b` that never wrap, `follow` putting the cursor's block on top instead
  of the least scroll, and no anchor after `t`. Round 2 fixed the span and
  added tests 13–15 with G8–G11; the first three are now caught. The `t`
  anchor is not observable with these fixtures: off, three and six frames
  give the same blocks, and a `Cds` layout's moved edges stay within a block
  of them, so the old top base is in the block of the same index.
* Choices left open and accepted: `j`/`k` stay at either end even when
  circular; `End` is the last base; the title names no CDS when the record
  has no usable one; the small-screen notice is one unframed line. A block
  taller than the body is not drawn at all (`draw_blocks` stops); the
  tallest pBR322 block at 80 × 24 is 13 lines of 20, so no fixture reaches it.
  A layout refusal on `select_cds` would leave no layout; it is unreachable
  for a usable CDS (the only `Domain` source is `translate`).
* The worker's report misdescribed some of its own code: `visible()` does
  not slide back at the end; there is no "codon straddles a block boundary"
  refusal; the key line does list `T`. Its "deviations" cited contract text
  and prompt guards that do not exist. The planner fixed one test comment
  (cursor 0 is the first base).

## 1. What this contract builds

The view as a screen: a pure state that moves a cursor over 8j's blocks,
scrolls them, cycles the translation lanes and picks the CDS on open, a
`draw` that renders title, blocks, status line and key line, and the
terminal session. No dispatch (8h), no go-to (8k2), no feature list or CDS
picker (8k3), no search (8l1), no overview (8l2), no `?` help.

Rules that apply, already settled:
- Plan §5.1 (cursor, status line, title bar) and §5.2; proposal
  `docs/tasks/proposals/proposal-sequence-view.md` §2–§3 for the screen's shape.
- D048 M8-11 (`scripts/decision D048`): exactly one usable CDS → selected;
  several → none, and the title says `no CDS selected`.
- D049 M8-3: a FASTA's topology is `linear (assumed)`; `T` switches it to
  circular for the session; a GenBank record's topology never changes.
- D055: no `r` or `space` toggles. D052: frames do not wrap (8j's layout
  already draws that).

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `layout`, `block_lines`, `LaneMode`, `Layout`, `LayoutProblem` | `crates/cli/src/sequence_view/layout.rs` (8j) | every block and every line drawn; base `p` of block `[s, e)` is at inner column `gutter + 1 + (p - s)` (`tests/fixtures/view/README.md`, "Block layout") |
| `review::app::App::handle`, `review::terminal::run` | `crates/cli/src/review/app.rs:281`, `review/terminal.rs:77` | the pattern: a pure `handle(Key) -> Action`, and a session that draws, reads a key, maps it, and restores the terminal on every exit |
| `review::view::frame_block` | `crates/cli/src/review/view.rs:72` (`pub(crate)`) | the rounded frame with the title in its top border |
| `review::view::MIN_WIDTH`, `MIN_HEIGHT` | `review/view.rs:27-29` | the 80 × 24 minimum; below it row 0 says the size and the minimum, and nothing else is drawn (write the sentence here; `too_small` names `luria review`) |
| `CdsMap::new`, `base_residue` | `crates/core/src/cds_map.rs:93`, `:151` | which codon the cursor is in, its place (2 = middle) and residue number; `None` outside the CDS |
| `translate(sequence, cds)` | `crates/core/src/translation.rs:130` | the selected CDS's residues, once per selection |
| `cds_span(record, cds)` | `crates/core/src/name_match.rs:194` (8i1) | the title's 1-based span, as `luria find` prints it |
| `review_view.rs` helpers | `crates/cli/tests/review_view.rs:101-124` | `TestBackend`: draw, then read `buffer` cells for symbol and modifier |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view.rs`: add `pub mod screen; pub mod state;
   pub mod terminal;` beside 8j's `pub mod layout;`.
2. `crates/cli/src/sequence_view/state.rs`: new, with its `mod tests`.
3. `crates/cli/src/sequence_view/screen.rs`: new.
4. `crates/cli/src/sequence_view/terminal.rs`: new; no unit test, as
   `review/terminal.rs`.
5. `crates/cli/tests/sequence_view.rs`: new.

## 4. The public surface, exactly

```rust
// state.rs
pub enum Key { Char(char), Left, Right, Up, Down, PageUp, PageDown, Home, End, CtrlC, Other }
pub enum Action { Continue, Quit }

pub struct ViewState { /* private fields */ }
impl ViewState {
    pub fn new(record: ViewRecord, width: u16, height: u16) -> ViewState;
    pub fn resize(&mut self, width: u16, height: u16);
    pub fn handle(&mut self, key: Key) -> Action;
    /// 8k3's picker: `false`, and nothing changes, for an index out of range or
    /// a CDS that is not `CdsStatus::Usable`.
    pub fn select_cds(&mut self, cds: Option<usize>) -> bool;
    pub fn record(&self) -> &ViewRecord;
    pub fn selected(&self) -> Option<usize>;    // index into record.cdss
    pub fn mode(&self) -> &LaneMode;
    pub fn circular(&self) -> bool;             // the topology the view moves on
    pub fn cursor(&self) -> u64;                // zero-based base
    pub fn layout(&self) -> Option<&Layout>;    // None below 80 × 24
    pub fn visible(&self) -> std::ops::Range<usize>; // blocks drawn; empty below 80 × 24
    pub fn message(&self) -> Option<&str>;      // cleared by the next key
}
// screen.rs
pub fn draw(frame: &mut Frame, state: &ViewState);
// terminal.rs
pub fn run(record: ViewRecord) -> std::io::Result<()>;
```

Rules (the §5 tests in brackets):

1. **Screen.** Rows `0..height-1` are `frame_block`, its title in the top
   border; inside it, the blocks, then the status line on the frame's last
   inner row. Row `height - 1` is the key line. `layout` gets the inner
   width, `width - 2`. Blocks are drawn from `visible().start`, one blank row
   between blocks; `visible()` is the longest run of whole blocks whose lines
   and gaps fit in `height - 4` rows (at least one). [6, 9]
2. **Open.** M8-11 picks `selected`; with a CDS selected the mode is
   `Cds(i)`, else `Off`. Cursor 0, first block on top. [1]
3. **Base moves.** `l`/`Right` +1, `h`/`Left` −1. Past either end: wraps when
   `circular()`, else stays. [2]
4. **Line moves.** `j`/`Down` and `k`/`Up` go to the same offset in the next or
   previous block, clamped to that block's last base; at the last or first
   block they stay. `PageDown`/`PageUp` move by `visible().len()` blocks, the
   same way, clamped to the last or first block, and put the cursor's block
   on top. `Home`/`End`: base 0, last base. [3, 6]
5. **Codon moves.** With a CDS selected, `w` moves to the nearest base 1 to 3
   bases right (wrapping when `circular()`) that is a codon's middle base
   (`place == 2`) of that CDS, and `b` the same to the left; if none, or no
   CDS is selected, the cursor stays. [4]
6. **Scrolling.** After every key, if the cursor's block is above
   `visible()`, it becomes the top block; if below, the top becomes the
   smallest block from which the cursor's block is visible. After `t`, `T`
   or `resize`, the top is first set to the block holding the old top
   block's first base. [6]
7. **`t`** cycles `Off → Cds(selected) → ThreeFrames → SixFrames → Off`,
   skipping `Cds` when no CDS is selected. The cursor stays. If `layout`
   refuses a mode with `Domain`, the mode is `Off` and `message` is the
   refusal's message. [5]
8. **`T`** on FASTA flips `circular()`; on GenBank nothing changes and
   `message` says the file states its own topology. [2]
9. **`select_cds(Some(i))`** of a usable CDS selects it, mode `Cds(i)`;
   `select_cds(None)` deselects, and a `Cds` mode becomes `Off`. [7]
10. **`q`** and `CtrlC` quit; every other key is `Continue` and changes
    nothing. Below 80 × 24 only `q` and `CtrlC` do anything. [8, 11]
11. **Title**: the record's name, `{length} bp`, the topology (`circular`,
    `linear`, `linear (assumed)`, or `circular (set with T)`), and the
    selected CDS's display name, strand and 1-based span (`cds_span`), or
    `no CDS selected` when the record has usable CDSs and none is selected.
    [9, 10]
12. **Cursor**: its cell on the top-strand row is `REVERSED`. With a CDS
    selected and the cursor in one of its codons, that codon's three
    top-strand cells are `UNDERLINED` (only those on screen). [9]
13. **Status line**: `base {n}` (1-based); with a CDS selected and the cursor
    in one of its codons, the residue letter and number (`M1`, or `stop`),
    the codon as read (minus strand: the complement, in reading order) and
    its 1-based reference-order span; then the drawn features other than
    `source` whose location holds the cursor; or `message` when set. [9, 12]
14. **Session**: `terminal::run` follows `review::terminal::run`: `ratatui`
    init and restore on every exit, key presses only, `Event::Resize` calls
    `resize`, `c` with the control modifier is `CtrlC`. It never writes a file.

## 5. Tests the worker writes (`tests/sequence_view.rs`)

`pbr` is `tests/fixtures/genbank/raw/J01749.1.gb` (cdss: `tet` 0, ROP 1
unusable, `bla` 2), `lin` is `tests/fixtures/genbank/synthetic-linear.gb`
(393 bp, one CDS `synA`, zero-based `[42, 345)`, plus), `w16` is a FASTA
`>w\nTTCAAAAAAAAAAGAA\n`. Records come from `open_record`; size 80 × 24
unless given. Values below were derived by the planner from the files.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `the_cds_on_open_follows_m8_11` | `lin`: `selected() == Some(0)`, mode `Cds(0)`. `pbr` and `w16`: `None`, `Off` |
| 2 | `base_moves_wrap_only_when_circular` | `pbr`: `h` at 0 → 4360, `l` → 0. `w16`: `h` at 0 stays 0; `End` → 15, `l` stays 15; `T` → `circular()`, `l` → 0; `T` → linear again. `pbr` `T`: `circular()` still true, `message()` is `Some` |
| 3 | `line_moves_keep_the_offset` | `pbr`, `Off`, line width 60: from 5, `j` → 65, `k` → 5, `k` → 5. From 4310, `j` → 4360 (last block holds 41 bases), `j` → 4360 |
| 4 | `codon_moves_step_codon_middles` | `lin` from 42: `w` → 43, `w` → 46, `b` → 43, `b` → 43. `pbr` with no CDS: `w` from 100 stays 100. `pbr` after `select_cds(Some(2))` (`bla`, minus), from 4151: `w` stays 4151, `b` → 4148 |
| 5 | `t_cycles_the_lanes` | `lin`: `Cds(0)`, `ThreeFrames`, `SixFrames`, `Off`, `Cds(0)`. `pbr`: `Off`, `ThreeFrames`, `SixFrames`, `Off`. The cursor is the same throughout |
| 6 | `the_cursor_block_stays_visible` | `pbr`: `End` → the last block (72) is in `visible()`; `Home` → `visible().start == 0`; from 5, `n = visible().len()`, `PageDown` → cursor `5 + 60n` and `visible().start == n`; at 120 × 40 after `resize`, the cursor's block is still visible |
| 7 | `select_cds_takes_only_a_usable_one` | `pbr`: `select_cds(Some(1))` → false, `selected() == None`; `Some(2)` → true, mode `Cds(2)`; `Some(3)` → false; `None` → true, mode `Off` |
| 8 | `q_and_ctrl_c_quit` | `q` and `CtrlC` → `Quit`; `x` → `Continue` with cursor and mode unchanged |
| 9 | `the_screen_shows_cursor_codon_and_status` | `lin`, cursor at 43 after 43 presses of `l`: exactly one `REVERSED` cell on screen, symbol `T`, at screen column 48 (`1 + 3 + 1 + 43`); exactly three `UNDERLINED` cells, columns 47–49 on that row. Title row contains `synlin`, `393 bp`, `linear`, `synA`. Status row contains `44`, `M1`, `ATG`, `43` and `45`. `pbr`'s title contains `no CDS selected` and not `(assumed)` |
| 10 | `a_fasta_title_says_assumed` | `w16`: title contains `linear (assumed)`; after `T`, `circular (set with T)` |
| 11 | `a_small_screen_draws_only_a_notice` | `pbr` at 79 × 24: row 0 contains `79x24` and `80x24`; every other row is blank; `layout()` is `None`; `l` leaves the cursor at 0 |
| 12 | `a_minus_strand_codon_is_read_on_its_strand` | `pbr`, `select_cds(Some(2))`, cursor 4151: status row contains `M1`, `ATG`, `4151` and `4153`; after `b`, `S2` and `AGT` |
| 13 | `a_codon_across_the_origin_spans_it` | round 2. `frames-62.gb`, `select_cds(Some(0))` (`fP`), cursor 0: status contains `T3`, `ACA`, `62..2`; `b` → 59, `w` → 0. `select_cds(Some(1))` (`fM`): `C3`, `TGT`, `62..2`; `w` → 3, `b` → 0 |
| 14 | `visible_counts_the_gaps` | round 2. `pbr`, `Off`: `visible() == 0..2` (blocks of 7, 6, 6 lines in 20 rows) |
| 15 | `the_scroll_moves_as_little_as_it_can` | round 2. `pbr`, `Off`: `j`, `j` → cursor 120, `visible() == 1..4` |

## 6. Checks

`scripts/check`, plus `cargo test -p luria-cli --test sequence_view` and
`python3 -I tests/fixtures/verify.py` (still `all 25 checks passed`).

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | Base moves wrap on a linear record too | 2 |
| G2 | `j` keeps the offset without clamping to a short last block | 3 |
| G3 | `w`/`b` step 3 bases whatever the codon place | 4 |
| G4 | `t` visits `Cds` with no CDS selected (as `Cds(0)`) | 5 |
| G5 | Open selects the first usable CDS when there are several | 1, 9 |
| G6 | The status codon is the top-strand letters in reference order | 12 |
| G7 | The top block never moves | 6 |
| G8 | The codon span as min..max, round 2 | 13 |
| G9 | `w`/`b` never wrap, round 2 | 13 |
| G10 | `visible()` without the gaps, round 2 | 14, 15 |
| G11 | `follow` puts the cursor's block on top, round 2 | 15 |

## 8. Do not

- No new dependency; no change to `crates/core`, `review`, `layout.rs`
  (8j), or any existing test or fixture.
- No dispatch in `main.rs` or `args.rs` (8h). No key beyond §4's.
- No file is ever written.

## 9. Report

Every guard experiment with its failure line, and any behaviour §4 leaves
open that you had to choose.
