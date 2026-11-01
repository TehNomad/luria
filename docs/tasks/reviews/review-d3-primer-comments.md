# Review cleanup D3: comments in `primer.rs` and `primer_design.rs`

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review. The worker found that D035 misdescribed the code (circular templates are refused, not designed as linear); corrected in docs separately.
Planner: Claude. Parent: `docs/tasks/reviews/review-post-m4.md` §D.

**Comments only.** No code, string, test or behavior change.

## 1. Rules

Follow §2 of `docs/tasks/reviews/review-d1-q5-comments.md` exactly; its 11 rules
apply here unchanged. In short:
- describe the code as it is now;
- keep every D-number, spec section, H/I invariant and owner decision that
  justifies a rule still in code;
- one-line item summaries;
- don't touch trailing comments, test modules or `Defensive check:`
  comments;
- make no unverified claims;
- accuracy over length.

## 2. Facts (verified by the planner)

- **`tm.rs`** implements the Tm (D030). `primer_design.rs` sizes primers by
  Tm and reports the annealing temperature. `primer.rs` computes neither.
- **Placement.** `construct_primer` refuses `ConfineToTail` as a caller's
  choice. `construct_tailed_primer` builds primers whose placement is
  `ConfineToTail`, derived from their shape.
- **Tails** exist on insertion primers (D032) and on tail-path replacement
  primers (D033, `max(w, m) > 6`).
- **`PrimerPair::deleted_width`** is the number of template positions between
  the two annealing portions:
  - the deleted region for a deletion;
  - the replaced region for a tail-path replacement;
  - 0 otherwise, including insertions.
- **The design scope in `primer_design.rs`:**
  - substitution regions of 1–3 bases (D029/D030, `MAX_REPLACEMENT_REGION`);
  - inline replacements with `max(w, m) ≤ 6` (D033,
    `MAX_INLINE_REPLACEMENT`);
  - tail-path replacements above that;
  - deletions (D031) and insertions (D032), on linear templates (D035).
- **The forward Tm basis:**
  - inline with `m == w`: D030's hybrid (mismatch-aware, falling back to
    matched for tandem or motif gaps, B6);
  - inline with `m ≠ w`: matched;
  - deletion and tail path: matched.

  `forward_whole_tm` and `reverse_whole_tm` are `Some` only on the tail path,
  i.e. insertions and tail-path replacements.
- **`tm_balance_met`** is false when the colder primer couldn't be grown
  within 5 °C of the other before running out of template.

## 3. Known false or stale statements

**`crates/core/src/primer.rs`:**
- module doc, around lines 38–41: "No Tm, no `Ta`. §11 states that the
  project's Tm function is not yet defined". Rewrite it: this module computes
  no Tm; `tm.rs` and `primer_design.rs` do;
- module doc, the `confineToTail` bullet: make it match §2's placement fact;
- `PrimerPartRole` doc, around line 188: "`construct_tailed_primer` builds one
  for an insertion". Add tail-path replacements;
- `PrimerPair::abuts()` and `boundary()` docs, around line 470: "the width of
  the deletion they straddle". Use §2's `deleted_width` wording;
- `construct_tailed_primer`'s doc, around line 992: "the only shape in which
  the `tail` field is filled". Still true, but it describes only the
  insertion case;
- the comment above `changed_positions` in `construct_tailed_primer`: "an
  insertion writes into a gap". Add the replacement case.

**`crates/core/src/primer_design.rs`:**
- module doc, line 13: "`1 ≤ L ≤ 3`" as the whole scope. Give §2's scope;
- `PrimerDesign` accessor docs, around lines 300–380: `edit_span`, the flank
  accessors, `reverse_length`, `forward_tm`, `forward_whole_tm` and
  `tm_balance_met`. They are worded for substitution and deletion only.
  Cover the cases in §2;
- `Sizer` and `size_pair` docs, around lines 1235–1250: worded for
  substitution and deletion. Add the tail path and inline `m ≠ w`.

## 4. Guard

Run the §5 guard of `docs/tasks/reviews/review-d1-q5-comments.md`, with these file
arguments:

```
crates/core/src/primer.rs crates/core/src/primer_design.rs
```

It must print nothing and exit 0.

## 5. Do not

- Edit any other file.
- Change any line that isn't a whole-line comment.
- Touch the test modules.
- Fix the `]` brackets in reason strings (B13). Those are code.

## 6. Checks

Same as D1 §7.

## 7. Handoff

Report:
- comment line counts before and after, per file;
- each §3 item and how you fixed it;
- any citation you dropped, and why;
- the exact check results.
