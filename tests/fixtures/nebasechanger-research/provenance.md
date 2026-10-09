# Provenance of the NEBaseChanger research fixtures

This file records where each expectation came from and what remains unverified.
It is deliberately conservative: one captured NEBaseChanger output is not enough
to freeze a contract.

## Sources used

| ID | Source | Retrieved | Notes |
| --- | --- | --- | --- |
| `NEBC-JS-2.5.0` | `https://nebasechanger.neb.com/assets/index.fc8d957d.js` via Arquivo.pt snapshot `20240611100829` | 2026 research pass | NEBaseChanger v2.5.0 (its own changelog ends at v2.5.0, 2024-04-16). |
| `NEBC-JS-2.4.x` | `https://nebasechanger.neb.com/assets/index.5fcbb0b3.js` via Arquivo.pt snapshot `20231121085439` | 2026 research pass | Earlier bundle (v2.4.x). |
| `NEBC-OBS` | `tests/fixtures/example-construct/raw/sdm*` | pre-existing | Captured tool output for SID-1 D404N. |
| `RECON` | Independent Python reimplementation of the extracted `calcSDMPrimers` + Tm method 4 | 2026 research pass | Reproduces the captured D404N output exactly. |
| `PTM-CODE` | `https://github.com/Dioskurides/polymerase-tm` @ v2.3.2 | 2026 research pass | Run with a stub for the optional `primer3` risk check. |
| `NEB-DOC` | NEB site, FAQ, manuals, tutorial, NEBaseChanger help | via search indexes | Live pages returned HTTP 403; text obtained from search-engine caches/snippets. |

## What is verified

1. The exact E. coli K-12 codon table used by NEBaseChanger's "maximum codon
   usage" (`NEBC-JS-2.5.0`), and that `polymerase-tm` selects the same preferred
   codon for every amino acid.
2. Maximum parsimony = minimum Hamming distance to the original codon; ties are
   explicitly undefined in NEBaseChanger (`NEBC-JS-2.5.0`, `NEB-DOC`).
3. The NEBaseChanger SDM Tm model: SantaLucia 1998 NN, initiation `dS=-5.7`,
   `dH=200`, terminal A/T `+6.9`/`+2200`, Owczarzy salt correction at 150 mM
   equivalent monovalent, 500 nM primer, `R=1.987`
   (`NEBC-JS-2.5.0`). This reproduces the captured D404N Tms of 74 and 70
   (`RECON`, `NEBC-OBS`).
4. `Ta = min(Tm_fwd, Tm_rev) + 1`, capped 72 (`NEBC-JS-2.5.0`, `NEBC-OBS`).
5. The inline-substitution primer builder: pad 10, grow to min Tm, GC-clamp
   below 63 °C, balance within 5 °C (`NEBC-JS-2.5.0`, `RECON`).
6. `polymerase-tm` does not reproduce the captured D404N result
   (`PTM-CODE`).

## What is NOT verified

- NEBaseChanger **v2.7.2** specifically (bundle `index.dfcf8292.js`, cited by
  `polymerase-tm`) is **not** archived at Arquivo.pt. The reconstruction is
  v2.5.0. The core logic appears stable across v2.4.x–v2.5.0, but a later change
  cannot be excluded.
- The current live NEBaseChanger: every live fetch returned HTTP 403
  (Cloudflare). No interactive experiments were run.
- Whether NEBaseChanger's SDM Tm uses the mismatch-aware path or the
  no-mismatch path for display. The captured D404N Tms match the no-mismatch
  method-4 values; the code passes a wild-type complement that would trigger
  mismatch handling. Open item.
- GC-rich / AT-rich regions, boundary/origin cases, linear/partial substrates,
  tail confinement, degenerate expansion, indels, and multi-site behaviour.
- The magnitude of the difference between `polymerase-tm`'s bivariate 50 mM/2 mM
  Tm and NEBC's 150 mM monovalent Tm on arbitrary primers. Only one primer pair
  was checked.

## Coordinate conversion note

NEBC reports 1-based positions and describes substitutions with flanking anchors
("between bases 1209-1213"). The captured FASTA header states the inclusive
codon range `1210 to 1212`. Luria's canonical interval is zero-based half-open
`[1209, 1212)`. The existing `example-construct/` fixture already documents this
in detail; see `tests/fixtures/README.md`.

## Reproducing

The reconstruction scripts and the archived JS used are working artifacts under
`target/research/` (git-ignored). They are not part of Luria and must not become
a dependency. The algorithm is fully described in
`docs/research/nebasechanger-polymerase-tm.md`, Sections 3, 7 and Appendix B.
