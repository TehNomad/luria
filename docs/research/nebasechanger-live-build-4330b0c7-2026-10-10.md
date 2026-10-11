# NEBaseChanger live build `index.4330b0c7.js` — **v2.11.0** (2026-10-10)

Research only. No Luria production code, no Milestone 3 code.
Capture: `tests/fixtures/nebasechanger-research/live-capture/` (Playwright run
`capture-js.mjs`, `capturedAt 2026-10-10T19:29:19.309Z`, `startedAt 19:29:01.852Z`).

> **Correction to earlier working notes.** The beautified fixture was
> previously named `index.4330b0c7.v2.9.0-live.beauty.js`. It is **v2.11.0**.
> Renamed to `live-js/beautified/index.4330b0c7.v2.11.0-live.beauty.js`.
> `4330b0c7` is the Svelte asset **content hash in the URL**, not a version.

## 1. Capture inventory

`manifest.json` keys: `capturedAt, startedAt, page, browser, outcome, assets,
excluded, failures, sanitization`. `browser.mode: persistent-profile`,
`headless: false`, `profileDir: browser-profile`.
`outcome.appReady: true`, `outcome.challengeSeen: false`,
`outcome.navigationError: null`, `outcome.safeErrors: []`, `failures: []`.

**The manifest indicates the real application loaded**: `appReady true`,
`challengeSeen false`, no navigation error, and the app bundle was fetched
200/`application/javascript`.

### Manifest: 6 assets + shell (9 files on disk)

| # | URL | file | HTTP | content-type | bytes | sha256 (12) | role |
|---|---|---|---|---|---|---|---|
| 0 | `https://nebasechanger.neb.com/assets/index.4330b0c7.js` | `004eae75971a-index.4330b0c7.js` | 200 | application/javascript | 401,373 | `004eae75971a` | **application bundle** |
| 1 | `https://geolocation.onetrust.com/cookieconsentpub/v1/geo/location` | `4da8a6638ad7-location` | 200 | application/json | 71 | `4da8a6638ad7` | OneTrust geo lookup (not JS) |
| 2 | `https://www.googletagmanager.com/gtm.js?id=GTM-PQVCKDF` | `ca06b0749147-gtm.js` | 200 | application/javascript; charset=UTF-8 | 359,844 | `ca06b0749147` | Google Tag Manager |
| 3 | `https://www.googletagmanager.com/gtag/js?id=G-V6Q0SZ7FRP&cx=c&gtm=4e6a72` | `cedfbda94d5d-js` | 200 | application/javascript; charset=UTF-8 | 543,080 | `cedfbda94d5d` | Google Analytics gtag |
| 4 | `https://nebasechanger.neb.com/analytics.js` | `5f43d215d44c-analytics.js` | **404** | text/html | 916 | `5f43d215d44c` | **error page, not JS** |
| 5 | `https://nebasechanger.neb.com/gtm.js` | `5f43d215d44c-gtm.js` | **404** | text/html | 916 | `5f43d215d44c` | **error page, not JS** |
| — | `https://nebasechanger.neb.com/assets/index.4d8eac54.js` | `5f43d215d44c-index.4d8eac54.js` | **404** | text/html | 916 | `5f43d215d44c` | **error page, not JS — ORPHAN** |

`shell.html`: 197,698 B, sha256 `90ff7a0a3c3bf1423d0bb3e832e6f7b1cd15a9bd1dd4d621b02746baa14e7965`.

**Capture-tool inconsistency (real bug in `capture-js.mjs`):** the directory holds
7 asset files but the manifest's `assets` array has only 6 entries.
`5f43d215d44c-index.4d8eac54.js` is on disk but is **not** listed in `assets`,
`excluded`, or `failures`. Verified by set difference. The file is byte-identical
to the other two 404 bodies, so it does not affect the analysis, but the manifest
under-reports the capture and any completeness claim must be made against the
directory listing, not the manifest alone.

### Completeness against the shell

`outcome.shell.refs` (8 entries):

| ref | status |
|---|---|
| `/assets/index.4330b0c7.js` | **captured** (asset 0) |
| `/assets/index.3bc63f16.css` | not captured — CSS, out of scope |
| 5 × `/assets/*.svg` (NEB logo, appMainLogo, about/help icons) | not captured — images |
| `/cdn-cgi/challenge-platform/scripts/jsd/main.js` | **deliberately excluded** (Cloudflare challenge machinery) |

**Every same-origin JavaScript asset referenced by the captured shell was obtained,
except the Cloudflare challenge script, which the tool excludes by policy.**
The bundle's own external script reference (`otSDKStub.js`, OneTrust) is also
excluded by policy. No chunks, no workers, no source maps, no truncated bodies.

### Session material — exposure

- `manifest.json` records **no** request/response headers, cookies, clearance
  tokens, consent values or POST bodies. Its own `sanitization` block states this
  and lists `fieldsRecorded: [url, file, status, contentType, resourceType, bytes,
  sha256, capturedAt, kind, looksLikeJavaScript, classifyReasons]` and the
  `checkedMarkers` it scanned for. Verified by reading the manifest: no `cookie`,
  `set-cookie`, `cf_clearance`, `authorization`, or user-agent field is present.
- **`browser-profile/` is a real Chrome profile** (`Default/`, `Local State`,
  `BrowserMetrics`, `component_crx_cache`). It contains cookies, history and
  profile state. It **is** git-ignored (`…/.gitignore:1 browser-profile*/`).
- **Gap:** `live-capture/manifest.json` and the asset blobs are **not** ignored by
  that file — `git check-ignore` returns rc=1 for the manifest, and
  `git status --short` shows `?? tests/fixtures/nebasechanger-research/` as
  untracked-but-addable. The manifest itself is clean, so the risk is low, but a
  future `git add -A` would commit the whole fixture tree. Recommend adding
  explicit ignore rules or an explicit allow-list before any commit.
- The 404 HTML bodies and the OneTrust/GTM blobs may carry consent identifiers;
  not reproduced here.

## 2. Prior research: verified paths

| Path | Status on this branch (`research/nebasechanger_reverse_engineer`) |
|---|---|
| `docs/research/nebasechanger-polymerase-tm.md` | **MISSING** — exists only on branch `research/polymerase-tm` (head `b71c0e7`, 2026-10-09), 44,311 B |
| `docs/research/nebasechanger-har-and-builds-2026-10-10.md` | present, 25,171 B |
| `docs/research/nebasechanger-client-investigation.md` | present, 29,989 B (not in the task list) |
| `docs/research/nbc_help_v284.txt` | present, 22,088 B (not in the task list) |
| `tests/fixtures/nebasechanger-research/README.md` | **MISSING** — on `research/polymerase-tm` only |
| `…/cases.tsv` | **MISSING** — on `research/polymerase-tm` only (4,017 B, 19 cases) |
| `…/provenance.md` | **MISSING** — on `research/polymerase-tm` only (3,959 B) |
| `…/sid1-d404n-reproduction.json` | **MISSING** — on `research/polymerase-tm` only (2,747 B) |
| `…/ecoli-k12-codon-usage.tsv` | **MISSING** — on `research/polymerase-tm` only (referenced by README, not in the task list) |
| `tests/fixtures/example-construct/` (SID-1 D404N ground truth) | **MISSING — deleted from this branch** (`git status` shows 6 ` D` entries). Present on `main`, `research/polymerase-tm`, `fixture/nebasechanger-sdm`, `research/nebasechanger-fixtures` |
| `…/app-js/index.a5da1636.v2.8.4.js` | present, 395,491 B |
| `…/app-js/index.dfcf8292.v2.7.2.js` | present, 375,662 B |
| `…/app-js/index.17c7600d.v2.5.3.js` | present, 292,233 B |
| `…/app-js/index.*.v2.5.0.js` | **MISSING** — README cites v2.5.0 as `index.fc8d957d.js`; no `fc8d957d` file exists anywhere in the repo |
| `…/app-js/index.*.v2.5.0` under any other hash | **MISSING** — no v2.5.0 bundle on disk |
| `…/nebasechanger-js/` | present — the **failed** 2026-10-09 run (Cloudflare challenge assets only) |
| `…/live-capture/` | present — this capture |
| `…/raw/`, `…/diffs/`, `…/observations/aa-substitutions/` | present |

Prior conclusions are treated as hypotheses. Two are re-tested below and both
survive with corrections; one open item is now resolved (§5).

## 3. Build identity

Strongest defensible identification, each independently greppable:

1. **`v2.11.0`, dated `October 10, 2026`.** The bundle embeds the NEBaseChanger
   "Updates" changelog as HTML. The `<dl>` is newest-first; its first `<dt>` is
   `v2.11.0 October 10, 2026`. 30 entries total.
2. **v2.8.4's changelog has 27 entries and tops out at `v2.8.4 July 10 2026`.**
   The live bundle adds exactly three: `v2.9.0 October 8, 2026`,
   `v2.10.0 October 9, 2026`, `v2.11.0 October 10, 2026`.
3. **Content hash `4330b0c7`** is the URL asset hash
   (`https://nebasechanger.neb.com/assets/index.4330b0c7.js`); the file's own
   sha256 is `004eae75971aa84ac5325eea7a7a663ffa622e2eb8756837ac0a09c5d475dfb0`.
4. **Served metadata** (manifest asset 0): `status 200`,
   `contentType application/javascript`, `bytes 401373`.
5. **The live bundle is the newest build we hold**, captured 2026-10-10T19:29:02Z,
   nine days after its own release date. It is **not** "current because it was
   captured recently" — it is current because its embedded changelog has no entry
   after 2026-10-10.

**Not** version evidence: the NEB help-document resource paths and the SVG/logo
asset names are byte-identical across builds; the `4d8eac54` reference is a
*dangling* asset link in the shell (404), not a build marker.

## 4. Diff v2.8.4 → v2.11.0

| Metric | v2.8.4 | live v2.11.0 |
|---|---|---|
| bytes | 395,491 | 401,373 (**+5,882**) |
| beautified lines | 20,444 | 20,126 (**−318**) |
| `console.log` call sites | **480** | **0** |
| `X.prototype.M=function` bodies | 71 | 69 (2 removed) |
| `X.M=function` assignments (distinct property names) | 132 | 164 (**+32**) |
| methods identical modulo renaming | 13 byte-identical, 29 rename-only | — |
| methods identical modulo renaming **and** log-stripping | 21 | — |
| methods with a genuine token-level change | **6** | — |
| property names added in live | **37** | — |
| property names removed in live | **5** | — |

Method-level classification was produced by a rename-blind, log-stripping,
trailing-comma-normalising token comparator (`@` replaces every non-keyword
identifier; numeric and string literals are preserved). Formatting-only
differences (`{ … }` vs bare statement, `var a;x;` vs `var a,x;`, redundant
parentheses) are reported separately from token changes.

