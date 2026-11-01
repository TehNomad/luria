---
type: Research Note
title: "NEBaseChanger client-side investigation"
description: "A read-only investigation of the NEBaseChanger client, dated 2026-10-09."
status: stable
---
# NEBaseChanger client-side investigation

Status: research report. Read-only investigation. No Luria source, fixture,
dependency, branch, or configuration was modified. The only file created by
this task is this report.

Date of investigation: 2026-10-09 (UTC).
Investigator: Luria research task (Pi harness).
Target: the public NEBaseChanger web application (NEB) and its publicly
delivered client assets.

Evidence tags used below:

| Tag | Meaning |
| --- | --- |
| `[LIVE-OBS]` | Observed directly from the live site during this investigation. |
| `[ARCHIVE]` | Retrieved from a public web archive (Arquivo.pt) during this investigation. |
| `[BUNDLE-2.5.0]` | Read from the archived v2.5.0 client bundle `index.fc8d957d.js`. |
| `[BUNDLE-2.4.x]` | Read from the archived v2.4.x client bundle `index.5fcbb0b3.js`. |
| `[REPRO]` | Computed by executing the extracted archived v2.5.0 code in Node. |
| `[CAPTURE]` | Pre-existing captured NEBaseChanger output in the Luria repository. |
| `[LURIA-CODE]` | Read from the current Luria repository. |
| `[PRIOR]` | Claim from a prior research branch that this investigation did not independently re-verify. |
| `[UNKNOWN]` | Not established. |

Cloud-sharing warning (D023): this report cites primer and flanking sequences
derived from the real *C. elegans* SID-1 construct fixture, which has no
recorded public cloud-sharing approval. Do not send this report or its
sequence-bearing tables to a cloud model without explicit approval.

---

## 1. Scope and method

Goals addressed:

1. Determine whether primer design runs client-side or server-side.
2. Identify publicly delivered JS bundles, source maps, static data, and network
   requests.
3. Determine whether public client code contains identifiable primer-design
   algorithms, constants, or heuristics.
4. Document observable behaviour with representative input/output examples.
5. Produce actionable recommendations for Luria.

Method, and what was not done:

- The live site was probed with `curl` (`[LIVE-OBS]`). It is behind a Cloudflare
  challenge and returned HTTP 403; no interactive behaviour test was possible.
  No attempt was made to bypass the challenge, access controls, or rate limits.
- Public assets were retrieved from Arquivo.pt (`[ARCHIVE]`); the newest
  archived build is v2.5.0 (2024-06-11).
- The archived v2.5.0 bundle was read, its primer-design functions extracted,
  and the extracted code was executed unmodified in Node (`[REPRO]`).
- The result was anchored against the one pre-existing captured NEBaseChanger
  output in `tests/fixtures/nebasechanger-research/` (`[CAPTURE]`).
- A prior research branch (`research/polymerase-tm`) contains a related report,
  `docs/research/nebasechanger-polymerase-tm.md`. This investigation
  independently re-derived the architecture and algorithm from the archived
  bundle and confirms the core of that report; items not re-derived here are
  marked `[PRIOR]`.

Minification limitations: v2.5.0 ships one minified bundle. Variable names are
meaningless and some functions are only reachable from the Svelte component
tree. Algorithmic meaning was recovered from structure (string literals, DOM
help text, numeric constants) and validated by executing the code.

---

## 2. Architecture findings

**Finding: primer design executes entirely in the browser. There is no
server-side design endpoint.**

Evidence:

1. `ge.calcSDMPrimers` `[BUNDLE-2.5.0]` is a pure function of
   `(template, minLen, minTm, pad, wildTypeRegion, replacement)` that computes
   forward primer, reverse primer, and both Tm values. It calls
   `ge.createTmCalc` / `ge.tm` and the local Needleman–Wunsch aligner
   (`nn.alignDNAglobal`); none of these perform I/O.
2. The whole design pipeline is reachable from the bundle: codon selection
   (`Ee`, `Le`), Tm (`t.TmCalc`, `ge.createTmCalc`), SDM primer growth
   (`ge.calcSDMPrimers`), the inline builder `xe`, and the tail builder
   `autogenPrimers`/`autogenSDMPrimers` are all client objects.
3. The only network calls in the bundle are:
   - `fetch("/seqs/" + name)` — loads a built-in sample vector
     (`loadSampleVec`) `[BUNDLE-2.5.0]`.
   - `XMLHttpRequest` `GET`/`HEAD` in the file-download helper (Blob save and
     an availability check) `[BUNDLE-2.5.0]`.
   - `fetch(s.href)` in the Vite module-preload polyfill `[BUNDLE-2.5.0]`.
