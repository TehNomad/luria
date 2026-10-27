# Contract 8l1 — cli: the view's search prompt (§5.4)

Planner: Claude. Status: delegated to `w-8l1` 2026-10-26; runs after 8k2 (D057 order).
Tier: light (D051): tests assert behaviour, not wording; the hand test (8m)
may change keys and sentences. Plan row: **8l1** in
`docs/tasks/milestone-8-view.md` §7.

## 1. What this contract builds

`/` opens a search prompt with a `DNA`/`AA` mode badge. `enter` searches
with core's matchers, the hits are marked on the bases, the first hit at
or after the cursor becomes current, and `n`/`N` cycle through them. No
overview (8l2: it marks hits there later).

Rules that apply, already settled:
- Plan §5.4 (`docs/tasks/milestone-8-view.md:407-428`) and proposal §4
  (`docs/tasks/proposal-sequence-view.md:113-143`): the mode rule, `tab`,
  the two prompt details, AA in the selected CDS or six frames.
- D048 M8-12 (`scripts/decision D048`): at most 200 hits shown, with the
  real count said. D052: an all-ambiguity pattern hits every base once,
  strand `both`. D054: core returns hits left to right by base.
- D049 M8-3: after `T` a FASTA is searched as circular, as `luria find
  --circular` searches it (`crates/cli/src/find.rs:367-382`).

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `find_dna`, `DnaHit`, `HitStrand::symbol` | `crates/core/src/search.rs:188`, `:84`, `:72` | DNA mode |
| `find_protein`, `ProteinTarget`, `ProteinHit` | `crates/core/src/protein_search.rs:393`, `:98`, `:110` | AA mode: `Cds(selected)`, else `SixFrames` |
| `NUCLEOTIDE_LETTERS`, `nucleotide_letter` | `crates/cli/src/find.rs:77`, `:115` | the mode rule; make `nucleotide_letter` `pub(crate)`, change nothing else in `find.rs` |
| `normalize` on a circular copy | `find.rs:367-382` | the sequence searched after `T` |
| `ViewState`, `handle`, `follow`, the open list and go-to prompt | `crates/cli/src/sequence_view/state.rs` (8k1–8k3, 8k2) | the search prompt is routed the same way |
| `mark_bases` | `sequence_view/screen.rs` | hit marks go on the bases row the same way |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view.rs`: add `pub mod search;`.
2. `crates/cli/src/sequence_view/search.rs`: new, with its `mod tests`.
3. `crates/cli/src/sequence_view/state.rs`, `screen.rs`, `terminal.rs`.
4. `crates/cli/src/find.rs`: `nucleotide_letter` becomes `pub(crate)`.
5. `crates/cli/tests/sequence_view_search.rs`: new.

## 4. The public surface, exactly

```rust
// search.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchMode { Dna, Aa }
/// `Aa` as soon as one character is not a nucleotide letter (either case);
/// `Dna` otherwise, the empty text included.
pub fn auto_mode(text: &str) -> SearchMode;
pub struct SearchPrompt { /* text, mode, whether tab was pressed */ }
impl SearchPrompt { pub fn text(&self) -> &str; pub fn mode(&self) -> SearchMode; }
pub struct Hit { pub start: u64, pub length: u64, pub label: String }
pub struct Hits { /* the shown hits (≤ 200), the total, the current index */ }
impl Hits {
    pub fn shown(&self) -> &[Hit];
    pub fn total(&self) -> usize;
    pub fn current(&self) -> usize;           // index into `shown`
}
// state.rs
pub enum Key { /* … */ Tab }
impl ViewState {
    pub fn search(&self) -> Option<&SearchPrompt>;  // the open prompt
    pub fn hits(&self) -> Option<&Hits>;            // the hits marked now
}
```

Rules (the §5 tests in brackets):

1. **Open.** `/` opens the prompt: no text, `Dna`, tab not pressed.
   Nothing below 80 × 24; inside an open list or go-to prompt `/` is text.
   [1]
2. **Keys while open.** `Char` appends; `Backspace` drops the last
   character; until `tab` is pressed the mode is `auto_mode(text)` after
   each of these, so deleting the letter that forced `AA` returns to `DNA`.
   `Tab` flips the mode and from then on the mode no longer follows the
   text. `Esc` closes, the hits unchanged. `Enter` closes and searches
   (rule 3). `CtrlC` quits; `q` is text, except below 80 × 24, where it
   quits. The status line shows the badge (`DNA` or `AA`) and the text. [2, 8]
3. **Search.** Empty text: nothing changes. `Dna`: `find_dna`. `Aa`:
   `find_protein` with the selected CDS, or `SixFrames` with none. The
   sequence searched is the record's, or its circular copy when
   `circular()` is true and the record's topology was assumed. A refusal:
   `message` is core's message, the hits are cleared, the cursor stays.
   No hits: the hits are cleared, `message` says so, the cursor stays. [5,
   7, 9]
4. **Hits.** `shown` is the first 200 of core's hits, in core's order;
   `total` is how many core returned. The current hit is the first shown
   hit whose start is at or after the cursor, else the first; the cursor
   goes to its start and the scroll follows. `label` is the strand symbol
   for DNA, the frame (`+2`) in six frames, and the residue numbers
   (`1..6`) in a CDS. [3, 5, 6]
5. **`n`/`N`** (no prompt or list open): the next or previous shown hit,
   wrapping within `shown`; the cursor goes to its start. No hits: nothing.
   [3, 6, 10]
6. **`T`** clears the hits (they were found on the other topology). [9]
7. **Status line** with hits: `hit {current + 1} of {shown.len()}`, the
   label and the 1-based span; when `total > 200`, also `showing the first
   200 of {total} hits — refine your pattern`. [5, 6]
8. **Marks.** Every base of every shown hit on the bases rows drawn gets
   `bg = Color::Yellow`; a hit across the origin marks both of its runs.
   The cursor's `REVERSED` and the codon's `UNDERLINED` stay as 8k1 draws
   them. [4]
9. The normal key line adds `/` and `n N`.

## 5. Tests the worker writes (`tests/sequence_view_search.rs`)

`pbr` is `tests/fixtures/genbank/raw/J01749.1.gb` (4361 bp, circular, no
CDS selected; `bla` = cdss 2). `w16` is the FASTA `>w\nTTCAAAAAAAAAAGAA\n`.
80 × 24. "`/ X`" means `/`, type `X`, `Enter`. The hit values are rows of
`tests/fixtures/view/search-dna-hits.tsv` and `search-protein-hits.tsv`
(zero-based starts): `AATT` at 58, 251, 1319, 1333, 3233, 3539, 3794 and
4359 (wraps); `GAATTC` at 4358 (wraps, length 6); `bla` `M` 10 hits, the
first at 3349; six-frame `MKSNNA` one hit at 85; `N` 4361 hits.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `slash_opens_a_dna_prompt` | `pbr`, `/`: `search()` open, text `""`, mode `Dna`; `Esc` → `search()` is `None`, `hits()` is `None`, cursor 0 |
| 2 | `the_mode_follows_the_text_until_tab` | `pbr`, `/`, type `GA` → `Dna`; `E` → `Aa`, the status row (21) contains `AA`; `Backspace` → `Dna`; `Tab` → `Aa`; `C` → `Aa`; three `Backspace` → text `""`, still `Aa`. `Esc`, `/` → `Dna`. `/`, type `MKSNNA` → `Dna` (every letter is IUPAC) |
| 3 | `the_first_hit_at_or_after_the_cursor` | `pbr`: `/ AATT` → `total() == 8`, `current() == 0`, cursor 58; `n` → 251; `N` → 58; `N` → 4359 (`current() == 7`); `n` → 58. `n`, `n` → 1319; `/ AATT` → `current() == 2`, cursor 1319; `l`, `/ AATT` → cursor 1333. `End`, `/ AATT` → cursor 58 (none at or after 4360) |
| 4 | `a_hit_across_the_origin_is_marked` | `pbr`: `/ GAATTC` → cursor 4358; exactly three cells on screen have `bg == Yellow`, at columns 44, 45 and 46 of one row (gutter 4: `1 + 4 + 1 + (4358 − 4320)`; block 0 is not on screen) |
| 5 | `aa_searches_the_cds_or_six_frames` | `pbr` (none selected): `/`, `Tab`, type `MKSNNA`, `Enter` → `total() == 1`, cursor 85. `select_cds(Some(2))`, `Home`, `/`, `Tab`, type `M`, `Enter` → `total() == 10`, cursor 3349, status row contains `of 10` |
| 6 | `two_hundred_are_shown` | `pbr`: `/ N` → `total() == 4361`, `shown().len() == 200`, cursor 0, status row contains `200` and `4361`; `N` → `current() == 199`, cursor 199 |
| 7 | `no_hits_and_refusals_leave_the_cursor` | `pbr`: `/ AATT` (cursor 58), then `/ GAATTT` → cursor 58, `hits()` is `None`, `message()` is `Some`. `/ A1` (mode `Aa`; `1` is no residue) → cursor 58, `hits()` is `None`, `message()` is `Some`. `/ AATT`, then `/`, `Enter` → `hits()` still has `total() == 8` |
| 8 | `keys_go_to_the_search_prompt` | `pbr`: `/`, `q`, `n` → each `Continue`, text `qn`, cursor 0; `CtrlC` → `Quit`. `/`, `resize(79, 24)`, `q` → `Quit` |
| 9 | `t_searches_a_fasta_as_circular` | `w16`: `/ GAATTC` → `hits()` is `None`. `T`, `/ GAATTC` → `total() == 1`, cursor 13. `T` → `hits()` is `None` |
| 10 | `n_without_hits_does_nothing` | `pbr`: `n` → `Continue`, cursor 0 |

Plus unit tests in `search.rs` for `auto_mode`: `""`, `"gaattc"`, `"RATY"`,
`"MKSNNA"` → `Dna`; `"E"`, `"GAE"`, `"A1"`, `"*"` → `Aa`.

## 6. Checks

`scripts/check`, plus `cargo test -p luria-cli --test sequence_view_search`
and the other three `sequence_view*` targets.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | The mode never switches by itself | 2 |
| G2 | `Backspace` does not recompute the mode | 2 |
| G3 | The current hit is always the first | 3 |
| G4 | `n` stops at the last hit | 3 |
| G5 | `Aa` always searches six frames | 5 |
| G6 | No cap: every hit is shown | 6 |
| G7 | The search ignores `T` (always the record's sequence) | 9 |
| G8 | Only the first run of a hit across the origin is marked | 4 |

## 8. Do not

- No new dependency; no change to `crates/core`, `layout.rs`, `list.rs`,
  `goto.rs`, `review`, or any existing test or fixture; in `find.rs` only
  the one visibility change.
- No overview marks (8l2), no search history, no mismatch count.
- No file is ever written.

## 9. Report

Every guard experiment with its failure line and md5 lines, and every
choice §4 left open.
