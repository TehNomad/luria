# Contract 8a2 — worktree safety: `luria commit`

Status: **issued 2026-10-24**. Planner: Claude. Worker: local model.
Parent: `docs/tasks/milestone-8-view.md` §1 step 1 and §3.1, full tier (D051).
This is the first slice of contract 8a; the record opening, digest pinning, CDS
selection and `luria construct` come in 8a3. Nothing here depends on the view.

## What and why

`luria construct` refuses to run outside a work tree and refuses one with a
dirty working tree (plan §1 step 1, D022). A person trying it in an exported
directory, or with an unrelated uncommitted change, currently gets a
file-not-found on `constructs.json`. This contract builds the git plumbing once,
behind the cli boundary, and gives it its own command so the rule can be tested
and demonstrated before the view exists.

The design failure is deliberately not a core error. `worktree_dirty` and
`construct_path_taken` are facts about the checkout and the working directory,
which core does not know (D042: core gains no `worktree_dirty` variant).

## Requirements

1. **`luria commit`** is a new subcommand with no options. It commits the
   work tree's tracked changes with `git commit -m <message> -- <pathspec>`,
   where the pathspec is exactly `constructs.json` and `constructs/`. In 8a2 the
   message is fixed to `luria: update constructs`; 8a3 replaces it with the one
   7c1's `commit_message` builds from the design. Passing `--` before the
   pathspec is load-bearing: without it a pathspec that names a branch is taken
   as a revision and commits something else.
2. **Work-tree detection.** Before anything else, `git -C <dir> rev-parse
   --is-inside-work-tree` must print `true`, where `<dir>` is the directory
   `constructs.json` is read from (the current directory). Non-zero exit, empty
   stdout, or output that is not `true` is a design failure naming the
   directory and telling the user to run it inside a work tree.
3. **Dirty check.** `git -C <dir> status --porcelain` output must be empty. Any
   line is a design failure. The two checks are separate commands with separate
   failures: "not a work tree" and "work tree is dirty" are different problems
   and the messages must not merge them.
4. **Tracked construct paths.** `git -C <dir> ls-files --error-unmatch --
   constructs.json` and `git -C <dir> ls-files --error-unmatch -- constructs/`
   must both succeed; each failing one is a design failure naming that path.
   `git diff --quiet HEAD -- constructs/` is then used to prove the committed
   files under `constructs/` are not modified in the working tree. **Use
   `ls-files`, not `diff`, for the tracked question, and pass an explicit
   `HEAD` to `diff`; these git behaviours were measured on this machine, do not
   re-derive them:**
   - `git ls-files --error-unmatch -- constructs.json` in a work tree with
     commits but no such tracked file: exit 1, `error: pathspec ... did not
     match any file(s) known to git`.
   - the same for `constructs/` when the directory exists on disk but nothing
     under it is tracked: exit 1.
   - `git diff --quiet HEAD -- constructs/` in that same state: **exit 0**. A
     diff against `HEAD` says nothing about untracked paths, so it cannot answer
     "is anything committed here"; a rule that relied on it would pass the case
     test 6 exists to catch.
   - `git diff --quiet HEAD -- constructs/` in a work tree with **no commits
     yet**: exit 128, `fatal: bad revision 'HEAD'`. Do not map that to a git
     fault; requirement 4a detects it first.
   - `git status --porcelain` in that same no-commit work tree lists the files
     as `??`, so requirement 3 already reports it as dirty.
   4a. **Commits exist.** Before the diff, `git -C <dir> rev-parse --verify
   HEAD` must succeed; exit 1 with no output means the branch is unborn. That is
   a design failure saying luria needs at least one commit in this work tree, and
   it is checked by the exit status, not by matching git's stderr — the wording
   of that message is not stable across git versions.
5. **Nothing destructive.** `luria commit` never runs `push`, `pull`, `fetch`,
   `checkout`, `restore`, `reset`, `clean`, `stash`, `rebase`, `merge`,
   `switch`, `clone`, `rm`, `mv`, `branch`, `tag`, `gc`, `init` or `--force`
   (D022). A test greps the recorded commands and refuses any of them.
6. **The same plumbing serves `luria construct`.** The functions below are
   public within the crate so 8a3's construct flow calls them; 8a2 must not
   invent a second copy of the checks.

## Error codes

