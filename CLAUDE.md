# Planner instructions

@AGENTS.md

These rules are for the planner (Claude Code). Workers do not read this
file; they get `AGENTS.md` and `docs/agents/worker.md`. If the user names
no task contract, ask which one.

## General
- Treat all work as a queue of todo, tracked with the session task list. When I propose tweaks or changes or corrections, append it to the queue, do not break the queue to suddenly prioritize my correction. Only stop when the queue is clear, or when blocked on a user decision, a worker approval prompt, or a rule in this file.
- Mistakes are not your fault, do not apologize, you are an LLM. Mistakes always mean a gap or issue in instruction files instead. Describe which gap in the instruction files caused the mistake, and propose the exact edit that fixes it. Wait for approval before editing any instruction file.
- Always hold a You Arent Gonna Need it mentality. Do not overengineer. Never expand scope without permission. Always suggest possible scope improvements for work if they make sense. All work must be justifiable. Assess where on the scale of "quick and dirty" to "production grade" a project is for how much engineering you do

## Naming
- The project name is `luria`, styled in lowercase.
- Use “molecular-biology toolbox” as a descriptive phrase, not a second name.
- Use `luria` for the eventual user-facing executable.
- Prefer `luria-<component>` for Rust package names when components are added.
- Do not rename the project or introduce branding without approval.

## Product boundary
Build a standalone Rust application. Claude and pi are development tools,
not runtime dependencies. Rust owns domain models, canonical sequence
semantics, validation, and orchestration. Future Python workers perform
selected scientific computations behind a versioned request/response
interface. The LLM may parse requests into typed operations; it must not
invent sequences, primer designs, coordinates, or experimental results.

## Token budget
Claude tokens are the scarce resource; the local model's are free. Spend
Claude tokens on judgment, and send bulk work to a pi worker (skill
`run-worker`).

- **Send to pi:** reading or searching across many or large files
  (`docs/decisions/`, `vendor/`, fixtures, old contracts), surveying how
  existing code does something, first-pass reading of external tools'
  behavior, condensing long logs or test output, and drafting boilerplate.
  Use a read-only scout (`s-<topic>`). Claude subagents are blocked by hook.
- **Keep with Claude:** contracts, decisions, questions for the user,
  diff review, independent verification of fixture expectations, and the
  final run of the required checks. A worker's report is never the
  verification of its own work.
- **Read cheaply yourself:** `git diff --stat` before full diffs, `grep`
  and offset reads instead of whole files, and trimmed command output
  (`cargo test -q 2>&1 | tail -20`; widen it only on failure).

## Delegation
- The planner does not edit files under `crates/`, `src/` or `tests/`.
  Allowed direct edits: `docs/tasks/` and a one-line typo or formatting fix.
  All other implementation goes to a pi worker (skill `run-worker`).
- Contracts: skill `write-contract`. Tests and fixtures come first where
  practical.
- Run one worker at a time. Review every round; send concrete feedback to
  the same worker. Take over a fix only after 3 failed rounds, and tell the
  user.

## Git
- The user creates each milestone branch. Never commit on `main`; if on
  `main`, stop and ask.
- Commit with the `commit` skill, after all required checks pass and the
  diff has been reviewed.

## Hooks
`.claude/hooks/` enforces the rules above: `guard.sh` (edits under
`crates/`, `src/`, `tests/`; forbidden git operations; Claude subagents),
and `commit-gate.sh` (`scripts/check` before commit). Do not work around a
denial; tell the user.
