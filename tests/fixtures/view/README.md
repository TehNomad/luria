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
| `frames-amb.fa` | a 33 bp FASTA holding IUPAC ambiguity codes: the only record here that exercises the `?` column of `frame-tables.tsv` |
| `frame-tables.tsv` | every complete codon of every frame on both strands of six records, with that frame's own residue number, the translated letter, and the codon's bases in ascending reference order |
| `frame-tails.tsv` | per record, strand and frame: how many bases the frame covers, how many complete codons that is, and the 1 or 2 leftover reference positions view mode must not draw |
| `search-letters.tsv` | all 225 pairs of IUPAC letters, pattern against template: whether luria's containment rule matches, whether the intersection rule would, and where the two part company |
| `search-dna-patterns.tsv` | every DNA-search pattern the tables pin, per record: as written, after folding, its reverse complement, whether the record was searched as wrapping, and what the row is there to pin |
| `search-dna-hits.tsv` | every hit of every listed pattern: strand, zero-based start, length, whether it wraps the origin, and the bases it covers |
| `search-protein-patterns.tsv` | every protein-search pattern the tables pin, per record and target (`frames` or a CDS id): as written, after folding, its length, its hit count, and what the row is there to pin |
| `search-protein-hits.tsv` | every hit of every listed protein pattern: strand, frame, zero-based start and length of the bases its codons cover, whether that run wraps, first and last residue numbers (`stop` for the stop codon), the symbols matched, and the bases |
| `layout-blocks.tsv` | every block of sixteen layout cases (record, width, mode): the line width that width allows, and each block's first and last base and base count |
| `layout-cells.tsv` | for chosen blocks, every item the layout places: line, role, kind, first column, columns covered, and text — columns only, no glyph or style |
| `reference/make_frames_62.py` | the deterministic search that chose `frames-62.gb`, and the check that the tracked record still satisfies what it claims |
| `reference/cds_map.py` | the five CDS-map tables above |
| `reference/frame_tables.py` | the two frame tables above, and the check that every ACGT codon of every frame equals Biopython's translation of the same linear slice |
| `reference/search_dna.py` | the three search tables above, and the cross-checks described in the DNA-search section |
| `reference/search_protein.py` | the two protein-search tables above, and the cross-checks described in the protein-search section |
| `reference/layout_columns.py` | the two layout tables above, and the cross-checks described in the block-layout section |

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
* `?` in the `aa` column means a **complete** codon that cannot be read, and
  nothing else. A codon made only of A, C, G and T gets a letter, or `*` for a
  stop; a codon holding any IUPAC ambiguity code gets `?`, including `N` (D052,
  D050, D010). That is what core does rather than what a translation library
  tends to do: `genetic_code::lookup` answers `CodonOutcome::Ambiguous` for any
  codon with a base outside `DNA_BASES`, and `translate` turns that into a
  refusal instead of a guess.

`frames-amb.fa` exists to pin that rule. Its plus-strand frame 1 is eleven
hand-designed codons — `ATG TAT TAA TAG TAR AAN GGN CTN NNN GTG TSK` — and the
row that follows each is exactly what a hand reading gives:

```
M  Y  *  *  ?  ?  ?  ?  ?  V  ?
```

`GGN` and `CTN` are the discriminating rows: every expansion of them is the same
amino acid (Gly, Leu), and D010 says there is no guess to make, so they read `?`.
A script that collapsed them would write `G` and `L` here, and `TSK` and `TAR`
cover the case where the expansions disagree. The record's minus-strand frames
re-run the same reading rule on complemented ambiguity codes: plus `TSK` at bases
31-33 becomes minus-frame-1 codon 1 `MSA`, and plus `TAR` at 13-15 becomes
`YTA`, because the frame reads bases 15, 14, 13 and complements each. Length 33
adds nothing to the modulo-3 coverage — it is 0 mod 3 like `synthetic`.

