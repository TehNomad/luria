# Q5 primer policy review — the M3b review-and-decision pass

**What this is.** A review of the policy written in [docs/specs/q5-primer-design.md](/specs/q5-primer-design.md)
§§16–20: the `luriaNative` candidate policy and the `NebaseChangerV2_11_0`
compatibility target. It is a review, an evidence pass and a decision pass. It
implements nothing: `PrimerDesign` is still not implemented, no Rust changed in
this pass, and no runtime behaviour was added.

**What it changed.** §§17.3, 18.1, 18.3, 18.4, 18.5 and 19 of the design
contract were amended in place, and §21 was added as the index to this report.
§20 was not changed. The decisions this pass settled are recorded as D028 and
D029 in [docs/decisions/](/decisions/). Everything else is recorded here as unresolved,
with the smallest experiment that would settle it.

**Date.** 2026-10-12.

**Superseded in part (2026-10-13).** This report is kept as the record of that
pass. One rule it settled was changed the next day: the **availability-aware
split** it records at A8, at D-5's "the split of `a_f` among ties" row, at
Part D item 4, at Part E's D029 bullet, and at the handoff bullet "No
biological assumption was changed" is **no longer the native rule**. [docs/specs/q5-primer-design.md](/specs/q5-primer-design.md)
§18.1 now states a **mutation-centered** split (`u = ceil(a_f/2)`,
`v = floor(a_f/2)`, extra base 5′), availability is a feasibility test on the
centered candidate rather than a rule that moves it, and the split is no longer
"arbitrary among ties". Read those five passages as the reasoning of the
2026-10-12 pass, not as the current contract. The follow-up and its
consequences are §22 of the design contract and
[docs/research/q5-primer-implementation-readiness-2026-10-13.md](/research/q5-primer-implementation-readiness-2026-10-13.md). Nothing else
in this report was changed, and no compatibility claim, Tm rule or maximum
primer length statement was touched.

---

## 0. What was read, and what is new since the policy pass

The research reports and the fixtures are present on this branch:

```
docs/research/nebasechanger-cycles-1-4-2026-10-10.md
docs/research/nebasechanger-live-build-4330b0c7-2026-10-10.md
docs/research/nebasechanger-har-and-builds-2026-10-10.md
docs/research/nebasechanger-client-investigation.md
docs/research/nbc_help_text_v284.txt
tests/fixtures/nebasechanger-research/
tests/fixtures/q5-primer/
```

Two things about the fixture tree were found while working, and both matter for
anyone re-running the reproduction commands in §17.4 and in the cycles report:

* The tree was **reorganised during this session**. `archived-js/v2.8.4/` no
  longer exists; the v2.8.4 bundle is now at
  `tests/fixtures/nebasechanger-research/app-js/index.a5da1636.v2.8.4.js`
  (with `beautified/` beside it) and
  `raw/index.a5da1636.v2.8.4.min.js`. `example-construct/` moved out to
  `tests/fixtures/example-construct/`. Paths quoted in the earlier reports that
  predate the move no longer resolve.
* `q5_verify.py` no longer prints the `pad / minlen / minTm / tail` line that
  cycles report §6.1 quotes; it now prints V1–V8 diagnostics. That is why the
  `tail = 10 / 11` row of §6.1 cannot be re-run (see A9 below).

Worktree state at the start of this pass, unchanged by it:

```
$ git status --short --branch
## feat/milestone-3...origin/feat/milestone-3
 M crates/core/src/error.rs
 M crates/core/src/lib.rs
 M crates/core/src/q5.rs
 M crates/core/tests/core_foundation.rs
 M crates/core/tests/mutation_fixtures.rs
 M crates/core/tests/mutation_invariants.rs
 M tests/fixtures/README.md
?? crates/core/src/primer.rs
?? docs/specs/q5-primer-design.md
?? docs/specs/q5-sdm.md
?? docs/research/
?? docs/tasks/contracts/milestone-3-q5-workflow.md
?? docs/tasks/contracts/milestone-3b-q5-primer-contract.md
?? package-lock.json
?? package.json
?? tests/fixtures/nebasechanger-research/
?? tests/fixtures/q5-primer/

$ git diff --stat
 crates/core/src/error.rs              |  119 +-
 crates/core/src/lib.rs                |    1 +
 crates/core/src/q5.rs                 | 3318 +++++++++++++++++++++++++++---
 crates/core/tests/core_foundation.rs  |    9 +
 crates/core/tests/mutation_fixtures.rs|    9 +
 crates/core/tests/mutation_invariants.rs |  9 +
 tests/fixtures/README.md              |    1 +
 7 files changed, 3205 insertions(+), 261 deletions(-)
```

Nothing was staged, committed, reset or discarded. The only files this pass
wrote are documentation: [docs/decisions/](/decisions/) (modified: D028 and D029 appended),
[docs/specs/q5-primer-design.md](/specs/q5-primer-design.md) (untracked; §§17.3, 18.1, 18.3, 18.4, 18.5, 19 amended
and §21 added) and this report under [docs/research/](/research/).

---

## 1. Part A — geometry and biological semantics

### A1. The declared geometry is representable, and satisfies H1–H8

Checked against `crates/core/src/primer.rs`, not against prose. For an edit at
template span `[s, e)` with `L = e − s`, a candidate `(a_f, a_r)` with split
`(u, v)` and `w = a_r` produces:

| Object | Span (template, half-open) | Text |
| --- | --- | --- |
| forward | `[s − u, s + L + v)` | `edited[s − u, s + L + v)` |
| reverse | `[s − u − w, s − u)` | `revcomp(template[s − u − w, s − u))` |

