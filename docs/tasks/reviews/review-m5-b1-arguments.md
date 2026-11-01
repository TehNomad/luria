# Review M5 B-1, B-7: non-UTF-8 arguments and `--`

Status: **Implemented 2026-10-18** by the local worker; accepted on planner review. Planner: Claude. Parent:
[docs/tasks/reviews/review-post-m5.md](/tasks/reviews/review-post-m5.md) B-1 and B-7. Requirement:
[docs/tasks/contracts/milestone-5-cli.md](/tasks/contracts/milestone-5-cli.md) §3.7, under which exit 2 means "Usage error:
bad arguments. Text on stderr, no JSON".
Files: `crates/cli/src/args.rs`, `crates/cli/src/main.rs`,
`crates/cli/tests/cli.rs`, [docs/specs/cli.md](/specs/cli.md).

## 1. Defects

- **B-1.** `main.rs:80` collects `env::args()`. That panics on any argument
  that isn't valid UTF-8, so `luria q5 design "$(printf 'r\xff.json')"`
  exits 101 with a Rust panic message instead of a usage error (exit 2).
- **B-7.** `parse_design` treats `--` as an unknown option. A request file
  whose name starts with `-` can't be named except as `./-x.json`.

## 2. Required change

### 2.1 `args.rs`

Add, above `parse`:

```rust
/// The command line's words as text. A word that isn't valid UTF-8 is a
/// usage error naming its 1-based position after the program name.
pub fn utf8_arguments(arguments: Vec<std::ffi::OsString>) -> Result<Vec<String>, UsageError>
```

- Convert each word with `OsString::into_string`.
- The first word that fails gives
  `UsageError { message: format!("argument {} is not valid UTF-8", n) }`,
  where `n` counts from 1 (the first word after the program name is 1).
- Otherwise return the words in order.

In `parse_design`, a word exactly equal to `--` ends option parsing. Every
later word is a positional argument (the request path), even when it starts
with `-`. Its existing rules still apply: the empty-path refusal and "takes
one request file, got a second". `--` itself is not a request path.
`--write-edited` after `--` is therefore a request path, not the option.
A second `--` after the first is an ordinary positional word.

Update the doc comment of `parse_design` to mention `--`.

### 2.2 `main.rs`

Replace `env::args().skip(1).collect()` with `env::args_os().skip(1)`,
collected into a `Vec<OsString>` and passed to `utf8_arguments`. On `Err`,
call `report_usage` with that error, so the result is exit 2, text on
stderr and nothing on stdout. On `Ok`, continue exactly as now.

### 2.3 [docs/specs/cli.md](/specs/cli.md)

In the section that describes usage errors (around line 26), add one
sentence each:
- an argument that isn't valid UTF-8 is a usage error;
- in `q5 design`, `--` ends the options, so a request file whose name starts
  with `-` can be given after it.

## 3. Tests (write first, confirm they fail before the change)

In `args.rs`'s test module:
1. `utf8_arguments` on `["q5", "design", "a.json"]` as `OsString`s returns
   the same three strings.
2. On Unix only (`#[cfg(unix)]`), build a word from bytes `b"r\xff.json"`
   with `std::os::unix::ffi::OsStringExt::from_vec`. `utf8_arguments` on
   `["q5", "design", <that>]` returns
   `Err` with message `"argument 3 is not valid UTF-8"`.
3. `parse_words(&["q5", "design", "--", "-x.json"])` gives `Q5Design` with
   request `-x.json` and no directory.
4. `parse_words(&["q5", "design", "--write-edited", "out", "--", "-x.json"])`
   gives request `-x.json`, directory `out`.
5. `parse_words(&["q5", "design", "--", "a.json", "b.json"])` gives the
   existing "got a second" error.
6. `parse_words(&["q5", "design", "--"])` gives the existing
   "'q5 design' needs a request file" error.
7. `parse_words(&["q5", "design", "--", "--write-edited"])` gives request
   `--write-edited`.

In `crates/cli/tests/cli.rs`, following the style of the existing usage-error
tests there, add one `#[cfg(unix)]` test that runs the built binary with
arguments `q5`, `design` and the non-UTF-8 word from test 2, and asserts:
exit status 2, empty stdout, and stderr containing
`argument 3 is not valid UTF-8`.

## 4. Do not

- Add dependencies.
- Change any other file, or any existing test.
- Change any existing usage-error message text.
- Accept `--` anywhere other than inside `q5 design`'s arguments.

## 5. Checks

From the repository root, after `export TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 6. Handoff

Report the changed files, the new tests and whether each failed before the
change, the check results, and any deviation.
