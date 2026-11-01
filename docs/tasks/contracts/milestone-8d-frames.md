# Contract 8d — core: frame translation (§4.3)

Planner: Claude.

Status: **Implemented 2026-10-24** by the local worker in one round (label
`w8d`), reviewed and accepted 2026-10-26. Full tier (D051). Plan row **8d** in
`docs/tasks/contracts/milestone-8-view.md` §7. The worker's own report was not kept; its
`md5-8d.txt` matched the tree, so its guard experiments were restored. Findings
from review:

* The one defect was formatting: two asserts in `frames.rs`'s
  `a_minus_frame_counts_from_the_far_end` failed `cargo fmt --check`. The
  planner applied `cargo fmt --all` (a formatting fix, allowed by `CLAUDE.md`);
  no other line changed.
* The planner re-ran E1–E8 on a scratch copy; every one fails at least one
  test. Two §7 predictions were wrong, both planner errors: E6 (ignore the
  window) cannot fail test 11, whose window is the whole record — it fails
  test 10 and test 8's empty window instead; E4 first fails test 4 on
  `synthetic` frame 2, not `frames62`, which is the same rule.
* Contract gap: §3 let the worker reword only `FrameResidue`'s own doc and its
  `middle` doc, so `number`'s doc (`frame.rs` line 55) still reads "1 at the
  CDS's first codon", false for `frame_residues`. Fix it in the next contract
  that touches `frame.rs`.
* Latent, older than 8d: `codon_at`'s minus branch checks only `position - 2 <
  0`, so a position at or past the end panics on the index. 8d never passes one
  (a minus frame starts at `length - f`), but E1 reached it. Now that two
  modules share `codon_at`, a later contract should bound it.

## 1. What this contract builds

`crates/core/src/frames.rs`: the six reading frames of a record — three per
strand — as the residues a frame produces, in the frame's own numbering, each
with the position of its codon's middle base.

The rules are already written down. They are not open:

* `docs/tasks/contracts/milestone-8-view.md` §4.3 (lines 254–274): frame numbering, the
  window, the no-wrap rule, `*`, `?`, and what a leftover base does.
* `docs/decisions/` D052 (`## D052: Frames, ambiguity codes, palindromic
  sites, and what \`find\` exits with`, line 1858): frames do not run across the
  origin, `N` is always `?`, and the reversibility note lists what may not be
  changed without the owner.
* `docs/decisions/` D050 (line 1764), especially its bullet "A leftover codon
  in a frame is not drawn": `?` means a *complete* codon that cannot be read.
* The fixtures in `tests/fixtures/view/` written for this: `frame-tables.tsv`
  (1318 codon rows) and `frame-tails.tsv` (36 rows), whose rules and
  independently verified expectations are recorded in
  `tests/fixtures/view/README.md` under "Six frames, and the bases a frame
  cannot use (8a1b)". Read that section including the `?` rule and the
  "Biopython answers that differ from luria's `?`" block.

## 2. What already exists, and must be reused rather than duplicated

`crates/core/src/frame.rs` is the mutant reading frame (plan §5.3a, M7-18 to
M7-21, contract 7i1). It already contains three things a reading frame needs,
each already tested by `tests/fixtures/review/`:

| Item | Where | Use in 8d |
| --- | --- | --- |
| `pub struct FrameResidue { pub number: u64, pub symbol: char, pub middle: u64 }` | `frame.rs` line 50 | return this type from 8d; do not define a second residue struct |
| `fn codon_symbol(codon: String) -> char` | `frame.rs` line 70 | call it; it is the `*` / `?` rule (`Residue` → letter, `Stop` → `*`, `Ambiguous` → `?`, `Invalid` → `?`) built on the public `genetic_code::lookup` |
| `fn codon_at(letters: &[char], position: i64, plus: bool) -> Option<String>` | `frame.rs` line 104 | call it; it returns `None` when fewer than three whole bases stand at that position, which **is** §4.3's "a frame never runs across the origin" |
| `mutant_residues(edited, cds, edit, window_start, window_end)` | `frame.rs` line 144 | copy its window convention: half-open `[window_start, window_end)`, and `window_end < window_start` refused with `DomainError::ReversedInterval(ReversedIntervalProblem { sequence, start, end })` |