`construct_primer` takes the text from `result.edited()` sliced at the span
(`slice(after, span.start(), span.end())`, reverse-complemented for a reverse
primer). For a substitution the edited artifact and the template have equal
length — `construct_primer` rejects otherwise with
`MutationVerificationFailure` — so the table above is exactly what the function
accepts. The forward primer decomposes into three parts
(context `u`, mutation `L`, context `v`); the reverse primer into one
(context `w`). `annealing_overlap` is therefore `u + v = a_f` forward and `w = a_r`
reverse.

Consequence: H1–H8 hold for every candidate that satisfies the availability and
alphabet conditions. H2 holds because the forward span contains `s`; H4 because
`oriented_of(role)` gives `Same`/`Opposite`; H5 because `PrimerPair::separation`
is derived from the two spans and is `0` for an abutting pair; H7 because
`built_against` is the template reference; H8 because `runs()` partitions the
primer and `annealing_of` counts only context parts. **Verified against the
code, not asserted.**

### A2. The coordinate convention is precise, and it is not the reference's

`[s, e)` is zero-based half-open (D004); `one_based_span()` renders `[s+1, e]`
inclusive (D015). The reference's batch grammar names **outer anchors**
(§13.2): `"ID_1 234 239 ACGT"` substitutes the bases strictly between 234 and
239. The SID-1 fixture's own description string — `Replace gat between bases
1209-1213` — is that anchor form, and the edit is 1-based 1210–1212. The
convention is stated, the conversion is explicit, and nothing guesses. **Adequate.**

### A3. The forward primer's replacement-containing text

The forward primer's text is a substring of the **edited** artifact, not of the
template, and its span is an interval of the **pristine template** (§4, H3,
I-2). `Primer.edited_span` is `Some(span)` for the forward primer and `None`
for the reverse one, which is exactly where H3 states the clause. For the
native scope `L = e − s = 1`, so the two intervals coincide; §18.1 writes `L`
generically, which invites reading it as an insertion. **Amended in §18.1 to
state `L = e − s` and to mark `L ≠ e − s` as out of scope.**

### A4. The reverse primer maps to the template only where unaffected

`reverse.annealing == revcomp(template[reverse.annealingSpan])` (§5) holds
because the reverse span lies entirely 5′ of the abut boundary, which is at or
left of `s`, so it contains no changed position. `construct_primer` enforces it
twice: once as `PrimerTextAtSpanMismatch` (text against the edited artifact)
and once as `PrimerAnnealingNotTemplate` (annealing text against the template).
**Coherent.**

### A5. Full length vs annealing span vs overlap — distinguished

`Primer.length` (whole primer), `annealing_length` (count of non-changed
positions), `annealing_overlap` (annealing positions whose base equals the
template under them) and `whole_primer_match_count` are four separate stored
fields. §18.1's "primer lengths are `a_f + L` and `a_r`" is the relation between
length and the annealing totals, and it is right. **No confusion found.**

### A6. `minAnnealingOverlap` is applied to every emitted primer's annealing region

`require_design_constraints` compares `annealing_overlap()` of **both** primers
against the bound, and never consults `whole_primer_match_count()` or `length`.
It also rejects a bound below 1. **Correct as the task requires.**

### A7. Boundary substitutions, short templates, insufficient flanking sequence

The native geometry admits a design **if and only if**

```
s ≥ minAnnealingOverlap          and          n ≥ 2·minAnnealingOverlap + L
```

(for a linear template, no `maxAnnealingOverlap`, and the split rule of §18.1).
Derivation: `w ≥ bound` and `u ≤ s − w` force `s ≥ bound`; maximising `u + v`
at `w = bound` gives `u ≤ s − bound` and `v ≤ n − s − L`, so `a_f ≥ bound` is
achievable iff `(s − bound) + (n − s − L) ≥ bound`, i.e. `n ≥ 2·bound + L`. The
condition is independent of `s` except for the 5′ side, which is asymmetric
because the reverse primer must fit whole 5′ of the boundary while the forward
primer may be all-5′-context.

Concretely, on a 30-base template with `bound = 15` and `L = 1`: no design
exists at all (`2·15 + 1 = 31 > 30`), which is what `q5p-09` records. On a
100-base template with `bound = 15`, an edit at `s = 0` has no design, and an
edit at `s = 99` does (u = 15, v = 0, w = 15). **The asymmetry is a real
property of the geometry and must be stated, not discovered at implementation
time.** Amended into §18.1.

### A8. The split rule as written is a defect

§18.1 fixes `u = ceil(a_f/2)`, `v = floor(a_f/2)`. For an edit near the end of
a template this rejects a feasible `a_f`: with `n − s − L = 0` and `a_f = bound`,
the fixed split gives `v = floor(bound/2) > 0`, which does not fit, while
`u = bound, v = 0` does. The search is complete for the *declared* geometry, but
the declared geometry is not complete for the pair. **Decision taken:
availability-aware split** (see the amended §18.1).

### A9. The reference's per-side wording vs the contract's total

§13.1 item 4 says the replacement is placed "with 10 or more bases of annealing
sequence flanking the inserted bases"; item 6 says "at least 10 nucleotides
that are 100% complementary on both sides of the mutation region". Both are
per-side statements. Every reference forward primer in `designs.json` has
annealing runs of exactly `[10, 10]` — totals 20 — for all six designs, at
bounds of 15, 24 and 25 alike:

```
q5p-01          fwd span [1199, 1222]  anneal 20  runs [[1199,1209],[1212,1222]]
q5p-02          fwd span [1199, 1222]  anneal 20  runs [[1199,1209],[1212,1222]]
q5p-03          fwd span [1199, 1222]  anneal 20  runs [[1199,1209],[1212,1222]]
q5p-04/design   fwd span [1199, 1222]  anneal 20  runs [[1199,1209],[1212,1222]]
q5p-07/variant-a fwd span [1199, 1222] anneal 20  runs [[1199,1209],[1212,1222]]
q5p-07/variant-b fwd span [1493, 1516] anneal 20  runs [[1493,1503],[1506,1516]]
```

That is the observation that explains why the forward primer's annealing never
moves when the bound rises from 15 to 25: the reference's forward 5′ context is
the pad (10) and its 3′ extension is driven by Tm, not by the length parameter.

**Decision taken: do not add a per-run constraint.** H1 stays a bound on the
total. The divergence is recorded as an open question, not absorbed: a native
design at `a_f = 15` has runs 8/7, which the reference would never emit. Stated
in §18.2 and in §20.

---

## 2. Part B — enumeration and optimality

### B1. The objective does mean what the documentation claims

Primer lengths are `a_f + L` and `a_r`, so total primer length is
`a_f + a_r + L`. `L` is fixed by the edit, so minimising `a_f + a_r` **is**
minimising total primer length. The claim in §18.3 is justified — but §18.3 did
not say *why*, and the task asked whether it is justified at all. It is; the
justification is now stated in §18.3. What the objective is **not** is a bound
on the longer primer, or on Tm balance: a candidate `(20, 10)` and a candidate
`(15, 15)` tie at 30 and the tie-break picks `(15, 15)`, which is the more even
pair by accident, not by rule. Stated.

### B2. Completeness

Complete for the declared geometry once A8's split rule replaces the fixed one.
The domain stays two-dimensional `(a_f, a_r)`; the split is a function of
`(a_f, availability)`, so no third dimension is introduced.

### B3. Determinism and the tie-break

§18.3's third key, "`u ≥ v` — already fixed by §18.1, so no further tie is
possible", is wrong once the split is availability-aware: `u ≥ v` is a property
only when availability permits it, and it is not a tie-break key at all. With
`a_f` fixed, `(u, v)` is a function of `a_f`, so the two keys `a_f + a_r` then
`a_f` already determine a unique candidate. **Amended: the third key is removed
and the uniqueness argument restated.**

The order is input-independent: it is a function of `(s, L, n, minAnnealingOverlap,
maxAnnealingOverlap)` only. Sequence *content* never enters the order, so I-11
holds by construction. That is stronger than §18.3 claimed, and is now stated as
such.

### B4. Alphabet and replacement validation

For the native scope the replacement is one `char` from a `NucleotideMutationIntent`
and the template is canonical DNA, so H6 is satisfied before the search runs and
can never reject a candidate. It is still checked, on the output, per §7's rule
that H1–H8 are checked rather than assumed. **No change; recorded so that a
later reader does not build an alphabet filter that never fires.**

### B5. Failure semantics distinguish three outcomes, not two

§18.5 describes one failure. There are three distinguishable ones, and collapsing
them loses information the caller needs:

1. **Proven absence** — the domain was capped by availability alone
   (`maxAnnealingOverlap` absent, or above the availability cap): no design
   exists in the declared geometry for this edit on this template.
2. **Bounded absence** — the domain was capped by a supplied
   `maxAnnealingOverlap` strictly below the availability cap: no design exists
   *within the supplied bounds*; a design may exist above them.
3. **Not searched** — the request was rejected before enumeration
   (`minAnnealingOverlap < 1`, `L ≠ e − s`, a non-linear template, a composed
   variant request): the failure is in the request, not in the geometry.

Amended into §18.5. `q5p-09` is outcome 1 and its note already says the failure
"is not a property of one search" — the fixture and the semantics now agree.

### B6. Consistency of the bounds

H1 (`a_f ≥ bound`, `a_r ≥ bound`), availability (`u ≤ s − a_r`, `v ≤ n − s − L`),
the split interval being non-empty, and `maxAnnealingOverlap` are applied to the
same candidate in one place in the amended §18.1/§18.3. No candidate is
accepted that fails any of them, and no candidate is excluded that passes them.

---

## 3. Part C — tests required before implementation

Named in the conventions the repository already uses: fixture rows in
`tests/fixtures/q5-primer/cases.tsv` (14 columns, `reference_check` and
`invariants` columns already defined) and Rust tests in
`crates/core/tests/` alongside `mutation_fixtures.rs` / `mutation_invariants.rs`.
No existing test is rewritten.

**Coordinate and interval invariants** (fixture rows plus unit tests; these
catch off-by-ones before any search exists)

1. An edit declared `[s, e)` yields a forward span whose start is `≤ s` and
   whose end is `≥ e`, and a reverse span whose end equals the forward span's
   start. `PrimerPair::boundary()` is `Some` of that value.
2. `one_based_span()` of the SID-1 forward primer is `(1200, 1222)` — the
   anchor-form description in `raw/sdmsummary.txt` is *not* a span, and a test
   that confuses them is the point of §13.2.
3. `reverse.annealing == revcomp(template[reverse.annealingSpan])` — already
   enforced by `PrimerAnnealingNotTemplate`; assert it as an invariant test, not
   only as a construction error.
4. `annealing_spans()` for a forward primer is two intervals; for a reverse
   primer, one. Assert the count, not just the total.

**Boundary substitutions** (new fixture rows; the geometry's asymmetry is the
point)

5. `q5p-11-boundary-start`: edit at `s = 0` on a linear template, bound 15 →
   no design; failure names the 5′ availability limit.
6. `q5p-12-boundary-end`: edit at the last position, bound 15, `n = 100` → a
   design exists with `v = 0`; assert that the split is availability-pulled and
   that `u ≥ v` is *not* assumed.
7. `q5p-13-exactly-feasible`: `n = 2·bound + L` → exactly one feasible split at
   `a_f = bound`; assert the design and that `a_f = bound + 1` is infeasible.
8. `q5p-14-one-short`: `n = 2·bound + L − 1` → proven absence, outcome 1 of §18.5.

**Minimum-overlap enforcement**

9. A candidate at `a_f = bound` is accepted (`≥`, not `>`); `a_f = bound − 1`
   rejected. `h1_accepts_a_primer_exactly_at_the_bound` already covers the pair
   level; add the per-primer and the off-by-one-below case.
10. A design whose *whole primer* meets the bound but whose annealing does not
    is rejected — the `whole_primer_match_count` / `annealing_overlap`
    distinction of §5 and H8, asserted as a rejection rather than only as two
    reported numbers.
11. The existing `q5p-02`/`q5p-03` reference designs stay recorded as H1
    VIOLATED; a test asserts that the reference records are *not* silently
    repaired.

**Symmetric cases and tie-breaking**

12. A template whose 5′ and 3′ flanks are equally long, with `a_f + a_r` tied
    across two candidates: assert the `a_f` ascending tie-break picks the
    smaller `a_f`, and that reversing the template's content does not change
    which candidate is chosen.
13. Assert the chosen candidate is invariant under permuting the order in which
    candidates are enumerated (I-11) — a property test, not one example.

**Multiple valid candidates and objective ordering**

14. A case where the shortest-total candidate is not the shortest *maximum*
    primer: `(20, 10)` vs `(15, 15)`. Assert the chosen one, and assert in the
    row's `note` column that the objective is total length, not balance.

**No-valid-design cases**

15. Outcome 1 (proven absence) and outcome 2 (bounded absence) as two rows with
    the same edit and different `maxAnnealingOverlap`, so the two reports cannot
    be confused.
16. Outcome 3 (not searched): a request with `minAnnealingOverlap = 0`, and a
    request whose edit has `L ≠ e − s`.

**Alphabet and replacement validation**

17. A template containing a non-DNA symbol, and a replacement outside ACGT —
    assert the failure names the position and the character, matching
    `InvalidSequenceCharacters`' existing shape. Note that for the native scope
    these cannot fire (B4); the test is of the *check*, driven directly at
    `construct_primer`, not of the search.

**Determinism independent of input ordering** — see 13.

**Compatibility conformance (separate suite, separate name)**

18. A conformance suite that compares primer **text** against the reference for
    the six `designs.json` cases. This is the only comparison the current
    evidence supports (see D3 and D8 below). It must not be described as a Tm
    comparison, and it must not be described as span equality.

---

## 4. Part D — the compatibility audit, claim by claim

Labels: **Verified** = read in the stored bundle and quoted here;
**Strongly supported** = clear source evidence, no end-to-end equivalence;
**Unresolved** = evidence incomplete or not reproduced;
**Out of scope** = excluded from the target.

| # | §19 claim | Label | Evidence in this pass |
| --- | --- | --- | --- |
| C1 | minimum length floor 15 | **Verified** | live `Ne.calcSDMPrimers=function(I,X,re,se,Se,Me,Le,je){X=X<15?15:X,…}`; v2.8.4 `…function(E,ee,ne,ae,ue,ie){…ee=ee<15?15:ee…}`. Same in both. |
| C2 | minimum Tm clamped to `[55, 72]` | **Verified** | live `re=re<55?55:re,re=re>72?72:re`; the dispatcher clamps again at live lines 16977–16979. |
| C3 | no maximum length | **Verified** | no length ceiling anywhere in the live function body (lines 17194–17291 read in full); growth is bounded only by `Le`/`je`. |
| C4 | terminal-GC target 63 | **Verified** | `Tt = 63` (live) / `el = 63` (v2.8.4); used as `while tmf < 63 and not G/C-terminated`. |
| C5 | pad of 10 | **Verified in both builds** | live `St = 10` at line 16969 of the beautified live bundle (byte offset 500481; a second `St = 10` at line 17219 is the alignment pad inside `calcSDMPrimers`); v2.8.4 `ot=10` at byte offset 313404 in `app-js/index.a5da1636.v2.8.4.js` and `raw/index.a5da1636.v2.8.4.min.js`, and at 500985 in the beautified v2.8.4. **This contradicts nothing and fixes the 10→11 claim** (below). |
| C6 | forward primer carries the replacement inline and grows 3′ | **Verified** | `oe.pfwd = I.slice(0,se) + Ye.alnSeq1 + I.slice(Pe, Pe+ge)` with `ge` starting at the pad `se` and `++ge` in the growth loops. |
| C7 | reverse primer is `revcomp` of 5′ context and grows from the minimum length | **Verified** | `wt = Ne.revcomp(I); oe.prev = wt.slice(0, xe)` with `xe = X = max(minLen,15)`, `++xe` in the growth loops. |
| C8 | **the minimum-length parameter applies only to the reverse primer's initial length; the forward primer's 3′ extension starts at the pad** | **Verified, and now stated explicitly** | `var oe={}, ge = se, xe = X` — `ge` (forward extension) is seeded from the pad, `xe` from the clamped minimum length. This is why every reference design has `a_f = 20` at bounds 15, 24 and 25 alike. §17.1 stated it in a row; §19 item 2 now states it as a rule. |
| C9 | growth order: reach min Tm → reach 63 → revert to last good → balance while Tm differs by more than 5 | **Verified** | read in full: `for(; $e < re;){…}`, `for(; $e < Tt && … "[GgCc]$"){…}`, the revert block, the mirror block for `oe.prev`, then `if($e > yt){ for(; $e - yt > 5 && xe < je;)… }` and `if(yt > $e){ for(; yt - $e > 5 && ge < Le;)… }`. |
| C10 | availability limits throw rather than returning a shorter design | **Verified** | `Le = Math.min(Le, I.length - Pe)`, `je = Math.min(je, I.length)`; `if(ge > Le) throw "cannot reach min primer length … only N bases available 3' of the mutation"`; `if(xe > je) throw …5'…`; `if(ge >= Le) throw "cannot reach min Tm…"`; `if(xe >= je) throw …`. Dispatcher converts a message containing `"cannot reach"` into a per-op warning (live lines 17048–17050 and the two `catch` blocks). |
| C11 | availability limits are finite only on the linear branch | **Verified** | `let gt = 1/0, hn = 1/0;` then `if(f.isCircular) u.subseq=[mt-St+1, mt-St]; else { … (gt = i.length - ft), (hn = mt - St + 1); }` at live lines 17103–17106. |
| C12 | alignment short-circuit: equal-length, shorter than the pad, both ACGT-only | **Verified, with a correction** | live `Me.length === Se.length && Me.length < St && Sl.test(Me) && Sl.test(Se)` — the bound is the **pad** (`St`), not a literal 10. It reads as 10 only because the pad is 10. Stated as "shorter than the pad". |
| C13 | otherwise `alignDNAglobal` with `mat` ±2 diagonal, `a/g = g/a = +1`, other off-diagonals −1, `gap = -2.5`, `term = -2.5`, `extend = -1` | **Verified in source** | the inline `ft` object read at live lines 17216–17224. |
| C14 | the short-circuit is a pure optimisation | **Unresolved** | equivalence with `alignDNAglobal`'s result on the guard's inputs is not proven. Note that on the *short path* the routing guard already forces `Ye.length < 6`, `X.length < 6` and ACGT-only, so the short-circuit fires whenever old and new have equal length; for the native scope (`L = 1`, old length 1) it **always** fires. The alignment machinery is therefore inert for the native/compatibility intersection — which is the reason this unresolved item does not block the native policy. |
| C15 | engine identity: `createTmCalc`, method 4, 500 nM, 150 mM | **Verified** | live `Ne.tm=function(I,X){var re=.5,se="Q5",Se=150,Me=4; … Ne.createTmCalc({seq:I,ct:re,salt:Se,compseq:X}); switch(se){case"Phusion":…default:Me=4} Le.setMethod(Me); …}`. `ct = 0.5` µM with `primerc = ct*1e-6` → 500 nM. |
| C16 | initiation `−5.7/200` plus `+6.9/+2200` per terminal A/T | **Verified** | live offset 84860, `case 3: case 4: … (te=this.sym?-1.4:0, ie=-5.7, z=200, (he[0]==="a"||he[0]==="t")&&(ie+=6.9,z+=2200), (he[u.length-1]==="a"||he[this.wseq.length-1]==="t")&&(ie+=6.9,z+=2200) …`. |
| C17 | **the Tm value is reproducible** | **Unresolved, but the blocker named in §17.3 was wrong and is now corrected** | `buildPairMap` **is** present in the stored live bundle: the closure's builder is the minified local `K = function () {…}` at byte offset 127266 (line 3865), called as `this.buildPairMap()` at 130164 (line 3940) inside the `pairmap === ""` branch, and exposed in the returned object literal as `buildPairMap: K` at 134894 (line 4063). It builds a mask over the primer against the reverse-complemented compseq — `\|` where the base is the complement, `^` where either side is `-`, `x` otherwise — then collapses runs: `x` next to `x`/`_` → `_`; `^+_+`/`_+\^+` → `_`; `^+x`/`x\^+` → `_`; `_+[x^]`/`[x^]_+` → `_`. So the mismatch handling is **readable from the stored artifact**, and the claim "not read, therefore not reproducible" is out of date. The value is still **Unresolved** for a different and honest reason: no `luria` Tm port exists, and no fixture compares one. Reproducing it is an implementation task, not an evidence gap. **Do not conflate** the class-level `TmCalc.prototype.buildPairMap`, which is present in v2.8.4 (byte offset 69194, called at 73484) and **absent** in live. |
| C18 | the rotation is a coordinate device, and the saved spans are template spans | **Strongly supported** (was **Unresolved**) | `tests/fixtures/nebasechanger-research/replay_dispatcher_geometry.py` re-run in this pass reproduces, for all six reference designs, the primer **text**, the primer **lengths**, the **spans**, the abutment and the availability limits, from the live bundle's own pad literal and rotation expression. See §6 for the run. What keeps it at *strongly supported* rather than *verified*: **the reference reports no spans at all.** `raw/sdmsummary.txt` carries `ID, Description, Fwd Primer Seq, Length, Tm Fwd, Rev Primer Seq, Length, Tm Rev, Ta` — no positions. There is nothing to compare a span against, so span equivalence with the reference tool is not a testable claim today. |
| C19 | the 10 → 11 minimum-annealing-tail change (cycles §6.1) | **Refuted as stated; the claim is dropped** | the pad literal is 10 in **both** builds (C5); neither `calcSDMPrimers` body contains the substring `11` (v2.8.4 body checked programmatically; live body read in full); and `q5_verify.py` no longer emits the `tail=` line that §6.1 quotes, so the row cannot be re-derived from the current script. Recorded as an artifact of an older script version, not as a build difference. |
| C20 | fragment-path compatibility | **Out of scope** | unchanged; §17.2's parameter set is evidence about the build, and the coordinate mapping and NEBuilder assembly layer are unverified and unexercised. Requesting fragment design under the target is an error. |
| C21 | insert and delete compatibility | **Out of scope** | routing at live line 17052 sends both to the fragment path; and the mutation layer cannot represent either. Both reasons are independent and both are stated. |
| C22 | degenerate replacements, merged codon ops, NEBuilder assembly, batch mode, legacy mode | **Out of scope** | unchanged. |
| C23 | `verifyPair` is part of the specification | **Strongly supported as a safety net, not a specification requirement** | live `vt.verifyPair` at byte offset 162031 (line 4807). Its return strings are `"missing primer pair"`, `"could not reconstruct the mutant sequence (…)"`, `"reverse primer is not a valid IUPAC sequence"`, `"empty primer pair"`, `"forward/reverse primers do not match the mutant sequence at the edit site"`, `"forward flap + annealing region does not reconstruct the full primer"`, `"reverse flap + annealing region does not reconstruct the full primer"`, plus `vt.verifyOp`. Its only consumer, live byte offset 484084 (line 16534), pushes `"internal check failed for " + re.idprefix + ": " + se + " - primers not shown. Please report this to NEB."` — the wording is a bug report, not a biological rule. **Also note §17.2's "its four checks" understates it: there are seven distinct outcomes.** |
| C24 | the pair-coherence invariant that `verifyPair` encodes | **Verified as an invariant the native geometry satisfies; not adopted as a new requirement** | `verifyPair`'s central test is that `revcomp(reverseSeq) + forwardSeq` occurs in the mutant sequence spanning the edit window. For the native geometry, `revcomp(rev.text) + fwd.text = edited[s−u−w, s+L+v)`, a contiguous substring of the edited artifact covering the changed position. It follows from H3 and the abutment, so it needs no new requirement — it is worth stating as a derived check (candidate I-14). |
| C25 | v2.8.4 differences that must not be transferred to the target | **Verified** | v2.8.4's routing is `E=="insert"||E=="replace"&&(ie||Pe.length>=6||ee.length>=6)` — **no** ACGT test, so the ACGT-only routing guard is new in live. v2.8.4's `calcSDMPrimers` takes **six** parameters, so it has no availability limits and cannot throw the "cannot reach … bases available" errors; it also reads the wild-type from a closure variable (`w`) rather than a parameter. Both are recorded so a v2.8.4 target is not assumed to behave like the live target. |

