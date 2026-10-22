# The `luria` command line

This page describes `luria-cli` 0.1.0, which reads requests in schema
`luria.q5-request/1` and writes results in schema `luria.q5-result/1`.
The decisions behind it are D041 and D034 in `docs/decisions.md`.
`luria q5 interpret`, which turns a request in words into a proposed
request, is §10 (D044). `luria review`, which shows a result or a
proposal in the terminal, is §11 (D046). `luria new`, a form for a new
design, is §12 (D047).

`luria` runs one workflow, Q5 site-directed mutagenesis. It reads a request
and the sequence file the request names, applies the edit, verifies it,
designs a primer pair and prints one JSON document on stdout.

## 1. Commands

```
luria q5 design REQUEST.json [--write-edited DIR]
luria q5 interpret --template FILE.gb (--text TEXT | --text-file FILE)
    --backend openai-compatible --endpoint URL --model NAME
    [--design PROFILE.json] [--allow-remote] [--write-request FILE.json]
luria review REQUEST.json [--export DIR]
luria review --template FILE.gb (--text TEXT | --text-file FILE)
    --backend openai-compatible --endpoint URL --model NAME
    --request FILE.json [--design PROFILE.json] [--allow-remote] [--export DIR]
luria new [--design PROFILE.json] [--export DIR]
    [--backend openai-compatible --endpoint URL --model NAME [--allow-remote]]
luria version
luria help
```

| Command | What it does |
| --- | --- |
| `q5 design REQUEST.json` | Runs the request and prints one result document. |
| `--write-edited DIR` | Also writes each edited construct to `DIR/<member name>.fa`. `DIR` is created if needed. An existing file is never overwritten. The file's header is the result name, and it records no topology. |
| `q5 interpret ...` | Asks a language model what a request in words means and prints the proposed request, or luria's questions, as one document. It runs nothing (§10). |
| `review ...` | Shows a result, or a proposal it runs once you approve, in the terminal, and exports it (§11). |
| `new ...` | A form for a template and mutations, which writes and runs the request once you approve (§12). |
| `version`, `--version` | Prints the package version, every algorithm's name and version, and the two schema names (§4). |
| `help`, `--help`, `-h` | Prints the usage text. |

A command line luria can't act on prints a one-line reason and the usage
text on stderr, nothing on stdout, and exits 2. An argument that isn't valid
UTF-8 is a usage error. In `q5 design`, `--` ends the options, so a request
file whose name starts with `-` can be given after it.

## 2. Before you rely on a result

- **Results are computational.** `verified` means luria applied the edit,
  checked the edited construct independently and derived the primers from
  the template. Every result says `"experimentalValidation":
  "not_performed"`.
- **Circular templates cannot be designed yet.** Primer design covers
  linear templates only (D029). On a plasmid, or any template whose
  topology is `circular`, the edit is still applied, verified and
  reported, but each member carries an `unsupported_primer_design_scope`
  design failure instead of primers, and the run exits 7.
- **Nothing is guessed.** An ambiguous request gives
  `needs_clarification`, and a wrong one gives `failed`. luria never picks
  a CDS, corrects a base or fills in a missing setting.

## 3. A worked example

Save this request as `d404n.json` next to `sid-1_cDNA.fa`, which is in
this repository at `tests/fixtures/example-construct/input/`. It edits the
SID-1 cDNA at residue 404, aspartate to asparagine, writing the codon
`AAC`; paths in a request are relative to the request file's own directory.

<!-- luria:request -->
```json
{
  "schema": "luria.q5-request/1",
  "coordinates": "zero-based-half-open",
  "template": {
    "id": "sid-1",
    "fasta": "sid-1_cDNA.fa",
    "topology": "linear",
    "cds": [
      {
        "id": "sid-1",
        "intervals": [[0, 3153]],
        "strand": "+",
        "completeness": "complete",
        "geneticCode": "standard",
        "numbering": "from-cds-start"
      }
    ]
  },
  "scope": "sid-1",
  "edit": {
    "name": "D404N",
    "residue": {
      "cds": "sid-1",
      "number": 404,
      "from": "D",
      "to": "N",
      "codon": "AAC"
    }
  },
  "design": {
    "policy": "luriaNative",
    "style": "q5_polymerase",
    "minAnnealingOverlap": 15,
    "placement": "inline",
    "buffer": "q5_reaction"
  }
}
```

Then:

```
luria q5 design d404n.json
```

The run exits 0 and prints the document below.

**Melting temperatures and the two thermodynamic terms are written here to
the precision the verification tables record** — two decimals for a Tm,
one for ΔH and ΔS, two for a G+C percentage. luria itself prints every
float at full precision (D041), so those figures are the document's values
to that precision, not its exact bytes. Every other field is exact,
including `annealingTemperature`, which is a whole number of degrees by
definition (D030).

