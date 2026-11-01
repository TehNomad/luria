---
type: Task Contract
title: "Milestone 5d: the luria-cli crate skeleton"
description: "This contract adds the second crate and the luria executable, with everything that doesn't need a request: argument parsing, luria version, luria help, usage errors, and the exit-code type."
status: deprecated
tags: [milestone-5]
---
# Milestone 5d: the `luria-cli` crate skeleton

Status: **Implemented 2026-10-17** by the local worker; accepted after one review round (stdout written with `write_all`, so a failed write exits 1; test added).
Planner: Claude. Parent: [docs/tasks/contracts/milestone-5-cli.md](/tasks/contracts/milestone-5-cli.md) §3.1, §3.2, §3.7 and
§5 (5d).

## 1. What and why

This contract adds the second crate and the `luria` executable, with
everything that doesn't need a request: argument parsing, `luria version`,
`luria help`, usage errors, and the exit-code type. `luria q5 design`'s
arguments are parsed here, but the command itself is wired in 5g. Until
then it says so and exits 1.

## 2. Requirements this contract applies

- **D003, plan §3.1:** the CLI holds no scientific logic. Every number it
  prints comes from `luria-core`.
- **D041:** `luria-cli` may depend on `serde` and `serde_json`, and on nothing
  else besides `luria-core`. Arguments are parsed by hand. Output is JSON
  only. (`serde`'s `derive` feature is part of `serde`.)
- **Plan §3.2:** the commands are `luria q5 design REQUEST.json
  [--write-edited DIR]`, `luria version` and `luria help | --help`.
  `version` prints, as JSON, the package version, every algorithm's name and
  version (mutation, primer, Tm, native design, Q5 workflow) and the request
  and result schema versions.
- **Plan §3.5:** output is deterministic, with keys in a fixed order.
  **Decided here for every JSON document the CLI prints:** `serde_json`
  pretty form (two-space indent), followed by one `\n`.
- **Plan §3.7:** exit codes 0–7, with usage errors as code 2: text on stderr,
  nothing on stdout.

## 3. Files

### 3.1 `Cargo.toml` (root)

Add `"crates/cli"` to `members`, after `"crates/core"`. Change nothing else.

### 3.2 `crates/cli/Cargo.toml`

```toml
[package]
name = "luria-cli"
description = "The luria command-line interface: typed requests in, structured JSON results out."
version = "0.1.0"
edition = "2021"

[lib]
name = "luria_cli"
path = "src/lib.rs"

[[bin]]
name = "luria"
path = "src/main.rs"

[dependencies]
luria-core = { path = "../core" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

`Cargo.lock` changes as cargo resolves these. If the network is not
available, the crates are in the local cargo cache, so retry with
`--offline`. If that fails too, stop and report.

### 3.3 `crates/cli/src/lib.rs`

```rust
//! luria's command-line interface: argument parsing, exit statuses and the
//! JSON documents the `luria` executable prints. It holds no scientific
//! logic (D003): every number comes from `luria-core`.

pub mod args;
pub mod exit;
pub mod version;

/// The request schema `luria q5 design` reads (plan §3.4).
pub const REQUEST_SCHEMA: &str = "luria.q5-request/1";
/// The result schema `luria q5 design` writes (plan §3.5).
pub const RESULT_SCHEMA: &str = "luria.q5-result/1";
/// The schema of `luria version`'s document.
pub const VERSION_SCHEMA: &str = "luria.version/1";
```

### 3.4 `crates/cli/src/args.rs`

```rust
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Help,
    Version,
    Q5Design { request: PathBuf, write_edited: Option<PathBuf> },
}

/// A command line luria can't act on. `message` is one line, no trailing
/// newline and no `luria: ` prefix.
#[derive(Clone, Debug, PartialEq)]
pub struct UsageError {
    pub message: String,
}

/// The usage text `help` prints and every usage error repeats.
pub const USAGE: &str = "usage:
  luria q5 design REQUEST.json [--write-edited DIR]
  luria version
  luria help

Reads a Q5 site-directed mutagenesis request (JSON), runs the workflow and
writes one JSON result document to stdout. See docs/specs/cli.md.
";

