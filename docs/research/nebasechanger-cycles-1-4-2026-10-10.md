# NEBaseChanger research — cycles 1–4 (2026-10-10)

Covers the four priority questions: runtime Tm-model selection, `ct` units,
the `maxTm` field, and thermodynamic-table differences. Every claim is
labelled `[LIVE]` (live v2.11.0 capture), `[v2.5.3]`/`[v2.7.2]`/`[v2.8.4]`
(archived builds), `[NODE]` (locally executed), `[PTM]` (`polymerase-tm`
reference implementation), `[NEB-DOC]` (in-app manual), `[DERIVED]`
(arithmetic on captured values), `[INFERRED]` (hypothesis), `[UNKNOWN]`.

Byte offsets are into the **minified** bundle named at the top of each
section. Beautified files are for reading only.

## Provenance and integrity

| Build | File | Bytes | sha256 |
|---|---|---|---|
| v2.5.3 | `tests/fixtures/nebasechanger-research/raw/index.17c7600d.v2.5.3.min.js` | 292233 | `17c7600d…` |
| v2.7.2 | `raw/index.dfcf8292.v2.7.2.min.js` | 375662 | `dfcf8292…` |
| v2.8.4 | `raw/index.a5da1636.v2.8.4.min.js` | 395491 | `a5da1636…` |
| live v2.11.0 | `live-capture/004eae75971a-index.4330b0c7.js` | 401373 | `4330b0c7…` |

`tests/fixtures/nebasechanger-research/manifest.json` records the full digests
and the fetch method for each. Reproduce with `sha256sum <file>`.

---

## 1. Runtime Tm-model selection (Q1)

### 1.1 Two independent Tm engines exist in every build

`[LIVE]` `live-capture/004eae75971a-index.4330b0c7.js`:

1. **`createTmCalc`** @ 76574..87147 — a closure factory. Its returned object
   is `{setSeq:R, setSalt:ae, setCt:Q, setDMSO:W, setMethod:L, saltCorrect:D,
   Tm:x, …}`. The `Tm` method is the minified local `x`.
2. **`TmCalc`** — a class-like constructor @ 32042 with prototype methods
   including `calcTm` and `calcTm_nomm` (@ 45211). Reached only through
   `createTmFunction` @ 46722.

They are **not** equivalent. Both are reachable, by different routes.

### 1.2 The Q5 SDM route uses `createTmCalc`, and method 4 is forced

`[LIVE]` `Ne.tm` @ 322456..322710 (254 bytes), verbatim:

```js
Ne.tm = function(I, X) {
  var re = .5, se = "Q5", Se = 150, Me = 4;
  X = X || Ne.revcomp(I);
  var Le = Ne.createTmCalc({seq: I, ct: re, salt: Se, compseq: X});
  switch (se) {
    case "Phusion": case "Phusion Hot Start Flex": Me = 1; break;
    default: Me = 4
  }
  Le.setMethod(Me);
  var je = Le.Tm(), oe = je.tm;
  return oe
};
```

Findings:

* `se` is a **local literal `"Q5"`**, never reassigned: `[LIVE]` the only two
  occurrences of the token `se` inside `Ne.tm` (brace range 322456..322710,
  254 bytes) are its declaration and `switch(se)`. Therefore `switch(se)` always
  takes `default:` and `Me = 4`. **The `Me = 1` (Phusion) branch is dead code in
  all four builds** — `[v2.5.3]` `ge.tm`, `[v2.7.2]` `Ee.tm`, `[v2.8.4]` `Ge.tm`
  have the identical shape. `[INFERRED]` the branch is a leftover from an
  earlier build where `polGroup` was a parameter.
* `ct = 0.5`, `salt = 150` are literals, not read from any UI/config object.
  `[LIVE]` `[v2.8.4]` `[v2.7.2]` `[v2.5.3]` all identical modulo minifier names.
* `createTmCalc`'s own default `method` is `1` (`a = l.method || 1`), but
  `Ne.tm` never passes `method`; it calls `setMethod(4)` afterwards. So the
  effective method is **4**, and the `method||1` default is never observed on
  this path. `[LIVE]`

### 1.3 What `case 4` actually computes (the closure `Tm`, live)

