# Q5 primer design: contract

**Status:** specification. No primer-design algorithm is implemented in
`crates/core`, and nothing here is asserted as a property of code that does not
yet exist.

This document fixes the boundary of a future deterministic primer-design layer:
what it takes, what it must return, which rules are validity requirements rather
than preferences, and how its behaviour can be tested before an algorithm is
chosen.

The project principle applies throughout: **AI interprets intent; deterministic
software performs the biology.** A primer pair is a biological product. It is
produced by code under this contract, never by a model.

---

## 1. What is decided, and by what

Each rule below carries the decision that fixes it, or the word **new** when no
existing decision covers it. New rules are proposals in this document; they are
not yet recorded in `docs/decisions/` and are not binding until they are.

| Rule | Source |
| --- | --- |
| Q5 is the default design style | D006 |
| Primer design is not reproduced from NEBaseChanger | D008 |
| NEBaseChanger is an external reference, not the source of truth | D011, D016 |
| Coordinates are zero-based half-open internally | D004 |
| FASTA headers carry 1-based inclusive ranges | D015 |
| Sequence case is non-semantic | D001 |
| A failed operation produces no partial output | D010 |
| An empty result is not a valid empty answer | D014 |
| A requested operation that cannot be honoured is an error, not a silent substitution | D013 |
| Mutations are applied against the pristine template | D021 |
| Separate variants are composed at the request layer, not by re-running SDM | D020 |
| A construct is a linear sequence unless a circular CDS is annotated | D005, D019 |
| Strand is explicit and typed | D002, D003 |
| **Primer representation is 5′→3′, explicit, and typed** | new |
| **A primer is split into an annealing portion and a 5′ tail, and the two are typed separately** | new |
| **A hard bound applies to the annealing overlap, not to the whole primer** | new |
| **Tm is computed by a project-owned function, not by a model or an external service** | new |
| **A design reports the constraint set it was evaluated against** | new |
| **A design reports the sequence it was built against, by digest** | new |
| **The primer-design layer is a pure function of its inputs** | new |
| **Design policy is explicit, typed, and required: `luriaNative` or a versioned compatibility target** | new (§16) |
| **A compatibility mode is not a parameter preset; matching numerical defaults are not algorithmic compatibility** | new (§16) |
| **The initial Luria-native scope is one inline substitution of a contiguous region of at most 3 bases, one back-to-back pair, no Tm selection** | D029 (approved 2026-10-14) |
| **The initial Luria-native split is mutation-centered, with at least 10 unchanged bases on each side, the extra base 3′, and a reverse primer of at least 15; availability is a feasibility test, not a split rule** | D029 (approved 2026-10-14) |
| **A compatibility design reports which behaviours it reproduces and which it does not** | new (§19) |
| **The existing fixture's `minimum_overlap_bp` is not mapped to `minAnnealingOverlap`** | new (§20) |

---

## 2. Inputs

The layer is a pure function. It has no clock, no randomness, no environment,
no service call, and no access to a model.

### 2.1 The mutation result

A validated `MutationResult` (D017) is the only biological input. The design
layer must not reach behind it to re-derive anything:

| Field | Use in design |
| --- | --- |
| `sequenceBefore` | The **pristine template**. The reference for primer construction (§4). |
| `sequenceAfter` | The **edited sequence**. The primer text is a subsequence of this (§5). |
| `edits[]` | Each edit's `position` (zero-based), `original`, `replacement`, `strand` (D003). |
| `verification` | The applied-vs-expected check (D018). A design consumes a verified result; it does not re-verify. |
| `provenance` | Carried into the output so a primer pair traces to the request that produced it. |

**Rule (new):** the design layer accepts a `MutationResult`, not a bare
sequence and a coordinate. A design whose input is only `(sequence, position)`
has lost the provenance that makes it reproducible.

**Rule (new):** the layer re-checks the one structural invariant that matters to
it — that `sequenceAfter` equals `sequenceBefore` with each edit applied at its
recorded position. A mismatch is a defect in the upstream layer, reported as
such. This is a guard against an assembled `MutationResult`, not a
re-implementation of `verifyMutation`.

### 2.2 The design request

| Input | Type | Required | Meaning |
| --- | --- | --- | --- |
| `style` | enum | yes | `q5_polymerase` (D006) or `classic_two_primer`. No other style is in scope. |
| `minAnnealingOverlap` | integer ≥ 1 | yes | Hard lower bound on the annealing overlap of every emitted primer. A bound, not a target (§9). |
| `maxAnnealingOverlap` | integer ≥ `minAnnealingOverlap` | no | Upper bound. Absent means "not supplied" (§10). **Not accepted for now** (owner, 2026-10-14): it cannot change any outcome under the centered split, so it is added only with a Tm-based length policy. |
| `minTm` | number | no | Minimum acceptable Tm per primer. Absent means no preference. |
| `tmFunction` | identifier | no | Which Tm function to use. Absent means the project default (§12). |
| `saltModel` | identifier | no | Which salt/buffer model the Tm function assumes. Absent means "not supplied" (§10). |
| `mutationPlacement` | enum | no | `inline` or `confineToTail`. A caller's choice, not a biological rule. |
| `maxDesignCandidates` | integer ≥ 1 | no | Search bound. A bound, not a biological rule. |
| `designPolicy` | enum | **yes** | `luriaNative` or `nebBaseChangerCompatible`. Required, not defaulted: a policy the caller did not choose is a decision the caller did not make (§16). |

**Rule (new):** every field of the design request is carried into the output
verbatim, including the fields that were absent. An absent field is reported as
absent; it is never silently replaced by a default. This is D013 and D022
applied at the design layer: an unasked-for default is a decision the caller
did not make, and it must be visible.

**Naming note.** The existing fixture `input/mutation_spec.json` calls this
field `minimum_overlap_bp`. The reference tool's own documentation calls the
corresponding parameter *minimum primer length*, and states that when
mutations are confined to the 5′ tail it "only applies to the annealing
portion of the primer" (§13.1 item 5). Those are not the same quantity. The
contract names the quantity it means — `minAnnealingOverlap` — and the
existing fixture's name is left as it is, with the difference recorded in
§15. Question 4.

### 2.3 What the layer is not given

The layer is **not** given, and must not invent:

- a Tm value, a salt concentration, or a DNA/DMF/DMSO composition;
- a primer length chosen to satisfy an unstated preference;
- a "typical" or "recommended" annealing temperature;
- an interpretation of a construct name, a part name, or a phenotype.

If the caller did not supply a `minTm`, the layer does not pick one (§10).

---

## 3. Outputs

A design is a typed result, not text.

```
PrimerDesign
  style              q5_polymerase | classic_two_primer
  policy             luriaNative | nebBaseChangerCompatible   ← which rules produced this
  policyTarget       compatibility target version, or absent for luriaNative (§16)
  substantiation     which behaviours a compatibility design reproduces, and which it does not (§19)
  builtAgainst       digest of sequenceBefore      ← which template this describes
  evaluatedAgainst   the constraint set actually used, absent fields included
  pair               PrimerPair
    forward          Primer
    reverse          Primer
    relationship     how the pair relates to the edit and to each other
  constraints        per-primer record of each hard rule, evaluated
  tm                 per-primer Tm under the stated function and salt model
  unresolved         inputs the contract could not satisfy, and why
```

Each `Primer` carries:

| Field | Meaning |
| --- | --- |
| `role` | `forward` or `reverse` |
| `sequence` | primer text, **5′→3′**, exactly as it is to be typed into the kit |
| `orientation` | the strand this primer reads on, as an explicit value |
| `tail` | the 5′ portion that carries the mutation and does not anneal (§5) |
| `annealing` | the 3′ portion that is wild-type context and does anneal |
| `span` | the interval of the **original template** the primer covers, zero-based half-open internally, 1-based inclusive in any header (D004, D015) |
| `annealingSpan` | the interval of the template the `annealing` portion covers |
| `overlap` | the count of `annealing` positions that coincide with the original template |
| `overlapPositions` | which primer positions those are, so the overlap is checkable and not merely counted |
| `changedPositions` | primer positions whose text differs from the template |
| `tm` | the computed value, plus the function and salt model that produced it |

**Rule (new):** the primer text and the span are both reported and checked
against each other. A primer whose text is not the expected subsequence at its
span is a defect, and the check catches it.

---

## 4. The reference sequence

**Rule:** the reference for primer construction is `sequenceBefore` — the
pristine template, the sequence that was mutated.

This is fixed by D021: mutations are applied against the pristine template, so
the template is the single sequence that all edits, all coordinates, and all
primer spans share. For a composed separate-variant request (D020), the
reference is the same pristine template, and each variant's primer pair is
built against it.

Consequences the tests must hold:

1. **Every primer span is an interval of the original template**, not of the
   edited sequence. The two sequences have the same length only for a
   substitution; for an insertion or a deletion they do not, and a span stated
   against one is meaningless against the other.
2. **The primer text is taken from the edited sequence.** A primer that carries
   the mutation contains the replacement text, not the original. Primer text and
   primer span therefore refer to two different sequences, and the contract says
   so rather than leaving the reader to infer it.
3. **Coordinates do not shift.** Because spans are stated against the template,
   a primer pair for a separate variant is stated in template coordinates
   regardless of which other variants were requested.

**Rule (new):** for a composed request, each variant's primer pair is reported
as its own `PrimerDesign`, each with its own `builtAgainst`. A composed request
does not produce one merged pair. This is the primer-layer expression of D020:
separate variants are independent edits against one template, and a single pair
cannot express two of them.