So 8d is a thin module: frame start arithmetic, stepping, the window filter, and
a `ReadingFrame` type. Everything that decides a *letter* or whether a codon
exists at all is delegated to code that already has tests.

**Decision (planner), with the alternative recorded.** `frame_residues` takes
`ReadingFrame`, not `frame: u64`. An out-of-range frame is unrepresentable, so
the module has no invalid-input case to invent an error rule for; adding one
would mean new variants in `crates/core/src/error.rs`, which is off limits here.
A command that parses `--frame 4` produces a usage error (exit 2) in the cli,
not a domain error. The rejected alternative was `frame: u64` plus a new
`DomainError` rule; it widens the public error surface to express something the
type already expresses.

**Decision.** `frames.rs` re-exports the residue type (`pub use
crate::frame::FrameResidue;`) instead of defining its own, because the two uses
mean the same three things: an index counting from 1 in the context being read,
the one-letter symbol, and the zero-based position of the codon's middle base.

## 3. Files the worker may touch

1. `crates/core/src/frames.rs` — new. Module doc must name §4.3, D052 and D050,
   and say outright that this module and `frame.rs` are different things:
   `frame.rs` reads a CDS through an edit on an edited construct and stops at the
   first stop or unreadable codon; `frames.rs` reads a record's frames on
   unchanged sequence text and stops only at the end of the sequence.
2. `crates/core/src/lib.rs` — one line, `pub mod frames;`, placed alphabetically
   (after `pub mod frame;`, before `pub mod genbank;`).
3. `crates/core/src/frame.rs` — only these three changes, nothing else:
   * `fn codon_symbol` → `pub(crate) fn codon_symbol`;
   * `fn codon_at` → `pub(crate) fn codon_at`;
   * the doc of `FrameResidue` and of its `middle` field generalised so they are
     true for both call sites (one residue laid on the sequence it was read
     from: the edited construct for `mutant_residues`, a record for
     `frames::frame_residues`), and the `middle` doc naming both zero-based
     coordinate spaces. Keep the fields, their order, their types and their
     meaning.
4. `crates/core/tests/frames_fixtures.rs` — new, the fixture-driven test file.

   **Naming correction, 2026-10-24, a planner error in this contract's first
   draft.** It originally named `crates/core/tests/frame_fixtures.rs`, which is
   **not free**: that file is contract 7i1's mutant-frame table test from
   milestone 7 (committed in 9218b08, one test `every_frame_row_matches`, reading
   `tests/fixtures/review/frame.tsv`). Writing there would replace another
   contract's test, which §8 forbids. The file name follows the module under
   test, as `cds_map_fixtures.rs` follows `cds_map` and `fasta_fixtures.rs`
   follows `fasta`: plural `frames_fixtures.rs` for `frames.rs`. The singular
   file is not to be opened, edited or referenced by this contract, and its own
   command, `cargo test -p luria-core --test frame_fixtures`, still runs it as
   part of `cargo test --workspace`.

Nothing else in the repository. `tests/fixtures/` is read, never written, by
this contract; its tables and its Python reference scripts are already committed
and independently verified.

## 4. The public surface, exactly

```rust
/// One of the three reading frames of a strand. Six frames means three per
/// strand, numbered from the base of that strand's own first codon (§4.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadingFrame {
    First,
    Second,
    Third,
}

impl ReadingFrame {
    /// All three, in order — the view draws six lanes, this times two strands.
    pub const ALL: [ReadingFrame; 3];
    /// 1, 2 or 3. This is the number `frame-tables.tsv`'s `frame` column holds.
    pub fn index(self) -> u64;
    /// `None` for 0 and for anything above 3.
    pub fn from_index(value: u64) -> Option<ReadingFrame>;
}

/// The residues of one frame of `sequence` whose codon's middle base lies in
/// `[window_start, window_end)` of that sequence.
pub fn frame_residues(
    sequence: &Sequence,
    strand: Strand,
    frame: ReadingFrame,
    window_start: u64,
    window_end: u64,
) -> Result<Vec<FrameResidue>, DomainError>;
```

