# Milestone 8c: the CDS map (core)

Status: **round 1 in progress**. Planner: Claude. Parent: `docs/tasks/milestone-8-view.md`
§4.2, full tier (D051). The fixtures it needs are already written and
independently verified (commit `9dc1d38`, `tests/fixtures/view/`); nothing in
this contract may change them.

**Corrections the planner issued in round 1**, after the worker proved the
§6.2 block could not pass as handed over. All three are defects in the
planner's own test code, not in the implementation, and no assertion changed:
`STANDARD_CODE_NAME` is what every other call site passes to `Cds::create` (a
bare `"Standard"` is refused as `UnsupportedGeneticCode`); the joined 6-mer's
arc read `TAG…`, a non-terminal stop `translate` refuses before `CdsMap` is
built; and `CdsMap` cannot derive `Eq` because its fields are `PartialEq`-only.
§8's first guard experiment also predicted a failure that does not happen,
because `codon_positions` refuses residue number 0 on its own. The §6.2 text
below carries all three corrections. One more defect is formatting only: the
block as written is not `rustfmt`-clean, so the long `assert_eq!` calls are
reflowed onto several lines. `cargo fmt --all` is sanctioned for that, and a
reflow is not a change to a test — no assertion, name, literal or comment may
differ afterwards.

## 1. What and why

The view must answer two questions without doing coordinate arithmetic of its
own: which three reference bases encode a numbered residue, and which residue
a base the reader points at belongs to. The map bar, the status line and every
residue-level hit in milestone 8 go through these two functions, which is why
this contract comes before any drawing code.

It is a small surface with a long tail of edge cases: minus strands read
backwards, CDSs whose codons straddle the origin, a terminal stop codon that is
a codon but not a residue, a CDS with no terminal stop at all, and a CDS whose
length is not a multiple of three. The fixtures pin every one of them before
any Rust exists.

## 2. Requirements this contract applies (`docs/decisions.md`)

- **D006:** reference coordinates inside core are zero-based and half-open
  (`[start, end)`); 1-based inclusive numbering is a user-facing convention
  that appears at the CLI edge. The fixture tables are GenBank-style —
  1-based inclusive — so the test converts, and says so where it converts.
- **D010:** residues are numbered from 1 for the first translated codon; the
  terminal stop codon is a codon and **not** a residue. A CDS whose last codon
  is not a stop has no codon to exclude — `a_complete_cds_needs_no_start_or_stop_codon`
  in `crates/core/src/translation.rs` already proves core does not assume one.
- **D036:** a `join(...)` whose parts abut across the origin is one arc; a CDS
  stores its intervals in reference order, not reading order. The reading of a
  wrapped CDS is settled by D036 plus the hand-derived `mj1`/`mj2` entry in
  `tests/fixtures/genbank/README.md`: the CDS's text is the joined arc along
  the molecule, and a minus-strand CDS takes its reverse complement, so it
  starts at the complement of the arc's lowest-numbered base.
- **D040:** the intervals a *result* records stay in reference order, and D040
  says outright that this "is not the strand's reading order". That is about
  result records, not about this function: `residue_bases` returns reading
  order, because that is what `mutation.rs::codon_positions` returns today and
  what the view draws. Naming the order in the doc comment and in every fixture
  column header is what keeps the two from being confused (D050).
- **D050:** a hit that ends on the stop codon carries no residue number;
  leftover bases at the end of a frame or CDS get nothing.
- **D042:** step 10 of `import_cds` translates every CDS it imports and passes
  translation errors through, so a CDS and a `Translation` taken from the same
  import always agree. `CdsMap` does not re-check that.

## 3. The API (new file `crates/core/src/cds_map.rs`)

Add `pub mod cds_map;` to `crates/core/src/lib.rs`, keeping that list's
existing alphabetical order (so after `pub mod cds;`, before `pub mod
codon_usage;`). Make `mutation.rs::codon_positions` `pub(crate)`; change
nothing else in `mutation.rs`.

