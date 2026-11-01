# Contract 9k — cli: the messages that promise milestone 9

Planner: Claude. Status: implemented. Tier: light (plan M9-10).
Plan row: `docs/tasks/contracts/milestone-9-editor.md` §7, row 9k.

## 1. What this contract builds

Two refusals tell the user that something "comes in milestone 9". Milestone
9 does not build either, so both are reworded to say the feature is not
supported yet (plan §5 M9-1). The behaviour, the exit statuses and the
order of the checks do not change.

## 2. What exists

| Item | Where (`file:line`) |
| --- | --- |
| `RANGE_MESSAGE` and its doc comments | `crates/cli/src/find.rs:8`, `:81`, `:122` |
| The design-file refusal and its comment | `crates/cli/src/args.rs:178`, `:608` |
| Tests pinning them | `crates/cli/tests/open_cli.rs:144-157`, `crates/cli/tests/find_cli.rs:335` |

## 3. Files the worker may touch

The five above. This contract may change those two tests' expected text,
and nothing else in them.

## 4. The new words

- `find.rs:81`: `searching a range of bases is not supported yet`.
- `args.rs:178`: `opening a design is not supported yet; use 'luria review {}'`.
- The comments at `find.rs:8`, `:122` and `args.rs:608` stop naming
  milestone 9. They say the grammar is kept for a later range search, and
  that a design file is refused before the file is read. Keep each
  comment's other content.
- `grep -rn "milestone 9" crates/` finds nothing after the change. Report
  any other hit instead of changing it.

## 5. Tests

The two existing tests, with the new text. No new test.

## 6. Checks

`scripts/check`, and the `grep` above.

## 7. Do not

Change any exit status, check order or other message.