Field and argument names are literal. `Strand` is `crate::strand::Strand`,
`Sequence` is `crate::sequence::Sequence`, and the returned residues are
`crate::frame::FrameResidue` re-exported by `pub use`.

Behaviour, point by point, each one already pinned by the fixtures:

1. **Where a frame starts.** Plus frame *f* starts at 1-based base *f*, i.e.
   zero-based position `f - 1`, and steps up by 3. Minus frame *f* starts at the
   last base minus (*f* − 1), i.e. zero-based `length - f`, and steps down by 3,
   reading each codon through `codon_at(letters, position, false)`. Do not
   reverse the sequence and reuse the plus path: the fixture's
   `bases_ref_ascending` column and `frame.rs`'s `codon_at` both keep reference
   coordinates, and a rotated record is a different reading (§4.3, D052).
2. **Where a frame stops.** `codon_at` returns `None` as soon as three whole
   bases do not stand at the position, so a frame ends there and its 1 or 2
   leftover bases produce no residue at all (D050). `frame_residues` must not
   pad, wrap, or read base 1 after the last base of a circular record.
3. **Topology is never consulted.** `sequence.topology()` is not read by this
   module. A circular record is framed exactly like a linear record of the same
   text (§4.3: "The numbering is the same on a circular record"). A test pins
   this directly.
4. **The frame's own numbering.** `number` is 1 at the frame's first codon and
   counts every codon, including stops and unreadable codons. It has nothing to
   do with a CDS residue number (`cds-map-bases.tsv` holds that scale; the two
   columns look alike and are not the same thing, D050).
5. **A frame does not stop at a stop codon or at `?`.** Every complete codon of
   the frame gets a residue. This is where 8d differs from `mutant_residues`,
   whose reading ends at the first `*` or `?`; the fixture proves the difference
   (framesamb plus frame 1 keeps going after both). Say so in the function doc.
6. **The symbol** comes from `crate::frame::codon_symbol`, unchanged: letter,
   `*` for a stop, `?` for a codon holding any base outside `A C G T`
   (`genetic_code::lookup` reports `Ambiguous` for it, and D052 makes `N`
   always `?`). Do not write a second mapping and do not collapse an ambiguous
   codon to the letter its expansions agree on: Biopython does that (`GGN` →
   `G`, `CTN` → `L`, `TAR` → `*`), D010 and D052 say there is no guess.
7. **The window** is half-open `[window_start, window_end)` in zero-based
   reference coordinates, and the test is the codon's **middle** base, not its
   first base and not its span. A residue is kept when `window_start <= middle
   < window_end`. `window_end < window_start` is refused with
   `DomainError::ReversedInterval`, built the same way `mutant_residues` builds
   it. A window with `window_start == window_end` is legal and empty.
8. **`middle`** is the zero-based reference position of the codon's middle base:
   position `position + 1` on the plus strand, `position - 1` on the minus
   strand. It is a reference position, so it decreases as a minus frame reads —
   this is the same fact D040 records about reading order versus reference order,
   and the module doc must say it in those words.
9. **Arithmetic**: no `unwrap` or `expect` outside `mod tests`, no panicking
   index, no silent `u64` wrap. A record of length 1 or 2 has frames whose start
   is past the end or below zero; those produce empty vectors. Use the same
   `i64` idiom `codon_at` uses (`letters.len() as i64`) rather than inventing a
   checked variant, and keep every subtraction on `u64` behind
   `checked_sub`/`saturating_*` where a record could be shorter than the frame
   offset.

## 5. Tests the worker writes

`crates/core/tests/frames_fixtures.rs` reads the two tracked tables. Follow the
house pattern of `crates/core/tests/cds_map_fixtures.rs`: one `zero_based()`
helper applying the D006 conversion once per parsed column, a `record_of()` map
from record name to file, headers pinned verbatim, no rule re-derived in the
test that core also derives.