<!-- luria:result -->
```json
{
  "schema": "luria.q5-result/1",
  "state": "verified",
  "experimentalValidation": "not_performed",
  "provenance": {
    "package": {
      "name": "luria-cli",
      "version": "0.1.0"
    },
    "algorithms": [
      {
        "role": "mutation",
        "name": "luria-core-mutation",
        "version": "0.3.0"
      },
      {
        "role": "primer",
        "name": "luria-core-primer",
        "version": "0.2.0"
      },
      {
        "role": "tm",
        "name": "nn-santalucia-hicks-2004-owczarzy-2004-monovalent",
        "version": "0.3.0"
      },
      {
        "role": "native_design",
        "name": "luria-core-primer-design-native",
        "version": "0.7.0"
      },
      {
        "role": "q5_workflow",
        "name": "luria-core-q5-sdm",
        "version": "0.5.0"
      }
    ],
    "requestSha256": "def773534f112340da19af055721d09ba772ac62a0da937741e35e0c859fcba0",
    "operation": "edit",
    "template": {
      "id": "sid-1",
      "sha256": "54704740a9f04b79d17147ef95788ad9f0f11e5263843a7ed490924d43b55e3a",
      "length": 3153,
      "topology": "linear",
      "source": {
        "format": "fasta",
        "path": "sid-1_cDNA.fa",
        "header": "3xFLAG-mVenus-sid-1 cDNA"
      },
      "genbank": null
    },
    "buffer": {
      "preset": "q5_reaction",
      "monovalentMillimolar": 150.0,
      "primerNanomolar": 500.0,
      "dmsoPercent": 0.0
    }
  },
  "request": {
    "schema": "luria.q5-request/1",
    "coordinates": "zero-based-half-open",
    "template": {
      "id": "sid-1",
      "fasta": "sid-1_cDNA.fa",
      "topology": "linear",
      "cds": [
        {
          "id": "sid-1",
          "intervals": [[0, 3153]],
          "strand": "+",
          "completeness": "complete",
          "geneticCode": "standard",
          "numbering": "from-cds-start"
        }
      ]
    },
    "scope": "sid-1",
    "edit": {
      "name": "D404N",
      "residue": {
        "cds": "sid-1",
        "number": 404,
        "from": "D",
        "to": "N",
        "codon": "AAC"
      }
    },
    "design": {
      "policy": "luriaNative",
      "style": "q5_polymerase",
      "minAnnealingOverlap": 15,
      "placement": "inline",
      "buffer": "q5_reaction"
    }
  },
  "target": {
    "id": "sid-1",
    "intervals": [[0, 3153]],
    "strand": "+",
    "completeness": "complete",
    "geneticCode": "standard",
    "numbering": "from-cds-start",
    "length": 3153
  },
  "clarifications": [],
  "candidates": [],
  "warnings": [],
  "results": [
    {
      "name": "D404N",
      "mutation": {
        "kind": "residue",
        "state": "verified",
        "cds": "sid-1",
        "residue": 404,
        "originalResidue": "D",
        "writtenResidue": "N",
        "originalCodon": "GAT",
        "writtenCodon": "AAC",
        "codonPolicy": "supplied",
        "codonPolicySource": null,
        "coordinates": [1209, 1210, 1211],
        "edited": {
          "id": "D404N",
          "sha256": "6ab7d5a6fdd995692ed5cda5079db64c97a70935deb9edfd10fe402628b7ff50",
          "length": 3153
        }
      },
      "warnings": [],
      "design": {
        "policy": "luriaNative",
        "style": "q5_polymerase",
        "algorithm": {
          "name": "luria-core-primer-design-native",
          "version": "0.7.0"
        },
        "minAnnealingOverlap": 15,
        "placement": "inline",
        "editSpan": [1209, 1212],
        "forward": {
          "role": "forward",
          "sequence": "CGTTGAGCAAAACTTCGGGGCGC",
          "length": 23,
          "span": [1199, 1222],
          "editedSpan": [1199, 1222],
          "tail": null,
          "parts": [
            {
              "role": "context",
              "annealing": true,
              "sequence": "CGTTGAGCAA",
              "span": [1199, 1209],
              "primerStart": 0,
              "primerEnd": 10
            },
            {
              "role": "mutation",
              "annealing": false,
              "sequence": "AAC",
              "span": [1209, 1212],
              "primerStart": 10,
              "primerEnd": 13
            },
            {
              "role": "context",
              "annealing": true,
              "sequence": "TTCGGGGCGC",
              "span": [1212, 1222],
              "primerStart": 13,
              "primerEnd": 23
            }
          ],
          "annealing": {
            "sequence": "CGTTGAGCAATTCGGGGCGC",
            "length": 20,
            "overlap": 20,
            "spans": [[1199, 1209], [1212, 1222]]
          },
          "tm": {
            "celsius": 67.72,
            "basis": "mismatched_wild_type_site",
            "substitutedBecause": null,
            "enthalpyKcal": -139.9,
            "entropyCal": -373.3
          },
          "wholePrimerTm": null,
          "warnings": [
            {
              "code": "gc_content_outside_band",
              "gcPercent": 60.87,
              "message": "G+C is 60.9 %, outside the 40–60 % guideline"
            },
            {
              "code": "single_base_run",
              "base": "A",
              "length": 4,
              "start": 8,
              "message": "a run of 4 A at position 8; more than 3 can cause primer slippage"
            },
            {
              "code": "single_base_run",
              "base": "G",
              "length": 4,
              "start": 16,
              "message": "a run of 4 G at position 16; more than 3 can cause primer slippage"
            },
            {
              "code": "three_prime_gc_run",
              "length": 8,
              "message": "the last 8 bases are all G or C; more than 2 invites primer-dimers"
            }
          ]
        },
        "reverse": {
          "role": "reverse",
          "sequence": "AGCGGATTGAGCAGTTTTC",
          "length": 19,
          "span": [1180, 1199],
          "editedSpan": null,
          "tail": null,
          "parts": [
            {
              "role": "context",
              "annealing": true,
              "sequence": "AGCGGATTGAGCAGTTTTC",
              "span": [1180, 1199],
              "primerStart": 0,
              "primerEnd": 19
            }
          ],
          "annealing": {
            "sequence": "AGCGGATTGAGCAGTTTTC",
            "length": 19,
            "overlap": 19,
            "spans": [[1180, 1199]]
          },
          "tm": {
            "celsius": 62.90,
            "basis": "matched_complement",
            "substitutedBecause": null,
            "enthalpyKcal": -146.8,
            "entropyCal": -397.7
          },
          "wholePrimerTm": null,
          "warnings": [
            {
              "code": "single_base_run",
              "base": "T",
              "length": 4,
              "start": 14,
              "message": "a run of 4 T at position 14; more than 3 can cause primer slippage"
            }
          ]
        },
        "annealingTemperature": 64.0,
        "tmBalanceMet": true,
        "pair": {
          "deletedWidth": 0,
          "separation": 0,
          "boundary": 1199,
          "abuts": true,
          "relationship": "reverse span [1180, 1199) and forward span [1199, 1222): abutting"
        },
        "unresolved": [
          {
            "input": "minTm",
            "effect": "not accepted: luriaNative applies its fixed 55 °C floor (D030)"
          },
          {
            "input": "maxAnnealingOverlap",
            "effect": "not accepted (owner, 2026-10-14): no upper bound was applied; the chosen lengths are reported"
          }
        ]
      },
      "designFailure": null
    }
  ],
  "error": null
}
```

