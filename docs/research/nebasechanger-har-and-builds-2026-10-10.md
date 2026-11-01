# NEBaseChanger HAR capture and archived-build comparison (2026-10-10)

Status: research report. Read-only investigation of fixture data. No Luria
source, Rust code, dependency, branch, or configuration was modified. The only
file created by this task is this report.

Date: 2026-10-10.
Target fixture: `tests/fixtures/nebasechanger-research/nebasechanger.neb.com_Archive [26-10-10 01-47-35].har`

Evidence tags:

| Tag | Meaning |
| --- | --- |
| `[HAR]` | Read from the HAR file named above. |
| `[BUNDLE-2.8.4]` | Read from `app-js/index.a5da1636.v2.8.4.js`. |
| `[BUNDLE-2.7.2]` | Read from `app-js/index.dfcf8292.v2.7.2.js`. |
| `[BUNDLE-2.5.3]` | Read from `app-js/index.17c7600d.v2.5.3.js`. |
| `[MANIFEST]` | Read from `tests/fixtures/nebasechanger-research/manifest.json`. |
| `[CHECK]` | A command I ran in this session, with its result. |
| `[PRIOR]` | Claim from [docs/research/nebasechanger-client-investigation.md](/research/nebasechanger-client-investigation.md) (2026-10-09, Arquivo.pt v2.5.0/v2.4.x bundles); not re-derived here unless tagged. |
| `[UNKNOWN]` | Not established. |

Cloud-sharing warning (D023): this report contains no sequence-bearing
biological fixture content. It is still treated as research material; do not
send to a cloud model without explicit approval.

---

## 1. Does the HAR contain response bodies, and JavaScript bodies?

Yes, partially. The HAR contains five non-empty response bodies, but **none of
them is NEBaseChanger application JavaScript**. The NEBaseChanger bundle
appears in the HAR only as a request/response header pair with an empty body.

HAR envelope `[HAR]`:

- `log.version`: `1.2`; `creator`/`browser`: Firefox `157.0.1`.
- `log.pages`: 1 page, `https://nebasechanger.neb.com/`,
  `startedDateTime` `2026-10-10T01:44:34.121-07:00`, `onContentLoad` 311 ms,
  `onLoad` 329 ms.
- `log.entries`: 11 entries.
- Header objects use `{name, value}` and are **not** redacted (see §3).

Per-entry inventory `[HAR]` (`bodySize` is the HAR field; `content.size` is the
transferred size; `text_len` is the stored body length in characters):

| # | URL (abbreviated) | Status | `bodySize` | `content.size` | `text_len` | Body present |
| -- | --- | --- | --- | --- | --- | --- |
| 0 | `cdn.cookielaw.org/.../otSDKStub.js` | 200 | 186581 | 0 | 0 | **no** |
| 1 | `nebasechanger.neb.com/assets/index.4d8eac54.js` | 200 | 1196535 | 0 | 0 | **no** |
| 2 | `static.cloudflareinsights.com/beacon.min.js/...` | 0 | 0 | 0 | 0 | no (blocked; `content.encoding` key present) |
| 3 | `cdn.cookielaw.org/consent/<uuid>/<uuid>.json` | 200 | 2324 | 2324 | 8395 | yes |
| 4 | `nebasechanger.neb.com/cdn-cgi/challenge-platform/scripts/jsd/main.js` | 302 | 11238 | 0 | 0 | no (`content.comment` only; `redirectURL` set) |
| 5 | `cdn.cookielaw.org/.../202508.1.0/otBannerSdk.js` | 200 | 1188106 | 0 | 0 | **no** |
| 6 | `nebasechanger.neb.com/cdn-cgi/challenge-platform/h/g/scripts/jsd/4df4a60fa397/main.js?` | 200 | 11238 | 11238 | 21985 | yes |
| 7 | `cdn.cookielaw.org/consent/<uuid>/.../en.json` | 200 | 25522 | 25522 | 143825 | yes |
| 8 | `nebasechanger.neb.com/cdn-cgi/challenge-platform/h/g/jsd/oneshot/...` | 200 | 16668 | 0 | 0 | no (request has `postData`, 13966 chars) |
| 9 | `cdn.cookielaw.org/.../assets/otFlat.json` | 200 | 3145 | 3145 | 15345 | yes |
| 10 | `cdn.cookielaw.org/.../assets/v2/otPcCenter.json` | 200 | 13271 | 13271 | 66689 | yes |

