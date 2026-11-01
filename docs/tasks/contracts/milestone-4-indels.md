# Milestone 4 task contract: insertions and deletions

Status: **All owner decisions recorded (§4.7); the 4.4 reading was confirmed
2026-10-15.** Deletions are implemented through primer design
(`milestone-4a-deletion.md`, `-4b1-deletion-primers.md`, `-4b2-deletion-design.md`,
D031), including Q5 workflow wiring and the frameshift warning
(`-4c1`, `-4c2`). **Insertions (pass 4) are implemented end to end** under D032:
`-4d1` (represent, apply, verify), `-4d2` (5′ tails in `primer.rs`), `-4d3a`
(design), `-4d3b` (fixture and the 100-nt warning), `-4d4` (workflow and
frameshift warning). 396 tests pass. Still open: pass 5, unequal replacements
and degenerate expansion.
Date: 2026-10-14
Planner: Claude. Not implemented.

## 1. What this milestone is

luria designs primers for substitutions of 1–3 bases. An insertion or a
deletion cannot be *represented* today, let alone designed. This milestone
adds both, end to end: representation, verification, and primer design.

It is deliberately a separate milestone because it is the first change that
touches the mutation layer's own rules rather than adding a layer on top.

## 2. What currently blocks it, established 2026-10-14

Each of these is a fact about the code as it stands, not a design choice:

1. **`MutationResult` has two variants**, `Nucleotide` and `Residue`
   (`mutation.rs`). Neither can describe a length change.
2. **`NucleotideMutationIntent` carries one `replacement: char`** at a
   one-base `position` interval, and its constructor refuses an interval that
   selects more than one base. There is no multi-base replacement, so no
   insertion or deletion.
3. **An empty interval is not representable.** `interval::create` refuses
   `high == low` with `EmptyInterval` (`interval.rs`). An insertion point is
   naturally a zero-width position, so the obvious encoding is unavailable.
   This is the single largest representation question (§4.1).
4. **`verify` requires equal lengths**: "a substitution preserves length"
   (`mutation.rs`). It is a postcondition of verification, so a
   length-changing edit fails verification today.
5. **The primer layer assumes a primer's text is as wide as its span**
   (`primer.rs`: `total as u64 != width` is an error). For an insertion the
   forward primer is longer than the template interval it covers, so this
   check, as written, refuses such a primer.
6. **D029 and D030 scope themselves to "a contiguous replacement region of at
   most 3 bases" on a linear template.** An indel is outside that scope by
   construction, so the design policy needs extending, not just the layers
   below it.
7. **There is no reference design to check against.** The research notes
   record that the reference tool routes insertions and deletions to its
   fragment path, which this project never investigated
   (`docs/research/nebasechanger-live-build-4330b0c7-2026-10-10.md` §17.8
   item 1, via `docs/specs/q5-primer-design.md` §19). So unlike substitutions,
   there is no captured external design to compare against, and D014's
   "comparison is development validation, not proof" leaves us with no
   comparison at all.

## 3. Biology this has to get right

Stated here so the decisions in §4 can be judged against it, not to prescribe
an answer.

In Q5 site-directed mutagenesis the two primers abut and amplify the whole
plasmid outward from the junction; the product is then circularised. So:

- **A deletion** is made by placing the primers so that the bases to be
  removed lie *outside* the amplified product: the forward primer starts
  immediately 3′ of the deleted region and the reverse primer immediately 5′
  of it. Neither primer needs to carry any non-template sequence. The primers
  still abut in the *product*, but on the *template* they are separated by
  exactly the deleted bases.
- **An insertion** is made by carrying the inserted bases as a 5′ tail on one
  primer (or split across both), with the annealing portions abutting on the
  template. The tail does not anneal, which is why the contract already
  separates a primer's `tail` from its `annealing` portion
  (`docs/specs/q5-primer-design.md` §5) and why `minAnnealingOverlap` is about the
  annealing portion only.

Two consequences that matter for §4:

- A deletion breaks the current pair rule that the two primers abut **on the
  template**. The `abuts()` invariant in `primer.rs` is a template-coordinate
  property.
- An insertion needs the tail machinery that `primer.rs` has fields for but
  `luriaNative` has never produced: D029 only emits inline placement, and
  `PrimerPlacement::ConfineToTail` is currently refused.

