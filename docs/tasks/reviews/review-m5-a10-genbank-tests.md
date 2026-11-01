---
type: Task Contract
title: "Review M5 A-8, A-10: GenBank test gaps"
description: "The test helper fixture_id skips an id qualifier that has no value and moves on to the next one."
status: deprecated
tags: [milestone-5]
---
# Review M5 A-8, A-10: GenBank test gaps

Status: **Implemented 2026-10-18** by the local worker; accepted on planner review. Planner: Claude. Parent:
[docs/tasks/reviews/review-post-m5.md](/tasks/reviews/review-post-m5.md) A-8 and A-10. Test-only: no production code
changes.

Files:
- `crates/core/tests/genbank_fixtures.rs`
- `crates/core/src/genbank_cds.rs` (test module only)

## 1. Gaps

- **A-8.** The test helper `fixture_id` skips an id qualifier that has no
  value and moves on to the next one. luria's `id_of` (D041 G2,
  `genbank_cds.rs` §4.2 step 1) instead stops at the first of `/locus_tag`,
  `/gene`, `/label` that appears, and a feature whose first occurrence of it
  has no value has no id. The helper's doc comment claims luria's rule.
- **A-10.** Fixture columns no test reads:
  - `pbr322-features.tsv`: `id` and `id_source`;
  - `pbr322-cds.tsv`: `codon_start`, `start_codon` and
    `translation_matches`.
  And rules with no test: `RepeatedQualifier` for the id qualifier,
  `/transl_table` and `/translation`, and the order step 4 checks them in;
  an empty id value (`/gene=""`).

## 2. Required change

### 2.1 `genbank_fixtures.rs`

1. **`fixture_id`:** for each of `locus_tag`, `gene`, `label` in that order,
   find the first qualifier with that name. If one is found, return its
   value, or `-` when it has none, and stop. If none of the three is found,
   return `-`. Keep the doc comment, and add that a first occurrence with no
   value means no id.
2. **`pbr322_imports_tet_and_bla_as_biopython_reads_them`:** for each
   imported row (tet, bla), also assert:
   - `codon_start` (`row[6]`) equals the value of the record feature's first
     `/codon_start` qualifier, or `-` when it has none. Read the feature from
     `record.features()` at the row's index;
   - `start_codon` (`row[8]`) equals the first three letters of luria's
     extracted coding text for the imported CDS (`extract_cds` on the
     record's sequence and the imported `Cds`, or whatever accessor the
     existing test uses for residues);
   - `translation_matches` (`row[10]`) is `yes`, and the record feature has a
     `/translation` qualifier. (An imported CDS with a `/translation` has
     passed the G3 check, so `yes` is the only value an imported row can
     have.)
   Each assertion message names the file, the CDS and both values, as the
   existing assertions do.
3. **`pbr322_lists_every_other_feature_as_unused`:** for every row of
   `pbr322-features.tsv`:
   - if the row's index is an imported one (10 or 38): the imported CDS's id
     equals `row[2]` and its `id_qualifier().name()` equals `row[3]`;
   - otherwise: the unused feature at that index has `id()` equal to `row[2]`
     (`None` when the row says `-`), and `row[3]` is `-`.
   If a row's id is set but the feature is unused, or the other way round,
   fail with a message naming the index.
4. Add, in a comment above the pBR322 CDS test, that row 26 (ROP)'s
   `start_codon` and `translation_matches` are Biopython facts with no luria
   counterpart: luria refuses that feature at step 1 (`NoId`), so they are
   not compared.

### 2.2 `genbank_cds.rs` test module

Using `record` (a 12-base linear sequence `ATGGCTTGGTAA`), `key`, `q`,
`names` and `cds_problem`. In each case the feature is `key("CDS",
"1..12")`, which is line 3, plus the qualifiers listed, and the request is
`["g"]` unless stated:
1. `/gene="g"`, `/gene="g"` → `RepeatedQualifier`, detail `Some("gene")`,
   line `Some(3)`.
2. `/gene="g"`, `/transl_table=1`, `/transl_table=1` → `RepeatedQualifier`,
   detail `Some("transl_table")`.
3. `/gene="g"`, `/translation="MAW"`, `/translation="MAW"` →
   `RepeatedQualifier`, detail `Some("translation")`.
4. Order: `/gene="g"`, `/translation="MAW"`, `/translation="MAW"`,
   `/codon_start=1`, `/codon_start=1` → detail `Some("codon_start")`, because
   step 4 checks the id qualifier, then `/codon_start`, then
   `/transl_table`, then `/translation`, whatever the file order.
5. Order: `/gene="g"`, `/codon_start=1`, `/codon_start=1`, `/gene="g"` →
   detail `Some("gene")`.
6. Empty id: the feature with `/gene=""` only, and an **empty** request
   (`names(&[])`). The import succeeds, and the single unused feature's
   refusal has `code()` `"empty_identifier"`. If the refusal turns out to be
   a different error, stop and report it rather than changing the
   expectation.

## 3. How to proceed

These tests describe current behavior; they should pass without production
changes. If any fails, do not change production code: stop and report the
failure output.

## 4. Do not

- Change any production code, fixture or other test.
- Add dependencies.

## 5. Checks

From the repository root, after `export TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 6. Handoff

Report the changed files, the new assertions and tests, the check results,
and any deviation.
