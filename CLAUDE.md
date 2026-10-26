# Planner instructions

@AGENTS.md

These rules are for the planner (Claude Code). Workers do not read this
file; they get `AGENTS.md` and `docs/agents/worker.md`.

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
- Name exact files, function signatures, types, and edge cases.
- List the tests that must pass and the commands to run.
- Include a "Do not" list (no new dependencies, no changes outside the named
  files, no changes to existing tests or fixtures).
- Prefer several small contracts, one function or module each, over one
  large one.
- Give the worker the relevant `docs/decisions.md` sections inline or by
  exact heading, not "see the docs".
- For each new rule, name the test that must fail if the rule is disabled,
  and have the worker prove it by disabling the rule once and restoring it
  (a weak assertion can pass with the rule removed).

## Running workers
Workers are pi on `strata/qwen3.8-flash-next`, one herdr pane per contract,
driven with `scripts/pi-worker`. The script runs pi under
`nono wrap --profile luria-worker`. The worker gets no Claude config and no
network except the model server on port 8080, and `--no-context-files`
keeps this file out of its prompt.

1. `scripts/pi-worker start w-<contract>` (e.g. `w-8e`) opens a sibling
   pane and waits until pi is idle.
2. Write the prompt to `target/tmp/prompts/<name>-<round>.md`: the contract
   path and anything the contract leaves to the planner. Then run
   `scripts/pi-worker prompt <name> <file> [minutes]` with
   `run_in_background`. It returns when the worker settles; the default
   wait is 60 minutes.
3. `scripts/pi-worker report <name>` prints the worker's last reply. Use
   `read` only to see the pane, e.g. after a failure.
4. Review the diff and run the required checks yourself. Send feedback to
   the same worker with another `prompt`, which keeps its context.
5. After the commit, `scripts/pi-worker stop <name>`.

If `prompt` exits 3, the worker is waiting at an approval or question
prompt. Read the pane and ask the user; do not answer it yourself. On a
timeout the prompt may still be running: check `read` before sending it
again. Sessions are kept in `target/tmp/pi-sessions/`.

## Git
- The user creates each milestone branch. Commit only on the current
  milestone branch, never on `main`; if on `main`, stop and ask.
- Commit after all required checks pass and the diff has been reviewed.
- One commit per completed task contract, or per review round for large
  contracts.
- Stage files explicitly by path; never use `git add -A` or `git add .`.
- Commit message: a short summary line, then the contract filename and
  the checks that passed.