### 4.1 The six genuine method-level changes

| Method | Change |
|---|---|
| `Primer.calculatePrimers` | **bounds guards added** (see §6). Loop bodies wrapped in blocks; `len < seqLen` added to every grow condition; `throw new Error("cannot reach min Tm " + minTm + " for forward/reverse primer within " + seqLen + " bp")` inserted at the top of the grow-to-minTm loops; `&& !(len >= seqLen)` added to the two Tm-balance loops. |
| `Fragment.autogenPrimers` | (a) options object is now defensively copied: `opts = Object.assign({}, opts)` before defaults; (b) forward-primer end-coordinate wrap guard changed `m > _ → m -= _` to `v >= _ → v -= _` (off-by-one: a primer ending exactly at the sequence end now wraps). |
| `Primer.comp` / `Primer.revcomp` (×3 variants) | complement table value string changed: `TGCAAMKWSYRVHDBN…` → `TGCAAMKSWYRVHDBN…`. At key index 7 (`S`) the mapping changes `S→W` to `S→S`; at index 8 (`W`) it changes `W→S` to `W→W`. |
| `isValidAASeq`, `isValidNASeq` | `regex.lastIndex = 0` inserted before `.test()` — fixes the shared-`RegExp.lastIndex` corruption bug (this is the changelog's "primer table sometimes freezing after an invalid character was typed"). |
| `Primer.translate` | live adds a pre-pass `seq = f(new Seq(seq))` and **removes** the ambiguous-codon collapse `x[0] === x[1] ? x[0] : "X"`. |
| `parseSeqFile` | substantially rewritten: adds `format`/`raw`/`fasta`/`genbank` metadata to every record, bounds-checked loops, `ORIGIN`-section search with `throw new Error("GenBank entry has no ORIGIN section")`, and a `>`-count preamble scan. |

Formatting-only (no token change after normalisation): `Alignment.getBasesAtS1Coord`,
`getBasesAtS2Coord`, `Fragment.setRange`, `rebuildLeftRE`, `rebuildRightRE`.
Debug-only: `dmsg` body emptied (`typeof window.console … → {}`), `onerror` emptied,
`DialogManager.forceRegisterDialog` guard restructured (third-party `<dialog>` polyfill).

### 4.2 Removed in live (5)

`buildPairMap`, `calcTm`, `calculateSDMPrimers`, `buildFrag`, `mergeAlignments`.
`TmCalc.prototype.calcTm` (4,483 B) and `TmCalc.prototype.buildPairMap` (1,277 B)
were the **class-level** mismatch-aware TM path. The mismatch-aware path still
exists in live, but only as the **closure** `Tm` built inside `createTmCalc`
(see §5). `Ne.tm()` calls that closure, never `calcTm_nomm`.

### 4.3 Added in live (37 property names)

Two coherent groups:

- **Mutation/operation editing + validation**: `applyMutation`, `batchLineError`,
  `buildAction`, `canonicalFlanks`, `checkCodonOp`, `classifyOp`, `codonAt`,
  `codonOpError`, `codonRangeError`, `duplicateOpsWarning`, `escapeRegExp`,
  `flankError`, `matchesIUPAC`, `mergeCodonOps`, `nebuilderBlocker`,
  `nebuilderFragments`, `normalizeFlanks`, `normalizeOpText`, `notIntegerError`,
  `orfStarts`, `positionError`, `replacementError`, `sequenceLengthError`,
  `stripReplacement`, `verifyOp`, `verifyPair`.
- **Output/serialisation**: `buildFasta`, `describeOp`, `downloadName`,
  `finalSeqRecord`, `primerId`, `primerList`, `primerSeq`, `shorten`,
  `summary`, `tsvCell`, `wrap`.

`matchesIUPAC` is new and is the degenerate-flank matcher used by the new
validation layer. `wrap` implements the changelog's "FASTA files are wrapped at
70 characters".

## 5. Temperature model (live v2.11.0)

Call chain, verified by extraction:

```
Ne.tm(seq, compseq)
  ct = 0.5, salt = 150, method = 4            (polGroup "Q5"; "Phusion"/"Phusion Hot Start Flex" → 1)
  compseq ||= Ne.revcomp(seq)
  → Ne.createTmCalc({seq, ct: 0.5, salt: 150, compseq}).Tm()
```

`createTmCalc` normalisation (live):

```js
seq  = trim(seq).toLowerCase().replace(/\s/g, "").replace(/u/g, "t")   // ← .replace(/u/g,"t") is NEW in live
comp = compseq ? trim(compseq).toLowerCase().replace(/s/g, "")
                : revcomp(seq)
sym  = (seq === comp)
R = 1.987 ; primerc = ct*1e-6 ; saltc = salt*1e-3
```

Method 4 (`calcTm_nomm` and the closure `Tm` share this structure):

> **CORRECTION (2026-10-10, cycle 1 — see
> `docs/research/nebasechanger-cycles-1-4-2026-10-10.md` §1.3/§1.4).** The two
> engines do **not** share this structure. The SDM path (`createTmCalc` closure
> `Tm`, reached from `Ne.tm`) uses initiation `ΔS = −5.7`, `ΔH = 200` and
> `+6.9 / +2200` per terminal A/T, with **no** G/C terminal term. The block
> below, with `+4.1/+2300` and `−2.8/+100`, is `TmCalc.prototype.calcTm_nomm`
> (the NEBuilder route via `createTmFunction`), which is a different engine.
> Verified verbatim against `live-capture/004eae75971a-index.4330b0c7.js` @
> 76574..87147 (closure) and @ 45211 (`calcTm_nomm`). Also corrected there:
> `calcSDMPrimers` never reads `primerMaxTm`; its GC-clamp target is a
> hardcoded `63` °C in all four builds.

```
ds += sym ? -1.4 : 0
leading  a/t → ds +4.1, dh +2300      leading  g/c → ds -2.8, dh +100
trailing a/t → ds +4.1, dh +2300      trailing g/c → ds -2.8, dh +100
for each bigram p of seq: ds += dSSa[p]; dh += dHSa[p]
dh *= 1e3
Tm  = (dh + endAdj) / (ds + symAdj + init + R*ln(ct*1e-6))
Tm  = 1 / (1/Tm + sc_ow)                       // method 4 only
sc_ow = (4.29*fgc - 3.95)*1e-5*ln(saltc) + 94e-7*ln(saltc)^2
Tm -= 273.15
Tm -= dmso * 0.6
```

`ct = 0.5` → `5e-7` M (500 nM primer); `salt = 150` → `0.15` M (150 mM monovalent).
This matches the `NEBC-JS-2.5.0` constants recorded in `provenance.md`
(`R = 1.987`, 150 mM, 500 nM) — **the salt correction and initiation constants did
not change from v2.5.0 to v2.11.0.**

### 5.1 TM numeric changes v2.8.4 → live (complete list)

Pair-table entries were compared by key across all 176 `"xx/yy":{dh,ds}` entries in
`createTmCalc`; exactly three differ:

| table | key | v2.8.4 | live v2.11.0 |
|---|---|---|---|
| `nndhds` | `aa/tc` | `dh -9.1, ds -24` | `dh 2.3, ds 4.6` |
| `nndhds` | `ct/aa` | `dh -9.1, ds -24` | `dh 2.3, ds 4.6` |
| `nndhds` | `ta/at` | `dh -7.2, ds -20.4` | `dh -7.2, ds -21.3` |
| `nnbulge` | `25` | `dh 0, ds -28.1` | `dh 0, ds -18.1` |

`nnbulge[25]` sits in the monotone run `…20: -17.1, 25: ?, 30: -19`; `-28.1` is an
obvious typo for `-18.1`. The `aa/tc`/`ct/aa` pair had **identical** (copied) values
in v2.8.4 and both were replaced. These four edits correspond exactly to the v2.9.0
changelog line *"Corrected melting temperatures (TA/AT and A·C mismatch nearest-neighbor
values)"*.

### 5.2 Degenerate-base handling changes

`DEGENERATES` (identical in every other respect):

| code | v2.8.4 | live v2.11.0 | IUPAC DNA |
|---|---|---|---|
| `B` | `AGT` | `CGT` | `[CGT]` — live correct, v2.8.4 wrong |
| `D` | `ACT` | `AGT` | `[AGT]` — live correct, v2.8.4 wrong |
| `S` | `CG` | `CG` | `[CT]` — **still wrong in live** |
| `H` | `ACT` | `ACT` | `[ACT]` correct |
| `V` | `ACG` | `ACG` | `[ACG]` correct |

Complement table (`comp`/`revcomp`, key string `ACGTUKMSWRYBDHVNacgtukmswrybdhvn-`):

| key | v2.8.4 | live v2.11.0 | correct DNA complement |
|---|---|---|---|
| `S` | `W` | `S` | `R` — **live still wrong** |
| `W` | `S` | `W` | `W` — live correct |

All other 14 mappings are unchanged and correct (`M↔K`, `R↔Y`, `B↔V`, `D↔H`, `U→A`).
`expandDNASeq` is identical modulo renaming.

**These are the only two degenerate-base defects I could confirm as still present in
the live build.** They matter for independent implementation: NEBaseChanger v2.11.0
is *not* IUPAC-correct for `S`.

## 6. Primer search strategy (live v2.11.0)

### 6.1 Inline substitution / SDM: `calcSDMPrimers`

Signature (live, 8 params; v2.8.4 had 6):
`calcSDMPrimers(template, minLen, minTm, mutStart, newStr, oldStr, avail3p, avail5p)`

Call site (live): `Ne.calcSDMPrimers(u.template, oe, ge, St, X, Ye, gt, hn)` where
`gt`/`hn` are `Infinity` for circular templates and
`seqLen - mutationLen` / `mutStart + 1` for linear ones.

Clamps and setup:

```js
minLen  = max(minLen, 15)
minTm   = clamp(minTm, 55, 72)
avail3p = min(avail3p, Infinity)   // default Infinity
avail5p = min(avail5p, Infinity)
Pe      = mutStart + newStr.length
avail3p = min(avail3p, template.length - Pe)
avail5p = min(avail5p, template.length)
if (minLen > avail3p) throw "cannot reach min primer length …: only N bases available 3' of the mutation"
if (minLen > avail5p) throw "cannot reach min primer length …: only N bases available 5' of the mutation"
```

Alignment of old→new (`St = 10` short-circuit threshold):

```js
if (|old| == |new| && |old| < 10 && /^[acgt]+$/i.test(old) && /^[acgt]+$/i.test(new))
    aln = { alnSeq1: old.toLowerCase(), alnSeq2: new.toLowerCase() }
else
    aln = alignDNAglobal(new Alignment("global", old, new, {mat, gap:-2.5, term:-2.5, extend:-1}))
```

Forward primer construction (`wt = revcomp(template)`):

