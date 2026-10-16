# Post-Milestone-4 code review: findings

Status: Phase 1 (read-only review) complete, 2026-10-16. No code changed.
Branch: `review/post-m4`. Planner: Claude.

Baseline at review start: `cargo fmt --check` clean, 462 tests pass,
`cargo clippy -D warnings` clean.

Method: the planner read the public surface, `q5.rs`, `mutation.rs` and the
docs. Three read-only reviewers covered (a) primer/design/Tm, (b) the
mutation and Q5 layers, (c) tests and fixtures. Scratch tests were run in
copies under `/tmp`, never in the repo. Items marked **confirmed** were
reproduced by a scratch test or verified by direct reading. Items marked
**suspected** are readings that need a second look.

Phase 2: the owner picks findings. The planner turns them into small worker
contracts. Correctness fixes are allowed to change behavior; refactors are
not.

---

## A. Owner decisions needed

These block or shape the fixes below. None should be chosen by the planner or
the worker.

**Owner answers, 2026-10-16:**

- **A1:** the CLI and machine-readable output become Milestone 5, and natural
  language becomes Milestone 6. Recorded as D034, and `product-brief.md` is
  updated.
- **A2:** NEBaseChanger designs on linear templates only, so circular
  templates are ignored for now. Recorded as D035; §18.2 is amended to say
  every template is designed as linear. No code change.
- **A3:** the existing warning past 64 designs (three fully ambiguous
  positions) stays, and there is no cap. The overflow is still a bug: the
  members are built before they are counted, so 32×`N` overflows before the
  warning can fire. It moves to §B as **B15**: count with checked arithmetic
  and return a structured error only when the count can't be represented. No
  policy change.
- **A4–A7:** deferred until the other items are done.
- **A8:** granted for this pass. The doc fixes in §D are done (see §D).

| # | Question | Context |
| --- | --- | --- |
| A1 | **What is the next milestone, and should the reordering be recorded?** | `product-brief.md` defines Milestone 4 as "Machine-readable and CLI workflow" and Milestone 5 as natural language. It says milestones are reordered "only through an explicit scope decision". The repo's Milestone 4 became indels, and no reorder decision is recorded. No CLI or binary exists. `README.md` "Status" still describes Milestone 1. |
| A2 | **Circular templates in primer design.** | `q5-primer-design.md` §18.2: "A circular template has no availability bound, only a length bound." `primer_design.rs` never reads topology and applies the linear availability bounds everywhere (`:550-557`, `:717-724`, and the tail path). An edit near a plasmid's origin is refused with a false `FivePrime/ThreePrimeAvailability` obstacle. Options: implement wrap-around, or refuse circular templates explicitly and amend §18.2. |
| A3 | **A limit on degenerate expansion.** | `sequence.rs:251` `stride *= len` is unchecked. 32×`N` overflows: a panic in debug builds, while release returns an empty list and `expand_replacement` misreports it as a no-op. At about 12×`N` (16.7M members, each cloning the template), memory runs out first. D033 says "no cap", so a structured refusal needs a decision, and so does any practical limit. |
| A4 | **Abutting intervals in a circular CDS.** | A CDS written `[0,6)+[6,24)` is accepted. An unscoped 1-base insertion at 6 verifies with no frameshift warning, and a scoped insertion or deletion there is refused as `TargetOutsideFeature`. The code follows D032's literal "same interval" wording. Proposal: merge abutting intervals before every containment and frameshift test (`q5.rs:684-690`, `mutation.rs:964-1018`). |
| A5 | **Should `verify` re-translate DNA-level edits?** | D013 asks for "the resulting translation where applicable". For substitution, deletion, insertion and replacement, `verify` ignores the `cds` argument. Example: a scoped substitution creating a premature stop (`TGG→TAG`) comes back `Verified` with no warning. `q5-sdm.md` §8 records only the inert scope argument as a limitation. Choose: implement, or record it as an accepted gap. |
| A6 | **`ConfineToTail` for insertions.** | D032: "The 'confine to 5′ tails' option does not change an insertion." `primer_design.rs:440-446` refuses the placement for every edit. Meanwhile, tailed designs report `placement = ConfineToTail` for an `Inline` request. *Suspected* spec/code mismatch. |
| A7 | **Python verification environment.** | Biopython 1.88 is required by every `native*.py` generator and by the `tm_crosscheck.rs` values. It exists only in the gitignored `.tmp/tm/venv`. The scripts that verified `tm/` (`.tmp/tm/verify_imm.py`, `crosscheck.py`) and the Kazusa parse are untracked. Proposal: track the verification scripts under `tests/fixtures/*/reference/` plus a pinned `requirements.txt`. |
| A8 | **Permission to fix drift in `docs/decisions.md` and the fixture READMEs.** | AGENTS.md reserves these. The edits would be status and drift fixes only, listed in §D. |

