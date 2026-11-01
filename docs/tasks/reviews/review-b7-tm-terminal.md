# Review B7 and B16: terminal mismatches and the alphabet error position in `tm.rs`

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review.
Planner: Claude. Parent: [docs/tasks/reviews/review-post-m4.md](/tasks/reviews/review-post-m4.md) B7 and B16.
File: `crates/core/src/tm.rs` only.

## 1. Defects

- **B16.** `melting_temperature` checks the alphabet with
  `primer.iter().chain(site.iter()).enumerate()`. A non-ACGT base at site
  index 9 of an 18-base primer is reported as "position 27".
- **B7.** A mismatch at the first or last position of the duplex is scored
  with `MISMATCHED`, the table for *internal* mismatches. Terminal
  mismatches have their own published parameters, which this model doesn't
  carry. The module doc's "Deliberately absent" section already says there
  is no terminal-mismatch table, and the module's rule is that a missing
  parameter is reported, not approximated.

## 2. Required changes

1. **B16.** Check the primer's bases, then the site's, each indexed from 0
   in its own strand. The reason becomes
   `"{base} at position {index} of the primer is not A, C, G or T, and has no nearest-neighbour parameters"`,
   or the same with `of the template site`. The aspect stays `"alphabet"`.
2. **B7.**
   - After the alphabet check, and before the doublet walk, check position 0
     and then position `len − 1`.
   - If the pair there is not Watson–Crick (`is_watson_crick(primer[i],
     bottom[i])`), return `unavailable("terminal mismatch", …)`. The reason
     is `"position {i} is mismatched at an end of the duplex; terminal mismatches have their own parameters, which this model does not carry"`.
   - An internal mismatch (positions 1 to `len − 2`) is scored exactly as
     today.
3. **The fallback.** `melting_temperature_or_matched` also falls back on the
   aspect `"terminal mismatch"`. That is a mismatch pattern the model's
   parameters don't cover, which is what D030's fallback is for, just like
   `"motif"`. Update its doc comment to list the three aspects.
4. **Docs.** In the module doc's "Deliberately absent" paragraph, say that a
   terminal mismatch is reported as `TmParametersUnavailable` with aspect
   `"terminal mismatch"`. Don't change any other doc text.

## 3. Tests: write them first, and confirm 1–4 fail before the change

Use the module's existing `letters` helper, the primer
`ATGCATGCATGCATGCAT` (18 bases), and `BufferModel::q5_reaction()`.

1. `a_non_acgt_base_in_the_site_is_reported_at_its_site_position`: site =
   primer with index 9 set to `N`. `melting_temperature(primer, site)` is
   `TmParametersUnavailable` with aspect `"alphabet"`, and its reason
   contains `"position 9 of the template site"`.
2. `a_mismatch_at_the_first_base_has_no_parameters`: site = primer with index
   0 set to `T`. The error is `TmParametersUnavailable`, the category is
   `Unsupported`, the aspect is `"terminal mismatch"`, and the reason
   contains `"position 0 "`.
3. `a_mismatch_at_the_last_base_has_no_parameters`: index 17 set to `A`.
   Aspect `"terminal mismatch"`, and the reason contains `"position 17 "`.
4. `the_hybrid_falls_back_on_a_terminal_mismatch`: the site from test 2,
   through `melting_temperature_or_matched`. The basis is
   `MatchedComplement`, `substituted_because()` contains
   `"terminal mismatch"`, and `celsius()` equals the matched Tm of the
   primer exactly (as in `the_hybrid_falls_back_to_the_matched_basis_and_says_so`).
5. `a_non_acgt_base_in_the_primer_is_reported_at_its_primer_position`:
   primer index 8 set to `N`, site = that same primer. The reason contains
   `"position 8 of the primer"`. This may pass before the change; that's
   expected.
6. `a_mismatch_next_to_an_end_is_still_scored`: index 1 set to `A`. This is
   `Ok`, with basis `MismatchedWildTypeSite`. It may pass before the change.

## 4. Stop and report if

- any existing test fails after the change, including the fixture tests in
  `crates/core/tests/` (a fixture case would then depend on a terminal
  mismatch);
- `tm_crosscheck.rs` changes value.

## 5. Do not

- Edit any file other than `crates/core/src/tm.rs`.
- Change `TM_FUNCTION_VERSION` (B12 does that).
- Change any existing test.
- Add dependencies.

## 6. Checks

From the repo root, with `TMPDIR=$PWD/target/tmp` (run `mkdir -p
target/tmp` first):
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 7. Handoff

Report:
- the changed file;
- which tests failed before the change, and how;
- the check results;
- any deviation.
