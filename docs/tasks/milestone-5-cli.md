# Milestone 5 plan: the `luria` CLI and machine-readable output

Status: **Plan approved by the owner, 2026-10-16. Owner decisions recorded as
D041 (§4).** In progress; see the per-contract files `milestone-5*.md`.
Planner: Claude. Decision: D034 (Milestone 5 is "a minimal CLI and structured
output, so complete scientific workflows can be exercised without a TUI, web
frontend, or LLM"). Branch: `feat/milestone-5`.

## 1. What this milestone is

Every scientific capability so far is a Rust library call. Nothing outside
`cargo test` can run it. This milestone adds one executable, `luria`, that:
- reads a sequence file (FASTA or GenBank) and a typed request file;
- runs the Q5 workflow end to end: resolve the target, edit, verify, design;
- writes one structured JSON result.

The request file is also what the Milestone 6 LLM adapter will produce (D004,
D016). So its schema is the typed surface natural language will target, and
it has to be complete and strict now.

## 2. What exists today, established 2026-10-16

These are facts about the code, not design choices:
1. **No file import exists.** `sequence::normalize` takes text and skips `;`
   comment lines, but there is no FASTA reader: `>` headers are stripped by
   hand in `q5_primer_fixtures.rs`. There is no GenBank reader, so CDSs can
   only be built in code with `Cds::create`.
2. **`luria-core` has no dependencies**, and there is no serializer for any
   type. Every result is read through accessors.
3. **The entry points are ready.**
   - `q5::execute`, then `q5_design::design_run`, for one edit;
   - `q5::execute_set`, then `q5_design::design_set`, for a variant set;
   - `q5_expand::expand_replacement`, for a degenerate replacement, which
     returns a variant set.

   They all take a `Sequence`, a `Vec<Cds>`, an intent and a
   `PrimerDesignRequest`.
4. **Errors are structured, but two variants aren't.** Every failure is a
   `DomainError` variant with a typed payload and an `ErrorCategory`. Two
   variants are catch-alls told apart only by message text (review item C8):
   - `InvalidMutationRequest`, with 26 producer sites;
   - `InvalidPrimerCandidate`, with 18.
5. **Two algorithm versions aren't recorded on results.**
   `MUTATION_ALGORITHM_VERSION` and `PRIMER_DESIGN_VERSION` are never
   attached to anything (review note under §B), contrary to D021.
6. **Primer messages print spans three ways:** `[a, b)`, `[a, b]` (fixed in
   B13), and bare `a, b`.
7. **D028:** a design request with no policy is an error, not a default.

## 3. Proposed shape

### 3.1 Crates

| Crate | Package | What it holds |
| --- | --- | --- |
| `crates/core` | `luria-core` | Unchanged role. Gains the FASTA and GenBank readers (§3.3) and stable error codes (§3.6). Stays dependency-free. Import lives in core because the TUI and web interface will need the same readers (D003). |
| `crates/cli` | `luria-cli`, binary `luria` | Argument handling, request parsing, result serialization, exit codes. No scientific logic (D003). |

Request parsing and result serialization are library modules inside
`luria-cli`, so tests and the Milestone 6 adapter can call them without
running the binary. A separate crate waits until a second consumer exists
(D025).

### 3.2 Commands

```
luria q5 design REQUEST.json [--write-edited DIR]
luria version
luria help | --help
```

- **`q5 design`** runs the whole workflow and prints one JSON result
  document to stdout. With `--write-edited DIR`, it also writes each edited
  construct as FASTA, `DIR/<result-name>.fa`. It never overwrites an
  existing file.
- **`version`** prints, as JSON:
  - the package version;
  - every algorithm name and version: mutation, primer, Tm, native design,
    Q5 workflow;
  - the request and result schema versions.

There are no other commands. A `sequence inspect` helper is optional
(contract 5i).

### 3.3 Sequence input: FASTA or GenBank (D041)

**FASTA** (`crates/core/src/fasta.rs`):
- reads **exactly one** record: one `>` header line, then sequence lines;
- zero records, more than one record, or sequence text before the header is
  refused (`MalformedInput`);
- the body goes through `sequence::normalize` unchanged, so D008's rules
  stay in one place;
- the header text is kept as source metadata. It is **not** the sequence's
  identity: the `id` comes from the request (D007);
- FASTA carries no topology, so the request states it. A missing topology is
  an error (D009);
- CDSs are declared in the request: intervals, strand, completeness, genetic
  code, numbering.

**GenBank** (`crates/core/src/genbank.rs`):
- **One record only.** A second record after `//` is refused. The sequence
  comes from `ORIGIN`, with its position numbers and spacing removed, and
  goes through `normalize` as FASTA does. The LOCUS name is source metadata;
  the `id` still comes from the request.
- **Topology comes from the LOCUS line** (owner answer G4). A LOCUS line that
  states neither `linear` nor `circular` is refused. The request does not
  restate topology for a GenBank template.
- **The request names the CDSs to use** (G1). Only `CDS` features are
  candidates. The request lists ids, and each must match exactly one CDS
  feature: zero or two is an error. Each named CDS must be one luria can
  represent, or the run fails with that feature's own error. Every other
  feature, of any type, is listed in the result as *not used*, with the
  reason when it's a CDS luria couldn't have used. A problem in an unused
  feature never fails the run.
- **A CDS's id** is the first of `/locus_tag`, `/gene`, `/label` (G2). The
  result records which qualifier it came from. A CDS with none of the three
  can't be named.
- **Locations, converted from GenBank's 1-based inclusive form to `[start,
  end)`** (D006):
  - accepted:
    - `a..b`;
    - `complement(a..b)`;
    - on a circular template only, a join crossing the origin (D036),
      written either `join(a..n,1..b)` or with `complement(…)` around it or
      around each part;
  - refused, each with its own error:
    - fuzzy ends (`<`, `>`), which would be an incomplete CDS;
    - `^`;
    - `order()`;
    - any other `join` (exons);
    - single-base locations;
    - references into other records (`ACC:1..10`).
- **Qualifiers:**
  - `/codon_start` other than 1 and `/pseudo` are refused for a named CDS;
  - `/transl_table`: absent or 1 is the standard code. 11 (bacterial) is
    accepted only when the CDS's first codon is `ATG`, and the CDS is then
    imported as the standard code with the table recorded in provenance
    (owner answer G5). 11 with any other start codon, and every other
    table, is refused;
  - `/translation`, when present, must equal luria's translation of the CDS
    without its terminal stop. A mismatch refuses the CDS before any edit
    (G3), and the error names the first differing residue. A CDS without
    `/translation` is accepted;
  - other qualifiers are ignored.
- **Header sections** (DEFINITION, ACCESSION, REFERENCE and so on) are not
  interpreted. ACCESSION and VERSION are kept as source metadata when
  present.

### 3.4 The request file (schema `luria.q5-request/1`)

The request is JSON and **strict**. Unknown fields, missing required fields,
and wrong types are each refused, with the JSON path of the problem. Nothing
has a default that changes the science: policy, style, minimum overlap,
placement, coordinate convention and, for FASTA, topology are all required. Paths are
relative to the request file's directory.

Example (illustrative; contract 5g pins the real one against the fixtures):

```json
{
  "schema": "luria.q5-request/1",
  "coordinates": "zero-based-half-open",
  "template": {
    "id": "sid-1",
    "fasta": "sid-1_cDNA.fa",
    "topology": "linear",
    "sha256": "optional; if present it must match",
    "cds": [
      { "id": "sid-1", "intervals": [[0, 3153]], "strand": "+",
        "completeness": "complete", "geneticCode": "standard",
        "numbering": "from-cds-start" }
    ]
  },
  "scope": "sid-1",
  "variants": {
    "composition": "separate_variants",
    "members": [
      { "name": "D404N",
        "residue": { "cds": "sid-1", "number": 404, "from": "D", "to": "N",
                     "codon": "AAC" } }
    ]
  },
  "design": { "policy": "luriaNative", "style": "q5_polymerase",
              "minAnnealingOverlap": 15, "placement": "inline",
              "buffer": "q5_reaction" }
}
```

- **The template is either form, not both.**
  - FASTA: `fasta`, `topology` and declared `cds` objects, as above.
  - GenBank: `genbank` and `cds` as a list of ids to import, e.g.
    `"cds": ["bla"]`. No `topology`.
- **Exactly one** of three edit forms, each mapped to its own core entry
  point:
  - `edit`: one edit, through `execute`;
  - `variants`: a set, through `execute_set`;
  - `degenerate`: a degenerate replacement, through `expand_replacement`
    and then `execute_set`.
- **Six edit kinds, matching `Q5MutationIntent`:**
  - `residue`, where `codon` gives `Supplied`; `"policy": "default"` gives
    `Default`, and one of the two is required;
  - `residue` without `cds`, which is `ResidueWithoutCds` and can come back
    as `needs_clarification`;
  - `substitution`;
  - `deletion`;
  - `insertion`, which asserts the base on each side;
  - `replacement`.
- **Residue numbers are 1-based,** as they are in core. Nucleotide
  coordinates follow the `coordinates` field, which must be `"zero-based-half-open"` (D041).
- **`scope`** is optional. When it's absent, target resolution applies, as in
  the library.
- **`combined_construct`** is accepted by the parser and refused by the
  workflow, exactly as the library refuses it today.

### 3.5 The result document (schema `luria.q5-result/1`)

One JSON document on stdout, for success, clarification and failure alike.
Its top-level fields:
- `schema`;
- `state`: `verified`, `needs_clarification` or `failed` (D020);
- `experimentalValidation`: always `"not_performed"`. This keeps
  computational verification from being read as experimental validation
  (D020, product brief);
- `provenance` (D021):
  - the request file's SHA-256;
  - the template's id, SHA-256, length and topology, and its source: the
    format, the FASTA header or the GenBank LOCUS name, accession and
    version;
  - for GenBank, each imported CDS with the qualifier its id came from, and
    every feature not used, with the reason where there is one;
  - the operation;
  - every algorithm name and version, including the two the results don't
    carry today (§2 item 5);
  - the package version.
- `request`: the normalized request as luria understood it (D004, D016).
- `target`: the resolved CDS, or `null`.
- `clarifications` and `candidates`, when `state` is `needs_clarification`.
- `results`, one per member. Each has:
  - the mutation: kind, recorded coordinates (D040 order), original and
    written text, codon and residues for a residue edit, and the edited
    construct's SHA-256 and length;
  - its warnings, structured, each with a `code`;
  - `design` or `designFailure`.
- `design` gives:
  - the policy, style, algorithm and version;
  - both primers 5′→3′, with span, tail, length, annealing Tm with its basis
    and any `substitutedBecause`, whole-primer Tm, and quality warnings;
  - the annealing temperature, balance, edit span and pair relationship.
- `error`, when `state` is `failed`: `code`, `category`, `message` and
  `details`, the payload fields.

Rules:
- **Deterministic:** the same inputs give byte-identical output (D012). No
  timestamps, hostnames or absolute paths. Map keys come out in a fixed
  order. Floats print in shortest round-trip form and are never rounded.
- **Intervals are always `[start, end]` JSON pairs** under the declared
  convention. They never appear in prose fields that a consumer has to
  parse.
- **The edited sequence text** goes in the JSON only through
  `--write-edited`. A 64-member expansion of a plasmid shouldn't produce a
  multi-megabyte document by default.

### 3.6 Error codes (review item C8)

- Every `DomainError` variant gets a stable `code()`: its name in snake_case,
  e.g. `expected_residue_mismatch`. A test pins the full list and checks the
  codes are distinct.
- `InvalidMutationRequestProblem` and `InvalidPrimerCandidateProblem` each
  gain a `rule` field, an enum with one variant per distinct rule. The 26
  and 18 producer sites each name their rule. The roughly 20 tests that
  match on `reason.contains(…)` assert the rule instead. Message text stays
  as it is.
- Errors raised by the CLI itself get their own codes, with category
  `MalformedInput`: `request_unreadable`, `request_invalid` (with the JSON
  path), `template_digest_mismatch`, `output_exists`. The FASTA and GenBank
  readers are in core, so their errors are `DomainError` variants with codes
  of their own: one per refused location form and per refused qualifier.

### 3.7 Exit codes

| Code | Meaning |
| --- | --- |
| 0 | `verified`, and every member designed |
| 1 | Internal error or I/O failure (the document still goes to stdout if it can be written) |
| 2 | Usage error: bad arguments. Text on stderr, no JSON |
| 3 | `needs_clarification` |
| 4 | `failed`, `MalformedInput` |
| 5 | `failed`, `BiologicalValidation` |
| 6 | `failed`, `Unsupported` |
| 7 | Verified, but at least one member, of a set or a single edit, could not be designed (`designFailure`, G7) |

### 3.8 Outside this milestone

- Natural language (M6), a TUI (M7) and a web interface (M8).
- Python workers (D017 stays deferred).
- Other annotated formats (EMBL, SnapGene `.dna`, GFF), writing GenBank, and
  persistence (D022).
- Network access of any kind (D023).
- Any change to the science. Every number the CLI prints comes from an
  existing, tested library call.

## 4. Owner decisions, 2026-10-16 (recorded as D041)

| # | Question | Answer |
| --- | --- | --- |
| Q1 | Dependencies | `serde` and `serde_json` in `luria-cli` only; arguments parsed by hand. `luria-core` stays dependency-free. |
| Q2 | Coordinate convention | Zero-based half-open everywhere in the JSON, declared in a required `coordinates` field. |
| Q3 | Sequence input | FASTA and GenBank, both in this milestone. |
| Q4 | Human-readable output | JSON only. |
| G1 | Unsupported CDSs in a GenBank file | The request names the CDSs to use; everything else is listed as not used. |
| G2 | GenBank CDS identity | The first of `/locus_tag`, `/gene`, `/label`, recorded in the result. |
| G3 | `/translation` mismatch | An error: the CDS is refused before any edit. |
| G4 | GenBank topology | From the LOCUS line. Missing is an error. |
| G5 | `/transl_table=11` (2026-10-17) | Accepted only when codon 1 is `ATG`, where tables 1 and 11 give the same protein residue for residue; imported as standard, recorded in provenance. Refused with any other start codon. |
| G6 | Buffer in the request (2026-10-17) | Required. Either the three numbers (`monovalentMillimolar`, `primerNanomolar`, `dmsoPercent`) or the named preset `"q5_reaction"` (150 mM, 500 nM, 0 % DMSO, `BufferModel::q5_reaction`). The result echoes the numbers, and the preset name when one was used. |
| G7 | Single-edit design failure (2026-10-17) | Treated like a one-member set: `verified`, the member has its mutation and a `designFailure`, exit 7. |
| G8 | `--write-edited DIR` that doesn't exist (2026-10-17) | Created, parents included. Existing files are still never overwritten (`output_exists`). |
| G9 | FASTA against GenBank (2026-10-17) | Proven on pBR322, not on a hand-written SID-1 GenBank record. |
| R1 | Public cross-check record (2026-10-17) | pBR322, J01749.1. pUC19 L09137.2 was the first choice but has no CDS features. |

## 5. Contracts, in order

Each is one small worker contract with tests first.

| # | Contract | Scope |
| --- | --- | --- |
| 5a1 | Stable error codes | `DomainError::code()`, all variants, plus a pinned-list test. `error.rs` only. |
| 5a2 | `InvalidMutationRequest` rules | A `rule` enum, all 26 sites, and the `reason.contains` tests converted. |
| 5a3 | `InvalidPrimerCandidate` rules | The same for the 18 sites. |
| 5b | FASTA reader | `fasta.rs` in core, one record, with the fail cases above, and fixture-driven tests including `sid-1_cDNA.fa`. |
| 5c1 | GenBank location parser | GenBank location text to intervals and strand: the accepted forms, every refusal, and the 1-based conversion. A pure function, tested on its own. |
| 5c2 | GenBank record reader | LOCUS (topology, name), ACCESSION and VERSION, the FEATURES table (keys, locations across continuation lines, qualifiers), ORIGIN, `//`. No CDS rules yet. |
| 5c3 | GenBank CDS import | Selecting named CDSs, the id rule, the qualifier rules and the `/translation` check, producing `Cds` values and the not-used list. |
| 5d | CLI crate skeleton | Workspace member, `luria version`, `luria help`, usage errors, exit-code type. |
| 5e | Request parsing | Strict JSON to core types for both template forms, all three edit forms and six kinds, path-located errors, and one test per refusal. |
| 5f | Result serialization | Every result type to JSON, provenance, determinism (same input twice gives identical bytes). |
| 5g1 | `luria q5 design` wiring | The three run paths, the result document, exit codes, unit tests. |
| 5g2 | `--write-edited` | Writing each edited construct as FASTA, creating the directory (G8), never overwriting (`output_exists`). |
| 5g3 | End-to-end tests | §6: the fixture-checked cases, fail-closed, FASTA against GenBank (G9), and byte-identical determinism. |
| 5h | Docs | `docs/cli.md` (both schemas, exit codes, error codes), README usage, `product-brief.md` status. |
| 5i | Optional: `luria sequence inspect` | Digest, length, topology and source metadata of a FASTA or GenBank file, and a GenBank file's CDS features with their ids and whether luria can use them. Useful for writing a request. |

