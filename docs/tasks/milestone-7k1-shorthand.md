# Milestone 7k1: the mutation shorthand parser

Status: **Implemented 2026-10-21.** Proofs 1-3 failed as required. Positions are ASCII digits only; w7k1 found the reference script accepted other Unicode digits (and crashed on `²`), now fixed there with no change to the table. Planner: Claude. Parent:
`docs/tasks/milestone-7k-new.md` §3 (owner answer M7-35). The
specification is `tests/fixtures/new/reference/shorthand.py` (read its
docstring); its output `tests/fixtures/new/shorthand.tsv` is the expected
reading of every line, written independently of `crates/`.

## 1. What and why

`luria new` takes mutations as one line each, in a shorthand luria reads
itself, with no model. This contract writes that reader as a pure
function. Nothing calls it yet (7k2 does).

## 2. The module

New file `crates/cli/src/shorthand.rs`; `crates/cli/src/lib.rs` gains
`pub mod shorthand;`.

```rust
/// What one line of shorthand asks for, in a request's own terms:
/// zero-based half-open template coordinates (D006).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading {
    /// `D404N`, `D404N:AAC`: `codon` is `None` for the form's codon choice.
    Residue { number: i64, from: char, to: char, codon: Option<String> },
    /// `1210 G>A`, `1210-1212 GAT>AAC` (same length).
    Substitution { start: i64, end: i64, from: String, to: String },
    /// `1210-1212 GAT>TT` (different length).
    Replacement { start: i64, end: i64, from: String, to: String },
    /// `del 1210-1212`, `del 46`.
    Deletion { start: i64, end: i64 },
    /// `ins GAT after 1209`: `point` is the zero-based insertion point,
    /// equal to the 1-based base it follows.
    Insertion { point: i64, insert: String },
}

/// Why a line is not read; `code()` gives the fixture's code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Empty, Unreadable, ZeroPosition, ReversedRange, LengthMismatch,
    NotDna, SameBases, BadCodon, Stop, Unstated,
}

impl Refusal {
    /// The snake-case code: `empty`, `unreadable`, `zero_position`,
    /// `reversed_range`, `length_mismatch`, `not_dna`, `same_bases`,
    /// `bad_codon`, `stop`, `unstated`.
    pub fn code(self) -> &'static str;
    /// One English sentence a user can act on, naming what to change;
    /// for example `Unstated`: "a part of the residue change is `?`;
    /// write the residue, number and new residue". ASCII or the screens'
    /// Unicode; no trailing full stop.
    pub fn message(self) -> &'static str;
}

pub fn parse_line(line: &str) -> Result<Reading, Refusal>;
```

Rules: exactly `shorthand.py`'s `read`, in its order. In particular:

- Normalise first: runs of whitespace to one space, trim, U+2013 `–` to
  `-`.
- Residue lines: check for a stop (`*`, `X`, `Ter` as either residue,
  any case) first, then a `?` part, then hand the token (without any
  `:codon`) to `crate::interpret::notation::parse_substitution` and map
  any error to `Unreadable`. Do not write a second residue table. The
  codon after `:` must be three of ACGT (any case), else `BadCodon`;
  stored upper case.
- Bases are stored upper case. `from` and `to` are the typed text, not
  checked against the template here (7k3 does that).
- Positions are decimal with no sign and no leading zero; a position of
  0 is `ZeroPosition` except `ins … after 0`. Use checked arithmetic: a
  number that does not fit in `i64` is `Unreadable`.

If `parse_substitution` reads a token `shorthand.py` refuses, or the other
way round, stop and report the line; do not change either.

## 3. Tests

### 3.1 `crates/cli/tests/shorthand.rs` (new)

`every_shorthand_row_matches`: read `tests/fixtures/new/shorthand.tsv`
at test time; for each row compare `parse_line(line)` with the row (a
`refused` row compares `refusal.code()` with `from`; a residue's
`codon` column `policy` is `None`). Name the failing line.

### 3.2 Unit tests in `shorthand.rs`

1. `every_refusal_has_a_message`: each variant's `message()` is
   non-empty and distinct.
2. `a_huge_position_is_unreadable`: `"99999999999999999999 G>A"` is
   `Unreadable`, without panicking.

### 3.3 Rules that must be proven

Disable each once, run, restore; report the failing rows.

1. The 1-based conversion (store `A` instead of `A-1`): substitution,
   replacement and deletion rows fail.
2. The stop check before notation: `D404*`, `D404X`, `Asp404Ter` fail.
3. The same-length rule choosing substitution (superseded by 7k1b: only one base to one base is a substitution): `1210-1212 GAT>AAC`
   fails.

## 4. Do not

- Change only `crates/cli/src/lib.rs` (one line), the new
  `crates/cli/src/shorthand.rs` and `crates/cli/tests/shorthand.rs`.
- No change to `notation.rs` (only call it), fixtures, docs, Cargo files
  or `vendor/`. No new dependency (no regex crate: parse by hand).
- Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the §3.3 proofs, and anything that did not match.