### D.1 The name should say it is a partial target

**Decision taken.** `NebaseChangerV2_11_0` names a *build*, not a
compatibility guarantee, and after this pass C17, C18 and C19 are still not
end-to-end equivalent. The name stays a version identifier, and the record that
accompanies it now carries an explicit `partial` marker with the list of what is
not substantiated. A mode that cannot reproduce a Tm value and cannot be
compared on spans is not "compatible" with anything; §16.1's rule that the word
*compatible* appears only qualified by the substantiation record is now enforced
by a field, not by prose alone.

### D.2 Conflicts between reference behaviour and Luria requirements

Recorded, not absorbed:

1. `q5p-02` and `q5p-03`: reference designs with forward annealing 20 against
   bounds 24 and 25 — H1 VIOLATED. Unchanged from §19.
2. The reference's per-side ≥10 forward grip (§13.1 items 4 and 6) is **stronger
   than H1** for the forward primer: it implies `a_f ≥ 20`. A native design at
   `a_f = 15` satisfies H1 and would never be emitted by the reference. Neither
   is adjusted to fit the other.
3. The reference **throws** when availability is exhausted (C10); the native
   policy reports a failure with a named outcome (§18.5). Same shape of
   behaviour, different reporting vocabulary — §18.5 already says the reference
   wording is not reused.
