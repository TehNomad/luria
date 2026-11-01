# Planner instructions

@AGENTS.md

These rules are for the planner (Claude Code). Workers do not read this
file; they get `AGENTS.md` and `docs/agents/worker.md`. If the user names
no task contract, ask which one.

## General
- Treat all work as a queue of todo, tracked with the session task list. When I propose tweaks or changes or corrections, append it to the queue, do not break the queue to suddenly prioritize my correction. Only stop when the queue is clear, or when blocked on a user decision, a worker approval prompt, or a rule in this file.
- Mistakes are not your fault, do not apologize, you are an LLM. Mistakes always mean a gap or issue in instruction files instead. Describe which gap in the instruction files caused the mistake, and propose the exact edit that fixes it. Wait for approval before editing any instruction file.
- Always hold a You Arent Gonna Need it mentality. Do not overengineer. Never expand scope without permission. Always suggest possible scope improvements for work if they make sense. All work must be justifiable. Assess where on the scale of "quick and dirty" to "production grade" a project is for how much engineering you do

## Communication style
Keep responses brief and to the point, using Simplified Technical English (STE), formalized as ASD-STE100. Prefer active voice. Do not say "Now I'll build to check that it compiles", say "Checking that it compiles", for example. You may use "-ing" verbs despite what ASD-STE100 says.

After each commit, give a biology summary of 5 lines or fewer. Cover:
- what the change does to sequences, coordinates or strands;
- which biological assumptions it relies on (cite D-numbers);
- what the user should verify.

Use molecular-biology terms, not Rust terms. Build the summary from the
contract and the worker's handoff report; do not re-read the full diff for
it. Give a longer explanation only when asked.

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
Claude tokens on judgment, and send bulk work to a pi worker.

- **Send to pi:** reading or searching across many or large files
  (`docs/decisions/`, `vendor/`, fixtures, old contracts), surveying how
  existing code does something, first-pass reading of external tools'
  behavior, condensing long logs or test output, and drafting boilerplate.
  Use a read-only scout (`s-<topic>`) whose prompt says the task is
  read-only. Do not use Claude subagents (Explore, general-purpose) for
  this.
- **Keep with Claude:** contracts, decisions, questions for the user,
  diff review, independent verification of fixture expectations, and the
  final run of the required checks. A worker's report is never the
  verification of its own work.
- **Treat scout output as leads, not facts.** Ask for `file:line`
  references and spot-check the claims a decision rests on.
- **Read cheaply yourself:** `git diff --stat` before full diffs, `grep`
  and offset reads instead of whole files, and trimmed command output
  (`cargo test -q 2>&1 | tail -20`; widen it only on failure).

## Delegation rules
- The planner does not edit files under `crates/`, `src/` or `tests/`.
  Allowed direct edits: `docs/tasks/` and a one-line typo or formatting fix.
- All other implementation goes to a pi worker (see Running workers).
- Do not skip delegation because the task seems small or the worker's style
  differs from what the planner would write. Style differences are not
  defects; failing checks, wrong behavior and scope violations are.
- If worker output fails review or checks, send concrete feedback to the
  same worker: file, line, what is wrong, what is expected.
- The planner may take over a fix only after 3 failed rounds on the same
  issue, and must tell the user when it does and why.
- Tests and fixtures come first where practical: the planner writes or
  approves them, then the worker makes them pass. Fixture expectations still
  need independent verification (see Working method in `AGENTS.md`).

## Writing contracts for the worker
- Start from `docs/tasks/TEMPLATE.md`. Aim for 150 lines or fewer, not
  counting the public-surface code and the test tables; split rather than
  exceed it. Cite decisions by D-number instead of restating them, and do
  not repeat rules `AGENTS.md` or `worker.md` already state.
- Name exact files, function signatures, types, and edge cases.
- List the tests that must pass and the commands to run.
- Include a "Do not" list (no new dependencies, no changes outside the named
  files, no changes to existing tests or fixtures).
- Prefer several small contracts, one function or module each, over one
  large one. Contracts run one after another (one worker at a time), so
  order them so each builds on the last.
- Give the worker the relevant `docs/decisions/` entries by D-number, not
  "see the docs".
- New contracts go in `docs/tasks/contracts/` and start with OKF frontmatter
  (`type: Task Contract`, `title`, `description`, `status: draft`).
- For each new rule, name the test that must fail if the rule is disabled,
  and have the worker prove it by disabling the rule once and restoring it
  (a weak assertion can pass with the rule removed).
- When a rule changes existing behavior, grep the tests for assertions of
  the old behavior, and name each one in the contract's file list as a
  permitted change.

## Running workers
Workers are pi on the local model set in `scripts/pi-worker`, one herdr tab
per contract, driven with `scripts/pi-worker`. The script runs pi under
`nono wrap --profile luria-worker`. The worker gets no Claude config and no
network except the local model server (`LLM_PORT` in the script), and
`--no-context-files` keeps this file out of its prompt.

The local model server serves one request stream at a time, so run **one
worker at a time**. Do not start a second worker, or send a second `prompt`,
while another worker is running. Run contracts and review rounds
sequentially: finish and review one (and `stop` its tab) before starting
the next. Scouts (`s-<topic>`) count as workers. Use planner-side reading
(`grep`, offset reads) while a worker runs, not another pi session.

1. `scripts/pi-worker start w-<contract>` (e.g. `w-8e`) opens a new tab
   (unfocused) and waits until pi is idle.
2. Write the prompt to `target/tmp/prompts/<name>-<round>.md`: the contract
   path and anything the contract leaves to the planner. Then run
   `scripts/pi-worker prompt <name> <file> [minutes]` with
   `run_in_background`. It snapshots the working tree first, then returns
   when the worker settles; the default wait is 60 minutes.
3. `scripts/pi-worker report <name>` prints the worker's last reply. Use
   `read` only to see the pane, e.g. after a failure.
4. Review with `scripts/pi-worker diff <name> --stat`, then the full diff
   of the files that matter. It shows only what changed since the last
   prompt, untracked files included. Run `scripts/check` yourself. Send
   feedback to the same worker with another `prompt`, which keeps its
   context.
5. After the commit, `scripts/pi-worker stop <name>`.

If `prompt` exits 3, the worker is waiting at an approval or question
prompt. Read the pane and ask the user; do not answer it yourself. On a
timeout the prompt may still be running: check `read` before sending it
again. Sessions and `snapshots.log` are kept in `target/tmp/pi-sessions/`.

To undo a worker round, ask the user first, then restore paths from the
round's snapshot: `git restore --source=<tree> -- <path>`. Files the worker
created are not removed by this; list them from `diff --stat`.

## Hooks
`.claude/hooks/guard.sh` runs before every Edit, Write and Bash call. It asks
before edits under `crates/`, `src/` or `tests/`, and denies the git
operations forbidden in Git below and in `AGENTS.md`. It matches command
text, so a commit message or heredoc that mentions a forbidden command is
denied too: write the message to a file and use `git commit -F`, and edit
scripts with the Edit tool. Do not work around a denial; tell the user.

## Git
- The user creates each milestone branch. Commit only on the current
  milestone branch, never on `main`; if on `main`, stop and ask.
- Commit after all required checks pass and the diff has been reviewed.
- One commit per completed task contract, or per review round for large
  contracts.
- Stage files explicitly by path; never use `git add -A` or `git add .`.
- Commit message: a short summary line, then the contract filename and
  the checks that passed.