```rust
use crate::cds::Cds;
use crate::error::DomainError;
use crate::interval::NucleotideInterval;
use crate::strand::Strand;
use crate::translation::Translation;

/// Where a reference base sits inside a CDS.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodonPlace {
    /// Which codon of the CDS, counted in reading order, starting at 1.
    pub codon: u64,
    /// The base's place within that codon, 1..=3 in reading order.
    pub place: u64,
    /// The residue number this codon produces, or `None` for the terminal
    /// stop codon (D010) and for a codon the CDS does not have.
    pub residue: Option<u64>,
}

/// The residue ↔ coordinate map of one CDS, built from the CDS and its
/// translation. `Cds` alone cannot answer either direction: whether the last
/// codon is a stop is a fact about the sequence, and the residue count is the
/// translation's, not `total_length() / 3` (D010).
///
/// `Eq` is not derivable here: `NucleotideInterval` and `Strand` derive
/// `PartialEq` only, and this contract may not touch their files.
#[derive(Clone, Debug, PartialEq)]
pub struct CdsMap {
    intervals: Vec<NucleotideInterval>,
    strand: Strand,
    total_length: u64,
    residues: u64,
    terminal_stop: bool,
}

impl CdsMap {
    pub fn new(cds: &Cds, translation: &Translation) -> CdsMap;

    /// Residues, excluding a terminal stop codon (D010).
    pub fn residues(&self) -> u64;
    /// Complete codons: `total_length() / 3`, floored.
    pub fn codons(&self) -> u64;
    pub fn terminal_stop(&self) -> bool;

    /// The three reference positions of residue `number`'s codon, in the
    /// order the codon is read: ascending on the plus strand, descending on
    /// the minus strand. Zero-based half-open coordinates (D006).
    ///
    /// A number with no residue — 0, or anything above `residues()`, which
    /// would address the terminal stop codon — is refused the way
    /// `codon_positions` refuses it: `DomainError::InvalidMutationRequest`
    /// with rule `CodonOutsideSequence`.
    pub fn residue_bases(&self, number: u64) -> Result<[u64; 3], DomainError>;

    /// Which codon holds this zero-based reference position, and what residue
    /// that codon produces. `None` for a position outside the CDS, and for a
    /// base of an incomplete final codon when the CDS's length is not a
    /// multiple of three (D050 gives leftover bases nothing).
    pub fn base_residue(&self, position: u64) -> Option<CodonPlace>;
}
```

`CdsMap::new` takes `cds.total_length()` and `cds.intervals()`, and
`translation.residue_count()` and `translation.terminal_stop()`. It does not
translate, does not take a `Sequence`, and returns no `Result`.

## 4. The arithmetic, exactly

Everything below is zero-based biological offsets inside the CDS, measured in
the order the intervals are stored — the same convention
`mutation.rs::reference_of` uses, where biological offset 0 is the first base
of the first stored interval.

- **residue → positions:** call `codon_positions(intervals, strand, number,
  total_length)`; it already walks biological offsets `(number-1)*3 ..
  +3`, descending from the end on the minus strand, and already refuses an
  offset with no coordinate. Convert its `Vec` to `[u64; 3]` without panicking
  on `unwrap()`: build the array from three known-length steps or match on the
  length and return `CodonOutsideSequence` if it is ever not 3. Guard
  `number == 0` and `number > residues()` **before** calling it, so residue
  `residues() + 1` (the stop codon) is refused rather than drawn.
- **position → codon:** walk the stored intervals to get the biological offset
  `b` of `position`, and `total_length` as the sum of interval lengths. Then
  `r = if plus { b } else { total_length - 1 - b }`; `codon = r / 3 + 1`;
  `place = r % 3 + 1`. `residue = if r / 3 < residues { Some(r / 3 + 1) } else
  { None }`. `None` for a position in no interval, and for `r / 3 >= codons()`
  (the incomplete final codon of a CDS whose length is not a multiple of 3).
  That second case cannot be reached from an import — D042 refuses `len13` and
  `translate` treats such a length as validation — so it is defensive
  arithmetic, and §6.2 tests it by building a `CdsMap` directly rather than
  pretending `translate` would allow it.
- `use_checked_arithmetic`: every multiply and subtract goes through
  `checked_*`/`checked_sub` style guards, as `codon_positions` already does, so
  a huge `number` cannot wrap. No `unwrap()` outside `#[cfg(test)]`, and no
  `panic!` reachable from a public function (review-b3).