4. The reference's forward primer's 5′ context is the pad and never shrinks; the
   native policy's split is availability-aware and can be `u = bound, v = 0`.
   Different geometries, both valid under their own rules.

---

## 5. Part E — decisions

### Settled in this pass, and recorded as decision records

* **D028** — a design names a policy, and a compatibility policy names a build.
  Settled: the policy field is required, has no default, and a
  `compatibilityTarget` supplied with `luriaNative` is an error. Evidence: D013
  and D022 already make an un-chosen decision an error; the target build is
  identified by `tests/fixtures/nebasechanger-research/manifest.json`.
* **D029** — the native candidate geometry: abutting pair, availability-aware
  split, no maximum primer length, no Tm bound, deterministic two-key order,
  three-way failure reporting. Settled by this pass with A7/A8/B1/B3/B5 above.

### Settled by evidence, not needing a new decision

* C19 (the 10→11 tail claim) — refuted, claim dropped.
* C17 (`buildPairMap`) — the evidence exists; the remaining gap is an
  implementation task.
* C23 (`verifyPair`) — strongly supported as a safety net; new question 9 of §20
  is answered, with the residual being whether `luria` wants the derived
  invariant of C24 as a stated requirement.
* C12 (the alignment bound is the pad, not 10) — corrected wording.
* C25 (v2.8.4 differences) — recorded.

