# Milestone 1 task record: core sequence foundation

Status: Implemented in `crates/core`
Date: 2026-10-06

This file records the concrete semantic choices made while implementing
Milestone 1. It does not restate `docs/decisions.md`; each entry names the
decision it implements and the specific choice taken here. If an entry
contradicts `docs/decisions.md`, that document wins and this entry is wrong.

## Scope actually implemented

Only sequence, feature, coordinate, strand, CDS, genetic-code, and
translation models, plus their tests and the shared fixtures. No LLM adapter,
CLI, persistence, Python worker, primer logic, or mutation model was built.

## Concrete choices

### Sequence alphabet and normalization (D008)

- Accepted letters are the DNA IUPAC symbols in `DNA_ALPHABET`: `A C G T`
  plus the ambiguity symbols `N R Y S K F M V W`. Anything else is
  `InvalidSequenceCharacters`.
- Case normalization is ASCII-only: `a`–`z` map to `A`–`Z`. There is no
  Unicode case folding. `ß` (U+00DF), `ſ` (U+017F), and accented letters are
  rejected as `InvalidSequenceCharacters`, not folded into DNA letters.
  This is fixture case `seq_i_non_ascii_fold` and the unit test
  `sequence::tests::non_ascii_characters_are_never_folded_into_dna_letters`.
- Normalization never deletes a bad character, never invents a base, and never
  changes topology. An input with no bases is `EmptySequenceContent`.

### Sequence artifact identity (D007)

- A `Sequence` carries a `SequenceReference`, which is the pair
  (`Identifier` display name, `ContentHash` digest).
- The hash is SHA-256 over the canonical ASCII bytes of the normalized
  sequence text: uppercase DNA/IUPAC letters only, no line endings, no
  comments, no length prefix, no serialized struct.
- The digest is computed by a handwritten SHA-256 in `identity.rs`. The unit
  tests in `identity.rs` compare it against digests produced by Python
  `hashlib` (recorded as literals, including the 55/56/63/64/65-byte padding
  boundaries), and the fixture table pins the sequence digests.
- Equality of two `SequenceReference` values requires both the name and the
  digest to match. Two artifacts that share a display name but differ in
  content are therefore not interchangeable. This is proven by
  `identity::tests::sequence_reference_equality_requires_name_and_content`,
  by the per-structure tests in `interval.rs`, `feature.rs`, and `cds.rs`,
  and by fixture cases `feat_same_name_other_content` and
  `cds_a_wrong_reference`.
- Source metadata is deliberately **not** part of content identity. Nothing in
  this milestone claims provenance beyond the display name and the digest.

### Intervals (D006, D009)

- `NucleotideInterval` fields are private. Construction goes through
  `interval::create`, which rejects `start < 0`, `end < 0`, `start == end`,
  and `start > end`. `start()` and `end()` are the only accessors.
- `start > end` is never a wraparound encoding. A circular origin-spanning
  feature is an ordered list such as `[21,24), [0,6)`, and it is legal only
  when the earlier interval ends exactly at the sequence length and the next
  starts exactly at 0. Any other decrease is `IntervalsOutOfOrder`.
- Bounds are validated against the referenced sequence (`CoordinateOutOfBounds`),
  which is a `BiologicalValidation` failure, not a malformed-input failure.

### Strand and extraction (D009)

- `Strand` is an explicit enum (`Plus`, `Minus`); there is no boolean strand
  and no default.
- `extract_feature` and `extract_cds` take only the sequence and the domain
  object. The strand used for extraction is read from the `Feature` or the
  `Cds`, never passed by the caller. The lower-level `extract_on_strand` is
  private for that reason; the fixture driver cannot reach it.
- Extraction concatenates intervals in listed (biological) order, then applies
  the strand. Fixture cases `feat_a_first12_minus` and `feat_c_wrap_minus`
  pin this down.

### Genetic code (D010)

- Genetic code metadata is a typed `GeneticCode` enum with one inhabitant
  today: `Standard`. Construction accepts only the exact, case-sensitive name
  `standard`; `STANDARD`, `Standard`, and `standard ` are rejected as
  `UnsupportedGeneticCode` rather than aliased. No versioned alias table exists
  yet because only one code is in scope.