Four cases a reader can check by eye, each against the fixture value in
`tests/fixtures/view/cds-map-codons.tsv` (fixture columns are 1-based; the
Rust values below are the same positions minus one, per D006):

| CDS | call | expected |
| --- | --- | --- |
| `m1` (synthetic.gb, minus) | `residue_bases(1)` | `[23, 22, 21]` |
| `fP` (frames-62.gb, plus, joined) | `residue_bases(3)` | `[61, 0, 1]` |
| `fM` (frames-62.gb, minus, joined) | `residue_bases(1)` | `[7, 6, 5]` |
| `tet` (pBR322, plus) | `residue_bases(396)` then `residue_bases(397)` | `[1270, 1271, 1272]`, then `CodonOutsideSequence` |

`fP`'s codon 3 is the origin jump inside one codon — 62, then 1 and 2 in
fixture terms — and `fM` is what a wrapped minus-strand CDS reads as: its first
codon sits on bases 8, 7, 6 of a 62 bp record, the complement of the arc's
lowest base, exactly the rule `tests/fixtures/genbank/README.md` derives by
hand for `mj1` and `mj2`.

## 5. Fixtures (planner-written, already verified — read them, do not edit them)

| File | Use |
| --- | --- |
| `tests/fixtures/view/cds-map-codons.tsv` | one row per codon of `p1`, `m1`, `pj`, `synA`, `fP`, `fM`, in reading order: `codon_reading`, `residue`, `bases_reading`, `bases_ref_ascending` |
| `tests/fixtures/view/cds-map-bases.tsv` | one row per base of the same CDSs: `position`, `codon_reading`, `place_reading`, `residue`, `codon` |
| `tests/fixtures/view/cds-map-counts.tsv` | per CDS: `nt`, `codons`, `residues`, `terminal_stop`, and the first/last codon and last residue's reference positions |
| `tests/fixtures/view/cds-map-boundaries.tsv` | first three and last three codons of `tet` and `bla` |
| `tests/fixtures/view/cds-map-refused.tsv` | features the view must not number, with the rule that refuses each |
| `tests/fixtures/view/frames-62.gb` | the 62 bp circular record behind `fP` and `fM` |

How to build the maps in the test: `read_genbank` the record, `import_cds` the
name, `translate` the imported CDS, `CdsMap::new(&cds, &translation)`. `fP` and
`fM` come from `tests/fixtures/view/frames-62.gb` read with the existing
reader; if that record does not read, or `import_cds` refuses one of the two,
**stop and report the error** — do not edit `frames-62.gb`, and do not relax a
reader rule. Every fixture position is 1-based inclusive; convert once, in a
named helper with a comment citing D006, and never mix the two conventions
inside one comparison.

`cds-map-refused.tsv` names the rule as `luria`'s own code strings
(`cds_map.rs` never sees these CDSs; the test only checks that
`import_cds`/`read_genbank` still refuse them, so a future change that starts
numbering a refused feature fails here).

`python -I tests/fixtures/verify.py` must print `all 21 checks passed`. Those
Python checks are not part of the cargo checks (D039, D026).

## 6. Tests

### 6.1 `crates/core/tests/cds_map_fixtures.rs` (fixture-driven)

Write it in the style of `crates/core/tests/genbank_fixtures.rs`: a module doc
comment saying which fixture is compared against which implementation and that
the Python checks are separate; a TSV reader that checks each header row
exactly; a helper for the 1-based → zero-based conversion that names D006.
Tests, with these names:

1. `the_pbr322_cds_map_matches_the_fixture` — `tet` and `bla`: `nt`, `codons`,
   `residues`, `terminal_stop` from `cds-map-counts.tsv`; every
   `cds-map-boundaries.tsv` row compared with `residue_bases(codon_reading)`
   (as a sorted set against `bases_ref_ascending`, and as an ordered list
   against `bases_reading` translated into positions); and
   `base_residue` for each of those positions giving back that row's
   `codon_reading` and `place_reading`.