### Left open, with the smallest experiment that would settle each

| Item | Why it is still open | Smallest experiment |
| --- | --- | --- |
| per-run grip (A9) | a Luria choice; the reference's per-side wording is reference scope only | none available: no source states a Luria rule. Settle by the caller stating a requirement, or by declining to need it. |
| Tm value equivalence (C17) | no `luria` Tm port exists | port `buildPairMap` + case 4 into the fixture generator and compare the six `designs.json` Tm values (74, 70, 74.18, 69.94) — a fixture task, not this pass |
| span equivalence with the reference (C18) | the reference reports no spans | capture a reference output format that carries positions, or accept that span equivalence is untestable and say so in the target record |
| must a pair abut (§15 q3) | Luria choice; reference back-to-back is scope-limited | none — decide |
| maximum primer length (§15 q1) | Luria choice; reference documents none and clamps only a minimum | none — decide |
| composed separate-variant representation (§15 q5, q6) | no source; `mergeCodonOps` limits are codon-mode only | none — decide |
| does the layer emit `Ta` (§15 q7) | formula verified in both builds, meaning not given anywhere | none — decide |
| `minAnnealingRun` as a field | deliberately not added in this pass (A9 decision) | — |

---

## 6. Reproduction — exact commands and actual results

All commands were run in this pass. Results below are what they printed.