4. There is no `"POST"`, no `__data.json`, no `/api`, no WebSocket, no
   server-sent events, no `axios`, and no `sendBeacon` in the bundle
   `[BUNDLE-2.5.0]`.
5. The HTML shell loads `/assets/index.<hash>.js` and
   `/assets/index.<hash>.css` plus third-party analytics/consent scripts only
   (Cloudflare Insights, OneTrust) `[ARCHIVE]`.
6. The application stores nothing locally: no `localStorage`,
   `sessionStorage`, or `indexedDB` references `[BUNDLE-2.5.0]`.

Consequence for Luria: the archived bundle is a faithful record of the exact
code the browser ran for that build. Executing it is a valid substitute for
observing the corresponding live build, for as long as the live build matches
the archived one.

**Live-site status `[LIVE-OBS]`:**

- `GET https://nebasechanger.neb.com/` → HTTP 403,
  `cf-mitigated: challenge`, `server: cloudflare`.
- `GET https://nebasechanger.neb.com/assets/index.dfcf8292.js` → HTTP 404.
- Previously archived asset names
  (`index.fc8d957d.js`, `index.c71d257a.css`) → HTTP 404 on the live origin.
- `GET https://nebasechanger.neb.com/robots.txt` → HTTP 404.

The 404s for the old asset names are expected: the current build has a new
content hash. Because the live shell is Cloudflare-challenged, the current
bundle name could not be observed. The current live version is therefore
`[UNKNOWN]`, but it is a v2-era single-page application like the archived one.

---

## 3. Public assets, URLs, and hashes

All retrievals from Arquivo.pt. `id_` returns the raw object; the default
replay wraps it in a Wayback/pywb `wombat` shim and therefore has a different
digest.

| Asset | URL / snapshot | Bytes | SHA-256 |
| --- | --- | --- | --- |
| v2.5.0 bundle (raw) `[ARCHIVE]` | `https://nebasechanger.neb.com/assets/index.fc8d957d.js`; `https://arquivo.pt/wayback/20240611100829id_/...` | 291722 | `50cf872ef7cd9f7e046bd2163db43bf8477ded1caee03edfdfc8ea8acca2635b` |
| v2.5.0 bundle (replay-wrapped) `[ARCHIVE]` | `https://arquivo.pt/wayback/20240611100849js_/...` | 293961 | `759a3b0b822e6f6e15fe81b37c0911d367e6135a7cef1d3447796679c6113884` |
| v2.4.x bundle (raw) `[ARCHIVE]` | `https://nebasechanger.neb.com/assets/index.5fcbb0b3.js`; `https://arquivo.pt/wayback/20231121085439id_/...` | 284758 | `7691c7631cbef2da7929e76bf3322b6de7519ba7d485805de4b25b09a55a7578` |
| v2.4.x bundle (replay-wrapped) `[ARCHIVE]` | `https://arquivo.pt/wayback/20231121085439js_/...` | 286997 | `89d2b61a31f1d59afcd242d3909b43bb695fb984a1e65bfcb0fd08196a89c229` |
| v2.5.0 stylesheet `[ARCHIVE]` | `https://nebasechanger.neb.com/assets/index.c71d257a.css`; `https://arquivo.pt/wayback/20240611100829id_/...` | 27031 | `c71d257ac6579cbf08d0a0dcb6d9b27beb77ccd44dcf91098e5347e51c0beb53` |
| v2 SPA shell, 2024-06-11 `[ARCHIVE]` | `https://arquivo.pt/wayback/20240611100829id_/https://nebasechanger.neb.com/` | 1892 | `b37b5dee23e02795c874acfccb27becb191b2f5ce40f1f1b9690a7183a0beab7` |
| v2 SPA shell, 2023-11-21 `[ARCHIVE]` | `https://arquivo.pt/wayback/20231121085423id_/...` | 1893 | `80bf55ca4ab614595898c9695f7df1492121b178436accf3ff34829868226373` |
| v1 legacy shell, 2023-05-04 `[ARCHIVE]` | `https://arquivo.pt/wayback/20230504194028id_/...` | 12507 | `2d54cb76cd85e055533af1776212d089225ddac074b533da602a38dea4ee734c` |

Observations:

- The 2024-06-11 shell references `/assets/index.fc8d957d.js` and
  `/assets/index.c71d257a.css`; the 2023-11-21 shell references
  `index.5fcbb0b3.js`. The newest successful `nebasechanger.neb.com` snapshot
  is 2024-06-11; the next one, 2024-10-15, is already a Cloudflare 403.
- The shell embeds a build marker `"version":"2024.4.1"` (v2.5.0) and
  `"version":"2023.10.0"` (v2.4.x). These are build/deploy versions, not the
  tool's own changelog version.
- The v2.5.0 bundle carries an in-app changelog whose newest entry is
  **v2.5.0, April 16, 2024** `[BUNDLE-2.5.0]`. The `polymerase-tm` project
  targets `index.dfcf8292.js` v2.7.2; that file is not archived at Arquivo.pt
  `[ARCHIVE]` and is not retrievable live `[LIVE-OBS]`. All algorithm details
  below are v2.5.0.
- No `.js.map` source map was referenced by the shells or found in archive
  indexes `[ARCHIVE]`. The bundle is minified without a public source map.
- Static data loaded at runtime: `/seqs/<name>` sample vectors (for example
  pUC18, pUC19, pBR322, pET11c, pET21a) `[BUNDLE-2.5.0]`. No primer-design
  data is fetched.
- The retired v1 application (`sdmtool_v1_3_0.min.js`,
  `tmcalc_mm_v1_2_7.min.js`, `tmcalculator_v1_2_7.min.js`, jQuery/Biojs) is
  referenced by the 2023-05 shell, but those files are not archived at
  Arquivo.pt (`nebasechanger.neb.com/js/*` CDX query returned no rows)
  `[ARCHIVE]`. NEB documents v1 as retired on 2025-10-01 `[PRIOR]`.

---

## 4. Verified algorithm and biological details

### 4.1 Codon table and codon selection

`t.cut.Ecoli_K12` is a 65-row `[three-letter AA, codon, fraction]` table
`[BUNDLE-2.5.0]`. The table is byte-identical between v2.4.x and v2.5.0
`[ARCHIVE]`. Convenience wrapper `t.AMINO_ACID_OLC` maps one-letter to
three-letter codes (`X -> All`, `* -> Ter`).

- **Maximum codon usage** (`Ee`): filter `Ecoli_K12` by the requested
  three-letter code, stable-sort descending by fraction, take the first codon
  `[BUNDLE-2.5.0]`.
- **Maximum parsimony** (`Le`): filter by the requested code, stable-sort
  ascending by Hamming distance to the original codon, take the first
  `[BUNDLE-2.5.0]`. NEB help text: "a simple sequence distance is calculated as
  the required number of base changes to go from the original codon to the
  replacement codon" `[BUNDLE-2.5.0]`.
- Ties are documented by NEB as undefined `[PRIOR]`. In practice the code uses
  `Array.prototype.sort`, which is stable in modern JS engines, so a tie
  resolves to table order. Luria must not rely on that.

Maximum-usage preferred codon per amino acid, computed from the archived table
`[BUNDLE-2.5.0, REPRO]`:

```
A GCG   C TGC   D GAT   E GAA   F TTT   G GGC   H CAT   I ATT
K AAA   L CTG   M ATG   N AAC   P CCG   Q CAG   R CGC   S AGC
T ACC   V GTG   W TGG   Y TAT   * TAA
```

For the D404N fixture (original codon `GAT`, target Asn):

- maximum usage → `AAC` (fraction .55); captured NEBC output used `aac`
  `[CAPTURE]`.
- maximum parsimony → `AAT` (one base change vs two) `[REPRO]`.

Enumerating parsimony vs usage from `GAT` shows they differ for A, C, G, L, N,
P, R, S, T, V `[REPRO]`; this is a useful fixture axis.

### 4.2 Melting-temperature model

`ge.tm(seq, compseq)` creates `ge.createTmCalc({seq, ct: 0.5, salt: 150,
compseq})`, sets method 4, and returns the full-primer Tm `[BUNDLE-2.5.0]`.
Key values:

- primer concentration `ct = 0.5 µM` (500 nM);
- monovalent salt `150 mM`, divalent `0` (the Q5 condition);
- gas constant `R = 1.987`;
- method 4: nearest-neighbour duplex thermodynamics with initiation
  `dS = -5.7`, `dH = 200`; per terminal `A`/`T` `dS += 6.9`, `dH += 2200`;
  self-complementarity `dS += -1.4`;