`[LIVE]` inside `createTmCalc`, the `switch(this.method)` has cases `1`,
`3`/`4` (fallthrough), and `7`. **There is no `case 2` and no `case 5`**
(`[RUN]` `q1b_createTmCalc.py live`: cases in the closure `Tm` = `[1, 3, 4, 7]`,
cases in the whole `createTmCalc` = `[1, 3, 4, 7]`).

`case 3`/`case 4` (verbatim, condensed):

```js
te = this.sym ? -1.4 : 0;      // self-complementarity penalty
ie = -5.7;  z = 200;           // initiation  ΔS, ΔH
(he[0] === "a" || he[0] === "t") && (ie += 6.9, z += 2200);
(…last char a/t…)              && (ie += 6.9, z += 2200);
// nearest-neighbour sum over pairmap positions where both neighbours are
// aligned ("|" or "x"), looked up in nndhds keyed "ab/cd":
//   le += nndhds[key].ds ; J += nndhds[key].dh
U = (J*1e3 + z) / (ie + te + le + this.R*Math.log(pe));
// then internal loops (this.loop), asymmetric loops, bulges (this.bulge),
// each also contributing nndhds terms; finally:
J *= 1e3;
U = (J + z) / (ie + te + le + this.R*Math.log(pe));
this.method === 3 ? U = 1/(1/U + this.sc_sl/(J+z))
                  : this.method === 4 && (U = 1/(1/U + this.sc_ow));
U -= 273.15;
```

So the SDM path is **mismatch-aware**: it builds a `pairmap` (`buildPairMap`,
local `K`) and uses `nndhds`, `loop`, `bulge`. `[LIVE]` `[v2.8.4]` `[v2.7.2]`
`[v2.5.3]` — this structure is unchanged across builds; only minifier names
differ.

### 1.4 What `calcTm_nomm` computes, and where it is reachable

`[LIVE]` `TmCalc.prototype.calcTm_nomm` @ 45211. Its `switch(method)` has
cases `1`, `3`, `4`, `5`, `7` — **no `case 2`**. Its `case 3`/`4`/`5` uses a
*different* initiation convention:

```js
ie = this.sym ? -1.4 : 0;
(he[0] a/t)          && (ie += 4.1,  z += 2300);
(he[last] a/t)       && (ie += 4.1,  z += 2300);
(he[0] g/c)          && (ie -= 2.8,  z += 100);
(he[last] g/c)       && (ie -= 2.8,  z += 100);
```

i.e. SantaLucia `TERMINAL` corrections, not NEBaseChanger's `-5.7/200` +
`+6.9/+2200`.

Reachability: `calcTm_nomm` occurs **exactly twice** in each bundle
(`[LIVE]` @ 45211 definition, @ 46887 use). The use is inside
`createTmFunction`:

```js
e.createTmFunction = function(n, a, i) {
  var u = i || 250, f = a || 50, h = n || 4;
  u = u/1e3;
  (h === 1 || h === 5) && (u = u/4);
  var d = new s.TmCalc({ct: u, monosalt: f, method: h});
  var p = function(v, _) { return d.calcTm_nomm(v).tm };
  return p
};
```

`createTmFunction` is called at three places (`[LIVE]` @ 47161 in the
`Primer` constructor, @ 50922 in `Primer.prototype.calculatePrimers`,
@ 54743 in `BuilderPrimer`), never from `Ne.tm`.

**Conclusion (Q1).** Two engines, two routes, both live:

| Route | Reached from | Engine | Initiation convention |
|---|---|---|---|
| SDM / inline mutation | `Fragment.autogenSDMPrimers`? no — see §1.5 | — | — |
| Inline mutation (`replace`/`insert` op) | dispatcher @ 320934 → `Ne.calcSDMPrimers` @ 322714 → `Ne.tm` | `createTmCalc` closure `Tm` | `−5.7/200`, `+6.9/+2200` per terminal A/T |
| NEBuilder / autogen | `Fragment.autogenSDMPrimers` @ 72991 → `s.Primer.calculatePrimers` → `createTmFunction(4,150,500)` | `TmCalc.calcTm_nomm` | `+4.1/+2300`, `−2.8/+100` per terminal G/C |