```
$ cargo test --workspace
test result: ok. 179 passed; 0 failed; …
test result: ok.   8 passed; 0 failed; …
test result: ok.   6 passed; 0 failed; …
test result: ok.  14 passed; 0 failed; …
test result: ok.   0 passed; 0 failed; …   (doc-tests)
```
207 tests, 0 failures. No Rust changed in this pass; the count is unchanged
from the 207 recorded in [docs/tasks/contracts/milestone-3b-q5-primer-contract.md](/tasks/contracts/milestone-3b-q5-primer-contract.md) §8.

```
$ cd tests/fixtures/nebasechanger-research
$ python3 replay_dispatcher_geometry.py
q5p-01: n=3153 b=1199 ge=10 xe=23 le=1941 je=1199 avail3p=1941 avail5p=1199 -> MATCH
q5p-02: n=3153 b=1199 ge=10 xe=25 le=1941 je=1199 avail3p=1941 avail5p=1199 -> MATCH
q5p-03: n=3153 b=1199 ge=10 xe=25 le=1941 je=1199 avail3p=1941 avail5p=1199 -> MATCH
q5p-04/design: n=3153 b=1199 ge=10 xe=23 le=1941 je=1199 avail3p=1941 avail5p=1199 -> MATCH
q5p-07/variant-a: n=3153 b=1199 ge=10 xe=23 le=1647 je=1493 … -> MATCH
q5p-07/variant-b: n=3153 b=1493 ge=10 xe=24 le=1647 je=1493 … -> MATCH
    q5p-07 spansDisjoint replayed: True  saved: True

ALL MATCH
```