- Owczarzy 2004 salt correction. With divalent = 0 the monovalent branch is
  used: `sc_ow = (4.29·fgc - 3.95)·1e-5·ln[Na+] + 9.4e-6·(ln[Na+])²`;
- `Tm = 1 / (1/Tm_1M + sc_ow) - 273.15`, minus `0.6·DMSO%` (DMSO default 0);
- the nearest-neighbour table is a 176-entry match-and-mismatch duplex table
  (`nndhds`), i.e. mismatch-aware; `calcTm_nomm` holds a separate pure
  SantaLucia path that `ge.tm` does not use.

**Mismatch handling.** `ge.calcSDMPrimers` computes the forward-primer Tm with
`compseq = revcomp(wild-type binding site)`, not the primer's own complement
`[BUNDLE-2.5.0]`. The SDM Tm is therefore mismatch-aware. This resolves a
prior open item `[PRIOR]`: the code path is mismatch-aware; the captured D404N
output (74) merely coincides with the matched value after rounding.

Executing the archived code `[REPRO]` gives, for the D404N forward primer
`CGTTGAGCAAAACTTCGGGGCGC`:

| Computation | Tm (°C) | Rounded |
| --- | --- | --- |
| `ge.tm` with wild-type complement (mismatch-aware; the real path) | 73.7841 | 74 |
| `ge.tm` with self complement (fully matched) | 74.3490 | 74 |
| `calcTm_nomm` (pure SantaLucia path, not used by `ge.tm`) | 74.2397 | 74 |

The earlier hand reconstruction reported 74.18 °C `[PRIOR]`; the residual
<0.6 °C difference comes from the actual code using the mismatch-inclusive
duplex table rather than the pure `dSSa` table. It does not change rounded
display or Ta.

### 4.3 SDM primer construction (`ge.calcSDMPrimers`)

Constants and clamping `[BUNDLE-2.5.0]`:

- `pad = 10` (fixed annealing bases on the 5' side of the mutation, and the
  initial 3' flank length);
- `minLen` is clamped up to 15 (`X < 15 ? 15 : X`);
- `minTm` is clamped to `[55, 72]`;
- GC-clamp extension threshold `63 °C`;
- Tm-balance tolerance `5 °C`;
- `Ta = min(TmF, TmR) + 1`, capped at 72 (raw floats, rounded for display).

Algorithm (inline, equal-length substitution):

1. Rotate the (circular) template so the mutation codon starts at index `pad`.
2. Globally align the replacement against the wild-type replaced region
   (Needleman–Wunsch, match `+2`, mismatch `-1`, gap `-2.5`, terminal `-2.5`,
   extend `-1`) to obtain aligned replacement/template strings for indels.
3. `fwd = L[0:pad] + alignedReplacement + L[pad+len(rep) : pad+len(rep)+O]`,
   `O` starting at `pad`. Grow `O` while `Tm < minTm`.
4. If `Tm < 63`, keep growing until the 3' base is `G`/`C`; if a GC terminus
   cannot be reached, revert to the shortest Tm-satisfying forward primer.
5. `rev = revcomp(L)[0:ce]`, `ce` starting at `minLen`. Grow while
   `Tm < minTm`, then the same GC-clamp/revert rule.
6. If `|TmF - TmR| > 5`, extend the lower-Tm primer toward the other, preferring
   a GC-clamped 3' end, reverting if it would overshoot.
7. `Ta = min(TmF, TmR) + 1`, cap 72. Displayed Tm/Ta are `Math.round`ed.

For substitutions the returned forward primer preserves the replacement in
lower case (because the aligner lower-cases input), which is exactly the mixed
case seen in the captured summary `[CAPTURE]`.

Related documented behaviour `[BUNDLE-2.5.0]`:

- Substitutions of ≤6 nt are placed inline in the forward primer with ≥10
  bases of annealing sequence flanking the inserted bases.
- Target regions longer than 6 nt (or when "Confine mutations to primer 5'
  tails" is set) are attached as 5' non-annealing tails and split between the
  two primers.
- Degenerate replacement codes are expanded combinatorially by
  `expandDNASeq` (Cartesian product over `DEGENERATES`) unless "preserve
  degenerate base codes" is set.
- Stop codons are selectable; `X`/`All` maps to `NNK` (changed from `NNN` in
  v2.4.0).
- The "old AA" is checked but a mismatch only produces a warning; processing
  continues.
- The "Use Reverse Complement" checkbox reverse-complements the entered
  sequence before use.
- Defaults: minimum primer length 15 nt (since v2.3.1), minimum Tm 55 °C
  (both adjustable since v2.5.0).

The bundle's in-app changelog documents v2.0.0 (2023-05-17) through v2.5.0
(2024-04-16), including the ambiguous-base expansion (v2.0.0), `X -> NNK`
(v2.4.0), preserve-degenerate (v2.4.2), and adjustable minimum Tm/length
(v2.5.0) `[BUNDLE-2.5.0]`.

---

## 5. Behavioural test cases and outputs

All rows were produced by executing the extracted archived v2.5.0 code on the
SID-1 D404N template with Q5 defaults `(pad 10, minLen 15, minTm 55)` `[REPRO]`,
unless noted. Only row 1 is anchored to a captured NEBC output `[CAPTURE]`; the
rest are archived-code reconstructions and are **not** live captures.

`GAT` is the wild-type codon at 1-based 1210–1212; the mutation context is
`...CGTTGAGCAA | GAT | TTCGGGGCGC...`.

| # | Case | Replacement | Forward primer | Len | TmF | Reverse primer | Len | TmR | Ta | Source |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | D404N, usage (default) — **captured NEBC output** | `aac` | `CGTTGAGCAAaacTTCGGGGCGC` | 23 | 74 | `AGCGGATTGAGCAGTTTTCCGTC` | 23 | 70 | 71 | `[CAPTURE]` + `[REPRO]` exact |
| 2 | Same, archived-code run | `AAC` | `CGTTGAGCAAaacTTCGGGGCGC` | 23 | 74 | `AGCGGATTGAGCAGTTTTCCGTC` | 23 | 70 | 71 | `[REPRO]` |
| 3 | Parsimony codon for N | `AAT` | `CGTTGAGCAAaatTTCGGGGCGC` | 23 | 73 | `AGCGGATTGAGCAGTTTTCCG` | 21 | 68 | 69 | `[REPRO]` |
| 4 | Single change at codon position 1 (N) | `AAT` | `CGTTGAGCAAaatTTCGGGGCGC` | 23 | 73 | `AGCGGATTGAGCAGTTTTCCG` | 21 | 68 | 69 | `[REPRO]` |
| 5 | Single change at position 2 (V) | `GTT` | `CGTTGAGCAAgttTTCGGGGCGC` | 23 | 70 | `AGCGGATTGAGCAGTTTTCC` | 20 | 66 | 67 | `[REPRO]` |
| 6 | Single change at position 3 (E) | `GAA` | `CGTTGAGCAAgaaTTCGGGGCGC` | 23 | 72 | `AGCGGATTGAGCAGTTTTCCG` | 21 | 68 | 69 | `[REPRO]` |
| 7 | Two changes at positions 1+2 (T) | `ACT` | `CGTTGAGCAAactTTCGGGGCGC` | 23 | 62 | `AGCGGATTGAGCAGTTTTC` | 19 | 63 | 63 | `[REPRO]` |
| 8 | Two changes at positions 1+3 (K) | `AAA` | `CGTTGAGCAAaaaTTCGGGGCGC` | 23 | 70 | `AGCGGATTGAGCAGTTTTCC` | 20 | 66 | 67 | `[REPRO]` |
| 9 | Two changes at positions 2+3 (A) | `GCG` | `CGTTGAGCAAgcgTTCGGGGCGC` | 23 | 65 | `AGCGGATTGAGCAGTTTTC` | 19 | 63 | 64 | `[REPRO]` |
| 10 | Stop codon | `TAA` | `CGTTGAGCAAtaaTTCGGGGCGC` | 23 | 65 | `AGCGGATTGAGCAGTTTTC` | 19 | 63 | 64 | `[REPRO]` |
| 11 | No change (synonymous re-expression) | `GAT` | `CGTTGAGCAAgatTTCGGGGCGC` | 23 | 74 | `AGCGGATTGAGCAGTTTTCCGTC` | 23 | 70 | 71 | `[REPRO]` |
| 12 | `minLen=10, minTm=40` (clamped to 15/55) | `AAC` | same as row 2 | 23 | 74 | same as row 2 | 23 | 70 | 71 | `[REPRO]` |
| 13 | `minTm=72` (Ta cap) | `AAC` | `CGTTGAGCAAaacTTCGGGGCGC` | 23 | 74 | `AGCGGATTGAGCAGTTTTCCGTCAGTATAG` | 30 | 72 | 72 | `[REPRO]` |
| 14 | Synthetic GC-rich 10-base flanks | `AAC` | `GCGCGCGCGCaacGCGCGCGCGC` | 23 | 88 | 88-mer (Tm-balanced to the 88 °C forward primer) | 88 | 82 | 72 | `[REPRO]` |
| 15 | Synthetic AT-rich 10-base flanks | `AAC` | `ATATATATATaacATATATATATACAGAAAGAGGCACAG` | 39 | 60 | `AGCGGATTGAGCAGTTTTC` | 19 | 63 | 61 | `[REPRO]` |

Notes on the table:

- Row 1 was independently reproduced by the archived code exactly (sequence,
  length, rounded Tm, Ta), which is the strongest available validation of the
  reconstruction.
- Rows 3–11 isolate codon-position effects on Tm and on the growth/balance
  loop. The forward primer's 5' 10-base flank and the reverse primer's 3' end
  stay anchored; only lengths and mismatch-driven Tm change.
- Rows 12–13 demonstrate the min-length/min-Tm clamping and the 72 °C Ta cap.
- Rows 14–15 demonstrate edge cases in Tm balancing. With GC-rich flanks the
  forward primer is already hot (88 °C), so the algorithm extends the reverse
  primer to 88 nt to stay within 5 °C. This is a design failure mode Luria
  should detect and warn about rather than reproduce silently.
- Rows 14–15 use synthetic flanking contexts and are not NEB outputs.

Not exercised: tail/confinement mode, insertions and deletions, >6 nt
replacements, degenerate-code expansion, batch/multi-site and NEBuilder modes,
linear/partial substrates, and the "Use Reverse Complement" toggle end to end.
These are listed as capture-needed in Section 8.

---

## 6. Comparison with the current Luria implementation

`[LURIA-CODE]` At the time of this investigation Luria's `crates/core`
implements sequence, feature, CDS, interval, translation, and mutation
(Milestones 1–2). It does **not** implement primer design, Tm, codon usage, or
a Q5 workflow.

| Capability | NEBaseChanger v2.5.0 (`[BUNDLE-2.5.0]`, `[REPRO]`) | Luria today (`[LURIA-CODE]`) | Gap / risk |
| --- | --- | --- | --- |
| Primer design | Inline substitution builder + tail builder, client-side | none | Milestone 3 work |
| Tm | SantaLucia/Allawi NN + Owczarzy salt, 150 mM, 500 nM, method 4 | none | Milestone 3 work |
| Ta | `min(Tm)+1`, cap 72 | none | Milestone 3 work |
| GC clamp / balance | extend below 63 °C; balance within 5 °C | none | Milestone 3 work |
| Codon policy | `max usage (E. coli K12)` and `max parsimony (Hamming)` | `Unspecified`, `Default` (first codon in standard table order T,C,A,G), `Supplied` | Different default; no usage/parsimony policy |
| Ties | undefined (stable sort in practice) | Luria's rules are deterministic | Luria must not import NEBC's non-determinism (D012) |
| Stop codons | selectable (`Ter` rows) | `Codon::create` and `parse_residue` refuse `*` | Deliberate divergence; decide explicitly |
| Degenerate replacement | combinatorial expansion, or preserve | not modelled | Decide explicitly (D005/D010) |
| Old-residue mismatch | warning; processing continues | must match, else fail (D011) | Deliberate divergence; keep |
| Strand / reverse complement | UI checkbox reverse-complements input | explicit strand model (D009) | Keep explicit; do not copy the toggle silently |
| Coordinates | 1-based inclusive, "between bases X-Y" anchors | zero-based half-open (D006) | Already handled by the D404N fixture |
| Circular substrate | rotate the circle so the codon sits after a 10-base pad | explicit topology/interval model | Reuse Luria's model; do not copy the rotation |
| Provenance | changelog version only | algorithm name/version and provenance (D021) | Luria is stronger; record the target NEBC version |

Directional differences worth stating explicitly:

- Luria's `CodonSelectionPolicy::Default` selects the first codon of the
  residue in published table order. NEBC's default is maximum E. coli K12 usage.
  These produce different DNA edits for many residue changes, so any
  NEBC-comparison fixture must pin the policy.
- Luria failing closed on a wrong expected original residue and on stop codons
  is the intended behaviour under D005/D011; NEBC is more permissive. This is a
  deliberate divergence, not a bug to fix.

---

## 7. Recommended next steps, ranked by expected value

1. **Write the Milestone 3 Q5 SDM task contract** and pin the target behaviour
   to "NEBaseChanger v2 inline-substitution semantics as observed in the
   archived v2.5.0 bundle, validated against the captured SID-1 D404N output",
   recording the algorithm id/version in provenance (D014, D021). Do this
   before coding.
2. **Implement Tm as a separate, independently testable domain function**
   (D003, D013): SantaLucia/Allawi nearest-neighbour duplex table, initiation
   `-5.7/200`, terminal `A/T +6.9/+2200`, `R=1.987`, Owczarzy monovalent salt
   at 150 mM, 500 nM primer, method 4. Add the three D404N Tm vectors
   (73.7841 mismatch-aware, 74.3490 matched, rev 70.0544) as regression tests.
3. **Add codon-selection policies** `MaxUsage(EcoliK12)` and
   `MaxParsimony` alongside the existing `Default`, with an explicit
   deterministic tie-break and a recorded `tie` outcome (D005, D012). Keep
   ties out of the silent path.
4. **Implement the inline substitution primer builder as an explicit
   deterministic state machine** (pad 10, growth to min Tm, GC clamp below
   63 °C, balance within 5 °C, `Ta = min+1` capped 72) with a documented
   termination guarantee and mismatch-aware Tm. Return lengths, Tm, Ta,
   annealing region, and any warnings.
5. **Add fixtures in the existing style** (`tests/fixtures/`, D018):
   - anchor fixture: the captured SID-1 D404N output (already present);
   - reconstructed-vector fixture: rows 2–13 of the table above, tagged
     `reconstructed-from-bundle-v2.5.0`, not as NEB ground truth;
   - codon-policy fixture: a parsimony-vs-usage table computed independently
     (Section 4.1).
6. **Handle strand explicitly** (D009): represent a reverse-strand input as a
   strand + reference rather than a hidden reverse-complement step, and add a
   test that a reverse-strand CDS yields the correct primers.
7. **Decide and document the intentional divergences**: stop-codon
   replacements, degenerate replacement handling, and wrong-original-residue
   behaviour. Luria should keep fail-closed semantics; record the NEBC
   behaviour for comparison.
8. **Do not** add `polymerase-tm`, the NEBC bundle, or any extracted copy as a
   dependency, port source, or test oracle (D002, D012, D014). The bundle is
   evidence; the algorithm must be re-implemented independently.
9. **Capture live observations before freezing fixtures** for the cases in
   Section 8; the current corpus has exactly one real NEB output.

---

## 8. Capture-needed cases and other limitations

### Limitations of this investigation

- The live site is Cloudflare-challenged (HTTP 403, `cf-mitigated: challenge`).
  No interactive test was run; nothing was bypassed.
- The newest archived bundle is v2.5.0 (2024-06-11). v2.7.2
  (`index.dfcf8292.js`) is not archived and the current build's asset name
  could not be observed. All algorithm details are v2.5.0.
- Only one captured end-to-end NEB output exists (`[CAPTURE]`). Rows 2–15 are
  reconstructions produced by executing the archived code; they are strong
  evidence for v2.5.0 behaviour but are not live observations.
- Minification prevents full recovery of some control flow (for example, the
  module-level `g` that `calcSDMPrimers` aligns against). The reconstruction is
  nevertheless anchored by exact reproduction of the captured D404N result.
- Tail/confinement, indels, >6 nt replacements, degenerate expansion,
  batch/multi-site/NEBuilder, and linear/partial substrates were not exercised.
- The v1 legacy application's JS is not archived, so no independent v1
  cross-check was possible.
- `[PRIOR]` claims re-used here (not independently re-derived): NEB
  documentation of tie non-determinism, v1 retirement, and the
  `polymerase-tm` comparison. They do not affect the v2.5.0 findings.

### Cases that need a human-run live capture before becoming fixtures

1. GC-rich and AT-rich flanking regions (real constructs, not synthetic).
2. Mutation near the circular origin / origin-spanning.
3. Linear or "partial" substrate input.
4. Minimum primer length 15 vs legacy 18; minimum Tm other than 55.
5. "Confine mutations to primer 5' tails" on and off.
6. Insertion ≤6 nt, insertion >6 nt (tail split), deletion of N nt.
7. Degenerate replacement expansion and "preserve degenerate" mode.
8. Stop-codon replacement and `X`/`All` → `NNK`.
9. Wrong "old AA" warning behaviour.
10. "Use Reverse Complement" end to end.
11. Multiple point mutations and batch/NEBuilder mode.

---

## 9. Confidence levels

| Finding | Confidence | Basis |
| --- | --- | --- |
| Primer design runs client-side; no design API | High | Bundle inspection; only `/seqs/` fetch and download XHR |
| v2.5.0 codon table and max-usage / parsimony rules | High | Bundle + executed policies |
| SDM Tm model (method 4, 150 mM, 500 nM, mismatch-aware) | High | Bundle + exact D404N reproduction |
| Inline builder (pad 10, min Tm 55, GC 63, balance 5, Ta cap 72) | High | Bundle + exact D404N reproduction |
| D404N output reproduced exactly | High | `[REPRO]` vs `[CAPTURE]` |
| Ta = min(Tm)+1 capped at 72 | High | Bundle + capture |
| Stop codons allowed; old-AA mismatch is a warning | High | Bundle + changelog/help text |
| Degenerate expansion semantics | Medium-High | Bundle `expandDNASeq`; not executed |
| Tail/indel/insert split behaviour | Medium | Bundle + help text; not executed |
| Tie-break behaviour | Medium | Code is stable-sort; NEB docs say undefined |
| Live site still behaves like v2.5.0 | Low | Live version not observable |
| v2.7.2 behaviour | None | Bundle not retrievable |
| v1 legacy behaviour | None | Assets not archived |

---

## Appendix A — Reproduction harness

The extracted code was executed with Node 26 using `vm`. Extracted verbatim
from `index.fc8d957d.js` (v2.5.0):

- the `TmCalc` IIFE (`var ti={};(function(l){...})(ti);const Ei=ti;`);
- `ge.trim`, `ge.revcomp`, `ge.comp`, `ge.dmsg`, `ge.createTmCalc`;
- the `nn` aligner module (`nn.alignDNAglobal`, `nn.Alignment`);
- `ge.tm` and `ge.calcSDMPrimers`.

For equal-length substitutions the aligner reduces to identity; the real
aligner was used regardless. The circular template was constructed as
`wt[codon0-10:] + wt[:codon0-10]` to match the model's `subSeq` circular
rotation (the template getter calls `subSeq(start, end+1)` with
`subSeq(e,n) = wseq.slice(e) + wseq.slice(0,n)` when `e >= n`).

These harness files live under `/tmp/luria-research/` and are working
artifacts only; they were not added to the repository and must not become a
Luria dependency.

## Appendix B — Key code excerpts (verbatim, minified)

Codon selection:

```
Ee=function(L){var X=ve.AMINO_ACID_OLC[L],te;return te=ve.cut.Ecoli_K12.filter(oe=>oe[0]==X),te.sort((oe,$)=>$[2]-oe[2]),te[0][1]}
Le=function(L,X){...X=X.toUpperCase();var te=ve.AMINO_ACID_OLC[L],oe=[],$=function(me,W){var O=0;O+=me[0]==W[0]?0:1;O+=me[1]==W[1]?0:1;O+=me[2]==W[2]?0:1;return O};...oe=ve.cut.Ecoli_K12.filter(me=>me[0]==te),oe.sort((me,W)=>$(me[1],X)-$(W[1],X)),oe[0][1]}
```

Tm call:

```
ge.tm=function(L,X){var te=.5,oe="Q5",$=150,me=4;X=X||ge.revcomp(L);
  var W=ge.createTmCalc({seq:L,ct:te,salt:$,compseq:X});
  switch(oe){case"Phusion":case"Phusion Hot Start Flex":me=1;break;default:me=4}
  W.setMethod(me);var O=W.Tm(),ce=O.tm;return ce}
```

Ta assignment in the inline replace path:

```
Xe.ta=Math.min(Xe.tmf,Xe.tmr)+1,Xe.ta>72&&(Xe.ta=72)
```

## Appendix C — Evidence sources

- Live site: `https://nebasechanger.neb.com/` (HTTP 403 Cloudflare challenge,
  2026-10-09).
- Arquivo.pt snapshots: 20240611100829, 20240611100849, 20231121085439,
  20231121085423, 20230504194028 (URLs in Section 3).
- Luria repository: `crates/core/src/mutation.rs`,
  `tests/fixtures/nebasechanger-research/...` (captured D404N),
  `tests/fixtures/README.md`, [docs/decisions/](/decisions/).
- Prior report (context only): `docs/research/nebasechanger-polymerase-tm.md`
  on branch `research/polymerase-tm`.
