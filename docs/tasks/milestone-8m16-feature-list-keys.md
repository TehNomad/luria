# Contract 8m16 — cli: the feature list coloured, and filtered by key too (M8-39)

Planner: Claude. Status: written 2026-10-28, to run after 8m15. Tier:
light (D051): list drawing and filtering. Owner's answer:
`docs/tasks/milestone-8m-polish.md` §1, M8-39.

## 1. What this contract builds

- Each row of the `f` list is drawn in its feature's lane colour (8m7's
  `feature_colour`: the file's ApE colour, else the key's), as the row's
  foreground, with the colours on. With the colours off (`C`, `NO_COLOR`)
  no row has a colour. The CDS picker (`c`) does not change.
- The `f` list's filter matches a feature's key as well as its names, with
  the same matcher (D056): `CDS` lists the CDS features, `bind` lists every
  `misc_binding`. Go-to (`g`) does not change: it still matches names only.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `rows_for`, `feature_names`, `Drawn`, `drawn` | `crates/cli/src/sequence_view/list.rs:83`, `:52`, `:212`, `:220` | the filter and the row |
| `match_name`, `NamedItem`, `NameSource` | `crates/core/src/name_match.rs` | the one matcher (D056) |
| `feature_colour`, `key_colour` | `crates/cli/src/sequence_view/layout.rs:810`, `:785` | the row's colour |
| `draw_list` | `crates/cli/src/sequence_view/screen.rs` | apply the colour |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/list.rs`, `screen.rs`.
2. `crates/cli/tests/sequence_view_list_keys.rs`: new.
3. No existing test should change. If one fails, §9 says what to do.

## 4. Rules

1. **The colour.** `Drawn` gains `colour: Option<Color>`. For a feature row
   it is `feature_colour(feature, strand)` with the strand its drawn location
   was written on, and `key_colour(&feature.key)` for a feature that is not
   drawn. For a CDS picker row it is `None`. [1]
2. **Drawing it.** `draw_list` sets the row's text cells' `fg` to the colour
   when `state.colours()` is on and the colour is `Some`. `DIM` (refused)
   and `REVERSED` (highlight) are still added as today. [1, 2]
3. **The filter.** Only in `rows_for` for `ListKind::Features`: the names
   passed to `match_name` are `feature_names(record)` plus one entry per
   feature whose `name` is the feature's key (`whole_only: false`; the
   source can be `NameSource::DisplayName`, since `rows_for` reads only the
   item). The matcher answers items, so a feature named by both its name
   and its key is still one row, in file order. `feature_names` itself, and
   so go-to, do not change. [3, 4]

## 5. Hand values (the planner's, from `J01749.1.gb`)

`pbr` has 50 features in file order. Feature 0 is `source 1..4361`, feature
2 is `misc_binding 24..27` (named `misc_binding`: it has no name qualifier),
feature 9 is `gene 86..1276` (`/gene="tet"`), feature 10 is `CDS 86..1276`
(`tet`). No feature's name, `/gene`, `/locus_tag` or `/protein_id` starts a
word with `cds` or `gene`; the file has 3 `CDS` features (`tet`, `ROP
protein`, `bla`) and 2 `gene` features (`tet`, `bla`).

Key colours (8m7): `CDS` 214, `gene` 179, `misc_binding` 168; `source` has
none. The file has no ApE colours.

## 6. Tests (`tests/sequence_view_list_keys.rs`)

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `rows_have_their_lane_colour` | `pbr` at 80 × 24, `f`: the row for feature 10 (`tet CDS 86..1276 (+)`) has `fg` `Indexed(214)` on its first cell and its last text cell; feature 9's row 179; feature 2's row 168; feature 0's row (`source`) has `fg` `Reset` |
| 2 | `colours_off_rows_have_none` | as test 1 after `C`: no cell of the body has an `fg` other than `Reset`. And the `c` picker with colours on: no body cell has an `fg` |
| 3 | `the_filter_matches_the_key` | `f`, type `CDS`: exactly 3 rows, the 3 CDS features in file order (`tet`, `ROP protein`, `bla` first in their text). Clear it and type `gene`: exactly the 2 `gene` features. Type `tet` instead: the rows are the same as before this contract (every feature whose names match `tet`, each once) |
| 4 | `go_to_still_matches_names_only` | `pbr`, `g`, type `gene`, `enter`: no list opens and the cursor does not move (no feature is named `gene`); the status line says what go-to says for a name with no match today |

## 7. Checks

`scripts/check`, `python3 -I tests/fixtures/verify.py`, and every
`sequence_view*` target.

## 8. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | no key entries in `rows_for` | 3 |
| G2 | the key entries added in `feature_names` instead (so go-to sees them too) | 4 |
| G3 | the colour drawn whatever `state.colours()` says | 2 |
| G4 | `colour` always `None` | 1 |

## 9. Do not

- No new dependency. No change to `crates/core`, `layout.rs`, any fixture,
  or any existing test.
- No change to a row's text, the CDS picker's rows, or go-to.
- If an existing test fails, finish everything else, then stop and report it
  with its assertion, what it expected and what it got. Don't change it.
- Don't edit this contract.

## 10. Report

Every guard with its failure line and md5 lines before and after, the check
results, any existing test that failed, and any choice §4 left open.
