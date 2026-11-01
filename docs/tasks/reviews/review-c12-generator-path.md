# Review fix C12: the reference generator hard-codes one machine's path

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review. Regenerated designs.json differs only in the three `reportedBy` strings.
Planner: Claude. Parent: `docs/tasks/reviews/review-post-m4.md` finding C12.

## 1. The defect

`tests/fixtures/q5-primer/reference/neb-q5-sdm-reference.py`, line 40, sets
`FIXTURES = "/home/timw/Projects/luria/tests/fixtures"`. The script runs only
on one machine, and that absolute path ends up in three `reportedBy` strings
in `reference/designs.json`. The four `native*.py` scripts in the same
directory already derive their paths from `__file__`.

## 2. Required changes

1. **The script.** Derive `FIXTURES` the way `native-design.py` does: `HERE`
   is the script's directory, and `FIXTURES` is two levels up, i.e.
   `tests/fixtures`. Check this against the existing uses
   (`{FIXTURES}/example-construct/...`, `{FIXTURES}/q5-primer/reference/...`).
   Add `import os` if needed.
2. **The `reportedBy` value** (around line 488). Make it independent of the
   machine: the path relative to `tests/fixtures`, i.e.
   `"example-construct/raw/sdmsummary.txt (row D404N)"`. Keep building the
   absolute path for reading the file, if the script reads it.
3. **Regenerate.**
   - Copy the repository's `tests/fixtures` tree to a scratch directory under
     `target/c12-scratch/` (git-ignored; the sandbox denies read-write access
     to `/tmp`). Delete the scratch directory when done.
   - Run the modified script there with `python3 -I`. Do not run it inside
     the repo first.
   - Diff the generated `designs.json` against the committed one. The
     **only** differences allowed are the three `reportedBy` strings, which
     lose the `/home/timw/Projects/luria/tests/fixtures/` prefix.
   - If anything else differs, or the script needs a package that is not
     installed, stop and report.
   - Only when the diff is exactly those three strings, copy the generated
     `designs.json` into the repo.
4. **`crates/core/tests/q5_primer_fixtures.rs`.** If any test reads
   `reportedBy` and expects the absolute path, stop and report. Do not
   change that test.
5. **`tests/fixtures/q5-primer/README.md`.** If it mentions the absolute path
   or says the script runs only on one machine, update that sentence. If it
   doesn't, leave the README alone.

## 3. Do not

- Change the script's algorithm, inputs or output format, or any other
  generator.
- Change any other fixture file, or any value in `designs.json` other than
  the three strings.
- Add a check mode or a requirements file. That is owner question A7,
  deferred.

## 4. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

## 5. Handoff

Report the changed files, the exact `designs.json` diff, the command used to
regenerate it, the check results, and any deviation.