---

## 5. The pair, the tail, and the annealing portion

A Q5 primer pair is **back-to-back**: the two primers meet at the edit rather
than overlapping it. Each primer has two parts, and the contract types them
separately because the rules apply to different ones:

- the **5′ tail** carries the mutation. It does not match the template and is
  not counted as overlap;
- the **annealing portion** is wild-type context. It is what must overlap the
  template, and what Tm is meaningful about.

For a substitution at template span `[s, e)` with replacement length `L`:

- the **changed region** is `[s, e)` in template coordinates;
- the **edited region** is `[s, s + L)` in edited-sequence coordinates;
- the pair's inner boundary sits at the edit, and each primer's annealing
  portion extends away from it into unaffected template.

For an **insertion** at position `p` — `original` empty, `replacement`
non-empty — the changed region is empty and the rule degenerates. The contract
states the case rather than leaving it to a general rule:

- the insertion point is a single template position `p`, the boundary before
  template index `p`;
- the inserted text occupies `[p, p + L)` in the edited sequence and nothing in
  the template;
- the pair's inner boundary sits at `p`;
- the inserted text is carried in the primers' 5′ tails, so **all** of each
  primer's overlap must come from context on either side.

For a **deletion** of `[s, e)`, the removed text has no representation in the
edited sequence, so no primer position can "carry" it. A pair for a deletion is
defined by the context it preserves: both primers' annealing portions must
overlap unaffected template, one on each side of the removed span.

**Rule (new):** overlap is counted **position by position** — the count of
primer positions whose base equals the template base underneath that position.
It is not the span width minus the changed width. The two differ when the
replacement shares a base with the original, and only the position-by-position
count is meaningful, because a shared base is a base a human assembler cannot
tell apart. The SID-1 fixture is exactly this case: `GAT → AAC` shares the
middle base, so the forward primer's whole-primer position-match count is 21
while its span is 23 wide. Its **annealing** overlap is 20, because the shared
base is a mutated position and not wild-type context — H8 counts only annealing
positions. `tests/fixtures/q5-primer/reference/designs.json` records both
quantities under different names (`annealingOverlap`, `wholePrimerMatchCount`)
so that neither is read as the other.

**Rule (new):** the reverse primer's text is the reverse complement of the
template region its `annealing` portion covers, stated as a checkable
invariant: `reverse.annealing == revcomp(template[reverse.annealingSpan])`.
A design reporting a reverse primer that fails this invariant is rejected.

---

## 6. Strand orientation and 5′→3′ representation

**Rule:** every primer is reported in **5′→3′** form, as it is to be typed into
the kit, together with an explicit `orientation` value. There is no implicit
convention and no case-based convention.

The two roles are defined by the construct's own orientation, not by a sequence
operation:

- the **forward** primer reads on the construct's own strand and its 5′ end sits
  at the inner boundary;
- the **reverse** primer reads on the opposite strand and its 5′ end also sits
  at the inner boundary, so the two 5′ ends meet back-to-back and its text is
  the reverse complement of the template region it covers.

**Rule (new):** case carries no orientation. Fixture primer text is
case-normalised (D001). Any case pattern in a primer — including the convention
of lower-casing changed bases — is presentation, and is not part of the
contract. A design that needs to communicate which positions changed uses
`changedPositions`, not case.

**Rule (new):** a design that cannot state both primers in 5′→3′ form with
consistent, opposite orientation is not a design. This is a validity
requirement, not a preference.

**Rule (new):** which primer is `forward` is a property of the construct's
orientation, not of the edit. A residue edit whose semantics are on a reverse
CDS (D003) is the same DNA edit and produces the same pair; only the residue
numbering that justified the edit differs. The fixture case `q5p-08` pins this.

---

## 7. Hard validity requirements

These must hold of every emitted pair. A pair that fails one is not a result;
it is a failure (§10).

| # | Requirement | Why it is hard |
| --- | --- | --- |
| H1 | Every primer's `overlap ≥ minAnnealingOverlap` | The caller's stated requirement, and the only numeric input with biological meaning. |
| H2 | The pair covers the changed region of its edit (§5) | A pair that does not cover the edit does not implement the request. |
| H3 | `forward.sequence` equals the edited sequence at its edited span, and `reverse.annealing == revcomp(template[reverse.annealingSpan])` | Text and coordinates must agree. This is the invariant that catches an off-by-one. |
| H4 | Both primers are stated 5′→3′ with consistent, opposite `orientation` (§6) | A primer whose orientation is unstated cannot be typed into a kit correctly. |
| H5 | The pair's relationship to the edit is stated, not implied | A reader must be able to tell which edit a pair implements from the output alone. |
| H6 | Every primer is a valid sequence over the project alphabet | A primer containing a non-base character is not a primer. |
| H7 | `builtAgainst` matches the digest of the sequence the design was given | A design cannot be attributed to the wrong template. |
| H8 | `tail`, `annealing` and the mutated positions partition `sequence` — under tail placement the mutation lies in `tail`, under inline placement it is a third part — and `overlap` counts only `annealing` positions | The bound in H1 is a bound on the annealing portion. Counting the whole primer would let a long tail, or the mutation itself, satisfy a bound that the context does not meet. |

**H2 for edits that change no template position in place** (D031–D033,
recorded 2026-10-16). A deletion, and an insertion or replacement placed in the
5′ tails, has no changed region for a primer to anneal over. For these, H2
means: (a) the pair invariant holds — reverse annealing end + `w` = forward
annealing start, with `w = 0` for an insertion; (b) neither annealing portion
overlaps the replaced or deleted region `[s, e)`; and (c) for a tailed edit,
the reverse tail read in the template's direction followed by the forward tail
equals the inserted or replacement text exactly. An inline replacement
(`max(w, m) ≤ 6`) is covered by the original H2.

**Rule (new):** H1–H8 are checked on the output, not assumed of the algorithm.
An algorithm that satisfies them by construction still gets checked, and one
that does not is caught.

---

## 8. Preferences and optimisation

These shape which valid design is chosen. Failing one is never a failure.

| # | Preference | Behaviour when unsupplied |
| --- | --- | --- |
| P1 | Tm of each primer at or above `minTm` | If `minTm` is absent, no Tm preference is applied, and the output says so. |
| P2 | Tm of the two primers close to each other | Applied only if P1 is applied. |
| P3 | Shorter primers preferred among those satisfying H1–H8 and P1–P2 | Always applied. It is what makes the search finite and the output stable. |
| P4 | Comparable annealing overlap on each side of the edit | Applied only if supplied. |

**Rule (new):** preferences never override hard requirements. A design that
improves Tm by violating H1 is not better; it is invalid.

**Rule (new):** the search is deterministic. Same inputs, same design. Where
several designs tie under P1–P4, the tie is broken by a stated, input-
independent order, and that order is part of the contract. A layer that returns
a different pair on a second run is not reproducible, and reproducibility is a
stated property of the product.

---

## 9. `minAnnealingOverlap`: a bound, not a target

**Rule:** it is a lower bound on the annealing overlap of **every** emitted
primer. It is not a target, not a starting length, and not a fallback.

The distinction is not hypothetical. In the SID-1 fixture the bound is 15 and
the reference pair's **annealing** overlaps are 20 and 23 — both above it,
neither equal to it. A design that read 15 as a target would emit a different,
shorter pair.

**Rule (new):** a design whose primer overlaps by exactly the bound is valid;
one below it is not. The fixture set contains both situations so that confusing
`>=` with `>`, or a bound with a target, is caught. The cases state which
primer they witness, because the reference tool does not apply its minimum-length
parameter to both primers in the inline `replace` path:

- `q5p-01`: both primers above the bound 15 (annealing 20 and 23) — the
  bound-versus-target witness.
- `q5p-02`: bound 24. The reverse primer moves to annealing 25, above it. The
  forward primer stays at 20, **below** it, so that reference design violates
  H1. The case witnesses that raising the bound changes one primer, not that
  every primer meets the bound.
- `q5p-03`: bound 25. The reverse primer's annealing overlap is **exactly** 25,
  which separates `>=` from `>` **for the reverse primer only**. The forward
  primer is at 20, below the bound, so the case is not a pair-level witness and
  the reference design violates H1.

`cases.tsv` records this per case in a `reference_check` column with the exact
measured quantity, so the claim and the data are checked against each other
rather than against prose.

---

## 10. Failure, and reporting what was not supplied

A design either satisfies H1–H8 under the supplied constraints or it fails.
There is no third state.

**Rule (D010):** a failed design produces no pair, no partial pair, and no
"best effort" pair.

**Rule (new):** a failure reports **which** constraint failed and **the range
that was searched**. "No design found" alone is not actionable.

**Rule (new):** an input the contract needs but the caller did not supply is
reported as **unresolved**, with the effect of leaving it unset.

| Unresolved input | Effect the output must state |
| --- | --- |
| `minTm` absent | No Tm preference was applied. Reported Tm values are informational, not satisfied constraints. |
| `tmFunction` absent | The project default was used, named in the output. |
| `saltModel` absent | The Tm function's own assumption is reported as unknown, and each Tm value is labelled as computed under an unstated salt assumption. |
| `maxAnnealingOverlap` absent | No upper bound was applied; the output states the overlap actually chosen. (Until the field is accepted, it is always absent.) |
| `mutationPlacement` absent | The default for the style was used, and named. |