### 3.1 Reading it

- **The edit.** `mutation.coordinates` are the three template positions the
  codon occupies, `[1209, 1210, 1211]`, and `editSpan` is the same region as
  a half-open interval, `[1209, 1212]`. `originalCodon` is what the template
  held and `writtenCodon` is what luria wrote; `codonPolicy` is `supplied`
  because the request named the codon, so `codonPolicySource` is null. A
  request that said `"policy": "default"` instead would carry `default` and
  name the codon-usage table it consulted.
- **The edited construct** is identified by digest, not by its text:
  `6ab7d5a6…`, still 3153 bases. `--write-edited DIR` writes it to
  `DIR/D404N.fa`.
- **The primers** are stated 5′→3′. The forward primer carries the new
  codon: its three `parts` are template context, the written `AAC`, then
  template context again, and the `mutation` part is the one part with
  `"annealing": false`. Its `annealing` block is therefore the 20 bases
  that do anneal, in two intervals either side of the codon. The reverse
  primer is pure template, one `context` part, and `editedSpan` is null
  because it reads the template rather than the edited construct.
- **Two Tms, on two bases.** The forward primer's `basis` is
  `mismatched_wild_type_site`: the first cycle anneals it over the
  unedited template, where the codon mismatches. The reverse primer
  mismatches nothing, so its basis is `matched_complement`. The two are
  never averaged. `annealingTemperature` is the lower Tm rounded to a
  whole degree, plus 1 °C, capped at 72 °C: the rule NEB's Tm calculator
  and NEBaseChanger use for Q5 (D030). Here that is 62.90 → 63, plus 1,
  so 64. `tmBalanceMet` says the two Tms are within 5 °C.
- **Warnings are guidelines, not failures.** This pair is still a design:
  the forward primer is 60.9 % G+C against a 40–60 % guideline, has runs
  of four A and four G, and ends in eight G or C. Each warning carries the
  measurement luria took as well as its text.
- **`unresolved`** names the design inputs this policy does not accept, so
  a caller can tell a value was not applied silently.

## 4. `luria version`

```json
{
  "schema": "luria.version/1",
  "package": "luria-cli",
  "version": "0.1.0",
  "algorithms": [
    {
      "role": "mutation",
      "name": "luria-core-mutation",
      "version": "0.3.0"
    },
    {
      "role": "primer",
      "name": "luria-core-primer",
      "version": "0.2.0"
    },
    {
      "role": "tm",
      "name": "nn-santalucia-hicks-2004-owczarzy-2004-monovalent",
      "version": "0.3.0"
    },
    {
      "role": "native_design",
      "name": "luria-core-primer-design-native",
      "version": "0.7.0"
    },
    {
      "role": "q5_workflow",
      "name": "luria-core-q5-sdm",
      "version": "0.5.0"
    }
  ],
  "schemas": {
    "request": "luria.q5-request/1",
    "result": "luria.q5-result/1"
  }
}
```

Every result repeats these algorithm names and versions in its
`provenance`, so a result says which code produced it (D021).

## 5. Exit codes

