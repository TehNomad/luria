# Withdrawn: worktree safety and `luria commit` (this file was labelled 8a2)

Status: **withdrawn 2026-10-24, before a single line of it was committed.**
Planner: Claude. Worker: local model (`w8a2`, assignment resolved).
This was `docs/tasks/contracts/milestone-8a2-worktree-safety.md`, committed as `a785e24`
and delegated to a worker, which implemented it. It is being withdrawn on the
owner's decision after review. The file is renamed so it can never again be read
as the plan's 8a2 row.

## Why it is withdrawn

**The contract invented scope that no document asks for.** Searching the whole
`docs/` tree, excluding this contract itself:

- `constructs.json`, `constructs/`, `luria commit` and `luria construct`:
  **no occurrences anywhere.**
- `docs/tasks/contracts/milestone-8-view.md` §3 lists saving as out of scope:
  *"Placing edits, designing, **saving**, `luria DESIGN.json`, the start screen
  for a bare `luria` (milestone 9)."*
- The plan's actual **8a2** row is a fixture contract: *"`tests/fixtures/view/`:
  the §5.1 column tables for the layout, same scripts and README"* — which is
  still unwritten, and is what 8a2 means.
- The parent sections the contract cited (§1 step 1, §3.1) do not exist, and
  neither §1 nor §3 mentions work trees, git, or a construct record.

Citations that were wrong in the same way:

| Cited for | What the source actually says |
| --- | --- |
| D022 — the list of forbidden git commands | "Reproducibility precedes persistence infrastructure"; says nothing about git |
| D042 — design failure exits 7 | "Every imported GenBank CDS starts with ATG". The design-failure/exit-7 rule is in **D041**'s bullets |
| 7c1's `commit_message` | No such function in `crates/` or `docs/`. 7c1 is the text of the overview, problem and provenance screens |

The worker found all of this by grepping and said so; the failure is entirely
the planner's, and it is the reason the contract-first rule exists.

## What the worker built, and how good it was

`crates/cli/src/worktree.rs` (357 lines), `crates/cli/tests/worktree_safety.rs`
(413 lines, 10 contract-named tests plus one), one module line, one `Command`
variant with a parse arm and a USAGE line, and `emit_commit`. All checks the
worker ran passed, and the planner re-ran the suite for itself:

```
cargo test -p luria-cli --test worktree_safety  →  ok. 11 passed; 0 failed
```

It also ran the six disable-the-check experiments the contract demanded,
reverting `worktree.rs` byte-identically between rounds, and each experiment
did break the test it was supposed to. Nothing here is a criticism of the
worker's execution; it implemented a contract that should not have been written.

## Measured git behaviour (git 2.56.0), worth keeping for milestone 9

Both parties measured these; the planner repeated them in `target/tmp` on
2026-10-24.

* `git -C d rev-parse --verify HEAD` in a work tree with **no commits**: exit
  **128**, `fatal: Needed a single revision` — not exit 1 with no output, which
  is what the contract claimed. Detection must use "did it succeed", never the
  wording.
* `git status --porcelain` in such a tree already lists every file as `??`, so a
  dirty check reports that case on its own.
* `git ls-files --error-unmatch -- constructs.json` exits 1 when the file is not
  tracked, and the same for `constructs/` when the directory exists on disk with
  nothing tracked under it.
* `git diff --quiet HEAD -- constructs/` exits **0** when `constructs/` is
  entirely untracked. A diff against `HEAD` cannot answer "is anything committed
  under this path"; only `ls-files --error-unmatch` can. A test written to catch
  a wrong choice passes wrongly if it is checked with `diff`.
* **A pathspec commit takes working-tree changes.**
  `git commit -m MSG -- constructs.json constructs/` with an unstaged
  modification under `constructs/` exits **0** and commits it, bypassing the
  index; on a clean tree the same command exits **1** with
  *"nothing to commit, working tree clean"*.
* Therefore the contract's own rules contradicted each other: a check that
  refuses **any** dirty state forbids the only state in which the command does
  anything. This is what ended the experiment, and it is a rule about saving,
  not about git.
* A testing trap worth recording: scratch repositories under `target/tmp/` are
  inside this checkout, so `git -C target/tmp/probe rev-parse
  --is-inside-work-tree` answers **`true`** about luria's own repository. The
  worker's tests set `GIT_CEILING_DIRECTORIES` to an **ancestor** of the scratch
  directory; a ceiling naming the directory itself does not stop discovery.

## The owner's ruling, 2026-10-24

* Discard the code; keep the findings. The five files were reverted or deleted;
  a local copy of the discarded round is in `target/tmp/` as
  `8a2-discarded-2026-10-24.patch`, `8a2-worktree.rs.copy` and
  `8a2-worktree_safety.rs.copy`. Those are scratch, not tracked, and will not
  survive a clean checkout.
* When saving does land, the git step's dirty rule is: **refuse only for changes
  outside the saved paths**, and let the saved paths' own working-tree changes
  be what the commit takes.

Milestone 9 still has to decide what the saved record is, where it lives, and
what the commit message says. Nothing here should be read as those answers.