2. `every_base_of_a_small_fixture_cds_maps_to_its_row` — for `p1`, `m1`, `pj`,
   `synA`, `fP`, `fM`: every `cds-map-bases.tsv` row, in file order, must
   equal `base_residue(position)`, including `residue == stop` rows mapping to
   `residue: None`.
3. `residue_bases_inverts_base_residue_for_every_fixture_base` — for every
   base row whose `residue` is a number, `residue_bases(that number)` sorted
   must contain that position; and for every codon row, `residue_bases` must
   equal the row's three positions in reading order. **Ruling on the `stop`
   rows (settled with the worker, who flagged it):** a codon row whose
   `residue` column reads `stop` is not checked with `residue_bases` at all —
   the number that would address it is refused by D010. Those rows are checked
   through `base_residue` giving `residue: None`, plus the `residues + 1`
   refusal in test 5. Say so in a comment on this test, so the next reader does
   not read "every codon row" literally.
4. `the_map_counts_match_the_counts_table` — one row per CDS in
   `cds-map-counts.tsv`, checked field by field against `codons()`,
   `residues()` and `terminal_stop()`.
5. `a_number_with_no_residue_is_refused` — for each CDS,
   `residue_bases(residues + 1)` and `residue_bases(0)` are
   `DomainError::InvalidMutationRequest` with rule `CodonOutsideSequence`, and
   `residue_bases(residues)` is `Ok`.
6. `frames62_reads_and_both_wrapped_cdss_map` — the record's topology is
   circular, `fP` and `fM` both import, and the four §4 spot values above hold
   for `m1`, `fP`, `fM` and `tet`.
7. `a_refused_cds_never_gets_a_map` — for each row of `cds-map-refused.tsv`,
   the import or the location read fails with the row's rule, using the
   `code()` strings already under test in `genbank_fixtures.rs`.

If any comparison fails, do not change a fixture or an expectation: stop and
report the file, the row, the expected value and what luria returned.

