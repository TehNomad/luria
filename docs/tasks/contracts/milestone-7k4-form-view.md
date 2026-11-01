# Milestone 7k4: the `luria new` form's text and drawing

Status: **Implemented 2026-10-22** by w7k4f (ran out of context) and w7k4g. Proofs 1-3 failed as required. Test 1 matched the contract's 28 lines with no change to either side. Untested: the style rules §4.2 does not name. Planner: Claude. Parent:
`docs/tasks/contracts/milestone-7k-new.md` §1 (the mock-up), M7-36, M7-39, and
7j2's frame. Builds on 7k3 (`new::form::NewForm` and its accessors).
Next: 7k5 (the `luria new` command and its loop).

## 1. What and why

The form exists as state (7k3). This contract writes its screen: plain
text lines, as every review screen is built (plan §5.1), then drawn in
7j2's rounded frame with colour and a cursor. No key or state changes.

## 2. Text, `crates/cli/src/new/text.rs` (`new.rs` gains `pub mod text;`)

```rust
/// The form as lines, the index of the focused field's first line, and the
/// cursor's column in that line when the focus is a text field.
pub fn form_lines(form: &NewForm) -> FormText;

pub struct FormText {
    pub lines: Vec<String>,
    pub focus_line: usize,
    pub cursor: Option<usize>,
}

/// A preview in words, 1-based (M7-24): uses `review::text::bases_at` and
/// `insertion_place`; no second conversion.
pub fn reading_words(reading: &Reading) -> String;
```

### 2.1 The lines, in order

Labels are padded to 11 characters. A row whose field is hidden by 7k3's
`is_visible` is left out. `◉` (U+25C9) marks the chosen option, `○`
(U+25CB) the other.

```
Template   ◉ file  ○ paste
  File     <path typed>
           <file name>, FASTA          (once a FASTA file is loaded)
           <file name>, GenBank <locus> (once a GenBank file is loaded)
  Paste    <first line of the paste>
           <each further line>
  Name     <name>
  Shape    ◉ linear  ○ circular
  CDS      ◉ whole sequence  ○ bases            (FASTA or paste)
  CDS      ◀ <id>  <gene or ->  <product or -> ▶ (GenBank: the selected usable entry)
  Bases    <first>–<last>                        (CDS = bases)
  Strand   ◉ forward  ○ reverse
(blank)
Mutations  <first line>
           <each further line>
           ✓ <n>  <reading_words>               (one per non-blank line, in order)
           ✗ <n>  <refusal's message()>
In words   <text>                                (with a backend)
In words   needs --backend                       (without one, M7-36)
(blank)
Primers    inline (confine to tail not supported yet)
Overlap    <digits> nt
Codon      ◉ default policy  ○ typed per mutation (D404N:AAC)
Save as    <save_as>
           template saved as <paste file>        (Source = paste)
(blank)
           [ Review ]
```

and, when `blocker()` is `Some`, a blank line and `✗ <message>` (with
`line <n>: ` before the message when it names a line). An empty text
field shows nothing after its label. The labels of the template's
sub-fields are indented by two spaces as shown. Every line has its
trailing spaces cut, so an empty mutation line is an empty string.

### 2.2 `reading_words`

| Reading | Words |
| --- | --- |
| `Residue` with no codon | `residue <n> <from>→<to>, codon by default policy` |
| `Residue` with a codon | `residue <n> <from>→<to>, codon <codon>` |
| `Substitution` | `base <bases_at> <from>→<to>` |
| `Replacement` | `bases <bases_at> <from>→<to>` |
| `Deletion` | `delete <bases_at>` (with `base`/`bases` before it as one or more) |
| `Insertion` | `insert <insert> <insertion_place>` |

A base text over 30 is written `<N> bp`, as `bases_text` does in review
text (make it `pub(crate)` and reuse it).

## 3. Drawing, `crates/cli/src/new/view.rs` (`new.rs` gains `pub mod view;`)

```rust
pub fn draw_form(frame: &mut Frame, form: &NewForm);
```

- Too small (under 80x24): the review's notice, reused.
- 7j2's rounded frame (`frame_block`), title ` luria new ` then the badge ` NEW DESIGN `
  (black on cyan, as `PROPOSAL`) then ` · nothing is written until you
  approve `. Make the review's `frame_block`, `too_small` and `key_line`
  `pub(crate)` and reuse them; add nothing to `review/`. The title `Line`
  is built in `new/view.rs`, because `title_line` writes ` luria review `
  (accepted 2026-10-22).