| Code | Meaning |
| --- | --- |
| 0 | `verified`, and every member designed; for `luria q5 interpret`, `proposed`; for `luria review`, you quit |
| 1 | An internal error, or a failure to write the output: stdout, a `--write-edited` file or a `--write-request` file; for `luria q5 interpret`, also a model server that could not be reached or answered badly (§10.7) |
| 2 | Usage error: bad arguments. Text on stderr, no JSON |
| 3 | `needs_clarification` |
| 4 | `failed`, category `malformed_input` |
| 5 | `failed`, category `biological_validation` |
| 6 | `failed`, category `unsupported` |
| 7 | Verified, but at least one member could not be designed |
| 8 | `failed`, from `luria q5 interpret`: the model's reply was refused (§10.7) |

Exit 7 is the plasmid case of §2, and also a single edit whose design
failed: the edit stands, and `designFailure` says why there are no
primers.

An unreadable file is not exit 1. A request file that cannot be read is
`request_unreadable`, and a template file the request names is
`template_unreadable`; both are `malformed_input` and exit 4 (§6). Exit 1 is
an internal error, or a write that failed — stdout, or a file `--write-edited`
or `--write-request` asked for. Unless stdout itself is what failed, the
document still goes to stdout. When the run has a reason outside the document,
it goes to stderr as one line, `luria: ` then the reason; for a failed write
that reason is the operating system's message. A failed `--write-edited` write
takes exit 1 in place of the status the run had reached, 7 included: the
document still shows each member's `design` or its `designFailure`.

## 6. Error codes

A `failed` document's `error`, and a member's `designFailure`, carry:

| Field | What it holds |
| --- | --- |
| `code` | A stable identifier, such as `expected_residue_mismatch`. |
| `rule` | The specific rule broken, for the six codes that carry one: `invalid_mutation_request`, `invalid_primer_candidate`, `malformed_fasta`, `unusable_genbank_location`, `malformed_genbank` and `unusable_genbank_cds`. Null otherwise. |
| `category` | `malformed_input`, `biological_validation` or `unsupported`, which chooses the exit code. |
| `message` | The human-readable reason. |
| `path` | The JSON path in the request, for an error found while the request is read and prepared; null otherwise, an error raised while the edit runs included — `expected_residue_mismatch` gives a path of null. Milestone 6 left this as it is (M6-12). |
| `details` | The error's own fields, such as the coordinates or residues involved. Four fields that hold nothing but interval text are left out: `primer_text_at_span_mismatch` loses its span, and `primer_pair_coverage` has empty `details`, so those intervals appear in `message` only. A refused GenBank CDS location gives `cds` and `line` as well, null when no named feature caused it. Of the CLI codes, `output_exists` gives `file` and `output_name_unusable` gives `name`; the other four give empty `details`. |

Match on `code` and `rule`, never on `message`. Six codes are the CLI's own;
every other code comes from `luria-core`.

