# Review B12: bump the algorithm versions whose behavior changed

Status: **Implemented 2026-10-16** by the local worker; accepted after adding the 0.6.0 history entry at the planner's request.
Planner: Claude. Parent: `docs/tasks/review-post-m4.md` B12. Decision:
D021, under which every result names its algorithm and algorithm version.

## 1. Defect

Four version constants weren't bumped after their behavior changed:
- **`MUTATION_ALGORITHM_VERSION`** (`mutation.rs`), `"0.2.0"`: deletion,
  insertion and replacement were added in Milestone 4. On this branch, B3,
  B4 and B10 changed it too.
- **`PRIMER_DESIGN_VERSION`** (`primer.rs`), `"0.1.0"`: the tail path came in
  Milestone 4. On this branch, B1 refuses pairs it used to accept, and B5
  and B13 changed its messages.
- **`TM_FUNCTION_VERSION`** (`tm.rs`), `"0.1.0"`: changed by B6, B7, B8 and
  B16 on this branch.
- **`NATIVE_DESIGN_ALGORITHM_VERSION`** (`primer_design.rs`), `"0.5.0"`:
  changed by B2 and A6 on this branch.

`Q5_WORKFLOW_VERSION` was already bumped to `"0.5.0"` on this branch (A5).
That covers B9 too, because the branch hasn't been merged, so leave it.

## 2. Required changes

One minor bump each, covering this whole branch:
- `MUTATION_ALGORITHM_VERSION`: `"0.2.0"` → `"0.3.0"`
- `PRIMER_DESIGN_VERSION`: `"0.1.0"` → `"0.2.0"`
- `TM_FUNCTION_VERSION`: `"0.1.0"` → `"0.2.0"`
- `NATIVE_DESIGN_ALGORITHM_VERSION`: `"0.5.0"` → `"0.6.0"`

Update the one test that pins a version,
`the_design_records_its_inputs_and_what_it_left_unresolved` in
`primer_design.rs` (around line 2444), from `"0.5.0"` to `"0.6.0"`.

## 3. Before editing, check

Grep `crates/`, `tests/` and `docs/` for each old version string next to its
constant's name, and for any test asserting `function_version()`,
`algorithm_version()` or the like. Report anything else that pins one of
these four values. Don't change docs that describe history.

## 4. Do not

- Change any constant other than these four, any name constant, or any
  behavior.
- Change any other test.
- Edit files other than `mutation.rs`, `primer.rs`, `tm.rs` and
  `primer_design.rs`.

## 5. Checks

The usual cargo checks with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 6. Handoff

Report:
- the diff;
- the grep results;
- the check results.