- The body scrolls so `focus_line` is visible.
- Style: labels dim; the focused field's label bold and reversed; `◉`
  green; `✓` lines green, `✗` lines red; `needs --backend` dim; `[ Review ]`
  bold, reversed when focused; the blocker line bold red.
- The cursor: `frame.set_cursor_position` at `cursor` on `focus_line` when
  it is `Some`.
- Key line: `tab next  shift-tab back  ←/→ choose  enter review  esc quit`.

## 4. Tests

### 4.1 `crates/cli/tests/new_text.rs` (new)

Build forms with keys and `loaded` as `crates/cli/tests/new_form.rs` does
(copy its helpers).

1. `the_sid1_form_reads_as_drawn`: the form `new_form.rs`'s `sid1`
   helper builds with `requests.py`'s `sid1-set` lines and overlap `15`
   (the helper names the template `sid-1` and presses `Enter` after every
   line, so `Mutations` ends with an empty line), before `Review`:
   `form_lines(..).lines` equals exactly:

   ```
   Template   ◉ file  ○ paste
     File     sid-1_cDNA.fa
              sid-1_cDNA.fa, FASTA
     Name     sid-1
     Shape    ◉ linear  ○ circular
     CDS      ◉ whole sequence  ○ bases
     Strand   ◉ forward  ○ reverse

   Mutations  D404N
              del 1210-1212

              ins GAT after 1209
              1210 G>A
              1210-1212 GAT>AAC

              ✓ 1  residue 404 D→N, codon by default policy
              ✓ 2  delete bases 1210–1212
              ✓ 4  insert GAT after 1209
              ✓ 5  base 1210 G→A
              ✓ 6  bases 1210–1212 GAT→AAC
   In words   needs --backend

   Primers    inline (confine to tail not supported yet)
   Overlap    15 nt
   Codon      ◉ default policy  ○ typed per mutation (D404N:AAC)
   Save as    sid-1.request.json

              [ Review ]
   ```

   The layout is the contract's; if 7k3's state makes a line differ,
   stop and report rather than adjust either.
2. `a_genbank_cds_reads_as_its_entry`: after loading
   `genbank/synthetic-linear.gb`, the `CDS` line is `  CDS      ◀ synA  `
   + the catalogue entry's gene and product (`-` when absent) + ` ▶`, and
   no `Name`, `Shape` or `Strand` line appears.
3. `a_refusal_is_shown_and_named`: lines `D404N`, `D404*`, then
   `Review`: a `✗ 2  ` preview line with `Refusal::Stop.message()`, and the
   last line is `✗ line 2: ` + the blocker's message.
4. `reading_words_are_one_based`: one test per §2.2 row, from
   `parse_line` of `D404N`, `D404N:AAC`, `1210 G>A`, `1210-1212 GAT>TT`,
   `del 46`, `del 1210-1212`, `ins GAT after 1209`, `ins GAT after 0`.
5. `the_focus_line_follows_tab`: from `Source`, after one `Tab` the
   `focus_line` is the `File` line's index, and `cursor` is `Some`.

### 4.2 `crates/cli/tests/new_view.rs` (new)

On a 80x24 `TestBackend`:

6. `the_form_has_its_badge`: row 0 contains `NEW DESIGN` and `nothing is
   written until you approve`, and not `verified`.
7. `the_cursor_sits_in_the_focused_field`: after typing `sid` into
   `File`, the backend's cursor is at the column after `sid` on the
   `File` row: column 15 (1 for the border, 11 for the label, then 3).
   Corrected 2026-10-22: the contract first said 2 for the border and
   16 in all; w7k4f measured the frame's inner area starting at column 1.
8. `a_refused_preview_is_red`: the `✗` cell of test 3's preview line has a
   red foreground.

### 4.3 Rules that must be proven

1. The 1-based conversion in `reading_words` (use start, not start + 1):
   test 4 fails.
2. Hidden fields left out: test 2 fails.
3. The cursor set: test 7 fails.

## 5. Do not

- Change only `crates/cli/src/new.rs` (two `pub mod` lines), the new
  `crates/cli/src/new/text.rs` and `crates/cli/src/new/view.rs`, the two
  new test files, and in `crates/cli/src/review/view.rs` and
  `crates/cli/src/review/text.rs` only `pub(crate)` on the functions named
  above.
- No change to the form's state or keys, `main.rs`, `args.rs`, luria-core,
  fixtures, docs, Cargo files or `vendor/`.
- Do not stage or commit.

## 6. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the §4.3 proofs, and anything that did not match.
