# Review M6 A-2, A-4 to A-8: the HTTP backend and the endpoint

Status: **Ready for the worker.** Planner: Claude. Parent:
`docs/tasks/review-post-m6.md` A-2, A-4, A-5, A-6, A-7, A-8. Owner,
2026-10-20: "Go ahead and fix them".

Files (no others):
- `crates/nl/src/openai.rs`
- `crates/nl/src/endpoint.rs`
- `crates/nl/tests/openai.rs`
- `crates/nl/tests/proxy.rs` (new)

## 1. Defects

- A-2: `.proxy(None)` in `agent_of` (`openai.rs`) is what stops
  `HTTP_PROXY` / `ALL_PROXY` from sending the prompt and the key to a host
  the loopback gate never checked. No test covers it: removing it leaves
  every test green.
- A-4: a 200 body that is not UTF-8 is `BackendErrorKind::Unreachable`
  ("the response broke off"), so the CLI calls it a network failure. The
  server answered; the answer is bad: it is `Response`.
- A-5: `Endpoint::parse` accepts path characters a URL cannot carry
  (`{ } | ^ \ " < >`, non-ASCII), so the mistake surfaces at send time as
  `backend_unreachable` instead of as a usage error.
- A-6: the module doc says "no name is resolved", but `localhost` is
  accepted by name and then resolved by the system resolver.
- A-7: adversarial hosts that are handled correctly today have no
  regression test.
- A-8: the response-cap test would pass if the body were read whole and
  checked afterwards.

## 2. Required change

### 2.1 A-4, `openai.rs`

Read the body as bytes with the same limit
(`.with_config().limit(MAX_RESPONSE_BYTES).read_to_vec()`), keeping the
`BodyExceedsLimit` arm as it is and every other read error as
`Unreachable` ("the response broke off: …") as now. Then
`String::from_utf8(bytes)`: on failure,
`BackendError { kind: BackendErrorKind::Response, message: "the response is not UTF-8".to_string() }`.
The status handling after that is unchanged.

### 2.2 A-5, `endpoint.rs`

After the existing whitespace-and-control check of the path, refuse any
path char that is not an ASCII letter, an ASCII digit, or one of
`- . _ ~ ! $ & ' ( ) * + , ; = : @ / %`, and any `%` not followed by two
ASCII hex digits, with a new refusal:

```rust
/// Why the path holds a character a URL path cannot carry.
fn path_char_refusal() -> EndpointError {
    EndpointError {
        message: "an endpoint path may hold only the characters a URL path allows".to_string(),
    }
}
```

The order of the existing checks is unchanged; this one comes right after
the whitespace check, before the host is read. Chars, never bytes.

### 2.3 A-6, `endpoint.rs`

Rewrite the module doc's sentence "No name is resolved: …" and the docs of
`is_loopback` and `local` to say exactly what happens: the gate decides from
the text alone, and the one name it accepts as loopback, `localhost`, is
then resolved by the system's resolver when the call is made (normally
`/etc/hosts`); every other name is remote. No behavior changes.

### 2.4 Tests

Existing tests unchanged, and they must pass.

**`crates/nl/tests/proxy.rs` (new, A-2).** Its own test binary, because it
sets environment variables; it holds one `#[test]`,
`a_proxy_in_the_environment_is_not_used`:
1. bind two `TcpListener`s on `127.0.0.1:0`: the real server and the
   "proxy";
2. set `HTTP_PROXY`, `http_proxy`, `ALL_PROXY` and `all_proxy` to
   `http://127.0.0.1:<proxy port>` with `std::env::set_var`;
3. serve one complete chat completion from the real server on a thread
   (copy the stub style of `tests/openai.rs`; a helper may be duplicated);
4. call `OpenAiCompatible::new(Endpoint::parse("http://127.0.0.1:<real port>/v1"), "m", None).complete(&prompt)`
   and assert it is `Ok`;
5. set the proxy listener non-blocking and assert `accept()` gives
   `ErrorKind::WouldBlock`: no connection reached it.

**`crates/nl/tests/openai.rs`.**
- `a_body_that_is_not_utf8_is_a_bad_response` (A-4): a stub answering
  `200` with `Content-Length: 4` and body bytes `[0xff, 0xfe, 0x41, 0x42]`
  → `Err` with kind `BackendErrorKind::Response` and message
  `"the response is not UTF-8"`. (The stub must write bytes, not a
  `String`.)
- `an_endless_body_is_cut_off_while_it_is_read` (A-8): a stub answering
  `200` with `Transfer-Encoding: chunked`, then writing 64 KiB chunks in a
  loop until a write fails → the call is `Err` with kind `Response` and a
  message containing `"larger than"`, and the call returns (the test
  itself finishing is the proof; do not join the stub thread before
  asserting).

**`endpoint.rs` test module.**
- `a_path_char_a_url_cannot_carry_is_refused` (A-5): each of
  `http://127.0.0.1/v1{}`, `http://127.0.0.1/v1|x`, `http://127.0.0.1/v1^`,
  `http://127.0.0.1/v1\x`, `http://127.0.0.1/v"1`, `http://127.0.0.1/v<1>`,
  `http://127.0.0.1/v\u{e9}` (the char é), `http://127.0.0.1/v1%`,
  `http://127.0.0.1/v1%4`,
  `http://127.0.0.1/v1%zz` (write the backslash case as a raw string,
  `r"http://127.0.0.1/v1\x"`) is `Err` with message
  `"an endpoint path may hold only the characters a URL path allows"`; and
  each of `http://127.0.0.1/v1`, `http://127.0.0.1/a-b.c_d~e`,
  `http://127.0.0.1/v1/%41`, `http://127.0.0.1/x!$&'()*+,;=:@` is `Ok`.
- `no_disguised_host_is_loopback` (A-7): for each of `127.0.0.1.evil.com`,
  `localhost.`, `[::ffff:127.0.0.1]`, `127.0.0.1\@evil.com`,
  `evil.com@127.0.0.1`, `2130706433`, `0177.0.0.1`, `0x7f.0.0.1`,
  `[fe80::1%25eth0]`, `0.0.0.0`, `127.1`, the URL `http://<host>/v1` either
  fails to parse or parses with `is_loopback() == false`.

Disabling proofs (once each, run, restore, report): remove `.proxy(None)`
→ `proxy.rs` fails; read with `read_to_string` again → the not-UTF-8 test
fails; skip the new path check → the path test fails. No disabling proof
for A-8: reading an endless body without the limit would grow memory
until the timeout. Do not try it.

## 3. Do not

- Add dependencies or change `Cargo.toml`.
- Change any file not listed, any existing message, or any existing test.
- Change the timeout, the caps, or what is sent.

## 4. Checks

From the repository root, after `export TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 5. Handoff

Report the changed files, each new test and whether it failed before the
change (A-2, A-7 and A-8 cover behavior that is already right: say they
pass before and after), each disabling proof and its result, the check
results, and any deviation.
