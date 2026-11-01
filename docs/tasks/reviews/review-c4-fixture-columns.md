---
type: Task Contract
title: "Review fix C4, C6, C9: read every column of cases.tsv and the NEBaseChanger files"
description: "Every expectation below is a consistency check between files that already exist, or a check of luria against external NEBaseChanger output."
status: deprecated
---
# Review fix C4, C6, C9: read every column of `cases.tsv` and the NEBaseChanger files

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review. `cases.tsv` has 14 columns, not 13 as §1 says; all 14 are read and 11 tested. No fixture mismatch was found.
Planner: Claude. Parent: [docs/tasks/reviews/review-post-m4.md](/tasks/reviews/review-post-m4.md) findings C4, C6, C9.

## 1. The gaps

- **C4.** `crates/core/tests/q5_primer_fixtures.rs` reads 4 of the 13 columns
  of `tests/fixtures/q5-primer/cases.tsv` (`case_id`, `sequence_file`,
  `sequence_digest_16`, `reference_design`), and checks digests only for
  reference-design rows. The `edits` column can drift from `designs.json`
  unnoticed, and rows q5p-05, -06, -08, -09 and -10 are never checked.
- **C6.** No test reads `tests/fixtures/example-construct/` beyond the input
  FASTA. Unread: `raw/sdmsummary.txt`, `raw/sdmprimer.txt`,
  `raw/sdmfinalseq.fa`, `expected/expected_mutation.json`,
  `input/mutation_spec.json`.
- **C9.** `q5-primer/README.md` says the reference script reproduces the
  NEBaseChanger pair in `sdmsummary.txt`. No test checks it.

Every expectation below is a **consistency check between files that already
exist**, or a check of luria against external NEBaseChanger output. None
comes from running luria and copying its output.

## 2. Required tests, all in `crates/core/tests/q5_primer_fixtures.rs`

Reuse the file's existing helpers: `read`, `case_rows`, `case_row`,
`template_of`, `template_from`, the `Json` reader, `designs_json`,
`reference_designs`, `edit_of`. Extend `CaseRow` to carry the columns it
doesn't read yet. Each test below is a separate `#[test]`, with the name
given.

### 2.1 `cases.tsv`

1. `every_case_names_the_digest_of_its_sequence_file`: for **all 10 rows**,
   the first 16 hex digits of the template's digest equal
   `sequence_digest_16`. Follow the existing reference-only digest test.
2. `every_case_edit_asserts_the_bases_its_template_holds`:
   - parse `edits` as `start:end:original:replacement`, with `;` between the
     edits of a `replace-set`;
   - for every edit of every row, `original` equals the template's canonical
     text at `[start, end)`, and `end - start == original.len()`.
3. `every_case_kind_agrees_with_its_edits`:
   - `replace`: one edit, `original` and `replacement` both non-empty;
   - `insert`: one edit, `original` empty, `start == end`, `replacement`
     non-empty;
   - `delete`: one edit, `replacement` empty, `original` non-empty;
   - `replace-set`: two or more edits.
4. `every_reference_case_agrees_with_its_design_record`: for each
   `reference_design = yes` row, compared against `designs.json` under the
   case's short id:
   - `edits` equals the record's edit(s). Read how q5p-07 stores its two
     edits in `designs.json` and compare in order. If the structure doesn't
     allow an exact comparison, stop and report;
   - `min_overlap` equals `request.minOverlapSupplied`;
   - `target_tm` equals `request.targetTm` (`55` against `55.0`).
5. `every_case_names_only_defined_invariants`: every comma-separated token of
   `invariants` is one of `H1`…`H8` or `I-1`…`I-13`
   ([docs/specs/q5-primer-design.md](/specs/q5-primer-design.md) §7 and §14.1).
6. `every_reference_check_matches_its_record`:
   - a reference row's `reference_check` starts with `H1=pass` or
     `H1=VIOLATED`, and agrees with the H1 outcome its `designs.json`
     record holds. Use the same field the existing test
     `luria_reaches_the_same_h1_outcome_as_each_reference_design` reads;
   - a non-reference row's `reference_check` starts with `unverified(`.
