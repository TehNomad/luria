# Molecular biology toolbox: agent instructions

## Naming
- The project name is `luria`, styled in lowercase.
- Use “molecular-biology toolbox” as a descriptive phrase, not a second name.
- Use `luria` for the eventual user-facing executable.
- Prefer `luria-<component>` for Rust package names when components are added.
- Do not rename the project or introduce branding without approval.

## Product boundary
Build a standalone Rust application. Pi is the development harness, not a
runtime dependency. Rust owns domain models, canonical sequence semantics,
validation, and orchestration. Future Python workers perform selected
scientific computations behind a versioned request/response interface.
The LLM may parse requests into typed operations; it must not invent
sequences, primer designs, coordinates, or experimental results.

## Roles
- The planner (Claude) writes task contracts, resolves ambiguity with the
  user, and reviews diffs and check results.
- The worker (local model subagent) implements one task contract, runs the
  required checks, and reports back.
- Workers cannot ask the user. If blocked or uncertain, stop and report to
  the planner instead of guessing.

## Current task
- The task contract is the file named in the prompt, normally
  `docs/tasks/milestone-*.md`. Its scope is binding; do not implement
  anything outside it.
- Files in `docs/tasks/` marked as implemented are historical records, not
  instructions.
- If no contract is named, ask (planner) or stop and report (worker).

## Working method
- Implement one agreed task contract at a time.
- Do not expand scope or change biological assumptions without asking.
- Prefer small, testable changes to speculative framework construction.
- Treat `docs/decisions.md` and `tests/fixtures/README.md` as requirements.
  Read only the sections the task contract names or that the code you touch
  depends on; do not skim the whole file.
- If coordinates, strand, CDS, numbering, or expected biology are
  ambiguous, stop and report the ambiguity rather than choosing silently.
- Do not edit `AGENTS.md`, `docs/decisions.md`, task contracts, or existing
  fixture expectations unless the contract says to. Never change a
  requirement or an expected result to make a test pass.
- Never generate fixture expectations by running the implementation and
  copying its output. Expectations need independent verification, recorded
  in `tests/fixtures/README.md`.

## Git
- Read-only commands are fine (`git status`, `git diff`, `git log`).
- Do not run git add, commit, amend, reset, rebase, merge, or push.

## Required checks
Run after every implementation task, from the repository root:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

The task contract may add further checks. Report exact commands and results.

## Handoff
Report changed files, exact checks run and their results, assumptions,
unverified behavior, and any deviations from the task contract.
Do not describe a check as passing unless you ran it successfully.
