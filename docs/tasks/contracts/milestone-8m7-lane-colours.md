# Contract 8m7 — lanes coloured by key or by the file, the selected CDS bold (M8-31)

Planner: Claude. Status: **Implemented 2026-10-27** by worker `w-8m7` in
two rounds, reviewed and accepted. Tier: light (D051): the palette can
change at the hand test. Owner's answer: [docs/tasks/contracts/milestone-8m-polish.md](/tasks/contracts/milestone-8m-polish.md)
§1, M8-31. Findings from review:

* G1–G5 each failed the test §7 names. G5's tie case,
  `cube_index([115, 155, 195]) == 67`, stays in U1.
* Two gaps in this contract. §3 left out `state.rs`, which rule 6 needs.
  §8 also forbade the one change rule 6 forces: 8m5's
  `the_key_list_names_c` pins the `C` row's text. Round 2 updated that one
  expected string.
* Choices accepted: the first ApE qualifier of each kind decides, even when
  its value is refused; `overview_drawn` returns the styled line, and
  `overview_lines` is kept and delegates to it; the key colours stay
  literal indexes, and only file colours go through the cube.

## 1. What this contract builds

- Each feature lane is drawn in a colour: its name, its arrow and its `═`.
  The colour is the one the file gives the feature (ApE's qualifiers), or
  else its key's colour from rule 2.
- The overview strip's bars use the same colour.
- The lane of the selected CDS is bold while its residues are drawn.
- `C` (M8-29) turns these colours off and on with the residue colours.
  Bold stays, because it marks the selection and isn't a colour.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `ViewFeature`, `view_features`, `display_name` | `crates/core/src/view_record.rs:118`, `:250`, `:214` | add the file's colours, read as `display_name` reads qualifiers |
| `lanes`, `Lane`, the lane cells in `block_lines`, `Drawing` | `crates/cli/src/sequence_view/layout.rs` | the lane style |
| `LaneMode::Cds(i)`, `ViewRecord::cdss[i].feature` | `layout.rs:92`, `view_record.rs:140` | which feature is the selected CDS |
| `overview_lines`, `run_text`, `draw_overview` | `crates/cli/src/sequence_view/overview.rs:75`, `:222`, `screen.rs:123` | which strip cell belongs to which feature |

## 3. Files the worker may touch

1. `crates/core/src/view_record.rs`, and its `mod tests`.
2. `crates/cli/src/sequence_view/layout.rs`, `overview.rs`, `screen.rs`.
3. `crates/cli/tests/sequence_view_lane_colours.rs`: new.
4. `crates/cli/src/sequence_view/state.rs`: only `C`'s `?` row (rule 6).

## 4. Rules

1. **The file's colours (core).** `ViewFeature` gains
   `pub ape_colours: ApeColours`, where:

   ```rust
   #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
   pub struct ApeColours { pub forward: Option<[u8; 3]>, pub reverse: Option<[u8; 3]> }
   ```

   - `forward` comes from the first `/ApEinfo_fwdcolor`, and `reverse` from
     the first `/ApEinfo_revcolor`.
   - Each is read only when its value is `#` followed by six hex digits,
     either case.
   - Anything else gives `None`: a colour name such as `pink`, a bare
     qualifier, or a short or long value. [C1]
2. **The key colours** (cli, `fn key_colour(key: &str) -> Option<Color>`,
   using `Color::Indexed`):

   | Keys | Colour |
   | --- | --- |
   | `CDS` | 214 |
   | `gene` | 179 |
   | `mRNA`, `exon`, `intron`, `5'UTR`, `3'UTR` | 141 |
   | `promoter`, `regulatory`, `terminator`, `RBS`, `enhancer`, `polyA_signal` | 77 |
   | `rep_origin` | 75 |
   | `primer_bind`, `misc_binding`, `protein_bind` | 168 |
   | `repeat_region`, `mobile_element`, `LTR` | 109 |
   | `sig_peptide`, `mat_peptide`, `transit_peptide` | 180 |
   | anything else | `None` (the terminal's own colour) |
3. **A feature's colour.** The minus strand uses `reverse` and any other
   strand uses `forward`. That file colour becomes the nearest colour of
   the xterm 6 × 6 × 6 cube: each channel goes to the nearest of the
   levels 0, 95, 135, 175, 215, 255 (index `i` from 0 to 5; a tie goes to
   the lower level), and the colour is `Indexed(16 + 36·r + 6·g + b)`.
   Without a file colour, the key colour applies. Put this in
   `fn feature_colour(feature: &ViewFeature, strand: &Strand) -> Option<Color>`
   and `fn cube_index(rgb: [u8; 3]) -> u8`, both in `layout.rs`.
   `overview.rs` uses them too. [U1, 3]
4. **Lanes.** With `drawing.colours` on, every cell of a lane line is
   `fg(feature_colour)`, or unstyled when that is `None`. In
   `LaneMode::Cds(i)`, the lane of feature `cdss[i].feature` also gets
   `Modifier::BOLD`, whether colours are on or off. The `+N more` line is
   never coloured. [1, 2, 4]
5. **The strip.** With colours on, every cell of line 1 that a feature's
   run takes (its name, arrow and `═`) is `fg(feature_colour)`. The `─`
   cells and both end numbers keep no style, and line 2 is unchanged. The
   bars stay where they are: compute the placement once and use it for
   both the text and the colours, so the two cannot disagree. Return the
   colours however suits `overview.rs` best, but keep `overview_lines`'
   signature and its tests unchanged. [3, 4]
6. **`C`'s `?` row** now covers both: for example
   "C  colour the residues and the features, and back". `state.rs` may
   change only that string. Add `state.rs` to §3 for this one line.

## 5. Tests

`pbr` is `J01749.1.gb`. Its `tet` gene and `tet` CDS are both `86..1276`,
and the gene comes first in the file. 8m4's table gives the lanes of block
1 (bases 67–130) at width 78 in `LaneMode::Cds(0)`: lane 1 is the gene, lane
2 the CDS, and lanes 3 and 4 the two `misc_binding` sites. `ape` is a
GenBank record written in the test: 30 bp, linear, with three features.

- `misc_feature 1..10`, `/gene="a"`, with `/ApEinfo_fwdcolor="#ff0000"`
  and `/ApEinfo_revcolor="#00ff00"`;
- `misc_feature complement(11..20)`, `/gene="b"`, with the same two
  qualifiers;
- `misc_feature 21..30`, `/gene="c"`, with `/ApEinfo_fwdcolor="pink"`.

| # | Test | Asserts |
| --- | --- | --- |
| C1 | core unit | `ape`'s features: `a` and `b` have `forward [255,0,0]` and `reverse [0,255,0]`; `c` has `ApeColours::default()`. Also `"#FF00Aa"` is read and `"#ff00a"` is not |
| U1 | cli unit | `cube_index([255,0,0]) == 196`, `[0,255,0] → 46`, `[0x84,0xb0,0xdc] → 110` (132→135, 176→175, 220→215: 16 + 72 + 18 + 4), `[0,0,0] → 16`. `key_colour("CDS") == Some(Indexed(214))`, and `key_colour("old_sequence")` is `None` |
| 1 | `lanes_take_their_key_colours` | `block_lines` for `pbr`, `Cds(0)`, block 1, colours on. Every lane 1 cell is fg 179 and not bold. Lane 2 is fg 214 and bold. Lanes 3 and 4 are fg 168 |
| 2 | `bold_follows_the_selection_not_the_colours` | The same block with colours off: no lane cell has an fg, and lane 2 is still bold. `LaneMode::Off` at width 78, block 1 (bases 67–132), colours on: no lane cell is bold |
| 3 | `the_file_colour_wins` | `ape` at width 78, `Off`, block 0, colours on: `a`'s lane cells are fg 196, `b`'s are fg 46 (minus strand: `reverse`), and `c`'s have no fg (`misc_feature` has no key colour) |
| 4 | `the_strip_matches_the_lanes` | `pbr` view at 80 × 24, where the strip is on at open. Row 1, columns 4–23 (the `tet` gene's bar, 8l2 test 5): fg 179. Columns 54–73 (`mobile_element`): fg 109. A `─` cell has no fg. After `C`, no cell of row 1 has an fg |

## 6. Checks

`scripts/check`, `python3 -I tests/fixtures/verify.py`, `layout_fixtures`
and every `sequence_view*` target.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | the file colour is ignored | 3 |
| G2 | `reverse` is never used | 3 |
| G3 | bold only when colours are on | 2 |
| G4 | the strip colours `─` cells too | 4 |
| G5 | a tie in `cube_index` goes up | U1 is not enough: add a tie case of your own, show it fails, and keep it |

## 8. Do not

- No new dependency. No change to the GenBank parser, any fixture or table,
  or any existing test.
- No change to a lane's text, column or order, or to the strip's
  characters.

## 9. Report

Every guard with its failure line and md5 lines before and after, the check
results, and any choice §4 left open.