So the bodies that are present are OneTrust/CookieLaw consent-SDK JSON (entries
3, 7, 9, 10) and one Cloudflare bot-challenge script (entry 6). Entries 0, 1, 5
are JavaScript responses whose bodies were **not captured** (`content.size` 0,
`content.text` `""`) even though `bodySize` is large.

Consequence for research: **the HAR cannot be used to recover NEBaseChanger
algorithms.** The only NEBaseChanger payload in it is metadata about the live
bundle.

## 2. What the HAR does contribute

Entry 1 is the only NEBaseChanger asset record `[HAR]`:

- URL: `https://nebasechanger.neb.com/assets/index.4d8eac54.js`
- `startedDateTime`: `2026-10-10T01:44:34.122-07:00`, HTTP/2, status 200.
- Response headers include `content-type: application/javascript`,
  `content-encoding: gzip`, `last-modified: Sat, 10 Oct 2026 01:35:35 GMT`,
  `etag: W/"6ac99667-60435"`, `cache-control: public, max-age=14400`,
  `cf-cache-status: BYPASS`.
- `bodySize`: `1196535`; `content.size`: `0`.

Findings:

1. **The live build hash is `index.4d8eac54.js`, last modified 2026-10-10.**
   This is newer than every bundle available in the Internet Archive
   `[MANIFEST]`, and `[MANIFEST]` records that CDX returns 404 for it. The
   archived `index.a5da1636.js` (v2.8.4) is the nearest available proxy.
2. **Size discrepancy — reported, not resolved.** The Cloudflare-style weak
   ETag suffix `60435` hex-decodes to 394293 bytes, close to the archived v2.8.4
   bundle size (395491 bytes `[CHECK]`). The HAR `bodySize` is 1196535, which is
   not consistent with either number. HAR `bodySize` semantics are inconsistent
   across entries in this file (entry 9: `bodySize` 3145 equals the compressed
   `content-length` 3145 while `content.text` is 15345 chars; entry 5: `bodySize`
   1188106 versus `content-length` 122614). I did not resolve which number is
   authoritative. Treat the live bundle size as **unknown**, with 394293 as the
   ETag-derived candidate.
3. **The capture is page-load only.** All 11 entries are page/consent/challenge
   assets; `onLoad` is 329 ms. There are no tool-usage requests. Therefore the
   HAR does **not** demonstrate that primer design is client-side; it is
   consistent with the client-side claim but does not evidence it. The
   client-side architecture claim remains `[PRIOR]` plus the bundle reading in
   §5–§6.
4. The HAR confirms third-party dependencies (OneTrust `202508.1.0`,
   Cloudflare Insights, Cloudflare challenge platform) and Firefox 157 as the
   capture browser.

## 3. Fixture integrity problem — needs a decision, not silently fixed

`manifest.json` asserts `[MANIFEST]`: `"privacy": "Contains NO request headers,
cookies, cf_clearance, Authorization, or OptanonConsent values."`

That assertion is **false**. Verified `[CHECK]` from entry 1:

- `request.headers` contains a full `Cookie` header whose value carries
  `cf_clearance=…`, `OptanonConsent=…`, `_ga_V6Q0SZ7FRP=…`, `_ga=…`,
  `OptanonAlertBoxClosed=…`.
- `request.cookies` additionally parses the same five cookies with
  `{name, value}` intact.
- Entry 8's response `Set-Cookie` value issues a **different** live
  `cf_clearance` with `Domain=neb.com; Expires=Sun, 10 Oct 2027 …`, and entry 8's
  request carries a 13966-character Cloudflare challenge `postData`.
- `User-Agent`, `Referer`, `Sec-Fetch-*`, and `Host` are also intact.

So the fixture as committed contains live Cloudflare clearance material and
visitor identifiers. This is a privacy/fixture-integrity defect, not a
biological one. I did **not** modify the HAR or the manifest, because AGENTS.md
forbids scope expansion without approval and because redaction changes fixture
bytes that other tasks may depend on. Decision needed from the maintainer
(see §8).

