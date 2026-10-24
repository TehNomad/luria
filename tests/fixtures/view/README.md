# Milestone 8 view fixtures

These files pin what the sequence view and `luria find` must print before any
of it is implemented (contract 8c of `docs/tasks/milestone-8-view.md`). They
are inputs to Rust tests that do not exist yet; nothing under `crates/` reads
them today.

Nothing here was produced by running luria. Every value comes from Biopython,
from the hand-derived tables in `../genbank/README.md`, or from arithmetic
written out in `reference/`, and each is re-derivable with `--check` (D039).

## Files

| File | What it pins |
| --- | --- |
| `frames-62.gb` | a 62 bp circular record: length not a multiple of 3, a plus-strand CDS and a minus-strand CDS that both cross the origin and overlap each other |
| `cds-map-codons.tsv` | every codon of every small fixture CDS, in reading order, with the same codon's bases in ascending reference order |
| `cds-map-bases.tsv` | every base of every small fixture CDS: which codon it belongs to, its place inside that codon, and the residue it produces |
| `cds-map-counts.tsv` | per CDS: length, codon count, residue count, whether the last codon is a stop, and the reference positions of the first codon, the last codon and the last residue's codon |
| `cds-map-boundaries.tsv` | the first three and last three codons of `tet` and `bla` in `../genbank/raw/J01749.1.gb` |
| `cds-map-refused.tsv` | the features the view must refuse to number, and the rule that refuses each |
| `frames-61.fa` | a 61 bp FASTA with no features at all: length 1 mod 3, and a record with no CDS for the picker and for frame mode |
| `frame-tables.tsv` | every complete codon of every frame on both strands of five records, with that frame's own residue number, the translated letter, and the codon's bases in ascending reference order |
| `frame-tails.tsv` | per record, strand and frame: how many bases the frame covers, how many complete codons that is, and the 1 or 2 leftover reference positions view mode must not draw |
| `reference/make_frames_62.py` | the deterministic search that chose `frames-62.gb`, and the check that the tracked record still satisfies what it claims |
| `reference/cds_map.py` | the five CDS-map tables above |
| `reference/frame_tables.py` | the two frame tables above, and the check that every frame's whole letter run equals Biopython's translation of the same linear slice |

No fixture here pins a decision that was still open when it was written.
`M8-2` — six frames do not run across the origin, D052 — is settled, and the
frame tables below are what pins it; `M8-18` (D050) says what happens to the
leftover bases of a frame.

## How a wrapped CDS reads, and why that is not an open question here

D036 accepts a `join(...)` whose parts abut across the origin and stores a
CDS's intervals in reference order, not reading order. The reading itself is
pinned by `../genbank/README.md`, which derives it by hand:

> **`mj1`** and **`mj2`**: … The plus-strand text is `ATGAAA` + `ATG`. Its
> reverse complement, `CATTTTCAT`, reads H F H. It starts `CAT` …

So the CDS's text is the joined arc along the molecule — `55..60` then `1..3`,
because base 60 and base 1 are neighbours on a circle — and a minus-strand CDS
takes the reverse complement of that whole arc. A minus-strand CDS that crosses
the origin therefore begins at the **complement of the arc's lowest-numbered
base**, not at the top of the text as written. `reference/cds_map.py` encodes
that as: join the parts into one continuous arc through the origin, then read
it forward (plus) or backward (minus).

That rule is cross-checked, not asserted. For every local, exactly-positioned
CDS in these fixtures, `cds_map.py` compares its own codon list with
Biopython's `location.extract()` of the same feature, and the comparison covers
origin-crossing CDSs on **both** strands — `pj`, `fP` and `fM`. Biopython agrees
with the walk for all of them, including the wrapped minus-strand `fM`, whose
first codon is the complement of bases 8, 7 and 6. If the walk and Biopython
ever disagree, the script raises rather than writing a table.

`mj1` and `mj2` stay refused as `alternative_start`, which is why
`frames-62.gb` exists: without it the fixtures would hold no usable
minus-strand CDS crossing the origin, and `base_residue`'s hardest case would
be unpinned.

## `frames-62.gb`

62 bases, circular, so `62 % 3 == 2`: a six-frame run that starts at base 1
ends with two bases no codon covers (M8-18, D050 — those two bases get no
letter, and the `?` marker is reserved for a complete unreadable codon).