| CLI code | Category | When |
| --- | --- | --- |
| `request_unreadable` | `malformed_input` | The request file cannot be read, or its bytes are not UTF-8. |
| `request_invalid` | `malformed_input` | The request is not valid JSON or breaks the schema, a template path is absolute, or two declared CDSs share an id. `path` names the field. |
| `template_unreadable` | `malformed_input` | The template file the request names cannot be read. |
| `template_digest_mismatch` | `malformed_input` | The request stated `template.sha256` and it is not the digest of the template's normalized sequence (§7). |
| `output_exists` | `malformed_input` | With `--write-edited`: the file luria would write already holds something — a file, a directory or a link, a dangling one included. Nothing is written. |
| `output_name_unusable` | `malformed_input` | With `--write-edited`: a result name holds a path separator (`/` or `\`), or one of `:` `*` `?` `<` `>` and the vertical bar, or two result names differ only in case. Nothing is written. |

The complete list of core codes is pinned by
`luria_core::error::tests::every_variant_has_its_pinned_code`, and every
code is distinct.

## 7. The request at a glance

`luria.q5-request/1` is strict: an unknown field, a missing required
field or a wrong type is refused, and `path` names where. Nothing that
changes the science has a default.

| Field | Required | What it is |
| --- | --- | --- |
| `schema` | yes | `"luria.q5-request/1"`. |
| `coordinates` | yes | `"zero-based-half-open"`, the convention of every nucleotide interval in the request and the result. Residue numbers are 1-based. |
| `template` | yes | Either a FASTA template — `fasta`, `topology`, and `cds` as declared objects — or a GenBank one: `genbank`, and `cds` as a list of ids to import. Never both. `id` is the template's identity and comes from the request, not the file (D007). `sha256` is optional; when present it is checked against the SHA-256 of the normalized sequence — upper-cased bases, line breaks and whitespace removed — the value `provenance.template.sha256` reports, not `sha256sum` of the file. A GenBank CDS is imported only from an `ATG` start, under every `/transl_table` (D042). |
| `scope` | no | The CDS the edit applies to. Without it, luria resolves the target, and asks when more than one CDS could be meant. |
| one of `edit`, `variants`, `degenerate` | yes | One edit, a set of named members, or a degenerate replacement expanded into a set. Exactly one. |
| `design` | yes | `policy`, `style`, `minAnnealingOverlap`, `placement` and `buffer`. `buffer` is either the preset `"q5_reaction"` or the three numbers `monovalentMillimolar`, `primerNanomolar` and `dmsoPercent` (G6). A stated buffer needs a positive salt and primer concentration and a DMSO proportion of at least zero, and the most luria accepts is 1000 mM monovalent, 10000 nM primer and 20 % DMSO: above that the request is refused as `invalid_buffer_model` (D043). |

An edit is one of six kinds, each its own object inside `edit` or a
member: `residue`, `substitution`, `deletion`, `insertion`,
`replacement`, and `residue` without a `cds`, which is the form that can
come back as `needs_clarification`. A `residue` edit needs either a
`codon` or `"policy": "default"`.

## 8. The result at a glance

One document for every outcome, with every top-level key always present —
`null` or `[]` where it does not apply — so a consumer never has to test
whether a key exists.

| Field | What it holds |
| --- | --- |
| `schema` | `"luria.q5-result/1"`. |
| `state` | `verified`, `needs_clarification` or `failed`. |
| `experimentalValidation` | Always `"not_performed"`. |
| `provenance` | The package, every algorithm and version, the request file's SHA-256 over its raw bytes (unlike `template.sha256`, which is over the normalized sequence), the operation, the template and the buffer. For a GenBank template, `template.genbank` carries `importedCds`, each with its `intervals` and `strand` next to where it came from, and `unusedFeatures`: every feature not imported, a `CDS` one with the refusal importing it would have given. |
| `request` | The request as luria understood it. It is itself a valid request: parsing the echo gives the same request back. |
| `target` | The resolved CDS, or null. |
| `clarifications`, `candidates` | Why luria is asking, and what it could have meant. Non-empty only when `state` is `needs_clarification`. |
| `warnings` | Warnings about the operation as a whole, such as a degenerate expansion that skipped the wild type. |
| `results` | One entry per member: its `name`, its `mutation`, its edit `warnings`, and `design` or `designFailure` — one or the other, never both. |
| `error` | The failure, when `state` is `failed`; null otherwise. |

The result is deterministic: the same inputs give byte-identical output
(D012). There are no timestamps, hostnames or absolute paths, keys come
out in a fixed order, floats print at full precision, and every
nucleotide interval is a `[start, end]` pair rather than text to parse.
The edited sequence appears only through `--write-edited`.

## 9. What this page does not cover yet

This page is the worked example and the tables above. The full reference —
every GenBank location form and qualifier rule luria accepts or refuses,
every request field in detail, every result field in detail, and every
error code with its `details` — is not written yet. Until it is,
`docs/tasks/milestone-5-cli.md` §3.3 to §3.5 is the authority, and
`luria_core::error` is the authority on codes.

## 10. Natural language: `luria q5 interpret`

`luria q5 interpret` asks a language model what a request in words
means, and writes the **proposed** `luria.q5-request/1` request, or the
questions luria needs answered first, as one JSON document. It does not
run the request. You read the proposal, and if it says what you meant,
you run it with `luria q5 design`, unchanged. The decisions behind it are
D044 and `docs/tasks/milestone-6-nl.md`.

```
luria q5 interpret --template FILE.gb (--text TEXT | --text-file FILE)
    --backend openai-compatible --endpoint URL --model NAME
    [--design PROFILE.json] [--allow-remote] [--write-request FILE.json]
```

### 10.1 What the model may and may not do

The model **extracts**; luria **decides**. The model returns a small JSON
document saying what you asked for, in your own words: the CDS you named,
each substitution as a token such as `A42G` with the words of yours that
say so, whether the variants are separate or combined, and what you said
about numbering. Then luria, with no model involved:

- **grounds** every string the model returned against your text. A CDS
  name, a quote, a codon or anything else that is not in what you wrote
  ends the run, `failed` with `ungrounded_extraction`;
- **checks each token against its quote**: the number must be written
  there, and both residues named there (one-letter code beside a digit,
  three-letter code, or English name), the original before the target.
  If your words do not show it, luria asks rather than assume;
- **reads the notation**, the CDS and your design profile, and builds the
  request itself. The model never writes a request field, a coordinate, a
  sequence or a primer.

So a model that invents a mutation, swaps a direction or borrows a number
cannot turn that into a request: the worst it can do is make luria ask, or
refuse. The proposal then goes through `luria q5 design`, which checks the
original residue against the CDS as for any request.

### 10.2 What leaves your machine

Your text, and for each CDS of the record: its id, `/gene`, `/product`,
length in residues, strand, and whether luria can use it. **Never the
sequence, and never a coordinate.** The document records the exact
messages sent (`interpretation.messages`), so you can check.

An endpoint whose host is not this machine (`localhost`, `127.x.x.x` or
`[::1]`) is refused unless you add `--allow-remote`, before any file is
read or anything is sent. This build speaks plain HTTP only: an
`https://` endpoint is refused, and HTTP to another machine is
unencrypted. HTTPS and the Anthropic API are deferred
(`docs/tasks/later-tls-and-anthropic.md`).

### 10.3 Setting it up

The backend is any server with an OpenAI-compatible `/chat/completions`
endpoint, such as llama.cpp's `llama-server`, Ollama or vLLM. Give its
base URL and the model's name; luria has no defaults for either:

```
luria q5 interpret --template synthetic-linear.gb --design profile.json \
    --text "In synA, design Q5 primers for A42G and W87A as separate variants. Number residues from the start of the CDS." \
    --backend openai-compatible --endpoint http://127.0.0.1:8080/v1 \
    --model qwen3.8-flash-next-coder-iq1_m --write-request proposal.json
luria q5 design proposal.json
```

If the server wants a key, put it in the environment variable
`LURIA_OPENAI_API_KEY`; it is sent as a bearer token and appears in no
document or message. luria refuses to send it to an endpoint that is not
this machine, because this build has no HTTPS. luria sends temperature 0, a
fixed seed, and the extraction's JSON schema as `response_format`, so a
server that supports it constrains the reply. It sends no thinking or
token-limit setting: the server's own configuration decides. On the small
local model luria was measured with, thinking made extraction four times
slower and no better (`docs/research/m6-local-model-extraction-2026-10-19.md`).

**Templates are GenBank only**, so the CDSs come from the record. A FASTA
template is refused (`interpret_fasta_unsupported`); write that request by
hand. Primer design still covers linear templates only (§2).

### 10.4 The design profile

Words rarely state a primer design, a buffer or a codon, and luria never
defaults them. You keep them in a file of your own,
`luria.q5-design-profile/1`, and pass it with `--design`:

```json
{
  "schema": "luria.q5-design-profile/1",
  "design": {
    "policy": "luriaNative",
    "style": "q5_polymerase",
    "minAnnealingOverlap": 15,
    "placement": "inline",
    "buffer": "q5_reaction"
  },
  "codon": "default",
  "numbering": "from-cds-start"
}
```

`design` has exactly the shape of a request's `design` (§7) and is checked
by the same rules. `codon` is used for a substitution whose text names no
codon; `numbering` when the text says nothing about numbering. Each member
is optional: one that is absent, and that the request needs, is asked
about. Your words win over the profile, and the document's `origins` say
which values came from where.

### 10.5 The document at a glance

One document for every outcome, every key always present, `null` or `[]`
where it does not apply, deterministic for the same reply.

| Field | What it holds |
| --- | --- |
| `schema` | `"luria.q5-interpretation/1"`. |
| `state` | `proposed`, `needs_clarification` or `failed`. |
| `package` | `luria-cli` and its version. |
| `request` | The proposed request, exactly as `--write-request` writes it; null otherwise. |
| `questions` | Each with a stable `code`, its `subject` and a `message`. |
| `origins` | For each value of the proposal, `text` or `profile`. |
| `assumptions` | What luria took as given: the template's id is the record's LOCUS name. |
| `error` | `code`, `rule` and `message` when `state` is `failed`. |
| `template` | The template's file name, LOCUS name and sequence digest. |
| `interpretation` | The backend, the prompt version, the messages sent, the reply exactly as received, the extraction as parsed, and the first ungrounded string if any. Kept apart from the request (D021). |

`--write-request FILE` writes the bare request too, only when the state
is `proposed`. The file must be in the template's directory, because a
request names its template relative to itself, and luria never replaces
an existing file.

### 10.6 Questions

<!-- luria:question-codes -->
| Code | Asked when |
| --- | --- |
| `operation_unsupported` | The text asks for something other than amino-acid substitutions, or something besides them. |
| `mutation_unstated` | The text names no substitution. |
| `mutation_notation_unsupported` | A token is not a one- or three-letter substitution (a deletion, a stop, a DNA change). |
| `residue_number_unstated` | Your words for a substitution do not state its residue number. |
| `original_residue_unstated` | Your words do not state the residue there now, which luria checks rather than assume. |
| `target_residue_unstated` | Your words do not state the residue to put there. |
| `substitution_direction_unclear` | Your words do not name the residue there now, then the number, then the residue to put there. |
| `mutation_quote_ambiguous` | The words given for one substitution hold more than one number. |
| `mutation_repeated` | The same substitution is listed twice. |
| `mutation_not_extracted` | Your text holds a substitution such as `A42G` that the reply does not list. |
| `codon_unstated` | No codon in the text and no codon policy in the profile. |
| `codon_not_in_quote` | A codon is in the text, but not in the words given for its substitution. |
| `cds_unnamed` | The text names no CDS. luria never picks one, even when the record has one. |
| `cds_not_found` | The record has no CDS of that name (matched exactly). |
| `cds_unusable` | The record has it, but luria refuses to import it; the message gives core's code. |
| `composition_unstated` | Two or more substitutions, and not said whether separate or combined. |
| `composition_unclear` | Your words say both, or negate it, or the model's reading differs from luria's. |
| `numbering_unstated` | Neither the text nor the profile says how residues are numbered. |
| `numbering_unsupported` | The text gives a numbering other than from the start of the CDS. |
| `design_unstated` | No profile states the primer design. |

All the questions are asked at once. A combined construct is proposed as
asked; `luria q5 design` then refuses it (`unsupported_composition`), as it
would a hand-written one.

luria checks that every substitution written as a token (`A42G`,
`Ala42Gly`, `p.A42G`) is in the reply. It cannot check that for one
written only in words ("alanine 42 to glycine"): read the proposal's
members before you run it.

### 10.7 Errors and exit codes

Exit codes are those of §5: **0** is `proposed`, **3** is
`needs_clarification`, and **8** is a model reply that was refused.

| Code | Exit | When |
| --- | --- | --- |
| `text_unreadable`, `text_empty` | 4 | The text file cannot be read, is not UTF-8, or the text is empty. |
| `profile_unreadable`, `profile_invalid` | 4 | The profile cannot be read, or breaks its schema; the message gives the JSON path. |
| `template_unreadable` | 4 | The template cannot be read, or is not UTF-8. A malformed record gives core's code, such as `malformed_genbank`. |
| `interpret_fasta_unsupported` | 6 | The template is FASTA. |
| `input_too_large` | 4 | The text is over 8 KiB, the profile over 64 KiB, or the template over 32 MiB. |
| `request_location_unusable` | 4 | `--write-request` is not in the template's directory. |
| `output_exists` | 4 | The `--write-request` file already exists, a dangling link included. |
| `backend_unreachable`, `backend_status`, `backend_response` | 1 | The server could not be reached, answered with an error status, or answered in a shape that is not a chat completion. |
| `model_output_invalid` | 8 | The reply is not an extraction: wrong JSON, an unknown or missing field, or over 64 KiB. A whole response over 1 MiB is `backend_response`. |
| `ungrounded_extraction` | 8 | The reply holds a string your text does not contain. |
| `internal_request_round_trip` | 1 | A bug in luria: the proposal does not read back as itself. |

Every local input is checked before anything is sent, so a missing file or
a bad profile never costs a model call. A usage error, the remote gate and
an `https://` endpoint included, exits 2 with nothing on stdout.

When the `--write-request` file cannot be written, the `proposed`
document still goes to stdout, the reason goes to stderr as `luria:
could not write <name>: <reason>`, and the exit is 1.

## 11. Reviewing in the terminal: `luria review`

`luria review` shows a result, or a proposal, on a full-screen terminal
view, so you can read it before you rely on it. Every value on screen is
a field of the documents §8 and §10.5 describe, except the variant's
protein, which luria-core reads from the edited construct. The decisions behind it are D046 and
`docs/tasks/milestone-7-tui.md`.

```
luria review REQUEST.json [--export DIR]
luria review --template FILE.gb (--text TEXT | --text-file FILE)
    --backend openai-compatible --endpoint URL --model NAME
    --request FILE.json [--design PROFILE.json] [--allow-remote] [--export DIR]
```

Both forms need a terminal on stdin and stdout, and a screen of at least
80 by 24; with less, the screen says so and shows nothing else.

### 11.1 A result

`luria review REQUEST.json` runs the request as `luria q5 design` does
and opens on the overview, or on the problem screen when the result
failed or asks a question. Every screen sits in one frame whose title
shows the state as a coloured badge (`✓ verified`, `✗ failed`, `?
needs clarification`) and the experimental validation (`not tested at
the bench`). Positions on screen are 1-based and inclusive, as GenBank
writes them (`GAT→AAC at 1210–1212`, `insert GAT after 1209`); the
request and result files stay zero-based (§3). `?` opens a help pop-up
with the legend and the keys.

| Screen | What it shows | Keys |
| --- | --- | --- |
| Overview | The template, the target CDS and its strand, any operation warnings, and the variants, one row each with its edit and whether a primer pair was designed | `j`/`k` move, `enter` open, `p` provenance, `e` problem, `x` export |
| Variant | The forward primer above the top strand, the variant's protein between the strands, the bottom strand, the reverse primer below it written 3'→5'; edited bases and primer bases that do not anneal in lower case and coloured; then a table of each primer's length, Tm (one decimal) and sequence 5'→3', the annealing temperature, and each warning in words | `d` details, `i` expand or collapse a long insertion, `j`/`k` scroll |
| Details | Everything the result records for the variant: the edit, its warnings by code, and each primer with its parts, Tm basis and warnings; or why no pair was designed | `j`/`k` scroll |
| Provenance | The package, every algorithm and version, the request and template digests, what a GenBank record imported and did not, the buffer | `j`/`k` scroll |
| Problem | The error and its details, or the clarifications and candidates | `p` provenance |

`esc` goes back, and quits from the overview; `q` or `ctrl-c` quits from
anywhere; `PgUp`, `PgDn` and `Home` scroll. The variant screen reads the
template again and draws it only when its digest is the one the result reports.
The protein is the one luria-core reads from the edited construct, in
the CDS's frame, on past a frameshift, ending at the first stop codon;
the construct is used only when its digest is the variant's. An
insertion longer than 30 bases is shown by its boundaries until `i`
expands it, and starts collapsed each time the screen opens.

### 11.2 A request in words

The second form interprets your text as `luria q5 interpret` does (§10),
with the same backend, remote gate and key rules, and shows the proposal:
any questions or error first, then the proposed request exactly as it
will be written, where each value came from, the model's reading with
each quote, and everything sent to and received from the model.

Its frame's badge is `PROPOSAL`, never a result's. Nothing is written
and nothing runs until you press `a`, then `y` on a
screen that names the file. luria then writes the request to
`--request`, under `--write-request`'s rules (beside the template,
never replacing a file), runs `luria q5 design` on that file, and shows
the result as §11.1 does. `--request` is checked before the model is
called. Quitting before you approve writes nothing.