Also note the HAR is a Firefox export, not a sanitized capture: the filename
says `Archive` but `log.creator` is Firefox 157.0.1, and `[MANIFEST]` provenance
describes Internet Archive replays for the JS bundles. The HAR and the bundles
have **different provenances**; the HAR is a local browser capture.

## 4. Build lineage available in the repository

`[MANIFEST]` plus `[CHECK]` (sha256 of `app-js/*.js` equals sha256 of
`raw/*.min.js`, i.e. the copies are byte-identical):

| Build | Bundle hash | Version | Archive snapshot | Bytes | sha256 (first 16) |
| --- | --- | --- | --- | --- | --- |
| newest archived | `index.a5da1636.js` | v2.8.4 | `20260730111722` | 395491 | `82db4e9675f0ecb2` |
| prior | `index.dfcf8292.js` | v2.7.2 / v2.7.3 | `20260417033937` | 375662 | `73d21fb469e93785` |
| older | `index.17c7600d.js` | v2.5.3 | `20250622130245` | 292233 | `c40e727e55258816` |
| live (2026-10-10) | `index.4d8eac54.js` | unknown | **not archived** | unknown | — |

Two SPA shells are also captured (`shell.2026-07-30.html`,
`shell.2026-04-17.html`) `[MANIFEST]`; each body is `<div id="app">` and all UI
comes from the single `type=module` JS bundle.

## 5. Algorithm stability across builds

Verified by direct reading and by structural comparison `[CHECK]`:

- `calcSDMPrimers` is **structurally identical** between v2.5.3 and v2.8.4,
  differing only in minifier identifier names and one added
  `console.log("calcSDMPrimers *************")` `[CHECK]`.
- The Needleman–Wunsch scoring object is **identical** in v2.5.0, v2.5.3, v2.7.2
  and v2.8.4 `[CHECK]`:

  ```
  mat = {a:{a:2,c:-1,g:1,t:-1}, c:{a:-1,c:2,g:-1,t:1},
         g:{a:1,c:-1,g:2,t:-1}, t:{a:-1,c:1,g:-1,t:2}}
  gap = -2.5, term = -2.5, extend = -1
  ```

- The `tm` wrapper is unchanged in shape `[BUNDLE-2.8.4]`: `ct = 0.5`,
  `salt = 150`, method `4` (method `1` for `Phusion` / `Phusion Hot Start Flex`).
- The v2.7.2 → v2.8.4 diff adds **UI**, not algorithms: a Q5-SDM
  Troubleshooting Guide state machine, a table-of-contents component, and sample
  vector list updates `[CHECK]`.

Conclusion: the primer-design core documented for v2.5.0 `[PRIOR]` still
describes v2.8.4. The live build `4d8eac54` is unverified.

## 6. v2.8.4 primer construction, read from the bundle

Signature `[BUNDLE-2.8.4]`:

```
Ge.calcSDMPrimers = function (E, ee, ne, ae, ue, ie)
```

Call site `[BUNDLE-2.8.4]` (the `seqop == "replace"` branch):

```
let Oe = Ge.calcSDMPrimers(a.template, ge, Te, ot, ee, Pe);
Oe.ta = Math.min(Oe.tmf, Oe.tmr) + 1;
Oe.ta > 72 && (Oe.ta = 72);
```

Parameter reading `[BUNDLE-2.8.4]`:

| Param | Role | Evidence in body |
| --- | --- | --- |
| `E` | template sequence | `Ve = Ge.revcomp(E)`, `E.slice(...)` |
| `ee` | minimum primer length | `ee = ee < 15 ? 15 : ee` |
| `ne` | minimum Tm | `ne = ne < 55 ? 55 : ne; ne = ne > 72 ? 72 : ne` |
| `ae` | offset: index in `E` where the replaced region begins | `Ne = ae + w.length`; `E.slice(0, ae)` is the 5' prefix |
| `ue` | labelled `repseqtpl` | **only used in `console.log("repseqtpl", ue)`** |
| `ie` | labelled `repseq` (replacement) | `new mi.Alignment("global", ie, w, At)` |