## 4. Decisions required before any implementation

### 4.1 How is an insertion point represented?

An insertion happens *between* two bases. Options:

1. **A one-base interval plus a side.** Reuse `NucleotideInterval` and say
   whether the insertion goes before or after it. No change to `interval.rs`;
   adds a side field that every caller must get right, and makes "insert at
   the very start" and "insert at the very end" asymmetric.
2. **Allow zero-width intervals.** Remove the `EmptyInterval` refusal for
   this use, so `[p, p)` means "between p−1 and p". Mathematically the right
   encoding for a half-open scheme, and uniform at both ends. But
   `EmptyInterval` is a Milestone 1 rule that other code relies on, and
   relaxing it would let an empty interval appear anywhere an interval is
   accepted, including as a feature or CDS extent.
3. **A separate `InsertionPoint` type.** A distinct type carrying one
   coordinate, with its own validation and its own meaning, leaving
   `NucleotideInterval` untouched. More types, no weakening of an existing
   rule.

**Planner's recommendation: option 3.** It leaves Milestone 1's interval rule
intact, and D009's "coordinate conventions are explicit" is better served by a
type whose name says what the coordinate means than by a general interval with
a side flag. The cost is one more type.

### 4.2 Does `verify` keep a length rule?

Verification currently asserts that an edit preserves length, and that is a
real check that has caught nothing yet only because no edit changes length.
Options:

1. **Per-kind expected length.** Verification computes the length the edit
   should produce — source length, minus the deletion, plus the insertion —
   and requires exactly that. Keeps a strong check and makes it
   kind-dependent.
2. **Drop the length check for indels** and rely on the difference check.
   Weaker, and D013 exists precisely so verification does not trust the
   generator.

**Planner's recommendation: option 1.** It is the same strength of check,
restated for an edit that changes length, and it keeps verification
independent of the generator.

### 4.3 What is the pair invariant for a deletion?

`abuts()` is a template-coordinate property today. Options:

1. **Abut in the product, separated on the template by exactly the deletion.**
   The invariant becomes: reverse end + deleted width = forward start. The
   existing `abuts()` stays true for substitutions and insertions as a special
   case where the deleted width is zero.
2. **Keep template abutment and refuse deletions.** Not a real option if this
   milestone is to happen; recorded for completeness.

**Planner's recommendation: option 1**, with the generalised invariant named
in the contract and the old one derived from it, so no existing test changes
meaning.

### 4.4 Which primer carries an insertion, and is it split?

For an insertion the inserted bases must be carried as a 5′ tail. Options:

1. **All on the forward primer.** Simplest, and matches how the contract
   describes a tail. Makes the forward primer longer than the reverse by the
   insertion length, which the Tm balance rule of D030 then has to cope with
   — and a tail does not anneal, so it should arguably not count toward Tm at
   all (§4.5).
2. **Split between the two primers.** The reference tool's own UI text
   mentions splitting a long insertion across both primers
   (`docs/research/nebasechanger-client-investigation.md`, the
   `seqfpflap`/`seqrpflap` fields at the call site suggest a 6-base
   threshold). Balances the two primers; needs a documented split rule, which
   we do not have, and the fragment path it comes from was never
   substantiated.
3. **All on the forward primer below a stated length, refuse above it.**
   Conservative: supports the common short insertion and refuses what it
   cannot justify, rather than inventing a split rule.

**Planner's recommendation: option 3**, with the threshold stated as a policy
value in a new decision entry. A split rule with no source is exactly what
D014 forbids.

### 4.5 Does a tail count toward Tm?

A 5′ tail does not anneal to the template. D030 sizes primers by Tm and
balances the pair. Options:

1. **Tm of the annealing portion only.** Thermodynamically the right quantity
   for deciding whether a primer anneals, and consistent with
   `minAnnealingOverlap` being about the annealing portion.
2. **Tm of the whole primer.** Simpler, and what a user typing the oligo into
   a Tm calculator would get, but it overstates annealing for a long tail.

**Planner's recommendation: option 1**, with the whole-primer Tm also reported
and labelled, since that is the number a user will see elsewhere.

### 4.6 Scope of this milestone

