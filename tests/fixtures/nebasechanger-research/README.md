# NEBaseChanger / polymerase-tm research fixtures (PRELIMINARY)

Status: **research evidence only.** These are **not** Luria's final Q5 fixtures
and they do **not** define the Milestone 3 contract. They preserve controlled
observations so the contract can be designed and independently verified later.
See `docs/research/nebasechanger-polymerase-tm.md`.

The existing `tests/fixtures/example-construct/` remains the home of the one
captured NEBaseChanger output (SID-1 D404N). This directory adds the reconstructed
expectation, the codon-usage reference, and a controlled comparison table.

## Contents

| File | Purpose |
| --- | --- |
| `cases.tsv` | Controlled comparison matrix (one row per case). |
| `sid1-d404n-reproduction.json` | Independent reconstruction of the captured NEBC D404N result and the `polymerase-tm` output side by side. |
| `ecoli-k12-codon-usage.tsv` | The exact E. coli K-12 codon table extracted from the NEBaseChanger v2.5.0 bundle, with max-usage and parsimony columns. |
| `provenance.md` | Exactly where every expectation came from, and what is still unverified. |

## Evidence classification

Each expectation is tagged with its source:

- `NEB-DOC` — documented by NEB.
- `NEBC-JS-2.5.0` — read from the archived NEBaseChanger front-end bundle
  `index.fc8d957d.js` (v2.5.0, archived 2024-06-11 by Arquivo.pt).
- `NEBC-OBS` — captured NEBaseChanger tool output.
- `RECON` — our independent Python reconstruction of the extracted algorithm.
- `PTM-CODE` — output of the `polymerase-tm` code (not assumed correct).
- `INFERENCE` — our interpretation.
- `UNKNOWN` — not established.

## Rules for using these fixtures

1. Do **not** accept an expectation because `polymerase-tm` produced it.
2. Do **not** treat `polymerase-tm` as an oracle. It diverges from the recovered
   NEBaseChanger algorithm (see through-line in `cases.tsv`).
3. The reconstruction is validated against exactly **one** captured tool output.
   More end-to-end captures are required before these become regression fixtures.
4. Coordinates follow Luria conventions (`D006`): zero-based, half-open. The
   captured NEBC strings use 1-based positions; conversions are shown explicitly.

## Cloud sharing

`cases.tsv` and the reproduction JSON reference the real *C. elegans* SID-1
construct sequence/primers already present in `example-construct/`, which has no
public cloud-sharing approval. Do not send these files to a cloud model without
explicit approval (`D023`). The codon table and JS-derived constants may be
shared; they are public.
