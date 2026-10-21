# Milestone 7f1: the request file's rules, callable on their own

Status: **Implemented 2026-10-21** by the local worker; accepted on the
first review. `run.rs`'s own unit test `a_write_never_replaces_a_file` had
its three `write_new` calls renamed, its assertions unchanged. Planner:
Claude. Parent: `docs/tasks/milestone-7-tui.md` §3 rule 4 ("Approving
shows the request file's path first; the file follows `--write-request`'s
rules"), §6 (7f). A refactor with no change in behaviour; 7f2 uses it.

## 1. What and why

In the TUI flow (M7-1), `interpret` runs **without** writing the request.
The user reads the proposal; only on approval is the request written and
designed. The file must follow exactly the rules `luria q5 interpret
--write-request` follows today (`docs/cli.md` §10.5, §10.7): beside the
template, a name that is a file, never replacing anything, written
atomically. Those rules live inside `interpret`'s steps 4 and 8
(`crates/cli/src/interpret/run.rs`). This contract moves them into three
public functions, and makes `interpret` call them, so the TUI and the
command share one implementation.

## 2. Changes, all in `crates/cli/src/interpret/run.rs`

```rust
/// Why the request cannot be written at `path` for a template at
/// `template`, as (code, message), or `None` when it can (plan §5.7, C-1).
/// Decided from the file system without writing anything.
pub fn request_location_problem(template: &Path, path: &Path) -> Option<(String, String)>;

/// The request as `--write-request` writes it: `to_string_pretty` of the
/// echo and one `\n`.
pub fn request_text(echo: &RequestEcho) -> String;

/// Write `text` at `path` as a new file, never replacing one, atomically
/// with the check (C-1). The `String` is the operating system's reason.
pub fn write_request_file(path: &Path, text: &str) -> Result<(), String>;
```

- `request_location_problem` holds today's step 4 checks, in today's
  order, with today's codes and messages, unchanged: not the template's
  directory, or a path that does not end in its file name, is
  `request_location_unusable` with the message about the template's
  directory; an existing entry (a dangling link included) is
  `output_exists` with the message naming the file; any other metadata
  failure is `request_location_unusable` with "cannot check whether …".
- `request_text` is today's step 8 text. Today's `panic!` on a
  serialization failure stays as it is.
- `write_request_file` is today's `write_new`, renamed and made public;
  `write_new` is removed.
- `interpret`'s step 4 becomes: `match request_location_problem(..)`,
  `Some((code, message))` → `refused(&answered, code, None, message,
  ExitStatus::MalformedInput)`; step 8 uses `request_text` and
  `write_request_file`. Nothing else in `interpret` changes.

## 3. Tests

**Every existing test must pass unchanged**: this is the proof that the
refactor changed nothing (in particular `tests/interpret_run.rs`'s
location and `output_exists` cases, and `tests/interpret_cli.rs`).

Add unit tests in `run.rs`'s test module, or in a new
`crates/cli/tests/request_file.rs`, in directories under
`env!("CARGO_TARGET_TMPDIR")`:
1. `a_free_name_beside_the_template_is_usable`: `None`.
2. `another_directory_is_unusable`: code `request_location_unusable`.
3. `an_existing_file_is_never_replaced`: code `output_exists`, and after
   `write_request_file` on the same path, `Err`, with the old content
   still in the file.
4. `the_text_is_the_pretty_echo_and_a_newline`: for the echo of
   `tests/fixtures/nl/request/edit-a42g.json` parsed with
   `parse_request` and turned into an echo with `request_echo`, the text
   ends with exactly one `\n` and parses back (`parse_request`) to the
   same request.

### 3.1 Proof

Make `request_location_problem` always return `None` once: test 2, test
3's first assertion, and the existing `interpret_run.rs` location and
`output_exists` cases fail, which shows `interpret` now depends on the
new function. Restore, and report the run.

## 4. Do not

- Change only `crates/cli/src/interpret/run.rs` and, if you choose, the
  new `crates/cli/tests/request_file.rs`.
- Do not change any code, message or exit status `interpret` produces.
- Do not change other files, tests, fixtures, `docs/`, any `Cargo.toml`
  or `vendor/`. Do not add dependencies. Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the proof of §3.1, and any point where this contract
does not match the code; stop and report rather than guess.