| Outcome | stderr | exit |
| --- | --- | --- |
| not a work tree, or dirty, or an untracked construct path | one or more `luria:` lines naming the problem and the directory, no JSON | 7 (D042's design-failure code) |
| `luria commit` given an argument it does not take | usage text on stderr, no JSON | 2 |
| `git` is missing, or too old to answer `rev-parse --is-inside-work-tree` | one `luria:` line naming what is missing and what luria needs | 6 (`ExitStatus::Unsupported`: luria cannot do the job in this environment — it is not the user's command line that is wrong, and it is not an internal fault) |
| a `git` call failing unexpectedly after the checks pass | `luria:` line with git's stderr | 1 |

Exit 7 is `ExitStatus::PartialDesign`; that variant's doc comment is about
design, but D042 defines 7 as *any* run ending in a design failure. Name that in
a comment where it is returned; `crates/cli/src/exit.rs` is not touched.

## Files

- `crates/cli/src/worktree.rs` (new) — all of this contract's code.
  Suggested surface, exact names:
  - `pub struct Git { dir: PathBuf, calls: Vec<String> }` with
    `pub fn new(dir: &Path) -> Git`.
  - `pub fn inside_work_tree(&self) -> Result<bool, GitProblem>`,
    `pub fn has_commits(&self) -> Result<bool, GitProblem>` (`rev-parse --verify
    HEAD`), `pub fn status_porcelain(&self) -> Result<String, GitProblem>`,
    `pub fn is_tracked(&self, path: &str) -> Result<bool, GitProblem>`,
    `pub fn diff_against_head_is_empty(&self, pathspec: &str) -> Result<bool, GitProblem>`,
    `pub fn commit(&self, message: &str, pathspec: &[&str]) -> Result<String, GitProblem>`,
    and `pub fn calls(&self) -> &[String]` for the test that greps commands.
  - `pub enum GitProblem { NotAWorkTree { dir: String }, NoCommits { dir: String },
    WorktreeDirty { changes: String }, NotTracked { path: String },
    ModifiedUnderHead { path: String }, Missing { program: String },
    Failed { command: String, reason: String } }`
    with `pub fn message(&self) -> String`.
  - `pub fn check(git: &Git) -> Result<(), GitProblem>` — requirements 2, 3, 4a
    and 4 in that order, stopping at the first problem.
  - `pub fn run(worktree_root: &Path, message: &str) -> ExitStatus` — the body
    of `luria commit`, in the style of `review::run`: it writes its own
    diagnostics and never panics.
- `crates/cli/src/lib.rs` — one `pub mod worktree;` line, alphabetically.
- `crates/cli/src/args.rs` — one `Commit` variant on `Command` plus its parse
  arm, in the style of the other subcommands; `USAGE` grows one line.
- `crates/cli/src/main.rs` — one `Ok(Command::Commit) => emit_commit(),` arm and
  an `emit_commit` that calls `worktree::run`.
- `crates/cli/tests/worktree_safety.rs` (new) — the tests below. It creates its
  own repository with `git init` in a temporary directory under
  `target/tmp/`, and every git call records the argv it was given.

## Tests

Name them exactly:

1. `a_clean_work_tree_passes_the_checks` — `git init`, commit one file, then
   `check` succeeds and `calls()` shows `status --porcelain` was run.
2. `an_untracked_file_is_dirty` — add an untracked file to a clean work tree:
   `check` fails with `WorktreeDirty`, and the message lists the file.
3. `a_modified_tracked_file_is_dirty` — modify a committed file: same failure.
4. `a_directory_that_is_not_a_work_tree_is_refused` — a fresh temp directory
   with no `.git`: `check` fails with `NotAWorkTree`.
5. `a_construct_path_that_is_untracked_is_refused` — a clean work tree that has
   a commit but no `constructs.json`: `check` fails with `NotTracked` naming
   `constructs.json`.
6. `a_construct_directory_with_no_committed_files_is_refused` — `constructs.json`
   committed, `constructs/` existing on disk but containing nothing committed:
   `check` fails with `NotTracked` naming `constructs/`, not `constructs.json`.
   This test is what requirement 4's `diff` warning is about: an implementation
   that checks the directory with `git diff --quiet HEAD -- constructs/` passes
   it wrongly and must fail this test. Prove that by making the check use `diff`
   once, watching test 6 fail, then restoring `ls-files`.
6a. `a_work_tree_with_no_commits_yet_is_refused` — `git init` and nothing
   committed: `check` fails with `NoCommits`, and the message says at least one
   commit is needed. The failure must come from `rev-parse --verify HEAD`'s exit
   status, not from matching git's stderr.
7. `the_commit_names_its_paths_after_a_double_dash` — after a dirty-but-staged
   change, `commit` runs `git commit -m <message> -- constructs.json
   constructs/`: assert the argv exactly, including `--` and both paths in that
   order.
8. `no_destructive_git_command_is_ever_run` — after `run` on a clean work tree,
   join `calls()` and assert that none of the words in requirement 5's list
   appears as an argument. Prove the test is real by adding a `git checkout`
   call once and watching it fail.
9. `luria_commit_exits_7_when_the_work_tree_is_dirty` — run the built binary in
   a dirty temp work tree and assert exit status 7 and a `luria:` line on
   stderr, with no JSON on stdout. Run it the way `crates/cli/tests/cli.rs`
   does, with `env!("CARGO_BIN_EXE_luria")` and `current_dir` set to the temp
   work tree, so the test drives the real argv → exit-status path. Add a second
   assertion in the same test for a run in a directory that is not a work tree.

Prove each of tests 2, 4, 5, 6 and 6a actually constrains the code: for each,
disable the corresponding check once, watch that test fail, restore it, and
report the observed failure. Tests 1, 3, 7, 8, 9 need no such experiment.

## Do not

- Do not change `crates/cli/src/exit.rs`, any file under `crates/core/`,
  anything under `tests/fixtures/`, or any existing test.
- Do not touch `crates/cli/src/review/` or its submodules: this contract has
  nothing to do with the TUI.
- No new dependency, including dev-dependency. Use `std::process::Command`.
- Do not call `git` outside `worktree.rs`; do not shell out through a shell.
- Do not implement record opening, digests, CDS selection or `luria construct`
  — that is 8a3, and it will reuse what you write here.
- Do not print JSON in `luria commit`'s output. It is a human report.

## Checks

From the repository root, with `export TMPDIR=$PWD/target/tmp` first:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
python -I tests/fixtures/verify.py
```

The last must still print `all 21 checks passed`; this contract does not touch
fixtures.

## Handoff

Report: every file changed, exact commands and results, each disable-the-check
experiment with the test that failed and what it said, the exact argv `commit`
builds, every deviation from this contract, and anything you believe the plan
gets wrong — §1's framing in particular has been wrong before, and saying so
costs nothing.
