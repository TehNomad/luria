# Contract 8m3 — cli: `F`, `T` and the key list (M8-23, M8-25, M8-26)

Planner: Claude. Status: **Implemented 2026-10-27** by worker `w-8m3` in one
round, reviewed and accepted. Tier: light (D051): tests assert behaviour,
not wording. Owner's answers: `docs/tasks/contracts/milestone-8m-polish.md` §1,
M8-23, M8-25 and M8-26. Findings from review: tests 1–3 and G1–G5
behaved as §7 says. Choices accepted: one `drawn_lines` filter shared by
`lines` and `line_count`; the key list has no highlight and nothing
`enter` acts on; the lanes flag survives lists, prompts and resizes; the
descriptions are the worker's wording, for the owner to judge.

## 1. What this contract builds

- `F` hides and shows the feature lanes.
- `?` opens a list of every key the view has.
- The key line keeps the most-used keys plus `? keys`.
- `T` appears in the key list only for a record whose topology is assumed.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `ViewState::lines`, `line_count`, `visible`, `follow` | `crates/cli/src/sequence_view/state.rs:710`, `:895` | the lines a block draws, and the rows it takes |
| `LineRole::Lane`, `LineRole::More` | `crates/cli/src/sequence_view/layout.rs:101-105` | the feature lanes' lines |
| `ViewState::keys`, `handle`, `cycle`, `flip` | `state.rs` | the key line, the key dispatch, `t` and `T` |
| `TopologySource::Assumed` | as `flip` uses it (`state.rs:1069`) | whether `T` is offered |
| `draw`, `draw_body`, the list's drawing | `crates/cli/src/sequence_view/screen.rs` | where the key list is drawn: in the body, as a list is |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/state.rs`, `screen.rs`.
2. `crates/cli/tests/sequence_view_keys.rs`: new.

## 4. The public surface and rules

```rust
impl ViewState {
    pub fn lanes_shown(&self) -> bool;   // whether the feature lanes are drawn
    pub fn help(&self) -> bool;          // whether the key list is open
}
```

1. **`F`.** In the normal view, `F` flips `lanes_shown`, which is true at
   open. While it is false, a block's lines leave out every
   `LineRole::Lane(_)` and `LineRole::More` line. Filter in one place that
   both `lines` and `line_count` use, so the rows `visible` and `follow`
   measure are the rows drawn. After the flip the scroll follows the
   cursor. `layout.rs` is unchanged. [1]
2. **`?`.** In the normal view, `?` opens the key list. The list takes the
   body as a list does: the blocks and the overview strip are not drawn,
   and the status line stays. It has one row per key, in this order. Each
   row starts with the key text shown here, then two spaces, then a short
   description of what the key does. `T` has a row only when the topology
   is assumed. [2, 3]

   `hjkl`, `PgUp PgDn`, `Home End`, `w b`, `t`, `F`, `T`, `f`, `c`, `g`,
   `/`, `n N`, `o`, `?`, `q`

   Write each description from what the key does in the code (`t` calls
   `cycle`). While the list is open, the key line reads `any key closes`.
   Any key closes the list and does nothing else; `q` closes it and does
   not quit. `ctrl-c` still quits, as it does everywhere. Inside a list or
   a prompt, `?` and `F` are text, as every printable key is there.
3. **The key line.** The normal key line is exactly
   `hjkl PgUp PgDn Home End  t frames  F lanes  f c g  / n N  ? keys  q quit`
   (72 characters). The list, go-to and search key lines are unchanged.
4. **`T`** behaves as it does today: it switches an assumed topology, and
   where the file states one it changes nothing and says so. Only where it
   is listed changes (rule 2). [2]

## 5. Tests the worker writes (`tests/sequence_view_keys.rs`)

`pbr` is `tests/fixtures/genbank/raw/J01749.1.gb` (GenBank, circular as the
file states). `w16` is the FASTA `>w\nTTCAAAAAAAAAAGAA\n` (topology
assumed). Both at 80 × 24. The body is screen rows 3–20 with the overview
strip, or rows 1–20 without it. The status line is row 21, and the key
line is row 23. A help row is one body row with its border cells removed
and its leading and trailing spaces trimmed.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `f_hides_and_shows_the_feature_lanes` | `pbr` at open: `lanes_shown()`, and some body row (3–20) contains `tet` (the `tet` gene starts at base 86, in block 1, which is drawn). Note `visible()`. `F`: `!lanes_shown()`, no body row contains `tet`, and `visible().len()` is greater than before. `F`: `lanes_shown()`, and a body row contains `tet` again |
| 2 | `t_is_listed_only_for_an_assumed_topology` | `pbr`, `?`: `help()`, and no help row starts with `T `. `w16`, `?`: exactly one help row starts with `T ` |
| 3 | `question_mark_lists_every_key` | `pbr`, `?`: the help rows that start with a key, in order, start with `hjkl`, `PgUp PgDn`, `Home End`, `w b`, `t`, `F`, `f`, `c`, `g`, `/`, `n N`, `o`, `?`, `q`, each followed by two spaces. Row 23 contains `any key closes`. `j`: `!help()` and the cursor is still 0. `?`, then `q`: `handle` returns `Action::Continue` and `!help()`. Then row 23 (trailing spaces trimmed) is exactly rule 3's line |

## 6. Checks

`scripts/check`, the eight `sequence_view*` test targets, and
`python3 -I tests/fixtures/verify.py`.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | `F` does nothing | 1 |
| G2 | the lanes are filtered in `lines` only, not in `line_count` | 1 |
| G3 | the help lists `T` whatever the topology | 2 |
| G4 | `q` with the help open quits | 3 |
| G5 | a key pressed with the help open also does its normal job | 3 |

## 8. Do not

- No new dependency. No change to `crates/core`, `layout.rs`, `list.rs`,
  `goto.rs`, `search.rs`, `overview.rs`, any existing test, or any fixture.
  The existing `the_key_line_fits_eighty_columns` must still pass as it is.
- No new binding other than `F` and `?`.

## 9. Report

Every guard with its failure line and md5 lines before and after, the check
results, and every choice §4 left open.