---

## B. Correctness defects (no decision needed)

**Fixed 2026-10-16** (one commit each, contracts in `docs/tasks/review-b*.md`):
B1 (`efd83d3`), B2 (`c38eec9`), B3 (`82a867f`), B15 (`5c3668f`), B4 (`c6a2b8f`),
B5 (`a20b9ba`), B6. Still open: B7–B14, B16.

### High

**B1. The pair validator accepts a tailed replacement primer that doesn't carry the edit** (confirmed)
- **Where:** `primer.rs:966` (`construct_tailed_primer`) and `primer.rs:1307-1308` (`assemble_pair`).
- **What's wrong:** `construct_tailed_primer` never checks that a replacement primer's span stays outside `[s, e)`. In `assemble_pair`, the tail-rebuild branch is entered only when `forward.span.start >= e`. Any other forward span falls through to the substitution branch, where covering `[s, e)` with wild-type bases passes.
- **Evidence:** replacement `[50,53)→AAAA`, a forward primer of tail `GGG` + `template[40,70)`, and a reverse primer on `[20,40)`. The pair passed `assemble_pair` and `require_design_constraints(…,15)`, which violates H2, H3 and H8.
- **Impact:** today's generator produces correct pairs. The bug is in the independent check that is meant to catch generator bugs.
- **Fix direction:** refuse an overlapping span in `construct_tailed_primer`, and run the tail-rebuild check whenever either primer has a tail.

### Medium

**B2. u64 overflow in design feasibility sums** (confirmed)
- **Where:** `primer_design.rs:529`, `:696`, `:912`.
- **Trigger:** `PrimerDesignRequest` with `min_overlap = i64::MAX`.
- **Effect:** debug builds panic. Release builds wrap and report `NoPrimerDesignFits` (Unsupported) instead of `NoPrimerDesignPossible`.
- **Fix:** checked or saturating arithmetic, or refuse a bound larger than the template first.

**B3. Public functions that can panic** (confirmed)
- `mutation.rs:2032` `codon_positions`: `number - 1` underflows at `number == 0`; `total_length - 1 - read` underflows on a minus-strand codon past the CDS end.
- `sequence.rs:375` `letter_at(u64::MAX)`: `position + 1` overflows while building the error.
- `mutation.rs:933` `require_within_feature` is public and checks only the interval start.
- **Fix:** make these private, or use checked arithmetic and return structured errors.

**B4. Residue verification checks only the CDS id** (confirmed, by reading)
- **Where:** `mutation.rs:1906`.
- **What's wrong:** `ResidueMutation` doesn't record the CDS intervals or strand. Any same-id annotation on the artifact is accepted. `original_codon` and `actual_original` are never re-checked.
- **Fix direction:** record the intervals and strand in the result and compare them during verification.

**B5. Tailed replacements are reported in deletion and insertion wording** (confirmed)
- `primer.rs:1347` stores `deleted_width = w` for a replacement, so `relationship()` prints "abutting across a deletion of 8 template positions" for an 8→10 replacement.
- `tail_reason` (`primer.rs:~1120`) says "carries the insertion".
- `PrimerPair::deleted_width` and `PrimerDesign::deleted_width` docs are wrong for replacements.
- **Fix:** a neutral "spanned width" plus the edit kind, with wording chosen per kind.