### 6.2 Unit tests in `cds_map.rs` (planner-written; add verbatim)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cds::Cds;
    use crate::cds::Completeness;
    use crate::cds::Numbering;
    use crate::error::DomainError;
    use crate::error::MutationRequestRule;
    use crate::genetic_code::STANDARD_CODE_NAME;
    use crate::identity::Identifier;
    use crate::interval::create;
    use crate::interval::OrderedIntervals;
    use crate::sequence::normalize;
    use crate::sequence::Sequence;
    use crate::sequence::SequenceTopology;
    use crate::strand::Strand;
    use crate::translation::translate;

    fn sequence(name: &str, text: &str, topology: SequenceTopology) -> Sequence {
        normalize(
            Identifier::create(name.to_string()).unwrap(),
            text.to_string(),
            topology,
        )
        .unwrap()
    }

    fn map_of(sequence: &Sequence, intervals: OrderedIntervals, strand: Strand) -> CdsMap {
        let cds: Cds = Cds::create(
            sequence,
            Identifier::create("cds-1".to_string()).unwrap(),
            intervals,
            strand,
            Completeness::Complete,
            STANDARD_CODE_NAME.to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap();
        let translation: Translation = translate(sequence, &cds).unwrap();
        CdsMap::new(&cds, &translation)
    }

    // ATG GCT TGG TAA: M A W stop.
    fn plus_linear() -> (Sequence, CdsMap) {
        let seq: Sequence = sequence("seq-p", "ATGGCTTGGTAA", SequenceTopology::Linear);
        let intervals: OrderedIntervals = vec![create(&seq.reference(), 0, 12).unwrap()];
        let map: CdsMap = map_of(&seq, intervals, Strand::Plus);
        (seq, map)
    }

    // The plus-strand text of m1 in tests/fixtures/genbank/synthetic.gb: its
    // reverse complement reads ATG CAG CAG TAG, M Q Q stop.
    fn minus_linear() -> (Sequence, CdsMap) {
        let seq: Sequence = sequence("seq-m", "CTACTGCTGCAT", SequenceTopology::Linear);
        let intervals: OrderedIntervals = vec![create(&seq.reference(), 0, 12).unwrap()];
        let map: CdsMap = map_of(&seq, intervals, Strand::Minus);
        (seq, map)
    }

    // A 6-mer whose CDS arc is [2,6) then [0,2): codon 1 is the arc's first
    // three bases — reference 2, 3, 4, which is TGG (Trp) — and codon 2 is the
    // arc's last three, reference bases 5, 0 and 1, which straddle the origin
    // and read TAA, the terminal stop. The literal is chosen for that shape:
    // an arc that reads a stop in codon 1 would be refused by `translate`
    // before a map could exist.
    fn joined_circular() -> (Sequence, CdsMap) {
        let seq: Sequence = sequence("seq-j", "AATGGT", SequenceTopology::Circular);
        let intervals: OrderedIntervals = vec![
            create(&seq.reference(), 2, 6).unwrap(),
            create(&seq.reference(), 0, 2).unwrap(),
        ];
        let map: CdsMap = map_of(&seq, intervals, Strand::Plus);
        (seq, map)
    }

    #[test]
    fn a_plus_codon_is_returned_ascending() {
        let (_seq, map) = plus_linear();
        assert_eq!(map.residues(), 3);
        assert_eq!(map.codons(), 4);
        assert!(map.terminal_stop());
        assert_eq!(map.residue_bases(1).unwrap(), [0, 1, 2]);
        assert_eq!(map.residue_bases(3).unwrap(), [6, 7, 8]);
    }

    #[test]
    fn a_minus_codon_is_returned_descending() {
        let (_seq, map) = minus_linear();
        assert_eq!(map.residues(), 3);
        assert_eq!(map.residue_bases(1).unwrap(), [11, 10, 9]);
        assert_eq!(map.residue_bases(3).unwrap(), [5, 4, 3]);
    }

    #[test]
    fn a_joined_codon_straddles_the_origin() {
        let (_seq, map) = joined_circular();
        assert_eq!(map.codons(), 2);
        assert_eq!(map.residues(), 1);
        assert_eq!(map.residue_bases(1).unwrap(), [2, 3, 4]);
        // The stop codon's three bases: 5, then 0 and 1.
        for position in [5, 0, 1] {
            let place: CodonPlace = map.base_residue(position).unwrap();
            assert_eq!(place.codon, 2);
            assert_eq!(place.residue, None);
        }
        assert_eq!(map.base_residue(4).unwrap(), CodonPlace { codon: 1, place: 3, residue: Some(1) });
    }

    #[test]
    fn the_terminal_stop_has_a_codon_and_no_residue() {
        let (_seq, map) = plus_linear();
        assert_eq!(map.base_residue(9).unwrap(), CodonPlace { codon: 4, place: 1, residue: None });
        assert_eq!(map.base_residue(11).unwrap(), CodonPlace { codon: 4, place: 3, residue: None });
        // D010: the stop codon is not residue 4.
        let error: DomainError = map.residue_bases(4).unwrap_err();
        assert!(matches!(
            error,
            DomainError::InvalidMutationRequest(problem)
                if problem.rule == MutationRequestRule::CodonOutsideSequence
        ));
        assert!(map.residue_bases(3).is_ok());
    }

    #[test]
    fn residue_zero_is_refused() {
        let (_seq, map) = plus_linear();
        assert!(matches!(
            map.residue_bases(0).unwrap_err(),
            DomainError::InvalidMutationRequest(problem)
                if problem.rule == MutationRequestRule::CodonOutsideSequence
        ));
    }

    // ATG GCT TGG: no stop codon at all. Nothing is excluded, so every codon
    // of this CDS is a residue (D010).
    #[test]
    fn a_cds_with_no_terminal_stop_numbers_every_codon() {
        let seq: Sequence = sequence("seq-n", "ATGGCTTGG", SequenceTopology::Linear);
        let intervals: OrderedIntervals = vec![create(&seq.reference(), 0, 9).unwrap()];
        let map: CdsMap = map_of(&seq, intervals, Strand::Plus);
        assert!(!map.terminal_stop());
        assert_eq!(map.codons(), 3);
        assert_eq!(map.residues(), 3);
        assert_eq!(map.residue_bases(3).unwrap(), [6, 7, 8]);
        assert_eq!(map.base_residue(8).unwrap(), CodonPlace { codon: 3, place: 3, residue: Some(3) });
        assert!(map.residue_bases(4).is_err());
    }

    // 13 bases: the final codon is incomplete, and D050 gives leftover bases
    // nothing. `translate` refuses such a CDS and D042 refuses the feature, so
    // no import can produce this map; the type is public, so its rule is
    // tested by building the value directly.
    #[test]
    fn a_base_of_an_incomplete_final_codon_has_no_place() {
        let seq: Sequence = sequence("seq-l", "ATGGCTTGGAATCAG", SequenceTopology::Linear);
        let map: CdsMap = CdsMap {
            intervals: vec![create(&seq.reference(), 0, 13).unwrap()],
            strand: Strand::Plus,
            total_length: 13,
            residues: 4,
            terminal_stop: false,
        };
        assert_eq!(map.codons(), 4);
        assert_eq!(map.base_residue(11).unwrap(), CodonPlace { codon: 4, place: 3, residue: Some(4) });
        assert_eq!(map.base_residue(12), None);
        assert_eq!(map.residue_bases(4).unwrap(), [9, 10, 11]);
        assert!(map.residue_bases(5).is_err());
    }

    #[test]
    fn a_position_outside_the_cds_maps_to_nothing() {
        let (_seq, map) = plus_linear();
        assert_eq!(map.base_residue(12), None);
        assert_eq!(map.base_residue(1000), None);
    }
}
```

Two of those tests exist to be *provably* load-bearing: disabling the
`number > residues()` guard must make
`the_terminal_stop_has_a_codon_and_no_residue` fail, and disabling the
`r / 3 >= codons()` guard must make
`a_base_of_an_incomplete_final_codon_has_no_place` fail. §8 asks you to
demonstrate both.

## 7. Do not

- No new dependencies. No `unsafe`. No changes outside:
  `crates/core/src/cds_map.rs` (new), the one `pub mod cds_map;` line in
  `crates/core/src/lib.rs`, the `pub(crate)` visibility on
  `mutation.rs::codon_positions`, and `crates/core/tests/cds_map_fixtures.rs`
  (new).
- No changes to `translation.rs`, `genbank_cds.rs`, `genbank.rs`,
  `genbank_location.rs`, `interval.rs`, `cds.rs`, `error.rs`, or any existing
  test. `error.rs` gets nothing new: reuse
  `InvalidMutationRequestProblem` / `MutationRequestRule::CodonOutsideSequence`
  as they are.
- Do not touch anything under `tests/fixtures/` — not a `.tsv`, not
  `frames-62.gb`, not a reference script, not `README.md`. They are
  planner-written and independently verified. A value that looks wrong is a
  finding to report, not a thing to fix.
- No CLI or view code, no `luria find`, no rendering. Those are 8h–8l.
- No new public API beyond §3. In particular do not add a map for a *frame*
  (that is 8d) and do not expose the interval walk as a public function.
- Do not edit `docs/decisions.md`, this contract, or
  `docs/tasks/milestone-8-view.md`.
- Do not stage or commit anything.

## 8. Checks

From the repository root, after the implementation:

```
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
python -I tests/fixtures/verify.py
```

The last one must print `all 21 checks passed`; it does not depend on Rust.

Then prove the two guards are load-bearing, each in one temporary edit that you
revert before finishing (report both experiments and the tests that failed):

1. remove the `number > residues()` (and `number == 0`) guard in
   `residue_bases` → `the_terminal_stop_has_a_codon_and_no_residue` must fail.
   `residue_zero_is_refused` is **expected to survive**: `codon_positions`
   refuses number 0 through its own `checked_sub(1)` (`mutation.rs:2105`), and
   that path already has coverage in `mutation.rs`'s tests. Report which of the
   two failed rather than making the prediction come true.
2. remove the `r / 3 >= codons()` guard in `base_residue` →
   `a_base_of_an_incomplete_final_codon_has_no_place` must fail.

If a guard can be removed without any test failing, add the test that catches
it rather than keeping dead code, and say so in the handoff.

## 9. Handoff

Report: files changed; the exact commands run and their results; the two guard
experiments with the failing test names; anything about the fixtures that
surprised you (a value you could not reproduce, a record the reader refused);
and every deviation from this contract, with the reason. If you are blocked or
unsure, stop and report — do not guess and do not ask the user.
