# Review fix B5: tail-path replacements are described as deletions and insertions

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review. Three further single-case doc comments (`PrimerPartRole`, `abuts`/`boundary`, the `changed_positions` comment) are left for the §D doc-trimming contract.
Planner: Claude. Parent: `docs/tasks/review-post-m4.md` finding B5.

## 1. The defect

D033's tail path designs a replacement of `[s, e)` with the two annealing
portions at the region's edges, like a deletion, and the text in the tails,
like an insertion. The pair layer reuses the deletion and insertion wording
for it:

- `assemble_pair` (`crates/core/src/primer.rs`, around line 1426) stores the
  region width in `PrimerPair::deleted_width`. `relationship()` then prints
  "abutting across a **deletion** of 8 template positions" for an 8→10
  replacement. That sentence is H5's user-facing statement of what the pair
  does.
- `construct_tailed_primer` sets `tail_reason` to "a 5′ tail of N bases
  carries **the insertion** and does not anneal", even for a replacement.
- The doc comments of `PrimerPair`, `PrimerPair::deleted_width`, `Primer::tail`,
  `construct_tailed_primer`, the module doc's "No design for an insertion or a
  deletion" bullet (around line 47), and `PrimerDesign::deleted_width`
  (`primer_design.rs`, around line 333) all describe the field and the tail as
  deletion-only or insertion-only.

The numbers are right. Only the wording and docs are wrong.

## 2. Required behaviour

**2.1 `PrimerPair` records why its spans are apart.**
- Add a private field `replaced: bool`. It is true exactly when
  `assemble_pair` takes the D033 replacement tail branch, and false
  otherwise (including deletions).
- Do not add a public accessor. Keep `deleted_width()`, `boundary()` and
  `abuts()` and their arithmetic unchanged.

**2.2 `relationship()`.** In the `width > 0 && gap == signed(width)` arm:
- when `replaced`, the phrase is `"abutting across a replaced region of "` +
  width + `" template positions"`;
- otherwise it stays `"abutting across a deletion of "` + width +
  `" template positions"`, exactly as today.

Nothing else in `relationship()` changes, including the brackets (B13 is a
separate finding).

**2.3 `tail_reason` in `construct_tailed_primer`:**
- insertion: unchanged, `"a 5′ tail of N bases carries the insertion and does
  not anneal"`;
- replacement: `"a 5′ tail of N bases carries part of the replacement and
  does not anneal"`.

**2.4 Doc comments.** Rewrite the comments listed in §1 so they describe
what the code does now:
- `deleted_width` is the number of template positions between the two
  annealing portions: the deleted region for a deletion, the replaced region
  for a tail-path replacement, and 0 otherwise;
- a tail is present on insertion primers and on tail-path replacement
  primers;
- the module-doc bullet no longer claims that nothing designs insertions or
  deletions. `primer_design.rs` does.

Keep each rewrite to the sentences that are wrong.

## 3. Required tests, in `primer.rs`'s test module

Write these first. Tests 1 and 2 must fail before the fix.

1. `a_tail_path_replacement_pair_states_a_replaced_region`: reuse the
   construction of the existing test
   `a_tail_path_replacement_with_an_untailed_reverse_primer_is_accepted`
   (region `[2, 12)` replaced by `G`). Assert that `relationship()` contains
   `"abutting across a replaced region of 10 template positions"` and does
   not contain `"deletion"`.
2. `a_tailed_replacement_primer_says_its_tail_carries_the_replacement`: the
   tailed forward primer from that test has a `tail_reason()` containing
   `"part of the replacement"` and not containing `"insertion"`.
3. Regression: the existing deletion test that asserts
   `relationship().contains("deletion of 3")` still passes unchanged, and a
   tailed insertion primer's `tail_reason()` still contains
   `"carries the insertion"`. Add an assertion only if no existing test
   covers it.

## 4. Do not

- Edit files other than `crates/core/src/primer.rs` and the one doc comment
  in `crates/core/src/primer_design.rs` (`PrimerDesign::deleted_width`).
- Rename `deleted_width` or any public item, add public items, or change any
  arithmetic.
- Change existing tests or fixtures.
- Fix the `]` bracket (B13) or other findings.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

## 6. Handoff

Report the changed files, the new tests and their pre-fix failures, the exact
check results, and any deviation.