`[LIVE]` `Fragment.prototype.autogenSDMPrimers` @ 72991 calls
`s.Primer.calculatePrimers(this.template, n)` — i.e. the **NEBuilder** engine,
not `calcSDMPrimers`. `[LIVE]` The dispatcher's `replace` branch calls
`Ne.calcSDMPrimers(u.template, oe, ge, St, X, Ye, gt, hn)` @ 320934.

So the name "autogenSDMPrimers" does **not** mean "uses the SDM Tm model".
`[INFERRED]` the two routes were intended to agree; they do not, because the
initiation constants differ. Whether the difference is observable in the UI
depends on which route a given request takes; not established here.

### 1.5 Corrections to the earlier live-build report

The earlier [docs/research/nebasechanger-live-build-4330b0c7-2026-10-10.md](/research/nebasechanger-live-build-4330b0c7-2026-10-10.md)
§5 stated that the SDM path uses `+4.1/+2300` and `−2.8/+100`, and that the
GC-clamp loop is bounded by `primerMaxTm` (default 70). Both are wrong for the
SDM path:

* The SDM path (`createTmCalc` closure) uses `−5.7/200` and `+6.9/+2200`.
  `[LIVE]` verbatim above.
* `calcSDMPrimers` contains **zero** occurrences of `primerMaxTm` or `maxTm`
  (`[LIVE]` verified by counting inside the function's brace-balanced body,
  @ 322714..325893). The GC-clamp target is a **hardcoded literal 63 °C** in
  all four builds: `[v2.5.3]` `ze=63`, `[v2.7.2]` `Ht=63`, `[v2.8.4]` `el=63`,
  `[LIVE]` `Tt=63`. `primerMaxTm` bounds the GC clamp only in
  `Primer.prototype.calculatePrimers` (`[LIVE]` @ 51201 `p = a.primerMaxTm||70`).

---

## 2. `ct` concentration units and semantics (Q2)

Three different unit conventions appear. All are read directly from source;
none is inferred from plausibility.

| Site | Parameter | Unit | Conversion to mol/L |
|---|---|---|---|
| `createTmCalc` @ 76574 | `l.ct` | **µM** | `p = t*1e-6` → `this.primerc` |
| `TmCalc.init` @ 32042 | `t.ct` | **µM** | `calcTm_nomm`: `h = this.ct*1e-6` |
| `createTmFunction` @ 46722 | 3rd arg `i` | **nM** | `u = i/1e3` (→ µM), then `TmCalc` multiplies by `1e-6` |

`[LIVE]` `Ne.tm` passes `ct: 0.5` to `createTmCalc` → `primerc = 5e-7 mol/L`
= **500 nM**. `[NEB-DOC]` the in-app manual states "the preferred primer
concentration (500 nM)". These agree, so `ct` in the SDM path is µM and the
value is 500 nM. `[DERIVED]`

`[LIVE]` `Fragment.autogenSDMPrimers` @ 72991 defaults `n.primerCt = 500`, and
`Primer.calculatePrimers` @ 50922 passes `primerCt` as the **third** argument
of `createTmFunction`, i.e. nM. So `primerCt` is nM and `createTmFunction`'s
input is nM. `[LIVE]` `[DERIVED]`

The `/4` for methods 1 and 5:

```js
u = u/1e3;
(h === 1 || h === 5) && (u = u/4);
```

`[LIVE]` With the default `primerCt = 500` nM and `primerMethod = 4`, the
`/4` does **not** fire; `primerc = 5e-7 mol/L`. With `method = 1` it would be
`1.25e-7 mol/L`. `[UNKNOWN]` why methods 1 and 5 divide by 4: nothing in the
bundle states it. `[INFERRED]` Breslauer's original model was calibrated at a
different primer concentration; not evidenced here. This is the one place
where a documented experiment would be needed (see §7).

The `0.5` argument: `[LIVE]` `Ne.tm`'s `var re = .5` is the **first** value
passed as `ct`, in µM. It is not a separate "0.5 argument"; it is the literal
`ct`. `[LIVE]`

