# Fixture: core-foundation sequences, features, and CDSs

## Purpose

Reference values for the first milestone (`crates/core`): canonical sequence
text, content hashing, coordinates, features on linear and circular
sequences, CDS extraction, and translation under the standard genetic code.

These are synthetic sequences, not organism data. They exist only to pin down
the domain model, so nothing here carries provenance or approval status.

## Files

| Path | Role |
| --- | --- |
| `seq-a.txt` | Linear, 30 bases: `ATGGCTTGGAATCAGCAGCAGCAGCAGCAG` |
| `seq-b.txt` | Linear, 30 bases: `ATTCCAAGCCATCAGCAGCAGCAGCAGCAG` |
| `seq-c.txt` | Circular, 24 bases: `GCTTGGAACGAACGAACGAACATG` |
| `seq-d.txt` | Linear, 18 bases, ends with a stop codon: `ATGGCTTGGAATTTTTAA` |
| `seq-e.txt` | Linear, 12 bases, internal stop codon: `ATGTAAGCTTGG` |
| `seq-f.txt` | Linear, 12 bases, ambiguous third codon: `ATGGCTNGGAAT` |
| `seq-g.txt` | Linear, 12 characters, one outside the DNA alphabet: `ATGGCTTGGA#T` |
| `seq-h.txt` | A record with a comment line and no bases |
| `seq-i.txt` | Linear, 11 characters, one non-ASCII character (`ß`, U+00DF) that Unicode case folding would turn into the DNA letter `S` |
| `cases.tsv` | 41 cases: one per line, tab separated |

Sequence files carry a `;`-prefixed comment line. The canonical text is the
remaining text with whitespace removed and letters upper-cased.

## Case conventions

`cases.tsv` columns:

`case_id`, `kind`, `sequence_file`, `topology`, `reference`,
`interval_reference`, `intervals`, `strand`, `completeness`,
`genetic_code`, `numbering`, `expected`, `category`, `note`

- `kind` is `sequence`, `feature`, or `cds`.
- `intervals` is `none` or `start:end[;start:end…]`, zero-based half-open.
- `interval_reference` is `same` (the intervals point at the row's own
  sequence), or `identifier:fixture-file`. The second form builds a sequence
  artifact whose display identifier is `identifier` and whose content is
  `fixture-file`, which is how a same-name/different-content artifact is
  expressed.
- `strand` is `+`, `-`, or `none`. `none` is legal only for a `cds` row and
  means the attribute was not supplied.
- `completeness` is `complete`, `incomplete`, or `none`.
- `genetic_code` is the requested code name. Matching is exact and
  case-sensitive: `STANDARD` and `standard ` are not aliases of `standard`.
- `numbering` is `from-cds-start`, `explicit`, or `none`. Only
  `from-cds-start` is applied in this milestone.
- `expected` is `hash:<hex>`, `extracted:<text>`, `residues:<LETTERS>[+stop]`,
  or `error:<VariantName>`. A `+stop` suffix means the CDS ended with a stop
  codon, which is reported separately and is not a residue.
- `category` is filled only for error cases: `MalformedInput`,
  `BiologicalValidation`, or `Unsupported`.

## Derivation of the expected values

Nothing here was produced by the Rust code under test.

### Digests

SHA-256 over the ASCII bytes of the canonical text, no header, no newline, no
topology. Computed with Python's `hashlib`:

| Sequence | Canonical text | Digest |
| --- | --- | --- |
| `seq-a` | `ATGGCTTGGAATCAGCAGCAGCAGCAGCAG` | `a83f6d825c144cef563438c085821ecbd8986cdaf0f65c75f7994ed616a2fd80` |
| `seq-b` | `ATTCCAAGCCATCAGCAGCAGCAGCAGCAG` | `748c00c04185e923b15fdd4033b3c76f9374ed7ac5728b9024311da1c76a06fa` |
| `seq-c` | `GCTTGGAACGAACGAACGAACATG` | `46ff16f41b721953db78b03c231e5e5dee9cbd82cf85b2eca20dc9236dcfc81a` |
| `seq-d` | `ATGGCTTGGAATTTTTAA` | `d4e4418df44cfd4e44157b02cf3eb66535a32a603d43b0159c8acb0424942daf` |
| `seq-e` | `ATGTAAGCTTGG` | `92daabb494d4ef48b447bf24aabafcae5f1f0d254f00f54cb0487e6e345711a5` |
| `seq-f` | `ATGGCTNGGAAT` | `8ba0b33bc46a5721b5efcdbf5034cc3c053341489a09f68d7f0ccb70003b1f7e` |

The digest of the empty string, `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`,
and of `abc`, `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`,
are the published test vectors and are asserted in `crates/core/src/identity.rs`.

### Extracted text

Half-open selection over the canonical text:

- `seq-a[0,12)` = `ATGGCTTGGAAT`.
- Its reverse complement is `ATTCCAAGCCAT`, which equals `seq-b[0,12)`. That
  is the point of having both sequences: a minus-strand feature on `seq-a`
  must produce the same text as the plus-strand feature on `seq-b`.
- `seq-c[21,24)` = `ATG`, `seq-c[0,6)` = `GCTTGG`; the origin-spanning
  feature selects `ATGGCTTGG`, whose reverse complement is `CCAAGCCAT`.
- `seq-c[3,12)` = `TGGAACGAA`.

