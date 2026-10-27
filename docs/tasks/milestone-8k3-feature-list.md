# Contract 8k3 — cli: the feature list and the CDS picker (§5.2, M8-5)

Planner: Claude. Status: delegated to `w-8k3` 2026-10-26; runs after 8h (D057 order).
Tier: light (D051): tests assert behaviour, not wording; the hand test (8m)
may change keys and sentences. Plan row: **8k3** in
`docs/tasks/milestone-8-view.md` §7. Go-to (8k2) opens this list filtered.

## 1. What this contract builds

Two lists over the view: `f` lists every feature, `c` lists every CDS.
Typing filters a list with the shared matcher, `enter` acts on the
highlighted row, `esc` closes it. No go-to (8k2), no search (8l1).

Rules that apply, already settled:
- Proposal `docs/tasks/proposal-sequence-view.md` §3 (`f`, `c`); plan §4.1
  (an undrawn feature is listed as "not drawn" with its reason) and §5.3
  (no jump to a feature whose start is not stated).
- D052 (M8-5, `scripts/decision D052`): the picker shows unusable CDSs,
  greyed, with the reason, not selectable.
- D053: a refusal is shown by its existing code; the display name is the
  feature's `name`. D056: filtering uses core's `match_name`.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `ViewState`, `Key`, `handle`, `select_cds`, `follow` | `crates/cli/src/sequence_view/state.rs` (8k1) | the list lives in the state; `handle` routes keys to it while it is open |