Which to support: insertion, deletion, or a replacement of unequal lengths
(delete 5, insert 2)? The last is what the reference tool calls
"Indel/Substitution" mode and is the general case. Options:

1. **Deletion first, then insertion, then unequal replacement**, as three
   passes.
2. **All three at once**, since a general replacement subsumes them.

**Planner's recommendation: option 1.** Deletion needs no tail machinery at
all, so it is a much smaller change and it exercises the §4.3 pair invariant
on its own.

### 4.7 Owner answers, 2026-10-14

| Decision | Answer |
| --- | --- |
| 4.1 insertion point | **Option 3**: a separate `InsertionPoint` type; `EmptyInterval` stays as it is |
| 4.2 verify length rule | **Option 1**: per-kind expected length, checked exactly |
| 4.3 deletion pair invariant | **Option 1**: reverse end + deleted width = forward start; template abutment is the zero-width case |
| 4.4 which primer carries an insertion | **Follow the reference tool**: a "confine mutations to primer 5′ tails" option, and splitting across both primers when it is off (see the note below) |
| 4.5 does a tail count toward Tm | **Option 1**: annealing portion only; whole-primer Tm also reported, labelled |
| 4.6 scope order | **Option 1**: deletion, then insertion, then unequal replacement |

**Note on 4.4, to settle before pass 4.** The reference tool's own help
text (`docs/references/nbc_help_v284.txt`, lines 26–28, 67–69, 155–157) does not
quite match "confine on = one primer, off = split":

- **Insertions are always 5′ tails**, whatever the option. Splitting is by
  *length*: "If 6 or fewer nucleotides are to be inserted, they will be placed
  at the 5′ end of one primer. If more than 6 … they will be split between the
  two primers to keep the primer lengths and Tm's similar." The help text does
  not say which primer gets a short insertion.
- **The confine option is about substitutions**: with it off, a substitution
  of 6 nt or fewer is placed *inline* in the forward primer, which is what
  D029/D030 do; with it on, the replacement "will always be attached to the
  5′ end of the primers, regardless of the length of the substitution". It
  says "primers", plural, so confinement does not by itself stop a split.
- **The split rule itself is unsubstantiated.** The extracted code has a split
  point `ceil(target.length / 2)`, but a recorded run of it attached an
  8-nt replacement whole to the forward primer with an empty reverse tail
  (`docs/research/nebasechanger-cycles-1-4-2026-10-10.md` §6.1), which
  contradicts the help text's ">6 split". The research table rates the
  tail and split behaviour "Medium: bundle + help text; not executed".

So the planner's reading of the owner's answer, **confirmed by the owner on
2026-10-15**:

1. An insertion is always carried as a 5′ tail (never inline).
2. At 6 nt or fewer it goes whole on the **forward** primer.
3. Above 6 nt it is split, `ceil(n/2)` on the forward tail and the rest on
   the reverse tail (as its reverse complement), as a **luria-native** rule
   stated in a new decision entry. It follows the help text's stated purpose,
   "to keep the primer lengths and Tm's similar", not the unexecuted code.
4. A "confine to 5′ tails" option, off by default, applies to substitutions:
   on, a substitution is carried as a forward 5′ tail rather than inline.

Rule 3 is luria-native, not a reproduction of the reference tool, and must
be recorded as such in the decision entry that pass 4 adds. None of this
affects passes 1–3: a deletion carries no tail.

### 4.8 D031: deletion primer geometry

**Approved by the owner 2026-10-15 as drafted below**; recorded as D031 in
`docs/decisions/`. The frameshift question at the end was answered the same
day: **warn, still apply**.

For a deletion of `[s, e)`, width `w = e − s`, on a linear template of length
`n`, with bound `b = minAnnealingOverlap`:

* **Geometry.** The forward primer's 5′ end is at `e` and it reads rightward,
  `[e, e + f)`. The reverse primer's 5′ end is at `s` and it reads leftward,
  `[s − r, s)`. Source: the reference tool's help text, "For deletions, the 5′
  ends of the primers align with the edges of the deletion"
  (`docs/references/nbc_help_v284.txt` line 155). Neither primer carries any
  non-template base.
