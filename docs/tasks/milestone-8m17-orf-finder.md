# Contract 8m17 — core: the ORF finder, with a reference table (M8-40)

Planner: Claude. Status: **Implemented 2026-10-28** by worker `w-8m17` in
one round, reviewed and accepted. Findings from review:

* The planner's own prototype (Biopython translation and the same regex,
  written before the contract) gives the same rows, in the same order, for
  all nine cases of §6 (45 rows). `orfs.py` uses §6's method, not a port of
  the scan.
* The scan keeps one queue of starts per frame phase over the reading (three
  copies on a circular record), drops the starts out of the `L` window at
  each stop and clears the queue: linear in `L`. It builds a `String` per
  codon and a `Vec<char>` per strand, so a 64 MiB record will be slow and
  large in memory, but not quadratic.
* Beyond §3: the worker also added a `view/` row and an "ORFs (8m17)"
  section to `tests/fixtures/README.md`, and flagged it. The text is
  accurate and kept.
* G1–G5 each failed the test §10 names (G1 also fails T1 and T2; G3 also T5).

Tier: full
(D051): new biology, so a table from an independent Python reference pins it.
Owner's answer: `docs/tasks/milestone-8m-polish.md` §1, M8-40. The lanes
that draw ORFs are 8m18; this contract adds no drawing.

## 1. What this contract builds

