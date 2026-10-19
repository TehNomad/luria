# GenBank fixtures

Three GenBank records drive `crates/core/tests/genbank_fixtures.rs`: a public
plasmid read by an independent parser, and two synthetic records written by
hand. The first two check luria's record reader (`genbank.rs`, 5c2) and CDS
import (`genbank_cds.rs`, 5c3) against D041. The third, the linear record,
is the template of the natural-language milestone (M6-13).

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

## The linear record

`synthetic-linear.gb` is a 393-base **linear** record written by hand on
2026-10-19 for the natural-language milestone (`docs/tasks/milestone-6-nl.md`
M6-13, contract 6i0). Every other GenBank fixture is circular, and luria
designs primers on linear templates only (D035, D041 G10), so this is the one
GenBank record on which a request can run from text to a designed primer
pair.

How it was written:
- **The protein first.** A synthetic 100-residue protein, `synA`, typed by
  hand in ten blocks of ten. It encodes no real gene. Residue 42 is Ala and
  residue 87 is Trp, so the product brief's own example, "A42G and W87A as
  separate variants", applies to it literally. Its `/translation` is that
  protein as typed.
- **Then a codon for each residue**, chosen by hand from common *E. coli*
  codons so that the sequence has no run of four identical bases (the primer
  quality limit is three, `MAX_SINGLE_BASE_RUN`), and the stop `TAA`.
  Residue 42's codon is `GCA` and residue 87's is `TGG`.
- **Then two flanks**, typed by hand: 42 bases 5′ of the CDS and 48 bases 3′
  of it, enough for a primer pair around residue 42 and residue 87 alike.
- The ORIGIN block was laid out (line numbers, groups of ten) by a throwaway
  script that joined the hand-typed pieces. It chose no base.

| Bases (1-based) | Half-open | What |
| --- | --- | --- |
| 1–42 | `[0, 42)` | 5′ flank |
| 43–345 | `[42, 345)` | CDS `synA`, plus strand: 100 codons and `TAA` |
| 346–393 | `[345, 393)` | 3′ flank |

Features, in order: `source` (index 0), a `gene` with `/gene="synA"`
(index 1), and the CDS (index 2), whose id is `synA` from `/gene` (D041 G2).
It states `/codon_start=1` and no `/transl_table`, and has a `/product`.

The expected values were derived by hand:
- **`synthetic-linear-record.tsv`**: LOCUS name, accession and version as
  written; 42 + 303 + 48 = 393 bases; `linear`; three features. The SHA-256
  of the upper-cased sequence was computed with coreutils, from the ORIGIN
  lines with digits and blanks removed (`tr -d ' 0-9\n' | tr a-z A-Z |
  sha256sum`), sharing no code with luria or Biopython.
- **`synthetic-linear-cds.tsv`**: `43..345` is `[42, 345)` (D006); 303
  bases are 101 codons, so 100 residues without the stop; the first codon is
  `ATG`; the `/translation` is the protein the codons were chosen for.
- **`synthetic-linear-residues.tsv`**: residue *k*'s codon is
  `[42 + 3(k − 1), 42 + 3k)`, so residue 1 is `[42, 45)`, 42 is
  `[165, 168)`, 87 is `[300, 303)` and 100 is `[339, 342)`. Each letter and
  codon is the one chosen above.

`reference/verify_synthetic_linear.py` re-derives every column of the three
tables with Biopython and `hashlib`, including the translation against
`/translation` and the absence of any internal stop. It writes nothing. It
was checked to fail on three deliberately wrong values, one per table: a
changed SHA-256 digit, residue 87 one codon early, and the minus strand.

Fit for purpose, not an expectation: on 2026-10-19 a hand-written request
for A42G and W87A as separate variants, with `"policy": "default"` codons,
`luriaNative`, `minAnnealingOverlap` 15, inline placement and the
`q5_reaction` buffer, ran through `luria q5 design` to `verified` with both
members designed and exit 0. No value from that run is recorded here.

## Rerunning

From the repository root, with the pinned environment
(`../requirements.txt`):

    python -I tests/fixtures/genbank/reference/pbr322.py --check
    python -I tests/fixtures/genbank/reference/verify_synthetic.py --check
    python -I tests/fixtures/genbank/reference/verify_synthetic_linear.py --check

All three run under `tests/fixtures/verify.py`. They are not part of the cargo
checks (D039, D026).