Record names in the tables map to:

| `record` | file | parser |
| --- | --- | --- |
| `synthetic` | `tests/fixtures/genbank/synthetic.gb` | GenBank |
| `synlin` | `tests/fixtures/genbank/synthetic-linear.gb` | GenBank |
| `frames62` | `tests/fixtures/view/frames-62.gb` | GenBank |
| `frames61` | `tests/fixtures/view/frames-61.fa` | FASTA |
| `framesamb` | `tests/fixtures/view/frames-amb.fa` | FASTA |
| `pbr322` | `tests/fixtures/genbank/raw/J01749.1.gb` | GenBank |

FASTA files go through `crate::fasta::read_fasta(Identifier, text,
SequenceTopology::Linear)` — D049 makes the view assume linear for a FASTA and
show the assumption; the fixture's `topology` column reads `unknown` because the
file says nothing, and a test asserts both facts side by side rather than
reconciling them.

Test names are literal:

1. `the_frame_table_headers_and_row_counts_are_pinned` — the two header lines
   verbatim (`record, seq_len, topology, strand, frame, codon_reading, residue,
   aa, codon, bases_ref_ascending, sampled` and `record, seq_len, topology,
   strand, frame, bases_in_frame, complete_codons, leftover_bases, leftover`),
   and 1318 codon rows, 36 tail rows.
2. `every_frame_row_of_a_short_record_matches_core` — for every row whose
   `sampled` is `no`, in the order the table lists them per record/strand/frame:
   `number == codon_reading == residue`, `symbol == aa`, and `middle ==
   median(bases_ref_ascending) - 1`. Assert all three per row with a message
   naming record, strand, frame and codon index. Derive `middle` from the row's
   own `bases_ref_ascending` column (the middle of its three reference bases) —
   that is reading the fixture, not re-implementing core's frame arithmetic.
3. `every_sampled_row_of_the_long_record_matches_core` — the `pbr322` rows:
   every stored row matches the core residue with the same strand, frame and
   `number`, and `sampled == yes` exactly when `seq_len > 500`, so the sample
   marker cannot drift away from the generator's rule.
4. `the_tail_table_pins_the_codon_count_and_the_shape_of_every_frame` — for all
   36 tail rows: `frame_residues(sequence, strand, frame, 0, length).len() ==
   complete_codons`; the middles of a frame are strictly increasing on the plus
   strand, strictly decreasing on the minus strand, and any two consecutive
   middles differ by exactly 3; and no returned `middle` equals (a `leftover`
   position from that row) − 1, which is how "the leftover bases are never
   drawn" is pinned without re-deriving where they sit.
5. `topology_does_not_change_the_frames` — take `frames-62.gb`'s text and build
   one record with `SequenceTopology::Circular` and one with
   `SequenceTopology::Linear`; all six frames return identical residue vectors,
   and the circular one is not longer at the far end (this is D052's no-wrap
   rule pinned in core, not only through a table).
6. `a_fasta_record_reports_unknown_topology_in_the_table_and_is_read_as_linear`
   — as described above, for `frames61` and `framesamb`.
7. `the_ambiguous_record_reads_letter_star_and_question_mark` — `framesamb`
   plus frame 1 and minus frame 1, each as one run of eleven symbols. The
   expected runs, read off the table's `aa` column by the person writing the
   test and written into it as literals:

   ```text
   plus  frame 1: MY**?????V?
   minus frame 1: ?H?????LLIH
   ```

   The test asserts both that the run built from the table equals the literal
   and that core's residues produce the same run. Do not build the expectation
   by calling core, and do not shorten the literal to "contains `?`": the
   position of every `?` is the point.
8. `a_reversed_window_is_refused` — `window_end < window_start` gives
   `DomainError::ReversedInterval`, and an empty window (`start == end`) gives
   an empty vector.

