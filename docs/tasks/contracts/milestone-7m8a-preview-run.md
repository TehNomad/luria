---
type: Task Contract
title: "Milestone 7m8a: run a proposed request without writing it"
description: "M7-60: when the model's reading can be approved, luria new shows the variant screens before anything is saved."
status: deprecated
tags: [milestone-7]
---
# Milestone 7m8a: run a proposed request without writing it

Status: **Implemented, 2026-10-22.** Planner: Claude. Parent:
[docs/tasks/contracts/milestone-7m-polish.md](/tasks/contracts/milestone-7m-polish.md), M7-60. Follows 7m7 (committed
`a65ca68`). 7m8b, the preview screens, builds on this.

## 1. Why

M7-60: when the model's reading can be approved, `luria new` shows the
variant screens before anything is saved. That needs the design run of
the proposed request with no file written. The preview must be the same
result the saved file later gives, byte for byte, including the request
digest. So it runs the exact bytes the save would write (`request_text`),
as though they were at the path the save will write.

## 2. Changes

### 2.1 `crates/cli/src/prepare.rs`

Add `pub fn prepare_bytes(request_path: &Path, contents: Vec<u8>) -> Result<Prepared, PrepareFailure>`.
It does what `prepare` does, but with `contents` in place of reading
`request_path`. `request_path` is used only for its directory, the one
the template paths are relative to (`directory_of`). Split `open` so the
digest-and-parse half takes bytes, and make `prepare` call
`prepare_bytes(request_path, bytes)` once it has read them. A read
failure stays `prepare`'s own. `prepare`'s behaviour does not change.

### 2.2 `crates/cli/src/run.rs`

Add `pub fn run_bytes_with_constructs(request_path: &Path, contents: Vec<u8>) -> (Outcome, Vec<(String, Sequence)>)`.
It is `run_with_constructs` with `prepare_bytes` in place of `prepare`.
Nothing is written.

### 2.3 `crates/cli/src/review.rs`

- Split `run_approved` so its body runs from a `(Outcome, constructs)`
  and a `Result<Prepared, _>`. Then add
  `pub fn preview(document: &InterpretationDocument, request: &Path) -> Result<Approved, String>`:
  - `Err("there is no proposed request to approve")` unless
    `proposal::approvable(document)`. Use the same check and words as
    `approve_replacing`.
  - `bytes = request_text(echo).into_bytes()`. Run
    `run_bytes_with_constructs(request, bytes.clone())`. The template
    letters and frames come from `prepare_bytes(request, bytes)`, exactly
    as `run_approved` gets them from `prepare(request)`.
  - Nothing is written, and no location check is made. The save makes
    that check later, as now.
- `run_approved(request)` keeps its signature and its behaviour.

## 3. Tests: `crates/cli/tests/review_approve.rs`

Use its existing helpers (`proposal_of`, corpus case `two-separate`).

1. `a_preview_is_the_saved_run`: `preview(&document, &request)` gives
   `Ok`. Then `approve(&document, &template, &request)` gives `Ok`. The
   two documents render the same (`render`), the request digests match,
   and both `template` and `frames` are equal.
2. `a_preview_writes_nothing`: after `preview`, `request` does not exist
   and the workspace directory lists the same entries as before.
3. `a_clarification_has_no_preview`: corpus case with state
   `needs_clarification`, the one `a_clarification_cannot_be_approved`
   uses. `preview` gives `Err("there is no proposed request to approve")`.

### 3.1 Rule that must be proven

Make `preview` hash something other than `request_text(echo)`'s bytes,
for example by appending `"\n"` to the bytes passed to
`run_bytes_with_constructs` only. Test 1 fails on the digest. Restore it.

## 4. Do not

- Change only `prepare.rs`, `run.rs`, `review.rs` and `review_approve.rs`
  (tests added only).
- No change to any command's behaviour, output or messages. All
  existing tests pass unchanged.
- No new dependency. Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result and the §3.1 proof.