**Rule (new):** the layer does not ask the caller to supply a missing input
mid-design. It designs under what it was given and reports the gap. Asking and
then assuming is the same error in two places; reporting once is enough, and a
caller that wants to re-issue does so at the request layer.

---

## 11. What the contract cannot know

This is the honest boundary, so that no one mistakes a reference behaviour for
a requirement.

| Item | Status |
| --- | --- |
| Tm is a function of primer text under a stated salt/buffer model | knowable — this is the shape of the contract |
| The project's default Tm function | **not yet defined** — deferred to a later milestone |
| Whether the project's Tm function will match the reference tool's | **unknown** — see §13.3 |
| A Q5 Tm range that is "acceptable" for `luria` | **not established** — no source in this workspace states one |
| A `Ta` the layer should emit | **not a contract output** — see §13.1 item 8 and §15 question 7 |
| Whether `minAnnealingOverlap` has a minimum permitted value | **not established** — see §15 question 4 |
| Whether a maximum primer length exists for `luria` | **not established** — see §15 question 1 |
| The meaning of the existing fixture's `minimum_overlap_bp` | **not established** — the term appears in no captured NEBaseChanger artifact, no code parses it, and the saved help text contains no occurrence of `overlap` at all (§20) |
| Whether the reference tool's short-substitution Tm can be reproduced by `luria` today | **not yet** — `buildPairMap` has not been read (§17, §20) |

**Rule (new):** nothing in this table may be filled in by an assumption. No
Tm range, no `Ta` formula, and no salt model may be added to the contract on
the basis of what a reference tool printed. Each requires its own source, and
the source must establish scope (§14).

---

## 12. Determinism and reproducibility

A design is reproducible when:

1. the inputs are recorded completely — the design request, the
   `MutationResult`, and the digest of the template;
2. the output names the constraint set it was evaluated against, absent fields
   included;
3. the Tm function is named, not merely applied;
4. the search order is stated;
5. no input is read from anywhere other than the request.

**Rule (new):** `builtAgainst` is a content digest of the template, not its
name. A construct name identifies a construct; it does not identify the
sequence that was in front of the layer. The digest does (D007).

---

## 13. Reference behaviour: what the saved material actually shows

The workspace contains saved NEBaseChanger material under
`tests/fixtures/nebasechanger-research/` and `target/research/`. The
statements below are about that material. They are **observations with a
stated source**, and an observation of a reference tool is not a biological
requirement for `luria` until its scope and meaning are established — which is
what D008 and D011 already insist on.

### 13.1 Documented in the saved help text

The saved help text (`target/research/nbc_help_text.txt`) documents the
reference tool's design rules. Quoted, in the order they appear:

1. **The pair is back-to-back.** *"NEBaseChanger will design the custom
   mutagenic back-to-back primers for use in the Q5 Site-Directed Mutagenesis
   Kit."*
2. **Deletions.** *"For deletions, the 5′ ends of the primers align with the
   edges of the deletion."* And: *"due to the use of back-to-back primers,
   there is no limit to the size of the deletion that can be introduced using
   this kit."*
3. **Insertions.** *"the inserted sequence is attached to the 5′ end of the
   primers as non-annealing tails. If 6 or fewer nucleotides are to be
   inserted, they will be placed at the 5′ end of one primer. If more than 6
   nucleotides are to be inserted, they will be split between the two primers
   to keep the primer lengths and Tm's similar."*
4. **Substitutions.** *"If the target region encompasses 6 or fewer
   nucleotides, the replacement sequence will be placed in the forward primer
   with 10 or more bases of annealing sequence flanking the inserted bases."*
   For a longer target region: *"the substitution sequence will be split
   between the two primers … The 5′ ends of the annealing portions of the
   primers align with the edges of the target region."*
5. **The two parameters.** *"There are two optional parameters that can be
   used to influence primer length: minimum primer length and minimum primer
   Tm with default values of 15nt and 55 °C, respectively. Increasing the value
   of either parameter will generally result in longer primers with higher
   specificity. … When modifications are constrained to the 5′ tails, the
   minimum primer length only applies to the annealing portion of the primer."*
6. **How length is determined.** *"NEBaseChanger uses primer melting
   temperature (Tm) when determining how long a primer should be. … extended
   in both the 5′ and 3′ directions so that there are at least 10 nucleotides
   that are 100% complementary on both sides of the mutation region and until a
   desired minimum Tm is achieved."*
7. **The Tm model, and its scope.** *"Tm's are calculated using nearest
   neighbor thermodynamics data for matched and internal mismatched bases and
   corrected for buffer composition. Tms are calculated assuming the
   recommended buffer composition for Hot Start High-Fidelity DNA Polymerase,
   and the preferred primer concentration (500nM)."* The help text cites the
   SantaLucia 1998 unified nearest-neighbour paper, the Allawi/SantaLucia
   internal-mismatch papers, and Owczarzy's sodium-ion paper. The saved code
   applies method 4 at salt 150 mM and cycle count 0.5.
8. **`Ta` is a suggested value, and its meaning is not given.** *"An
   annealing temperature (Ta) for the PCR is calculated in addition to the Tm,
   optimized for Q5 DNA polymerase."* The saved code computes the construct
   value as the lower of the two rounded Tm values plus one, capped at 72. A
   formula is not a meaning: nothing in the saved material states what makes a
   `Ta` correct.
9. **Stated capability limits of the kit.** *"multiple substitutions within a
   100 nucleotide region or insertions up to 100 nucleotides long."*

### 13.2 The coordinate convention, and why the existing fixture warns about it

The saved help text documents the batch grammar:

```
"ID_1 234 239 ACGT" --> substitutes ACGT between bases 234 and 239.
"ID_2 234 235 ACGT" --> inserts ACGT between bases 234 and 235.
"ID_3 234 245 ACGT" --> deletes sequence (235-244) between bases 234 and 245.
```

The two named numbers are **outer anchors**: the edited interval is strictly
between them, exclusive at both ends. `234 245` deletes `235–244`; `234 235`
inserts between consecutive anchors.

