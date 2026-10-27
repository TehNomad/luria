# Contract 8h — cli: opening a bare file (§5.6)

Planner: Claude. Status: delegated to `w-8h` 2026-10-26; runs after 8k1 (D056 order).
Tier: light (D051): tests assert behaviour; wording may change at the hand
test. Plan row: **8h** in `docs/tasks/milestone-8-view.md` §7.

## 1. What this contract builds

`luria FILE` opens the sequence view (8k1) on any existing path that is not
a subcommand word. It adds the dispatch, the terminal check before the file
is read, the `.json` refusal, and "did you mean" for a near-miss command.

Rules that apply, already settled:
- Plan §5.6 (`docs/tasks/milestone-8-view.md:479-500`).
- D048 M8-13 (`scripts/decision D048`): a terminal on stdin and stdout,
  checked by the code `review` and `new` use, before the file is opened; a
  usage error with exit 2 otherwise.
- D056: the file is read by `record_file::read_record_file` (8i2), exactly
  as `luria find` reads it.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `parse`, `reject`, `USAGE`, `Command` | `crates/cli/src/args.rs:122-154` | the dispatch; `parse`'s unknown-word branch is where this starts |
| `report_usage`, `write_err`, `emit_review` | `crates/cli/src/main.rs:473`, `:46`, `:152-219` | usage errors print with the usage text, exit 2; a failed terminal is `luria: the terminal failed: {e}`, exit 1 |
| `review::terminal_problem` | `crates/cli/src/review.rs:299` | the terminal check (see §4 rule 4) |
| `read_record_file`, `RecordFileProblem::report` | `crates/cli/src/record_file.rs` | reading the file; a problem prints `luria: {message}` with its status |
| `sequence_view::terminal::run` | `crates/cli/src/sequence_view/terminal.rs` (8k1) | the view |
| test idiom | `crates/cli/tests/find_cli.rs` (`luria`, `workspace`, `pbr`) | run the binary; write inputs under `CARGO_TARGET_TMPDIR` |

## 3. Files the worker may touch

1. `crates/cli/src/args.rs`: `Command::View { file }`, `parse_with`, the
   suggestion, one `USAGE` line `luria FILE` placed first among the
   commands, and the one existing expectation named in §8.
2. `crates/cli/src/main.rs`: call `parse_with`, and the `Command::View` arm.
3. `crates/cli/src/review.rs`: `terminal_problem_for` only (rule 4).
4. `crates/cli/tests/open_cli.rs`: new.

## 4. The public surface, exactly

```rust
// args.rs
pub enum Command { /* … */ View { file: PathBuf } }
/// `parse` with a way to ask whether a word names an existing path. `parse`
/// becomes `parse_with(arguments, &|_| false)`, so its tests are unchanged.
pub fn parse_with(arguments: &[String], is_path: &dyn Fn(&str) -> bool)
    -> Result<Command, UsageError>;
/// The subcommand nearest `word`, for "did you mean" (rule 3).
pub fn suggestion(word: &str) -> Option<&'static str>;
// review.rs
pub fn terminal_problem_for(command: &str, instead: &str,
    stdin_is_terminal: bool, stdout_is_terminal: bool) -> Option<UsageError>;
```

Rules (the §5 tests in brackets):

1. **Order.** Every subcommand word and spelling `parse` knows today wins
   first, even when a file of that name exists. Only an unknown first word
   reaches rules 2–3. [6]
2. **A path.** When `is_path(word)`: with more words → usage error
   `'{word}' opens the view and takes no further arguments; 'luria find
   FILE PATTERN' searches a file`; a name ending in `.json` (ASCII case
   ignored) → usage error `opening a design comes in milestone 9; use
   'luria review {word}'`; otherwise `Command::View`. [3, 4, 8]
3. **Not a path.** `unknown command '{word}'`, and when `suggestion` gives
   a name, `; did you mean 'luria {name}'?` after it. `suggestion` compares
   the ASCII-lowercased word with `help`, `version`, `q5`, `review`, `new`,
   `find` by Levenshtein distance; the nearest wins if its distance is at
   most 2 and less than the word's character count; ties go to the earlier
   name in that list. [5, 9]