**Ambiguity to report, not resolve (AGENTS.md rule).** The alignment and the
template arithmetic use a closure variable `w`, not the `ue` parameter. `w` is
assigned in the enclosing component as the template substring between two
mutation coordinates, e.g. `w = E.slice(ue + 1, ie)` and, for a wrap-around
case, `w = E.slice(ue + 1) + E.slice(0, ie)`. So `ue` appears to be a vestigial
parameter. The coordinate convention (`ue + 1`, inclusive/exclusive ends,
1-based vs 0-based) is **not established from the bundle alone**, and the
`isCircular` rotation branch rewrites `a.subseq` differently from the linear
branch. I am not free to choose a convention; this must be pinned by a live
capture or by asking.

Also note the initial 3' flank length is `J = ae` (the offset), not a fixed
`pad = 10` as documented for v2.5.0 `[PRIOR]`. Whether `ae` is always 10 in the
rotated-template case is `[UNKNOWN]`.

Extension and balancing rules `[BUNDLE-2.8.4]`, verbatim constants:

- forward primer: `E.slice(0, ae) + Pe.alnSeq1 + E.slice(Ne, Ne + J)`, grown
  while `Tm < minTm`;
- GC clamp: grow while `Tm < 63` and primer does not match `/[GgCc]$/`; if the
  GC terminus is not reachable, revert to the shortest Tm-satisfying primer
  (tracked by `Te`);
- reverse primer: `Ve.slice(0, ge)` with the same growth, GC clamp, and revert
  rule (tracked by `Xe`);
- balance: while `tmf - tmr > 5` extend the reverse primer; symmetric while
  `tmr - tmf > 5` extend the forward primer; then a GC-ending sub-loop that
  reverts to the last non-overshooting length;
- output: `ce.pfwd`/`ce.prev` have `-` stripped; returns `{pfwd, prev, tmf,
  tmr, prev}`;
- `Ta = Math.min(tmf, tmr) + 1`, capped at 72, computed at the call site.

Tm model `[BUNDLE-2.8.4]` — the full duplex Boltzmann calculator
(`createTmCalc`), including the loop/bulge machinery that v2.5.0 also had:

- gas constant `R = 1.987`; `primerc = ct * 1e-6`; `saltc = salt * 1e-3`;
Two implementations exist in v2.8.4. The **live** one is the `Tm` closure
returned by `createTmCalc` (17509 chars); the **dead** one is
`TmCalc.prototype.calcTm` (7982 chars, zero call sites). Their constants
overlap but are not identical, so they are listed separately.

Live path (`Tm`, method 4 for Q5) `[BUNDLE-2.8.4]`:

- `R = 1.987`; `primerc = ct * 1e-6`; `saltc = salt * 1e-3`; `fgc = gc_cnt/len`;
- method 1: `dS_init = sym ? -12.4 : -10.8`; sum `dSBr`/`dHBr` over adjacent
  pairs; `dH *= 1e3`; `Tm = dH/(dS_init + dS + R·ln primerc) - 273.15 + sc_sch`;
- method 3/4: `dS_init = -5.7`, `dH_init = 200`, `dS += sym ? -1.4 : 0`,
  `dS_init += 6.9` and `dH_init += 2200` for each terminal `a`/`t`;
  pair-table `nndhds` (176 match **and mismatch** entries, keyed
  `w1w2/c1c2`), duplex `loop` table (keys 3,4,5,6,7,8,9,10,12,14,16,18,20,25,30)
  with asymmetry penalty `dS -= |nW - nC| * 0.97`, `bulge` table (keys 1…30);
  `Tm = (dH*1e3 + dH_init)/(dS_init + dS + dS_init_extra + R·ln primerc)`;
- salt correction: `sc_ow = (4.29·fgc - 3.95)·1e-5·ln(saltc) + 9.4e-6·ln(saltc)²`,
  `sc_sl = 0.368·len·ln(saltc)`, `sc_sch = 16.6·log10(saltc)`;
  method 3 → `Tm = 1/(1/Tm + sc_sl/(dH*1e3 + dH_init))`, method 4 →
  `Tm = 1/(1/Tm + sc_ow)`; then `Tm -= 273.15`;
- method 7 (unused by `tm`): `Tm = 81.5 + 16.6·log10(saltc) + 0.41·fgc - 675/len`;
- method 8 (unused by `tm`): `Tm = 4·gc_cnt + 2·at_cnt` (WC formula);
- final: `Tm -= 0.6·dmso` (`dmso` default 0);
- alignment is built by `buildPairMap`: `|` matched, `^` gap, `x` mismatch, then
  `_` promotion passes for adjacent mismatches/gaps.

