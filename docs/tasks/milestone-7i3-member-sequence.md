# Milestone 7i3: the text of the redesigned member screen's sequence block

Status: **Ready for the worker, 2026-10-21.** Planner: Claude. Parent:
`docs/tasks/milestone-7-tui.md` §5.3a (owner answers M7-16 to M7-21).
The specification is `tests/fixtures/review/reference/member.py`: read its
docstring and `screen` function first. Its outputs,
`tests/fixtures/review/member-*.txt` (nine files), are the expected
text, independent of `crates/`. Next: 7i4 (drawing it, the keys, the
technical screen, the trimmed overview).

## 1. What and why

Opening a member will show its sequence: the forward primer above the
top strand, the mutant protein between the strands, the bottom strand,
the reverse primer below. This contract builds those lines as plain
strings, `text::member_sequence`, tested against the fixtures. It draws
nothing on the terminal and changes no key.

## 2. The function

In `crates/cli/src/review/text.rs`:

```rust
/// The redesigned member screen's sequence block (plan §5.3a), as
/// `tests/fixtures/review/reference/member.py` lays it out.
pub fn member_sequence(
    document: &ResultDocument,
    index: usize,
    template: Option<&[char]>,
    frame: Option<&MemberFrame>,
    expanded: bool,
) -> Vec<String>;
```

`MemberFrame` is 7i2's `crate::review::MemberFrame`. `template` is what
`matching_template` accepted; `frame` is `App::frame(index)`.

### 2.1 Cases before the layout, in order

1. `index` out of range: `["no such member"]`.
2. No design: `["no design, so no sequence"]`.
3. `template` is `None`: the existing `CONTEXT_UNAVAILABLE` line alone.

### 2.2 The layout

Exactly as `member.py`'s `screen`, with these sources (all from the
document; nothing is computed but display columns, plan §3 rule 1):

- **The edit** `(s, e, written)`: from `member.mutation`, the same
  mapping 7i2's private `edit_span` in `crates/cli/src/review.rs` uses
  (make it `pub(crate)` and reuse it; do not change its body). The written bases are
  the variant's `written` (`written_codon` for `Residue`; empty for
  `Deletion`). The removed bases are `template[s..e]`.
- **Region width** `R`: `max(e - s, written.len())`, or for an insertion
  (`s == e`) longer than 30 with `expanded == false`, the length of the
  marker `[+N bp <first 10>...<last 10>]` (the bases lower case, three
  ASCII dots).
- **Columns**: `col(p) = p` for `p < s`; `col(p) = s + R + (p - e)` for
  `p >= e`.
- **Template row**: template bases outside `[s, e)` at `col(p)`; the
  region's `R` columns from `s` hold the removed bases (insertion: the
  written bases) in lower case, then `-` up to `R`; or the marker.
- **Bottom row**: each template-row character's complement by
  `luria_core::sequence::complement_letter` (lower case kept lower, `-`
  kept `-`), none under the marker. Do not write a complement table.
- **Protein row**: for each `FrameResidue` of `frame` (when it is
  `Some(Ok(_))`), the symbol at the display column of `middle`, an
  edited-construct position: `m < s` → `m`; `s <= m < s + written.len()`
  → `s + (m - s)`, or nothing when collapsed; else `s + R + (m - s -
  written.len())`.