```
pfwd = template[0 .. mutStart] + aln.alnSeq1 + template[Pe .. Pe+ge]
tmf  = tm(pfwd, revcomp(template[0..mutStart] + aln.alnSeq2 + template[Pe .. Pe+ge]))
```

Reverse primer: `prev = revcomp(template)[0 .. xe]`, `tmr = tm(prev)`.

Grow loops, in order, with the **new** guards:

1. **grow forward to minTm**: `while (tmf < minTm) { if (ge >= avail3p) throw "cannot reach min Tm: only N bases available 3' of the mutation"; ++ge; …; best = ge }`
2. **extend forward to maxTm or terminal G/C**: `while (tmf < 63 && ge < avail3p && !/GgCc]$/.test(pfwd)) ++ge`
3. **GC revert**: if `pfwd` does not end in `[GCgc]`, revert to the last good length `best`.
4. **grow reverse to minTm**: `while (tmr < minTm) { if (xe >= avail5p) throw "…5' of the mutation"; prev = wt[0..++xe]; … }`
5. **extend reverse to maxTm or terminal G/C**: `while (tmr < 63 && xe < avail5p && !/GgCc]$/.test(prev)) ++xe`
6. **GC revert** for reverse.
7. **Tm balance, tolerance 5 °C**: if `tmf > tmr`, grow reverse while `tmf - tmr > 5 && xe < avail5p`, then extend to terminal G/C while `tmr < tmf && xe < avail5p && !endsGC`, remembering the last length that stayed below `tmf`; symmetric for `tmr > tmf`.
8. `pfwd`/`prev` have `-` stripped; return `{pfwd, prev, tmf, tmr}`.

Caller then computes `ta = min(tmf, tmr) + 1`, capped at 72.

The `63` threshold is the literal `Tt = 63` in the function, not `primerMaxTm`.
`primerMaxTm` (70/72) is used elsewhere; the GC-extension ceiling is hard-coded 63.

### 6.2 Whole-fragment autogen: `Primer.calculatePrimers`

Same shape as §6.1 but without the alignment/mutation term:
`pad = minLen (default 20)`, `minTm = 55`, `maxTm = 63` (hard-coded), GC-clamp,
balance tolerance 5, overlap checks `minLenF + minLenR > len` and
`prev.length + pfwd.length > len`, plus the same new `cannot reach min Tm … within N bp`
throws and `len < seqLen` loop guards. Defaults from `Fragment.autogenPrimers`:
`polGroup "Q5"`, `primerMethod 4`, `primerMonosalt 150`, `primerCt 500`,
`primerMinLen 15`, `primerMinTm 55`, `primerMaxTm 70`, `direction "both"`.

### 6.3 What did **not** change

The algorithmic skeleton — pad, grow-to-minTm, extend-to-63-or-terminal-GC,
revert-to-last-good, balance within 5 °C — is unchanged from v2.5.0 through v2.11.0.
This confirms the `provenance.md` item *"pad 10, grow to min Tm, GC-clamp below 63 °C,
balance within 5 °C"* and the HAR report's "byte-identical across v2.5.3→v2.8.4"
hypothesis, now extended to v2.11.0 modulo renaming, log-stripping, and the guards.

## 7. Constants (live v2.11.0)

| Constant | Value | Where |
|---|---|---|
| scoring matrix `mat` | `a:{a:2,c:-1,g:1,t:-1}`, `c:{a:-1,c:2,g:-1,t:1}`, `g:{a:1,c:-1,g:2,t:-1}`, `t:{a:-1,c:1,g:-1,t:2}` | `calcSDMPrimers` — **identical to v2.5.0–v2.8.4** |
| `gap` / `term` / `extend` | `-2.5` / `-2.5` / `-1` | `calcSDMPrimers` |
| alignment short-circuit length | `10` | `calcSDMPrimers` |
| min primer length floor | `15` | `calcSDMPrimers` clamp |
| min Tm clamp range | `[55, 72]` | `calcSDMPrimers` clamp |
| GC-extension Tm ceiling | `63` (hard-coded) | both builders |
| Tm balance tolerance | `5` °C | both builders |
| `Ta` | `min(tmf, tmr) + 1`, cap `72` | call site |
| TM `R` | `1.987` | `createTmCalc` |
| TM `ct` (SDM path) | `0.5` µM → `5e-7` M | `Ne.tm` |
| TM `salt` (SDM path) | `150` mM → `0.15` M | `Ne.tm` |
| TM method (SDM path) | `4` (Q5); `1` for Phusion | `Ne.tm` |
| initiation / terminal A/T / terminal G/C | `dS -5.7, dH 200` (class tables) / `+4.1, +2300` / `-2.8, +100` | `calcTm_nomm` |
| `simpleTm` | `4*(g+c) + 2*(len-g-c)` | unchanged |
| FASTA wrap width | `70` | new `wrap` |

## 8. Validation against existing fixtures

The existing fixtures validate **v2.5.0**, not v2.11.0. What the new capture
changes:

| Fixture claim | Status against v2.11.0 |
|---|---|
| `mat`, gap/term/extend identical across v2.5.0–v2.8.4 | **confirmed**, extends to v2.11.0 |
| `calcSDMPrimers` byte-identical v2.5.3→v2.8.4 | **confirmed modulo renaming**; v2.11.0 adds guards and 2 parameters |
| `Ta = min(Tm)+1`, cap 72 | **confirmed** at the live call site |
| pad 10 / grow to minTm / GC-clamp 63 / balance 5 | **confirmed** |
| `R = 1.987`, 150 mM monovalent, 500 nM primer | **confirmed** in `Ne.tm` |
| "mismatch-aware vs no-mismatch path" — **open item** | **resolved**: `Ne.tm` → `createTmCalc.Tm()` (closure), which uses `nndhds`/`loop`/`bulge`, i.e. the **mismatch-aware** path. `calcTm_nomm` is reachable only via the unused `TmCalc` class. |
| D404N Tm 74 / 70, `Ta` 71 | **not re-validated** — the `example-construct/` fixtures are deleted from this branch, and the reconstruction is v2.5.0. The four `nndhds`/`nnbulge` numeric edits mean the v2.5.0 reconstruction is **no longer expected to reproduce v2.11.0 output** on arbitrary primers. |

**Consequence for regression testing:** the single captured D404N result is a v2.5.0
observation. Any independent implementation targeting the live build must re-derive
Tm with the four corrected table entries, and must re-capture end-to-end output from
the live tool before treating D404N as a fixture.

## 9. Plan for independent implementation and validation

Research plan only; nothing here is implemented.

1. **Re-capture end-to-end tool output.** The current capture is page-load only —
   it contains no tool-usage traffic. Extend `capture-js.mjs` to drive the form
   (type sequence, submit mutation, read the primer table) and record the
   request/response traffic. The app is client-side, so the observable artefact is
   the rendered table plus the download blobs, not an API call. Capture at least:
   D404N-equivalent inline substitution on a circular template, the same on a
   linear/partial template, a mutation within `minLen` of each end, and a mutation
   straddling the origin.
2. **Freeze the corrected TM tables.** Extract `dSSa`, `dHSa`, `dSBr`, `dHBr`,
   `nndhds`, `loop`, `bulge`, `nnSa`, `nnBr`, `nntmm`, `nnloop`, `nnbulge` from the
   live bundle verbatim into a fixture, with the sha256 of the bundle they came from.
   Diff against the v2.5.0 extraction to localise the four numeric edits.
3. **Re-implement `calcSDMPrimers` independently** from §6, including the
   `avail3p`/`avail5p` bounds and the `cannot reach …` error strings, and check it
   against the re-captured live output — not against the v2.5.0 reconstruction.
4. **Adopt the guards as requirements.** The v2.11.0 bounds guards turn previously
   silent overruns into typed errors. Luria should raise the same conditions
   explicitly; the four error strings are a ready-made message catalogue.
5. **Decide the degenerate-base question.** v2.11.0 is IUPAC-correct for `B` and `D`
   but still wrong for `S` (`DEGENERATES[S] = "CG"`, `comp[S] = "S"`, correct DNA is
   `[CT]` and `R`). This is a NEBaseChanger defect, not a design choice. Ask NEB, or
   treat canonical IUPAC semantics as the Luria requirement and record the divergence.
6. **Restore the missing fixtures.** `tests/fixtures/example-construct/` and the four
   `research/polymerase-tm` fixture files are absent from this branch; the
   v2.5.0 bundle `index.fc8d957d.js` is absent everywhere. Re-fetch from Arquivo.pt
   before relying on any v2.5.0 claim.
7. **Fix `capture-js.mjs`.** The manifest under-reports one asset
   (`5f43d215d44c-index.4d8eac54.js` on disk, not in `assets`). Completeness claims
   must be made against the directory listing.
8. **Do not commit the fixture tree as-is.** `tests/fixtures/nebasechanger-research/`
   is untracked but not ignored; only `browser-profile*/` and `node_modules/` are.

---

## 10. Algorithm inventory (live build)

The library is a single minified line (line 42); the app is lines 43-500. Line
numbers below refer to the **beautified** live file
`live-js/beautified/index.4330b0c7.v2.11.0-live.beauty.js`.

### 10.1 Library classes (verified by `prototype` method enumeration)

| Class | Methods | Role |
|---|---|---|
| `NASeq` | `wseq`/`cseq`/`pairmap`, `revcomp`, `comp`, `trim`, `expandDNASeq` | DNA + IUPAC-degenerate sequence model |
| `Primer` | `getSeq`, `getTail`, `setTail`, `setForward`, `setReverse`, `direction`, `toJson`, `deepCopy` | one primer = annealing part + non-annealing tail |
| `BuilderPrimer` | `constructor`, `getSeq`, `getTail`, `setTail`, `updateTm` | NEBuilder\|Hi-Tec primer variant |
| `Fragment` | 25 methods: `deepCopy`, `toJson`, `coord`, `generateSeq`, `setFwdPrimer`, `setRevPrimer`, `getFwdPrimer*`, `getRevPrimer*`, `setLeftRE`, `setRightRE`, `rebuildLeftRE`, `rebuildRightRE`, `autogenPrimers`, `autogenSDMPrimers`, `reset`, `setRange`, `setStart`, `setEnd`, `adjustStart`, `adjustEnd`, `updateSrcseq`, `clearPrimers`, `updateTm` | one mutation site: range + left/right context + the primer pair |
| `Alignment` | 13 methods: `coordS1`, `coordS1Offset`, `coordS2`, `coordS2Offset`, `sanitize`, `setAlignedSeqs`, `isAlignedAtS1Coord`, `isAlignedAtS2Coord`, `isS2PresentAtS1Coord`, `isS1PresentAtS2Coord`, `getBasesAtS1Coord`, `getBasesAtS2Coord`, `getBasesAtOffset` | coordinate mapping between source and mutant sequence |
| `DialogManager` | 9 methods: `blockDocument`, `unblockDocument`, `updateStacking`, `containedByTopDialog_`, `handleFocus_`, `handleKey_`, `checkDOM_`, `pushDialog`, `removeDialog` | UI dialog stack (accessibility layer added in v2.11.0) |
| `TmCalc` | 25 methods: `nnBr`, `nnSa`, `nnde`, `nndhds`, `nnloop`, `nnbulge`, `nntmm`, `R`, `dSBr`, `dSSa`, `dHBr`, `dHSa`, `init`, `saltCorrect`, `setMethod`, `setSeq`, `setCt`, `setMonosalt`, `setDisalt`, `setDMSO`, `revcomp`, `comp`, `calcTm`, `calcTm_nomm`, `buildPairMap` | thermodynamic melting-temperature engine |

