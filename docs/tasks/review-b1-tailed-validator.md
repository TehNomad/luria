# Review fix B1: the pair validator must catch a tailed primer over the edit

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review.
Planner: Claude. Parent: `docs/tasks/review-post-m4.md` finding B1.

## 1. The defect

`primer.rs` is the independent check on designs. For a replacement it
accepts a pair whose forward primer anneals straight over the replaced bases
and carries an arbitrary 5′ tail, so the pair does not contain the edit
(violates H2, H3, H8). Reproduced:

- replacement `[50,53)` → `AAAA` on a 100+ base template;
- forward = `construct_tailed_primer(…, Forward, tail "GGG",
  annealing = template[40,70), span [40,70))` → accepted today;
- reverse = `construct_primer(…, Reverse, Inline, revcomp(template[20,40)),
  span [20,40))`;
- `assemble_pair(forward, reverse, result)` → accepted today, and
  `require_design_constraints(pair, 15)` passes.

Two causes:

1. `construct_tailed_primer` (`primer.rs`, around line 966) never checks
   where the annealing span sits relative to the edit.
2. `assemble_pair` (`primer.rs`, around line 1307) enters the replacement
   tail branch only when `forward.span().start() >= region end`. Any other
   forward span falls through to the substitution branch, whose coverage
   check passes because the span covers `[s, e)`.

## 2. The requirement (q5-primer-design.md §7, H2 note added 2026-10-16)

> For a deletion, and an insertion or replacement placed in the 5′ tails,
> H2 means: (a) the pair invariant holds — reverse annealing end + `w` =
> forward annealing start, with `w = 0` for an insertion; (b) neither
> annealing portion overlaps the replaced or deleted region `[s, e)`; and
> (c) for a tailed edit, the reverse tail read in the template's direction
> followed by the forward tail equals the inserted or replacement text
> exactly. An inline replacement (`max(w, m) ≤ 6`) is covered by the
> original H2.

On the D033 tail path, the forward primer always has a tail
(`ceil(m/2) ≥ 1`), but the reverse primer may have none (`floor(m/2) = 0`
when `m = 1`). An inline pair has no tail on either primer.

## 3. Required behaviour, `crates/core/src/primer.rs` only

**3.1 `construct_tailed_primer`**: after the existing span, source and
edit-kind checks, and before any text check, refuse a span that cannot hold a
tailed primer:

- replacement with region `[s, e)`: refuse when the span overlaps `[s, e)`,
  i.e. `span.start() < e && span.end() > s`;
- insertion at point `p`: refuse when the span straddles the point, i.e.
  `span.start() < p && span.end() > p`.

The error is `DomainError::InvalidPrimerCandidate` with `primer` the existing
`label`. Use the existing claim helpers (`replacement_claim`,
`insertion_claim`) for `s`, `e` and `p`. Do not re-derive them.

**3.2 `assemble_pair`**: the replacement tail branch is entered when the
result is a replacement **and either primer has a tail**
(`forward.tail().is_some() || reverse.tail().is_some()`), not when
`forward.span().start() >= e`. Inside the branch, keep the existing checks
(the edge check, then the tail rebuild) unchanged. A replacement pair with
no tail on either primer still goes to the substitution and coverage branch.
Update the comment on that branch to match.

Use whatever accessor `Primer` already has for its tail. If it has none,
stop and report; do not add a public API.

## 4. Required tests, in `primer.rs`'s test module

Write these first and confirm that 1–3 fail before the fix. Use the existing
helpers (`replacement`, `replacement_forward_primer`,
`replacement_reverse_primer`, `insertion`, `tailed_forward`,
`untailed_reverse`, etc.).

1. `a_tailed_replacement_primer_may_not_anneal_over_the_region`: the §1
   reproduction. `construct_tailed_primer` returns
   `Err(DomainError::InvalidPrimerCandidate(_))`.
2. `a_tailed_insertion_primer_may_not_straddle_the_point`: an insertion at
   `p`, and a tailed forward primer whose span has `start < p < end`. Expect
   `Err(DomainError::InvalidPrimerCandidate(_))`.
3. `an_inline_replacement_pair_with_a_tailed_reverse_primer_is_refused`:
   - the forward primer is untailed and inline, built with
     `construct_primer`, covering the replaced region with the edited
     artifact's text;
   - the reverse primer is built with `construct_tailed_primer`, with its
     span ending exactly at `s` (allowed by 3.1) and any non-empty tail;
   - `assemble_pair` returns `Err(DomainError::InvalidPrimerCandidate(_))`,
     because the forward primer does not start at `e`.
4. `a_tail_path_replacement_with_an_untailed_reverse_primer_is_accepted`:
   - a replacement with `m = 1` and `w ≥ 7` (e.g. a 10-base region replaced
     by one base);
   - the forward primer is tailed with the one base and its span starts at
     `e`;
   - the reverse primer is untailed with its span ending at `s`;
   - `assemble_pair` returns `Ok`. This pins the "either primer" condition.

All existing tests must keep passing unchanged.

## 5. Do not

- Edit any file other than `crates/core/src/primer.rs`.
- Add dependencies, public functions, error variants or fields.
- Change or delete existing tests. Change the deletion or insertion branches
  of `assemble_pair`.
- Touch the wording problems in B5 (`deleted_width`, `tail_reason`); they are
  a separate contract.

## 6. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

## 7. Handoff

Report the changed file, the new tests and that 1–3 failed before the fix,
the exact check results, and any deviation from this contract.