`crates/core/src/orf.rs`: one function that finds the open reading frames
of a sequence on both strands, as M8-40 defines them, and a reference script
and table that pin its output.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `lookup`, `CodonOutcome` (D010: an ambiguous codon is neither a stop nor a residue) | `crates/core/src/genetic_code.rs:181`, `:96` | start and stop codons |
| `Sequence::letters`, `length`, `complement_letter` | `crates/core/src/sequence.rs` | the bases |
| `Strand` | `crates/core/src/strand.rs:7` | the strand |
| D036 interval order (origin-crossing = two intervals, the one ending at the record's end first) | `docs/decisions.md` D036 | `Orf::intervals` |
| the fixture-test pattern (`VIEW`, loading records) | `crates/core/tests/frames_fixtures.rs:40` | the new test |
| a reference script's shape (`--print`, `--check`, md5 lines) | `tests/fixtures/view/reference/frame_tables.py` | the new script |

## 3. Files the worker may touch

1. `crates/core/src/orf.rs` (new) and one `pub mod orf;` line in
   `crates/core/src/lib.rs`.
2. `tests/fixtures/view/reference/orfs.py` (new) and the table it writes,
   `tests/fixtures/view/orfs.tsv` (never edited by hand).
3. `tests/fixtures/verify.py`: one `CHECKS` entry, `view/reference/orfs.py`.
4. `tests/fixtures/view/README.md`: a new section "ORFs (8m17)".
5. `crates/core/tests/orf_fixtures.rs` (new).

## 4. Public surface

```rust
/// M8-40's minimum: an ORF has at least this many residues, the stop not counted.
pub const ORF_MIN_RESIDUES: u64 = 75;

#[derive(Clone, Debug, PartialEq)]
pub struct Orf {
    pub strand: Strand,
    /// Zero-based, half-open, in D036 order: one interval, or two for an ORF
    /// across the origin (the one ending at the record's length first). The
    /// ORF's span includes its stop codon, as a CDS feature's does.
    pub intervals: Vec<(u64, u64)>,
    /// Codons from the ATG up to the stop, the ATG counted, the stop not.
    pub residues: u64,
}

pub fn find_orfs(sequence: &Sequence, circular: bool, min_residues: u64) -> Vec<Orf>;
```

## 5. Rules

Read each strand in its reading direction: the plus strand as written, the
minus strand as the reverse complement. Call that reading `R`, of length `L`.

1. **Codons.** A start is a codon `lookup` answers as `Residue(M)` (only
   `ATG`, either case). A stop is a codon `lookup` answers as `Stop`. Every
   other codon, ambiguous ones included (`TAR`, `NNN`), is neither: it does
   not end an ORF and does not start one (D010). [T1, T5]
2. **One ORF per stop.** For every stop codon in `R`, walk back from it one
   codon (3 bases) at a time. The walk ends at the previous stop codon in
   that frame (not included), at the start of `R` on a linear record, or —
   on a circular one, where the walk wraps — once the span from the codon
   reached to the stop's last base would exceed `L`. The ORF starts at the
   furthest-back start codon the walk met; with none, this stop has no ORF.
   [T1–T4]
3. **Length.** The ORF is kept when its `residues` is at least
   `min_residues`. [T2]
4. **Circular.** On a circular record a codon may wrap the origin, and an ORF
   may cross it (two intervals). On a linear record nothing wraps, and a
   reading that runs off either end without a stop is not an ORF. `circular`
   is the parameter, never `sequence.topology()`: the view's `T` can flip it.
   [T3, T4]
5. **Order.** Sorted by `intervals[0].0`, then plus before minus, then
   `intervals[0].1`.
6. **Speed.** One pass per strand over at most a few times `L` bases, with no
   walk back per stop: D056's 64 MiB record must not take quadratic time.
   Keep, per frame phase, what the scan needs (the earliest start since the
   last stop, and on a circular record the starts still inside the `L`
   window). [review]

## 6. The reference (`orfs.py`) and its table

The script must not share the Rust algorithm. Use this method:
translate each frame with Biopython (`Seq.translate()`, standard table) and
find ORFs with the regular expression `M[^*]*\*`, which takes the first `M`
after each stop. On a circular record translate `R` three times over
(`R + R + R`), keep the matches whose stop codon starts in the middle copy,
and where a match is longer than `L` bases take the first `M` inside the
last `L` bases before the stop's end instead (none: no ORF). Assert every
record in the table holds only `ACGT` (Biopython reads `TAR` as a stop; D010
does not, so ambiguous bases are tested in Rust only, T5).

Columns: `record`, `topology`, `min_residues`, `strand` (`+`/`-`),
`location` (GenBank text, 1-based: `86..1276`, `complement(3293..4153)`,
`join(55..60,1..12)`, `complement(join(53..62,1..8))`), `residues`. Cases,
with the planner's expected row counts:

| record | file | topology | min | rows |
| --- | --- | --- | --- | --- |
| `pbr322` | `genbank/raw/J01749.1.gb` | circular | 75 | 9 |
| `pbr322` | | circular | 81 | 8 |
| `pbr322` | | linear | 75 | 9 |
| `synthetic` | `genbank/synthetic.gb` | circular | 2 | 4 |
| `synthetic` | | linear | 2 | 4 |
| `synthetic_linear` | `genbank/synthetic-linear.gb` | linear | 10 | 4 |
| `synthetic_linear` | | circular | 10 | 5 |
| `frames62` | `view/frames-62.gb` | circular | 3 | 2 |
| `frames62` | | linear | 3 | 0 |

A case with no rows still appears in the script's `--print` summary.

## 7. Hand values (the planner's, from the files)

- `pbr322` circular, 75: `86..1276` 396 (`tet`), `complement(3293..4153)`
  286 (`bla`), and seven more, among them `complement(323..565)` 80 — which
  is why min 81 leaves 8.
- `synthetic` circular, 2: `+ join(55..60,1..12)` 5 (the `pj` feature). Read
  linear, the same stop gives `+ 1..12` 3 instead: the ATG at 55 is reached
  only across the origin (rule 2).
- `synthetic_linear` circular, 10: `+ join(284..393,1..34)` 47 joins the
  linear four.
- `frames62` circular, 3: `+ join(56..62,1..11)` 5 and
  `- complement(join(53..62,1..8))` 5. A minus-strand reading
  `MITCSAYSPKGPFNGLCALN*` would span 63 bases of the 62, so it is no ORF
  (rule 2's bound).

## 8. Tests (`crates/core/tests/orf_fixtures.rs`)

| # | Test | Asserts |
| --- | --- | --- |
| T1 | `every_case_matches_the_table` | for each case, `find_orfs` rendered to the table's columns equals the table's rows, in order |
| T2 | `the_minimum_is_inclusive` | `pbr322` circular: `complement(323..565)` (80) is kept at 80 and dropped at 81 |
| T3 | `the_longest_orf_per_stop_crosses_the_origin` | §7's `synthetic` pair |
| T4 | `no_orf_is_longer_than_the_record` | §7's `frames62` values, and no ORF anywhere in the table spans more than its record |
| T5 | `an_ambiguous_codon_is_not_a_stop` | a linear sequence written in the test, `ATGAAATARAAATAA`, min 1: one ORF, `+ 1..15`, 4 residues (`TAR` does not stop it). `ATGNNNTAA`: one ORF, 2 residues. `NTGAAATAA`: none |

## 9. Checks

`scripts/check`; `python3 -I tests/fixtures/view/reference/orfs.py --check`
after generating; `python3 -I tests/fixtures/verify.py` (now 26 checks).

## 10. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | rule 2 takes the nearest start, not the furthest | T1, T3 |
| G2 | the circular bound removed (span may exceed `L`) | T4 |
| G3 | `>` instead of `>=` for the minimum | T2 |
| G4 | an ambiguous codon counted as a stop when every reading of it is one | T5 |
| G5 | the reference takes the last `M` before each stop (change, `--check`, restore) | `--check` |

## 11. Do not

- No new dependency. No change to any other file in `crates/`, any other
  fixture, or any existing test.
- Don't write a table row by hand or from luria's output.
- Don't edit this contract. If a rule here is ambiguous, or the reference
  disagrees with §6's counts or §7's values, stop and report it.

## 12. Report

The table's row counts per case and the `--print` summary, every guard with
its failure line and md5 lines before and after, the check results, and any
choice the rules left open.
