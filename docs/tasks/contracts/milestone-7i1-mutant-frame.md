# Milestone 7i1: the mutant reading frame, in luria-core

Status: **Implemented 2026-10-21.** Proofs 1-4 failed as required. Refusals: split CDS `UnsupportedFeatureStructure`; first base in the edit `MutationVerificationFailure`; reversed edit or window `ReversedInterval`. The `?` rule is implemented but no fixture row exercises it. Planner: Claude. Parent:
`docs/tasks/contracts/milestone-7-tui.md` §5.3a (owner answers M7-18, M7-20, M7-21).
The expected values are in `tests/fixtures/review/frame.tsv`, written by
`tests/fixtures/review/reference/frame.py` (read its docstring: it is the
specification), independent of `crates/`.

## 1. What and why

The redesigned context screen shows the mutant protein between the two
strands, each residue letter over the middle base of its codon. The TUI
may not translate or compute coordinates (plan §3 rule 1, D003), so
luria-core does it: given the edited construct, the CDS and the edit,
it returns each mutant residue whose codon's middle base falls in a
window, with its number, its symbol and that middle base's position.

## 2. The rules (M7-20), as `frame.py` applies them

- The CDS's first base on its own strand (`start` for `+`, `end - 1` for
  `-`) is carried into the edited construct: kept when it lies before the
  edit's start, moved by the edit's length change (`written length - (edit
  end - edit start)`) when it lies at or past the edit's end. A first base
  inside the edited region is refused.
- From there, codons are read three bases at a time along the CDS's
  strand (for `-`, the complement of positions `p`, `p-1`, `p-2`), in the
  CDS's frame, on past any frameshift, whole codons only.
- The first stop codon gives `*` and ends the protein; a codon with any
  base that is not `A`, `C`, `G` or `T` gives `?` and ends it.
- Reading also ends at the construct's end, or once a codon's middle is
  past the window in the reading direction.
- A residue is returned when its middle base lies in `[window_start,
  window_end)` of the edited construct. Numbers count from 1 at the CDS's
  first codon.

## 3. Changes

New file `crates/core/src/frame.rs`; `lib.rs` gains `pub mod frame;`.
No new dependency.

```rust
/// One residue of the mutant protein.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameResidue {
    /// 1 at the CDS's first codon.
    pub number: u64,
    /// The one-letter symbol, `*` for a stop, `?` for an unreadable codon.
    pub symbol: char,
    /// The edited construct's zero-based position of the codon's middle base.
    pub middle: u64,
}

/// The edit, in template coordinates: `[start, end)` replaced by
/// `written_length` bases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditSpan { pub start: u64, pub end: u64, pub written_length: u64 }

/// The residues of the mutant protein whose codon's middle base lies in
/// `[window_start, window_end)` of `edited` (plan §5.3a, M7-20).
pub fn mutant_residues(
    edited: &Sequence,
    cds: &Cds,
    edit: EditSpan,
    window_start: u64,
    window_end: u64,
) -> Result<Vec<FrameResidue>, DomainError>;
```

- `cds` is the CDS as annotated on the **template**; its intervals give
  its first base and strand. Use its first interval in its own order
  (`cds.intervals()`, D040) for the first base: `start` of the first
  interval for `+`, `end - 1` of the first for `-`. A CDS with more than
  one interval is refused (see below); the Q5 workflow's CDSs today have
  one, and a split CDS needs its own rule.
- The codon table: `genetic_code::lookup`, `CodonOutcome::Residue(r)` →
  `r.symbol()`'s one character, `Stop` → `*`, `Ambiguous` or `Invalid` →
  `?`. The minus-strand complement: `sequence::complement_letter`.
- Refusals, with existing `DomainError` variants only (read
  `crates/core/src/error.rs` and pick the closest; name your choice in
  the report): a CDS with more than one interval; a first base inside the
  edited region; an edit whose `start > end`; a window with `start >
  end`.
- Positions past the construct's end simply end the reading; never
  panic, never underflow (use checked or signed arithmetic).

## 4. Tests

### 4.1 `crates/core/tests/frame_fixtures.rs` (new)

Read `tests/fixtures/review/frame.tsv` at test time. For each row: load
the template (a `seq:` column, or a FASTA file under `tests/fixtures/`
read with `luria_core::fasta`'s reader), build the edited construct by
splicing `written` (`-` is empty) into `[edit_start, edit_end)`, as a
`Sequence` with `normalize`, build the CDS on the **template** with the
row's interval and strand (`complete`, standard code, from-cds-start),
call `mutant_residues`, and compare with the `residues` column parsed as
`number:symbol:middle` (`-` is none). One test, `every_frame_row_matches`,
naming the failing case.

(The CDS is built on the template and passed with the edited construct:
the function must read the CDS's coordinates only, never its reference.
If `Cds` cannot be built that way, stop and report.)

### 4.2 Unit tests in `frame.rs`

1. `a_split_cds_is_refused`.
2. `a_start_inside_the_edit_is_refused`.
3. `an_empty_window_gives_nothing`.
4. `reading_stops_at_the_construct_end`: a CDS whose last codon is cut by
   the construct's end returns no residue for it, and does not panic.

### 4.3 Rules that must be proven

1. Reading on past a stop: `syn-rep-02` and `syn-stop` fail.
2. The start not moved past an upstream edit: `syn-edit-before-cds`
   fails.
3. The minus strand read without complementing: `syn-minus` fails.
4. The frame reset after the edit (reading the downstream in the
   original frame): `syn-del-frameshift` fails.

## 5. Do not

- Change only `crates/core/src/lib.rs` (one line), the new
  `crates/core/src/frame.rs` and `crates/core/tests/frame_fixtures.rs`.
- No new dependency; `luria-core` stays dependency-free. No change to
  any other core module, the CLI, fixtures, `docs/`, any `Cargo.toml` or
  `vendor/`. Do not stage or commit.

## 6. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the proofs of §4.3, the `DomainError` variants you
chose, and any point where this contract and `frame.py` disagree; stop
and report rather than guess.