**The Biopython comparison excludes codons with an ambiguity code, by rule, and
the disagreement is recorded rather than hidden.** `--print` lists what Biopython
answers where luria prints `?`:

```
TAR: Biopython '*', luria '?'
GGN: Biopython 'G', luria '?'
CTN: Biopython 'L', luria '?'
TTY: Biopython 'F', luria '?'
NNN: Biopython 'X', luria '?'
```

Biopython calls `TAR` a stop and collapses `GGN`, `CTN`, `TTY`, `ACN`, `CCN`,
`GTS` and `YTA` to one letter; luria refuses. Comparing those codons would have
the fixture follow the library rather than D010. Every codon of an ACGT-only
record is still compared codon by codon, and the script asserts that an
ACGT-only slice compares *all* of its codons, so narrowing the check cannot hide
a wrong frame mapping.

`frames-61.fa` is `frames-62.gb`'s sequence without its last base — that is the
whole relationship, and it is why the file exists: 62 is 2 mod 3 and 61 is 1 mod
3, so between them the tables cover all three tail cases (the other records are
60, 393, 33 and 4361, and 4361 is 2 mod 3). Its canonical text — upper case, no line
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

`frames-amb.fa`'s canonical text hashes to
`dad6739778314aed63617363569469465d36df95353a236de7ca535254370469` under the
same pipeline, and `reference/frame_tables.py` asserts that too.

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
* `framesamb − frame 1`, codon 11: the record's first three bases are `A T G`,
  so the frame's last codon reads the complement of each in decreasing position
  order — `C` from base 3, `A` from base 2, `T` from base 1 — which is `CAT` =
  His. Codon 1 of the same frame is `MSA`, the complement-image of the plus
  strand's `TSK`, and it reads `?`.

For `pbr322` (4361 bases) the table stores a **sample** — the first three codons
of each frame, the last three, and every 100th — marked `sampled = yes`;
records of 500 bases or fewer store every codon and read `sampled = no`. The
script still compares **every** codon of **every** frame against Biopython's
translation of the same linear slice — codon by codon, over ACGT codons, for the
reason given above — before it writes anything, so the sample is about how much
data the repository carries, not about how much is verified. A
frame implementation that was off by one, or that rotated a circular record
first, breaks the sample somewhere in a 4361-base record as surely as a full
table would.

```
python -I tests/fixtures/view/reference/frame_tables.py --print
python -I tests/fixtures/view/reference/frame_tables.py --check
```

## DNA search: which letters a pattern may match, and what a hit covers (8a1c)

`search-letters.tsv`, `search-dna-patterns.tsv` and `search-dna-hits.tsv` come
from §4.4 of the plan and from D052 (M8-7, M8-8, M8-16) and D050. They cover the
DNA half of §4.4 only; the protein rows of §4.5 wait for the types contract 8f
fixes, because a fixture written against a signature that changes gets rewritten
(§7).

* **Containment, not intersection** (M8-7). A record letter matches a pattern
  letter only when every base the record letter stands for is in the pattern
  letter's set. `search-letters.tsv` pins the whole 15 × 15 grid: a template
  `A` is covered by exactly `A R W M D H V N`, a template `R` by exactly
  `R D V N`, and a template `N` by `N` alone. 110 of the 225 pairs disagree with
  an intersection rule, and every one of those is intersection-yes,
  luria-no — containment can only refuse what intersection would accept.
* **One hit per place** (M8-8). pBR322's four landmark sites appear once each,
  strand `both`: zero-based 4358 (`GAATTC`), 374 (`GGATCC`), 28 (`AAGCTT`), 2294
  (`CATATG`) — D052's 1-based 4359, 375, 29, 2295. The EcoRI row reads
  `wraps_origin = yes`, because the site is `GAA` at 4359-4361 plus `TTC` at
  1-3; the pattern `GAATTT` does not match there, and `AATT` gives a hit at
  zero-based 4359, also wrapping — the same site, one base in.
