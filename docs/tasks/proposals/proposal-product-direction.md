# Proposal: product direction (sequence first)

Status: **Draft for the owner, 2026-10-23. Not approved and not a task
contract.** If approved, it replaces the "Purpose", "Product thesis" and
"Subsequent milestones" parts of [docs/product/brief.md](/product/brief.md) and supersedes
D034's milestone order. Companion drafts: `proposal-decision-triage.md`,
`proposal-sequence-view.md` and `proposal-commands.md`.

## 1. The gap

A biologist who wants a point mutation or an indel today:

1. opens the construct in a sequence editor (SnapGene, Benchling,
   ApE),
2. finds the residue by scrolling, searching, or counting from the start
   codon,
3. copies a stretch of sequence around it,
4. pastes it into NEBaseChanger, finds the spot again in a bare linear
   fragment numbered from 1, and places the edit,
5. copies the primers into an order form.

Steps 3 and 4 are where time goes and mistakes happen. NEBaseChanger can't
read the construct, doesn't know the CDS or its residue numbering, and has
almost no way to search. Sequence editors know all of that but can't hand
off to NEBaseChanger.

## 2. Thesis

**luria opens your construct, lets you find the place you want to change,
and gives you Q5 primers for the edit, checked against the whole
construct.**

- **Sequence first.** The main screen is the sequence: both strands,
  translation, residue and base numbering, features. The user navigates
  it by residue, base, feature, DNA motif or protein motif. The edit is
  placed where the user is looking.
- **The whole construct is the template.** The user opens the plasmid
  file they already have. luria uses the sequence around the edit for
  the primers. The rest of the plasmid matters only for the rare chance
  of off-target priming, which luria can check and warn about.
- **Edits are checked, not trusted.** luria applies each edit, translates
  the edited construct, and shows the mutant protein next to the primers.
  These invariants stay as they are (see the triage draft).
- **Back to the user's tools.** Export the order table, plus the edited
  construct as GenBank with the primers as features, so the user's
  sequence editor stays the record.
- **Sensible defaults, shown.** Where a default can't silently give wrong
  biology, luria uses it, shows it, and lets the user change it. It asks
  only when a guess could land on the wrong base or residue.

## 3. Natural language

Natural language moves from the product's headline to a later
accelerator.

Its real value isn't parsing "A42G", which the shorthand already does
without a model. It's naming a residue the user can't number: "the
catalytic histidine", "the gatekeeper residue of the kinase domain", "the
second cysteine of the zinc finger". That needs a model with strong
knowledge of protein structure and domains (or tools such as domain
databases and structure alignment).

The sequence view is what makes this safe. The model proposes a location,
and luria **moves the cursor there and highlights it in the translated
sequence**, with the model's reasoning beside it. The user confirms what
they see before anything is designed. The model still writes no
coordinate into a request (D004); it suggests a place to look. This
extends `later-annotated-sites.md`. Grounding can come from record
annotations, a domain database or the model, and the user's confirmation
on screen is the control.

Until then, the existing `luria q5 interpret` stays as it is, frozen: no
new work.

## 4. Who it is for

Bench biologists and protein scientists making point mutants, small
indels and residue scans in plasmids they already hold as GenBank,
SnapGene or FASTA files. They should never need to know luria's request
schema, coordinate conventions or provenance model. Those stay available
for scripting and audit, behind the scenes.

## 5. Not goals

- A general sequence editor: no cloning simulation, map drawing, or
  annotation editing beyond what placing an edit needs.
- Ordering oligos.
- Replacing NEBaseChanger's own algorithm. luria's design is luria's own
  (`luriaNative`); the reference tool is a comparison, not a target.

## 6. Proposed milestone order (replaces D034 from Milestone 8)

| Milestone | Content |
| --- | --- |
| 8 | **Sequence view, read-only.** `luria FILE` opens a GenBank or FASTA file: strands, translation, numbering, features, go-to and search. Hand-tested by the owner before going further. Decide TUI versus web from that test. |
| 9 | **Place edits from the view.** Select, then substitute, delete, insert or replace; the shorthand as a command line; a list of variants; design and show primers on the sequence. Whole plasmids designable from the local sequence (triage D035). ORF detection for FASTA files without annotation. |
| 10 | **Export back.** Edited constructs as GenBank with primer features; order table (exists); SnapGene `.dna` import if the owner wants it. |
| 11 | **Off-target check.** Warn when a primer's 3′ end also matches elsewhere in the construct. |
| later | Web interface; natural language that names residues by structure or domain (section 3). |

`luria new` and `luria review` keep working until milestone 9, which
replaces them (`proposal-commands.md` §4).

## 7. Questions for the owner

1. TUI first and decide after the milestone-8 hand test, or commit to
   one now?
2. How important is SnapGene `.dna` import (proprietary but documented
   format) compared with GenBank export from SnapGene?
3. Combined mutants (two edits in one construct) are refused by core
   today. Are they needed in milestone 9 or later?