* **Pair invariant** (owner decision 4.3): reverse end + `w` = forward start.
  In the product the two 5′ ends meet at `s`.
* **Starting lengths.** Both primers start at `max(b, 15)`. D030's forward
  start of 20 exists to put 10 unchanged bases on each side of a mismatch; a
  deletion primer has no mismatch, so both primers are entirely annealing
  sequence and use the reverse primer's existing floor.
* **Everything else is D030 unchanged.** The 55 °C floor, G/C 3′ end below
  63 °C, forward first, 5 °C balance, Ta = lower Tm + 3 capped at 72 °C,
  ambiguity codes ending availability, buffer named by the caller, and the
  same two failure kinds. Both Tms are on the matched basis, because neither
  primer has a mismatch.
* **Scope.** Linear templates only, as for D030. No cap on `w`. The reference
  tool refuses when fewer than 62 bases remain after a deletion, but that
  number has no stated source, so luria only requires both primers to fit.

**Not decided by D031.** A deletion inside a CDS whose width is not a multiple
of 3 shifts the reading frame. The mutation layer allows it, and Milestone 4a
does not warn. Whether to warn, refuse, or leave this to the caller is a
separate owner question.

### 4.9 D032: insertion representation and primer geometry

**Approved by the owner 2026-10-15** and recorded as D032 in `docs/decisions/`,
with two answers that settle the draft below. An insertion asserts **the base
on each side** of the point, so `1 ≤ p ≤ n − 1`. There is **no cap on
insertion length**, but any primer over **100 nt** (the owner's practical IDT
ordering limit) gets a quality warning.

Drafted 2026-10-15 for pass 4. Builds on decisions 4.1, 4.2, 4.4 (as
confirmed) and 4.5.

**Representation (4.1).** An `InsertionPoint` is one 0-based coordinate `p`
with `0 ≤ p ≤ n`, meaning "between base `p − 1` and base `p`". The inserted
bases occupy `[p, p + k)` of the edited artifact. Inserted bases are `A`, `C`,
`G`, `T` only: degenerate insertions are pass 5. There is no cap on `k`.

**Verification (4.2).** The edited artifact has exactly `n + k` bases and is
`source[..p] + inserted + source[p..]`.

**Geometry**, for an insertion of `k` bases at `p` on a linear template:

* Both primers' **annealing** portions abut on the template at `p`. The
  forward primer anneals to `[p, p + f)` and the reverse primer to
  `[p − r, p)`. The pair invariant is D031's with `w = 0`.
* `k ≤ 6`: the whole insertion is the forward primer's 5′ tail. The reverse
  primer has no tail.
* `k > 6`: the forward tail is the last `ceil(k/2)` inserted bases. The
  reverse tail is the reverse complement of the first `floor(k/2)`. In the
  product the two tails meet end to end and rebuild the insertion.
* Both annealing portions start at `max(b, 15)`. Neither has a mismatch, so
  D030's forward start of 20 does not apply, as in D031.
* **Tm (4.5).** Sizing, balance and Ta use the annealing portion's Tm on the
  matched basis. Each primer's **whole-primer** Tm is also reported, labelled.
  It is the Tm the primer has against the product in later cycles, and the
  number a user's own calculator will show.
* Everything else is D030: 55 °C floor, G/C end below 63 °C, forward first,
  5 °C balance, ambiguity codes ending availability, and the same two failure
  kinds.
* The "confine to 5′ tails" option does not change an insertion: an
  insertion is always a tail.

**Not decided by D032.** What an insertion **asserts** under D011 (see the
question below). A substitution asserts the base it replaces and a deletion
the bases it removes; an insertion removes nothing.

### 4.10 D033: unequal replacements and degenerate expansion

**Approved by the owner 2026-10-15** and recorded as D033 in
`docs/decisions/`, with the two open points answered: the expansion count is
**not capped** but warns past **64** designs, and an expanded design is named
the base name plus its concrete replacement sequence (`mut-1-AC`).

Drafted 2026-10-15 for pass 5, from three owner answers of the same day:
an unequal replacement **behaves as a substitution does**; the reference
tool's **6-nucleotide rule** decides inline versus tails; and ambiguity codes
in the replacement **expand combinatorially into one primer pair per concrete
sequence**.