**B6. Tm fallback is broader than D030 allows** (confirmed, by reading)
- **Where:** `tm.rs:510-520`.
- **What's wrong:** `melting_temperature_or_matched` falls back to matched for any `TmParametersUnavailable` (length, alphabet), not only for adjacent mismatches. The design paths are guarded today.
- **Fix:** match on the `aspect`.

- **B15. Degenerate expansion overflows before it warns** (confirmed; see A3).
  - **Where:** `sequence.rs:251` (`stride *= len`, unchecked), reached from `q5_expand.rs:169`.
  - **Effect:** 32×`N` panics in debug builds. In release the count wraps to 0, and `expand_replacement` misreports the request as a no-op.
  - **Fix:** `checked_mul` while computing the count, and a structured error on overflow. The count limit is representability (`usize`), not a policy cap.

### Low

- **B7. Terminal mismatches scored with internal parameters** (confirmed, by reading). `tm.rs:409-422` scores them with the internal-mismatch table, contradicting the module's "reported rather than approximated". Reachable through the public `melting_temperature` only.
- **B8. Symmetry correction gated wrong** (confirmed, by reading). `tm.rs:451-458` applies the symmetry correction on the mismatch-aware basis. Gate it on `MatchedComplement`. Negligible effect.
- **B9. Wild-type skip compares against the request** (confirmed, by reading). `q5_expand.rs:173` compares against the caller's `expected_original`, not the template. A length mismatch can then be hidden behind "would edit nothing".
- **B10. Origin-straddling codon coordinates out of order** (confirmed). `mutation.rs:2078` `intervals_for` returns them as `[(0,1),(22,24)]`, which is not biological order. The edit itself is correct.
- **B11. `verify` uses `unreachable!`** (`mutation.rs:1621-1627`). A sixth result kind would panic at runtime instead of failing to compile. Fix: one exhaustive match.
- **B12. Algorithm version not bumped.** `MUTATION_ALGORITHM_VERSION` is still `0.2.0` (`mutation.rs:69`) after deletion, insertion and replacement were added (D021).
- **B13. Half-open intervals printed with a closed bracket.** `primer.rs:~495`, `:613`, `:1226` print `[20, 40]` for `[20,40)` (D004). `:842` is correct.
- **B16. Wrong position in the Tm alphabet error** (confirmed; found during
  B6). `tm.rs` checks `primer.iter().chain(site.iter())`, so a non-ACGT base
  in the site is reported at its index plus the primer length (site index 9
  of an 18-base primer is reported as "position 27"). Reporting text only;
  the error variant and aspect are correct.
- **B14. Inline ambiguity scan has drifted** (suspected). `primer_design.rs:570` includes the edit region in the inline path's ambiguity scan; the deletion and tail paths skip it. Check against §18.2.

---

## C. Tests and fixtures

### High

- **C1. Unit-test expectations credited to a script that cannot produce them.** `primer_design.rs:1377-1381` says its expectations came from `native-design.py`, but that script's `--synthetic` mode doesn't cover the `TEMPLATE` at `:1386`. The real drivers are untracked (`.tmp/review/native_design.py`, `.tmp/tm/rederive.py`). The same applies to the "run by the planner" values at `:1997` and `:2035`. Spot checks reproduce, but the provenance isn't reproducible from tracked files. This is the AGENTS.md fixture rule.
- **C2. The Watson–Crick verification can't be rerun.** `tm/README.md:55-57` claims the matched table was verified against ΔG°37, but `nn-matched.tsv` has no ΔG°37 column and `tm_tables.rs` reruns the check for mismatches only.
- **C3.** See A7: verification scripts and the Python environment are untracked.

### Medium

