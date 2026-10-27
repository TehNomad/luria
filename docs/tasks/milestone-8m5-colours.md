# Contract 8m5 — cli: residues coloured by Zappo group, `C` toggles (M8-29)

Planner: Claude. Status: delegated to `w-8m5` 2026-10-27. Tier: light
(D051): tests assert behaviour, and the hand test may change the colours
again. Owner's answer: `docs/tasks/milestone-8m-polish.md` §1, M8-29.

## 1. What this contract builds

In every residue line (the selected CDS's line and each frame's line), each
residue's codon gets a background colour by Zappo group across all three of
its bases. The letter stays over the codon's middle base, in black. `C`
turns the colours off and on. They are on at open unless `NO_COLOR` is set
and not empty. Drawing only: no block, row or column changes, so the layout
tables stay as they are.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `block_lines`, `residue_style`, `cds_residues`, `frame_cells`, `drawn_line` | `crates/cli/src/sequence_view/layout.rs` | where residue cells are made |
| `CdsMap::base_residue` → `CodonPlace { codon, place, residue }` | `crates/core/src/cds_map.rs:63` | which codon each base of a CDS belongs to, including codons split by the origin or an intron |
| `frame_residues(…, window_start, window_end)` | `crates/core/src/frames.rs:119` | gives only residues whose **middle** is in the window. Frames never wrap the origin, so a frame codon is `middle − 1 ..= middle + 1` |
| `drawn_lines`, `help_rows`, the `F` key | `crates/cli/src/sequence_view/state.rs:986`, `:783`, `:395` | the call site, the `?` list, and the toggle pattern |
| `run` | `crates/cli/src/sequence_view/terminal.rs:97` | where the state is made for a real terminal |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/layout.rs`.
2. `crates/cli/src/sequence_view/state.rs`, `terminal.rs`.
3. `crates/cli/tests/layout_fixtures.rs`: only the one `block_lines` call
   gains the new argument (`true`).
4. `crates/cli/tests/sequence_view_colours.rs`: new.

## 4. Rules

1. **The groups.** `fn zappo(symbol: char) -> Option<Color>` in `layout.rs`,
   using `Color::Indexed`:

   | Group | Letters | Colour |
   | --- | --- | --- |
   | aliphatic/hydrophobic | `I L V A M` | 217 |
   | aromatic | `F W Y` | 220 |
   | positive | `K R H` | 63 |
   | negative | `D E` | 196 |
   | hydrophilic | `S T N Q` | 46 |
   | conformationally special | `P G` | 201 |
   | cysteine | `C` | 226 |

   Every other symbol (`*`, `X`, anything else) gives `None`. [U1]
2. **The cells.** With colours on, every base of the block that belongs to
   a codon with a `Some` colour gets that background on the residue line.
   The middle base's cell is the letter with `fg(Color::Black)` and that
   background. The other two bases' cells are a space with the background.
   A codon split across two lines, by a block edge or the origin, is
   coloured on each line for the bases that line holds. The letter is drawn
   only where its middle base is. With colours off, or for a `None` symbol,
   every cell is what it is today (the stop stays bold red, with no
   background). [1, 2, 3]
   - CDS line: a base's codon comes from `base_residue`. `place.residue`
     `Some(n)` gives the letter `n` of the translation. `None` (the stop,
     or a codon the CDS does not have) gets no colour.
   - Frame line: call `frame_residues` with the window widened by one base
     on each side, clamped to `[0, length]`. Colour the bases of each codon
     that fall in `[start, end)`. Draw a letter only for a middle in
     `[start, end)`, so the letters are exactly today's letters.
3. **`block_lines`** gains a last parameter `colours: bool`. `drawn_lines`
   passes the state's flag. The unit tests in `layout.rs` pass `false`,
   except the new ones. `layout_fixtures` passes `true`: the tables pin text
   only, and they must still pass with colours on.
4. **The flag.** `ViewState` holds `colours: bool`, `true` from `new`.
   `pub fn colours(&self) -> bool` and `pub fn set_colours(&mut self, on: bool)`.
   `C` flips it, as `F` flips the lanes: not inside a list or a prompt,
   where `C` is text. [2]
5. **`NO_COLOR`.** `pub fn no_color(value: Option<&std::ffi::OsStr>) -> bool`
   in `terminal.rs` is true when the value is present and not empty
   (no-color.org). `run` calls `state.set_colours(!no_color(…))` with
   `std::env::var_os("NO_COLOR")`. Tests never set the variable. [U2]
6. **Keys.** The `?` list gets a row for `C` after `F`'s, for example
   "C  colour the residues by type (Zappo) and back". The key line is
   unchanged. [4]

## 5. Tests

`f62` is `tests/fixtures/view/frames-62.gb` (62 bp, `d = 2`, one 72-base
line at 80 × 24, so base `b` is at screen column `4 + b`). `fP` is
`join(56..62,1..11)`, `/translation="MRTDH"`. Zero-based codons: M 55–57,
R 58–60, T 61, 0, 1 (split by the origin), D 2–4, H 5–7, stop 8–10. `pbr` is
`J01749.1.gb` (`d = 4`, 66-base lines, base `b` of a block starting at `s`
at column `6 + b − s`). The planner derived every value from the files. Read
cells from a `TestBackend` draw, as `sequence_view_strands.rs`'s `yellow`
does. Find a line by its content or gutter label, not by a fixed row.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `a_cds_codon_is_coloured_across_its_bases` | `f62`, `c`, `Home`, `enter` (selects `fP`). On `fP`'s residue line: columns 59–61 bg 217 with `M` at 60; 62–64 bg 63 with `R` at 63; 65, 4 and 5 bg 46 with `T` at 4 and spaces at 5 and 65; 6–8 bg 196 (`D` at 7); 9–11 bg 63 (`H` at 10); 12–14 no background, `*` at 13 bold red; columns 15–58 no background. Every letter cell has `fg` black except `*` |
| 2 | `c_turns_the_colours_off_and_on` | Test 1's state: `C` → `colours()` false, no cell of that line has a background, and its letters are unchanged. `C` → true, and test 1's backgrounds are back. Then `f` and type `C`: the list filter holds `C`, and `colours()` is unchanged |
| 3 | `a_frame_codon_split_by_a_block_edge` | `pbr`, `t` (three frames), `o` (the strip off, so blocks 0 and 1 are drawn). Frame 3 reads from zero-based base 2, so its codon 65–67 (`AAC`, `N`) has base 65 at the end of block 0 and 66–67 at the start of block 1. On block 0's `+3` line, column 71 has bg 46 and holds a space. On block 1's `+3` line, columns 6–7 have bg 46, with `N` at 6 |
| 4 | `the_key_list_names_c` | `?`: some drawn row starts with `C` followed by spaces |
| U1 | unit, `layout.rs` | `zappo` for all twenty letters as in the table, and `None` for `*` and `X` |
| U2 | unit, `terminal.rs` | `no_color(None)` false, `Some("")` false, `Some("1")` true |

## 6. Checks

`scripts/check`, `python3 -I tests/fixtures/verify.py`, the eight
`sequence_view*` targets, and `layout_fixtures`.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | only the middle base gets the background | 1 |
| G2 | the frame window is not widened | 3 |
| G3 | `C` does nothing | 2 |
| G4 | `R` mapped to 46 instead of 63 | 1, U1 |
| G5 | the stop gets a background | 1 |

## 8. Do not

- No new dependency. No change to `crates/core`, `screen.rs`, `list.rs`,
  `goto.rs`, `search.rs`, `overview.rs`, any fixture or table, or any
  existing test except §3 item 3 and the `false` argument in `layout.rs`'s
  own unit tests.
- No change to a block's lines, rows or columns, or to any text on screen.
- No true-colour detection.

## 9. Report

Every guard with its failure line and md5 lines before and after, the check
results, and any choice §4 left open.
