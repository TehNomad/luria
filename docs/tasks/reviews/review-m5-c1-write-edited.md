---
type: Task Contract
title: "Review M5 C-1, C-2: --write-edited never overwrites, never writes elsewhere"
description: "write_edited checks path.try_exists() and then calls fs::write, which truncates an existing file and follows symlinks."
status: deprecated
tags: [milestone-5]
---
# Review M5 C-1, C-2: `--write-edited` never overwrites, never writes elsewhere

Status: **Implemented 2026-10-18** by the local worker; accepted on planner review. Planner: Claude. Parent:
[docs/tasks/reviews/review-post-m5.md](/tasks/reviews/review-post-m5.md) C-1 and C-2. Requirements: D041 G8 ("An
existing file is never overwritten"), `milestone-5g2-write-edited.md` §4.1
("a request cannot choose where the constructs are written"). Owner answer
C-2, 2026-10-18: "Refuse case-folded clashes with --write-edited", and also
refuse `:` `*` `?` `<` `>` `|` in names there. Windows reserved names (`CON`,
`NUL`, …) are left alone.

Files: `crates/cli/src/write.rs`, `crates/cli/src/run.rs` (`write_outcome`
only), `crates/cli/tests/write_edited.rs`.

## 1. Defects

- **C-1.** `write_edited` checks `path.try_exists()` and then calls
  `fs::write`, which truncates an existing file and follows symlinks.
  - `try_exists` follows a symlink, so a **dangling** symlink at
    `DIR/<name>.fa` reads as free, and the write goes through it to wherever
    it points, outside DIR (reproduced by the reviewer).
  - A file created between the check and the write is overwritten.
- **C-2.** Names that differ only in case (`D404N`, `d404n`) both pass the
  check. On a case-insensitive filesystem the second file replaces the
  first, while the document reports both. Names may also hold characters
  that aren't portable in file names.

## 2. Required change (`write.rs`)

1. **Name rules, before anything is created** (right after the existing
   separator check):
   - a name holding any of `:` `*` `?` `<` `>` `|` is refused with the
     existing `WriteFailure::NotAFileName(name)`;
   - add a variant `WriteFailure::CaseClash(String)`, documented as: two
     result names differ only in case, so their files would replace each
     other on a case-insensitive file system; nothing is created; the
     `String` is the later name. Walking the artifacts in order, a name whose
     ASCII-lowercased form equals an earlier name's ASCII-lowercased form is
     refused with it.
2. **Existence check:** replace `path.try_exists()` with
   `fs::symlink_metadata(path)`:
   - `Ok(_)` (a file, a directory, or any symlink, dangling or not) →
     `WriteFailure::Exists(file_name)`;
   - `Err` with kind `NotFound` → free;
   - any other `Err` → `WriteFailure::Io(message)`.
3. **The write itself:** replace `fs::write` with
   `fs::OpenOptions::new().write(true).create_new(true).open(path)` followed
   by `write_all` of `render_fasta(name, sequence)`. An open error of kind
   `AlreadyExists` → `WriteFailure::Exists(file_name)`; any other error →
   `WriteFailure::Io(message)`. `create_new` refuses an existing path,
   symlinks included, atomically, so a file that appears after the check is
   still never replaced.
4. Update the module doc comment's bullets and the `NotAFileName` doc to
   state the new rules: the extra characters, the case-folded clash, and
   that a path holding anything at all, a dangling symlink included, is
   never written through or replaced.

Keep the up-front check of every path (step 2), so the all-or-nothing
behavior for an ordinary existing file stays.

## 2a. Required change (`run.rs`, `write_outcome`)

- The `NotAFileName` message becomes
  `"the result name '{}' cannot be a file name: it contains a path separator or one of : * ? < > |"`.
- Add an arm for `CaseClash(name)`, shaped exactly like the `NotAFileName`
  arm (code `output_name_unusable`, category `malformed_input`, path
  `None`, details `[("name", name)]`, exit `MalformedInput`), with the
  message
  `"the result name '{}' differs from an earlier one only in case, so their files would collide"`.
- Update the comments above both arms, and the module doc's
  `--write-edited` bullet, to match.

## 3. Tests (write first; confirm they fail before the change)

In `crates/cli/tests/write_edited.rs`, following its existing style and its
scratch-directory helper. Call `write_edited` directly if the existing tests
do, otherwise run the binary as they do.
1. `#[cfg(unix)]` **dangling symlink:** in DIR, create
   `std::os::unix::fs::symlink("../elsewhere.fa", DIR/"<name>.fa")`, where
   `elsewhere.fa` does not exist. Writing that name fails with `Exists`
   (`output_exists` through the binary), and `elsewhere.fa` still does not
   exist afterwards.
2. **Case clash:** two members `D404N` and `d404n` fail with
   `CaseClash("d404n")` (`output_name_unusable` through the binary), and
   DIR is not created.
3. **Unportable characters:** for each of `:` `*` `?` `<` `>` `|`, a name
   `a<c>b` fails with `NotAFileName`, and DIR is not created. Identifiers
   may refuse some of these characters before `write_edited` is reached; if
   so, test only the ones an `Identifier` accepts, and list which in the
   handoff.
4. The existing tests still pass unchanged.

## 4. Do not

- Add dependencies.
- Change any other file or the CLI error codes.
- Change any existing test, except that an existing assertion on the exact
  `NotAFileName` message text may be updated to the new text; list it.
- Refuse Windows reserved names.

## 5. Checks

From the repository root, after `export TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 6. Handoff

Report the changed files, each new test and whether it failed before the
change, the check results, and any deviation.