Questions are read only. To answer one, quit, change your text or your
profile, and run again.

### 11.3 Export

With `--export DIR`, `x` on the overview lists the files an export would
write, and `y` writes them:

| File | What it holds |
| --- | --- |
| `<request>.result.json` | The result, byte for byte what `luria q5 design REQUEST.json` prints |
| `<member>.fa` | Each edited construct, as `--write-edited` writes it |
| `<request>.order.tsv` | An order table for IDT's Bulk Input |

`<request>` is the request file's name without `.json`. If any of these
files exists, nothing is written. Before it writes, luria runs the
request again and refuses if the result is not the one on screen.

<!-- luria:order-scales -->
The order table is tab-separated, with the header `Name`, `Sequence`,
`Scale`, `Purification`, and one row per primer of each designed member,
forward then reverse, named `<template id>_<member>_F` and `_R`. Every row
is `STD` (standard desalting). The scale is set by the primer's length:

| Length | Scale |
| --- | --- |
| 15-60 nt | `25nm` |
| 61-90 nt | `100nm` |
| 91-100 nt | `250nm` |
| 101-200 nt | `4nmU` |

A primer outside 15-200 nt, a base other than A, C, G or T, or a result
that is not verified gives no table, and the export screen says why; the
other files are still written. luria writes the file and never places an
order. Check the scale and purification against your own needs before
you paste it.