`[v2.8.4]` `Primer` constructor default options are
`{primerMethod:4, primerMonosalt:50, primerCt:200, …}` — **200 nM** — while
`this._modelPrimer.primerCt = i.primerCt || 500` is **500 nM**, and
`Fragment` defaults `primerCt = 500`. `[LIVE]` identical. So the constructor's
own default object disagrees with the model default (200 vs 500 nM) whenever
the caller supplies no options at all. Reachability of that case is not
established. `[UNKNOWN]`

---

## 3. Primer maximum-Tm field (Q3)

`[LIVE]` The bare identifier `maxTm` occurs **exactly once** in the bundle,
@ 47329, inside `Primer = function(n, a, i)` (brace range 46918..49755):

```js
i && i.primerMaxTm && (this.primerMaxTm = i.maxTm),
```

`[v2.5.3]` `[v2.7.2]` `[v2.8.4]` `[LIVE]`: the same single occurrence exists
in all four builds (`maxTm` count 1, `primerMaxTm` count 25/25/25/24).

Analysis:

1. The guard `i.primerMaxTm` is truthy in every default options object
   (`primerMaxTm: 70`), so the assignment **does execute**.
2. `i.maxTm` is `undefined` — no options object in any build contains a bare
   `maxTm` key (`[LIVE]` `[v2.8.4]` `[v2.7.2]` `[v2.5.3]`: `maxTm` occurs once,
   at this read).
3. Later in the **same** constructor, `[LIVE]` @ 49637:
   `Object.defineProperty(this,"primerMaxTm",{get:…,set:…})`.
   Brace range confirms 46918 ≤ 49637 ≤ 49755.
4. `Object.defineProperty` replaces the own data property created in (2) with
   an accessor. The accessor's getter returns `this._modelPrimer.primerMaxTm`,
   which is set to `i.primerMaxTm || 70` @ 47729.

**Verdict: not a reachable bug. It is an inert vestigial write.** `[NODE]`
locally executed minimal reproduction:

```
$ node -e '…P(i){ if(i&&i.primerMaxTm){this.primerMaxTm=i.maxTm}
   Object.defineProperty(this,"primerMaxTm",{get:()=>this._m.primerMaxTm,…});
   this._m={primerMaxTm:i.primerMaxTm||70} }'
typeof descriptor: "Object"
o.primerMaxTm = 70
after set 99 -> 99  _m = 99
```

`o.primerMaxTm` reads 70, not `undefined`. The write is overwritten.

Residual risk, stated honestly: `[UNKNOWN]` whether any code path reads the
raw own property before `defineProperty` runs. `[LIVE]` counting `this.primerMaxTm`
occurrences shows only the one write @ 47310 and `_modelPrimer.primerMaxTm`
accesses; no read of the bare own property exists. So even the transient
`undefined` is unobservable. `[DERIVED]`

---

## 4. Thermodynamic table differences (Q4)

Reproduced by `tests/fixtures/nebasechanger-research/tm_tables_diff.py`
(`[RUN]` `python3 tm_tables_diff.py`, rc=0) and independently by
`tests/fixtures/nebasechanger-research/q4b_sdm_tables.py` (`[RUN]` rc=0).

### 4.1 Two copies of every table

`tm_tables_diff.py` compares the **first** occurrence of each table name, which
is the `TmCalc.prototype.*` table (the NEBuilder engine). `q4b_sdm_tables.py`
instead maps the `createTmCalc` returned-object keys (`nndhds:q, loop:T,
bulge:N, dSSa:w, dHSa:A, dSBr:b, dHBr:y` in live) to their local literals and
compares those. Both engines keep separate copies.

`[RUN]` `q4b_sdm_tables.py` results:

| table | v2.5.3 | v2.7.2 | v2.8.4 | live | changed v2.8.4 → live |
|---|---|---|---|---|---|
| `dSSa` | 141 B | 141 | 141 | 141 | no |
| `dHSa` | 126 | 126 | 126 | 126 | no |
| `dSBr` | 141 | 141 | 141 | 141 | no |
| `dHBr` | 127 | 127 | 127 | 127 | no |
| `nndhds` | 4418 B / 176 keys | 4418 | 4418 | **4416 B / 176 keys** | **3 keys** |
| `loop` | 277 | 277 | 277 | 277 | no |
| `bulge` | 310 | 310 | 310 | 310 | **1 key** |