### 10.2 Melting-temperature models

`TmCalc.setMethod(n)` selects one of four documented models (observed in the
app-layer `createTmFunction` switch and `Ge.tm` wrapper):

| `method` | Model | Tables used |
|---|---|---|
| 1 | Bresluer / "basic" Wallace-Marmur | `dSBr`, `dHBr` |
| 3 | SantaLucia (1999) nearest-neighbour | `dSSa`, `dHSa` |
| 4 | Owczarzy salt correction (default for Q5 SDM) | nearest-neighbour + `saltCorrect` |
| 7 | basic salt-adjusted (Phusion default) | nearest-neighbour |
| 2 | Takahashi & Sagar distance-weighted geometric mean (DMG) | `nnde`, `nndhds`, `nnloop`, `nnbulge`, `nntmm` |

`TmCalc` also carries a DMSO buffer correction: `calcTm_nomm` subtracts 0.6 °C
per mol/L of DMSO (`setDMSO`).

### 10.3 Verified constants (live build)

| Constant | Value |
|---|---|
| Substitution scoring matrix `mat` | `{a:{a:2,c:-1,g:1,t:-1}, c:{a:-1,c:2,g:-1,t:1}, g:{a:1,c:-1,g:2,t:-1}, t:{a:-1,c:1,g:-1,t:2}}` |
| Gap / term / extend penalties | `-2.5 / -2.5 / -1` |
| `TmCalc.R` (gas constant) | `1.987` cal/(mol·K) |
| `createTmFunction` defaults | `method=4`, `monosalt=50` µM, `ct=250` µM; for `method` 1 or 5, `ct` is divided by 4 |
| `Primer` constructor defaults | `primerMethod=4`, `primerMonosalt=50`, `primerCt=200`, `primerMinLen=20`, `primerMinTm=55`, `primerMaxTm=70` |
| `Primer._modelPrimer` defaults | `primerMonosalt=150`, `primerDisalt=0`, `primerCt=500`, `primerMethod=4`, `primerMinLen=20`, `primerMinTm=55`, `primerMaxTm=70` |
| `Ge.tm` (Q5 polymerase wrapper) | `ct=0.5` µM, `salt=150` µM, `method=4` |
| `Ge.tm` (Phusion wrapper) | `method=1` |
| `Ge.calcSDMPrimers` clamps | primer min length `>= 15`; min Tm clamped into `[55, 72]` °C; substitution overlap pad `10` |
| `DEGENERATES` (live) | `B=CGT`, `D=AGT`, `H=ACT`, `V=ACG`, `N=ACGT`, `S=CG`, `W=AT`, `M=AC`, `K=GT`, `R=AG`, `Y=CT` |
| `COMPLEMENTS` | `ACGTUNSWMKRYVDHB -> TGCAANSWKMYRBHDV` (S->S, W->W, M<->K, R<->Y, V<->B, D<->H) |
| Amino-acid table | `AMINO_ACID_TLC` (20-letter coding incl. `XLE:"J"`, `PYL:"O"`) |

### 10.4 Codon selection (amino-acid mutation mode)

- Codon frequency table: `Ecoli_K12`, keyed by amino acid (`Ecoli_MEGA` is not
  present in the live bundle — see 5.8).
- `Me` = highest-frequency codon selection ("cut" strategy).
- `Ue` = minimum-Hamming-distance codon selection ("mindist" strategy).
- Both are present in the live build and unchanged in structure from v2.8.4.

### 10.5 Dispatcher

`Tt` (app layer) routes each mutation:

- deletions and long replacements (`>= 6` nt, or explicitly flagged) ->
  `Fragment.autogenPrimers` (split-across-both-primers strategy);
- short replacements (`< 6` nt) -> `Ge.calcSDMPrimers` (replacement carried
  entirely in the forward primer, `>= 10` complementary annealing bases).

This matches the in-app manual text extracted earlier (`nbc_help_v284.txt`).

## 11. What actually changed between v2.8.4 and the live build

Version history recovered from the in-app changelog (`<dt>` entries in the live
bundle). The live site is **v2.11.0, dated 2026-10-10** — released the same day
as this capture.

| Version | Date | |
|---|---|---|
| v2.8.4 | 2026-07-10 | last version in the Wayback capture set |
| v2.9.0 | 2026-10-08 | accuracy/reliability release |
| v2.10.0 | 2026-10-09 | state-preservation fixes |
| v2.11.0 | 2026-10-10 | keyboard/accessibility + ORF-start handling |

### 11.1 Verified data corrections (v2.8.4 -> live)

`tm_tables_diff.py` compares the lookup tables byte-for-byte. Result:

| Table | v2.8.4 | live v2.11.0 | Note |
|---|---|---|---|
| `dSBr`, `dSSa`, `dHBr`, `dHSa` | — | — | **byte-identical** |
| `nnde`, `nnloop`, `nntmm` | — | — | **byte-identical** |
| `mat` (substitution scoring) | — | — | **byte-identical** |
| `nndhds` | `aa/tc = {dh:-9.1, ds:-24}` | `aa/tc = {dh:2.3, ds:4.6}` | corrected |
| `nndhds` | `ct/aa = {dh:-9.1, ds:-24}` | `ct/aa = {dh:2.3, ds:4.6}` | corrected |
| `nndhds` | `ta/at = {dh:-7.2, ds:-20.4}` | `ta/at = {dh:-7.2, ds:-21.3}` | corrected |
| `nnbulge` | `25 = {dh:0, ds:-28.1}` | `25 = {dh:0, ds:-18.1}` | corrected |

`DEGENERATES` correction (present as a bug in v2.5.3, v2.7.2 **and** v2.8.4):

| Code | v2.5.3 / v2.7.2 / v2.8.4 | live v2.11.0 | IUPAC correct? |
|---|---|---|---|
| `B` | `AGT` | `CGT` | live yes, archived no |
| `D` | `ACT` | `AGT` | live yes, archived no |
| `H` | `ACT` | `ACT` | both yes |
| `V` | `ACG` | `ACG` | both yes |

These match the v2.9.0 changelog line: *"Corrected melting temperatures
(TA/AT and A·C mismatch nearest-neighbor values) and degenerate base handling
(IUPAC B and D codes, S/W complements)."* The S/W half of that line refers to the
**app-layer** complement table, not `t.COMPLEMENTS` (see 6.3).

### 11.2 Behavioural changes claimed by the changelog (not yet verified in code)

- v2.9.0: "Every primer pair is now checked against the intended mutant sequence
  before it is shown or downloaded." -> a new validation pass; **not located in
  the code yet**.
- v2.9.0: primer naming unified as `ID_n_F` / `ID_n_R`; lowercase marks the
  mutation in every download; FASTA wrapped at 70 characters.
- v2.9.0: mutations near either end of linear sequences and across the origin of
  circular sequences.
- v2.11.0: changing ORF start no longer clears entered mutations.

## 12. Open questions for the Luria contract

1. **Which build is the reference?** The archived v2.8.4 bundle and the live
   v2.11.0 bundle disagree on `nndhds`, `nnbulge`, and `DEGENERATES` B/D. Luria
   must pick one, or encode the corrected (live) values and record the
   divergence.  *Narrowed in round 3 (see 16.3): the three disagreeing `nndhds`
   keys are not reachable in the live build's default configuration, so the
   choice does not affect the Q5 SDM numbers.  It does affect any Luria model
   that scores sequences containing `x` or `|`.*
2. **Units of `ct` and `monosalt`.** `createTmFunction` defaults to `ct=250`
   µM and divides by 4 for methods 1/5, while `Ge.tm` passes `ct=0.5`. The
   in-app manual says "500 nM" and "25-cycle". The unit convention is not
   self-evident in the code; do not guess.
3. **`Primer` constructor bug**: `i.primerMaxTm && (this.primerMaxTm = i.maxTm)`
   reads `i.maxTm`, not `i.primerMaxTm`. Reproduce or fix? Decide explicitly.
   *Resolved as inert in round 3 (see 16.2): the write is overwritten by a
   following `defineProperty`, and the field is unreachable from the Q5 SDM
   path in any case.*
4. **Which Tm model does the Q5 SDM path actually use at runtime?** `method=4`
   (Owczarzy) is the documented default, but the DMG tables (`method=2`) are
   shipped. Confirm by running the live page, not by reading defaults.
5. **Primer-pair validation pass** (v2.9.0 changelog) — locate it before
   claiming the live algorithm set is understood.

### 12.1 Verified code-level diffs (v2.8.4 -> live v2.11.0)

Confirmed by direct string search in both bundles, not inferred:

1. **Debug logging removed.** `dmsg` is `function(l){}` (no-op) in live; in
   v2.8.4 it is a real `window.console.log` wrapper. v2.8.4 also contains bare
   `console.log("repseq", …)`, `console.log("offset", …)`,
   `console.info(…)` inside `calcSDMPrimers`; live has none of these.
2. **`TmCalc.prototype.calcTm` and `TmCalc.prototype.buildPairMap` deleted.**
   v2.8.4 ships both (with `console.log` debug lines); live ships only
   `calcTm_nomm`. A *different* `buildPairMap` exists at Fragment level in both.
3. **New short-substitution fast path.** Live:
   `Sl=/^[acgt]+$/i, St=10` and
   `Ye = (old.length===new.length && old.length<St && Sl.test(old) && Sl.test(new))`
   `? {alnSeq1: old.toLowerCase(), alnSeq2: new.toLowerCase()}`
   `: new Alignment("global", old, new, ft)`.
   v2.8.4 always runs Needleman-Wunsch. So for equal-length pure-DNA
   substitutions shorter than 10 nt, live skips alignment entirely.
4. **New explicit failure modes** (live only, absent in v2.8.4):
   - `cannot reach min primer length N: only K bases available 3' of the mutation`
   - `cannot reach min primer length N: only K bases available 5' of the mutation`
   - `cannot reach min Tm: only K bases available 3' of the mutation` (x2)