## 12. Entering a design in the terminal: `luria new`

`luria new` is a form for a new design: a template, the mutations and
two choices. It builds the request, shows it exactly as it will be
written, and writes and runs it only once you approve. The result then
opens on `luria review`'s screens (§11). The decisions behind it are D047
and `docs/tasks/milestone-7k-new.md`.

```
luria new [--design PROFILE.json] [--export DIR]
    [--backend openai-compatible --endpoint URL --model NAME [--allow-remote]]
```

It needs a terminal on stdin and stdout, and a screen of at least 80 by
24.

### 12.1 The form

| Field | What you give |
| --- | --- |
| Template | A file, FASTA or GenBank, by its path; or a sequence you paste, raw or as FASTA. A paste is saved as a new FASTA file when you approve. |
| Name, Shape | For FASTA or a paste: the template's name and whether it is linear or circular. A GenBank record gives its own. |
| CDS | For FASTA or a paste: the whole sequence, or its first and last base (1-based, inclusive), and its strand. For GenBank: one of the record's CDSs that luria can use. Never guessed. |
| Mutations | One per line, in the shorthand of §12.2. Each line shows what luria reads it as, or why it cannot. |
| In words | Your request in words, sent to your model as `luria q5 interpret` does (§10). Only with `--backend`, and only with a GenBank template. Use words or mutation lines, not both. The form's overlap and codon choice go with them, and residues count from the CDS's first codon unless your words say otherwise. |
| Primers | `inline`, shown and fixed: confine-to-tail placement is not supported yet (D029). |
| Overlap | The minimum annealing overlap in nucleotides. Empty until you or `--design` give it. |
| Codon | For a residue change without a codon: the default codon policy, or a codon on each line (`D404N:AAC`). |
| Save as | The request file's name. |

