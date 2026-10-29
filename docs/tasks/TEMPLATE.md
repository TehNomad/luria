# Contract template

Template for task contracts, not a contract. Copy it to
`docs/tasks/milestone-<n><letter>-<slug>.md`, fill each section, and delete
the guidance in brackets. Aim for 150 lines or fewer, not counting the code
block in §4 and the tables in §5 and §7. If a contract needs more, split it.

What keeps a contract short:
- Cite, don't restate. A rule that lives in `docs/decisions.md`, a plan or a
  fixture README is named by D-number or section, with one line saying what
  it settles here. The worker reads it with `scripts/decision D0nn`.
- `AGENTS.md` and `docs/agents/worker.md` already apply. Do not repeat them:
  no staging or committing, no editing fixtures, decisions or other people's
  tests, stop on ambiguity, the five checks, the handoff format.
- Background research goes to a scout first. Put its findings in the contract
  only as file:line references the worker needs.

---

# Contract <id> — <crate>: <what it builds>

Planner: Claude. Status: draft. Tier: <full | presentation (D051)>.
Plan row: <plan file and section>.

## 1. What this contract builds

[Two to five sentences: the module or function and the behavior, in the
domain's terms.]

Rules that apply, already settled:
- D0nn — [what it settles for this contract, one line]
- <plan file> §x.y — [one line]
- `tests/fixtures/<dir>/README.md`, section "<heading>" — [one line]

## 2. What exists and must be reused

| Item | Where (`file:line`) | Use here |
| --- | --- | --- |

## 3. Files the worker may touch

1. `<path>` — new | change: [exactly what may change]

Everything else is out of scope.

## 4. The public surface, exactly

```rust
// signatures, types and doc-comment requirements
```

[Edge cases and errors as a list: input, then the required result.]

## 5. Tests the worker writes

| # | Test (`file::name`) | Asserts |
| --- | --- | --- |

## 6. Checks

`scripts/check`, plus:
- [contract-specific commands only, with their required output]

## 7. Proving the tests bite

Make each change once, run the named test, quote the failure, restore the
file byte-identically, and report.

| # | Change | Must fail |
| --- | --- | --- |

## 8. Do not

[Only what is specific to this contract: files or items that look in scope
but are not, behaviors that must not change, tempting shortcuts.]

## 9. Report

[Only what this contract needs beyond the standard handoff, e.g. "every
guard experiment with its failure line" or "stop and report if X does not
fit §4".]