### Residues

The standard genetic code is taken from NCBI's Genetic Codes reference,
"1. The Standard Code" (`transl_table=1`),
<https://www.ncbi.nlm.nih.gov/Taxonomy/Utils/wprintgc.cgi> (last updated
2024-09-23). That entry gives the 64 residues in codon order as
`FFLLSSSSYY**CC*WLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG` over
`Base1 = T…C…A…G`, `Base2 = TT..CC..AA..GG..`, `Base3 = TCAG` repeated, so a
codon's index is `16 * place(first) + 4 * place(second) + place(third)` with
`T=0, C=1, A=2, G=3`. The one-letter residue symbols are the IUBMB
recommendations, Table 1 of α-amino acids incorporated into protein
(<https://iupac.qmul.ac.uk/AminoAcid/tab1.html>).

Written codon by codon, from that table:

| Case | Codons | Residues |
| --- | --- | --- |
| `cds_a_first12` | `ATG GCT TGG AAT` | `M A W N` |
| `cds_a_first12_minus` | `ATT CCA AGC CAT` | `I P S H` |
| `cds_c_wrap` | `ATG GCT TGG` | `M A W` |
| `cds_c_wrap_minus` | `CCA AGC CAT` | `P S H` |
| `cds_d_terminal_stop` | `ATG GCT TGG AAT TTT TAA` | `M A W N F`, final `TAA` is a stop, not a residue |
| `cds_e_internal_stop` | `ATG TAA GCT TGG` | stop at codon 2 of 4: the CDS claims residues past it |
| `cds_f_ambiguous_codon` | `ATG GCT NGG AAT` | `NGG` is not a codon of plain bases: it expands to `AGG`, `CGG`, `GGG`, `TGG`, which that table gives as `R`, `R`, `G`, `W`. There is no single residue for it, so it is reported rather than resolved. |

### Error cases

Each is named for the rule it exercises:

| Case | Rule | Category |
| --- | --- | --- |
| `seq_g_invalid_character` | `#` is not a DNA IUPAC letter | MalformedInput |
| `seq_i_non_ascii_fold` | `ß` is not a DNA IUPAC letter, and ASCII-only normalization must not fold it into one | MalformedInput |
| `seq_h_empty_content` | a record with no bases | MalformedInput |
| `seq_a_bad_identifier` | an identifier with a space | MalformedInput |
| `feat_negative_coordinate` | a coordinate below zero | MalformedInput |
| `feat_reversed` | end before start is not a strand encoding | MalformedInput |
| `feat_empty_interval` | the interval selects nothing | MalformedInput |
| `feat_overlapping` | the same nucleotides selected twice | MalformedInput |
| `feat_out_of_order` | intervals not listed in increasing order | MalformedInput |
| `feat_without_intervals` | a feature needs at least one interval | MalformedInput |
| `cds_d_missing_strand` | strand is mandatory, there is no default | MalformedInput |
| `feat_out_of_bounds` | `seq-a` has 30 bases, `[0,31)` asks for 31 | BiologicalValidation |
| `feat_wrong_reference` | the intervals point at another sequence | BiologicalValidation |
| `feat_same_name_other_content` | the intervals name `seq-a` but carry another artifact's content digest | BiologicalValidation |
| `cds_a_length_not_multiple` | 11 nucleotides cannot form codons | BiologicalValidation |
| `cds_e_internal_stop` | a stop codon in the middle of the CDS | BiologicalValidation |
| `cds_f_ambiguous_codon` | the codon stands for more than one residue | BiologicalValidation |
| `cds_a_wrong_reference` | the CDS was built for another artifact that shares the display name | BiologicalValidation |
| `feat_linear_multi` | a linear feature is a single interval | Unsupported |
| `feat_circular_gap` | `3:6;10:14` is not contiguous | Unsupported |
| `feat_circular_out_of_order` | `21:24;1:6` decreases without an origin crossing | MalformedInput |
| `feat_c_wrap` | the only legal crossing: `[21,24)` ends at 24 and `[0,6)` starts at 0 | — |
| `cds_d_incomplete` | only explicitly complete CDSs are translated | Unsupported |
| `cds_d_iso_code` | only the standard genetic code is supported | Unsupported |
| `cds_d_explicit_numbering` | an explicit residue numbering mapping is not applied in this milestone | Unsupported |

A decreasing start is legal only at an origin crossing, which is why
`feat_c_wrap` is accepted and `feat_circular_out_of_order` is rejected as
`IntervalsOutOfOrder` rather than as a structure problem: the listing order is
wrong before contiguity is even considered.

## Scope and known limitations

- One sequence per topology family; no DNA-only vs IUPAC distinction beyond
  `N`, no RNA alphabets, no number systems other than zero-based half-open.
- No primer, mutagenesis, or Q5 behaviour: that is the other fixture.
- Residue positions (`ResiduePosition`) are exercised by the unit tests in
  `crates/core/src/cds.rs` and `crates/core/src/translation.rs`, not by
  `cases.tsv`: Milestone 1 had no operation that consumes a residue
  position. Residue mutations (`apply_residue`, Milestone 2) are covered by
  the `mutation/` set.
- No multi-sequence records, no FASTA parsing: these files are plain text with
  `;` comments.

The concrete semantic choices these cases pin down are recorded in
`docs/tasks/milestone-1-core-foundation.md`.
