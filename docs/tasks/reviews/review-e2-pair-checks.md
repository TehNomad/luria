# Review refactor E2: one tail-rebuild check and one set of claim readers

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review.
Planner: Claude. Parent: `docs/tasks/reviews/review-post-m4.md` finding E2.

**Behavior-preserving.** Every existing test must pass unchanged, and every
error payload must be byte-identical, message text included.

## 1. The duplication

1. In `crates/core/src/primer.rs`, `assemble_pair` has two copies of the
   tail-rebuild check, one in the insertion branch and one in the D033
   replacement tail branch. They do the same thing:
   - read `tail_text_of(forward)`, `tail_text_of(reverse)` and
     `tail_in_template_direction(reverse)?`;
   - join the reverse head with the forward tail;
   - compare the result with the written text.

   On a mismatch both return `InvalidPrimerCandidate` with `primer: "pair"`
   and a reason. The two reasons are identical except for one word,
   `insertion` or `replacement`, which appears twice in each.
2. `primer.rs` reads edit claims through `deleted_region`, `insertion_claim`
   and `replacement_claim`. `primer_design.rs` reads the same quantities with
   its own `match` blocks, in `design_primers` (the replacement's region and
   text) and `design_insertion` (point and inserted text).

## 2. Required changes

**2.1 `primer.rs`.** Add

```rust
fn require_tails_rebuild(forward: &Primer, reverse: &Primer, written: &str, kind: &str)
    -> Result<(), DomainError>
```

`kind` is `"insertion"` or `"replacement"`. It must produce exactly today's
reason strings. Check that by reading both branches. Replace both copies with
a call to it.

**2.2 Make the three claim readers `pub(crate)`:** `deleted_region`,
`insertion_claim` and `replacement_claim` in `primer.rs`. Don't change their
bodies or signatures.

**2.3 `primer_design.rs`.**
- In `design_primers`, replace the replacement-triple `match` with
  `primer::replacement_claim(result.clone())`. On `None`, return the same
  `out_of_scope(&policy, "the result is not a replacement: design_primers read it as one")`.
- In `design_insertion`, replace its `match` with
  `primer::insertion_claim(result.clone())`. On `None`, return the same
  `out_of_scope(...)` message as today.
- Leave `design_deletion`'s region reading as E1 left it. It needs the empty
  check, which `deleted_region` does not perform.

## 3. Guards

- `python3 -I target/api_guard.py crates/core/src/primer.rs crates/core/src/primer_design.rs`
  prints nothing and exits 0. `pub(crate)` lines are not public API, so the
  guard ignores them. The script text is in
  `docs/tasks/reviews/review-e1-design-paths.md` §3.
- No change below either file's `#[cfg(test)]` line, and none under
  `crates/core/tests/`.

## 4. Do not

- Edit any other file.
- Change messages, payloads or behavior.
- Merge the deletion, insertion and replacement edge checks. Their messages
  differ in structure.
- Touch `runs` or `replacement_runs`.
- Change tests.

## 5. Checks

Same as E1 §5, with both files passed to the API guard.

## 6. Handoff

Report the changed files, the exact check results (510 tests), and any
deviation.