5. **`DEGENERATES` B/D correction** (see 6.1).
6. **`nndhds` / `nnbulge` corrections** (see 6.1).

### 12.2 `calcTm_nomm` method semantics (verified, live build)

`h = this.ct * 1e-6` (so `ct` is in µM, converted to mol/L).

| `method` | Tables | Formula shape |
|---|---|---|
| 1 | `dSBr`, `dHBr` | Breslauer: `tm = dh_sum / (ds_sum + R·ln(h)) − 273.15`, initial term `sym ? −12.4 : −10.8`; then `+ sc_sch` |
| 3 | `dSSa`, `dHSa` | SantaLucia NN, then Allawi salt: `s = 1/(1/s + sc_sl/(dh+u))` |
| 4 | `dSSa`, `dHSa` | SantaLucia NN, then **Owczarzy** salt: `s = 1/(1/s + sc_ow)` |
| 5 | `dSSa`, `dHSa` | SantaLucia NN, then `s = s + sc_sch` |
| 7 | none | `s = 81.5 + 16.6·log₁₀(saltc) + 0.41·fgc − 675/len` |

There is **no `case 2`** in `calcTm_nomm`. The DMG tables (`nnde`, `nndhds`,
`nnloop`, `nnbulge`, `nntmm`) are consumed elsewhere, via the app-layer
`dmsg`-adjacent DMG path (`Ge.dmsg` / `Ne.dmsg` call sites at Fragment level).

SantaLucia NN edge corrections (methods 3/4/5): leading/trailing `a`/`t` add
`+4.1` to `ds` and `+2300` to `dh`; leading/trailing `g`/`c` add `−2.8` and
`+100`; symmetric sequence adds `−1.4` to `ds`.

Final correction applied to every method: `s = s − dmso · 0.6`.

`TmCalc.init` defaults: `ct=0.25`, `monosalt=50`, `disalt=0`, `dmso=0`,
`method=4`. `saltCorrect()` throws `{name:"Missing salt"}` when both
`saltc` and `disaltc` are zero.

### 12.3 `alignDNAglobal` parameters

The function's own defaults are `gap=-5, term=-1, extend=-1` and
`mat = {diag:+3, offdiag:−3}`. The **SDM caller overrides** them with
`gap=-2.5, term=-2.5, extend=-1` and
`mat = {a:{a:2,c:-1,g:1,t:-1}, c:{a:-1,c:2,g:-1,t:1}, g:{a:1,c:-1,g:2,t:-1}, t:{a:-1,c:1,g:-1,t:2}}`
(identical in v2.8.4 and live). Cite the caller values, not the defaults.

Codon tables in the live build: `Ecoli_K12` only (9 occurrences). `Ecoli_MEGA`
is **not** present in the live bundle.

### 12.4 App-layer complement table: S/W bug (verified, byte-level)

Two complement tables exist. The **library** one (`TmCalc.COMPLEMENTS`, built
from paired strings) is byte-identical in both builds and is **correct**:

```
src "ACGTUNSWMKRYVDHBacgtunswmkryvdhb -"
dst "TGCAANSWKMYRBHDVtgcaanswkmyrbhdv -"   ->  S->S  W->W  M<->K  R<->Y  V<->B  D<->H
```

The **app-layer** one (`Ge.comp` / `Ge.revcomp` in v2.8.4, `Ne.comp` /
`Ne.revcomp` in live) is a paired-string `indexOf`/`charAt` lookup and **differs**:

```
v2.8.4  src "ACGTUKMSWRYBDHVNacgtukmswrybdhvn -"
        dst "TGCAAMKWSYRVHDBNtgcaamkwsyrvhdbn -"   ->  S->W  W->S   (wrong)

live    src "ACGTUKMSWRYBDHVNacgtukmswrybdhvn -"
        dst "TGCAAMKSWYRVHDBNtgcaamkswyrvhdbn -"   ->  S->S  W->W   (correct)
```

All other positions agree (`A->T C->G G->C T->A U->A K->M M->K R->Y Y->R
B->V D->H H->D V->B N->N`). Occurrence counts confirm the swap is the only
difference: `MKWSY` appears 2x in v2.8.4 and 0x in live; `MKSWY` 0x and 2x.

**Consequence:** in v2.8.4, any primer or flanking context containing an IUPAC
`S` or `W` is complemented to the wrong set by the app-layer path. `Ne.tm()` and
the primer builders use this app-layer table, not the library one. This is a
real numerical/biological divergence between the archived fixture and the live
site, not a cosmetic one.

### 12.5 Other verified app-layer diffs (v2.8.4 -> live)

- `Ge.revcomp` in v2.8.4 logs (`console.log("NEB.revcomp", s)`) and throws via
  `throw console.log(...)`; live has neither.
- `Ge.tm(I, comp)` / `Ne.tm(I, X)` defaults are identical: `ct=0.5`, label
  `"Q5"`, `salt=150`, `method=4`.
- v2.8.4 error handling in the batch replace path: `console.log("error!!!!!--4", …)`
  then a generic `"Unexpected error in replace operation."`.
  Live: `msg.indexOf("cannot reach") !== -1 ? id + " (" + s + "-" + e + "): " + msg
  : "Unexpected error in replace operation."` — i.e. the new "cannot reach …"
  errors from 5.6(4) are surfaced to the user with the operation ID.

## 13. Reproduce

```
cd tests/fixtures/nebasechanger-research
python3 tm_tables_diff.py
```

`tm_tables_diff.py` prints, for both the archived v2.8.4 bundle and the live
v2.11.0 bundle: byte length and byte-identity of `dSBr`, `dSSa`, `dHBr`,
`dHSa`, `nnde`, `nndhds`, `nnloop`, `nnbulge`, `nntmm`, `mat`; the differing
`dh`/`ds` keys for the two tables that changed; and every paired complement
alphabet with its `S`/`W` mapping.

Changelog extraction (version history + v2.9.0/v2.10.0/v2.11.0 release notes):

```
grep -aoE '<dt[^>]*>[^<]{0,60}</dt>' \
  live-capture/004eae75971a-index.4330b0c7.js | grep -aiE 'v[0-9]'
```

## 14. Files changed by this pass

- `docs/research/nebasechanger-live-build-4330b0c7-2026-10-10.md` — this report.
- `tests/fixtures/nebasechanger-research/tm_tables_diff.py` — new, self-contained
  comparison script (section 6.1 and 6.3 are its output).

No Luria source, no Rust, no Milestone 3 code. Nothing was committed.

---

## 15. Verified v2.8.4 -> live deltas in the primer-design layer (round 2)

All of §15 is machine-checked by `q6_v284_vs_live.py` (run it; output is
reproduced verbatim below the claims). Nothing here is inferred from a
diff — each claim is a token/regex count against the two bundles.

### 15.1 `Primer.calculateSDMPrimers` is dead code that was *deleted*

| Build | occurrences of `calculateSDMPrimers` |
|---|---|
| v2.8.4 | **1** (the definition, line 2923) |
| live v2.11.0 | **0** |

The v2.8.4 definition opens with `console.log("calculatePrimers called")` and
references the undefined identifiers `repseq`, `repseqtpl`, `align` — it could
never run. The live build drops it. Consequence for Luria: **there is exactly
one primer-design engine**, `Primer.calculatePrimers`, in the live build. The
"SDM" name in `Fragment.autogenSDMPrimers` is a *label*, not a different
algorithm.

### 15.2 `Fragment.autogenPrimers` / `autogenSDMPrimers` are the same call

Both builds, both methods, identical option defaults and identical engine:

| Option | default (both builds, both methods) |
|---|---|
| `polGroup` | `"Q5"` |
| `primerMethod` | `4` |
| `primerMonosalt` | `150` |
| `primerCt` | `500` |
| `primerMinLen` | `15` |
| `primerMinTm` | `55` |
| `primerMaxTm` | `70` |
| `direction` | `"both"` |

Both then call `s.Primer.calculatePrimers(this.template, n)` inside a
`try`/`catch` that rethrows as `"primer autogen failed: " + message`.