The three archived builds are byte-identical for every table; only live differs.

`[RUN]` differing keys, SDM engine (`createTmCalc` locals), v2.8.4 → live:

| key | v2.8.4 | live |
|---|---|---|
| `nndhds["aa/tc"]` | `{dh:-9.1, ds:-24}` | `{dh:2.3, ds:4.6}` |
| `nndhds["ct/aa"]` | `{dh:-9.1, ds:-24}` | `{dh:2.3, ds:4.6}` |
| `nndhds["ta/at"]` | `{dh:-7.2, ds:-20.4}` | `{dh:-7.2, ds:-21.3}` |
| `bulge[25]` | `{dh:0, ds:-28.1}` | `{dh:0, ds:-18.1}` |

`[RUN]` The `TmCalc.prototype.nndhds` copy has **148** keys while the
`createTmCalc` copy has **176** — the two engines do not share the same key
set. `[UNKNOWN]` why; the 28 extra keys in the SDM copy were not enumerated
this cycle.

### 4.2 What is *not* yet established

`[RUN]` `q4_tm_numeric.py` evaluated method-4 Tm in the live and v2.8.4
engines for four primer/compseq pairs and every result was **identical**:

```
D404N fwd primer  GGGCTGATGAAACTGACAGCGAA  live 69.8481  v2.8.4 69.8481  delta 0
D404N rev primer  AATCGCTTCAAGC            live 50.1754  v2.8.4 50.1754  delta 0
plain 24-mer      AAGTCAAG…                live 65.6856  v2.8.4 65.6856  delta 0
```

Reason, `[DERIVED]` from the source: when `compseq` is omitted,
`createTmCalc` sets `compseq = revcomp(seq)`, so `de = rev(compseq) = seq`
and every aligned-bigram key has the form `XY/XY`. The changed keys have
`top != bottom`, so they cannot fire in that configuration.

`[RUN]` `q4c_changed_keys.py` tried to force them: realistic 2-nt
substitutions (`tc→aa`, `aa→tc`) on a 26-nt template gave realistic Tm
(59.28 / 57.54 °C) but **still zero delta** — `buildPairMap` classifies those
mismatches as loop/bulge features, not as aligned `|`/`x` bigrams. Earlier
synthetic constructions did change the result (e.g. `ct/aa`: −267.4590 vs
−93.5276) but produced non-physical Tm (−267 °C) because the pairmap fell into
loop/bulge branches whose tables lack small sizes; those numbers are **not**
evidence about real behaviour.

**Honest conclusion (Q4).** The table changes are real and byte-verified in
both engines. Their effect on any real NEBaseChanger output is **not
established**: no case tried so far exercises them, and the pairmap
construction (`buildPairMap`, local `K` in live) decides which alignments count
as `|`/`x`. Determining reachability requires reading `buildPairMap`, which is
the next cycle's task. Do not state a degree-of-impact figure.

### 4.3 Complement alphabets

`[RUN]` `tm_tables_diff.py`:

* `[v2.8.4]` library `COMPLEMENTS`: 4 occurrences map `S→S`, `W→W`.
* `[v2.8.4]` app layer: 3 occurrences map `S→W`, `W→S`.
* `[LIVE]`: 3 library occurrences `S→S`, `W→W` and 3 app-layer occurrences
  `S→S`, `W→W`.

`[UNKNOWN]` whether `S`/`W` ever reach a complement operation in the Q5 SDM
workflow. `[v2.8.4]` `[LIVE]` `createTmCalc` separately does
`.replace(/s/g,"")` on the *compseq*, an unrelated operation. Not
investigated further this cycle.

---

## 5. Reproduction