/// Parse the arguments after the program name.
pub fn parse(arguments: &[String]) -> Result<Command, UsageError>;
```

Rules (exact, case-sensitive; the messages are exact too):

| Input | Result |
| --- | --- |
| no arguments | `no command given` |
| `help`, `--help` or `-h`, alone | `Help` |
| `version` or `--version`, alone | `Version` |
| `help`/`--help`/`-h` plus anything | `'help' takes no arguments` |
| `version`/`--version` plus anything | `'version' takes no arguments` |
| `q5` alone | `'q5' needs a subcommand: design` |
| `q5 X`, `X` not `design` | `unknown q5 subcommand 'X'; the only one is 'design'` |
| any other first word `X` | `unknown command 'X'` |

After `q5 design`, read the remaining words left to right:
- `--write-edited` takes the next word as the directory, whatever it is. No
  next word gives `--write-edited needs a directory`. A second
  `--write-edited` gives `--write-edited is given more than once`.
- Any other word starting with `-` gives `unknown option 'W'`. That includes
  `--write-edited=DIR`; only the two-word form is read.
- Any other word is the request path. A second one gives `'q5 design' takes
  one request file, got a second: 'W'`.
- An empty word, used as the request path or as the directory, gives `an
  empty path is not a file`.
- At the end, no request path gives `'q5 design' needs a request file`.

The first error found, left to right, is the one reported.

### 3.5 `crates/cli/src/exit.rs`

```rust
use luria_core::error::ErrorCategory;

/// The process exit statuses of plan §3.7.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitStatus {
    /// 0: verified, and every member designed.
    Success,
    /// 1: an internal error or an I/O failure.
    Internal,
    /// 2: bad arguments; text on stderr, no JSON.
    Usage,
    /// 3: `needs_clarification`.
    NeedsClarification,
    /// 4: `failed`, `MalformedInput`.
    MalformedInput,
    /// 5: `failed`, `BiologicalValidation`.
    BiologicalValidation,
    /// 6: `failed`, `Unsupported`.
    Unsupported,
    /// 7: verified, but at least one variant-set member could not be designed.
    PartialDesign,
}

impl ExitStatus {
    pub fn code(self) -> u8;
    /// The status of a run that failed with an error of this category.
    pub fn for_failure(category: ErrorCategory) -> ExitStatus;
}
```

Both are exhaustive matches with no `_` arm.

### 3.6 `crates/cli/src/version.rs`

```rust
use serde::Serialize;

/// One algorithm a result can name (D021).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Algorithm {
    pub role: &'static str,
    pub name: &'static str,
    pub version: &'static str,
}

/// Every algorithm, in this order, from the `luria-core` constants:
/// `mutation` (MUTATION_ALGORITHM_NAME/VERSION), `primer`
/// (PRIMER_DESIGN_NAME/VERSION), `tm` (TM_FUNCTION_NAME/VERSION),
/// `native_design` (NATIVE_DESIGN_ALGORITHM_NAME/VERSION) and `q5_workflow`
/// (Q5_WORKFLOW_NAME/VERSION).
pub fn algorithms() -> Vec<Algorithm>;

/// `luria version`'s document, in the serde_json pretty form plus `\n`.
pub fn render_version() -> String;
```

The document's fields, in this order: `schema` (`VERSION_SCHEMA`),
`package` (`"luria-cli"`), `version` (`env!("CARGO_PKG_VERSION")`),
`algorithms` (`algorithms()`), and `schemas`, an object with `request` and
`result` in that order. Use `#[derive(Serialize)]` structs, not
`serde_json::Value` or `json!`, so the field order is the struct order.
`algorithms()` is public because 5f will put the same list in each result's
provenance.

### 3.7 `crates/cli/src/main.rs`

A thin `main` returning `std::process::ExitCode`:
- `Help`: print `USAGE` to stdout, exit 0.
- `Version`: print `render_version()` to stdout, exit 0.
- `Q5Design { .. }`: print `luria: 'q5 design' is not available in this
  build yet` plus a newline to stderr and nothing to stdout. Exit 1
  (`Internal`). 5g replaces this.
- `UsageError`: print `luria: {message}\n\n{USAGE}` to stderr and nothing to
  stdout. Exit 2.

Use `std::env::args().skip(1)`. Exit with
`ExitCode::from(status.code())`. No `unwrap` on I/O, and no `print!`, which
panics when stdout can't be written. Write stdout with
`std::io::Write::write_all` on `std::io::stdout().lock()`, then `flush()`.
If either fails, exit 1 (`Internal`). Stderr is written the same way, and a
failure there leaves the status unchanged. *(Made explicit 2026-10-17 in
review.)*

## 4. Tests (planner-written; add verbatim)

