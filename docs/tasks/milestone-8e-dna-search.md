# Contract 8e — core DNA search (plan §4.4)

Planner: Claude.

Status: written, not yet delegated — it waits for 8d to land, because both
contracts add one line to `crates/core/src/lib.rs`. Tier: **full** (D051):
tests first-class, every rule pinned by a named test, guard experiments
required. Plan row **8e** in `docs/tasks/milestone-8-view.md` §7.

Depends on: **8a1c** (landed as 8496c52 — `search-letters.tsv`,
`search-dna-patterns.tsv`, `search-dna-hits.tsv` and
`tests/fixtures/view/reference/search_dna.py`). Do not start 8e while a worker
still owns 8d.

Read before working: this file, plan §4.4, D052 (M8-7, M8-8, M8-16), D050, D049,
and `tests/fixtures/view/README.md` ("DNA search: which letters a pattern may
match"). Plan §4.4 is the rule source; the tables are the expected answers.

## 1. What this contract fixes

One new module, `crates/core/src/search.rs`, plus one line in
`crates/core/src/lib.rs`: `pub mod search;` after `pub mod q5_expand;` and
before `pub mod sequence;` (module list is alphabetical; `search` sorts before
`sequence` because `sea` < `seq`).

```rust
/// The strand a DNA hit was found on. `Both` is its own state, not a pair of
/// flags: a place that matches on both strands is one hit (M8-8, D052).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitStrand {
    Plus,
    Minus,
    Both,
}

impl HitStrand {
    /// The symbol the tables and `luria find` use: `+`, `-`, `both`.
    pub fn symbol(self) -> String;
}

#[derive(Clone, Debug, PartialEq)]
pub struct DnaHit {
    /// Zero-based position of the first covered base on the top strand.
    pub start: u64,
    /// Number of bases covered, which is the folded pattern's length.
    pub length: u64,
    pub strand: HitStrand,
    /// True when the span runs past the end of a circular record.
    pub wraps_origin: bool,
}

/// §4.4's matching rule, public so the 225-row letter table can pin it:
/// a template letter matches a pattern letter only when every base the
/// template letter stands for is one of the pattern letter's bases.
pub fn covers(template: char, pattern: char) -> bool;

/// Every hit of `pattern` on `sequence`, both strands, in the order §4.4 gives.
pub fn find_dna(sequence: &Sequence, pattern: &str) -> Result<Vec<DnaHit>, DomainError>;
```

`HitStrand` is a new type on purpose. `crates/core/src/strand.rs` has `Strand`
with exactly `Plus` and `Minus`, and its variants are matched exhaustively
across core; widening it to carry a `Both` state would turn a CDS question into
a search question, and `Option<Strand>` for "both" is ambiguous with "unknown".

Two new refusals in `crates/core/src/error.rs`, following the precedent
`EmptyIdentifier` and `InvalidIdentifierCharacters` already set (one variant for
empty, one for a bad letter, no shared reason enum):

```rust
/// Payload: a search pattern with nothing in it.
EmptySearchPattern,

/// Payload: a pattern letter outside the IUPAC DNA set after folding.
InvalidSearchPatternCharacters(InvalidSearchPatternCharactersProblem),

#[derive(Clone, Debug, PartialEq)]
pub struct InvalidSearchPatternCharactersProblem {
    /// The pattern as the user wrote it, before folding.
    pub pattern: String,
    /// Zero-based index into `pattern` of the first offending letter.
    pub position: u64,
    /// That letter, as written. The message names it: §4.4 requires it.
    pub characters: String,
}
```

Both go in the `MalformedInput` category, next to the sequence-character rules
(the arm group around `DomainError::InvalidSequenceCharacters(_)` near line 866),
get rule names `empty_search_pattern` and `invalid_search_pattern_characters`
(the arm near line 929), get a message (the arm near line 991 — a refusal that
names only "invalid pattern" fails this contract, because §4.4 requires naming
the letter), and get a row each in `every_variant_has_its_pinned_code`
(`error.rs:1446`), whose `assert_eq!(cases.len(), 52)` becomes 54. Nothing else
in that file. A refusal must name the letter and its position.

## 2. Reuse it must not skip

- `sequence::bases_of(letter: char) -> Option<Vec<char>>` (`sequence.rs:145`)
  is the letter-to-bases table. `covers` is built on it. A second copy of the
  IUPAC sets is a defect; so is comparing letters instead of base sets.
- `sequence::complement_letter` and `sequence::DNA_ALPHABET`. The reverse
  complement of the pattern is built by mapping each folded letter through
  `complement_letter` and reversing. Do not call `sequence::reverse_complement`
  on the pattern: it refuses `U`, and `U` is legal in a pattern (see below).
- `sequence::SequenceTopology` (`sequence.rs:63`) and the record's own topology
  decide wrapping. A FASTA record has already been given a topology by its
  caller (D049): the fixture test reads `frames-amb.fa` with
  `SequenceTopology::Linear`, exactly as the view will.
- The fixtures from 8a1c are the expected answers. Do not re-derive any of them
  in Rust, and do not write a Rust-side copy of the matching rule to compare
  against itself: the tables come from Python that shares no code with `crates/`
  (D039).

## 3. Files 8e may change

- `crates/core/src/search.rs` (new), with its `#[cfg(test)] mod tests`.
- One line in `crates/core/src/lib.rs`, as above.
- `crates/core/src/error.rs`: the two variants, one payload struct, the category
  arm, the rule-name arm, the message arm, and the two rows in the rule-name
  table. Nothing else in that file.
- `crates/core/tests/search_fixtures.rs` (new).

Off limits: every other file under `crates/` (including `frame.rs` and
`frames.rs`, which belong to 8d), everything under `tests/` and `docs/`,
`Cargo.toml` and any new dependency, `#[allow(...)]`, `unwrap`, `expect`,
`panic!`, `unreachable!` and unchecked indexing outside `mod tests`, and any git
write command. If something here cannot be done within those files, stop and
report instead of picking a different design.

## 4. Rules, and the test that pins each

In `search.rs`'s `mod tests` unless the row says otherwise.

| # | Rule (§4.4) | Test |
| --- | --- | --- |
| 1 | `covers` is containment, not intersection: `N` matches only `N`; `A` is matched by `A R W M D H V N`; `R` by `R D V N`; `S` is not matched by `R` even though they share `G`. | `covers_uses_containment_not_intersection`, and all 225 rows of `search-letters.tsv` in `search_fixtures.rs` |
| 2 | Pattern letters are folded before anything else: case is dropped, then `U` becomes `T`. A pattern may hold `U`, which a stored sequence may not. | `gaattuc_finds_the_ecori_site`, `lowercase_pattern_finds_the_same_hit` |
| 3 | An empty pattern is refused; a letter outside the IUPAC set is refused naming that letter and its index. Folding happens before validation, so `u` is not an error. | `empty_pattern_is_refused`, `illegal_pattern_letter_names_the_letter` (assert the message contains the letter) |
| 4 | Minus-strand hits are the top-strand hits of the pattern's reverse complement, reported with strand `-` and the same top-strand coordinates. | `minus_hit_of_a_constructed_pattern_sits_at_the_same_start_as_its_reverse_complement_plus_hit` |
| 5 | A span that matches on both strands is one hit, `Both`. | `a_palindromic_site_is_one_both_hit`, `pbr322_gaattc_is_one_hit_not_two` |
| 6 | On a circular record a hit may start in the last `width − 1` bases and wrap; `wraps_origin` is `start + width > length`, computed, not inferred from a doubled string. On a linear record nothing wraps. | `circular_site_spanning_the_origin_is_one_wrapping_hit`, `the_same_pattern_on_a_linear_record_finds_nothing_there` |
| 7 | A pattern longer than the record finds nothing — it is not an error. | `pattern_longer_than_the_record_finds_nothing` |
| 8 | A pattern as long as the record: on a circular record it hits at the rotation start it matches, and any such start above 0 wraps; on a linear record the only legal start is 0. | `equal_length_pattern_hits_at_its_rotation_and_wraps`, `equal_length_pattern_on_a_linear_record_hits_only_at_zero` |
| 9 | Every match position is reported, overlaps included: `AA` on `AAA` is three plus hits. | `aa_on_aaa_is_three_plus_hits` |
| 10 | There is no cap in core. A pattern that matches everywhere returns one hit per base. | `pattern_n_on_pbr322_returns_every_base` (4361 hits, all `Both`, all length 1, starts 0..=4360) |
| 11 | Hits are ordered by start ascending. A single pattern has one length, so at most one plus and one minus hit share a start and they merge: the "plus before minus" tie-break can never fire. Say so in a comment where the ordering happens. | `hits_are_ordered_by_start`, plus the fixture ordering assertion |
| 12 | An ambiguity letter in the record is matched only by containment. `AAR` finds nothing where the record has `A A N`; `AAN` matches there. | `aar_does_not_match_a_run_of_a_a_n`, and the `framesamb` rows in the fixture |

## 5. Guard experiments

Each one: disable the rule once, run the named test, confirm it fails, restore
the file byte-identically (`md5sum` before and after, snapshot in
`target/tmp`, never `git checkout --`). A named test that still passes with the
rule removed is a defect in the test: strengthen it and say so.

- **G1** `covers` → intersection (`bases_of(t).any_in(p)`) → rule 1 test and the
  letter-table fixture must fail.
- **G2** drop the reverse-complement arm → rule 4, rule 5 (`pbr322` landmarks).
- **G3** emit plus and minus separately instead of merging → rule 5 and the
  fixture's one-hit-per-start ordering.
- **G4** circular start range stops at `length − width` (no wrapping) → rule 6,
  and the fixture's `GAATTC` row at start 4358.
- **G5** allow wrapping on a linear record → rule 6's second test, and the
  fixture's `synlin` / `framesamb` rows.
- **G6** remove the `U` → `T` fold → rule 2. It must fail as a hit-count
  failure, not as an error-path accident; if the only failure is the illegal
  letter refusal, the test is wrong, because a `U` pattern would then be
  refused rather than searched.
- **G7** remove case folding → rule 2.
- **G8** validate before folding (so `u` and `U` are illegal) → rule 3.
- **G9** advance by `width` after a match (skip overlaps) → rule 9.
- **G10** cap results at 200 → rule 10.
- **G11** return hits in scan order without sorting, or sorted by start
  descending → rule 11.
- **G12** treat a too-long pattern as an error → rule 7.

## 6. The fixture test, `crates/core/tests/search_fixtures.rs`

Read the three tables the same way `crates/core/tests/cds_map_fixtures.rs` reads
its tables: names, a pinned header per file, and records read with
`genbank::read_genbank` / `fasta::read_fasta`, obtaining the `Sequence` the way
that file obtains one from a `GenBankRecord`. `frames-amb.fa` is read with
`SequenceTopology::Linear` (D049), which is what makes its `wrap_used = no`
column true.

- **`search-letters.tsv`** — assert the header verbatim and 225 rows. For each
  row, `covers(template_letter, pattern_letter)` must equal the `subset_match`
  column, and the `intersection_match` column must differ from what core does
  wherever `rules_agree` says `no` (110 rows). Assert the count of disagreeing
  rows is 110, so an accidentally narrower grid fails.
- **`search-dna-patterns.tsv` and `search-dna-hits.tsv`** — assert both headers
  verbatim and 44 / 4542 rows. Group hit rows by the pattern **as written**
  (never by `normalized_pattern`: `GAATTC`, `gaattc` and `GAATUC` fold together
  and would collapse into one group). For each pattern row: call `find_dna` with
  that pattern as written on that record, and assert the result matches the
  group's rows in length and in every field — `start`, `length`, `strand.symbol()`
  against the `strand` column, and `wraps_origin` against the column. Also
  assert the window of record bases the hit covers, read back from the
  `Sequence` and wrapped modulo the length when `wraps_origin`, equals the
  `top_strand_bases` column: that is an independent check of the coordinates,
  because the column came from Python.
- Assert `hit_count` in the patterns table equals the number of hit rows for
  that record and pattern, and assert the starts in each group are strictly
  ascending (rule 11).
- Assert `wrap_used` is `yes` exactly where the record core was given is circular.
- Every one of those assertions must be capable of failing: a missing pattern row
  or a missing group is a test failure, not a skip.

## 7. Checks, in this order

From the repository root, with `export TMPDIR=$PWD/target/tmp` after
`mkdir -p target/tmp`:

1. the named tests, written first, against a stub that does not implement the
   rules yet — they must fail for the right reason, not because of a compile
   error;
2. implement, then `cargo fmt --all -- --check`;
3. `cargo test --workspace`;
4. `cargo clippy --workspace --all-targets -- -D warnings`;
5. `python3 -I tests/fixtures/verify.py` (must stay at "all 22 checks passed":
   `search_dna.py` is deliberately not registered yet — do not register it);
6. the twelve guard experiments, each restored byte-identically;
7. `git diff --check` and `git status --short`.

Report: files changed, the commands and their exact output, every guard
experiment with what failed when the rule was out and the `md5sum` proof of
restoration, plus anything the contract left ambiguous. Never describe a check
as passing without running it. Do not stage or commit anything.
