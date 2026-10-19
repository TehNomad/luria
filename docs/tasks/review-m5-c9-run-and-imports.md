# Review M5 C-6, C-9, C-12: run invariants, the I/O exit, imported CDS locations

Status: **Implemented 2026-10-18** by the local worker; accepted on planner review. Planner: Claude. Parent:
`docs/tasks/review-post-m5.md` C-6, C-9 and C-12. Owner answers, 2026-10-18:
C-6 "Exit 1 wins; document it" (the docs are a separate contract; this one
adds the test); C-12 "add intervals + strand to importedCds".

Files:
- `crates/cli/src/run.rs`
- `crates/cli/src/result.rs` (`ImportedCdsView` and `imported_cds_view`
  only)
- `crates/cli/tests/result.rs`, `crates/cli/tests/write_edited.rs`

## 1. Defects

- **C-9.** `run.rs:153` calls `execution.mutation().unwrap()`. The module
  doc says a broken core invariant gives an `Internal` document, but this one
  panics instead (exit 101, no JSON).
- **C-6.** The path where `--write-edited` fails with an I/O error (exit 1,
  document still printed, the OS message on stderr) has no test.
- **C-12.** A GenBank template's `provenance.template.genbank.importedCds`
  entries give the id, its qualifier, the feature index, the line and the
  table, but not where the CDS is. A `needs_clarification` document for a
  GenBank request therefore lists candidates whose locations a consumer can't
  see. FASTA requests have them in the request echo.

## 2. Required change

### 2.1 `run.rs` (C-9)

Replace the `unwrap()` with a `match`. On `Some(mutation)`, continue as now.
On `None`, return at once:

```rust
(
    Outcome {
        document: verified_document(prepared, target.as_ref(), vec![], vec![]),
        status: ExitStatus::Internal,
        note: Some("core verified the edit but gave no mutation result".to_string()),
    },
    artifacts,
)
```

(Compute `target` before the match, as now.) Update the comment above it: the
invariant is still pinned by `a_verified_run_is_verified_by_core`, and a
broken one is `Internal` rather than a panic.

### 2.2 `result.rs` (C-12)

Add two fields to `ImportedCdsView`, right after `id`:

```rust
    pub intervals: Vec<(u64, u64)>,
    pub strand: &'static str,
```

Fill them in `imported_cds_view` exactly as `cds_view` fills its own:
the intervals from `entry.cds().intervals()` in core's order as
`(start, end)`, and `strand_symbol(entry.cds().strand())`. Update the
struct's doc comment.

## 3. Tests (write first where they can fail; confirm)

1. **C-12,** `crates/cli/tests/result.rs`,
   `a_genbank_template_lists_its_imported_and_unused_features`: add
   assertions that the imported `tet` has intervals `[(85, 1276)]` and
   strand `"+"`, and `bla` has `[(3292, 4153)]` and `"-"`. These values are
   `tests/fixtures/genbank/pbr322-cds.tsv`'s `start`, `end` and `strand`
   columns for those rows, which Biopython verified. Add them as separate
   assertions; leave the existing tuple assertion unchanged.
2. **C-6,** `crates/cli/tests/write_edited.rs`, following its style and
   scratch-directory helper: create a regular file `afile` in the scratch
   directory, then run a request that the existing tests show exits 0 or 7,
   with `--write-edited <scratch>/afile/x`. Assert:
   - exit status 1;
   - stdout parses as JSON with `"state": "verified"`;
   - stderr starts with `luria: `.
   This describes current behavior and should pass without a code change;
   if it doesn't, stop and report.
3. **C-9** has no test: the branch is reachable only if core breaks its
   invariant. Say so in the handoff.

If any other existing test fails to compile because of the new
`ImportedCdsView` fields, add the two fields to its struct literal with the
values that test's CDS has, and list the test in the handoff. Change nothing
else in it.

## 4. Do not

- Add dependencies.
- Change any other file, error code or exit status.
- Change `cds_view` or the target's shape.

## 5. Checks

From the repository root, after `export TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 6. Handoff

Report the changed files, each new test and whether it failed before the
change, the check results, and any deviation.
