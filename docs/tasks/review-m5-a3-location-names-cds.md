# Review M5 A-3: a refused CDS location names its CDS

Status: **Contract, 2026-10-18.** Planner: Claude. Parent:
`docs/tasks/review-post-m5.md` A-3. Requirement: D041, under which a named CDS
that can't be represented fails the run "with that CDS's own error". Amends
`docs/tasks/milestone-5c3-genbank-cds.md` §4.2 step 8 ("passed through"),
which stays a historical record: do not edit it.

Files:
- `crates/core/src/error.rs`
- `crates/core/src/genbank_location.rs`
- `crates/core/src/genbank_cds.rs`
- `crates/cli/src/result.rs`
- `crates/cli/tests/error_details.rs` (only as §3 allows)

## 1. Defect

`import_cds` passes `parse_genbank_location`'s `UnusableGenBankLocation`
through unchanged. Its payload is `{location, rule, sequence_length}`, with no
CDS id and no line. When a request names several CDSs, the user can't tell
which one failed. The other errors a named CDS can fail with
(`UnusableGenBankCds`, `GenBankTranslationMismatch`, and the `Cds::create`
errors) already name it.

## 2. Required change

### 2.1 `error.rs`

Add two fields to `UnusableGenBankLocationProblem`, after `sequence_length`:

```rust
    pub cds: Option<String>,
    pub line: Option<u64>,
```

Doc comment: `cds` is the id text of the CDS feature whose location this is,
and `line` its key line in the GenBank text, counted from 1. Both are `None`
when the location was read on its own, outside a record import.

### 2.2 `genbank_location.rs`

Every construction of the problem sets `cds: None, line: None`. Nothing else
changes.

### 2.3 `genbank_cds.rs`, `evaluate` step 8

Map the error from `parse_genbank_location`: an
`UnusableGenBankLocation(problem)` becomes the same variant with
`cds: Some(<the id text step 1 found>)` and `line: Some(<the feature's key
line>)`, the same values the `UnusableGenBankCds` refusals in this function
use. Any other error passes through unchanged. Update the module doc's step
8 to say the refusal is given the CDS's id and key line.

### 2.4 `result.rs`, the details of `UnusableGenBankLocation`

After `sequenceLength`, add `("cds", Text or Absent)` and
`("line", Count or Absent)`, following the pattern the `MalformedGenBank`
arm uses for its `line`.

## 3. Tests (write first; confirm they fail before the change)

1. In `genbank_cds.rs`'s test module: a record (`record` helper, a 12-base
   linear sequence) with `key("CDS", "<1..12")`, `q("/gene=\"g\"")` and a
   second feature `key("CDS", "1..12")`, `q("/gene=\"h\"")`. Importing
   `["h", "g"]` fails with `UnusableGenBankLocation` whose `rule` is
   `FuzzyEnd`, `cds` is `Some("g")`, and `line` is `Some(3)` (LOCUS is line
   1, FEATURES 2, the first key line 3).
2. In `genbank_location.rs`'s test module: a refused location read directly
   with `parse_genbank_location` has `cds: None` and `line: None`.
3. In `crates/cli/tests/error_details.rs`, the existing test that builds an
   `UnusableGenBankLocationProblem` (around line 453): add the two new fields
   to the struct literal, with `cds: Some("g".to_string())` and
   `line: Some(3)`, and extend its expected details with `"cds": "g"` and
   `"line": 3` in that order after `sequenceLength`. If that test is shaped
   so this can't be done in that way, stop and report. Add one more case
   with both `None`, whose details carry `"cds": null` and `"line": null`, if
   the test's style supports several cases; otherwise report.

If any other existing test fails to compile because of the new fields, add
`cds: None, line: None` to its struct literal and nothing else, and list it
in the handoff.

## 4. Do not

- Add dependencies.
- Change the error's code, category or rule.
- Change the not-used list's format.
- Change any file not listed above, except struct-literal fixes as §3 allows.

## 5. Checks

From the repository root, after `export TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 6. Handoff

Report the changed files, each new test and whether it failed before the
change, every existing test touched and why, the check results, and any
deviation.