- `fP` at `join(56..62,1..11)`: 18 bases, 6 codons, plus strand, crossing the
  origin. Codon 3 sits on bases 62, 1 and 2, so its bases in ascending
  reference order are `1,2,62` — a codon whose reference positions are not
  contiguous.
- `fM` at `complement(join(53..62,1..8))`: 18 bases, 6 codons, minus strand,
  crossing the origin. Its first codon is the complement of bases 8, 7, 6 and
  is `ATG`; on the same three bases `fP` reads codon 5, `CAT` (His). One base
  triple, two CDSs, two readings, neither an internal stop.
- The two CDSs overlap at 53–62 and 1–8, which is the small-file form of the
  M8-16 case: D050 reports every hit, it does not collapse overlaps.

The sequence was found by a seeded search, not chosen by hand: seed
`20261024`, bases that no start or stop codon pins filled from
`random.Random(20261024)` in ascending position order, first candidate that
makes both CDSs importable wins. The constraints — ATG start, no internal
stop, terminal stop, length divisible by three, `/translation` equal to the
residues without the terminal stop — are D042's rules as recorded in
`../genbank/synthetic-expected.tsv`, so the record is a product of decisions
already made, not a new biological assumption. `--print` shows the winner and
both codon lists; `--check` re-reads the tracked record with Biopython and
asserts every one of those constraints, including that Biopython's extraction
equals the walk's codons.

## The tables

Column names say which order a value is in, because D050 makes the difference
explicit: `residue_bases` returns reading order, while the interval columns
D040 records stay in reference order.

`cds-map-codons.tsv` — one row per codon, listed in reading order:

- `codon_reading` counts codons from the first translated codon of the CDS.
- `residue` is the number `luria` will print. It runs 1..`residues` and the
  final codon says `stop`, because D010 says the terminal stop codon is not an
  amino-acid residue. `cds-map-counts.tsv` records `terminal_stop` per CDS; a
  CDS whose last codon is not a stop numbers every codon, and `translate()`
  in `crates/core/src/translation.rs` already tests that a complete CDS needs
  no stop at all (`a_complete_cds_needs_no_start_or_stop_codon`).
- `bases_reading` is the codon in the direction translation happens.
- `bases_ref_ascending` is the same bases sorted ascending. For every codon of
  a minus-strand CDS, and for `fP`'s codon 3, the two columns differ; that is
  the distinction the table exists to pin.

`cds-map-bases.tsv` — one row per base of the CDS, in reading order: `position`
is the reference position, `codon_reading` and `place_reading` say which codon
and which of its three places (1..3 as read), and `residue` repeats the codon's
residue, `stop` for the terminal stop codon. `base_residue`'s whole answer is
this table; `residue_bases` inverts it. Together they pin the round trip for
every base of every fixture CDS.

`cds-map-boundaries.tsv` uses the same columns as `cds-map-codons.tsv` for
`tet` and `bla`. They are 397 and 287 codons long, so only the first three and
last three codons are listed; `tests/fixtures/genbank/README.md` and
`genbank/pbr322-cds.tsv` already pin their totals (396 and 286 residues), and
`tests/codon-fixtures.rs` records pBR322 codons 2..4 through a separate route,
which is an independent check of the same codons.

`cds-map-refused.tsv` lists the features the view must not number — `ROP` in
pBR322 (no id), `len13` (13 bases, not divisible by three), `mj1` and `mj2`
(`alternative_start`), and the location forms D036 refuses — with the rule as
`luria`'s own code names it. D049 draws a partial feature in the view without
numbering it, so a refused feature still gets a bar; what it must not get is a
residue scale.

## Six frames, and the bases a frame cannot use (8a1b)

`frame-tables.tsv` and `frame-tails.tsv` come from the rules in
`docs/tasks/milestone-8-view.md` §4.3, D052 and D050:

* Frames are numbered **1, 2, 3 on either strand**. On the plus strand frame *f*
  starts at base *f* (1-based) and reads rightwards; on the minus strand frame
  *f* starts at **the last base minus (*f* − 1)** and reads leftwards. The
  tables keep `strand` and `frame` as separate columns, so nothing has to
  remember a 4-6 numbering that no part of the plan uses.