- **C4. Most of `q5-primer/cases.tsv` is unread.** `q5_primer_fixtures.rs:396-401` reads 4 of 13 columns. `edits`, `min_overlap`, `strand`, `expected` and `invariants` are never checked. Rows q5p-05, -06, -08, -09 and -10 are effectively dead.
- **C5. Minus-strand behavior is untested at the Q5 and primer layers.**
  - `q5-primer-design.md:281-284` says q5p-08 pins it, but no test reads q5p-08.
  - Test code has zero `Strand::Minus` cases in `q5.rs`, `q5_design.rs`, `primer.rs` and `q5_expand.rs`.
  - No minus-strand deletion, insertion or replacement test exists.
- **C6. `example-construct/` is unread.** No test reads it. Its only consumer, `tests/test_nebasechanger.py`, imports a module that doesn't exist. The README wording implies a live test.
- **C7. Error variants never asserted:** `PrimerAnnealingNotTemplate` (`primer.rs:888`), `PrimerPartition` (`:899`), `PrimerOrientationConflict` (`:1204`). Test them, or document them as unreachable defensive checks.
- **C8. Rules told apart by message text.** `InvalidMutationRequest` has 26 producer sites and `InvalidPrimerCandidate` has 18. About 20 tests tell those rules apart with `reason.contains(...)`, against `error.rs:4-6`. Fix direction: give each rule a structured sub-kind on the problem payload. That's a medium-sized change, so do it after B1–B6.
- **C9. Nothing checks the NEB reproduction claim.** No test checks that the reference script reproduces the SID-1 pair in `sdmsummary.txt` (`q5-primer/README.md:143-146`). It matches today by manual check.

### Low

- **C10. Bare `is_ok()`/`is_err()` assertions** where a variant should be checked: `cds.rs:373-394`, `translation.rs:365-367`, `identity.rs:311-312`, `q5.rs:3818`. In addition, `mutation_fixtures.rs:961-984` doesn't check the category.
- **C11. The codon-usage table has no fixture.** `codon_usage.rs:32-49` holds the 64 Kazusa counts; only the total and the order are tested.
- **C12. A generator hard-codes the developer's path.** `neb-q5-sdm-reference.py:40` hard-codes `/home/timw/...`, and the path is baked into `designs.json`. The generators also overwrite tracked files and have no check-only mode.
- **C13. Missing edge-case tests:** an insertion at point 0 or n on a circular sequence, and indels scoped to an origin-wrapping CDS.

Reproduction check (reviewer c):
- **Reproduced:**
  - All four `native*.tsv` files regenerate byte-identically.
  - `designs.json` regenerates identically except for the embedded absolute path.
  - The digests, the Biopython cross-check values and the SID-1 reconstruction.
- **Independence:** no generator imports or shells out to `crates/core`.

---

## D. Documentation drift

**In code** (worker contract, or planner one-liners):
- **`q5.rs`:**
  - Module docs (`:5-10`, `:50-55`) and `execute` (`:843`) say no primer-design or thermodynamic rules exist; D029–D033 and `tm.rs` define them.
  - `:47`, `:840`, `:938`, `:1596` list only `apply_nucleotide`/`apply_residue`.
  - `:169` says "the other two intents"; there are six intent variants.
  - `:612` says "stated once here and nowhere else".
  - `:614` mentions an intron position, which can't be represented.
- **`mutation.rs`:**
  - `:8`, `:17` say "Three intents"; there are five.
  - `:1347`'s whole-sequence rationale is wrong.
- **`primer.rs`:**
  - `:37-50` says no Tm exists (the tail and design wording was fixed in B5).
  - Still single-case after B5: the `PrimerPartRole` doc, the `abuts()` and
    `boundary()` docs, and the `changed_positions` comment in
    `construct_tailed_primer`.
  - `:345-349`, `:398-404` say tails and `deleted_width` are insertion/deletion-only.