Dead path (`calcTm`) `[BUNDLE-2.8.4]` — same method numbering but a different
property shape (`nnde`, `nntmm`, `nnloop`, `nnbulge`, `nndhds`) and different
loop arithmetic: dangling-end corrections via `nnde`, terminal-mismatch
`nntmm`, internal loops scanned from the pairmap with regexes `/[^_]_+[^_]/`,
`/[^=]=+[^=]/` (asymmetric: `dS += |G-R| * 0.3 * 1e3 / 310.15`), `/[^\^]\^+[^\^]/`
bulges, and `+0.5` per AT/TA loop terminus. It also returns `rawtm`, `fgc`, and
`len`. Because it is unreferenced, **none of these `nnde`/`nntmm`/`nnloop`/`nnbulge`
values are live behaviour** and they must not be transcribed into Luria as the
active model.
- method 7 (unused by `tm`): `Tm = 81.5 + 16.6·log10(saltc) + 0.41·fgc - 675/len`;
- method 8 (unused by `tm`): `Tm = 4·gc_cnt + 2·at_cnt` (WC formula);
- final: `Tm -= 0.6·dmso` (`dmso` default 0).

Call-count evidence `[CHECK]` on the minified v2.8.4 bundle: `.Tm(` → 1 (the
`tm` wrapper); `prototype.calcTm` → 2 definitions (`calcTm`, `calcTm_nomm`);
`d.calcTm_nomm(m)` → 1 call inside a `Primer` helper. `calcTm` itself is
referenced nowhere, hence the "dead" classification.

## 7. In-app changelog, v2.0.0 → v2.8.4

Extracted verbatim from the v2.8.4 bundle `[BUNDLE-2.8.4]` (the `Updates`
`<dl>`; version constant `let Gc="2.8.4"`):

| Version | Date | Change (abridged) |
| --- | --- | --- |
| v2.8.4 | July 10 2026 | Repaired file downloading issue when mutation was a deletion |
| v2.8.3 | June 23 2026 | Added Q5-SDM troubleshooting module |
| v2.7.3 | March 2 2026 | Help edit: when the NEBuilder option is accessible |
| v2.7.2 | March 2 2026 | Repaired reverse primer with partial sequence input; rev primer 5' end off by 1 nt |
| v2.7.1 | February 17 2026 | Repaired misplaced reverse primer with partial sequence input |
| v2.7.0 | February 9 2026 | Added link to NEBuilder Calculator when NEBuilder primers generated |
| v2.6.10 | February 6 2026 | Repaired broken Save Summary when NEBuilder primers generated |
| v2.6.9 | December 12 2025 | Bundled release: partial template sequences; merged mutations in Codon mode; Legacy mode emulating NEBaseChanger V1; text selection of target sequences; point-and-click codon selection; explicit triplet specification; ORF start reset / ORF auto location / OR… |
| v2.5.4 | July 15 2025 | Revised favicon |
| v2.5.3 | June 11 2025 | Removed extraneous underscore from primer ID; removed link to NEBaseChanger v1 |
| v2.5.2 | October 14 2024 | Updated Help text |
| v2.5.1 | July 5 2024 | New NEB logo |
| v2.5.0 | April 16 2024 | Added option to adjust primer minimum Tm and minimum length |
| v2.4.4 | February 15 2024 | Fixed download of final mutated sequence in AA mode |
| v2.4.3 | January 24 2024 | Fixed underscores inserted into Primer Set ID |
| v2.4.2 | October 17 2023 | Added option to preserve degenerate base codes in primers |
| v2.4.1 | September 28 2023 | Fixed minimum primer length falling below 15 with 5' tail confinement |
| v2.4.0 | September 27 2023 | Fixed minimum length below 15 with 5' tail confinement; `X` amino acid → `NNK` (was `NNN`) |
| v2.3.2 | September 7 2023 | Default NEBuilder primer type selection → 'multiple' |
| v2.3.1 | August 4 2023 | Raised minimum primer length to 15 nt |
| v2.3.0 | July 17 2023 | 'All' option in AA dropdown (≡ `NNN`); help dialog for one-letter codes; exclusivity between NEBuilder HiFi generation and 5' tail confinement |
| v2.2.1 | June 27 2023 | UI/documentation changes |
| v2.2.0 | June 26 2023 | NEBuilder HiFi-compatible primer generation for AA point mutation mode |
| v2.1.1 | June 7 2023 | Fixed/restored missing Gln→AA replacement list |
| v2.1.0 | June 1 2023 | Codon selection strategies for AA point mutation mode |
| v2.0.1 | May 23 2023 | Repaired OneTrust cookie manager |
| v2.0.0 | May 17 2023 | Initial NEBaseChanger V2 release |