### 4.1 `args.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn parse_words(words: &[&str]) -> Result<Command, UsageError> {
        let arguments: Vec<String> = words.iter().map(|word| word.to_string()).collect();
        parse(&arguments)
    }

    fn usage(words: &[&str]) -> String {
        match parse_words(words) {
            Err(error) => error.message,
            Ok(command) => panic!("{:?} parsed as {:?}", words, command),
        }
    }

    #[test]
    fn help_and_version_have_their_spellings() {
        for words in [&["help"][..], &["--help"][..], &["-h"][..]] {
            assert_eq!(parse_words(words), Ok(Command::Help), "{:?}", words);
        }
        for words in [&["version"][..], &["--version"][..]] {
            assert_eq!(parse_words(words), Ok(Command::Version), "{:?}", words);
        }
    }

    #[test]
    fn q5_design_takes_one_request_and_an_optional_output_directory() {
        assert_eq!(
            parse_words(&["q5", "design", "req.json"]),
            Ok(Command::Q5Design {
                request: PathBuf::from("req.json"),
                write_edited: None
            })
        );
        let with_dir = Ok(Command::Q5Design {
            request: PathBuf::from("req.json"),
            write_edited: Some(PathBuf::from("out")),
        });
        assert_eq!(
            parse_words(&["q5", "design", "req.json", "--write-edited", "out"]),
            with_dir
        );
        assert_eq!(
            parse_words(&["q5", "design", "--write-edited", "out", "req.json"]),
            with_dir
        );
        // The word after --write-edited is the directory, whatever it is.
        assert_eq!(
            parse_words(&["q5", "design", "req.json", "--write-edited", "-x"]),
            Ok(Command::Q5Design {
                request: PathBuf::from("req.json"),
                write_edited: Some(PathBuf::from("-x")),
            })
        );
    }

    #[test]
    fn each_usage_error_says_what_is_wrong() {
        let cases: [(&[&str], &str); 15] = [
            (&[], "no command given"),
            (&["frobnicate"], "unknown command 'frobnicate'"),
            (&["Help"], "unknown command 'Help'"),
            (&["help", "q5"], "'help' takes no arguments"),
            (&["version", "x"], "'version' takes no arguments"),
            (&["q5"], "'q5' needs a subcommand: design"),
            (&["q5", "plan"], "unknown q5 subcommand 'plan'; the only one is 'design'"),
            (&["q5", "design"], "'q5 design' needs a request file"),
            (
                &["q5", "design", "a.json", "b.json"],
                "'q5 design' takes one request file, got a second: 'b.json'",
            ),
            (&["q5", "design", "a.json", "--write-edited"], "--write-edited needs a directory"),
            (
                &["q5", "design", "a.json", "--write-edited", "o", "--write-edited", "p"],
                "--write-edited is given more than once",
            ),
            (&["q5", "design", "a.json", "--write-edited=o"], "unknown option '--write-edited=o'"),
            (&["q5", "design", "-v", "a.json"], "unknown option '-v'"),
            (&["q5", "design", ""], "an empty path is not a file"),
            (&["q5", "design", "a.json", "--write-edited", ""], "an empty path is not a file"),
        ];
        for (words, message) in cases {
            assert_eq!(usage(words), message, "{:?}", words);
        }
    }
}
```

### 4.2 `exit.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_its_documented_code() {
        let cases: [(ExitStatus, u8); 8] = [
            (ExitStatus::Success, 0),
            (ExitStatus::Internal, 1),
            (ExitStatus::Usage, 2),
            (ExitStatus::NeedsClarification, 3),
            (ExitStatus::MalformedInput, 4),
            (ExitStatus::BiologicalValidation, 5),
            (ExitStatus::Unsupported, 6),
            (ExitStatus::PartialDesign, 7),
        ];
        for (status, code) in cases {
            assert_eq!(status.code(), code, "{:?}", status);
        }
    }

    #[test]
    fn a_failure_exits_by_its_category() {
        assert_eq!(
            ExitStatus::for_failure(ErrorCategory::MalformedInput),
            ExitStatus::MalformedInput
        );
        assert_eq!(
            ExitStatus::for_failure(ErrorCategory::BiologicalValidation),
            ExitStatus::BiologicalValidation
        );
        assert_eq!(
            ExitStatus::for_failure(ErrorCategory::Unsupported),
            ExitStatus::Unsupported
        );
    }
}
```

### 4.3 Integration test: `crates/cli/tests/cli.rs`

```rust
//! The `luria` executable, run as a process: exit codes, which stream each
//! document goes to, and the exact text of `luria version`.

use luria_core::mutation::{MUTATION_ALGORITHM_NAME, MUTATION_ALGORITHM_VERSION};
use luria_core::primer::{PRIMER_DESIGN_NAME, PRIMER_DESIGN_VERSION};
use luria_core::primer_design::{NATIVE_DESIGN_ALGORITHM_NAME, NATIVE_DESIGN_ALGORITHM_VERSION};
use luria_core::q5::{Q5_WORKFLOW_NAME, Q5_WORKFLOW_VERSION};
use luria_core::tm::{TM_FUNCTION_NAME, TM_FUNCTION_VERSION};
use std::process::{Command, Output};

fn luria(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_luria"))
        .args(arguments)
        .output()
        .expect("the luria executable runs")
}

fn algorithm_block(role: &str, name: &str, version: &str, last: bool) -> String {
    format!(
        "    {{\n      \"role\": \"{}\",\n      \"name\": \"{}\",\n      \"version\": \"{}\"\n    }}{}\n",
        role,
        name,
        version,
        if last { "" } else { "," }
    )
}