The order makes the error codes stable before any JSON exposes them, and the
inputs exist before the output that depends on them.

## 6. How correctness is established

- **No golden file is produced by running luria** (AGENTS.md).
  End-to-end tests run the binary, parse its JSON, and compare each
  scientific field with a fixture that is already independently verified:
  - the SID-1 D404N design and edited construct, against
    `example-construct/` and `q5-primer/native.tsv`;
  - deletion, insertion and replacement rows, against `native-*.tsv`;
  - residue edits, including minus strand and origin-crossing, against
    `mutation/cases.tsv`.
- **GenBank fixtures,** in a new `tests/fixtures/genbank/`:
  - small synthetic records written by hand, one per accepted location form
    and one per refusal, with intervals derived by hand and recorded in the
    README;
  - one public plasmid record, cross-checked by a tracked script using
    Biopython's independent GenBank parser (D039): its sequence digest, and
    each CDS's intervals, strand and translation. I'll choose the record
    and ask before fetching it.
  - **The same template in either format gives the same design** (G9),
    proven on pBR322: a FASTA request declaring the `tet` CDS by interval
    and a GenBank request importing `tet` by name must give byte-identical
    results, apart from the provenance that states the format. The SID-1
    cDNA stays FASTA only; a hand-written 3153-base GenBank record with a
    `/translation` qualifier is not practical, and a generated one would not
    be independently verified.
- **The planner hand-writes one small expected document** in `docs/cli.md`
  for a synthetic case, with values taken from those fixtures. A test checks
  that the CLI's output matches it field by field. This pins the shape.
- **The brief's fail-closed example,** "mutate residue 42 to glycine" against
  a template with two CDSs, must exit 3 with `needs_clarification`, list both
  candidates, and perform no edit.
- **Each refusal in §3.4 and §3.6** has a test asserting its `code` and exit
  code.
- **Determinism:** each end-to-end case runs twice, and the outputs must be
  byte-identical.

## 7. Required checks

The AGENTS.md checks, from the repository root, with `TMPDIR=$PWD/target/tmp`.
`cargo test --workspace` now includes `luria-cli`'s tests, which run the
built binary.
`python -I tests/fixtures/verify.py` runs only when a fixture changes (D039).
