---
type: Task Contract
title: "Milestone 6i0: the linear GenBank fixture"
description: "Every GenBank fixture so far is circular, and luria designs primers on linear templates only (D035, D041 G10)."
status: deprecated
tags: [milestone-6]
---
# Milestone 6i0: the linear GenBank fixture

Status: **Implemented 2026-10-19.** Fixture written and verified by the
planner; tests L1–L4 by the local worker, accepted on the first review. §4.1's
`git diff --exit-code tests/fixtures/` could not pass, because the fixture
files were still untracked and the planner's README edits uncommitted; the
worker showed each restore with `sha256sum -c` instead, and the planner
compared each file with its own copy.
Planner: Claude. Parent: [docs/tasks/contracts/milestone-6-nl.md](/tasks/contracts/milestone-6-nl.md) §8 and M6-13.

## 1. What and why

Every GenBank fixture so far is circular, and luria designs primers on
linear templates only (D035, D041 G10). The natural-language milestone
needs one GenBank record on which a request can run from text to a
designed primer pair (owner answer M6-13). This contract adds that record
and checks luria's reader and CDS import against it, as `pbr322` and
`synthetic` are checked today.

## 2. What the planner has already done (do not change)

All under `tests/fixtures/genbank/`, recorded in its `README.md`, "The
linear record":

| File | What |
| --- | --- |
| `synthetic-linear.gb` | 393 bases, `linear`, LOCUS `synlin`. Features: `source` (index 0), `gene` with `/gene="synA"` (index 1), CDS `43..345` with `/gene="synA"`, `/codon_start=1`, a `/product` and a `/translation` (index 2). |
| `synthetic-linear-record.tsv` | One row, with the header and columns of `pbr322-record.tsv`. |
| `synthetic-linear-cds.tsv` | One row, with the header and columns of `pbr322-cds.tsv`. |
| `synthetic-linear-residues.tsv` | Header `number\tresidue\tcodon\tstart\tend`; rows for residues 1, 42, 87 and 100: the letter, the codon and the codon's template interval, zero-based half-open. |
| `reference/verify_synthetic_linear.py` | Re-derives all three tables with Biopython; listed in `tests/fixtures/verify.py`. |

The values were derived by hand and checked with Biopython and coreutils
(`tests/fixtures/genbank/README.md`). `python -I tests/fixtures/verify.py`
passes 13 of 13.

## 3. Requirements this contract applies

- **D041 G2:** a CDS's id is the first of `/locus_tag`, `/gene`, `/label`.
- **D041 G4:** topology comes from the LOCUS line.
- **D006:** GenBank `a..b` is `[a − 1, b)`.
- **D007:** the digest is over the upper-cased sequence
  (`Sequence::content_hash().hex_digest()`).
- `UnusedFeature::id()` and `refusal()` are `None` for every key other than
  `CDS` (`crates/core/src/genbank_cds.rs`).

## 4. Tests to add: `crates/core/tests/genbank_fixtures.rs` only

Reuse the file's helpers: `table`, `read_input`, `name`, `request`,
`or_dash`, `strand_text`, `first_qualifier`, `RECORD_HEADER`, `CDS_HEADER`.
Add, next to `PBR322_FILE` and `SYNTHETIC_FILE`:

```rust
const LINEAR_FILE: &str = "synthetic-linear.gb";
const RESIDUE_HEADER: &str = "number\tresidue\tcodon\tstart\tend";
```

and a reader `fn linear() -> GenBankRecord`, written like `synthetic()`.

Extend the module doc comment's first paragraph to name the third record
("and a hand-written linear record with one designable CDS, for the
natural-language milestone (M6-13)") and its verifier,
`reference/verify_synthetic_linear.py`. Change no other existing text.

| # | Test | What it asserts |
| --- | --- | --- |
| L1 | `the_linear_record_reads_as_written` | `synthetic-linear-record.tsv` has exactly one row, and the reader's locus name, `or_dash(accession())`, `or_dash(version())`, `sequence().length()`, `sequence().topology().name()`, `sequence().content_hash().hex_digest()` and `features().len()` each equal its column, as in `pbr322_reads_as_biopython_reads_it`'s first part. Also assert the three feature keys, in order, are `source`, `gene`, `CDS`. |
| L2 | `the_linear_record_imports_syna_as_written` | `import_cds(&record, &request(vec!["synA"]))` succeeds with exactly one CDS. Against the one row of `synthetic-linear-cds.tsv` (exactly one row), check every column as `pbr322_imports_tet_and_bla_as_biopython_reads_them` checks them: index, the single interval's start and end, strand, id qualifier, `or_dash(transl_table())`, residue count, `/codon_start`, the first codon of `extract_cds`, `translation_matches == "yes"` and that the feature states a `/translation`. |
| L3 | `the_linear_record_lists_source_and_gene_as_unused` | With `synA` imported, `unused()` is exactly two features: index 0, key `source`, id `None`, refusal `None`; then index 1, key `gene`, id `None`, refusal `None`. These values are written in the test; they follow from §3's last rule and the record's three features. |
| L4 | `the_linear_record_residues_sit_where_the_fixture_says` | `synthetic-linear-residues.tsv` has exactly four rows. For each row, with `synA` imported: the residue letter at position `number − 1` of `translate(...).residues_text()` equals `residue`; the characters `[start, end)` of `record.sequence().canonical_text()` equal `codon`; and characters `[3(number − 1), 3·number)` of `extract_cds(...)` equal `codon`. |

Each assertion message names the fixture file, the field and both values,
like the existing ones.

### 4.1 Proving the tests are load-bearing (AGENTS.md)

For each row, change the one fixture cell named, run the test, confirm that
it fails, and restore the file. Then run `git diff --exit-code
tests/fixtures/` and report that it exits 0.

| Change, temporarily | Test that must fail |
| --- | --- |
| `synthetic-linear-record.tsv`: `linear` → `circular` | L1 |
| `synthetic-linear-cds.tsv`: residues `100` → `99` | L2 |
| `synthetic-linear-residues.tsv`: residue 87's `start` `300` → `297` | L4 |

For L3, temporarily change the expected key `gene` in the test to `CDS`,
confirm that L3 fails, and restore it.

## 5. Do not

- Change any file under `tests/fixtures/`, except the temporary edits in
  §4.1, each restored.
- Change any existing test, helper or constant in `genbank_fixtures.rs`, or
  any file other than `genbank_fixtures.rs`.
- Add dependencies.
- Change production code. If luria disagrees with a fixture value, stop and
  report the file, the field and both values. Do not adjust either side.

## 6. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp` (the sandbox's
`/tmp` cannot be read back; see AGENTS.md):
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 7. Handoff

Report the changed file, the exact commands and their results, the §4.1
results, including the `git diff --exit-code tests/fixtures/` exit status,
and anything that did not match this contract.
