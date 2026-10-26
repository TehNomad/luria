# Molecular biology toolbox: agent instructions

Shared rules for every agent on `luria`. The planner's rules are in
`CLAUDE.md`; the worker's are in `docs/agents/worker.md`.

## Roles
- The planner (Claude) writes task contracts, reviews and commits. The
  worker (pi on a local model) implements all source and test code.
- Workers cannot ask the user. If blocked or uncertain, stop and report to
  the planner instead of guessing.

## Current task
- The task contract is the file named in the prompt, normally
  `docs/tasks/milestone-*.md`. Its scope is binding; do not implement
  anything outside it.
- Files in `docs/tasks/` marked as implemented are historical records, not
  instructions.

## Working method
- Implement one agreed task contract at a time.
- Do not expand scope or change biological assumptions without asking.
- Prefer small, testable changes to speculative framework construction.
- Treat `docs/decisions.md` and `tests/fixtures/README.md` as requirements.
  Read only the sections the task contract names or that the code you touch
  depends on; do not skim the whole file. `scripts/decision` lists every
  decision; `scripts/decision D052` prints one entry.
- If coordinates, strand, CDS, numbering, or expected biology are
  ambiguous, stop and report the ambiguity rather than choosing silently.
- Do not edit `AGENTS.md`, `CLAUDE.md`, `docs/agents/`, `docs/decisions.md`,
  task contracts, or existing fixture expectations unless the contract says
  to. Never change a requirement or an expected result to make a test
  pass; if a test seems wrong, report it.
- Never generate fixture expectations by running the implementation and
  copying its output. Expectations need independent verification, recorded
  in `tests/fixtures/README.md`.

## Temporary files
- Under the nono sandbox, `/tmp` is write-only. Put scratch files in
  `target/tmp/` (gitignored); create it with `mkdir -p target/tmp`.
- For other permission errors, run `nono why` before trying workarounds.

## Git
- Read-only commands are fine (`git status`, `git diff`, `git log`).
- Only the planner stages and commits. Nobody runs push, amend, reset,
  rebase, merge, checkout/switch of branches, force operations, or changes
  git config.

## Required checks
Run after every implementation task, from the repository root:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

`scripts/check` runs all five and prints a one-line result per check, with
the tail of any failure (`-v` for full output). The task contract may add
further checks.

## Handoff
Report changed files, exact checks run and their results, assumptions,
unverified behavior, and any deviations from the task contract.
Do not describe a check as passing unless you ran it successfully.