* **Every position, no cap** (M8-16). On `framesamb` pattern `N` gives 33 hits,
  one per base, each `both` and length 1. `S` gives 8, every one `both`; `R`
  gives 25, none of them `both`, because no IUPAC letter sits inside both
  `{A,G}` and `{C,T}`. A pattern that is its own reverse complement — `RATY`,
  not `RYAT`, whose reverse complement is `ATRY` — is `both` at every hit,
  including spans like `AATC` where a concrete base sits inside a pattern `Y`.
* **Wrapping only where the record says circular.** Several patterns are cut
  from a record's own text, so the hit follows from the rules rather than from
  arithmetic: a substring matches on the top strand at its own start, the
  reverse complement of one matches at the same coordinates on the minus strand,
  and a pattern made of the last *k* bases followed by the first *k* matches only
  if the record wraps, at zero-based start `len - k`. On the two linear records
  that same pattern gives zero hits, and the script refuses to accept that claim
  unless the pattern really would match at that start — a zero-hit assertion
  about a pattern that never occurs would pin nothing.
* **Case and `U` are folded before matching** (D052). `gaattc` and `GAATUC` each
  give one hit at the same start as `GAATTC`, and their rows carry the record's
  bases, `GAATTC`. A pattern may hold `U` where a stored sequence may not: core's
  alphabet rejects `U` as RNA, so this is a rule about patterns only.

`framesamb` is where the two matching rules part company, and the discriminating
rows are pinned by what must be absent as well as what is present. `AAN` hits at
1-based 8 and 16, plus a minus-strand hit at 5; `AAY` hits at 8 but **not** at 16,
which is the place an intersection rule would match; `AAR` finds nothing at all,
because the only `AA` run that could serve is followed by `N`. `GTGTSK` hits at
28, `GTGTNN` hits at the same place, and `GTGTCK` and `GTGTKK` find nothing.