7. `the_no_design_case_gets_no_design_from_luria`: for row
   `q5p-09-bound-exceeds-template` (`expected = no-design`), build the edit
   (mut-a, `4:5:C:T`) with the mutation layer and call `design_primers` with
   bound 31. Build the request the way the native tests in this file do,
   with the same buffer. Expect `Err(DomainError::NoPrimerDesignPossible(p))`
   with `p.template_length == 30`. That is I-9: a failed design emits no
   pair.
8. `the_no_tm_target_case_reports_the_tm_input_as_unresolved`: for row
   `q5p-10-no-tm-target` (I-10), design its edit on the SID-1 template under
   `luriaNative` with bound 15. Assert that the design's unresolved inputs
   include one whose `input` is `"minTm"`. If no `PrimerDesign` accessor
   exposes them, stop and report.

The `strand`, `cds` and `note` columns are descriptive. Do not test them;
§3 records that.

### 2.2 `example-construct/` (C6, C9)

Parse the tab-separated header and row of `sdmsummary.txt` by column name.

9. `the_reference_script_reproduces_the_nebasechanger_pair`: the summary
   row's `Fwd Primer Seq` and `Rev Primer Seq`, upper-cased, equal
   `designs.json` `q5p-01` `forward.sequence` and `reverse.sequence`.
   - Both `Length` columns equal the records' `length`.
   - `Tm Fwd` and `Tm Rev` equal the records' `tm` rounded to the nearest
     integer.
   - If the record has a `ta` value, `Ta` equals it rounded. If it has none,
     skip that comparison and say so in a comment.
10. `the_primer_list_agrees_with_the_summary`: `sdmprimer.txt` has two lines.
    Their sequences equal the summary's two primers, compared
    case-insensitively, and their `Tm=`/`Ta=` values equal the summary's.
11. `the_normalized_record_transcribes_the_summary`: in
    `expected/expected_mutation.json`:
    - `id`, both primer `id`s, both `sequence`s (exact, case preserved),
      `length`s, `tm`s and `ta_recommended` equal the summary row;
    - `replacement_bases == "AAC"`;
    - `wild_type_range == [1210, 1212]`.
12. `the_mutation_spec_is_case_q5p_01`: in `input/mutation_spec.json`:
    - `target_position − 1 == 1209` (one-based to zero-based);
    - `original_sequence` and `desired_sequence` equal q5p-01's `edits`
      original and replacement;
    - `minimum_overlap_bp` equals q5p-01's `min_overlap`.
13. `luria_reproduces_the_nebasechanger_final_construct`:
    - build q5p-01's edit with the mutation layer, as
      `each_reference_edit_is_reproduced_by_the_mutation_layer` does;
    - read `raw/sdmfinalseq.fa`: skip the header line, join the remaining
      lines, upper-case;
    - assert it equals the edited artifact's canonical text;
    - assert the header line contains `"replaced 1210 to 1212"` and that the
      edit is `[1209, 1212)`. That is the one-based inclusive to zero-based
      half-open conversion the fixture README describes.

## 3. README updates

- **`tests/fixtures/q5-primer/README.md`.** Replace the "Test coverage of
  `cases.tsv` is partial" bullet:
  - every column is now read, except `strand`, `cds` and `note`, which are
    descriptive;
  - list which test checks which column;
  - change the sentence "Running it reproduces the SID-1 D404N pair that …
    `sdmsummary.txt` records" to name test 9 as the check.
- **`tests/fixtures/README.md`.**
  - In the index row for `example-construct/`, replace "no test reads the
    NEBaseChanger output files yet" with the name of the test file that now
    reads them.
  - In the paragraph about the planned importer test, add one sentence:
    `q5_primer_fixtures.rs` checks the same `[1209, 1212)` reading against
    the final FASTA header.

## 4. Do not

- Edit files other than `crates/core/tests/q5_primer_fixtures.rs` and the two
  READMEs.
- Change any fixture data file, or any existing test or assertion.
- Add dependencies.
- Make a check pass by loosening it. If any check in §2 fails, stop and
  report the mismatch. It is a finding about the fixtures, not something to
  fix here.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

## 6. Handoff

Report the changed files, the new tests, the exact check results, any check
that failed on first run and why, and any deviation.
