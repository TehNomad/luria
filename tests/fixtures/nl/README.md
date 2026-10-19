# Natural-language corpus

Twenty-four cases for Milestone 6 (`docs/tasks/milestone-6-nl.md`): what
`luria q5 interpret` must make of a user's prose and a model's reply. They
drive contract 6e (building) and 6i (end to end), through the **scripted
backend**: no test contacts a model or the network.

Every case is a tuple of files, indexed by `cases.tsv`:

| Column | What |
| --- | --- |
| `case_id` | the case's name, unique |
| `template` | the GenBank record it runs against, in `../genbank/` |
| `text_file` | `text/<id>.txt`, the user's prose, one line and a newline |
| `reply_file` | `reply/<id>.json`, the exact bytes the model is scripted to return |
| `profile` | `profile/<name>.json`, the design profile, or `-` for none |
| `state` | the expected state: `proposed`, `needs_clarification` or `failed` |
| `codes` | the expected question or error codes, `;`-separated, **alphabetical**, or `-` |
| `request_file` | `request/<name>.json`, the expected request, for a `proposed` case |
| `design_exit` | the exit status `luria q5 design` must give that request |
| `note` | why the case exists |

Several cases share one expected request: `one-sub-prose` and `three-letter`
both propose `request/edit-a42g.json`, which is the point of the pair.

**How `codes` is compared.** As a set, sorted, not in order. The plan fixes
no order for questions, so the fixture must not pin one; a test that
compared sequences would be asserting something no decision has made. The
list is written alphabetically for that reason.

**How a `proposed` request is compared.** As a *parsed* request
(`RequestDocument`), not byte for byte, so that the fixture does not pin a
key order which belongs to `request_echo`. The end-to-end claim — the same
prose gives the same science as a hand-written request — is checked by
comparing `luria q5 design`'s whole output document byte for byte between
the proposal and the tracked expected request (contract 6i).

## How the expectations were derived

By hand, from the prose and from the record's own tables, before any
interpretation code existed. Nothing here was produced by running
`luria q5 interpret`, which does not yet exist.

- **Residues.** Every residue number, letter and codon comes from
  `../genbank/synthetic-linear-residues.tsv`, which
  `../genbank/reference/verify_synthetic_linear.py` re-derives with
  Biopython: residue 42 is Ala, codon `GCA`, and residue 87 is Trp, codon
  `TGG`. So `A42G` and `W87A` state the originals the record really holds.
  `cds-unusable` uses `../genbank/synthetic.gb`, whose `alt11` reads V A W
  (`GTGGCTTGGTAA`), so residue 1 is valine; that CDS is refused for its
  `GTG` start either way (D042).
- **The digest** in each expected request is the sequence digest from
  `../genbank/synthetic-linear-record.tsv`, computed there with coreutils.
- **The grounding outcomes** follow the plan's §5.3a, applied by reading:
  for each case, whether each quote occurs in its own text, and whether the
  quote shows the number, the original and the target. The writing script
  asserted the occurrence half of that for all 24 cases — each quote a case
  calls grounded really is a substring of its text, and each quote a case
  calls fabricated really is not — so a typo in a quote could not pass
  unnoticed.
- **`design_exit`** follows from the exit-code table (`docs/cli.md` §5) and
  needs no new judgement: `0` where the edit is verified and every member
  designs, and `6` for `variants-combined`, whose `combined_construct` is
  outside the supported workflow (`unsupported_composition`, category
  `unsupported`). That `synA` is designable at all is the linear record's
  purpose and was confirmed once by hand, as
  `../genbank/README.md` records under "Fit for purpose".

- **The profiles** (`profile/*.json`) are user files of plan §5.4, written
  by hand. `full.json` states the corpus's one design — `luriaNative`,
  `q5_polymerase`, `minAnnealingOverlap` 15, `inline`, the `q5_reaction`
  buffer — with `"codon": "default"` and `"numbering": "from-cds-start"`;
  `no-codon.json` and `no-numbering.json` each leave one of those two out.
  That design is the same object every expected request states, and
  contract 6d's tests check that a profile's `design` reads exactly as a
  request's.

**No scientific value is recorded here.** The corpus holds no primer
sequence, no Tm and no annealing temperature; those live in `../q5-primer/`
and `../tm/` with their own independent sources. A case asserts only which
typed request the prose means, or which question or error it earns.

## The cases

**Proposed (5).** Prose with no token at all (`one-sub-prose`); the
three-letter form of the same edit, giving the same request
(`three-letter`); the product brief's own two-variant example
(`two-separate`); a combined construct, proposed faithfully so that core,
not the adapter, refuses it (`two-combined`, M6-16); and a codon named in
the text, so the profile needs none (`codon-stated`).

**Needs clarification (15).** Everything the adapter must ask about rather
than guess: an unstated composition; a CDS unnamed, not found, or unusable;
a deletion token, which M6 does not support; numbering unstated or stated as
something other than the CDS start; a codon unstated; no profile at all,
where every unstated value is asked about at once; a request that is not
mutagenesis, and one that mixes in an unsupported edit; and the four
evidence failures of §5.3a — a quote with no number, a `42` that is only
part of `420`, a quote that does not show the original residue, and one that
does not show the target.

**Failed (4).** A fabricated mutation and a fabricated CDS name, neither in
the user's text (`ungrounded_extraction`); and two replies that are not the
schema — an unknown field and a wrong type (`model_output_invalid`).

## Rerunning

There is nothing to regenerate: every file here is source, written by hand.
The record-derived values are checked by the GenBank verifier, which runs
under `tests/fixtures/verify.py`:

    python -I tests/fixtures/genbank/reference/verify_synthetic_linear.py --check