The script reads the pad literal out of the beautified live bundle and asserts
it is 10 before using it; it recovers `ge` and `xe` from the saved primer
lengths, so no Tm engine is replayed.

```
$ python3 q_calcSDM_four_build_diff.py        # paths updated to app-js/ and raw/
live len 9000  v2.8.4 len 8828
DIFFER
first diff @ 24
v2.8.4: calcSDMPrimers=function(E,ee,ne,ae,ue,ie){…ee=ee<15?15:ee,ne=ne<55?55:ne,ne=ne>72?72:ne;var ce={},J=ae,ge=ee,Te=J,Xe=ge,el=63,…
live   : calcSDMPrimers=function(I,X,re,se,Se,Me,Le,je){X=X<15?15:X,re=re<55?55:re,re=re>72?72:re,Le=Le===void 0?1/0:Le,je=je===void 0?1/0:je;var oe={},ge=se,xe=X,et=ge,Ct=xe,Tt=63,…
```

The two differences that matter are the two extra parameters (the availability
limits, `Infinity` by default) and the seeding of the forward extension: `J=ae`
(v2.8.4, pad) and `ge=se` (live, pad) — both seed from the pad — against `xe=X`
(live) and `ge=ee` (v2.8.4, the clamped minimum length) for the reverse primer.

```
$ grep of the pad literals
v2.8.4  app-js/index.a5da1636.v2.8.4.js   "ot=10" at byte offset 313404
live    live-js/beautified/index.4330b0c7.v2.11.0-live.beauty.js  line 16969 (byte 500481)  St = 10

$ substring "11" inside the extracted calcSDMPrimers bodies
v2.8.4  False
live    no occurrence in lines 17194-17291 (read in full)

$ buildPairMap in the live bundle
127266 (K = function, line 3865), 130164 (this.buildPairMap(), line 3940),
134894 (buildPairMap: K in the returned object literal, line 4063)
$ buildPairMap in the v2.8.4 bundle
69194 (TmCalc.prototype.buildPairMap = function), 73484 (this.buildPairMap(this.wseq, this.cseq)),
148613 (closure call site), 153518 (buildPairMap: U in the object literal)

$ verifyPair in the live bundle
definition vt.verifyPair at 162031 (line 4807); sole consumer at 484084 (line 16534), which pushes
"internal check failed for <idprefix>: <reason> - primers not shown. Please report this to NEB."

$ python3 q5_verify.py
prints V1-V8 diagnostics per build; no pad/minlen/minTm/tail line
```

Not run in this pass, and not claimed: the fixture
generator `tests/fixtures/q5-primer/reference/neb-q5-sdm-reference.py`, which
was run in the earlier contract pass and was not re-run here.

```
$ cargo fmt --all -- --check
(no output) exit 0

$ cargo clippy --workspace --all-targets -- -D warnings
error: length comparison to zero  --> crates/core/src/primer.rs:538:8
error: length comparison to zero  --> crates/core/src/primer.rs:649:8
error: length comparison to zero  --> crates/core/src/primer.rs:672:29
error: length comparison to zero  --> crates/core/src/primer.rs:780:8
error: length comparison to zero  --> crates/core/src/primer.rs:901:8
error: could not compile `luria-core` (lib) due to 5 previous errors
… plus 11 further findings in the same file's test module (len_zero, cmp_owned,
bool_comparison), ending:
error: could not compile `luria-core` (lib test) due to 16 previous errors
exit 101
```

**Clippy does not pass.** Every finding is in `crates/core/src/primer.rs`, the
file the implementation pass added, and every one is stylistic (`len_zero`,
`cmp_owned`, `bool_comparison`). This pass changed no Rust, so the failure is
pre-existing and was not introduced here — but D026 names clippy as a validation
command, and the contract §8 records only `cargo fmt`, `cargo build`, `cargo test`
and `git diff --check` as having been run for that pass. The gap is reported, not
repaired: fixing it edits Rust, which is outside this pass's scope.

---

## 7. Deviations, discrepancies, and what this pass did not do

### D-1. The task's description of the worktree did not match the worktree