Inside `crates/core/src/frames.rs`, `mod tests` (build sequences with
`crate::sequence::normalize` as `cds_map.rs`'s unit tests do):

9. `reading_frame_indexes_one_two_and_three` — `ALL`, `index()`, and
   `from_index` returning `None` for 0 and 4.
10. `the_window_holds_residues_by_their_middle_base` — a 12-base record where
    plus frame 1 has middles at 1, 4, 7, 10; window `[4, 8)` gives the two
    residues with middles 4 and 7; window `[4, 7)` gives the one with middle 4;
    window `[0, 1)` gives none although base 0 is a codon's first base.
11. `a_record_shorter_than_a_codon_gives_no_residues_on_either_strand_and_no_frame`
    — lengths 1 and 2, all six frames, on both strands: empty, no panic.
12. `an_unreadable_codon_does_not_end_the_frame` — a 6-base record whose first
    codon holds `N` and whose second is an ordinary codon: two residues, the
    first `?`, the second a letter. Same again with a stop codon first: two
    residues, first `*`.
13. `a_minus_frame_counts_from_the_far_end` — a 6-base record written so that
    plus frame 1 and minus frame 1 disagree in both symbol and position; assert
    the middles and symbols explicitly.

## 6. Checks, run from the repository root

```
cargo fmt --all -- --check
cargo test --workspace
cargo test -p luria-core --test frames_fixtures
cargo clippy --workspace --all-targets -- -D warnings
python3 -I tests/fixtures/verify.py
git diff --check
git status --short
```

`verify.py` must print `all 22 checks passed`. It regenerates nothing; if it
fails, the fixtures changed and they must not.

## 7. Proving the tests bite (do each one, in order, then restore)

For each experiment: make the change once, run the named test, quote the failure
line, restore the file byte-identically (`md5sum` before and after), and report
the result. A weak assertion that passes with the rule removed is a defect in
the test.

| # | Change | Must fail |
| --- | --- | --- |
| E1 | Minus frame starts at `frame - 1` instead of `length - frame` | test 2, on `minus` rows |
| E2 | Plus frame starts at `frame` instead of `frame - 1` | test 2, at codon 1 of every plus frame of every short record |
| E3 | Step by 1 instead of 3 | test 4 (the spacing assertion) and test 2 |
| E4 | Pad a short codon by wrapping to position 0 instead of stopping | test 4 (count) on `frames62`, and test 5 |
| E5 | Map `Ambiguous` to `X` instead of `?` | test 7, and test 2 on `framesamb` |
| E6 | Ignore the window | tests 10 and 11 |
| E7 | Number from 0 | test 2 at the first row of every frame |
| E8 | Map `Stop` to `?` | test 7 only if it is not also caught by test 2 — report which caught it |

## 8. Do not

* No new dependency of any kind, dev or normal.
* Do not touch `crates/core/src/error.rs`, `genetic_code.rs`, `translation.rs`,
  `cds_map.rs`, `cds.rs`, `sequence.rs`, `interval.rs`, `fasta.rs`,
  `genbank*.rs`, `mutation.rs`, or anything under `crates/cli/`.
* Do not touch `crates/core/src/frame.rs` beyond the three changes in §3, and do
  not change any behaviour there; the review fixtures must keep passing.
* Do not edit, regenerate, reformat or delete anything under
  `tests/fixtures/`, including `verify.py` and the `reference/` scripts.
* Do not edit `docs/decisions/`, `docs/tasks/`, `docs/specs/cli.md`, or any test
  written by someone else. If a fixture looks wrong, stop and report it — three
  earlier rounds found real fixture bugs that way, and one found two bugs in a
  contract instead.
* Do not add a public item beyond §4. Do not add `#[allow(...)]`. No `unwrap`,
  `expect`, `panic!` or indexing that can go out of bounds outside `mod tests`.
* Do not stage or commit anything.

## 9. Report

List the files written with line counts, the exact commands run and their
results, every guard experiment with the failure it produced, anything you had
to decide that §2 and §4 left open, and anything you found suspicious but did
not change. If `codon_at` or `codon_symbol` turns out not to fit §4, stop and
report rather than re-implementing the rule a third time.