// The whole document is pinned, so the key order and the layout are too.
#[test]
fn version_prints_the_pinned_document() {
    let output = luria(&["version"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let blocks: String = [
        algorithm_block("mutation", MUTATION_ALGORITHM_NAME, MUTATION_ALGORITHM_VERSION, false),
        algorithm_block("primer", PRIMER_DESIGN_NAME, PRIMER_DESIGN_VERSION, false),
        algorithm_block("tm", TM_FUNCTION_NAME, TM_FUNCTION_VERSION, false),
        algorithm_block(
            "native_design",
            NATIVE_DESIGN_ALGORITHM_NAME,
            NATIVE_DESIGN_ALGORITHM_VERSION,
            false,
        ),
        algorithm_block("q5_workflow", Q5_WORKFLOW_NAME, Q5_WORKFLOW_VERSION, true),
    ]
    .concat();
    let expected: String = format!(
        "{{\n  \"schema\": \"luria.version/1\",\n  \"package\": \"luria-cli\",\n  \"version\": \"{}\",\n  \"algorithms\": [\n{}  ],\n  \"schemas\": {{\n    \"request\": \"luria.q5-request/1\",\n    \"result\": \"luria.q5-result/1\"\n  }}\n}}\n",
        env!("CARGO_PKG_VERSION"),
        blocks
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
}

#[test]
fn version_is_byte_identical_run_to_run() {
    assert_eq!(luria(&["version"]).stdout, luria(&["--version"]).stdout);
}

#[test]
fn help_prints_the_usage_to_stdout() {
    for spelling in ["help", "--help", "-h"] {
        let output = luria(&[spelling]);
        assert_eq!(output.status.code(), Some(0), "{}", spelling);
        assert!(output.stderr.is_empty(), "{}", spelling);
        assert_eq!(String::from_utf8(output.stdout).unwrap(), luria_cli::args::USAGE);
    }
}

#[test]
fn a_usage_error_exits_2_with_text_on_stderr_and_nothing_on_stdout() {
    let cases: [(&[&str], &str); 4] = [
        (&[], "no command given"),
        (&["frobnicate"], "unknown command 'frobnicate'"),
        (&["q5", "design"], "'q5 design' needs a request file"),
        (&["q5", "design", "a.json", "-v"], "unknown option '-v'"),
    ];
    for (arguments, message) in cases {
        let output = luria(arguments);
        assert_eq!(output.status.code(), Some(2), "{:?}", arguments);
        assert!(output.stdout.is_empty(), "{:?}", arguments);
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("luria: {}\n\n{}", message, luria_cli::args::USAGE),
            "{:?}",
            arguments
        );
    }
}

// Added in review: a document that cannot be written is an I/O failure,
// exit 1 (plan §3.7), not a panic (101). /dev/full refuses every write.
#[cfg(target_os = "linux")]
#[test]
fn a_failed_write_to_stdout_exits_1() {
    for arguments in [&["version"][..], &["help"][..]] {
        let full = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .expect("/dev/full opens");
        let output = Command::new(env!("CARGO_BIN_EXE_luria"))
            .args(arguments)
            .stdout(full)
            .output()
            .expect("the luria executable runs");
        assert_eq!(output.status.code(), Some(1), "{:?}", arguments);
    }
}

#[test]
fn q5_design_is_not_wired_yet_and_says_so() {
    let output = luria(&["q5", "design", "request.json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "luria: 'q5 design' is not available in this build yet\n"
    );
}
```

`cargo fmt` may reflow these; do not change content. If one fails, stop and
report what the binary printed. Do not change the test.

### 4.4 Proving the rules are load-bearing (AGENTS.md)

For each of these, disable the rule once, confirm the named test fails, and
restore:

| Rule | Test that must fail |
| --- | --- |
| a second `--write-edited` is refused | `each_usage_error_says_what_is_wrong` |
| a second request path is refused | `each_usage_error_says_what_is_wrong` |
| an empty path is refused | `each_usage_error_says_what_is_wrong` |
| `help` takes no arguments | `each_usage_error_says_what_is_wrong` |
| a usage error writes nothing to stdout (print the usage to stdout instead) | `a_usage_error_exits_2_with_text_on_stderr_and_nothing_on_stdout` |

## 5. Do not

- Add dependencies beyond the three in §3.2, or any to `luria-core`.
- Edit anything in `crates/core/`.
- Read request files, call any workflow, or define request or result types:
  that is 5e, 5f and 5g.
- Use `serde_json::Value` or `json!` to build output documents.
- Add commands or options beyond §3.4.

## 6. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace` (this now builds and runs the `luria` binary)
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 7. Handoff

Report the changed and new files, the exact commands and results, the §4.4
results, the `serde`/`serde_json` versions that `Cargo.lock` resolved, and
anything that did not match this contract.