**What a replacement is.** A target region `[s, e)` of width `w >= 1` and a
replacement text of length `m >= 1`. `m == w` is today's substitution,
generalised: the replacement may be any length, so one request can carry
several changes at once, which is how two codons are altered with one primer
pair. A replacement with `m != w` changes the artifact's length, as an indel
does. `w = 0` is an insertion (D032) and `m = 0` a deletion (D031); neither is
reached through this path.

**Geometry.** Let `t = max(w, m)`.

* `t <= 6`: **inline**, exactly D030's geometry with the replacement region in
  place of the substituted bases. The junction stays at `s - 10`, the forward
  primer keeps its fixed 10-base 5′ flank, and its 3′ flank starts at
  `max(10, b - 10)`. This raises `MAX_REPLACEMENT_REGION` from 3 to 6.
* `t > 6`: **split across both 5′ tails**, exactly D032's rule with the
  replacement text in place of the inserted text: the forward tail is the last
  `ceil(m/2)` bases and the reverse tail the reverse complement of the first
  `floor(m/2)`. Both annealing portions' 5′ ends sit at the edges of the
  target region — the forward anneals from `e`, the reverse to `s` — so the
  replaced bases are carried entirely by the tails and annealed over by
  neither primer. Starting lengths are `max(b, 15)`, as in D031 and D032.
* Sizing, the 55 °C floor, the G/C end below 63 °C, the 5 °C balance and
  `Ta = min(72, lower Tm + 3)` are D030's throughout. On the tail path, sizing
  uses the annealing portions only and the whole-primer Tm is reported, as
  D032 does.
* The pair invariant generalises D031's: reverse end + `w` = forward start.

**Degenerate expansion.** The replacement may contain IUPAC ambiguity codes.
Each is expanded over the bases it stands for, and the request yields **one
design per concrete sequence**. Each position's bases are taken in `A`, `C`,
`G`, `T` order and the **leftmost position varies fastest**, so `NS` gives
`AC, CC, GC, TC, AG, CG, GG, TG`, which is the order the help text lists.
The planner first drafted this as leftmost-slowest, which would give
`AC, AG, CC, CG, …`; the quoted list is the authority and the draft was
corrected before any code was written. The expansion happens at the
**request** layer, before any edit: each concrete sequence is its own
mutation, independently verified, with its own primer pair. Nothing in
`mutation.rs` loosens — an applied edit still writes only `A`, `C`, `G`, `T`.

**Expansion count and naming**, answered 2026-10-15: no cap, but a warning
past 64 designs; and an expanded design is named `<base>-<concrete
sequence>`, which keeps names distinct by construction.

## 5. Implementation outline

In order, each pass with its own tests and fixtures:

1. **Representation.** The §4.1 type, a `MutationIntent` for each kind, and
   the `MutationResult` variants to match. No design layer yet.
2. **Verification.** §4.2's per-kind expected length, and the difference
   check restated for an edit that moves coordinates.
3. **Deletion design.** §4.3's pair invariant; `primer.rs`'s text-width check
   generalised; D029/D030 extended in a new decision entry.
4. **Insertion design.** The tail path, §4.4's threshold, §4.5's Tm rule.
5. **Degenerate replacements.** The owner's note of 2026-10-14
   (`docs/specs/q5-primer-design.md` §19): ambiguity codes in the *replacement*
   expand combinatorially into one pair per concrete sequence. Needs the
   mutation layer's A/C/G/T restriction revisited, and makes one request
   produce several designs.

## 6. Do not

- Do not relax `EmptyInterval`, the `verify` length rule, or `primer.rs`'s
  text-width check without the matching §4 decision recorded first.
- Do not infer an insertion-splitting rule, a tail-length limit, or a
  Tm-for-tails rule from the reference tool's behaviour: its indel path is
  unsubstantiated (§2 item 7), so it is not evidence (D014).
- Do not change any existing substitution test's meaning. The generalised
  invariants of §4.2 and §4.3 must reduce to the current ones when the
  inserted and deleted widths are zero.

## 7. Required checks

The five in `AGENTS.md`, plus: every new fixture expectation independently
derived and recorded, as `tests/fixtures/tm/README.md` and
`tests/fixtures/q5-primer/reference/native-design.py` do.