- The residue table is the 64-character string
  `FFLLSSSSYY**CC*WLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG` over
  base order `T C A G`, taken from NCBI's Genetic Codes reference, Genetic
  Code Table 1 ("The Standard Code", `transl_table=1`). The fixture README
  cites the live page and states the index formula.

### Codon lookup (D010, D005)

- `genetic_code::lookup` returns a typed `CodonOutcome`, not a string and not
  a `"?"` sentinel. The four outcomes are kept distinct:
  `Residue(Residue)`, `Stop`, `Ambiguous`, `Invalid`.
- `Ambiguous` means the codon contains an IUPAC ambiguity letter and could
  stand for more than one residue; `NGG` expands to `AGG`, `CGG`, `GGG`,
  `TGG`, which the standard table gives as `R`, `R`, `G`, `W`. Nothing is
  guessed.
- `Invalid` means the codon contains a character that is not a nucleotide
  symbol at all.
- `Stop` at the last position is a terminal stop and is not a residue;
  `Stop` anywhere else is `InternalStopCodon`, a `BiologicalValidation`
  failure distinct from malformed input.

### Residue positions (D006)

- `ResiduePosition` is a typed value that names its CDS and carries a
  one-based `number`. Raw `usize` is not used as a biological position across
  API boundaries.
- A position of 0, a position past the residue count, and a position naming a
  different CDS are all `DomainError`s
  (`InvalidResiduePosition`, `ResiduePositionOutsideCds`,
  `ResiduePositionWrongCds`), never panics.
- `Numbering` is explicit (`FromCdsStart`, `ExplicitMapping`). Only
  `FromCdsStart` is applied in this milestone; an explicit mapping is refused
  with `UnsupportedNumberingScheme` rather than silently ignored. Fixture
  case: `cds_d_explicit_numbering`.

### Error categories (D005)

- Every `DomainError` variant answers `category()` with one of
  `MalformedInput`, `BiologicalValidation`, or `Unsupported`. Callers classify
  by calling that method; no test or fixture re-implements the mapping.
- `SequenceReferenceMismatch` is `BiologicalValidation`: the input is
  well-formed, it simply does not belong to this sequence.

## Fixture format

- `tests/fixtures/core-foundation/cases.tsv` now has 14 columns:
  `case_id`, `kind`, `sequence_file`, `topology`, `reference`,
  `interval_reference`, `intervals`, `strand`, `completeness`,
  `genetic_code`, `numbering`, `expected`, `category`, `note`.
  `numbering` was added because residue numbering origin is explicit in the
  model (D006, D010) and the fixture must be able to state it.
- `interval_reference` is `same`, `none`, or `identifier:fixture-file`. The
  last form builds a sequence artifact named `identifier` whose content comes
  from `fixture-file`, which is the only way to express "same display name,
  different content" in a table.
- The table has 41 cases. The documented count was 37; four cases were added
  while cleaning up the milestone and are recorded here:
  `seq_i_non_ascii_fold` (ASCII-only normalization),
  `feat_same_name_other_content` (identity by name and digest, feature side),
  `cds_a_wrong_reference` (identity by name and digest, CDS side),
  `cds_d_explicit_numbering` (explicit numbering origin is refused, not
  ignored).
- The fixture driver (`crates/core/tests/core_foundation.rs`) calls only the
  public domain API: `normalize`, `Identifier::create`, `interval::create`,
  `Feature::create`, `Cds::from_optional`, `extract_feature`, `translate`,
  `Sequence::content_hash`, `Translation::residues_text`, and
  `DomainError::category`. It never supplies a strand to an extraction, never
  supplies an expected residue string to the model, and never maps an error
  variant to a category itself.

## Known gaps at the milestone boundary

- Only one genetic code exists. Adding a second one means adding a
  `GeneticCode` variant and a table, not an alias mechanism.
- Compound and spliced features are rejected (`UnsupportedFeatureStructure`);
  no representation for them exists.
- `ResiduePosition` has no consumer operation yet, so it is exercised by unit
  tests only, not by `cases.tsv`.
- RNA alphabets, non-zero-based numbering systems, and sequence source
  metadata are out of scope.
- The SHA-256 implementation is handwritten and validated against Python
  `hashlib` in the test suite; it is not a general-purpose crypto primitive.
