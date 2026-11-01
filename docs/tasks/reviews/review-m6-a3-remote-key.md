---
type: Task Contract
title: "Review M6: no API key to a remote endpoint over plain HTTP (D045)"
description: "With --allow-remote, LURIA_OPENAI_API_KEY is sent as a bearer token, unencrypted, to a host that is not this machine: this build has no TLS (M6-18)."
status: deprecated
tags: [milestone-6]
---
# Review M6: no API key to a remote endpoint over plain HTTP (D045)

Status: **Implemented 2026-10-21** by the local worker; accepted on planner
review. Planner: Claude. Parent:
[docs/tasks/reviews/review-post-m6.md](/tasks/reviews/review-post-m6.md), reviewer A's question. Owner answer,
2026-10-20: "Refuse key + remote".

Files (no others):
- `crates/cli/src/args.rs`
- `crates/cli/src/main.rs`
- `crates/cli/tests/interpret_cli.rs`
- [docs/specs/cli.md](/specs/cli.md) (one sentence, §2.3)

## 1. Defect

With `--allow-remote`, `LURIA_OPENAI_API_KEY` is sent as a bearer token,
unencrypted, to a host that is not this machine: this build has no TLS
(M6-18).

## 2. Required change

### 2.1 `args.rs`

```rust
/// The bearer key to send (D045): `key` unchanged, unless it is non-empty
/// and the endpoint is not loopback, which is refused, because this build
/// speaks plain HTTP only (M6-18) and the key would cross the network
/// unencrypted. An empty or absent key is never refused.
pub fn checked_key(endpoint: &Endpoint, key: Option<String>) -> Result<Option<String>, UsageError>
```

The refusal's message, exactly:
`format!("LURIA_OPENAI_API_KEY is set, and the endpoint {} is not on this machine; luria does not send a key over plain HTTP to another machine (D045)", endpoint.url())`.

### 2.2 `main.rs`

In `emit_interpret`, after `checked_endpoint` succeeds, pass
`env::var("LURIA_OPENAI_API_KEY").ok()` through `checked_key`. An `Err` is
`report_usage(problem)` (exit 2), before anything is read or sent; `Ok(key)`
goes to `OpenAiCompatible::new` as the key does today. Import
`checked_key` beside `checked_endpoint`. Update `emit_interpret`'s doc
comment to say the key is refused for a remote endpoint (D045).

### 2.3 [docs/specs/cli.md](/specs/cli.md) §10.3

After the sentence that ends `it is sent as a bearer token and appears in
no document or message.`, add exactly: `luria refuses to send it to an
endpoint that is not this machine, because this build has no HTTPS.`

## 3. Tests (write first; confirm each new one fails before the change)

1. `args.rs` test module, `a_key_is_sent_to_this_machine_only`, with
   endpoints from `checked_endpoint(..., true)`:
   - `http://127.0.0.1:8080/v1` with `Some("sk-1")` → `Ok(Some("sk-1"))`;
   - `http://gpu.lan:8080/v1` with `None` → `Ok(None)`;
   - `http://gpu.lan:8080/v1` with `Some("")` → `Ok(Some(""))`;
   - `http://gpu.lan:8080/v1` with `Some("sk-1")` → `Err` with the message
     of §2.1 for `http://gpu.lan:8080/v1`.
2. `crates/cli/tests/interpret_cli.rs`, two tests using the file's own
   `workspace`, `luria` and `completion` helpers and the `two-separate`
   corpus text, profile and reply, as
   `words_become_a_request_over_http_and_the_request_runs` does:
   - `a_key_is_not_sent_to_another_machine`: bind a `TcpListener` on
     `127.0.0.1:0` yourself, and spawn a thread that accepts **one**
     connection, sends `()` on a `std::sync::mpsc` channel, then reads the
     request and answers with the two-separate completion (as the file's
     `stub` does). Run luria with `--endpoint http://0.0.0.0:<port>/v1
     --allow-remote` and key `Some("sk-remote")`. (`0.0.0.0` is not
     loopback to luria's gate, but on Linux a connection to it reaches
     this machine, so the test can see whether anything was sent.) Expect
     exit 2, empty stdout, stderr containing
     `luria does not send a key over plain HTTP`, and the channel's
     `try_recv()` giving `Err(TryRecvError::Empty)`. Do not join the
     thread: it stays blocked in `accept` when nothing is sent.
   - `a_remote_endpoint_without_a_key_is_still_called`: the same with key
     `None`, served by the file's `stub` with the two-separate completion →
     exit 0.

   If a connection to `0.0.0.0` does not reach the listener in this
   environment, stop and report; do not pick another address.

Disabling proof: skip the `checked_key` call in `main.rs` → the first
integration test fails (exit 0, and the listener sees a connection).
Restore it, and report.

## 4. Do not

- Add dependencies, or change any file not listed.
- Change `checked_endpoint`, the backend, or any existing test.

## 5. Checks

From the repository root, after `export TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 6. Handoff

Report the changed files, each new test and whether it failed before the
change, the disabling proof and its result, the check results, and any
deviation.