This is why the SID-1 fixture's description reads `Replace gat between bases
1209-1213` while the edit is 1-based 1210–1212: 1209 and 1213 are the
neighbours of the edited region, not its bounds. The existing fixture README
states the same warning; the help text is now the source that explains it.

**Rule (new):** the contract's own request form does not use anchor pairs. It
carries a zero-based half-open interval (D004) and renders 1-based inclusive in
headers (D015). If a caller supplies anchors, the conversion is explicit and
the anchor convention is named in the request. A layer that guesses which
convention a pair of numbers is in has invented a rule.

### 13.3 Where the saved material does not settle the project's choice

The saved material contains a Tm routine, and
`tests/fixtures/q5-primer/reference/neb-q5-sdm-reference.py` reproduces the
fixture's reported Tm values from the template alone. That reproduction is a
fact about the saved material. It is deliberately **not** treated as a fact
about `luria`:

- D008 and D011 already state that NEBaseChanger behaviour is not the source of
  truth for luria's algorithms.
- The reference model's scope is documented (§13.1 item 7), but the project has
  not decided to adopt that scope. Adopting it would be a biological decision,
  and the working method requires such decisions to be made explicitly.
- Reproducing an external routine is not the same as establishing that the
  quantity it computes is the quantity the project wants.

**Rule (new):** the Tm function for `luria` is chosen in a later milestone, on
its own merits, with its salt assumption written down. Until then, every Tm
value in a fixture is labelled a reference value, and no fixture asserts that
`luria` produces it.

---

## 14. How behaviour is testable before an algorithm is chosen

This is the point of the milestone. The tests below constrain a contract, not
an implementation strategy. Each is answerable without knowing how a design is
searched.

### 14.1 Invariants that hold of every design

| Test | Asserts |
| --- | --- |
| I-1 | `reverse.annealing == revcomp(template[reverse.annealingSpan])` |
| I-2 | `forward.sequence == editedSequence[forward.editedSpan]` |
| I-3 | every primer's `overlap ≥ minAnnealingOverlap` |
| I-4 | every pair covers the changed region of its edit |
| I-5 | `overlap` equals the count of positions where primer text equals template text — the count is not asserted to equal the bound, nor the span width |
| I-6 | `builtAgainst` equals the digest of the sequence actually supplied |
| I-7 | both primers state an orientation, and the two are opposite |
| I-8 | the reported relationship is derivable from the two spans, not stored independently |
| I-9 | a failed design emits no pair (D010) |
| I-10 | an absent constraint appears as absent in the output, never as a substituted default |
| I-11 | the same inputs produce the same output across runs |
| I-12 | a `MutationResult` whose `sequenceAfter` is inconsistent with its edits is rejected, naming the upstream layer |
| I-13 | `tail`, `annealing` and the mutated positions partition `sequence`, and neither a tail position nor a mutated position is counted in `overlap` (H8) |

### 14.2 What the fixtures pin

The fixture set in `tests/fixtures/q5-primer/` supplies inputs and invariants.
It does **not** supply expected primer text for `luria`, because `luria` has no
design algorithm yet. Where a reference design exists it is supplied, marked as
a reference value, and reproducible from a script in the fixture set.

**A reference design is not claimed to satisfy H1–H8.** The reference tool's
inline `replace` path applies its minimum-length parameter to one primer, so
the `q5p-02` and `q5p-03` reference designs leave the forward primer below the
supplied bound. Those designs are kept, and the violation is recorded
explicitly in each record's `h1` block and in `cases.tsv`'s `reference_check`
column. A reference fixture documents incompatibility with the project's hard
requirements; it must not be read as satisfying them.

A fixture that asserted expected primer text before an algorithm exists would
be asserting a property of a future choice, which is exactly what D008 says not
to do.

**What the reference designs do supply** is a decomposition: each record names
the annealing portion, the 5′ tail, the changed positions, the annealing
overlap, and the whole-primer match count, and the script asserts each against
the primer text and the template. That is a description of what the reference
produced, not a claim that it satisfies H1–H8; `q5p-02` and `q5p-03` fail H1
and are recorded as failing.

### 14.3 What the fixtures must cover

| Case | Why it is in the set |
| --- | --- |
| substitution, overlap above the bound | the bound-versus-target distinction (§9) |
| substitution, one primer's annealing overlap exactly at the bound | `>=` versus `>` for that primer only; the pair still fails H1, so the case is not a pair-level witness (§9) |
| substitution sharing a base with the original | the position-by-position overlap definition (§5), and the difference between the whole-primer match count (21) and the annealing overlap (20) that H8 and I-13 require |
| insertion | the degenerate changed region (§5) |
| deletion | the context-preservation case (§5) |
| separate variants | two independent pairs against one template (§4) |
| reverse-strand edit | orientation is a property of the construct, not the edit (§6) |
| unsatisfiable bound | the failure contract (§10) |
| absent Tm constraint | the unresolved-input contract (§10) |

Each case is derived from an artifact another fixture already validated, so the
primer contract does not carry its own unverified biology.

---

## 15. Open questions for the next milestone

Listed so that no one answers one by accident.

1. Is there a maximum primer length for `luria`, and is it a bound or a
   preference? The reference tool documents a minimum length and a minimum Tm,
   and no maximum.
2. Is Tm a hard constraint or a preference when `minTm` is absent?
3. Does a Q5 pair have to abut, or only cover the edit? The reference tool
   documents back-to-back primers and edges aligned to the region (§13.1
   items 1–4); one fixture case does not establish a rule for `luria`.
4. What is the relationship between the existing fixture's `minimum_overlap_bp`
   and the reference tool's *minimum primer length*? The help text says the
   latter applies only to the annealing portion when mutations are confined to
   the tail (§13.1 item 5), so the two names may denote different quantities.
   This must be settled before the field is implemented, not inferred.
5. How is a composed separate-variant request represented at the design layer —
   one `PrimerDesign` per variant, or one design carrying several pairs?
6. What does the layer do when two variants' primer spans overlap on the
   template? The reference tool documents *"multiple substitutions within a 100
   nucleotide region"* (§13.1 item 9), which is a capability statement, not an
   overlap rule.
7. Does the layer emit a `Ta` at all? The reference tool prints one and
   documents a formula (§13.1 item 8) without stating what makes it correct.
   Nothing in this workspace establishes the meaning.
8. Does `luria` support the caller's `confineToTail` placement at all, or only
   inline? The reference tool documents both (§13.1 items 4, 5).

Questions 4, 6 and 7 are the ones where the current material is insufficient,
not merely undecided. They should be settled by a source, or by the caller
declining to need them.

**Amendment (2026-10-12, policy pass).** §16–§20 below were added by the pass
that separated Luria's own design policy from a versioned NEBaseChanger
compatibility target. That pass re-reads §15 against the saved source; the
review is §20, and where it changes a question's status the change is recorded
there rather than by editing the question above.

---

## 16. Design policy: Luria-native and NEBaseChanger-compatible

A design is produced under a **policy**, and the policy is part of the output.
The two policies are not variants of each other; they are different rules with
different evidence behind them.

| Policy | Meaning | Evidence |
| --- | --- | --- |
| `luriaNative` | Luria's own documented candidate-generation and selection rules. Every rule is a stated Luria choice, recorded as **new** in §1 until a decision covers it. | none external — this document |
| `nebBaseChangerCompatible` | Behaviour substantiated from one named build of the reference tool. The target is versioned; the initial target is `NebaseChangerV2_11_0`, the live capture `4330b0c7` (`tests/fixtures/nebasechanger-research/manifest.json`). | the saved bundles and the scripts in `tests/fixtures/nebasechanger-research/` (§17) |

**Rule (new):** `designPolicy` is required. Neither policy has a silent default.
A request that does not name a policy has not chosen one, and D013/D022 make an
un-chosen decision an error rather than a fallback.

**Rule (new):** the compatibility target is a version identifier, not a name for
"the website". A design produced under `NebaseChangerV2_11_0` says which build
it was substantiated against, so a later build does not silently redefine what
the policy promised (§17 reproduction).

### 16.1 A mode is not a preset

These are three different things, and only the first two are a policy:

| Thing | What it changes | What it does **not** claim |
| --- | --- | --- |
| parameter preset | the numeric values of the design request (`minAnnealingOverlap`, `minTm`, `maxAnnealingOverlap`) | that any candidate-generation or selection rule matches the reference tool |
| `luriaNative` policy | Luria's candidate geometry, enumeration order, bounds, failure semantics | that it matches any reference build |
| `nebBaseChangerCompatible` mode | the reference build's candidate geometry, growth order, selection rules, availability limits and failure reporting, **as substantiated** | completeness beyond what §19 lists |

**Rule (new):** matching numerical defaults alone is never described as
compatibility. The reference tool's documented defaults — minimum primer length
15 nt and minimum primer Tm 55 °C (§13.1 item 5) — are the same numbers a
preset would set, and setting them does not reproduce the reference tool's
growth order, its terminal-GC rule, its balance loops, its revert-to-last-good
step, its availability limits, or its Tm engine (§17). Worse, the same
quantity has three different defaults in the live build depending on the call
site (live-build doc §15.3): `primerMinLen` is 20 at the `Primer` layer and
15 at the `Fragment.autogen*` layer, and `monosalt`/`ct` are 50/200 in the
`Primer` default object literal against 150/500 in the fallbacks. A preset
names numbers; it does not name a site, and it does not name a rule. A preset
is a preset.

**Rule (new):** the word *compatible* may appear in output, documentation, and
UI only for the mode, and only qualified by the substantiation record (§19).

### 16.2 Interface

Selection and configuration stay separate:

- **policy selection** — one required field, `designPolicy`, plus
  `compatibilityTarget` when and only when the policy is
  `nebBaseChangerCompatible`. A target supplied with `luriaNative` is an
  error (D013).
- **parameter configuration** — the existing request fields (§2.2). A preset
  fills those fields; it never appears in the policy field and never changes
  which rules run.
- **output labelling** — `policy`, `policyTarget` and `substantiation` (§3).
  A consumer that reads only the primer text still cannot mistake a
  compatibility design for a Luria design, because the labels are part of the
  record, not of a wrapper.

**Rule (new):** no public API changes in this pass. `PrimerDesign` is not
implemented — `crates/core/src/primer.rs` delivers `PrimerPair`, `construct_primer`,
`assemble_pair` and `require_design_constraints`, and §8 of
`docs/tasks/contracts/milestone-3b-q5-primer-contract.md` records that the output record
itself is unimplemented — so the policy fields are contract additions, not API
additions.

---

## 17. Reference evidence: the live build's two design paths

The live build (`4330b0c7`, `NebaseChangerV2_11_0`) reaches primer design by
two paths. They are documented separately because their parameters, their
growth rules and their Tm engines differ. Nothing from one is applied to the
other; §17.3 records what remains unverified.

Routing, verified in the live beautified bundle at line 17052 of
`live-js/beautified/index.4330b0c7.v2.11.0-live.beauty.js`:

```js
else if (I == "insert" || (I == "replace" && (Me || Ye.length >= 6 || X.length >= 6
                                   || !/^[acgt]*$/i.test(Ye))))