**Biopython is an oracle for some of this and not for the matching rule, and the
limit is measured.** `reference/search_dna.py` uses it for three things: the
letter-to-bases table (identical to `bases_of` in `crates/core/src/sequence.rs`),
the complement table (identical to core's, and asserted), and a **site count**
for the four landmark patterns on records that are all A, C, G and T, where
`len(hits)` must equal `len(enzyme.search(Seq(text), linear=...))` —
circular-aware exactly where the tables allow wrapping, and every enzyme sharing
a site must answer the same. That comparison is sound only for patterns that are
their own reverse complement, because luria merges a place that matches on both
strands into one `both` hit.

It is **not** an oracle for a record's own ambiguity codes. `Bio.Restriction`
compiles a regex out of the *site* and matches the record literally: `BstEII`
(`GGTNACC`) matches `GGTXACC`, where `X` is not an IUPAC letter at all, because
the site's `N` became a regex `.`; `Dpi3090II` (`AAGRAG`) refuses a record `R`
at its own `R`, where luria's containment rule accepts it and so would an
intersection rule. So its answer is neither subset-based nor intersection-based,
and the `intersection_match` column of `search-letters.tsv` says what the
rejected rule would answer, computed from Biopython's letter sets — not what
Biopython answers. That is why §4.4's rule is hand-coded here, as §6 of the plan
requires; but the plan's M8-7 note attributes the opposite behaviour to Biopython
and that attribution is not supported, which is recorded rather than argued away.

**The cross-checks are two, with separate ranges.** For a pattern of concrete
bases only, occurrences are enumerated with Python's own substring search over
the record plus its first `width - 1` bases when the record wraps — code shared
with nothing else in the file, which pins the legal start range, the wrapping
flag and the report-every-overlap rule. For a pattern holding ambiguity codes,
every candidate start whose window is all concrete is re-matched against Biopython's
letter classes; starts whose window holds an ambiguity code are outside that
range, counted, and asserted non-zero (200 such starts), so an oracle that
judged nothing could not pass as a check.

Two values were checked without the script. The `frames62` pattern
`ATGCGCACAGACCATTAA` is that record's `fP` join — bases 56-62 then 1-11, 18
letters — and read as codons it is `ATG CGC ACA GAC CAT TAA` = M R T D H stop,
which is the file's own `/translation="MRTDH"`; so the pattern in the table is
the annotated span, not something near it. And `frames-amb.fa`'s canonical digest
is pinned by this script too, the same digest `reference/frame_tables.py` and the
README pin, so a single base change anywhere in the record is caught by two
scripts that read it for different reasons.

Twelve controls, each run once and then restored: intersection instead of
containment fails at `framesamb + AAG: this script [15, 16, 23, 24, 25],
substring search []`; dropping the minus strand fails at a constructed `TTAAGCTT`
hit; searching a circular record as linear fails at a whole-record rotation that
only exists by wrapping; letting a linear record wrap fails at the same rule from
the other side; removing the `U` fold fails naming the letter `U`; not merging
the two strands fails at pattern `N`; stepping past overlaps instead of reporting
them fails at the substring oracle; ordering hits descending fails with
`hits must be ordered by start: ('synthetic', 'AAAT')`. Flipping one
`rules_agree`, moving the EcoRI start by one, and shortening one `purpose` cell
are each reported by `--check` with the line number and both rows. Changing base
24 of `frames-amb.fa` — outside every region this script pins by hand — fails
with the digest message.

`reference/search_dna.py` was registered in `../verify.py` only after contracts
8d, 8e and 8f had landed, because their runs were told the suite has 22
registered checks. It and `search_protein.py` were added together, taking the
count to 24.

What these tables do not pin: the cap on displayed hits (M8-12 is presentation,
§5.4 and §5.5), the refusal of an empty pattern or an illegal pattern letter
(they go through `normalize()` here, and belong in contract 8e's unit tests),
wording of any output, and anything from §4.5.

## Protein search: what `X` matches, and the order hits come in (8a1c)

`search-protein-patterns.tsv` and `search-protein-hits.tsv` are the §4.5 half of
8a1c, written once contract 8f fixed `ProteinTarget` and `ProteinHit`. They
come from §4.5 of the plan, D050, D052 and D054, on three records: `synthetic`
(its usable CDSs `p1`, `m1` and `pj`, and its frames), `pbr322` (`tet`, `bla`,
frames) and `framesamb` (frames only, read as linear). `frame` is `1`, `2`, `3`,
or `-` for a CDS hit; `first` and `last` are residue numbers, `stop` for a CDS's
stop codon, and in frame mode count codons within the frame (D050). Hits run by
zero-based `start`, plus strand first on a tie, in both modes (D054).

* **`X` matches the stop** (D054). On `synthetic` frames `X` gives 116 hits, one
  per complete codon of the six frames, stops included; `pj`'s `X` gives six,
  at starts 0, 3, 6, 9, 54, 57, and the one at 9 is the stop, `first` and
  `last` both `stop`. `MKMAWX` finds all of `pj`, `last` = `stop`, the run at
  54 for 18 bases and wrapping.
* **`?` is matched only by `X`.** On `framesamb` frame +1 reads `M Y * * ?`,
  where the `?` is the codon `TAR`: every reading of it is a stop, and D052
  still reads it `?`. So `**X` finds it and `***` finds nothing. `GGN` reads `?`
  though every reading is `G`, and `G` finds nothing on that record at all.
* **Base order, not reading order, in a CDS** (D054). `pj`'s `M` lists residue 3
  (base 0) before residue 1 (base 54); `m1`'s `Q` lists residue 3 before 2;
  `bla`'s `M` gives ten hits and residue 1, at 4150, is the last of them.
* **Frames do not wrap** (D052). `MKMA` is frame +1 of `synthetic` read across
  its origin, its last two residues then its first two; the script asserts it
  would match a wrapped +1, and the table pins zero hits.
* **One peptide, two modes.** `tet`'s and `bla`'s first six residues, searched
  in six frames, each give the hit their CDS gives: same start, same strand,
  in frame +2 and −2, numbered there by frame (70–75 for `bla`).
* **Length.** A pattern of 11 `X` matches `framesamb`'s two 11-codon frames,
  +1 and −1, once each; 12 finds nothing, nor does `MKMAW*M` on `pj` or `MAWXX`
  on `p1`. Case is folded: `mkm` and `y` give the hits `MKM` and `Y` give.

**How the values were checked.** The codon table is hand-coded and compared
with Biopython's table 11 for all 64 codons. Frames are rebuilt in the script
and compared with every `frame-tables.tsv` row for the three records (298
rows; pBR322's frame rows are sampled). A CDS's codons come from Biopython's
reading of its location; its residues must equal the file's own
`/translation`, its bases `feature.extract`, and for `synthetic` every codon
matches `cds-map-codons.tsv` (14 rows). Each hit is found twice, by the
script's matcher and by a regular expression over the same symbols (`X` as
`.`), and the offsets must agree. Each hit's `top_strand_bases` is translated
again on its own, reverse-complemented for `-`, and must give back `residues`.
Hand-derived values in contract 8f's tests 3–11 agree with these rows where
they overlap.

Eight controls, each run once against an unchanged copy of the script: `X`
not matching `*` fails at the regex cross-check on `synthetic frames X`; `?`
matched by every letter fails at `framesamb frames **X`; CDS hits in reading
order fail the `pj` `M` construction; a minus run taken from the first codon
fails because the bases read `PS` where the hit says `SH`; reading ambiguous
codons as Biopython does fails at `frame-tables.tsv` row `framesamb + 1 5`
(`TAR`); frames read across the origin fail at `synthetic - 1 1`; a numbered
stop codon fails the `MQQ*` construction; and ties ordered minus first fail
with `hits must run by start, plus first on a tie`.

What these tables do not pin: refusals of an empty pattern or an illegal
letter (contract 8f's unit tests), the cap on displayed hits (M8-12,
presentation), and wording. `reference/search_protein.py` is registered in
`../verify.py`, as `search_dna.py` is.

## Block layout: where every base, residue and number goes (8a2)

`layout-blocks.tsv` and `layout-cells.tsv` are the column tables of plan §6 for
§5.1's layout, under D050 (M8-19) and D055. They pin columns, not screens: no
border, box-drawing glyph, colour or style appears, so a glyph change in the
view never rewrites them. `width` is the number of columns the layout is given
for its lines; a `column` counts from 0 at the left edge of that width.

* **Gutter and line width.** `d = max(2, digits of the record length)`: a line's
  label (its first base's number, or a frame label) is right-aligned in columns
  `[0, d)`, column `d` is blank, bases start at `d + 1`, and the top strand's
  right number starts one blank column after the last base. The line width is
  the largest multiple of 10, or of 30 in any translation mode, with
  `2d + 2 + N <= width`: pBR322 (`d = 4`) gets 60 at width 78, 30 at width 40
  and 90 at width 118.
* **A shortened line, not a shifted grid** (D055). With `tet` selected on
  30-base lines the line from 61 ends at 88 and the next starts at `tet`'s codon
  at 89; on 60-base lines the line from 61 ends at 118. `frames-62.gb`'s two
  origin-crossing CDSs both put their block edge at 58, not 60, because
  59-61 is one codon of each; in six-frame mode the same record's edge stays at
  60, since without a CDS no line is shortened.
* **Residues over middle bases** (D055). `tet`'s `M` sits over base 87, the
  middle of 86-88. `pj` on synthetic gives `M` at columns 58 and 4 (codons 1 and
  3), and its lane is two runs on one line. `frames-62.gb`'s `fP` codon 3 is
  split by the origin (bases 62, 1, 2): its `T` is over base 1 in block 0.
  `framesamb`'s unreadable `TAR` codon is `?` over base 14 in frame +1.
* **Lines, top to bottom.** Up to four lanes, then `more`; a plus-strand CDS's
  residues then numbers; `+3 +2 +1`; top; bottom; a minus-strand CDS's numbers
  then residues; `-1 -2 -3`. Lanes are every drawn feature except `source`,
  longest first then file order: on `tet`'s first line the `tet` gene and CDS
  come before the two 4-base `misc_binding` sites. Synthetic, with 22 drawn
  features over base 1, shows four lanes and a `more` line.
* **Numbers** (D055). Multiples of 5 first, then the line's leftmost numbered
  residue, each centred on its residue, moved inward to stay over the line's
  bases, and dropped if it would touch a number already placed. On pBR322's
  1259-1318 line, `392` (leftmost) starts at column 5 and `395` at 14; the stop
  codon's `*` at column 21 has no number.

Rules a choice of the planner's, settled with the owner only through D055's
mockups and recorded in contract 8j: `source` features have no lane; lane order
is drawn length, then file order; a feature's lane covers its bases as cell
runs, with its name and arrow drawn inside those cells by the view; the `more`
line is a line of its own.

**How the values were checked.** Translations use a hand-coded table checked
against Biopython's table 11. Each CDS's codons come from Biopython's reading of
its location, must give the file's `/translation`, and must match
`cds-map-codons.tsv` where it has rows. Frames are checked against 418
`frame-tables.tsv` rows. Whether a synthetic feature is drawn comes from
`../genbank/synthetic-unused.tsv`: a location refusal other than `fuzzy_end` is
not drawn (D053). The script asserts that blocks tile the record, that no line
but the last is short without a CDS, that a short line is at most 2 bases short,
that no block edge falls inside a codon of the selected CDS (the codon the
origin splits excepted), and that no two items on a line overlap. The values
above are asserted by hand in the script.

Eight controls, each run once: a shifted grid fails at `frames62 cds:fM: a block
starts inside the codon at [58, 59, 60]`; no shortening fails the same way for
`fP`; residues over first bases fail at `tet`'s `M`; left-aligned numbers fail at
`392`; lanes by start fail at the `tet` line's lanes; frames across the origin
fail at `frame-tables.tsv` row `synthetic - 1 1`; a minus CDS's residues beside
the strand fail the line-order check. One is not caught: dropping the gutter's
minimum of 2, because every record here has at least 10 bases. Contract 8j
pins that with a unit test.

What these tables do not pin: glyphs, styles, the cursor, the status line, the
title bar, the overview inset (8l2), and the gutter's minimum of 2.

## Regenerating

From the repository root, with the pinned environment
(`tests/fixtures/requirements.txt`):

```
python -I tests/fixtures/view/reference/make_frames_62.py --print
python -I tests/fixtures/view/reference/make_frames_62.py --check
python -I tests/fixtures/view/reference/cds_map.py
python -I tests/fixtures/view/reference/cds_map.py --check
python -I tests/fixtures/view/reference/search_dna.py
python -I tests/fixtures/view/reference/search_dna.py --check
python -I tests/fixtures/view/reference/search_protein.py
python -I tests/fixtures/view/reference/search_protein.py --check
python -I tests/fixtures/view/reference/layout_columns.py
python -I tests/fixtures/view/reference/layout_columns.py --check
python -I tests/fixtures/verify.py
```

`cds_map.py` without `--check` writes the five tables; with `--check` it
regenerates them in memory and compares byte for byte (D039). It refuses to
write anything when its own cross-checks fail: a CDS listed as usable that
does not read as one, a codon list that differs from Biopython's extraction,
residue numbering that is not 1..`residues` plus `stop`, base rows that do not
cover a CDS's bases exactly once, or `place_reading` values that are not
1,2,3 in reading order.