Research-relevant version boundaries for Luria: **v2.3.1** (min length 15),
**v2.4.0** (`X → NNK`), **v2.4.2** (preserve degeneracy), **v2.5.0**
(adjustable min Tm/length), **v2.6.9** (partial templates, legacy V1 mode),
**v2.7.1/v2.7.2** (reverse-primer coordinate repairs with partial input — i.e.
the coordinate convention changed twice in 2026).

## 8. New v2.8.4 non-algorithm content

The Q5-SDM Troubleshooting Guide (added v2.8.3) is a declarative decision tree
`[BUNDLE-2.8.4]`, `meta: { name: "Q5-SDM Troubleshooting Guide", initialState:
"S1", version: "0.10" }`, states S1–S11 with `output`, `isTerminal`
(`S2` uses the snake-case key `is_terminal`, an inconsistency in the data), and
`transitions`. It is UI/diagnostic text, not primer design, but it is the only
place NEB encodes its own PCR-failure decision logic, so it is worth capturing
for Luria's explanation layer. Navigation is implemented as a path stack:
`h(q)` pushes the current state and moves to `n.transitions[q]`; `d()` pops;
`p()` resets to `meta.initialState`.

## 9. Checks run in this session

| Command (abbreviated) | Result |
| --- | --- |
| `python3` parse of the HAR, per-entry `bodySize`/`content.size`/`text_len` | table in §1; 5 bodies, 6 empty |
| `python3` dump of entry 1 `request.headers` and `request.cookies` | full `Cookie` header and 5 parsed cookies incl. `cf_clearance` (§3) |
| `sha256sum app-js/*.js` and `raw/*.min.js` | three matching pairs (§4) |
| `grep -n "calcSDMPrimers" beautified/index.a5da1636…beauty.js` | call site at line 17058 |
| `python3` brace-matched extraction of `calcSDMPrimers = function` from v2.8.4 | 4354 chars, quoted in §6 |
| `python3` brace-matched extraction of `createTmCalc = function` / `calcTm = function` | 17509 / 7982 chars, constants in §6 |
| `python3` regex counts of `.Tm(`, `prototype.calcTm`, `calcTm_nomm` | 1 / 2 / 1 → `calcTm` unreferenced (§6) |
| `python3` extraction of the `Updates` `<dl>` | changelog table §7 |
| `python3` extraction of `expandDNASeq` | 485 chars; Cartesian product over `DEGENERATES`; **no size cap found** (an earlier note about a 10000-item cap was not reproduced here) |
| `grep -n "^#{1,3} "` of the prior report | section map used for cross-referencing |

## 10. Assumptions, unverified behaviour, deviations

Assumptions:

- The Internet-Archive bundles in `app-js/` are the real NEB builds, as recorded
  by `[MANIFEST]`. I did not re-fetch them.
- `app-js/*.js` and `raw/*.min.js` are the same bytes (verified by sha256).

Unverified:

- The live build `index.4d8eac54.js` was never read; every algorithm claim is
  about v2.8.4 (2026-07-30 snapshot), while the HAR capture is 2026-10-10. A
  behavioural change between those dates is possible and unmeasurable from this
  fixture set.
- No NEBaseChanger code was executed in this session; §5–§6 are code-reading
  only. Numeric agreement with captured NEBC output is `[PRIOR]`, not re-derived
  here.
- The `w`/`ue` parameter ambiguity in `calcSDMPrimers` (§6) and the
  `isCircular` rotation convention are unresolved and deliberately not chosen.
- `Oe.tmf.tm` at the call site reads `.tm` off a value that `calcSDMPrimers`
  assigns as a number; I did not determine whether that is a logging-only bug.

Deviations from the task contract: none in scope — the task was analysis of the
HAR, and that is answered in §1–§2. The fixture privacy defect in §3 was
reported rather than repaired.