```

`Me` is the confine-to-5′-tails flag, `Ye` the expanded replacement, `X` the
wild-type target. An insert, or a replace with either side 6 bases or longer,
or with confine-to-5′-tails on, or with a replacement that is not ACGT-only,
takes the fragment path. The short-substitution path is reached from exactly
one call site (line 17108), and the parameter binding was settled from the
call site at line 16908.

### 17.1 Short inline substitution (`Ne.calcSDMPrimers`, 8 parameters)

Extracted by `q5_extract.py live` and read directly at lines 17194–17330.

| Quantity | Live value | Confidence |
| --- | --- | --- |
| minimum primer length | `X < 15 ? 15 : X` — floor 15, no ceiling | verified in source |
| minimum Tm | clamped to `[55, 72]` | verified in source |
| maximum Tm | **absent**. `primerMaxTm` and `maxTm` occur zero times in the function body | verified in source; cycles doc §1.5 |
| terminal-GC target | hardcoded literal `63` (line 17205) | verified in source |
| alignment pad | `10` (dispatcher, line 16969) | verified in source |
| alignment short-circuit | equal-length old/new, length < 10, both ACGT-only → identity alignment; otherwise `alignDNAglobal` with the **inline** scoring `mat` ±2 diagonal, `a/g = g/a = +1`, other off-diagonals −1, `gap = -2.5`, `term = -2.5`, `extend = -1` | verified in source; live-build doc §12.1 item 3 (new in live, absent in v2.8.4) |
| availability limits | 7th/8th arguments, default `Infinity`, tightened to `template.length − (pad + oldLen)` and `template.length`; finite only on the linear branch (line 17103–17105) | verified in source |
| failure reporting | `throw` with named messages (`cannot reach min primer length … only N bases available 3′/5′ of the mutation`, `cannot reach min Tm: only N bases available …`), converted to per-op warnings by the dispatcher | verified in source; live-build doc §12.1 item 4, §17.8 item 2, §17.8 item 6 |
| growth order | forward 3′ extension from `pad` while `tmf < minTm`; then extend while `tmf < 63` and the primer does not end in G/C; revert to the last good length; reverse primer grown the same way from `minLen`; then balance the two primers while their Tm differs by more than 5, with the same terminal-GC rule and revert | verified in source |
| `Ta` | `min(min(tmf, tmr) + 1, 72)`, computed by the dispatcher (lines 16971–16974, applied at 17109–17110) | verified in source |
| Tm engine | `Ne.tm(primer, revcomp(compseq))` → `createTmCalc({ct: 0.5, salt: 150})`, `polGroup` a local literal `"Q5"` so `method = 4` always; initiation `ΔS/ΔH = −5.7/200` plus `+6.9/+2200` per terminal A/T | verified in source; cycles doc §1.2, §1.3, §2 |
| concentration, salt | `ct = 0.5` µM = 500 nM; `salt = 150` mM | verified in source; agrees with the help text (§13.1 item 7) |

### 17.2 Fragment and NEBuilder design (`Primer.calculatePrimers`, reached via `Fragment.autogenPrimers`)

Extracted by `q5_extract.py live`; option sets reproduced by
`q6_v284_vs_live.py` and `q1q3_four_builds.py`.

| Quantity | Live value | Confidence |
| --- | --- | --- |
| `Fragment.autogenPrimers` defaults | `polGroup Q5`, `primerMethod 4`, `primerMonosalt 150`, `primerCt 500`, `primerMinLen 15`, `primerMinTm 55`, `primerMaxTm 70`, `direction both` | reproduced by `q6_v284_vs_live.py` |
| dispatcher options object | `{primerMinLen, primerMinTm, primerMaxTm: 72}` (line 16982) — so the effective fragment-path maximum Tm from the dispatcher is **72**, not the 70 default | verified in source |
| `calculatePrimers` fallbacks | `primerMinLen || 20` read **twice**, once for each primer; `primerMinTm || 55`; `primerMaxTm || 70` | verified in source |
| guards | `minLen > template.length` → `Frag (N bp) is shorter than min primer length`; `minLen(fwd) + minLen(rev) > template.length` → `Autogen primers will overlap.`, re-checked after growth | verified in source |
| growth order | forward grown 3′ while `tmf < minTm`; then extended while `tmf < primerMaxTm` and not G/C-terminated, with revert to last good; reverse the same; then balance while the two Tm differ by more than 5 | verified in source |
| Tm engine | `createTmFunction(primerMethod, primerMonosalt, primerCt)` → `TmCalc.calcTm_nomm`; SantaLucia terminal corrections `+4.1/+2300` per terminal A/T and `−2.8/+100` per terminal G/C; `primerCt` interpreted as nM | verified in source; cycles doc §1.4, §2 |
| NEBuilder assembly layer | duplicate-site drop, `NB_MIN_FRAGMENT = 60`, `clash`, `wraparound`, `ta = min(min(tmf,tmr)+1, 72)` | live-build doc §17.6 |

**The two engines are not the same engine.** Both run at 500 nM and 150 mM and
both select method 4, but the initiation conventions differ (`−5.7/200` versus
`+4.1/+2300` and `−2.8/+100`), so the same primer text can score differently on
the two paths (cycles doc §1.4, `[LIVE]`). A compatibility mode that claims one
engine for both paths is wrong.

### 17.3 What remains unverified or ambiguous

| Item | Status |
| --- | --- |
| `buildPairMap` (the mismatch/pairmap builder inside `createTmCalc`) | **read in this pass; the row above was out of date.** In the live bundle the closure's builder is the minified local `K = function () {…}`, defined at byte offset 127266, called as `this.buildPairMap()` at 130164 inside the `pairmap === ""` branch, and exposed in the returned object literal as `buildPairMap: K` at 134894. It builds a mask of the primer against the reverse-complemented compseq — `\|` where the base is the complement, `^` where either side is `-`, `x` otherwise — then collapses runs: `x` beside `x` or `_` → `_`; `^+_+` and `_+\^+` → `_`; `^+x` and `x\^+` → `_`; `_+[x^]` and `[x^]_+` → `_`. What remains **unreproduced** is not the rule but the value: no `luria` Tm port exists and no fixture compares one, so a compatibility mode can still state the engine's identity and parameters, not emit its numbers. The class-level `TmCalc.prototype.buildPairMap` is a **different** function: present in v2.8.4 (offset 69194, called at 73484) and **absent** in live, whose only occurrences are the call and the property above. The two must not be conflated. |
| Reachability of the changed `nndhds` keys (`x\|`, `x-`, `\|x`) and `bulge[25]` | `[DERIVED]` unreachable in the default configuration; `[UNKNOWN]` whether any user path places a literal `x` or `\|` into a scored sequence (cycles doc §6.3, live-build doc §16.3) |
| `Primer` constructor default `primerCt: 200` versus `_modelPrimer.primerCt || 500` | both present in the live build (lines 2387 and 2405); reachability of the 200 case not established (cycles doc §2, live-build doc §12 open question 4) |
| the minimum-annealing-tail change recorded as 10 → 11 from v2.8.4 onward (cycles doc §6.1; live-build doc §16.1) | **refuted as stated, and the claim is dropped.** The pad literal is 10 in **both** builds — live `St = 10` (beautified live, byte offsets 500481 and 510402) and v2.8.4 `ot=10` (min bundle 313404, beautified v2.8.4 500985) — and the substring `11` does not occur in either `calcSDMPrimers` body (live body read in full; v2.8.4 body checked programmatically). The stored `q5_verify.py` no longer prints the `pad/minlen/minTm/tail` line that cycles doc §6.1 quotes, so that row cannot be re-derived from the current script. Recorded as an artifact of an older script version, not as a build difference. |
| the identity-alignment short-circuit | whether `alignDNAglobal` returns the identity alignment in exactly the short-circuit's guard condition is **not proven**; the short-circuit is therefore not known to be a pure optimisation |
| the linear-sequence rotation and availability limits (lines 17103–17105) | read, but the coordinate semantics of the rotated source and of `gt`/`hn` are **not settled**; live-build doc §17.8 item 3 records a one-base shift and modulo wrapping without resolving intent |
| `verifyPair` (live-build doc §17.2) | whether it is a specification requirement for `luria` or an implementation safety net. **Read in this pass**: defined as `vt.verifyPair` at byte offset 162031 (line
4807), and its **only** consumer is at 484084 (line 16534), which pushes `"internal check failed for " + re.idprefix + ": " + se + " - primers not shown. Please report this to NEB."` — the wording is a bug report, not a biological rule. It has **seven** distinct failure outcomes, not four: `missing primer pair`; `could not reconstruct the mutant sequence (…)`; `reverse primer is not a valid IUPAC sequence`; `empty primer pair`; `forward/reverse primers do not match the mutant sequence at the edit site`; and the forward and reverse `… flap + annealing region does not reconstruct the full primer`, plus `vt.verifyOp`. Its central test is that `revcomp(seqr) + seqf` occurs in the mutant sequence spanning the edit window — for the native geometry that follows from the abutment and H3, so it needs no new requirement (see §20 question 9). |
| the fragment path end to end | the parameter set above is substantiated; the coordinate mapping through `setFwdPrimer`/`setRevPrimer`, `BuilderPrimer` composition and the NEBuilder assembly layer is **not** verified, and no fixture in this workspace exercises fragment-path output |

### 17.4 Reproduction

```
cd tests/fixtures/nebasechanger-research
python3 q5_extract.py live        # both paths, extracted from the live bundle
python3 q6_v284_vs_live.py        # option defaults and the three default sets
python3 q1q3_four_builds.py       # engine selection and maxTm plumbing, four builds
python3 q7_nndhds_keys.py         # createTmCalc option literal and nndhds keys
python3 q10_tm_tables.py          # numeric Tm-table deltas, entry by entry
python3 tm_tables_diff.py         # table copies, both engines
python3 q4b_sdm_tables.py         # createTmCalc's own table copies
python3 q1b_createTmCalc.py live  # the closure Tm: cases 1, 3, 4, 7
```

All read the fixtures only; they write nothing outside `/tmp/neb_*.js`. No
network is used.

---

## 18. The Luria-native policy

**Amended by D030 (2026-10-14).** Primer lengths are now set by melting
temperature, and §18.1's centred split, §18.3's enumeration and §18.4's
statement that Tm is not enabled are superseded for lengths: the forward
primer keeps a fixed 10-base 5′ flank, both primers grow at their 3′ ends to a
55 °C floor, prefer a G/C last base below 63 °C, and are balanced to within
5 °C, and the annealing temperature is the lower Tm, rounded to a whole
degree, + 1 °C, capped at 72 °C (D030 as amended 2026-10-18).
`docs/decisions/` D030 is the rule and its sources; the text below is kept as
the record of D029 and still governs scope, hard constraints and the failure
outcomes, as amended in §18.5. The Tm function is `crates/core/src/tm.rs`
(SantaLucia & Hicks 2004, Owczarzy 2004), validated as
`tests/fixtures/tm/README.md` records.

Small, coherent, and deliberately narrower than the contract. Everything
outside the scope below is out of scope for the initial native policy, not
unsupported by it.

**Scope (amended 2026-10-14):** one edit whose replacement region is one
contiguous template interval of at most 3 bases — a nucleotide substitution
(`L = 1`) or a residue substitution's codon (`L = 3`), the two edits the
mutation layer produces — one pair, linear template, inline placement,
separate variants not composed. The earlier scope was the nucleotide
substitution alone; it excluded the most common SDM request, a residue
change.

### 18.1 Candidate geometry

For an edit at template span `[s, e)` with replacement length `L`, edited span
`[s, s + L)`:

- the pair **abuts** at a template boundary `s − u`, `u ≥ 0`;
- the forward primer covers template `[s − u, s + L + v)`; its text is
  `template[s − u, s] + replacement + template[s + L, s + L + v]`;
- the reverse primer covers template `[s − u − w, s − u)`; its text is
  `revcomp(template[s − u − w, s − u])`;
- the forward primer's annealing portion is the two context runs, total
  `a_f = u + v`; the reverse primer's annealing portion is its whole span,
  `a_r = w`;
- `u` and `v` count the **unchanged template-derived bases** of the forward
  primer on the 5′ and the 3′ side of the replacement region. They are flanks,
  not primer lengths: the forward primer's length is `u + L + v = a_f + L`, and
  its annealing total is `a_f = u + v`, which excludes the replacement itself.
  The replacement region is the claimed interval `[s, e)`; a claimed position
  whose letter did not change is still part of it (§5, H8).

**Split rule (amended 2026-10-13, split-policy pass).** The split is
**mutation-centered**. For a candidate annealing total `a_f`:

```
u = floor(a_f / 2)     v = ceil(a_f / 2)      |u − v| ≤ 1
a_f ≥ max(minAnnealingOverlap, 2 · MIN_FLANK)  MIN_FLANK = 10
```

so the replacement region sits in the middle of the forward primer's unchanged
annealing flanks, each flank has at least 10 bases, and the extra base, when
`a_f` is odd, goes to the **3′** side.

**Amended 2026-10-14 (owner decisions, D029).** The extra base moved from the
5′ to the 3′ side: the 3′ flank primes extension, and it is the side the
reference tool grows. The per-side minimum is new: without it, a bound of 15
gave a 3′ flank of 7 bases, below the reference manual's "at least 10
nucleotides that are 100% complementary on both sides". The minimum adopts that
number as a length floor, not the reference algorithm. H1 is unchanged; it
remains a bound on the annealing total, and the per-side minimum is a rule of
`luriaNative` only.

`(u, v)` is a function of `a_f` **alone**. Availability never moves it:
availability is a feasibility test applied to the centered candidate,

```
u ≤ s − a_r            v ≤ template.length − (s + L)
```

on a linear template (D005/D019). A centered candidate that fails either test
is **not shifted** to one that fits; the search reports no design under this
policy and names the side that bound it (§18.5).

The initial native scope admits `L = e − s` from 1 to 3. For any **contiguous**
replacement region the definition is the same: the region's `L` bases
sit between the two flanks, so centering the region in the primer and equalising
the flanks are one statement. A changed set that is not one contiguous interval
— a residue edit on a CDS that wraps the origin of a circular reference — has
no centered definition here, and is out of the initial native scope rather than
centered by generalisation.

**Superseded (2026-10-12, review pass).** That pass replaced the fixed split
`u = ceil(a_f/2)`, `v = floor(a_f/2)` with an availability-aware one,
`v = min(floor(a_f / 2), hi3)`, `u = a_f − v`, so that a template whose 3′ flank
is exhausted would put the whole annealing budget on the 5′ side. The
availability-aware split is **no longer the native rule**; it is kept as the
record of that pass, and §18.3, §18.4 and §18.5 below now read against the
centered rule. What that pass established, and this pass keeps, is the finding
rather than the fix: the arithmetic it replaced is the centered rule, and a
search that reports no design where a shifted split would fit is now a **named
outcome** of the centered policy (§18.5), not a defect to be hidden by moving
the split.

A candidate is therefore the pair `(a_f, a_r)` of annealing totals, and its
flanks are fixed by `a_f`. Primer lengths are `a_f + L` and `a_r`.

### 18.2 Hard constraints

H1–H8 of §7, evaluated on the output, plus:

- **availability:** `s − u − w ≥ 0` and `s + L + v ≤ template.length` (linear,
  D005/D019). Primer design is for linear templates only (D035): a circular
  template is refused with `UnsupportedPrimerDesignScope`.
- **alphabet:** H6, the project alphabet, no degenerate symbol in a primer.
  An ambiguity code in the **template** under the centered pair is reported as
  *no centered design fits* (§18.5 outcome 2, obstacle `AmbiguousTemplateBase`):
  a shifted pair might avoid it, and this policy does not shift (owner,
  2026-10-14, for point-mutation mode).

**The centered split is not a hard constraint added here.** It fixes which
`(u, v)` a candidate has — the candidate set — and is stated in §18.1. The hard
feasibility tests are the two above. A centered candidate that fails
availability is not a candidate failing a new rule; it is a candidate set with
no member that fits, which §18.5 reports. Ranking (§18.3) and the closed form
(§18.4) choose among fitting candidates and never create one.

No other hard constraint is added. In particular there is **no** maximum
primer length (§15 question 1 remains open; the native policy states that the
bounds are the annealing bounds and availability, and nothing else), and no
Tm bound.

### 18.3 Deterministic enumeration and tie-break

Enumerate `a_f` from `max(minAnnealingOverlap, 20)` upward and `a_r` from
`max(minAnnealingOverlap, 15)` upward, each capped by availability. (Amended
2026-10-14: the starting `a_f` carries the per-side minimum, and
`maxAnnealingOverlap` is not part of the request; the owner left it out until a
Tm-based length policy exists, because it cannot change any outcome under the
centered split.) Order the candidate set by the total
order:

1. `a_f + a_r` ascending — P3, shorter primers preferred;
2. `a_f` ascending — the tie-break, stated here as part of the contract (§8).

**Amended (2026-10-13, split-policy pass).** The third key that §18.3 carried,
`u ≥ v`, is still removed, for a different reason: under the centered split
`u ≥ v` holds by construction for every candidate, so as a key it would order
nothing. With `a_f` fixed, `(u, v)` is a function of `a_f` alone, so the two
keys above determine a unique candidate and no third key is needed.

**Consequence of the centered split (2026-10-13).** Availability is monotone
along the order: `u = floor(a_f/2)` and `v = ceil(a_f/2)` are non-decreasing in
`a_f`, and the 5′ test `u ≤ s − a_r` only gets harder as `a_r` grows. So if the
first candidate in the order — `a_f = max(minAnnealingOverlap, 20)`,
`a_r = max(minAnnealingOverlap, 15)` — fails availability, every later candidate does
too. The enumeration is stated for the
general case, because a Tm rule enabled later can make a larger candidate the
first fitting one; under the initial native policy it has a closed form
(§18.4).

The objective is total primer length: primer lengths are `a_f + L` and `a_r`, so
minimising `a_f + a_r` **is** minimising total primer length, since `L` is fixed
by the edit. It is **not** a bound on the longer primer and not a Tm balance:
`(20, 10)` and `(15, 15)` tie at 30, and the second key picks `(15, 15)` because
it has the smaller `a_f`, not because the objective prefers an even pair.

The order is input-independent: it is a function of `(s, L, n,
minAnnealingOverlap)` alone. Sequence content never enters it, so I-11 holds by
construction. The first candidate in the order that satisfies the hard
constraints is the design.

### 18.4 Tm-based selection is **not enabled**

*Superseded by D030 (2026-10-14): Tm-based sizing is enabled. Kept as the
record of the D029-era policy.*

The initial native policy runs without a Tm function. P1 and P2 are inert; the
output says so (§10). This is a policy statement, not a deferral of a rule:

- the project's Tm function is not defined (§11), and inventing one would
  violate §11's rule that no Tm model may be added on the basis of a reference
  tool;
- with Tm inert, the policy has a closed form. With `b = minAnnealingOverlap`
  `A = max(b, 20)` and `R = max(b, 15)`: `a_r = R`, `a_f = A`, and
  `(u, v) = (floor(A/2), ceil(A/2))` from §18.1. A design exists if and only if
  that one candidate fits availability, i.e. if and only if

  ```
  s ≥ R + floor(A / 2)
  template.length − s − L ≥ ceil(A / 2)
  ```

  (amended 2026-10-14; checked against a full enumeration of the §18.3 order
  for templates of 20–74 bases, `L` of 1–3, and nine bounds, with no
  disagreement)

  and §18.5 reports no design when either fails. The enumeration in §18.3 is
  stated anyway, so that enabling Tm later extends the same order instead of
  replacing it.

**Rule (new, amended 2026-10-13 and 2026-10-14):** the split of `a_f` into
`(u, v)` is the mutation-centered split of §18.1, with at least 10 bases per
side and the extra base on the 3′ side — stated Luria choices, recorded in
D029, **approved 2026-10-14**. It is **not** the reference
tool's rule. The reference's code pins the forward primer's 5′ flank at its
pad of 10 and grows only the 3′ flank toward a Tm target (2026-10-12 review,
C8; `replay_dispatcher_geometry.py`). Its manual describes the result as
"at least 10 nucleotides that are 100% complementary on both sides of the
mutation region" (`docs/references/nbc_help_v284.txt` line 200, quoted at
`docs/research/nebasechanger-live-build-4330b0c7-2026-10-10.md` §15.4,
`@19261`). `luriaNative` adopts that number as a per-side length floor, and
nothing else: no Tm growth, no G/C clamp, no Tm balance.

The corpus cannot tell the two geometries apart. Every reference design has
forward flanks of exactly 10 and 10 (`q5p-01`: annealing spans
`[[1199, 1209], [1212, 1222]]` around `[1209, 1212)`), because the Tm target
was already met at the seed. Under the amended rule the native forward primer
covers the same span as the reference forward primer for the four designs at
bound 15 (`q5p-01`, `q5p-04`, `q5p-07` variants a and b); for `q5p-02` and
`q5p-03` the native `a_f` is 24 and 25, where the reference keeps 20 and fails
H1. Reverse primers differ throughout: the native reverse primer is
`max(bound, 15)`,
the reference's is grown for Tm. That is an observation about the corpus, not
a compatibility claim (D028).

### 18.5 Failure semantics

**Amended by D030 (2026-10-14).** Outcome 2 is renamed *no design fits*
(`NoPrimerDesignFits`) and gains a cause: a primer that runs out of template
before its Tm reaches 55 °C, reported with the side, the bases available and
the Tm reached. An unmet 5 °C balance or a missing G/C last base is not a
failure; the design records it.

A candidate set that yields nothing satisfying the hard constraints is a
failure, not a smaller design. Per D005 and §10: no pair, no partial pair.

**Amended (2026-10-12, review pass).** §18.5 described one failure. There are
three distinguishable ones, and collapsing them loses information the caller
needs:

**Amended again (2026-10-13, split-policy pass).** The three outcomes stay, but
their names and promises are restated against the centered split: outcome 1 is
renamed *no centered candidate fits* and is required to name the binding side;
outcome 2's promise that a design "may exist above them" is dropped, because
availability is monotone under a centered split (§18.3). The list below is the
current wording.

**Amended again (2026-10-14, owner decisions).** The outcomes are split by
cause, and the `maxAnnealingOverlap` outcome is dropped with the field:

1. **no design possible** — the template is shorter than the smallest pair,
   `L + max(b, 20) + max(b, 15) > template.length`, so no placement of any abutting pair
   fits. `BiologicalValidation`: the request is well formed and the template
   cannot support it. `q5p-09` (bound 31, 30-base template) is this outcome;
2. **no centered design fits** — a pair of that size fits the template, but not
   centered on this edit. The report names the side that bound it — the 5′ test
   `u ≤ s − a_r` or the 3′ test `v ≤ n − s − L` — with the bases available on
   that side and the flank the centered split asked for. `Unsupported`: another
   geometry might fit, and this policy does not shift;
3. **not searched** — the request was refused before enumeration.
   `minAnnealingOverlap < 1` is `MalformedInput`; a replacement region longer
   than 3, a non-contiguous changed set, a non-linear template, or a composed
   variant request is `Unsupported`.

A centered-geometry failure is outcome 2. It is never reported as an H1 bound
failure or as an unsupported-placement refusal: the bound was met by the
candidate that did not fit, and the placement was supported.

**Implementation note (2026-10-13).** The distinction is a contract requirement;
the error model does not yet carry it. `crates/core/src/error.rs` has no
variant for a design-level failure — the design layer is unimplemented — and
its primer-level variants sort as `AnnealingOverlapBelowBound` →
`BiologicalValidation`, `InvalidPrimerCandidate` → `MalformedInput`,
`UnsupportedPrimerPlacement` → `Unsupported`. **Implemented 2026-10-14:**
outcome 1 is `NoPrimerDesignPossible`, outcome 2 `NoPrimerDesignFits`, and
outcome 3 `InvalidPrimerDesignRequest` or `UnsupportedPrimerDesignScope`, with
the categories recorded above (`crates/core/src/primer_design.rs`).

Each failure names the constraint that bound the search and states the range
that was searched, as the pair of totals `(a_f, a_r)` actually enumerated.
`q5p-09` is outcome 1 (no design possible), and its fixture note already says
the failure "is not a property of one search"; the fixture and the semantics
agree. This is
Luria's own reporting rule; the reference tool's throw messages (§17.1) are not
reused as Luria's wording.

---

## 19. Compatibility scope and confidence

**The smallest supported compatibility scope** is one inline nucleotide
substitution on the live build's short-substitution path: ACGT-only
replacement and ACGT-only wild-type target, both shorter than 6 bases,
confine-to-5′-tails off, reached at line 17108. That scope is the
`NebaseChangerV2_11_0` target.

**Reproducible with high confidence** — read in the live bundle, reproduced by
the stored scripts, and expressible as a rule rather than as a number:

1. the parameter clamps: minimum length floor 15, minimum Tm clamped to
   `[55, 72]`, no maximum length, terminal-GC target 63;
2. the candidate geometry: a pad of 10, a forward primer that carries the
   replacement inline and grows 3′, a reverse primer that is the reverse
   complement of 5′ context and grows from the minimum length;
3. the growth and selection order: reach minimum Tm, then reach the terminal-GC
   target, then revert to the last good length, then balance the two primers
   while their Tm differs by more than 5;
4. the availability limits and the fact that a bounded search **throws** rather
   than returning a shorter design, with a message naming the side and the
   number of bases available;
5. the alignment rule: identity when old and new are equal-length, shorter than
   10, and ACGT-only; otherwise global alignment with the inline scoring above;
6. the engine identity: `createTmCalc`, method 4, 500 nM, 150 mM, initiation
   `−5.7/200` plus `+6.9/+2200` per terminal A/T.

**Not reproducible today, and marked unresolved rather than filled in:**

1. the Tm **value**. `buildPairMap` **has** been read (§17.3, amended), so the
   mismatch handling is available from the stored artifact. The value is still
   unreproduced for a different and honest reason: no `luria` Tm port exists and
   no fixture compares one. Reproducing it is an implementation task, not an
   evidence gap. A fixture that compares a `luria` Tm against a reference Tm is
   still not a compatibility check until that port exists.
2. the coordinate semantics of the linear-sequence rotation and its limits
   (§17.3). `replay_dispatcher_geometry.py` reproduces the primer **text**, the
   primer **lengths**, the **spans**, the abutment and the availability limits
   for all six reference designs, so the rotation is not what blocks span
   equivalence. What blocks it is that **the reference reports no spans at all**:
   `raw/sdmsummary.txt` carries `ID, Description, Fwd Primer Seq, Length, Tm Fwd,
   Rev Primer Seq, Length, Tm Rev, Ta` and no positions. Span equivalence with
   the reference tool is therefore not a testable claim today, and a target
   record must say so rather than imply that spans were compared.
3. whether `verifyPair` is part of the specification (§17.3).
4. the 10 → 11 minimum-annealing-tail claim (§17.3) — unreproduced.

**Explicitly not claimed:**

- **fragment-path compatibility.** §17.2 records its parameter set and growth
  order, which is evidence about the build, not a compatibility mode: the
  coordinate mapping and the assembly layer are unverified and no fixture
  exercises them. `NebaseChangerV2_11_0` does not offer fragment design, and
  requesting it is an error (D013), not a degraded approximation.
- **insert and delete compatibility.** The reference tool routes both to the
  fragment path (line 17052; live-build doc §17.8 item 1), so the one
  substantiated path does not apply, and the mutation layer cannot produce
  either edit: `NucleotideMutationIntent` carries one `replacement: char` at a
  one-nucleotide `position` interval, and its constructor rejects an interval
  that selects more than one nucleotide (`crates/core/src/mutation.rs`).
  Insertions, deletions and multi-base replacements therefore have no
  representation at the mutation layer, let alone at the design layer. Neither
  policy supports them; the contract's §5 cases stay contract-level.
- **all mutation types, degenerate replacements, merged codon ops, NEBuilder
  assembly, batch mode, legacy mode.** Out of scope for the target.

  **Owner note on degenerate replacements (2026-10-14).** If an
  indel/substitution mode like NEBaseChanger's is added, it should accept
  ambiguity codes in the replacement sequence and expand them combinatorially,
  one primer pair per concrete sequence. NEBaseChanger's help: "Ambiguous base
  codes can be entered in the replacement sequence (N,Y,S,W, etc) and will be
  expanded combinatorialy over multiple primers. For example, a replacement
  sequence containing NS will be expanded to 8 sequences containing: AC, CC,
  GC, TC, AG, CG, GG, and TG, generating 8 primer pairs." Each expanded primer
  is concrete, so H6 still holds of every primer emitted. Two things would have
  to change first: the mutation layer restricts replacements to A/C/G/T (core
  review, 2026-10-14), and one request would yield several designs rather than
  one. This concerns the replacement only; an ambiguity code in the template
  is still an obstacle (§18.2).

**Rule (new):** where the reference behaviour conflicts with a Luria hard
requirement, the incompatibility is recorded, not absorbed. The existing
`q5p-02` and `q5p-03` reference designs are the example: they leave the
forward primer's annealing overlap at 20 against bounds of 24 and 25, and H1
is recorded as violated (§14.2). A compatibility mode that reported those
designs as satisfying H1 would be reporting a different rule under the same
name.

---

## 20. §15 reviewed against the source evidence

| §15 question | Status after the policy pass | Basis |
| --- | --- | --- |
| 1 maximum primer length | **still a Luria choice**, now narrowed: the native policy states no maximum, and the reference build documents none (§13.1 item 5) and clamps only a minimum (§17.1) | source + policy |
| 2 Tm hard or preference when `minTm` absent | **still a Luria choice**; the native policy states preference-and-inert (§18.4) because no project Tm function exists | policy, pending decision approval |
| 3 must a Q5 pair abut | **still a Luria choice**; the native policy states abut (§18.1). The reference build's back-to-back rule is reference scope only (§13.1 items 1–4) | policy |
| 4 `minimum_overlap_bp` | **remains unresolved**, with stronger evidence: the name occurs in four fixture-authored `input/mutation_spec.json` files and in no captured NEBaseChanger artifact; no Rust or Python code reads it; the saved help text contains **zero** occurrences of `overlap`; and all 18 occurrences of `overlap` in the live bundle are assembly-tag and fragment-assembly vocabulary (`BuilderPrimer.overlap`, `maxOverlapLength`, `terminalOverlapLen`), never a primer-design parameter | grep of the schema, the code, the help text and the bundle |
| 5 composed separate-variant representation | **still a Luria choice**, out of the initial native scope (§18) | policy |
| 6 overlapping variant spans | **still unresolved, not merely undecided.** The live build's `mergeCodonOps` limits (span > 8·nOps, span > 60 bp) are codon-mode only, and `nebuilderFragments` drops duplicate sites — neither is a nucleotide-mode overlap rule (live-build doc §17.6, §17.7) | source |
| 7 does the layer emit a `Ta` | **formula now verified in both builds** (`min(min(tmf,tmr)+1, 72)`, live-build doc §17.5/§17.6, live lines 16971–16974); **meaning still unresolved**, so the layer still emits none | source |
| 8 `confineToTail` support | **settled as a reference behaviour, not adopted**: the live build routes confine-to-5′-tails to the fragment path (line 17052), so it changes the path, not the same path's parameters. The native policy does not offer it (§18) | source + policy |

**New questions this pass raises, in the same form:**

9. Is `verifyPair` a specification requirement or a safety net? It changes
observable behaviour, so it must be specified either way (live-build doc §17.10).
10. Is the identity-alignment short-circuit an optimisation or a rule? Its
guard is not proven equivalent to `alignDNAglobal`'s result (§17.3).
11. Does `luria` offer a fragment/NEBuilder compatibility target at all, and
would it need a second target version, or a second policy?
12. Which of the live build's four `BUILD_CLEAR_KEYS` fields correspond to
`luria`'s work state, and does that belong to the design layer or above it?
(live-build doc §18.2; recorded so it is not answered by accident.)

---

## 21. Review pass of §§16–20, 2026-10-12

This section is the index to the review pass that re-read §§16–20 against the
stored artifacts. The pass implements nothing: `PrimerDesign` is still not
implemented, no Rust changed, and no runtime behaviour was added.

**Report:** `docs/research/q5-primer-policy-review-2026-10-12.md`.

**What the pass changed in this document, and why:**

| Where | Change | Reason |
| --- | --- | --- |
| §17.3 `buildPairMap` row | **not read** → read, with the live offsets and the mask/collapse rule | the builder is present in the live bundle as the minified local `K`; the class-level `TmCalc.prototype.buildPairMap` is the one absent in live |
| §17.3 minimum-annealing-tail row | not reproducible → **refuted as stated, claim dropped** | the pad literal is 10 in both builds; `11` occurs in neither `calcSDMPrimers` body |
| §17.3 `verifyPair` row | "four checks" → seven outcomes, with the offsets and the sole consumer's wording | read in full; the consumer's wording is a bug report, not a rule |
| §18.1 split | fixed `ceil`/`floor` → availability-aware | the fixed split fails availability where a feasible split exists; it is a defect, not a preference |
| §18.3 ordering | three keys → two | `u ≥ v` is not a tie-break key once the split is availability-aware |
| §18.4 closed form and rule | restated for the availability-aware split | consistency with the amended §18.1 |
| §18.5 failure semantics | one outcome → three named outcomes | the three reasons are distinguishable and the caller needs the difference |
| §19 unresolved items 1 and 2 | reasons corrected | the Tm gap is an implementation task, not an evidence gap; span equivalence is untestable because the reference reports no spans |
| §18.5 citation | `D010` → `D005` | `docs/decisions/` D010 is "Initial CDS and translation scope is deliberately narrow"; the fail-closed rule is D005 |

**Superseded in part (2026-10-13, split-policy pass).** Three rows above —
§18.1 split, §18.3 ordering, §18.4 closed form and rule — describe the split as
it stood after this pass. The 2026-10-13 pass replaced the availability-aware
split with a mutation-centered split, so those rows are the record of the
earlier pass and not the current rule; the rows are kept rather than rewritten
so that the reasoning of both passes stays traceable. See §22 and
`docs/research/q5-primer-implementation-readiness-2026-10-13.md`.

**Decisions the pass recorded:** D028 (a design names a policy, and a
compatibility policy names a build) and D029 (the Luria-native candidate
geometry) in `docs/decisions/`.

**What the pass left open**, and the smallest experiment that would settle each,
is §20's table plus the report's Part E. Nothing in §§1–§15 was changed by the
pass.

---

## 22. Split-policy pass, 2026-10-13

This section indexes the pass that changed the native split rule. The pass
implements nothing: `PrimerDesign` is still not implemented, no Rust changed,
and no runtime behaviour was added.

**Report:** `docs/research/q5-primer-implementation-readiness-2026-10-13.md`.

**What the pass changed in this document, and why:**

| Where | Change | Reason |
| --- | --- | --- |
| §1 rule table | added the centered-split rule | the rule was unlisted, so a reader of §1 could not see it |
| §18.1 split | availability-aware → mutation-centered, `u = ceil(a_f/2)`, `v = floor(a_f/2)`, extra base 5′ | the centered geometry is the proposed native default; availability is a feasibility test on the centered candidate, not a rule that moves the split |
| §18.1 notation | `u`, `v` stated as unchanged template-derived flanks, and `a_f = u + v` stated as excluding the replacement | the earlier bullet conflated flanks with the annealing budget's placement |
| §18.1 multi-base statement | centering defined for a contiguous replacement region; a non-contiguous changed set left undefined and out of scope | the single-nucleotide equation must not be generalised without a definition |
| §18.2 | stated that the centered split fixes the candidate set, and that availability and alphabet remain the hard feasibility tests | keeps hard constraints and candidate-generation rules visibly separate |
| §18.3 ordering | third key still removed, reason restated; monotonicity consequence added | `u ≥ v` is now true by construction, so as a key it orders nothing; monotonicity is what makes outcome 2 of §18.5 honest |
| §18.4 closed form and rule | restated for the centered split, with the feasibility condition; the "arbitrary among ties" rule replaced | the split is no longer arbitrary; the reference's ≥ 10 per-side wording is kept as source evidence about its own path, not adopted |
| §18.5 failure semantics | outcome 1 renamed and required to name the binding side; outcome 2's "one may exist above them" clause dropped; the centered-geometry distinction stated; the error-model gap recorded | under a centered split a shifted candidate is not a design, so a bounded search cannot promise more than a capped one |
| §21 change table | marked three rows superseded | the rows describe the split as it stood after the 2026-10-12 pass |

**Decisions the pass amended:** D029 in `docs/decisions/`, in place. It stays
**proposed** and unapproved; the availability-aware split it previously stated
is recorded as superseded inside the entry.

**What the pass left open** is the readiness report's blocker list. Nothing in
§§1–§15 was changed by the pass, and no compatibility claim, no Tm rule and no
maximum primer length was touched.

---

## 23. Owner decisions on D029, 2026-10-14

| Where | Change |
| --- | --- |
| §1 rule table | scope and split rows restated |
| §18 scope | `L = 1` widened to any contiguous replacement region of at most 3 bases, so residue (codon) substitutions are in scope |
| §18.1 | per-side minimum of 10 unchanged bases (`a_f ≥ max(minAnnealingOverlap, 20)`); extra base of an odd split moved to the 3′ side |
| §18.3 | enumeration starts at `a_f = max(minAnnealingOverlap, 20)`; `maxAnnealingOverlap` removed from the request |
| §18.4 | closed form restated and checked against full enumeration; the reference comparison corrected: the reference pins its 5′ flank and grows its 3′ flank, and no saved design distinguishes that from centering |
| §18.5 | outcomes split by cause: *no design possible* (`BiologicalValidation`), *no centered design fits* (`Unsupported`), *not searched*; the `maxAnnealingOverlap` outcome dropped |
| `docs/decisions/` | D029 rewritten to match |
| §18.3, §18.4, §18.5 | reverse primer floor of 15 bases: `a_r ≥ max(minAnnealingOverlap, 15)` (owner, 2026-10-14) |
| D029 status | **approved** by the owner, 2026-10-14 |
| §18.2 | an ambiguity code in the template under the centered pair is *no centered design fits* (point-mutation mode) |
| §19 | future indel/substitution mode: expand ambiguity codes in the replacement combinatorially, one pair per concrete sequence |
| §18, §18.4, §18.5 | **D030**: Tm-driven lengths (55 °C floor, G/C preference below 63 °C, 5 °C balance, 3′-only growth with a fixed 10-base forward 5′ flank, Ta = rounded lower Tm + 1 capped at 72, amended 2026-10-18); outcome 2 renamed *no design fits* with a Tm-floor cause |
| §2.2 | under `luriaNative` the buffer (salt and primer concentration) is a required request field rather than an optional `saltModel`: the reference tool uses two different conditions at two call sites, so a default would be an unmade decision. `tmFunction` is fixed by the policy and `minTm` is not accepted (D030's floor is fixed) |
