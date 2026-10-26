# Molecular biology toolbox: agent instructions

Shared rules for every agent on this project. The planner's own rules are in
`CLAUDE.md`; the worker's are in `docs/agents/worker.md`.

## Naming
- The project name is `luria`, styled in lowercase.
- Use “molecular-biology toolbox” as a descriptive phrase, not a second name.
- Use `luria` for the eventual user-facing executable.
- Prefer `luria-<component>` for Rust package names when components are added.
- Do not rename the project or introduce branding without approval.

## Product boundary
Build a standalone Rust application. Claude and pi are development tools,
not runtime dependencies. Rust owns domain models, canonical
sequence semantics, validation, and orchestration. Future Python workers
perform selected scientific computations behind a versioned
request/response interface. The LLM may parse requests into typed
operations; it must not invent sequences, primer designs, coordinates, or
experimental results.

## Roles
- The planner (Claude) writes task contracts, resolves ambiguity with the
  user, reviews diffs and commits. It does not implement.
- The worker (pi on a local model) implements all source and test code.
  Code quality is established by tests, clippy, fixtures and planner
  review, not by who wrote the code.
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
- Do not edit `AGENTS.md`, `CLAUDE.md`, `docs/agents/`, `docs/decisions.md`,
  task contracts, or existing fixture expectations unless the contract says
  to. Never change a requirement or an expected result to make a test pass.
- Never generate fixture expectations by running the implementation and
  copying its output. Expectations need independent verification, recorded
  in `tests/fixtures/README.md`.

## Temporary files
- Under the nono sandbox, `/tmp` is write-only: files can be created there
  but not read back. Do not use it.
- Put scratch files in `target/tmp/` (gitignored, readable and writable).
  Create it with `mkdir -p target/tmp`.
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

The task contract may add further checks. Report exact commands and results.

## Handoff
Report changed files, exact checks run and their results, assumptions,
unverified behavior, and any deviations from the task contract.
Do not describe a check as passing unless you ran it successfully.