One real change: live `autogenPrimers` starts with `n = Object.assign({}, n)`
(copies the caller's options); live `autogenSDMPrimers` does **not**, and
neither v2.8.4 method does. So in three of four combinations the caller's
options object is mutated in place — a side effect Luria should not reproduce.

### 15.3 Three different default sets for the same quantities

Identical in both builds:

| Site | `monosalt` | `ct` | `minLen` | `minTm` | `maxTm` |
|---|---|---|---|---|---|
| `Primer(n,a,i)` default object literal | 50 | 200 | 20 | 55 | 70 |
| `Primer` `_modelPrimer` fallbacks | 150 | 500 | 20 | 55 | 70 |
| `Primer.calculatePrimers` fallbacks | 150 | 500 | 20 | 55 | 70 |
| `Fragment.autogen*` defaults | 150 | 500 | **15** | 55 | 70 |

Reading the code, the object literal only supplies 50/200 when the caller
omits `i` entirely; the `_modelPrimer` fallbacks then see those values, so
`Primer`'s own Tm function is built at **50 mM / 200 nM** while its
`_modelPrimer` records **50 / 200** too. The 150/500 pair is reached only via
`Fragment.autogen*` or `calculatePrimers` called without options. The
in-app manual states the user-visible defaults are **15 nt and 55 °C** and the
"preferred primer concentration (500 nM)" — i.e. the manual describes the
`Fragment` layer, not the `Primer` layer.

`[UNKNOWN]` whether any live call path omits `monosalt`/`ct` and therefore
silently uses 50 mM / 200 nM. Not settled by this pass.

### 15.4 In-app manual statements (verbatim, byte offsets in `nbc_help_v284.txt`)

- `@18794` — *"NEBaseChanger uses primer melting temperature (Tm) when
  determining how long a primer should be. For substitutions that span a
  mutation region greater than 6 nucleotides, the 5' end of the primer is
  anchored in the center of the region and extended in the 3' direction until
  a desired minimum Tm is achieved."*
- `@19261` — *"For fewer than 6 substitutions, the center of the primer is
  anchored to the nucleotide region and extended in both the 5' and 3'
  directions so that there are **at least 10 nucleotides that are 100%
  complementary on both sides** of the mutation region and until a desired
  minimum Tm is achieved."* — this is the documented meaning of the hardcoded
  `pad = 10` in `Ge.calcSDMPrimers`.
- `@19394` — *"For deletions, the primer is designed by anchoring its 5' end
  to the nucleotide adjacent to the selected mutation region and extending in
  the 3' direction until a desired minimum Tm is achieved."*
- `@20019` — *"Tm's are calculated using nearest neighbor thermodynamics data
  for matched and internal mismatched bases and **corrected for buffer
  composition** … assuming the recommended buffer composition for Hot Start
  High-Fidelity DNA Polymerase, and the **preferred primer concentration
  (500nM)**."*
- `@15210` — *"minimum primer length and minimum primer Tm with default
  values of 15nt and 55degC … To match the specifications used in NEBaseChanger
  v1, set the minimum length to 18 nt and kepp the minimum Tm at 55degC."*

These are the only in-app statements that pin the `6`-nt routing threshold,
the `10`-nt complementary-flank rule, and the `500 nM` concentration. The
Q5-SDM "Troubleshooting Guide" is a 10-state diagnostic tree about PCR and
transformation **failure modes** (colonies, gel bands, SNP sources); it
contains no primer-design numbers and must not be cited as design evidence.

### 15.5 Reproduce

```
cd tests/fixtures/nebasechanger-research
python3 q6_v284_vs_live.py
```

Prints: occurrence counts of `calculateSDMPrimers` in both bundles; the
option defaults and engine of `Fragment.autogenPrimers` /
`autogenSDMPrimers` in both bundles, plus whether each copies its options
argument; the three default sets inside `Primer` and
`Primer.calculatePrimers`; and the five manual quotations with their byte
offsets. Local only, no network.
## 16. Four-build comparison (Q5-Q7), round 3

Reproduced by `tests/fixtures/nebasechanger-research/q5_extract.py`,
`q5_verify.py`, `q1q3_four_builds.py`, `q7_nndhds_keys.py`.  All four builds
compared: v2.5.0 `17c7600d`, v2.5.3 `a8869455`, v2.8.4 `a5da1636`, live
`4330b0c7`.

### 16.1 Substitution routing and the alignment pad

| quantity | v2.5.0 | v2.5.3 | v2.8.4 | live |
|---|---|---|---|---|
| fragment path (`buildPrimers`) | absent | absent | present | present |
| routing threshold | n/a | n/a | `repl.length >= 6` or `target.length >= 6` | same |
| `confine5prime` forces the fragment path | n/a | n/a | yes | yes |
| replacement attached as a whole 5-prime tail | n/a | n/a | `repl.length <= 6` | same |
| split point `At = ceil(target.length/2)` | n/a | n/a | yes | yes |
| SDM-path alignment pad | 10 | 10 | 10 | 10 |
| `alignDNAglobal` penalties | -12 / -1 / +2 / -4 | same | same | same |
| `calcSDMPrimers` length floor | 15 | 15 | 15 | 15 |
| `calcSDMPrimers` Tm floor | clamp(55, 72) | same | same | same |
| `calcSDMPrimers` min annealing tail | 10 | 10 | **11** | **11** |

The only routing-relevant change across the four builds is the minimum
annealing tail in the SDM path, 10 nt in the 2.5.x builds and 11 nt from 2.8.4
onward.  The live build did not change it.

### 16.2 Q1 selection and Q3 maxTm, per build

| build | `Ne.tm` | body | `createTmCalc` default `method` | `case 4` salt block | `calcSDMPrimers` reachable | `primerMaxTm` reads / writes |
|---|---|---|---|---|---|---|
| v2.5.0 | `Ne.tm` | `createTmCalc` | 4 | yes | no | 0 / 0 |
| v2.5.3 | `Ne.tm` | `createTmCalc` | 4 | yes | yes | 0 / 0 |
| v2.8.4 | `Ne.tm` | `createTmCalc` | 4 | yes | yes | 1 / 1 |
| live | `Ne.tm` | `createTmCalc` | 4 | yes | yes | 1 / 1 |

`Ge.calcSDMPrimers` has exactly one call site in every build that has it: the
`replace` branch of the SDM dispatcher, guarded by
`!confine5prime && repl.length < 6 && target.length < 6`.  v2.5.0 has no such
dispatcher, so the function is dead there.

`primerMaxTm` is read once, in the GC-clamp loop of the fragment-path primer
growth code, and written once, in the `Primer` constructor.  Both are in the
fragment path, which the 2.5.x builds do not have.

### 16.3 Q7 - reachability of the changed `nndhds` keys

| build | `createTmCalc` option literal | `nndhds` bytes | keys |
|---|---|---|---|
| v2.5.3 | `{ct:0.5, salt:150, method:4}` | 4418 | 176 |
| v2.8.4 | `{ct:0.5, salt:150, method:4}` | 4418 | 176 |
| live | `{ct:0.5, salt:150, method:4}` | 4416 | 176 |

The changed keys are `x|` (0 to -1), `x-` (0 to -1), `|x` (0 to -1).  The pair
map is built from `seq` and `revcomp(seq)`, so a key containing `x` or `|`
requires the scored sequence to contain `x` or `|`.  Primer candidates are
built from `expandDNASeq` over `DEGENERATES`, which resolves degenerate
symbols before scoring.  `[DERIVED]` the three changed keys are unreachable in
the live build's default configuration.  `[UNKNOWN]` whether any user path can
place a literal `x` or `|` into a scored sequence.


---

## 17. Validation, verification, naming and download layers (round 4)

Round 4 answers a different question from §4-§16: not "what is the primer
algorithm" but "what does the live build check before it shows or downloads a
primer pair".  The live build added an entire validation/verification layer that
has no counterpart in v2.8.4, and that layer is what a Rust re-implementation
must reproduce if it wants to agree with the live site on *accept/reject*
behaviour, not just on primer strings.

### 17.0 Method and durable artifacts

Three new scripts, all in `tests/fixtures/nebasechanger-research/`, with their
output stored next to the existing diffs in `live-js/diffs/`:

| script | what it does | output file |
|---|---|---|
| `q8_live_vs_284.py` | extracts each named algorithm definition from both **minified** bundles, reduces each to a token stream with local identifiers masked to `ID`, and reports SAME / DIFFERENT | `q8-token-shape-v284-vs-live.txt` |
| `q9_live_deltas.py` | extracts the same definitions from both **beautified** bundles and prints a unified diff per function; prints the full body for names absent in one build | `q9-function-diffs-v284-vs-live.txt`, `q9-new-function-bodies-live.txt` |
| `q10_tm_tables.py` | entry-by-entry comparison of every numeric table inside `createTmCalc` | `q10-tm-table-entry-deltas.txt` |
| (inline, §17.8) | brace-matched extraction of the primer dispatcher (`Tt` in v2.8.4, `tl` in live) and a unified diff | `q10-dispatcher-v284-vs-live.diff` |

`q8` masks identifiers, so `SAME` means "identical modulo minifier renaming".
`DIFFERENT` from `q8` is a structural difference; `q9` then shows which one.

### 17.1 Token-shape verdict over the agreed algorithm surface

`[VERIFIED]` (q8, 20 names):

- identical modulo renaming: `expandDNASeq`, `simpleTm`, `createTmFunction`,
  `cut`.
- present in v2.8.4 only (deleted in live): `calcTm`, `Primer.calculateSDMPrimers`
  — the latter is the dead definition with the undefined free variables
  (`repseq`, `repseqtpl`, `pad`, `offset`, `align`, `minf`, `Ws`) already noted
  in §15.1.  Its deletion is the strongest available confirmation that it was
  dead code, not a live alternative path.
- present in live only: `matchesIUPAC`, `verifyOp` (and `verifyPair`, checked
  separately in 17.2).
- structurally different, with the difference enumerated in §17.2-§17.8:
  `DEGENERATES` (B/D only), `alignDNAglobal`, `alignDNAlocal`, `createTmCalc`,
  `calculatePrimers`, `calcSDMPrimers`, `autogenPrimers`,
  `autogenSDMPrimers`, `terminalOverlapLen`, `revcomp`, `translate`.

For `alignDNAglobal`, `alignDNAlocal`, `terminalOverlapLen`, `autogenPrimers`
and `autogenSDMPrimers` the q9 diff shows the difference is **renaming plus
`console.log` removal** (plus, in the two `autogen*` functions, the two items
listed in 17.8).  The alignment penalties are unchanged: `alignDNAglobal`
`gap=-5, term=-1, extend=-1, mat=+3/-3`; `alignDNAlocal` `gap=-12, extend=-1,
mat=+2/-4`.

### 17.2 The verification gate (live only)

`[VERIFIED]` `Ot`/`vt`/`Dt` namespaces aside, the live build filters every
primer pair through `verifyPair` before it is displayed:

```js
Wl = function (I, X) {
  return (I || []).filter(function (re) {
    var se = ct.verifyPair(re, i, f.isCircular);
    return se && X.push("internal check failed for " + re.idprefix + ": " + se
                     + " - primers not shown. Please report this to NEB."), !se;
  });
};
... Wl(I.primers, X); re.forEach(function (se, Se) { se.pairIndex = Se + 1 });
```

So: a failing pair is **dropped from the table** and a user-visible warning is
emitted; `pairIndex` is assigned *after* filtering, so indices stay contiguous.

`verifyPair(pair, seq, circular)` performs four independent checks, in order:

1. **Reconstruct the mutant sequence.** `applyMutation(seq, selstart, selend,
   repseq, circular)`; failure → `"could not reconstruct the mutant sequence (…)"`.
2. **Complement the reverse primer.** `revcomp(seqr)`; failure →
   `"reverse primer is not a valid IUPAC sequence"`.
3. **Round-trip the pair across the edit site.** With
   `f = (revcomp(seqr) + seqf).toLowerCase()`, the mutant `i`, and canonical
   flanks `d = start`, `p = end`:
   - circular and origin-spanning (`p <= d`): rotate `i` to start at
     `i.length - repseq.length`, double it, search window
     `b = A.length - 1`, `w = A.length + repseq.length`;
   - otherwise: `b = max(d - 1, 0)`, `w = min(d + repseq.length, len - 1)`.
   Some occurrence of `f` must satisfy `T <= b && T + f.length - 1 >= w`, i.e.
   the annealed pair must straddle the edit site.  Otherwise
   `"forward/reverse primers do not match the mutant sequence at the edit site"`.
   Empty `f` → `"empty primer pair"`; missing pair → `"missing primer pair"`.
4. **Flap + anneal must equal the primer.** For both directions,
   `(flap + anneal).toLowerCase() === primer.toLowerCase()` when the flap
   fields are present, else
   `"<direction> flap + annealing region does not reconstruct the full primer"`.

Then it appends `verifyOp(pair.op, repseq, seq, selstart, selend, circular)`
(17.7).  `[DERIVED]` this is exactly the changelog line "Every primer pair is
now checked against the intended mutant sequence before it is shown or
downloaded" (v2.9.0).

### 17.3 Coordinate normalisation layer (live only)

Four functions define the coordinate contract that every other check uses:

- `normalizeFlanks(start, end, len, circular)` — circular: maps both endpoints
  through `((u - 1) % len + len) % len + 1` (1-based, origin-wrapped); linear:
  errors `"the edit starts before the start of the sequence"` (start < 0),
  `"the edit runs beyond the end of the sequence"` (end > len + 1),
  `"the end position must be after the start on a partial (linear) sequence"`
  (start >= end).
- `canonicalFlanks(start, end, len, circular)` — circular normalisation, then
  `start == len && end > 1` → `start = 0`.
- `applyMutation(seq, start, end, repseq, circular)` —
  `!circular || end > start ? seq.slice(0,start) + repseq + seq.slice(end-1)`
  `: seq.slice(end-1, start) + repseq`.
- `codonAt(seq, idx, circular)` — 3 bases, wrapping modulo `len` when circular.

`classifyOp(start, end, repseq, len, circular)` sets `insert` when the span is
1, `delete` when `repseq` is empty, else `replace`; span is
`(end - start + len) % len` when circular.  It also emits the warning
`"this edit spans the origin and removes N bp, leaving M bp - check that start
and end are not swapped"` when the deleted span exceeds `len / 2`.

### 17.4 Input validation factories and limits (live only)

`[VERIFIED]` none of these names exists in the v2.8.4 bundle (grep count 0 for
each):

| constant | live value | meaning |
|---|---|---|
| `Ot.MAX_SEQ_LEN` | `5e4` | sequence length ceiling |
| `Ot.MAX_VARIANTS` | `256` | ceiling on degenerate expansion product |
| `Ot.VARIANT_COUNTS` | `{A:1,C:1,G:1,T:1,R:2,Y:2,K:2,M:2,S:2,W:2,B:3,D:3,H:3,V:3,N:4}` | per-symbol variant counts |
| `vt.MIN_FLANK_5` | `35` | minimum 5-prime flank (partial sequences) |
| `vt.MIN_FLANK_3` | `25` | minimum 3-prime flank (partial sequences) |
| `vt.NB_MIN_FRAGMENT` | `60` | minimum NEBuilder fragment length |
| `vt.ORF_MIN_CODONS` | `10` | minimum codons between stop and `ATG` for ORF auto-detection |
| `Dt.FASTA_WIDTH` | `70` | FASTA line wrap |

`replacementError` rejects characters outside `[ACGTRYKMSWBDHVN]` (case
insensitive) and reports the offending set; unless preservation of degenerate
codes is requested it multiplies `VARIANT_COUNTS` over the replacement and
returns `"too many variants (N); use at most 256"`.  `[DERIVED]` v2.8.4 had no
such ceiling: `expandDNASeq(ue)` is called with no guard, so a long degenerate
replacement expands combinatorially.

`flankError(start, end, len, circular, min5, min3)` replaces v2.8.4's inline
heuristic `Xe && (Ve <= ot || it > len - 40)` with two explicit messages
quoting the required and available flank lengths.  It is called at four sites
in the live batch paths and one in the single-op dispatcher.

`orfStarts(seq, minCodons)` scans each `[ACGT]+` run for stop codons (`TAA`,
`TAG`, `TGA`) per frame and for `ATG`, and returns ORF starts where a stop in
the same frame lies at least `3 * ORF_MIN_CODONS` before the `ATG`.

### 17.5 Naming and download layer (live only)

`[VERIFIED]` primer IDs changed format between builds:

```
v2.8.4  idprefix + ":" + (index + 1) + "_F"      (bundle offset 304646)
live    Dt.primerId = idprefix + "_" + pairIndex + "_" + (fwd ? "F" : "R")
```

`primerSeq(pair, dir)` is the single definition of the primer *string* used by
every download and by the table:

- forward: `seqfanneal == null ? seqf : lowercase(seqfpflap) + seqfanneal`
- reverse, non-NEBuilder: `lowercase(seqrpflap) + seqranneal`
- reverse, NEBuilder: `seqranneal + lowercase(seqrpflap)`

`[DERIVED]` lowercase carries the mutation (the flap), uppercase the annealing
region — the changelog's "lowercase marks the mutation in every download".

Other download functions: `buildFasta`/`finalSeqRecord` (header
`>seqid_idprefix[_pairIndex] description`, with `"; rotated to start at base N"`
appended for origin-spanning circular edits; body is the lowercased mutant
sequence wrapped at `FASTA_WIDTH`), `summary` (11 TSV columns: ID, Description,
Fwd Primer ID, Fwd Primer Seq, Length, Tm Fwd, Rev Primer ID, Rev Primer Seq,
Length, Tm Rev, Ta), `primerList` (`ID<TAB>seq<TAB>Tm=n Ta=n` per line),
`downloadName` (trim, fall back to a default, force an extension), `shorten`
(`>25` chars → first 10 + `...` + last 10, used in descriptions), and
`tsvCell` (tabs/newlines → spaces; a leading `=`, `+`, `-` or `@` is prefixed
with `'` to defuse spreadsheet injection).

`[VERIFIED]` `FASTA_WIDTH` and `tsvCell` do not exist in v2.8.4, matching the
changelog line "Downloads report problems instead of failing silently; FASTA
files are wrapped at 70 characters."

### 17.6 NEBuilder assembly layer (live only)

`nebuilderBlocker` states the preconditions as messages instead of UI
disabling: codon mode only; `"Confine mutations to primer 5′ tails"` off; at
least two mutations; no `X` (degenerate) targets (`/^[A-Z]\d+X/`).

`nebuilderFragments(pairs, seqLen, mode)`:

- sorts by `locf mod seqLen`;
- a pair starting at the same site as its predecessor is **dropped** with
  `"NEBuilder: <id> starts at the same position as <id>, so it was left out of
  the assembly."` (the changelog's "duplicate sites … are now flagged");
- `mode === "multiple"` and more than one fragment: the reverse primer of
  fragment *i* is `revcomp(seqf of fragment i+1)`, with
  `idprefix = A_B`, `locr = (locf(next) + len(seqrnbanneal)) % len`,
  `wraparound = locf(next) < locf(i)`, `clash = gap < NB_MIN_FRAGMENT`,
  `fraglen = len(seqfpflap) + gap + len(seqrnbanneal) + len(revcomp(seqfpflap))`;
- single fragment: self-complement, `fraglen = seqLen + len(seqrnbanneal)`;
- `ta = min(min(tmf, tmr) + 1, 72)` in both cases.

`[VERIFIED]` `seqrnbanneal` — the annealing region of the *next* fragment's
forward primer, used as the reverse annealing region — is populated in the
dispatcher's insert/replace branch in **both** builds
(`revcomp(fwdPrimer.anneal.toUpperCase())`), but the NEBuilder-specific
`clash`/`wraparound` flags exist only in live.

### 17.7 Merge and op-level verification (live only)

`mergeCodonOps(seq, ops, circular)` rejects, in order:

- overlapping or duplicate codon ranges →
  `"<ids>: duplicate mutation at the same codon"` /
  `"<ids>: overlapping codon positions"`;
- span `> 8 * nOps` → `"<ids>: mutations are too widely spread to safely merge (N bp)"`;
- span `> 60` → `"<ids>: merged span of N bp exceeds the 60 bp limit for a safe merge"`.

It then builds the merged `repseq` by interleaving the wild-type bases between
consecutive codons (`charAt` with modulo wrap when circular).  `verifyOp`
re-checks a merged op: merged `repseq` length must equal the span; the wild-type
run before each codon must equal the sequence; each 3-base slice must satisfy
`Gi(slice, target, triplet)`.

`Gi(triplet, target, explicit)` is the codon matcher: an explicit triplet is an
exact match; target `X` means `matchesIUPAC(triplet, "NNK")`; otherwise
`translate(triplet) === target`.  `matchesIUPAC(a, b)` requires every base of
`a` to be a member of the set denoted by the corresponding IUPAC symbol of `b`
— it is the live implementation of the v2.4.x "X means NNK" rule.

`codonOpError` / `codonRangeError` / `checkCodonOp` parse and validate the
batch codon grammar `Ot.codonOpRegex = /^([A-Z*])(\d+)([A-Z*])(?::([A-Z]+))?$/`
against `Ot.CODON_TARGETS = "ACDEFGHIKLMNPQRSTVWY*X"`, and reject an explicit
triplet that does not translate to the target amino acid (except `X`).
`[DERIVED]` v2.8.4 warned only through the generic
`oplist.length * 8` span check (§15) and silently accepted a mismatched
triplet; live rejects it with a named message.

### 17.8 Dispatcher deltas (`Tt` v2.8.4 → `tl` live)

Durable diff: `live-js/diffs/q10-dispatcher-v284-vs-live.diff` (207 lines →
183 lines).  Semantic changes, ignoring renaming:

1. **Routing.** v2.8.4: `insert || (replace && (confine5prime || repl.length >= 6 || target.length >= 6))`.
   live adds `|| !/^[acgt]*$/i.test(repl)` — a replacement containing any
   degenerate or non-ACGT symbol now always takes the fragment path, never the
   SDM closure.
2. **Availability limits are threaded into the SDM path.** live computes
   `lim3 = len - end0` and `lim5 = start0 - pad` for partial sequences
   (`Infinity` for circular) and passes them as two new trailing arguments to
   `calcSDMPrimers`; v2.8.4's call has six arguments.  Every growth loop in
   `calcSDMPrimers` and `calculatePrimers` gained the matching bound and a
   `throw new Error("cannot reach min Tm … only N bases available …")`.
3. **Reverse-primer coordinates moved by one base and wrapped.**
   v2.8.4 `locf = Ve - pad`, `locr = Ve - pad - 1`; live
   `locf = (mt - pad + 1) mod len`, `locr = (mt - pad) mod len`.  With
   `Ve = start0 - 1` and `mt = start0 - 1` these differ by exactly +1, and live
   additionally wraps modulo the sequence length.
4. **Deletion length check.** v2.8.4 bails silently when
   `len - (deleted span) < 62`; live computes the remaining length and pushes
   `"deleting N bp leaves only M bp, too little to design primers"`.
5. **Partial-sequence crossing guard.** live, after autogen, throws
   `"cannot reach min Tm N without crossing the end of the partial sequence"`
   when `fwdPrimer.anneal.length > len - end0` or
   `revPrimer.anneal.length > start0 + 1` (linear only).
6. **Errors become messages.** every `console.log("error!!!!!…")` is replaced by
   `warnings.push(id + " (start-end): " + msg)` when the message contains
   `"cannot reach"`, else a generic per-branch message; the dispatcher returns
   the pair array and the caller surfaces the warnings array.
7. **Descriptions.** the inline `"Delete … between bases …"` string building is
   replaced by `describeOp` (17.5), which is also the source of the
   `shorten`ed wild-type text and the `; rotated to start at base N` note.
8. **Pair payload.** live adds `op: <parsed op> || null` to every emitted pair,
   which is what `verifyOp` consumes in 17.2 step 4.

### 17.9 Verified unchanged (round 4 additions)

- `tm(seq, comp)` wrapper defaults are byte-equivalent in both builds:
  `ct = 0.5`, `salt = 150`, `polGroup = "Q5"`, `method = 4` (Phusion → 1).
- `BuilderPrimer` constructor parameter order is `(anneal, spacer, overlap,
  options)` in both builds; the `autogen*` call sites pass
  `(primer, spacer, overlap, options)` in both.  The apparent argument swap in
  the beautified diff is renaming, not reordering.
- `alignDNAglobal` / `alignDNAlocal` scoring parameters and tie-breaking
  (traceback pushes `0,1,2` in that order, last-pushed wins) are unchanged.
- Numeric tables: `dSBr`, `dSSa`, `dHBr`, `dHSa` (16 entries each), `nnde`
  (1635 bytes), `nnloop` (15 entries), `nntmm`, and the substitution/alignment
  `mat` are byte-identical; only `nndhds` (3 keys) and `nnbulge` (key `25`)
  changed, as §13's `tm_tables_diff.py` reports and `q10_tm_tables.py`
  reproduces.

### 17.10 Effect on the open questions in §12

- §12 open question 2 (which route a UI request takes) is now settled for the
  live build: the route is `insert`/`replace`-with-degenerates → fragment path,
  short ACGGT-only replacement → SDM closure, and the SDM closure additionally
  receives explicit flank limits.
- §12 open question 4 (`Primer` 200 vs 500 nM) is unaffected by round 4.
- New `[UNKNOWN]` for the contract: whether `verifyPair`'s round-trip check is
  a *specification* requirement for Luria or an implementation safety net.  It
  rejects pairs rather than adjusting them, so it changes observable behaviour
  (a pair disappears and a warning appears) and must be specified either way.
- New `[UNKNOWN]`: `clash` is computed in `nebuilderFragments` but no message is
  attached to it in that function; the user-visible text for a short fragment
  must be produced elsewhere.  Not resolved this round.

### 17.11 Reproduce

```
cd tests/fixtures/nebasechanger-research
python3 q8_live_vs_284.py            # token-shape verdict (minified bundles)
python3 q9_live_deltas.py            # per-function beautified diff
python3 q9_live_deltas.py verifyPair verifyOp matchesIUPAC classifyOp applyMutation \
       canonicalFlanks normalizeFlanks normalizeOpText   # new-function bodies
python3 q10_tm_tables.py             # numeric table deltas
```

`q9_live_deltas.py` takes function names as arguments; with no arguments it uses
the agreed algorithm surface.  The dispatcher diff was produced by brace-matching
`Tt = function (` at line 17256 of `app-js/beautified/index.a5da1636.v2.8.4.beauty.js`
and `tl = function (` at line 16964 of
`live-js/beautified/index.4330b0c7.v2.11.0-live.beauty.js`.

---

## 18. Help-text deltas and the state-preservation layer (round 5)

Round 5 answers two questions that round 4 left open: what the *in-app
documentation* says changed, and what mechanism implements the v2.10.0/v2.11.0
changelog lines about preserving work across page visits and mode switches.

### 18.0 Method and durable artifacts

The in-app manual is not one string. Svelte compiles it to ~163 small template
literals interleaved with `<br>`, `<i>`, `<a>` markup, so a byte diff of the
manual region is unreadable. `q11_help_diff.py` extracts every template literal
of at least 20 characters, strips tags and entities, collapses whitespace, keeps
prose-like fragments (≥ 4 alphabetic words, ≥ 55 % alphabetic), and compares the
resulting *sets*, pairing near-matches with `difflib` so wording changes are
visible rather than implied.

```
cd tests/fixtures/nebasechanger-research
python3 q11_help_diff.py --full > live-js/diffs/q11-help-prose-deltas.txt
```

`[VERIFIED]` run of that command: exit 0, 18,719 bytes written to
`live-js/diffs/q11-help-prose-deltas.txt`. Counts: 163 prose fragments in each
build, 8 live-only, 8 v2.8.4-only.

`[VERIFIED]` `/tmp/nbc_help_v284.txt` (22,088 B, §15.4's source) no longer
exists; §15.4's verbatim quotations are only recoverable from the bundles or
from the doc itself. Round 5 re-extracts from the bundles instead.

### 18.1 Help-prose deltas (v2.8.4 → live), verbatim

Four documentation changes, all in the live build:

1. **Confine-to-5′-tails paragraph gains a sentence.** v2.8.4's paragraph ends
   `…will be placed in the forward primer with 10 or more bases of annealing
   sequence flanking the inserted bases.` Live appends:
   `This option is always on while "Preserve degenerate bases" is selected.`
   (live offset 344238).
   `[VERIFIED]` this documents behaviour that already existed in v2.8.4:
   `setKeepDegen` is `C && (e(7,(q=!0)), e(17,(H=!1)))` and `setMergeMutations`
   is the same pair; live's equivalents are `q && (t(7,(A=!0)), t(16,(H=!1)))`
   and `k && (t(7,(A=!0)), t(16,(H=!1)))` with the debug logs removed and the
   second reactive index moved 17 → 16. **Documentation only, not a behaviour
   change.**

2. **ORF paragraph gains a sentence.** v2.8.4: `…Setting the ORF start
   position will highlight the ORF.` Live: `…highlight the ORF. Changing it
   keeps the mutations you've entered;` + `any that no longer match the new
   frame are skipped, with a warning.` (live offsets 324488, 324556). This is
   the v2.11.0 changelog line about ORF changes, stated in the help text.

3. **`% GC` removed from the primer-display description** (both copies of the
   paragraph, live offsets 357296 and 583807): v2.8.4 has 2 occurrences of the
   literal `% GC`, live has 0.
   `[VERIFIED]` `fGC` (fraction of g/G/C/c) and `cntGC` are defined identically
   in both builds and **never called in either** (`fGC(` call sites: 0 and 0).
   So v2.8.4's help text was already wrong; live removed the false claim. No
   GC readout exists in either UI.

4. **New paragraph under "Downloadable formats"** (live offset 584440), with no
   v2.8.4 counterpart:
   `Every table and download names primers the same way: ID_n_F and ID_n_R,
   where ID is the mutation's name (for example G10W) and n is the pair's
   number in the Required Primers table. In primer sequences, lowercase bases
   are the mutation and uppercase bases anneal to the template.`
   This is the user-visible specification of §17.5's `Dt.primerId`/`Dt.primerSeq`
   and is the only place the naming scheme is stated in words.

Unchanged wording, `[VERIFIED]` by the same comparison: the substitution-splitting
rules ("longer than 6 nucleotides" → split across both primers; "6 or fewer
nucleotides" → forward primer with ≥ 10 annealing bases), the 35/25 flank
requirement, the batch-mode grammar, the merge-mutations 6-base rule, the
NEBuilder incompatibility notes, and the Legacy-mode note.

Footer/legal fragments differ (live's fragment for the "Tech Support … Site Map"
row ends earlier and the Trademarks/Legal/Privacy/Cookie/Terms links live in a
different fragment). `[UNVERIFIED]` whether that is a layout change or only a
fragment split; it has no bearing on the algorithm contract and was not
pursued.

### 18.2 The state-preservation layer (live only)

`[VERIFIED]` `vt.BUILD_CLEAR_KEYS` and `vt.buildAction` do not exist in v2.8.4
(0 occurrences of either name in `app-js/beautified/index.a5da1636.v2.8.4.beauty.js`).

```
vt.BUILD_CLEAR_KEYS = ["inputmode", "codonops", "rawbatchinput", "seq"];
vt.buildAction = function (prev, cur) {
  if (!prev || prev.tick !== cur.tick) return "build";
  for (k of BUILD_CLEAR_KEYS) if (prev[k] !== cur[k]) return "clear";
  return "build";
};
```

The call site (live offset 499697) is the reactive that recomputes primers:

- `Le = { tick, inputmode, codonops, rawbatchinput, seq }` is the work state;
- `buildAction(prev, cur) == "clear"` (any of those four fields changed, same
  tick) → primers reset to `[]`, warnings reset, plus `pe = seq.length < 60`
  and `Y = !!sequenceLengthError(seq.length)`;
- `"build"` (tick changed, or none of the four changed) → recompute from the
  current state.

`[DERIVED]` this is the mechanism behind the v2.10.0 line "Work (sequence,
mutations, settings and primers) is now kept when you visit Help, About,
Resources or Troubleshooting and come back" and the v2.11.0 line "Pressing
Enter in the mutations box builds primers; switching modes keeps a minimum
primer length you set": settings that are *not* in `BUILD_CLEAR_KEYS` —
including primer minimum length/Tm — never trigger a clear, so they persist
across rebuilds.

### 18.3 Named constants (live only), with values

| Constant | Value | Meaning |
| --- | --- | --- |
| `vt.MIN_FLANK_5` | `35` | minimum 5′ flank for partial (linear) sequences |
| `vt.MIN_FLANK_3` | `25` | minimum 3′ flank |
| `vt.NB_MIN_FRAGMENT` | `60` | minimum NEBuilder fragment length (`clash`) |
| `vt.ORF_MIN_CODONS` | `10` | minimum codons between stop and ATG for an ORF |
| `Dt.FASTA_WIDTH` | `70` | FASTA line wrap |

`[VERIFIED]` `flankError` defaults to `MIN_FLANK_5`/`MIN_FLANK_3` and emits the
messages `insufficient 5' flanking sequence to build primers for partial
sequence (need at least 35 bp before the edit, have N)` and the 3′ analogue, so
the 35/25 rule quoted from v2.8.4's help text is now enforced in code with
those exact numbers.

`[VERIFIED]` `vt.orfStarts` scans `[ACGT]+` runs, records the last stop
(`TAA`/`TAG`/`TGA`) per frame, and accepts an `ATG` only if a stop occurred at
`d % 3` at least `3 * ORF_MIN_CODONS` bases earlier; it returns 0-based offsets
into the run. v2.8.4 has no `orfStarts` by name.

### 18.4 Reproduce (round 5)

```
cd tests/fixtures/nebasechanger-research
python3 q11_help_diff.py --full      # also written to live-js/diffs/q11-help-prose-deltas.txt
grep -c "% GC" app-js/beautified/index.a5da1636.v2.8.4.beauty.js   # 2
grep -c "% GC" live-js/beautified/index.4330b0c7.v2.11.0-live.beauty.js  # 0
grep -c "BUILD_CLEAR_KEYS" app-js/beautified/index.a5da1636.v2.8.4.beauty.js  # 0
```

### 18.5 New `[UNKNOWN]` for the contract

- `BUILD_CLEAR_KEYS` names the four fields that force a rebuild-clear. Luria's
  work-state model must decide which of its own fields play the same role; the
  live build's choice (`inputmode`, `codonops`, `rawbatchinput`, `seq`) is
  evidence, not a requirement.
- `NB_MIN_FRAGMENT = 60` is the threshold behind `clash`, but §17.10's open
  question stands: no user-visible message is attached to `clash` inside
  `nebuilderFragments`.