The task brief said [docs/decisions/](/decisions/) contains "D-010 'Primer policy review'"
and an "Open questions and superseded decisions" section. It does not:

* [docs/decisions/](/decisions/) is 599 lines, D001–D027, in a `## D0NN: Title` /
  `Decision:` / `Reason:` style, with **no** open-questions section and **no**
  primer-policy decision. D010 is "Initial CDS and translation scope is
  deliberately narrow".
* The file is unmodified since commit `813bf91` (mtime 2026-10-05), so nothing
  was reverted: the content the brief describes was never in it. The strings
  "Primer policy review" and "Open questions and superseded" occur in no file in
  the repository.
* The brief's "§18" is [docs/specs/q5-primer-design.md](/specs/q5-primer-design.md) §18 ("The Luria-native
  policy"), not the task contract: [docs/tasks/contracts/milestone-3b-q5-primer-contract.md](/tasks/contracts/milestone-3b-q5-primer-contract.md)
  has §§1–8 only (numbered 1–6, then 8, then 7), and its §6 "Deliberately left
  open" lists seven items pointing at [docs/specs/q5-primer-design.md](/specs/q5-primer-design.md) §§11 and 15.

Consequence: the two policy choices were **not** pre-recorded anywhere, so this
pass is the first record of them, and the brief's instruction to record "chosen
options in the Open questions section" had no such section to write into. D028
and D029 were added as numbered entries in the file's own style instead. Per
that file's own header — "Changes to these decisions require explicit human
approval" — both are recorded as **proposed**, and the parts that are Luria
preferences rather than evidence conclusions are flagged as such below.

### D-2. A wrong decision citation in the contract, corrected

[docs/specs/q5-primer-design.md](/specs/q5-primer-design.md) §18.5 opened "Per D010 and §10". D010 is the CDS-scope
decision; the fail-closed rule is **D005**. Corrected to D005 in place, and
flagged in §21's change table rather than silently fixed.

### D-3. Byte offsets carried by the earlier pass were wrong

The earlier pass cited `buildPairMap` at 84461/87057 and `verifyPair` at
104667/308035. Re-derived in this pass against the stored bundles, the live
offsets are 127266/130164/134894 and 162031/484084 (lines 3865/3940/4063 and
4807/16534). The **line** numbers the earlier pass used for `calcSDMPrimers`
(17194) and the pad (16969) are correct; only the byte offsets were wrong. C17,
C23, C5 and the §6 reproduction block now carry the re-derived values.

### D-4. What this pass did not do

* No Rust was written, formatted or built. `PrimerDesign` is still not
  implemented; `crates/core/src/primer.rs` still delivers `PrimerPair`,
  `construct_primer`, `assemble_pair` and `require_design_constraints` and
  nothing that carries a design request into an output record. The clippy
  findings above were left unfixed for the same reason.
* No candidate generation, ranking, Tm function or preference was added.
* No fixture row was added or changed; the tests in Part C are **specified**, not
  written.
* No git operation was run: nothing was staged, committed, amended, reset,
  rebased, merged or pushed.
* No biological assumption was changed. The availability-aware split in §18.1 is
  a search rule, not a biological rule; the abut requirement, the absence of a
  maximum primer length and the absence of a Tm bound are recorded as Luria
  choices, and §20 keeps them open.

### D-5. Luria preferences that D028 and D029 record but do not settle

D028 and D029 are decision records, so they state a choice. These parts of them
are **not** conclusions of evidence, and §20 keeps the corresponding questions
open:

| Recorded in | The part that is a choice | What would settle it |
| --- | --- | --- |
| D029 | the pair must abut (§15 q3) | a source stating a Luria rule, or the caller declining to need one |
| D029 | no maximum primer length (§15 q1) | a stated Luria requirement; the reference build documents a minimum and no maximum, which is evidence about the reference, not about `luria` |
| D029 | Tm inert, preference rather than hard (§15 q2) | a defined `luria` Tm function; §11 forbids inventing one from a reference tool |
| D029 | the split of `a_f` among ties | nothing available — no source states a Luria rule; recorded as arbitrary |
| D028 | that neither policy has a default | already forced by D013/D022, so this is a conclusion, not a choice |

### D-6. Checks run, and their actual results

Run in this pass, with results in §6: `cargo test --workspace` (207 tests, 0
failures), `cargo fmt --all -- --check` (clean, exit 0), `cargo clippy --workspace
--all-targets -- -D warnings` (**fails**, exit 101, 5 lib and 16 test-target
findings, all in `crates/core/src/primer.rs`, all stylistic), `git status --short
--branch`, `git diff --check` (clean), `git diff --stat`,
`python3 replay_dispatcher_geometry.py` (all six reference designs MATCH),
`python3 q_calcSDM_four_build_diff.py` (the live and v2.8.4 `calcSDMPrimers`
bodies differ), `python3 q5_extract.py live` and `q5_extract.py v2.8.4`, and
offset/line extraction from the stored bundles for the pad literal, the
`calcSDMPrimers` parameter lists, `buildPairMap`, `verifyPair` and
`BUILD_CLEAR_KEYS`.

Not run in this pass: the fixture generator
`neb-q5-sdm-reference.py`.

### D-7. Behaviour still unverified

* The Tm **value** on either engine: no `luria` port exists.
* Span equivalence with the reference tool: untestable, because the reference
  reports no positions.
* The fragment path end to end, and therefore any fragment/NEBuilder
  compatibility.
* Whether the rotated-source coordinate semantics of the live dispatcher are
  what the reference intends: the replay reproduces the geometry, but the
  reference publishes no spans to compare against.
* Whether `verifyPair` is a specification requirement for `luria` (§20 question
  9) — the derived invariant of C24 is stated as a candidate check, not adopted.