* A frame **never runs across the origin**, on a circular record exactly as on a
  linear one. `frames62 + frame 1` has 20 complete codons and stops at base 60;
  bases 61 and 62 are recorded in `frame-tails.tsv` as leftover and never
  translated. An implementation that rotated the sequence first — the habit
  six-frame callers have when they want every rotation to get the same codon
  set — would produce a 21st codon from 61, 62 and 1, and would fail this table.
* A frame's own residue numbering runs from 1 inside that frame and says nothing
  about a CDS residue. The CDS scale lives in `cds-map-bases.tsv`; the two
  columns look alike and are not the same thing (D050).
* `?` in the `aa` column would mean a **complete** codon that cannot be read.
  It appears nowhere: no fixture in this repository holds a base outside ACGT,
  so an ambiguity case arrives with the record 8a1c adds.

`frames-61.fa` is `frames-62.gb`'s sequence without its last base — that is the
whole relationship, and it is why the file exists: 62 is 2 mod 3 and 61 is 1 mod
3, so between them the tables cover all three tail cases (the other records are
60, 393 and 4361, and 4361 is 2 mod 3). Its canonical text — upper case, no line
breaks — hashes to
`60a3b3f88a4a09816494160a8b36694dcba03f23c76fa46a370b3db563b1143c`, computed
with the same pipeline `../fasta/README.md` documents:

```
grep -v '^>' tests/fixtures/view/frames-61.fa | tr -d '\n\r \t' | tr a-z A-Z | sha256sum
```

`reference/frame_tables.py` asserts that digest itself before it writes a row, so
the sentence above and the file cannot drift apart. Proved both ways: changing
one letter of `frames-61.fa` makes `verify.py` fail with
`canonical text hashes to …, README pins …`, and changing one codon in
`frame-tables.tsv` makes it fail with a byte-count difference.

**The `topology` column reports what the file says, not what the view assumes.**
A GenBank LOCUS line states `circular` or `linear`, and Biopython puts it in
`annotations["topology"]`; a FASTA record says nothing, and its rows read
`unknown`. D049 makes the view assume linear for a FASTA **and show that
assumption**, which is a display decision a fixture must not silently make for
it.

Two spot values were checked without going through the script at all:

* `frames62 − frame 1`, codon 1: the record's last three bases are `G C A`
  (positions 60, 61, 62), so the frame's first codon reads `TGC` = Cys — the
  complement of each reference base in decreasing position order, the same rule
  `../genbank/README.md` derives by hand for `mj1`/`mj2`. Taking the reverse
  complement of the bases *written in reading order* instead of of the ascending
  span gives `CGT`, and this table would be wrong in every minus-strand row; an
  earlier reference script had exactly that bug.
* `pbr322 + frame 1`, codons 1-3: the ORIGIN block of
  `../genbank/raw/J01749.1.gb` reads `ttctcatgtt` on its first line, so the
  codons are `TTC TCA TGT` = F S C, read off the file by eye.

For `pbr322` (4361 bases) the table stores a **sample** — the first three codons
of each frame, the last three, and every 100th — marked `sampled = yes`;
records of 500 bases or fewer store every codon and read `sampled = no`. The
script still compares **every** codon of **every** frame against Biopython's
translation of the same linear slice before it writes anything, so the sample is
about how much data the repository carries, not about how much is verified. A
frame implementation that was off by one, or that rotated a circular record
first, breaks the sample somewhere in a 4361-base record as surely as a full
table would.

```
python -I tests/fixtures/view/reference/frame_tables.py --print
python -I tests/fixtures/view/reference/frame_tables.py --check
```

## Regenerating

From the repository root, with the pinned environment
(`tests/fixtures/requirements.txt`):

```
python -I tests/fixtures/view/reference/make_frames_62.py --print
python -I tests/fixtures/view/reference/make_frames_62.py --check
python -I tests/fixtures/view/reference/cds_map.py
python -I tests/fixtures/view/reference/cds_map.py --check
python -I tests/fixtures/verify.py
```

`cds_map.py` without `--check` writes the five tables; with `--check` it
regenerates them in memory and compares byte for byte (D039). It refuses to
write anything when its own cross-checks fail: a CDS listed as usable that
does not read as one, a codon list that differs from Biopython's extraction,
residue numbering that is not 1..`residues` plus `stop`, base rows that do not
cover a CDS's bases exactly once, or `place_reading` values that are not
1,2,3 in reading order.
