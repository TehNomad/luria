# Milestone 7j1: the screens in plain words

Status: **Ready for the worker, 2026-10-21.** Planner: Claude. Parent:
`docs/tasks/milestone-7-tui.md` §5.3b (owner answers M7-24 to M7-30).
Next: 7j2 (borders, the title badge, colour, the help pop-up's
drawing). This contract changes **text and keys only**; `view.rs`
changes only where §2.5 says.

## 1. What and why

The owner found the screens still cluttered and in programmer's
notation. This contract rewrites the overview, the member screen's
heading, primer lines and warnings, renames the technical screen to
**Details** on `d`, and adds a `?` help text holding the legend.

Screens may now use Unicode: `→` (U+2192), `–` (U+2013, en dash), `°`
(U+00B0), `▶` (U+25B6). Files written by luria are unchanged.

## 2. Changes

### 2.1 Coordinates on screen (M7-24), `crates/cli/src/review/text.rs`

```rust
/// `[start, end)` zero-based as 1-based inclusive: `"1210–1212"`, or
/// `"1210"` for one base (plan §5.3b, M7-24; D006 keeps files zero-based).
pub(crate) fn bases_at(start: u64, end: u64) -> String;
/// An insertion at zero-based point `p`: `"after p"`, or `"at the start"`
/// for 0.
pub(crate) fn insertion_place(point: u64) -> String;
```

Only these two convert; nothing else on screen writes a coordinate. A
unit test pins: `bases_at(1209, 1212) == "1210–1212"`, `bases_at(45,
46) == "46"`, `bases_at(0, 1) == "1"`, `insertion_place(1209) == "after
1209"`, `insertion_place(0) == "at the start"`.

### 2.2 `mutation_summary`

Replace its output (keep the `<N> bp` rule for base text over 30, as
`bases_text`):

| variant | text |
| --- | --- |
| `Substitution`, `Replacement` | `<original>→<written> at <bases_at>` |
| `Deletion` | `delete <original> at <bases_at>` |
| `Insertion` | `insert <written> <insertion_place>` |
| `Residue` | `<original_residue><residue><written_residue> (<original_codon>→<written_codon>)` |

It is used by the overview row, the member heading and the details
screen's `mutation:` line; all three change.

### 2.3 The member heading (7i3's `member_sequence`)

The block now starts:

```
<name>  <summary>  <template id> bases <bases_at(window)>
(blank)
forward ...
```

The two legend lines are gone (they move to the help, §2.6). The
window's `bases_at` takes the template positions 7i3 computes for the
window's start and end. `tests/fixtures/review/reference/member.py`
and its nine `member-*.txt` (now UTF-8) are already rewritten for this;
`crates/cli/tests/review_member.rs` must pass unchanged against them
(it reads the fixtures; if it reads them as ASCII, change only that).

### 2.4 The overview (M7-25, M7-26)

```
Template  <template id>  <topology>, <length> bp
Target    <target id>  <forward|reverse> strand, bases <bases_at per interval, joined ", ">
(blank)
Warnings                         (only with operation warnings)
  <message>                      (one per operation warning)
(blank)                          (only after Warnings)
Variants (<n>)
▶ <name>  <summary>  <outcome>   (the selected row)
  <name>  <summary>  <outcome>
```

- The ids are padded to the longer of the two ids, so the second
  column lines up; `Template  none` / `Target    none` when absent.
- Strand `+` is `forward`, `-` is `reverse`.
- Rows: the marker `▶ ` or two spaces, the name padded to the longest
  name, two spaces, the summary padded to the longest summary, two
  spaces, the outcome: `designed`, `no primer pair` for a design
  failure, `no design` for neither. Trailing spaces cut.

### 2.5 Details screen and keys (M7-30), `app.rs` and `view.rs`

- `Screen::Technical(i)` is renamed `Screen::Details(i)`; key `d` opens
  it from `Member(i)`; `t` does nothing.
- `App` gains `help: bool`, `false` from `new`, and `pub fn help(&self)
  -> bool`. `?` toggles it on every screen except `Export`; while it is
  `true`, `esc` and `backspace` only set it `false`, `q` still quits,
  and every other key does nothing. A screen change sets it `false`.
- `view.rs`: while `app.help()` is true the body shows `text::help()`
  in place of the screen's lines (7j2 turns this into a pop-up). Key
  lines: `d details` replaces `t technical`; every result screen's key
  line ends with `  ? help` before the scroll suffix; while help is
  shown the key line is `q quit  esc close`.

### 2.6 `text::help() -> Vec<String>`

Exactly:

```
The sequence
  forward    the forward primer, 5'→3', over the bases it covers
  template   the template's top strand, 5'→3'
  protein    the variant's protein, each letter over its codon's middle base
  bottom     the bottom strand, 3'→5'
  reverse    the reverse primer, written 3'→5' under its bases
  lower case edited bases, and primer bases that do not anneal
  *          a stop codon: the protein ends there
  [+N bp …]  a long insertion, shown in full with i
(blank)
Keys
  enter      open a variant
  j k        move or scroll
  d          details: everything the result records for the variant
  i          show or hide a long insertion
  p          provenance
  e          the problem, when there is one
  x          export
  esc        back
  q          quit
```

(`…` is U+2026.) Write it as a `const` array of lines.

### 2.7 The member screen below the block (M7-28, M7-29)

Replaces 7i4's primer lines and warnings in `member_screen`:

```
(blank)
          length       Tm   sequence 5'→3'
forward    <n> nt  <tm> °C   <sequence>
reverse    <n> nt  <tm> °C   <sequence>
annealing temperature <annealing_temperature with {}> °C
(blank)                                 (only with warnings)
Warnings
  <edit warning message>
  forward: <primer warning message>
  reverse: <primer warning message>
```

- Columns: the label padded to 10; `"<n> nt"` right-aligned in 6;
  `"<tm:.1> °C"` right-aligned in 9 (counting characters, not bytes);
  three spaces; the sequence. The header is 10 spaces, `length`
  right-aligned in 6, `Tm` right-aligned in 9, three spaces, `sequence
  5'→3'`.
- Messages are the document's own `message` fields (7c2's
  `edit_warning_message`, `primer_warning_message`), never rewritten.
- A design failure: `(blank)`, `No primer pair could be designed`,
  `  <failure.message>`.

## 3. Existing assertions that change (approved)

Only to the new text above; report each:

- `crates/cli/tests/review_text.rs`: overview tests (template/target
  lines, rows, the design-failure row), the details screen's `mutation:`
  line; any test pinning `mutation_summary`'s old form.
- `crates/cli/src/review/text.rs` unit tests that pin the overview row or
  `mutation_summary`.
- `crates/cli/src/review/app.rs` unit tests and
  `crates/cli/tests/review_view.rs` tests that press `t` or expect
  `Screen::Technical` or `t technical`: `d`, `Screen::Details`, `d
  details`; key lines gain `  ? help`.
- `crates/cli/tests/review_view.rs` 7i4 tests whose expected primer lines
  or warnings follow 7i4's form: rebuild their expectations by §2.7,
  values still read from the document.

If any other test fails, stop and report.

## 4. New tests

In `crates/cli/tests/review_text.rs` unless named:

1. `the_overview_reads_in_words`: the two-member set: the lines are
   exactly `Template  synlin  linear, 393 bp`, `Target    synA    forward
   strand, bases 43–345`, blank, `Variants (2)`, and two rows; values read
   from the document except these words.
2. `rows_line_up`: in the two-member set, the summary starts at the same
   character column in both rows.
3. `the_primer_columns_line_up` (`review_view.rs`, native-01): the
   header's and both primer lines' `nt` and `°C` end at the same
   character columns; the numbers are the document's (`{:.1}` Tm).
4. `warnings_are_descriptive` (`review_view.rs`, native-01): every
   primer warning's document `message` appears as `  forward: <message>`
   or `  reverse: <message>`; no line holds a warning code.
5. `help_opens_and_closes` (`app.rs` unit test): `?` sets help; `j` does
   nothing to `scroll` while help is open; `esc` closes help and stays on
   the screen; `q` quits with help open.
6. `the_help_holds_the_legend`: `text::help()` holds a line starting
   `  protein` and one starting `  lower case`.

### 4.1 Rules that must be proven

1. `bases_at` adding 1 to the start: the `bases_at` unit test and test 1
   fail.
2. `esc` closing help before going back: test 5 fails.
3. Messages rather than codes: test 4 fails.

## 5. Do not

- Change only `crates/cli/src/review/{text,app,view}.rs`, the tests in
  §3 and §4, and `crates/cli/tests/review_member.rs` only as §2.3 allows.
- No colour, border or pop-up drawing (7j2). No change to luria-core,
  `review.rs`, `run.rs`, `main.rs`, the proposal screen, export,
  fixtures, docs, Cargo files or `vendor/`.
- Do not stage or commit.

## 6. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
python3 -I tests/fixtures/review/reference/member.py --check
```

Report each result, every §3 assertion changed, the §4.1 proofs, and
anything that did not match.
