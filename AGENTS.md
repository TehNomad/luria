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
  user, and reviews diffs and check results. The planner does not implement.
- The worker (local model subagent) is the default and expected implementer
  of all source and test code. It wrote most of milestones 1–2. Code quality
  is established by tests, clippy, fixtures and planner review, not by who
  wrote the code.
- Workers cannot ask the user. If blocked or uncertain, stop and report to
  the planner instead of guessing.

## Delegation rules
- The planner does not edit files under `crates/`, `src/` or `tests/`.
  Allowed direct edits: `docs/tasks/` and a one-line typo or formatting fix.
- All other implementation goes to the `worker` subagent.
- Do not skip delegation because the task seems small or the worker's style
  differs from what the planner would write. Style differences are not
  defects; failing checks, wrong behavior and scope violations are.
- If worker output fails review or checks, `resume` the same worker with
  concrete feedback: file, line, what is wrong, what is expected.
- Worker runs are capped at 80 turns by default. If runs often end with
  the turn limit reached, pass a higher `max_turns` on the `subagent` call
  for large contracts instead of resuming, and tell the user so the
  default can be revisited.
- The planner may take over a fix only after 3 failed rounds on the same
  issue, and must tell the user when it does and why.
- Tests and fixtures come first where practical: the planner writes or
  approves them, then the worker makes them pass. Fixture expectations still
  need independent verification (see Working method).

## Writing contracts for the worker
- Name exact files, function signatures, types, and edge cases.
- List the tests that must pass and the commands to run.
- Include a "Do not" list (no new dependencies, no changes outside the named
  files, no changes to existing tests or fixtures).
- Prefer several small contracts, one function or module each, over one
  large one.
- Give the worker the relevant `docs/decisions.md` sections inline or by
  exact heading, not "see the docs".

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
