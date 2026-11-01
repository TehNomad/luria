# Review M6 C-1, C-2, C-3, C-6, C-8: the command's local checks

Status: **Implemented 2026-10-21** by the local worker; accepted on planner
review. Planner: Claude. Parent:
[docs/tasks/reviews/review-post-m6.md](/tasks/reviews/review-post-m6.md) C-1, C-2, C-3, C-6, C-8. Owner, 2026-10-20:
"Go ahead and fix them".

Files (no others):
- `crates/cli/src/interpret/run.rs`
- `crates/cli/tests/interpret_run.rs`

## 1. Defects

- C-1: `--write-request new.json/` (or `name/.`) passes step 4:
  `Path::file_name` drops the trailing `/` or `/.`, and `symlink_metadata`
  says not found. The model is called, then the write fails (exit 1),
  against "nothing is sent until every local input is valid" (6h1).
- C-2: step 4 takes *any* `symlink_metadata` error as "the name is free";
  `write.rs` takes only `NotFound` so.
- C-3: `--template ..` is reported as "the template file's name is not
  UTF-8"; the path names no file at all.
- C-6: the text-file and profile caps can be raised to the template cap
  with every test green; non-UTF-8 text, profile and template, and a
  failed `--write-request` write, are untested.
- C-8: the unit test `a_write_never_replaces_a_file` writes under
  `std::env::temp_dir()`, which the nono sandbox makes write-only, so it
  fails there.

## 2. Required change (all in `run.rs`)

### 2.1 C-1: the last component must be the file name

In step 4, after `request_name` is computed: the path is refused with the
existing `request_location_unusable` refusal (same code, message and exit)
unless `written.to_str()` is `Some(text)` and the last piece of
`text.rsplit(std::path::is_separator)` equals `request_name`. So
`new.json/` (last piece empty) and `name/.` (last piece `.`) are refused
before anything is sent; `proposal.json` and `dir/proposal.json` are not.

### 2.2 C-2: only "not found" is free

In step 4, `symlink_metadata(written)`:
- `Ok(_)` → `output_exists`, as now;
- `Err` with `kind() == ErrorKind::NotFound` → the name is free, as now;
- any other `Err(problem)` → refused:
  `refused(&answered, "request_location_unusable".to_string(), None, format!("cannot check whether {} already exists: {}", name, problem), ExitStatus::MalformedInput)`
  where `name` is `request_name`'s value.

### 2.3 C-3: a template path that names no file

In step 3, split the two cases `name_of` folds together. When
`input.template.file_name()` is `None`, refuse with code
`template_unreadable`, exit `MalformedInput`, message
`"the template path names no file"`. When it is `Some` but not UTF-8, keep
the existing message. (A local match on `file_name()` and `to_str()` in
step 3 is fine; `name_of` itself is unchanged, since step 4 uses it.)

### 2.4 C-8: the unit test's directory

In `a_write_never_replaces_a_file`, build the directory as
`Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp").join(format!("luria-write-new-{}", std::process::id()))`
instead of `std::env::temp_dir()...` (AGENTS.md: scratch files go under
`target/tmp/`). Nothing else in the test changes.

## 3. Tests (in `crates/cli/tests/interpret_run.rs`; write first)

Existing tests unchanged. Add one new test,
`every_capped_or_odd_input_is_checked_before_anything_is_sent`, built
like `every_local_input_is_checked_before_anything_is_sent`: its own
`workspace`, a `Vec<LocalCase>`, and for each case the same loop body
(status, `"failed"` state, error code, `backend.prompts().is_empty()`,
`messages == None`, no `CARGO_TARGET_TMPDIR` in the rendered document).
Files it writes into its workspace:
- `long.txt`: `8 * 1024 + 1` bytes of `a`;
- `big-profile.json`: `64 * 1024 + 1` bytes of ` ` (spaces);
- `latin1.txt`, `latin1-profile.json` and `latin1.gb`: the bytes
  `[0x41, 0xe9, 0x0a]`.

Cases:

| Name | Change from `base()` | Status | Code |
| --- | --- | --- | --- |
| text file over the cap | `text: TextSource::File(dir.join("long.txt"))` | `MalformedInput` | `input_too_large` |
| profile over the cap | `profile: Some(dir.join("big-profile.json"))` | `MalformedInput` | `input_too_large` |
| text not UTF-8 | `text: TextSource::File(dir.join("latin1.txt"))` | `MalformedInput` | `text_unreadable` |
| profile not UTF-8 | `profile: Some(dir.join("latin1-profile.json"))` | `MalformedInput` | `profile_unreadable` |
| template not UTF-8 | `template: dir.join("latin1.gb")` | `MalformedInput` | `template_unreadable` |
| template names no file | `template: dir.join("..")` | `MalformedInput` | `template_unreadable` |
| request ends in a separator | `write_request: Some(dir.join("new.json/"))` | `MalformedInput` | `request_location_unusable` |
| request ends in a dot | `write_request: Some(dir.join("new.json").join("."))` | `MalformedInput` | `request_location_unusable` |

and, after the loop, assert the "template names no file" case's error
message is `"the template path names no file"` (run it once more, or keep
its outcome).

Then a second new test, unix only (`#[cfg(unix)]`),
`a_proposal_whose_write_fails_is_still_shown_and_exits_1`: a workspace,
the `one-sub-prose` text and reply, `write_request` `proposal.json`; set
the workspace directory's permissions to `0o555` before calling
`interpret`, and back to `0o755` right after (before any assertion, so a
failing assertion cannot leave it read-only). Expect
`outcome.status == ExitStatus::Internal`, `outcome.document.state ==
"proposed"`, `outcome.document.request` is `Some`, `outcome.note` holds
`"could not write proposal.json"`, and `proposal.json` does not exist.

Disabling proofs (once each, run, restore, report):
- read the text file with `MAX_TEMPLATE_BYTES` → "text file over the cap"
  fails; the same for the profile;
- skip §2.1 → both request cases fail (the backend is called);
- skip §2.3 → "template names no file" fails on its message.

C-2 has no test: the reviewer could not stage an error other than
`NotFound` that survives step 4's earlier checks. Say so in the handoff.

## 4. Do not

- Add dependencies, or change any file not listed.
- Change any cap, code, existing message, or existing test (other than
  §2.4's one line).

## 5. Checks

From the repository root, after `export TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

Also run `env -u TMPDIR cargo test -p luria-cli --lib a_write_never_replaces_a_file`
after a build with `TMPDIR` set, and report its result (C-8).

## 6. Handoff

Report the changed files, each new test case and whether it failed before
the change (the cap and UTF-8 cases cover behavior that is already right:
say they pass before and after), each disabling proof and its result, the
check results, and any deviation.