```
cd tests/fixtures/nebasechanger-research
python3 tm_tables_diff.py            # Q4 tables + complement alphabets
python3 q1_tm_callpath.py            # Ne.tm / createTmCalc header / calcTm_nomm / call sites
python3 q1b_createTmCalc.py live     # full createTmCalc tail: returned object + closure Tm
python3 q1q3_reach.py live           # SDM dispatch reachability + maxTm occurrences
python3 q4b_sdm_tables.py            # createTmCalc's own table copies, 4 builds
python3 q4_tm_numeric.py             # engine-vs-engine Tm on fixed primer pairs (Node)
python3 q4c_changed_keys.py          # try to force the changed keys to fire (Node)
python3 q5_extract.py live           # buildPrimers (Tt) + Ge.calcSDMPrimers (At), 4 builds
python3 q5_verify.py                 # run the extracted functions on real inputs (Node)
python3 q1q3_four_builds.py          # Q1 selection + Q3 maxTm reachability, 4 builds
python3 q7_nndhds_keys.py            # Q7: createTmCalc option literal + nndhds keys, 4 builds
node -e '…'                          # §3 defineProperty semantics (re-run from this file)
```

All scripts read fixtures only; they write nothing except `/tmp/neb_*.js`
generated Node modules. No network is used.

All four scripts read only; they write nothing.

## 6. Four-build comparison (Q5-Q7), added in the second pass

Reproduced by `q5_extract.py`, `q5_verify.py`, `q1q3_four_builds.py`,
`q7_nndhds_keys.py` (all `[RUN]` rc=0).  Builds: v2.5.0 `17c7600d`, v2.5.3
`a8869455`, v2.8.4 `a5da1636`, live `4330b0c7`.

### 6.1 Q5 - substitution routing and alignment pad

| quantity | v2.5.0 | v2.5.3 | v2.8.4 | live |
|---|---|---|---|---|
| fragment-path routing (`Tt`/`buildPrimers`) | absent | absent | present | present |
| routing threshold (`At.length >= 6` or `At.length >= 6`) | n/a | n/a | 6, 6 | 6, 6 |
| `confine5prime` branch | n/a | n/a | present | present |
| split point `At = ceil(target/2)` | n/a | n/a | yes | yes |
| SDM-path alignment pad (`Ge.calcSDMPrimers`) | 10 | 10 | 10 | 10 |
| SDM-caller `alignDNAglobal` parameters | gap -2.5, term -2.5, extend -1, nucleotide matrix | same | same | same |
| `calcSDMPrimers` length floor / Tm floor | 15 / clamp 55..72 | same | same | same |
| `calcSDMPrimers` tail rule | **row withdrawn 2026-10-14**: the 10 → 11 change was refuted (2026-10-12 review, C19) | | | |

`[RUN]` `q5_verify.py` runs the extracted functions on real inputs. **Note
2026-10-14:** the `tail=` values in the output below came from an older version
of the script and are refuted (2026-10-12 review, C19); the current script
does not print them. The alignment row above was also corrected: the old
`-12/-1/+2/-4` values are `alignDNAlocal`/`alignDNAhalfFit` defaults, not the
SDM caller's parameters (live-build note §16.1, §12.3).

```
v2.5.0  pad=10  minlen=15  minTm=55,72  tail=10
v2.5.3  pad=10  minlen=15  minTm=55,72  tail=10
v2.8.4  pad=10  minlen=15  minTm=55,72  tail=11
live    pad=10  minlen=15  minTm=55,72  tail=11

v2.8.4  buildPrimers  repl='ACGT'  target='GATTACAGATTACAG'  confine=false
        -> route=fragment  fwdTail='ACGT'  revTail=''  splitAt=7
v2.8.4  buildPrimers  repl='ACGTACGT'  target='GATTACAGATTACAG'  confine=false
        -> route=fragment  fwdTail='ACGTACGT'  revTail=''  splitAt=7
live    buildPrimers  repl='ACGTACGT'  target='GATTACAGATTACAG'  confine=false
        -> route=fragment  fwdTail='ACGTACGT'  revTail=''  splitAt=7
live    buildPrimers  repl='ACGTACGT'  target='GATTACAGATTACAG'  confine=true
        -> route=fragment  fwdTail='ACGTACGT'  revTail=''  splitAt=7
```

Note the observed behaviour: the reverse tail is empty in these cases because
the target is shorter than `2*At`; the split only produces a reverse tail when
`target.length > At`.  That is a property of the extracted code, not of the
fixture.

### 6.2 Q1 selection and Q3 maxTm across four builds

`[RUN]` `q1q3_four_builds.py`:

| build | `Ne.tm` | `Ne.tm` body | `createTmCalc` default `method` | `case 4` salt block | `calcSDMPrimers` reachable | `primerMaxTm` reads | writes |
|---|---|---|---|---|---|---|---|
| v2.5.0 `17c7600d` | `Ne.tm` | `createTmCalc` | 4 | yes | no | 0 | 0 |
| v2.5.3 `a8869455` | `Ne.tm` | `createTmCalc` | 4 | yes | yes | 0 | 0 |
| v2.8.4 `a5da1636` | `Ne.tm` | `createTmCalc` | 4 | yes | yes | 1 | 1 |
| live `4330b0c7` | `Ne.tm` | `createTmCalc` | 4 | yes | yes | 1 | 1 |

`calcSDMPrimers` is reachable only through `Ge.calcSDMPrimers`, which is called
from exactly one site, the `replace` branch of the SDM dispatcher, guarded by
`!confine5prime && repl.length < 6 && target.length < 6`.  In v2.5.0 that
dispatcher does not exist, so the function is dead code there.

`primerMaxTm` is read once, in the GC-clamp loop of the fragment-path
`calcPrimerTm`/`growPrimer` code, and written once, in the `Primer` constructor.
Both sites are in the fragment path, which v2.5.0 and v2.5.3 do not have.

### 6.3 Q7 - the `nndhds` keys are not reachable in the live build

`[RUN]` `q7_nndhds_keys.py`:

| build | `createTmCalc` option literal | `ct` | `salt` | `method` | `nndhds` bytes | keys |
|---|---|---|---|---|---|---|
| v2.5.3 | `{ct:0.5, salt:150, method:4}` | 0.5 | 150 | 4 | 4418 | 176 |
| v2.8.4 | `{ct:0.5, salt:150, method:4}` | 0.5 | 150 | 4 | 4418 | 176 |
| live | `{ct:0.5, salt:150, method:4}` | 0.5 | 150 | 4 | 4416 | 176 |

The three changed keys are `x|`, `x-`, `|x`.  `[RUN]` `q4c_changed_keys.py`
confirms they are the only differences, that the key *sets* are otherwise
identical, and that the changed values are `0` in v2.8.4 and `-1` in live.

`[RUN]` `q4b_sdm_tables.py` shows the pair map is built as
`compseq = revcomp(seq)`, so a pair is `seq[i] + '|' + compseq[j]`.  A pair
containing `x` or `|` at either position requires the sequence to contain a
non-ACGT character.  `[RUN]` `q1q3_reach.py` shows the dispatcher's `replace`
branch is reached only with a concrete replacement string, and the primer
candidates are built from `expandDNASeq` output over `DEGENERATES`, which
resolves degenerate symbols before any Tm call.

Conclusion: `[DERIVED]` the three changed keys are unreachable in the live
build's default configuration.  `[UNKNOWN]` whether any user path can place a
literal `x` or `|` into a Tm-scored sequence.

## 7. Open questions, and the smallest experiment that would settle each

1. `[UNKNOWN]` **Reachability of the changed `nndhds`/`bulge` keys in real
   primer pairs.** Established this cycle: they are *not* hit when
   `compseq = revcomp(seq)`, and not hit by 2-nt substitutions (which become
   loop/bulge features). Next step: read `buildPairMap` (local `K` in live,
   inside `createTmCalc`) and enumerate which mutation shapes produce `|` or
   `x` at a mismatched position. Local only, no network.
2. `[UNKNOWN]` Which route a given UI request takes (SDM closure vs
   `calcTm_nomm`). Settle locally: instrument `Fragment.autogenSDMPrimers` and
   the dispatcher's `replace` branch, run the local harness over SID-1 D404N.
3. `[UNKNOWN]` Why methods 1 and 5 divide `ct` by 4. Not answerable from the
   bundle; would need NEB documentation or an author statement. Flag as
   unresolvable statically.
4. `[UNKNOWN]` Whether the 200 vs 500 nM default mismatch in `Primer` is ever
   exercised. Settle locally: enumerate call sites of `new Primer(...)` with
   omitted options.
5. `[UNKNOWN]` Why the `createTmCalc` copy of `nndhds` has 176 keys while the
   `TmCalc.prototype` copy has 148. Enumerate the 28 extra keys.