4. **The terminal.** `terminal_problem_for(command, instead, …)` gives
   `'{command}' needs a terminal on stdin and stdout; {instead}`;
   `terminal_problem` calls it with `'q5 design' writes the same result as
   JSON`, so its message is unchanged. The view calls it with the file as
   given and `'luria find FILE PATTERN' searches a file without one`. It runs
   before the file is read. [1, 2]
5. **Then** `read_record_file`; a problem prints as `luria find` prints it.
   Then `sequence_view::terminal::run(record)`: `Ok` exits 0; `Err` prints
   `luria: the terminal failed: {e}`, exit 1.
6. `luria` alone is unchanged: `no command given`. `main` passes
   `&|word| Path::new(word).exists()`.

## 5. Tests the worker writes

Through the binary in `open_cli.rs` (the tests have no terminal), and unit
tests in `args.rs`'s `mod tests` (8, 9 and 6's first half).

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `a_file_needs_a_terminal` | `luria pbr` → exit 2, stdout empty, stderr starts with `luria: '{path}' needs a terminal on stdin and stdout` and holds the usage text |
| 2 | `the_terminal_comes_before_the_read` | a non-UTF-8 file (`>x\n\xff\n`) → the terminal message, exit 2, not `is not UTF-8 text` |
| 3 | `a_file_takes_no_further_arguments` | `luria pbr GAATTC` → exit 2, stderr holds `takes no further arguments` and `luria find` |
| 4 | `a_design_file_is_refused` | existing `d.json` and `D.JSON` → exit 2, stderr holds `opening a design comes in milestone 9` |
| 5 | `a_near_miss_suggests_the_command` | `fnd` → `did you mean 'luria find'?`; `revew` → `'luria review'`; `Version` → `'luria version'`; `zzfind` → `'luria find'`; `zzzfind`, `frobnicate`, `x` → stderr exactly `luria: unknown command '{word}'\n\n{USAGE}` |
| 6 | `a_command_word_beats_a_file` | unit: `parse_with(["find"], &\|_\| true)` is find's own usage error; `["help"]` → `Help`. Binary: in a directory holding a file named `version`, `luria version` (current dir set to it) prints the version |
| 7 | `a_missing_path_is_an_unknown_command` | `luria absent.gb` → exit 2, `unknown command 'absent.gb'` |
| 8 | `a_path_opens_the_view` | unit: `parse_with(["pbr.gb"], &\|_\| true)` → `View { file: "pbr.gb" }`; with `["pbr.gb", "x"]` → the rule 2 error |
| 9 | `suggestion_by_distance` | unit: `fnd` → `find`, `nwe` → `new`, `q6` → `q5`, `HELP` → `help`, `rvw` → `new` (distance 2 < 3), `ab` → `None`, `x` → `None`, `zzzfind` → `None` |

The values in tests 5 and 9 were derived by the planner by hand.

## 6. Checks

`scripts/check`, plus `cargo test -p luria-cli --test open_cli`.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | The terminal check after `read_record_file` | 2 |
| G2 | No `.json` refusal | 4 |
| G3 | Suggestion distance up to 3 | 5 or 9 (`zzzfind`) |
| G4 | Extra words after a file ignored | 3, 8 |
| G5 | The path check before the subcommand words | 6 |
| G6 | `suggestion` without lowercasing | 9 (`HELP`; `Version` is still distance 1) |

## 8. Do not

- No new dependency; no change to `crates/core`, `find.rs`, `record_file.rs`,
  `sequence_view/`, or any existing test, except this one expectation:
  `args.rs` `each_usage_error_says_what_is_wrong`'s `Help` case becomes
  `unknown command 'Help'; did you mean 'luria help'?`.
- No start screen for `luria` alone (milestone 9).

## 9. Report

Every guard experiment with its failure line, and the exact stderr of
tests 1, 3 and 5's `fnd`.
