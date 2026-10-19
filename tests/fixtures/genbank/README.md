# GenBank fixtures

Two GenBank records drive `crates/core/tests/genbank_fixtures.rs`: a public
plasmid read by an independent parser, and a synthetic record written by
hand. Together they check luria's record reader (`genbank.rs`, 5c2) and CDS
import (`genbank_cds.rs`, 5c3) against D041.

## pBR322, the public record

`raw/J01749.1.gb` is NCBI nuccore J01749.1, "Cloning vector pBR322, complete
sequence", 4361 bp, circular. It was fetched once on 2026-10-17 from

    https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=nuccore&id=J01749.1&rettype=gb&retmode=text

and saved unchanged (SHA-256
`7660b0fe95a91d492b102192dc72924508a69fda762988aa0923b3d5cc2073f6`). The
owner's first choice, pUC19 L09137.2, has no CDS features, so pBR322 was used
instead (`docs/tasks/milestone-5-cli.md` §4, R1).

`reference/pbr322.py` reads it with Biopython 1.88 and writes three tables:
- `pbr322-record.tsv`: LOCUS name, first accession, version, length,
  topology, the SHA-256 of the upper-cased sequence, and the feature count;
- `pbr322-features.tsv`: every feature's key and, for a CDS, its id under
  D041 G2 and the qualifier it came from;
- `pbr322-cds.tsv`: each CDS's interval (zero-based half-open), strand,
  `/codon_start`, `/transl_table`, first codon, residue count under the
  standard code, and whether that translation equals `/translation`.

The record has 50 features and three CDSs:
- `tet` is `86..1276` on the plus strand, so `[85, 1276)`, 396 residues;
- `bla` is `complement(3293..4153)`, so `[3292, 4153)` on the minus strand,
  286 residues;
- the ROP CDS has no `/locus_tag`, `/gene` or `/label`, so it can't be named.

All three state `/transl_table=11`. `tet` and `bla` start with `ATG`, and
their standard-code translations equal their `/translation`, so luria
imports them under D041 G5. ROP starts with `GTG`. These values were also
checked by hand on 2026-10-17 with a standard-library script and arithmetic:
the sequence length and SHA-256 from ORIGIN, the 50 feature key lines, the
three start codons, and residue counts of (1276 − 86 + 1) / 3 − 1 = 396 and
(4153 − 3293 + 1) / 3 − 1 = 286.

## The synthetic record

`synthetic.gb` is a 60-base circular record written by hand. Its sequence,
1-based, is:

| Bases | Text | Used by |
| --- | --- | --- |
| 1–12 | `ATGGCTTGGTAA` | `p1` (M A W stop) and most refusal cases |
| 13–24 | `CTACTGCTGCAT` | `m1`, on the minus strand: its reverse complement is `ATGCAGCAGTAG`, M Q Q stop |
| 25–36 | `GTGGCTTGGTAA` | `alt11`: a GTG start under `/transl_table=11` |
| 37–48 | `ATGTAAGCTTAA` | `stop`: a stop at codon 2 |
| 49–54 | `GGGCCC` | filler |
| 55–60 | `ATGAAA` | the origin joins |

It has 27 features, one per case:
- one per accepted location form: `a..b`, `complement(a..b)`, and the three
  spellings of an origin join (`pj`, `mj1`, `mj2`);
- the id precedence: `m1` has `/gene` before `/locus_tag` in the file, and
  its id still comes from `/locus_tag`;
- `/transl_table=11` with an ATG start (`atg11`, imported) and with a GTG
  start (`alt11`, refused); a first codon other than `ATG` is refused under
  every table, absent, 1 or 11 (D042);
- the refusals the record holds: `/pseudo` (`ps`), `/codon_start=2` (`cs2`),
  `/transl_table=4` (`tt4`) and a repeated `/codon_start` (`rep`); a CDS
  with no id, two CDSs sharing the id `dup`, an id with a blank (`bad id`),
  a length that is not a multiple of three (`len13`), an internal stop
  (`stop`); and the location refusals `<1..12` (`fuzzy`), `order(1..6,7..12)`
  (`ord`), a plain `join(1..6,7..12)` (`exons`), a bare position `5`
  (`single`) and `X00001.1:1..12` (`remote`);
- not in this record: `between_bases` (`^`), `position_zero`,
  `range_ends_before_start`, `past_sequence_end` and `syntax`, and the
  `/pseudogene` qualifier; the same code refuses them, but they are tested
  only in the unit tests of `genbank_location.rs` and `genbank_cds.rs`.
- a non-CDS `gene` feature with `/gene="p1"`, which must not count as a
  second `p1`.

The expected values were derived by hand from the table above and the
1-based to half-open rule (`a..b` is `[a - 1, b)`, D006):
- **`pj`**: `join(55..60,1..12)` is `[54, 60)` then `[0, 12)` (D036). It
  reads `ATGAAA` + `ATGGCTTGGTAA` = M K M A W stop.
- **`mj1`** and **`mj2`**: `complement(join(55..60,1..3))` and
  `join(complement(1..3),complement(55..60))` are both `[54, 60)` then
  `[0, 3)` on the minus strand. The plus-strand text is `ATGAAA` + `ATG`.
  Its reverse complement, `CATTTTCAT`, reads H F H. It starts `CAT`, so both
  are refused as `alternative_start` under D042, with detail `CAT`, at key
  lines 22 and 24; both still go through the location reader first, so both
  minus-strand spellings of an origin join are still read.
- **`badtr`**: states `MAV` where the CDS reads `MAW`, so the first
  difference is residue 3, V against W. **`shorttr`** states `MA`, so
  residue 3 is missing against W.
- **`len13`**: `1..13` is 13 bases. **`stop`**: `ATG TAA GCT TAA` has a
  stop at codon 2.
- Feature lines are counted in the file; the second `dup` is on line 57.

`synthetic-expected.tsv` records the outcome of importing each name on its
own, and `synthetic-unused.tsv` the not-used list when nothing is imported.
`reference/verify_synthetic.py` re-derives every biological column of both
with Biopython: feature order, ids, intervals, strands, translations, start
codons, qualifier values, location forms, the first translation difference,
the stop codon, and the length. It writes nothing. It was also checked to
fail on three deliberately wrong values: swapped origin-join intervals, a
wrong residue number, and an id given to the CDS that has none.

## Rerunning

From the repository root, with the pinned environment
(`../requirements.txt`):

    python -I tests/fixtures/genbank/reference/pbr322.py --check
    python -I tests/fixtures/genbank/reference/verify_synthetic.py --check

Both run under `tests/fixtures/verify.py`. They are not part of the cargo
checks (D039, D026).