## 11. Live capture attempt, 2026-10-10 (supersedes the HAR's asset URL)

Tool: `tests/fixtures/nebasechanger-research/capture-js.mjs` (research-only;
`capture.mjs` is the superseded earlier version, kept for provenance of
`nebasechanger-js/`). It listens on `page.on("response")`, awaits
`response.body()` — Playwright's `Response2.internalBody()` does
`await this._finishedPromise`, so the body is read only after the response
finishes streaming — records URL, status, content type, byte length, SHA-256
and capture timestamp in `live-capture/manifest.json`, and never writes
headers, cookies, clearance tokens, consent values or request POST bodies
(a self-check over the manifest aborts if those markers appear).

Commands run (both exit non-zero, i.e. no live bundle obtained):

```
node capture-js.mjs --out live-capture \
  --seed https://nebasechanger.neb.com/assets/index.4d8eac54.js   # exit 2
node capture-js.mjs --out live-capture-profile --profile browser-profile  # exit 2
node capture-js.mjs --verify app-js          # exit 0 (3/3 JavaScript)
node capture-js.mjs --verify app-js/beautified  # exit 0 (3/3 JavaScript)
node capture-js.mjs --verify live-capture     # exit 2 (1/1 NOT JavaScript)
```

Observed, fresh headless Chromium (Playwright 1.58, Node 26.11.1):

- Navigation to `https://nebasechanger.neb.com/` stops at a Cloudflare
  Turnstile interstitial: `page.content()` is 110,179 bytes whose text
  contains `Just a moment...`, `cf-chl` and `challenges.cloudflare.com`
  (SHA-256 `e0dfa1917705edd8c1cf6a919ddf7b0e9617e01e4969566a2b4036ce0bacae47`).
  That HTML is **not saved** — it is challenge material, not the app shell —
  so the current bundle reference was never observed.
- 11 requests were seen and excluded (challenge-platform, Turnstile, `blob:`).
- Direct navigation to the HAR's asset URL reaches nginx and returns **HTTP
  404**, 916 bytes, `text/html`, body `<title>404 Not Found</title>` …
  `nginx/1.28.0` (SHA-256 `5f43d215d44c8d33cb489b402f1b1dcd3963b3ead29d2f7927f32d84968a9980`).
  So `index.4d8eac54.js` is **no longer served**; the site was redeployed
  between the HAR capture (01:47:35) and 12:09 local. Asset paths are not
  challenge-gated, only the HTML document.
- Verification of the already-saved fixtures passes: `app-js` 292,233 B
  (`c40e727e…`), 395,491 B (`82db4e96…`), 375,662 B (`73d21fb4…`); beautified
  457,064 / 614,639 / 584,835 B. All classify as JavaScript.

Consequence: the live current build cannot be identified from this environment.
The next step that can work is a human-in-the-loop run —
`node capture-js.mjs --profile browser-profile --no-headless --wait-timeout 300000`
(solve the challenge in the open window; the profile dir is gitignored via
`tests/fixtures/nebasechanger-research/.gitignore`) or
`--cdp http://127.0.0.1:9222` against an existing browser session. No archived
build was substituted for the live one.

## 12. Recommended next steps (ranked)

1. Decide the fixture privacy policy for §3: redact `Cookie`/`cookies`/
   `Set-Cookie`/`postData` in the HAR and correct the `manifest.json` privacy
   string, or move the HAR out of the repository. This blocks nothing else but
   should happen before the fixture is referenced by any test.
2. Obtain the live build through an approved route. `index.4d8eac54.js` from
   the HAR now returns HTTP 404 (§11), so the current hash must be re-discovered
   from the SPA shell — a human-in-the-loop capture (§11) is the shortest route.
   Alternatively accept v2.8.4 as the reference and record that decision in
   [docs/decisions/](/decisions/). Do not substitute an archived build and label it current.
3. Pin the `calcSDMPrimers` coordinate convention with a live capture that
   exercises: substitution ≤6 nt, substitution ≥6 nt, insertion, deletion
   (v2.8.4 changelog says deletions changed file download, so deletion is a live
   edge case), and a wrap-around mutation on a circular template.
4. Only after 1–3: implement the sequence/feature/coordinate models in
   `crates/core` per the milestone-1 contract.
