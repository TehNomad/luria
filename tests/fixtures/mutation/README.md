# Fixture: mutation of sequences and CDSs

## Purpose

Reference values for the second milestone (`crates/core`): the deterministic
mutation domain. These cases pin down target validation, the expected-original
check, nucleotide and residue substitution, codon-selection policy, strand-aware
coordinate mapping, the provenance a result carries, and the verification pass
that is separate from generation.

They are synthetic sequences, not organism data, and no case here carries
provenance or approval status: mutation is computational only (D020).

`mut-a.txt` is the same content as `seq-a.txt` in the Milestone 1 fixture, so
the residue tables below can be read against that fixture's derivations.

## Files

| Path | Role |
| --- | --- |
| `mut-a.txt` | Linear, 30 bases: `ATGGCTTGGAATCAGCAGCAGCAGCAGCAG` |
| `mut-b.txt` | Linear, 30 bases: `ATTCCAAGCCATCAGCAGCAGCAGCAGCAG` — the reverse complement of `mut-a`'s first twelve bases repeated; used as the decoy content for the display name `mut-a` |
| `mut-c.txt` | Circular, 24 bases: `GCTTGGAACGAACGAACGAACATG` |
| `mut-d.txt` | Linear, 18 bases, terminal stop codon: `ATGGCTTGGAATTTTTAA` |
| `mut-e.txt` | Linear, 30 bases, one ambiguous codon (`N` at coordinate 6): `ATGGCTNGGAATCAGCAGCAGCAGCAGCAG` |
| `cases.tsv` | 47 cases: one per line, tab separated |

Sequence files carry a `;`-prefixed comment line. The canonical text is the
remaining text with whitespace removed and letters upper-cased, exactly as in
the Milestone 1 fixture.

## Case conventions

`cases.tsv` columns:

`case_id`, `kind`, `sequence_file`, `topology`, `reference`, `cds_id`,
`cds_intervals`, `cds_reference`, `cds_strand`, `completeness`,
`genetic_code`, `numbering`, `target`, `expected_original`,
`replacement`, `policy`, `result_id`, `expected`, `category`,
`verify_with`, `note`

- `kind` is `sequence`, `nucleotide`, or `residue`. A `sequence` row only
  normalizes the file and checks its digest; the other two run a mutation.
- `cds_id` is `none` (the request is unscoped) or an identifier. `cds_intervals`
  is `none` or `start:end[;start:end…]`, zero-based half-open, in biological
  order; a decreasing start is legal only at an origin crossing.
- `cds_reference` is `same`, or `identifier:fixture-file`, which builds a CDS
  against an artifact whose display name is `identifier` and whose content is
  `fixture-file`. That is how a CDS that names another artifact is expressed.
- `target` is a coordinate (`3`), a deliberately too-wide interval (`3:5`), a
  coordinate naming another artifact (`3#mut-b.txt`), a residue number (`2`),
  or a residue number naming another CDS (`2@cds-2`).
- `expected_original` and `replacement` are a base (`A`, `C`, `G`, `T`) for a
  nucleotide row — an ambiguity code is used only to ask for a refused request —
  and a residue letter for a residue row (`*` for the stop is used to ask for a
  mutation that is out of scope).
- `policy` is `default`, `unspecified`, or `supplied:XXX`. Matching is exact
  and case-sensitive; anything else is out of scope.
- `expected` is `hash:<hex>`, `error:<VariantName>`, or a `+`-joined record:
  `edited:<text>`, `residues:<LETTERS>[*]`, `codon:<TXT>`, `coords:<n,n,n>`.
  A trailing `*` on `residues` means the CDS still ends with a stop codon.
  `coords` are the reference coordinates the operation recorded, ascending.
- `category` is filled only for error cases.
- `verify_with` is `none` (the result is left `Calculated`), `same` (verify
  against the CDS the request used, expecting `Verified`), or
  `narrower:<intervals>` (verify against a CDS that does not cover the recorded
  target, expecting `MutationVerificationFailure`).

## Derivation of the expected values

Nothing here was produced by the Rust code under test. Digests were computed
with Python's `hashlib`; residues were read off the published standard genetic
code by hand, and the edited texts were written out coordinate by coordinate.

### Digests

SHA-256 over the ASCII bytes of the canonical text, no header, no newline, no
topology:

| Sequence | Canonical text | Digest |
| --- | --- | --- |
| `mut-a` | `ATGGCTTGGAATCAGCAGCAGCAGCAGCAG` | `a83f6d825c144cef563438c085821ecbd8986cdaf0f65c75f7994ed616a2fd80` |
| `mut-b` | `ATTCCAAGCCATCAGCAGCAGCAGCAGCAG` | `748c00c04185e923b15fdd4033b3c76f9374ed7ac5728b9024311da1c76a06fa` |
| `mut-c` | `GCTTGGAACGAACGAACGAACATG` | `46ff16f41b721953db78b03c231e5e5dee9cbd82cf85b2eca20dc9236dcfc81a` |
| `mut-d` | `ATGGCTTGGAATTTTTAA` | `d4e4418df44cfd4e44157b02cf3eb66535a32a603d43b0159c8acb0424942daf` |
| `mut-e` | `ATGGCTNGGAATCAGCAGCAGCAGCAGCAG` | `0a3fa25d9ab6959626a7d706e5dda2fa0c32429a0f58acd145c7307249b69254` |

`mut-a`'s digest is the one already published for `seq-a` in the Milestone 1
fixture, which is the cross-check that both fixtures describe the same text.

### Residues

The standard genetic code is the one Milestone 1 cites: NCBI's Genetic Codes
reference, "1. The Standard Code" (`transl_table=1`), read over `Base1 = T…C…A…G`
with codon index `16 * place(first) + 4 * place(second) + place(third)`,
`T=0, C=1, A=2, G=3`.

| Case | CDS text (codons) | Residues |
| --- | --- | --- |
| `mut-a` over `[0,30)+` | `ATG GCT TGG AAT CAG CAG CAG CAG CAG CAG CAG` | `M A W N Q Q Q Q Q Q` |
| `mut-a` over `[0,30)-` | `CTG CTG CTG CTG CTG CTG ATT CCA AGC CAT` | `L L L L L L I P S H` |
| `mut-c` over `[22,24)+[0,7)+` | `TGG CTT GGA` | `W L G` |
| `mut-d` over `[0,18)+` | `ATG GCT TGG AAT TTT TAA` | `M A W N F`, final `TAA` is a stop, not a residue |
| `mut-e` over `[0,30)+` | `ATG GCT NGG AAT …` | `NGG` expands to `AGG CGG GGG TGG` = `R R G W`: no single residue |

### Default codons

The `default` policy picks, for the replacement residue, the codon with the
highest count in the *E. coli* K-12 codon usage table: Kazusa Codon Usage
Database, *Escherichia coli* W3110 (a K-12 substrain), species 316407, 4332
CDSs, 1372057 codons, retrieved 2026-10-14 from
<https://www.kazusa.or.jp/codon/cgi-bin/showcodon.cgi?species=316407&aa=1&style=N>.
The counts the cases depend on, read off that page:

| Residue | Codon counts | Default |
| --- | --- | --- |
| `L` | `CTG` 72898, `TTA` 18894, `TTG` 18664, `CTC` 15272, `CTT` 15082, `CTA` 5266 | `CTG` |
| `F` | `TTT` 30462, `TTC` 22705 | `TTT` |
| `W` | `TGG` 20889 (the only W codon) | `TGG` |

The previous rule, first codon in standard table order, gave `TTA` for `L`.

These expectations were checked with a Python script that parses the Kazusa
page and uses a codon table written out per amino acid, sharing no code with
`crates/core`; it agreed with every success case.

### Edits

Each success case states the edited text as written out by hand. Two are worth
showing:

- `res-a-1`: residue 1 (`ATG`, `M`) to `L`, default codon `CTG`, written at
  coordinates 0, 1, 2 → `CTG GCT TGG AAT …`, residues `L A W N Q Q Q Q Q Q`.
  The edit differs at coordinate 0 only; coordinates 1 and 2 already held
  `T` and `G`.
- `res-a-minus-7`: `mut-a` under `[0,30)-`, residue 7 (`ATT`, `I`) to `L`
  (`CTG`). The codon sits at extracted indices 18, 19, 20, which are reference
  coordinates 11, 10, 9. Writing `C`, `T`, `G` there means reference 11 ← `G`,
  reference 10 ← `A`, reference 9 ← `C` — the complement is applied on write,
  so the edited text is `ATGGCTTGGCAGCAGCAGCAGCAGCAGCAG` and re-extracting the
  minus-strand CDS gives `…CTG…`.
- `res-c-origin` and `res-c-origin-write`: `mut-c` under `[22,24)+[0,7)+`,
  residue 1 is `TGG` (`W`) at reference coordinates 22, 23, 0. To `L` the
  default `CTG` changes only 22 and 23 (coordinate 0 keeps its `G`), giving
  `GCTTGGAACGAACGAACGAACACT`; to `F` the default `TTT` changes 23 and 0,
  giving `TCTTGGAACGAACGAACGAACATT`, so a write on both sides of the origin is
  still covered. Both record coordinates 0, 22, 23.

### Error cases

Each is named for the rule it exercises:

| Case | Rule | Category |
| --- | --- | --- |
| `nuc-a-past-end` | coordinate 30 does not exist in a 30-base sequence | BiologicalValidation |
| `nuc-a-negative` | a coordinate below zero | MalformedInput |
| `nuc-a-wrong-expected` | coordinate 3 holds `G`, the request claimed `T` | BiologicalValidation |
| `nuc-a-outside-scope` | coordinate 20 is outside the scoped CDS `[0,12)`; refused before any edit | BiologicalValidation |
| `nuc-a-bad-letter` | `Z` is not a DNA letter | MalformedInput |
| `nuc-a-ambiguous-replacement` | `N` is a sequence letter but not a base; a substitution writes only `A`, `C`, `G`, `T` | MalformedInput |
| `nuc-e-ambiguous-original` | coordinate 6 of `mut-e` holds `N`; an expected original must be a base, so the position cannot be targeted | MalformedInput |
| `nuc-a-wider-interval` | `3:5` selects three nucleotides, a substitution replaces one | MalformedInput |
| `nuc-a-other-artifact` | the target coordinate names another artifact, so the request is about two sequences | MalformedInput |
| `res-a-no-policy` | the request did not say how to choose a codon; nothing is guessed | MalformedInput |
| `res-a-bad-policy` | `minimal-edit` is not a policy this milestone has | Unsupported |
| `res-a-supplied-disagrees` | `supplied:GCT` encodes `A`, the request asked for `W` | MalformedInput |
| `res-a-supplied-invalid` | `XXX` is not a codon of plain bases, so no `Codon` can exist | MalformedInput |
| `res-a-wrong-residue` | residue 2 is `A`, the request claimed `Q` | BiologicalValidation |
| `res-a-past-last-residue` | residue 11 does not exist in a 10-residue CDS | BiologicalValidation |
| `res-a-zero-residue` | residue numbering is one-based, residue 0 is not a position | MalformedInput |
| `res-a-wrong-cds` | the position names `cds-2`, the request was applied to `cds-1` | BiologicalValidation |
| `res-a-no-change` | `supplied:GCT` re-states the codon already there: the request changes nothing | MalformedInput |
| `res-a-stop-replacement` | mutation to a stop residue is not in this domain's scope | MalformedInput |
| `res-a-ambiguous` | the targeted codon `NGG` stands for more than one residue | BiologicalValidation |
| `res-a-ambiguous-before-edit` | the CDS cannot be read at all, so the target is never reached | BiologicalValidation |
| `res-a-cds-other-artifact` | the CDS intervals name `mut-a` but carry `mut-b`'s content digest | BiologicalValidation |
| `res-a-incomplete` | only explicitly complete CDSs are read | Unsupported |
| `res-a-not-a-triplet` | 29 nucleotides cannot form codons | BiologicalValidation |
| `res-a-numbering` | an explicit residue numbering is not applied in this milestone | Unsupported |
| `res-a-genetic-code` | only the standard genetic code is supported | Unsupported |
| `res-a-verify-narrower` | the result targeted residue 1 of `[0,30)+`; verified against `[3,30)` the claims do not follow | BiologicalValidation |

`res-a-ambiguous` and `res-a-ambiguous-before-edit` are the same artifact read
at two different targets: reading the CDS is a single step, so an ambiguous
codon anywhere in it is reported before the requested residue is considered.
`res-a-no-change` is refused by the operation before anything is written, not
verified and then rejected: the codon the policy chose is the codon already
at that residue, so the request is not a mutation. A nucleotide request that
re-writes the letter already there is refused earlier, at intent construction.

## Scope and known limitations

- Substitutions only. Insertions, deletions, and length-changing edits are not
  modelled, so every success case preserves length.
- One intent per case. Composing mutations is a workflow concern, not a
  mutation-domain concern.
- `verify` is computational only: no case here claims experimental success
  (D020), and the fixture never presents a `Calculated` result as a `Verified`
  one.
- The table verifies a result against the CDS row names, or against a
  narrower set of coordinates under that same identifier (`narrower:`). Verifying
  a residue result against an annotation with a *different* identifier is pinned
  by `verification_requires_the_cds_the_result_names` in
  `crates/core/tests/mutation_invariants.rs`, not by a table row: the table has
  one CDS identifier per row.
- Primer design, mutagenesis strategy, and any optimisation of codon choice are
  deliberately absent; `default` is the highest-count *E. coli* K-12 codon,
  with no other optimisation (edit distance, restriction sites, other hosts).

The semantic choices these cases pin down are recorded in
`docs/tasks/milestone-2-mutation.md`. The driver is
`crates/core/tests/mutation_fixtures.rs`.