`tab` and `shift-tab` move between fields, `←`/`→` change a choice,
`enter` on `[ Review ]` builds the request, and `esc` quits. `ctrl-c`
quits from any screen, writing nothing. A field that is missing or a
line luria cannot read is named in red, and the cursor moves to it.

### 12.2 The mutation shorthand

Positions are template bases, 1-based and inclusive, as GenBank writes
them; residue numbers count from the CDS's first codon.

| Line | Meaning |
| --- | --- |
| `D404N`, `p.D404N`, `Asp404Asn` | Residue 404 from D to N, the codon by the form's choice |
| `D404N:AAC` | The same, with codon `AAC` |
| `1210 G>A` | Base 1210 from G to A |
| `1210-1212 GAT>AAC` | Bases 1210 to 1212 replaced (any length) |
| `del 1210-1212`, `del 46` | Bases deleted |
| `ins GAT after 1209` | `GAT` inserted between bases 1209 and 1210 |

A stop (`*`, `X`, `Ter`), a `?`, a position of 0, a range written
backwards, bases that do not match the range, or anything else is
refused with the reason; the typed bases of a substitution or a
replacement must be the template's. Two or more lines are separate
variants, one construct each.

### 12.3 Approving

`[ Review ]` shows the request exactly as it will be written and the
files that will be written: the request beside a template file, as
`--write-request` requires (§10), or, for a paste, the pasted FASTA and
the request in the current directory. With words, it shows `luria q5
interpret`'s proposal instead, as §11.2 does. Nothing is written until
you press `a`, then `y`. A file is never replaced. luria then runs the
request as `luria q5 design` does and opens the result as §11.1 does,
with export when `--export DIR` was given (§11.3). Quitting before you
approve writes nothing.