- **`primer_design.rs`:**
  - `:13` and `:56` give a `1 ≤ L ≤ 3` scope without D033.
  - `:291-376` (`edit_span`, the flanks, `forward_tm`, `forward_whole_tm`, `tm_balance_met`) and `:1111-1236` (`Sizer`, `size_pair`) don't cover replacement or the tail path.
- **General:** module docs narrate milestone history ("Milestone 4e-2's …") instead of current behavior. That's why they go stale. The `q5.rs` module doc alone is 95 lines. A trimming pass is worth one contract.

**In docs:** done 2026-10-16 under A8, except the `note` column of
`q5-primer/cases.tsv` (q5p-08 still gives `TAA`/`TTA`). That column is part
of a fixture file, so it was left alone and flagged in the README instead.
The items as found:
- **`decisions.md:934`:** D031's frameshift warning says "Not yet implemented"; it is implemented.
- **`q5-sdm.md:401`, `:572`:** describe length-preservation only, and list indels and warnings as out of scope.
- **`q5-primer-design.md` §7 H2:** not amended for D033's tail path, where neither primer covers the region.
- **`README.md` Status:** still describes Milestone 1.
- **`tests/fixtures/README.md`:**
  - The index lacks `tm/` and `primer-quality/`.
  - The q5-primer line is stale.
  - The example-construct text implies a live test.
- **`q5-primer/README.md`:**
  - `:101-106` says indels aren't representable.
  - `:134-138` says no Tm or Ta is asserted.
  - `:66-67` contradicts §6 on strand.
  - `:95` gives stale q5p-08 codons.
  - `:38-40` gives the wrong path base.
- **`mutation/README.md`:**
  - `:193-194` says indels aren't modelled.
  - `:100` lists 11 codons where there are 10.
- **`core-foundation/README.md:159-162`:** says no operation consumes a residue position.
- **`tm/README.md:116-118`:** "twelve primers" is really about 5 distinct ones.

---

## E. Duplication and refactor candidates (behavior-preserving)

Do these after B and C. Stronger tests make a refactor safe.

- **E1. Design paths.**
  - `design_deletion` (`primer_design.rs:676-839`) and `design_tailed` (`:894-1109`) are nearly identical; a deletion is the tail path with empty text.
  - `design_inline` repeats the same blocks and has already drifted (B14).
  - The `tm_balance_met` and annealing-temperature formulas are written out three times (`:625`, `:794`, `:1015`).
- **E2. Pair checks.**
  - `assemble_pair`'s insertion and replacement tail-rebuild checks (`primer.rs:~1256-1345`) are copies.
  - Claim extraction is duplicated between `primer.rs:1476-1515` and `primer_design.rs:472-487`, `:689`, `:856`.
  - `runs` and `replacement_runs` are near-copies.
- **E3. Containment and frameshift predicates.** "Point inside an interval" and "coding positions of a region" are implemented separately in `q5.rs` (`edit_warnings`, `insertion_warnings`, `replacement_warnings`) and `mutation.rs` (`require_*_within_feature`). The deletion warning tests the coding count while the replacement warning tests `m − w`; check that this is intended for edits straddling a CDS edge. Settle after A4.
- **E4. The four `apply_*` functions** repeat the same opening checks.

Not findings: explicit type annotations, `while` loops and clone-returning getters. They are a consistent house style. The owned-clone getters cost little at plasmid sizes; revisit only if profiling says so.

---

## Suggested Phase 2 order

1. ~~Owner answers A1–A8.~~ Done 2026-10-16, see §A. A4–A7 deferred.
2. **B1** (validator), then **B2 + B3** (panics), then **B15, B4, B5, B6** as single-function contracts, each with a failing test written first.
3. **C1, C2**: make verification reproducible from tracked files (partly
   blocked by A7, which is deferred).
4. **C4, C5**: read the remaining `cases.tsv` columns, and add minus-strand cases with independently verified expectations.
5. **D**: one doc-trimming contract for code comments (`crates/`). The
   non-code docs are done.
6. **E1–E3** refactors, once the tests above are in place.
7. Low items as time allows, or folded into the contracts above.
