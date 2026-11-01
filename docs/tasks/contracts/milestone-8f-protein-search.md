# Contract 8f — core: protein search (§4.5)

Planner: Claude. Status: **Implemented 2026-10-26** by worker `w-8f` in two
rounds, reviewed and accepted. It fixed `ProteinTarget` and `ProteinHit`
before the protein tables were written, as plan §4.5 asks; those tables
(8a1c's §4.5 half: `search-protein-patterns.tsv`, `search-protein-hits.tsv`)
landed in commit 8b81222. The matching and order rules in §4 are D054,
approved by the owner 2026-10-26. Tier: full (D051). Plan row: **8f** in
`docs/tasks/contracts/milestone-8-view.md` §7. Findings from review:

* Round 2, memory: six-frame mode built all six frames' codons before
  searching any, about 370 MB on a 4.6 Mb record and 800 MB at D048's
  ceiling. Frames are now built, searched and dropped one at a time; the
  worker measured the search's peak falling from 416 MiB to 124 MiB on a
  4.6 Mb record. The floor left is one frame's `frame_residues` vector plus
  its codons; lowering it means changing `frames.rs`, which no contract
  covers.
* §7 understated most guards: G2 also fails 3, G3 and G4 also 5, G6 also
  11, G7 also 2, G9 also 10, G11 also 5, and test 12 fails under every
  guard. G3 as written trips a `debug_assert` first; with it removed it
  fails 5, 7 and 8.
* CDS targets never hold `?`: `translate` refuses an unreadable codon, so
  only frame mode can exercise the `?` rule of plan §4.5.
* Planner re-ran nine mutations on a scratch copy (`X` not matching `?`,
  ties minus first, minus-frame start at `middle`, `wraps` on `>=`, `B`
  allowed, the last offset skipped, the refusal naming the folded letter,
  no stop appended, no sort): all caught, before and after round 2.

## 1. What this contract builds

`crates/core/src/protein_search.rs`: `find_protein`, which finds an amino-acid
pattern in one CDS's translation or in the six frames of a record, and gives
each hit's bases, strand, frame and residue numbers.

Rules that apply, already settled:
- Plan §4.5 (`docs/tasks/contracts/milestone-8-view.md:308-336`): the pattern letters,
  the two modes, and the rule that `?` matches only `X`.
- D054: `X` matches the stop too, and hits come left to right by base in
  both modes.
- D050 M8-16: every overlapping hit is reported, with no cap in core. D050
  M8-17: in CDS mode the numbers are the CDS's and the stop codon carries
  none; in frame mode they count codons within the frame.
- D010: the terminal stop codon is a codon, not a residue.
- D052 M8-2: frames do not run across the origin, and topology is never read
  in frame mode.
- D036: a CDS may cross the origin, so its bases do too.
- D006: zero-based, half-open positions.

## 2. What exists and must be reused

| Item | Where (`file:line`) | Use here |
| --- | --- | --- |
| `translate(sequence, cds)` | `translation.rs:130` | CDS mode's residues; `residues_text()` and `terminal_stop()`; its errors are returned as they are |
| `codon_positions(intervals, strand, number, total_length)` | `mutation.rs:2090` (`pub(crate)`) | the three bases of codon `number` in reading order, stop codon included; never redo codon arithmetic |
| `frame_residues(sequence, strand, frame, 0, length)` | `frames.rs:119` | frame mode's residues; each `FrameResidue` has `number`, `symbol` and `middle` |
| `ReadingFrame::ALL` | `frames.rs:77` | the three frames, in order |
| `DomainError::EmptySearchPattern`, `InvalidSearchPatternCharacters` | `error.rs:807`, `error.rs:52` | the only refusals this module adds; built the way `find_dna` builds them (`search.rs:188-213`) |
| `Strand` | `strand.rs` | a hit's strand; a protein hit is never `Both` |

## 3. Files the worker may touch

1. `crates/core/src/protein_search.rs`: new, with its `mod tests`.
2. `crates/core/src/lib.rs`: one line, `pub mod protein_search;`, between
   `pub mod primer_quality;` and `pub mod q5;`.
3. `crates/core/src/error.rs`: the doc comment above
   `InvalidSearchPatternCharactersProblem` only. "a DNA search pattern holds a
   letter the IUPAC DNA alphabet has no such set for" becomes "a search
   pattern holds a letter outside its alphabet: IUPAC DNA (`find_dna`) or the
   protein letters (`find_protein`)". Its `U` -> `T` sentence stays, marked
   as DNA only. No code changes.
4. `crates/core/tests/protein_search_fixtures.rs`: new (§5 #12).

## 4. The public surface, exactly

```rust
/// What a protein search reads (§4.5).
#[derive(Clone, Copy, Debug)]
pub enum ProteinTarget<'a> {
    /// One CDS's translation, with its terminal stop as `*`.
    Cds(&'a Cds),
    /// The six frames of the whole record, as `frame_residues` reads them.
    SixFrames,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProteinHit {
    /// Zero-based first base, in reference order, of the run of bases the hit's
    /// codons cover (D006). On a CDS that crosses the origin the run may wrap.
    pub start: u64,
    /// 3 × the pattern's length.
    pub length: u64,
    pub strand: Strand,
    /// `None` in CDS mode.
    pub frame: Option<ReadingFrame>,
    /// The first and last codons' numbers. CDS mode: the residue number, `None`
    /// for the stop codon (D010, D050). Frame mode: the codon's number within
    /// its frame, never `None`.
    pub first: Option<u64>,
    pub last: Option<u64>,
    /// `start + length > sequence.length()`.
    pub wraps_origin: bool,
}

pub fn find_protein(
    sequence: &Sequence,
    target: ProteinTarget<'_>,
    pattern: &str,
) -> Result<Vec<ProteinHit>, DomainError>;
```

Rules, each pinned by the §5 test in brackets:

1. **Pattern.** ASCII upper case, then validate. Legal letters:
   `ACDEFGHIKLMNPQRSTVWY`, `X` and `*`. An empty pattern gives
   `EmptySearchPattern`. Any other letter (`B`, `J`, `O`, `U`, `Z`, digits, a
   space) gives `InvalidSearchPatternCharacters`, with the pattern as given,
   the zero-based position and the letter as written. [1, 2]
2. **Letters.** Pattern `X` matches every target symbol, `*` and `?`
   included (D054). Pattern `*` matches only `*`. Any other pattern letter
   matches only itself. So `?` is matched only by `X`. [3, 4, 5, 7]
3. **CDS mode.** The target is `translate(sequence, cds)?.residues_text()`,
   with `*` added when `terminal_stop()`. Target symbol `i` (zero-based) is
   codon `i + 1`, and its number is `Some(i + 1)` unless that codon is the
   stop, then `None`. The hit's run: on `+`, `start` is the first codon's
   `codon_positions(..)[0]`; on `−`, the last codon's `[2]`. `strand` is the
   CDS's, `frame` is `None`. [5, 6, 7, 8]
4. **Frame mode.** For `Plus` then `Minus`, and each frame in
   `ReadingFrame::ALL`, the target is `frame_residues(sequence, strand,
   frame, 0, sequence.length())?`. `first` and `last` are the first and
   last matched residues' `number`. `start` is the first matched residue's
   `middle - 1` on `+`, and the last matched residue's `middle - 1` on `−`.
   `wraps_origin` is always false, and `sequence.topology()` is never read.
   [9, 10, 11]
5. **Every match.** Every offset where the whole pattern fits is tried;
   overlaps are all reported. A pattern longer than the target finds
   nothing, which is not an error. There is no cap. [5, 10]
6. **Order** (D054). Both modes: by `start`, ascending; frame hits with
   the same start, `+` before `−`. In CDS mode this is not reading order: a
   minus-strand CDS lists later residues first, and a hit that crosses the
   origin sorts by its start near the end. No two hits share a start and a
   strand: a CDS's codons are disjoint, and a strand's three frames start on
   different bases mod 3, so the tie-break decides only between strands. Say
   so in a comment where the sort is. [3, 5, 6, 9]

**Decision (planner) — signature.** §4.5's sketch
`ProteinTarget::SixFrames(&Sequence)` leaves CDS mode without the bases it
translates. Here the sequence is `find_protein`'s first argument for both
modes. No new error variants: an empty or bad pattern uses 8e's two
refusals, so one rule has one code.

## 5. Tests the worker writes

Units in `protein_search.rs`. `tiny` is FASTA `>t\nATGAAATAG\n` (9 bp,
linear). Its frames are: `+1` `MK*`, `+2` `*N`, `+3` `EI`, `−1` `LFH`, `−2`
`YF`, `−3` `IS`. `pj` and `m1` are imported from
`tests/fixtures/genbank/synthetic.gb` with `import_cds`. `pj` is
`join(55..60,1..12)`, `MKMAW*`, plus strand, codons at zero-based 54, 57, 0,
3, 6, 9. `m1` is `complement(13..24)`, `MQQ*`, with codon 1 at 23, 22, 21
and codon 4 at 14, 13, 12. A hit is written (start, length, strand, frame,
first, last, wraps).

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `empty_pattern_is_refused` | `""` → `EmptySearchPattern`, in both modes |
| 2 | `a_letter_outside_the_protein_alphabet_names_itself` | `MbW` → position 1, characters `b`; `MAU` → position 2, `U`; `M W` → position 1, `" "`; the message contains the letter. Also `mkm` on `pj` equals `MKM` |
| 3 | `x_matches_every_symbol_the_stop_included` | `X` on `tiny`, six frames: exactly these 14 hits, in this order: (0,3,+,1,1,1), (0,3,−,1,3,3), (1,3,+,2,1,1), (1,3,−,3,2,2), (2,3,+,3,1,1), (2,3,−,2,2,2), (3,3,+,1,2,2), (3,3,−,1,2,2), (4,3,+,2,2,2), (4,3,−,3,1,1), (5,3,+,3,2,2), (5,3,−,2,1,1), (6,3,+,1,3,3), (6,3,−,1,1,1), all `wraps` false |
| 4 | `an_unreadable_codon_matches_only_x` | FASTA `>a\nATGNNNTAA\n` (`+1` is `M?*`): `MX*` and `MXX` each give exactly [(0,9,+,1,1,3)]; for each of the 20 residue letters `L`, `M{L}*` has no `+1` hit; `M**` has no `+1` hit |
| 5 | `every_codon_of_a_wrapping_cds_is_searchable` | on `pj`: `MAW` → [(0,9,+,−,3,5,no)]; `KM` → [(57,6,+,−,2,3,yes)]; `mkm` → [(54,9,+,−,1,3,yes)]; `X` → 6 hits with starts 0, 3, 6, 9, 54, 57 in that order, the one at 9 with `first` and `last` `None`; `MKMAWX` → [(54,18,+,−,1,None,yes)]; `MKMAWW` → none (as long as the target, last letter differs); `MKMAW*M` → none (longer than the target) |
| 6 | `cds_hits_come_in_base_order` | on `pj`, `M` → [(0,3,+,−,3,3,no), (54,3,+,−,1,1,no)]; on `m1`, `Q` → [(15,3,−,−,3,3,no), (18,3,−,−,2,2,no)] |
| 7 | `the_stop_codon_has_no_residue_number` | on `pj`: `W*` and `WX` each → [(6,6,+,−,5,None,no)]; `*` → [(9,3,+,−,None,None,no)] |
| 8 | `a_minus_strand_cds_starts_its_run_at_the_last_codon` | on `m1`: `QQ` → [(15,6,−,−,2,3,no)]; `Q*` → [(12,6,−,−,3,None,no)]; `MQQ*` → [(12,12,−,−,1,None,no)] |
| 9 | `frame_hits_come_in_base_order_plus_first` | on `tiny`: `F` → [(2,3,−,2,2,2), (3,3,−,1,2,2)]; `*` → [(1,3,+,2,1,1), (6,3,+,1,3,3)]; `I` → [(4,3,−,3,1,1), (5,3,+,3,2,2)] |
| 10 | `overlapping_frame_hits_are_all_reported` | FASTA `>k\nAAAAAAAAAAAA\n` (12 bp; every plus codon is `AAA` = `K`, every minus codon `TTT` = `F`): `KK` → exactly (0,6,+,1,1,2), (1,6,+,2,1,2), (2,6,+,3,1,2), (3,6,+,1,2,3), (4,6,+,2,2,3), (5,6,+,3,2,3), (6,6,+,1,3,4) |
| 11 | `frames_ignore_topology` | `tiny` read as circular gives the same result as linear, for `X` and for `*` |
| 12 | `protein_search_fixtures::every_protein_pattern_row_matches` | see below |

**Test 12** reads `tests/fixtures/view/search-protein-patterns.tsv` and
`search-protein-hits.tsv`, the way `crates/core/tests/search_fixtures.rs`
reads the DNA tables. Their headers are fixed here:

```text
record	target	pattern	normalized_pattern	pattern_length	hit_count	purpose
record	target	pattern	hit_index	strand	frame	start	length	wraps_origin	first	last	residues	top_strand_bases
```

`target` is `frames` or a CDS id; `frame` is `1`, `2`, `3` or `-`; `first` and
`last` are a number, or `stop` for `None`. Records are `synthetic`,
`pbr322` and `framesamb` (`frames-amb.fa`, read as linear), as in the DNA
tables. For each pattern row, import the CDS by its id when `target` is not
`frames`, then call `find_protein` with the pattern as written. Assert that
the result matches the group's rows in count, order and every field. Group
by (`record`, `target`, `pattern` as written). Then assert each row's
`top_strand_bases` equals the record's bases at `[start, start + length)`,
wrapped when `wraps_origin`, and that `hit_count` equals the group size. A
missing group fails the test.

Expected values in tests 3–11 were derived by the planner by hand from the
bases, not by running code. Test 12's come from the reference script.

## 6. Checks

`scripts/check`, plus `cargo test -p luria-core --test protein_search_fixtures`
and `python3 -I tests/fixtures/verify.py` (still `all 22 checks passed`).

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | `X` does not match `*` | 3, 4, 5, 7 |
| G2 | `?` matched by every pattern letter | 4 |
| G3 | No `*` added after the CDS's residues | 7, 8 |
| G4 | Stop codon numbered `Some(residues + 1)` | 7, 8 |
| G5 | Minus-strand CDS run from the first codon's `[2]` | 8 |
| G6 | Minus-frame run from the first residue's `middle - 1` | 3, 9 |
| G7 | `wraps_origin` always false | 5 |
| G8 | CDS hits in reading order (by first codon) | 5, 6 |
| G9 | Frame hits not sorted (frame by frame) | 3, 9 |
| G10 | Skip past a match instead of trying the next offset | 10 |
| G11 | No case folding | 2 |

## 8. Do not

- Do not touch `search.rs`, `frames.rs`, `frame.rs`, `translation.rs`,
  `mutation.rs` or `cds_map.rs`. Use `codon_positions` as it is.
- Do not add a `DomainError` variant or change any error's code or message.
- Do not cap, merge or de-duplicate hits.
- Do not read `sequence.topology()` in frame mode.

## 9. Report

Every guard experiment with its failure line, and any symbol in a target that rule 2 does not answer.
