# Contract 8m6 — cli: `F` cycles all features, CDSs only, none (M8-30)

Planner: Claude. Status: delegated to `w-8m6` 2026-10-27. Tier: light
(D051). Owner's answer: `docs/tasks/milestone-8m-polish.md` §1, M8-30.

## 1. What this contract builds

`F` now cycles through three feature levels, opening on the first:

- all features (as today);
- CDSs only;
- none.

The lanes are chosen from the level's features, so with CDSs only a CDS
gets a lane even where genes or other features would have filled the four.
Under none there are no lane lines and no `+N more` line. The overview strip
is unchanged.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `lanes`, `LANE_LIMIT`, the lane and `+N more` lines in `block_lines` | `crates/cli/src/sequence_view/layout.rs:316`, `:766-790` | where the level filters |
| `block_lines(…, colours: bool)` | `layout.rs:754` | becomes `drawing: &Drawing` |
| `lanes: bool`, `lanes_shown`, `Key::Char('F')`, `drawn_lines`, `help_rows`, `message` | `crates/cli/src/sequence_view/state.rs:241`, `:533`, `:405`, `:1006`, `:783`, `:211` | the state, the key, today's lane filter, the `?` row and the status line |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/layout.rs`, `state.rs`.
2. `crates/cli/tests/layout_fixtures.rs`: only its `block_lines` call.
3. `crates/cli/tests/sequence_view_keys.rs`: only
   `f_hides_and_shows_the_feature_lanes`, as rule 5 says.
4. `crates/cli/tests/sequence_view_levels.rs`: new.

## 4. Rules

1. **Types**, in `layout.rs`:

   ```rust
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub enum FeatureLevel { All, Cdss, None }
   /// How the lines are drawn, as the view's keys set it.
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub struct Drawing { pub colours: bool, pub features: FeatureLevel }
   ```

   `block_lines`' last parameter becomes `drawing: &Drawing`, and
   `drawing.colours` does what `colours` did. Callers:
   - `layout.rs` unit tests: `colours: false, features: All`, except
     8m5's colour tests, which keep `true`.
   - `layout_fixtures`: `colours: true, features: All`.
   - `drawn_lines`: the state's own values.
2. **The level.** `lanes` takes the level:
   - `All` is today.
   - `Cdss` keeps only features whose key is `CDS`.
   - `None` gives no lanes, so `block_lines` makes no `Lane` line and no
     `More` line.

   The sort, `LANE_LIMIT` and `+N more` then apply to what is left. [1, 2]
3. **The state.** `lanes: bool` becomes `features: FeatureLevel`, `All` at
   open.
   - `pub fn features(&self) -> FeatureLevel`.
   - `lanes_shown()` stays, true unless the level is `None`.
   - `drawn_lines` no longer filters lane roles itself: `block_lines` does,
     so the rows `visible` and `follow` count still match the rows drawn.
   - `F` steps `All → Cdss → None → All` (still text inside a list or a
     prompt).
   - The status line names the new level, for example
     "features: CDSs only". [3]
4. **`?`.** `F`'s row says it cycles: for example
   "F  cycle the feature lanes: all, CDSs only, none". The key line is
   unchanged. [3]
5. **The existing `F` test.** `f_hides_and_shows_the_feature_lanes` now
   steps through all three levels:
   - one `F`: `features() == Cdss`, and block 1's `tet` lane (the CDS) is
     still drawn;
   - a second `F`: `None`, no lane drawn, and more blocks visible than at
     open;
   - a third `F`: `All`, and the lane is back.

   Keep its other assertions. Rename it `f_cycles_the_feature_levels`.

## 5. Tests (`tests/sequence_view_levels.rs`)

`pbr` is `J01749.1.gb`. `crowd` is a GenBank record written in the test:
30 bp, linear, sequence `ATGAAATAA` followed by 21 `C`s, with five
`misc_feature 1..20` named `m1` to `m5` (`/gene`, as `sequence_view_jumps.rs:68` names its features) and one `CDS 1..9` named
`c1` (`/gene`). The planner derived the values from those locations. A
lane's length is the bases it covers, so the five 20-base features come
before the 9-base CDS.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `cdss_only_keeps_the_cds_lanes` | `block_lines` on `pbr`, `layout(…, 78, &LaneMode::Cds(0))`, block 1 (record bases 67–130). With `All` there are four `Lane` lines: `tet` gene, `tet` CDS and the two `misc_binding` sites (8m4's table). With `Cdss` there is exactly one `Lane` line, whose text at column 24 starts `tet ▸`, and no `More` line. With `None` there is no `Lane` or `More` line |
| 2 | `a_cds_is_never_left_in_more` | `crowd` at width 78, `LaneMode::Off`, block 0. With `All`: four `Lane` lines, none containing `c1`, and a `More` line reading `+2 more`. With `Cdss`: one `Lane` line, containing `c1`, and no `More` line. With `None`: no `Lane` or `More` line |
| 3 | `f_steps_through_three_levels` | `pbr` view: `features()` is `All`. `F` gives `Cdss` and a status line containing `CDS`. `F` gives `None`, `F` gives `All`. `f` then `F`: the list filter holds `F`, and the level is unchanged. `?`: some row starts with `F` and contains `CDS` |

## 6. Checks

`scripts/check`, `python3 -I tests/fixtures/verify.py`, `layout_fixtures`
and every `sequence_view*` target.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | `Cdss` keeps every feature | 1, 2 |
| G2 | `Cdss` is applied after `LANE_LIMIT` (the four lanes are chosen from all features, and then filtered) | 2 |
| G3 | `F` toggles `All ↔ None` only | 3, and `f_cycles_the_feature_levels` |
| G4 | `None` keeps the `More` line | 2 |

## 8. Do not

- No new dependency. No change to `crates/core`, `screen.rs`, `overview.rs`,
  any fixture or table, or any other test.
- No change to what `All` draws.

## 9. Report

Every guard with its failure line and md5 lines before and after, the check
results, and any choice §4 left open.