| `draw`, `draw_blocks` | `sequence_view/screen.rs` | the list is drawn in the body rows in place of the blocks |
| `key_of` | `sequence_view/terminal.rs` | map `Enter`, `Esc`, `Backspace` |
| `match_name`, `NamedItem`, `NameSource`, `cds_names`, `cds_span` | `crates/core/src/name_match.rs` (8i1) | filtering; the CDS list's names; spans |
| `rule_code`, `DomainError::code` | `crates/cli/src/result.rs:1625`, `crates/core/src/error.rs:971` | a refusal's code: `rule_code(e)` when it has one, else `e.code()` |
| `FeatureShape`, `DisplayLocation` (`open_before`, `open_after`), `CdsStatus` | `crates/core/src/view_record.rs:111`, `display_location.rs:54` | what a row shows and what `enter` may do |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view.rs`: add `pub mod list;`.
2. `crates/cli/src/sequence_view/list.rs`: new, with its `mod tests`.
3. `crates/cli/src/sequence_view/state.rs`: the three keys, the routing,
   `list()`, `open_list`.
4. `crates/cli/src/sequence_view/screen.rs`: drawing the open list.
5. `crates/cli/src/sequence_view/terminal.rs`: three key mappings.
6. `crates/cli/tests/sequence_view_list.rs`: new.

## 4. The public surface, exactly

```rust
// state.rs
pub enum Key { /* 8k1's */ Enter, Esc, Backspace }
impl ViewState {
    /// Open a list with this filter, the first row highlighted (8k2 calls this
    /// with the go-to text). Nothing below 80 × 24.
    pub fn open_list(&mut self, kind: ListKind, filter: &str);
    pub fn list(&self) -> Option<&List>;     // None when no list is open
}
// list.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListKind { Features, Cdss }
pub struct List { /* private */ }
impl List {
    pub fn kind(&self) -> ListKind;
    pub fn filter(&self) -> &str;
    /// Indices into `record.features` (Features) or `record.cdss` (Cdss),
    /// in file order.
    pub fn rows(&self) -> &[usize];
    pub fn at(&self) -> Option<usize>;       // index into `rows`; None if empty
}
/// The names a feature is filtered by: its `name` (`DisplayName`), and for a
/// CDS feature every entry of its `ViewCds::names`.
pub fn feature_names(record: &ViewRecord) -> Vec<NamedItem>;
```

Rules (the §5 tests in brackets):

1. **Open.** `f` opens `Features`, `c` opens `Cdss`, each with an empty
   filter. [1, 5]
2. **Rows.** An empty filter lists every item. Otherwise the rows are the
   items `match_name` answers (`One` or `Several`, each item once, file
   order) over `feature_names` (Features) or `cds_names` (Cdss); `Nothing`
   gives no rows. The highlight goes to the first row whenever the rows
   change. [2]
3. **Keys while open.** Every `Char` is filter text, `q` and `h`–`l`
   included; `Backspace` drops the last character; `Up`/`Down` move the
   highlight one row, staying at the ends; `PageUp`/`PageDown` move it by
   the body rows (`height - 4`), clamped; `Home`/`End` go to the first and
   last row; `Esc` closes and changes nothing else; `CtrlC` quits; `Enter`
   acts (rules 4, 5). [7, 8]
4. **`Enter` on a feature.** Not drawn: the cursor stays, the list stays
   open, `message` holds the refusal's code. Drawn with `open_before`: the
   same, with a message that the start is not stated (§5.3). Otherwise the
   cursor goes to the first base of the first interval (zero-based, D036
   order), the list closes and the scroll follows (8k1 rule 6). When the
   feature is a CDS feature, a usable CDS is also selected (`select_cds`);
   an unusable one is not, and `message` holds its code. [3, 4]
5. **`Enter` on a CDS.** Usable: `select_cds(Some(i))`, the list closes,
   the cursor stays. Unusable: nothing is selected, the list stays open,
   `message` holds its code. [5]
6. **A row's text.** Features: the name, the key, then the 1-based span in
   D036 order with GenBank's `<`/`>` on an open end and the strand (`(+)`,
   `(-)`), or `not drawn: {code}`. Cdss: the name and the span (or `not
   drawn`), then `unusable: {code}` for an unusable CDS. [6]
7. **Drawing.** While a list is open its rows fill the body rows in place
   of the blocks, scrolled so the highlight is on screen; the highlighted
   row is `REVERSED`; a row `Enter` would refuse is `DIM`. The status line
   shows the filter (or `message` when set); the key line shows the list's
   keys. The title is unchanged. [6, 8]
8. The normal key line adds `f` and `c`.

## 5. Tests the worker writes (`tests/sequence_view_list.rs`)

`pbr` is `tests/fixtures/genbank/raw/J01749.1.gb` (50 features; `tet` gene
9, ROP CDS feature 26 = cdss 1, unusable `no_id`; `bla` gene 37, CDS 38 =
cdss 2, sig_peptide 39 and mat_peptide 40 also named `bla`). `syn` is
`tests/fixtures/genbank/synthetic.gb` (27 features; feature `i` for `i` in
1..=24 is cdss `i - 1`; codes from `genbank/synthetic-unused.tsv`). 80 × 24.
Values were derived by the planner from the files and those tables.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `f_lists_every_feature` | `pbr`, `f`: kind `Features`, `rows() == 0..50`, `at() == Some(0)`, filter empty; `Esc` → `list()` is `None`, cursor 0 |
| 2 | `the_filter_is_the_shared_matcher` | `pbr`, `f`, type `bla` → rows `[37, 38, 39, 40]`; three `Backspace` → 50 rows; type `prot` → `[26]`; `Esc`, `f`, type `rotein` → no rows, `at() == None` |
| 3 | `enter_jumps_to_a_feature` | `pbr`: `f`, `rop`, `Enter` → cursor 1914, `selected() == None`, `message()` contains `no_id`, list closed. `f`, `bla`, `Down`, `Enter` → cursor 3292, `selected() == Some(2)`, mode `Cds(2)`, the cursor's block in `visible()`. `f`, `tet`, `Enter` → cursor 85, `selected()` still `Some(2)` |
| 4 | `a_feature_without_a_stated_start_does_not_move` | `syn`, 5 × `l`: `f`, `ord`, `Enter` → cursor 5, list open, `message()` contains `order`. `Esc`, `f`, `fuzzy`, `Enter` → cursor 5, list open, `message()` is `Some`. `Esc`, `f`, `filler`, `Enter` → cursor 48, list closed |
| 5 | `c_selects_only_a_usable_cds` | `syn`: `selected() == None`; `c` → `rows() == 0..24`; `Enter` → `selected() == Some(0)`, mode `Cds(0)`, list closed. `c`, `ps`, `Enter` → `selected()` still `Some(0)`, list open, `message()` contains `pseudo`. `pbr`: `c`, `Down`, `Enter` → `selected() == None`, `message()` contains `no_id` |
| 6 | `the_list_is_drawn_in_the_body` | `pbr`, `c`: row 1 contains `tet` and `86..1276`; row 2 contains `ROP protein` and `no_id`; row 3 contains `bla` and `3293..4153`. The cell under the `t` of `tet` is `REVERSED`; the cell under the `R` of `ROP` is `DIM` and not `REVERSED`. No row shows a base letter run (the blocks are not drawn) |
| 7 | `keys_go_to_the_open_list` | `pbr`, `f`: `j` → filter `j`, cursor 0; `q` → `Continue`, filter `jq`; `CtrlC` → `Quit` |
| 8 | `the_highlight_stays_on_screen` | `pbr`, `f`, `End` → `at() == Some(49)`, and the `REVERSED` row on screen contains `4331..4334`. `Home`, `PageDown` → `at() == Some(20)`, the `REVERSED` row contains `1891..1892` |

## 6. Checks

`scripts/check`, plus `cargo test -p luria-cli --test sequence_view_list`
and `--test sequence_view` (8k1's 15 still pass).

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | Filter by case-folded substring instead of `match_name` | 2 |
| G2 | `Enter` on a CDS feature never selects | 3 |
| G3 | `Enter` jumps to a feature with `open_before` | 4 |
| G4 | `Enter` on an unusable CDS closes the list | 5 |
| G5 | `q` quits while a list is open | 7 |
| G6 | The list is drawn from row 0 of `rows` whatever the highlight | 8 |
| G7 | Unusable rows not `DIM` | 6 |

## 8. Do not

- No new dependency; no change to `crates/core`, `layout.rs`, `review`, or
  any existing test or fixture.
- No go-to, no `?` help, no "no CDS" row in the picker (8m may add one).
- No file is ever written.

## 9. Report

Every guard experiment with its failure line and md5 lines, and every
choice §4 left open.
