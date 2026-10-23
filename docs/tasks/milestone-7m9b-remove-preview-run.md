# Milestone 7m9b: remove the in-memory run

Status: **Implemented, 2026-10-23.** Planner: Claude. Parent:
`docs/tasks/milestone-7m-polish.md`, M7-65. Follows 7m9a (committed
`8499fd8`).

## 1. Why

M7-65: 7m8a's in-memory run goes, because nothing uses it any more. Only
tests call `review::preview`.

## 2. Changes

1. `crates/cli/src/review.rs`: delete `pub fn preview`. Keep `from_run`
   only if `run_approved` still uses it; otherwise put its body back into
   `run_approved`. Either way, `run_approved`'s behaviour is unchanged.
   Remove the imports that are then unused.
2. `crates/cli/src/run.rs`: delete `run_bytes_with_constructs`.
3. `crates/cli/src/prepare.rs`: delete `pub fn prepare_bytes`. `prepare`
   keeps its behaviour. A private helper taking bytes may stay if it
   keeps `prepare` readable, but it must not be `pub`.
4. `crates/cli/tests/review_approve.rs`: delete `a_preview_is_the_saved_run`,
   `a_preview_writes_nothing`, `a_clarification_has_no_preview`, their
   helper `entries_of` if it is then unused, and the module-doc paragraph
   about the preview.
5. `crates/cli/tests/review_preview.rs`: build each test's run with
   `review::approve(&document, &template, &request)` into its own fresh
   workspace, in place of `preview`. The workspace helper already makes a
   fresh directory per test. Fix the module doc's two lines that name
   `review::preview`. The assertions do not change.
6. Remove every comment in `crates/` that names `preview`,
   `prepare_bytes` or `run_bytes_with_constructs` as a function, or
   7m8a's in-memory run. Then `grep -rn "prepare_bytes\|run_bytes_with_constructs\|review::preview" crates`
   finds nothing.

## 3. Tests

No new tests. All remaining tests pass. No assertion changes. The only
test edits are the deletions in §2.4 and the setup change in §2.5.

## 4. Do not

- Change only the five files in §2 and comments under `crates/`.
- No change to any command's behaviour, docs, fixtures, Cargo files or
  `vendor/`.
- Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the grep from §2.6, and the test count before and
after.

## 6. Implementation notes (planner, 2026-10-23)

- 1147 tests passed, 0 failed, 1 ignored (1150 before; 3 deleted). fmt,
  clippy, diff check clean; fixtures 19 of 19. The §2.6 grep finds nothing.
- No assertion changed. Some assertion messages in `review_preview.rs`
  still say "preview"; they were left because §3 forbids changing
  assertions.
- The worker saw `end_to_end.rs::the_origin_crossing_residue_case_matches_its_fixture_row`
  fail once, and pass in every later run. That test uses a fixed
  workspace name (`res-c-origin`), so two test runs at the same time can
  collide in it. Not caused by this change; noted as a possible test
  isolation issue.