- **Primer rows**: each part as `member.py`'s `place`: spanned context
  parts at `col(p)` for each `p` of the span; a `mutation` part's text
  from column `s`; an unspanned tail just left of the forward primer's
  first spanned column, or right of the reverse primer's last (`max
  col(span.1 - 1) + 1`), reversed for the reverse primer; upper case when
  the part anneals, lower case when not; all of a tail hidden when
  collapsed, with its note. A context part whose sequence length differs
  from its span: `*` over the span and a note, as `text::context` does
  now.
- **Window** `[lo, hi)`: 5 columns either side of the drawn primer
  columns, within `[0, total)`, `total = len - (e - s) + R`.
- **Blocks**: 60 columns; a block that would end inside a collapsed
  marker ends where the marker starts.
- **Heading**: `<name>  <summary>  on <template id> [<a>, <b>)`, with
  `a`, `b` the template positions of `lo` and `hi` as `member.py`'s
  `template_position` gives them, and `<summary>` the overview's
  `mutation_summary`, changed so every base text longer than 30 is written
  `<N> bp` (this also changes the overview row; that is intended).
- **Rows**: label padded to 10, trailing spaces cut; the two lines after
  the heading and the blank lines exactly as the fixtures show.
- **Notes**: after a blank line, `insertion: <N> bp, i to expand` when
  collapsed, then one line per hidden tail as the fixtures show.
- **A frame error** (`Some(Err(reason))`) or no frame (`None`): no
  protein row is drawn (the label `protein` alone), and the last note is
  `protein: <reason>` (`protein: not available` for `None`). No fixture
  covers this; test 10 below does.

## 3. Tests: `crates/cli/tests/review_member.rs` (new)

Build each request as `crates/cli/tests/review_context.rs` does (copy
the helpers; do not change that file). Templates from rows of
`tests/fixtures/q5-primer/` are read at test time from their `seq:`
column; CDS `[[0, 99]]` `+` `complete` standard `from-cds-start` for
the synthetic template (`id` `syn`, file `syn.fa`), the SID-1 CDS for
SID-1. Design: `inline`, bound 15, buffer `q5_reaction`. Run with 7i2's
`run_with_constructs`, `prepare` the request, take `matching_template`
and `member_frames`, and compare `member_sequence(..)` joined with `\n`
(each line then `\n`) byte for byte with the fixture.

| test | edit | fixture |
| --- | --- | --- |
| 1 `native_01` | SID-1 replacement `[1209, 1212)` GAT→AAC | `member-native-01.txt` |
| 2 `del_11` | SID-1 deletion `[1209, 1212)` GAT | `member-del-11.txt` |
| 3 `ins_14` | SID-1 insertion at 1209, `GAT` | `member-ins-14.txt` |
| 4 `del_02` | row `del-02` of `native-deletion.tsv` | `member-del-02.txt` |
| 5 `ins_04` | row `ins-04` of `native-insertion.tsv` | `member-ins-04.txt` |
| 6 `rep_02` | row `rep-02` of `native-replacement.tsv` | `member-rep-02.txt` |
| 7 `rep_06` | row `rep-06` of `native-replacement.tsv` | `member-rep-06.txt` |
| 8 `ins_13_collapsed` | row `ins-13`, `expanded = false` | `member-ins-13.txt` |
| 9 `ins_13_expanded` | row `ins-13`, `expanded = true` | `member-ins-13-expanded.txt` |

Edits from rows use the row's own columns (`deletion` `s:e:X`,
`region` `s:e:X` with `replacement`, `point` with `inserted`; an
insertion's `before`/`after` are the template bases at `point - 1` and
`point`), never pasted values.

10. `a_frame_error_says_so`: native-01 with `frame` `Some(&Err("x".into()))`:
    the `protein` row is the bare label in every block and the last line
    is `protein: x`.
11. `no_design_and_no_template`: §2.1 cases 2 and 3, on a design failure
    (the `review_context.rs` `a_design_failure_has_no_context` request)
    and on native-01 with `template` `None`.

### 3.1 Rules that must be proven

Disable each once, run, restore; report which tests fail.

1. The protein's column shift past the edit (`m - s - written.len()`
   replaced by `m - s`): tests 2 and 6 fail.
2. Collapsing (treat every insertion as expanded): test 8 fails.
3. The reverse primer reversed: tests 1 to 7 fail.
4. Tails hidden when collapsed: test 8 fails.

## 4. Do not

- Change only `crates/cli/src/review/text.rs`, the new
  `crates/cli/tests/review_member.rs`, and, only to make 7i2's edit-span
  helper `pub(crate)`, `crates/cli/src/review.rs`.
- No change to keys, `view.rs`, `app.rs`, `main.rs`, existing tests,
  luria-core, fixtures, docs, Cargo files or `vendor/`. If the overview's
  `mutation_summary` change breaks an existing test, stop and report.
- No translation, codon or complement table of your own.
- Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the proofs of §3.1, and the first differing line of
any fixture that does not match (do not change the fixture; stop and
report).
