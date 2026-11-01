---
type: Task Contract
title: "Milestone 6g1: the OpenAI-compatible backend, over plain HTTP"
description: "This is the first code that sends anything off the process: the prompt, to a model served behind an OpenAI-compatible /chat/completions endpoint (llama.cpp, Ollama, vLLM and others)."
status: deprecated
tags: [milestone-6]
---
# Milestone 6g1: the OpenAI-compatible backend, over plain HTTP

Status: **Implemented 2026-10-19** by the local worker; accepted on the first
review. `cargo tree -p luria-nl` holds neither `ring` nor `rustls`.
Planner: Claude. Parent: [docs/tasks/contracts/milestone-6-nl.md](/tasks/contracts/milestone-6-nl.md) §3 ("Cloud use is
explicit"), §5.6, M6-1, M6-2, M6-17, M6-18, §6 (6g1).

## 1. What and why

This is the first code that sends anything off the process: the prompt,
to a model served behind an OpenAI-compatible `/chat/completions` endpoint
(llama.cpp, Ollama, vLLM and others). Two pieces:

- **`Endpoint`**: the base URL a user gives, read strictly, so the command
  (6h) can tell a loopback address from a remote one *before* sending, and
  refuse `https://`, which this build cannot speak (M6-18);
- **`OpenAiCompatible`**: the `Backend` (6f) that sends the prompt and
  returns the reply text unparsed.

Every test runs against a stub server on `127.0.0.1` inside the test. No
test uses the network or a real model.

## 2. Requirements this contract applies

- **M6-2, M6-18:** `ureq` 3, with its default features off: plain HTTP
  only, no TLS, no `ring`, no `rustls`. It builds offline from the local
  cargo cache.
- **Plan §5.6:** `POST {endpoint}/chat/completions`; `temperature` 0; a
  seed; the model named by the caller, never defaulted; an optional bearer
  key, given by the caller (6h reads it from the environment: `luria-nl`
  reads no environment).
- **M6-17:** no thinking switch: no `chat_template_kwargs`, no reasoning
  field. No `max_tokens` either: the server's own settings decide.
- **Plan §5.2:** the extraction schema is sent as
  `response_format: json_schema`, so a server that supports it constrains
  the reply. The reply text is returned exactly as the model wrote it.
- **What leaves the machine is only the prompt.** The request is never
  redirected elsewhere (a redirect would re-send the prompt to another
  host), never routed through a proxy taken from the environment, and the
  key is never in an error message or in `describe`.
- **Bounded:** a 600-second timeout for the whole call, and a response of
  at most 1 MiB; larger is refused, not read.

The planner verified on 2026-10-19, in a scratch crate against `ureq`
3.4.1 offline, every `ureq` call named in §3.3: `Agent::config_builder()`
with `timeout_global`, `http_status_as_error(false)`, `proxy(None)` and
`max_redirects(0)`; `.post(url).header(..).send(String)`;
`response.status().as_u16()`; and
`response.body_mut().with_config().limit(n).read_to_string()`, which fails
with `ureq::Error::BodyExceedsLimit(_)` past the limit. With
`max_redirects(0)` a `307` is returned as a status after exactly one
connection.

## 3. Files

### 3.1 `crates/nl/Cargo.toml`

Add to `[dependencies]`:

```toml
ureq = { version = "3", default-features = false }
```

Nothing else. `Cargo.lock` gains `ureq` and its dependencies. Build with
`--offline` if cargo cannot reach the network.

### 3.2 `crates/nl/src/lib.rs`

The module list becomes `backend`, `endpoint`, `extraction`, `openai`,
`prompt`, in that order. Change nothing else.

### 3.3 `crates/nl/src/endpoint.rs` (new)

```rust
/// A model endpoint's base URL, read strictly (milestone 6 plan §3, §5.6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoint { /* host, port, path: private */ }

/// Why a URL is not an endpoint luria will call. One line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndpointError {
    pub message: String,
}

impl Endpoint {
    pub fn parse(url: &str) -> Result<Endpoint, EndpointError>;
    /// The host as stored: ASCII lower case, an IPv6 address in brackets.
    pub fn host(&self) -> &str;
    /// Whether the host is this machine, decided from the text alone: no
    /// name is resolved.
    pub fn is_loopback(&self) -> bool;
    /// `http://` + host + `:port` when one was given + path.
    pub fn url(&self) -> String;
    /// `url()` followed by `suffix`, which starts with `/`.
    pub fn join(&self, suffix: &str) -> String;
}
```

`parse`, in order; the first failure is returned with exactly this message:

1. The URL must start with `http://`, ASCII case-insensitively. If it starts
   with `https://` instead: `https endpoints are not supported in this
   build; use http:// (M6-18)`. Otherwise: `an endpoint must start with
   http://`.
2. The rest splits at its first `/` into the *authority* and the *path*
   (the path keeps its leading `/`; with no `/`, the path is empty).
3. A `@` anywhere in the authority: `an endpoint may not carry a user name
   or password`.
4. A `?` or `#` anywhere in the authority or the path: `an endpoint may not
   carry a query or a fragment`.
5. Any char of the path that is whitespace or a control char: `an endpoint
   path may not hold spaces or control characters`.
6. The host and port. If the authority starts with `[`, the host runs to
   the first `]`, brackets included; the text inside must parse as
   `std::net::Ipv6Addr`; after the `]` comes nothing, or `:` and the port.
   Otherwise the authority is the host, or host `:` port when it holds
   exactly one `:`; more than one `:` is refused. A non-bracketed host must
   be non-empty and only ASCII letters, digits, `-` and `.`. Any failure
   here: `the endpoint's host is not valid`.
7. A port is 1 to 5 ASCII digits with a value from 1 to 65535; anything
   else, including an empty port after `:`: `the endpoint's port is not
   valid`.
8. Store the host ASCII-lower-cased, and the path with every trailing `/`
   removed.

`is_loopback` is true exactly when the host is `localhost`; or parses as
`std::net::Ipv4Addr` and `is_loopback()` is true (`127.0.0.0/8`); or is
bracketed and the address inside `is_loopback()` (`::1`). Everything else
is remote, including `0.0.0.0`, short forms such as `127.1`, and any name
other than `localhost`.

### 3.4 `crates/nl/src/openai.rs` (new)

```rust
use crate::backend::{Backend, BackendError, BackendErrorKind, BackendInfo, Reply};
use crate::endpoint::Endpoint;
use crate::prompt::{Prompt, EXTRACTION_SCHEMA};

/// The longest a whole call may take, in seconds. A slow local model
/// answers in seconds; this bounds a server that never does.
pub const REQUEST_TIMEOUT_SECONDS: u64 = 600;
/// The largest response read, in bytes. A chat completion around one
/// extraction is far smaller; larger is refused, not read.
pub const MAX_RESPONSE_BYTES: u64 = 1024 * 1024;
/// The seed sent with every request (plan §5.6).
pub const SEED: u64 = 1;

/// A model behind an OpenAI-compatible `/chat/completions` endpoint.
/// Deliberately not `Debug`: it holds the key.
pub struct OpenAiCompatible { /* endpoint, model, key, agent: private */ }

impl OpenAiCompatible {
    /// `key` is sent as a bearer token when it is `Some` and not empty.
    pub fn new(endpoint: Endpoint, model: &str, key: Option<String>) -> OpenAiCompatible;
}

impl Backend for OpenAiCompatible { /* §3.5 */ }
```

The agent is built once in `new`:
`ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))).http_status_as_error(false).proxy(None).max_redirects(0).build().into()`.

The request body is private `#[derive(Serialize)]` structs, serialized
with `serde_json::to_string`, in this field order:

```json
{
  "model": "<model>",
  "temperature": 0,
  "seed": 1,
  "messages": [
    {"role": "system", "content": "<prompt.system>"},
    {"role": "user", "content": "<prompt.user>"}
  ],
  "response_format": {
    "type": "json_schema",
    "json_schema": {"name": "extraction", "strict": true, "schema": <EXTRACTION_SCHEMA, as JSON>}
  }
}
```

`temperature` is the integer `0`. The schema is `EXTRACTION_SCHEMA` parsed
into a `serde_json::Value`. No other field is ever sent.

### 3.5 `complete`, in order

1. Parse `EXTRACTION_SCHEMA`; on failure, `Response`: `the extraction
   schema is not JSON`. Serialize the body; on failure, `Response`: `the
   request could not be written`.
2. `agent.post(endpoint.join("/chat/completions"))` with the header
   `Content-Type: application/json` and, when the key is `Some` and not
   empty, `Authorization: Bearer <key>`. Send the body with `.send(body)`.
3. A send error: `Unreachable`, message `could not reach <url>: <error>`,
   `<url>` being `endpoint.url()` and `<error>` the `ureq` error's
   `Display`.
4. Read the status, then the body as text with
   `.with_config().limit(MAX_RESPONSE_BYTES).read_to_string()`.
   `ureq::Error::BodyExceedsLimit(_)`: `Response`, `the response is larger
   than 1048576 bytes`. Any other read error: `Unreachable`, `the response
   broke off: <error>`.
5. A status outside 200 to 299: `Status`, message `the server answered
   HTTP <status>`, followed by `: <excerpt>` when the body is not empty,
   where `<excerpt>` is the body's first 200 chars with every control char
   replaced by a space.
6. Parse the body with private, lenient `#[derive(Deserialize)]` structs
   (unknown fields allowed: a server sends many): `choices`, each with a
   `message` holding `content: Option<String>`. Not that shape: `Response`,
   `the response is not a chat completion: <serde message>`. No choice:
   `Response`, `the response has no choices`. The first choice's content
   `None`, or empty after `trim`: `Response`, `the response's message has
   no content`.
7. Return `Reply { text: content }`, the content exactly as received.

`describe` returns `kind: "openai-compatible"`, `endpoint:
Some(endpoint.url())` and the model. No message and no `describe` field
ever holds the key.

## 4. Tests (planner-written; add verbatim)

### 4.1 Unit tests, at the end of `crates/nl/src/endpoint.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(url: &str) -> Endpoint {
        Endpoint::parse(url).unwrap_or_else(|error| panic!("{:?}: {}", url, error.message))
    }

    fn refusal(url: &str) -> String {
        match Endpoint::parse(url) {
            Err(error) => error.message,
            Ok(found) => panic!("{:?} parsed as {:?}", url, found),
        }
    }

    #[test]
    fn an_endpoint_is_normalized() {
        let cases: [(&str, &str, &str); 7] = [
            ("http://127.0.0.1:8080/v1", "127.0.0.1", "http://127.0.0.1:8080/v1"),
            ("http://127.0.0.1:8080/v1/", "127.0.0.1", "http://127.0.0.1:8080/v1"),
            ("HTTP://LocalHost:11434/v1//", "localhost", "http://localhost:11434/v1"),
            ("http://[::1]:8080/v1", "[::1]", "http://[::1]:8080/v1"),
            ("http://gpu-box.lan", "gpu-box.lan", "http://gpu-box.lan"),
            ("http://10.0.0.7:80", "10.0.0.7", "http://10.0.0.7:80"),
            ("http://localhost/", "localhost", "http://localhost"),
        ];
        for case in cases {
            let endpoint: Endpoint = parsed(case.0);
            assert_eq!(endpoint.host(), case.1, "{}", case.0);
            assert_eq!(endpoint.url(), case.2, "{}", case.0);
        }
        assert_eq!(
            parsed("http://127.0.0.1:8080/v1/").join("/chat/completions"),
            "http://127.0.0.1:8080/v1/chat/completions"
        );
    }

    #[test]
    fn loopback_is_decided_from_the_text_alone() {
        let local: [&str; 6] = [
            "http://localhost:8080",
            "http://LOCALHOST",
            "http://127.0.0.1:8080",
            "http://127.255.0.1",
            "http://[::1]",
            "http://[0:0:0:0:0:0:0:1]:8080",
        ];
        for url in local {
            assert!(parsed(url).is_loopback(), "{} is local", url);
        }
        let remote: [&str; 8] = [
            "http://192.168.1.5:8080",
            "http://10.0.0.1",
            "http://0.0.0.0:8080",
            "http://127.1",
            "http://example.com",
            "http://localhost.example.com",
            "http://[::2]",
            "http://my-localhost",
        ];
        for url in remote {
            assert!(!parsed(url).is_loopback(), "{} is remote", url);
        }
    }

    #[test]
    fn each_unusable_url_is_refused_with_its_reason() {
        let cases: [(&str, &str); 16] = [
            ("https://api.example.com/v1", "https endpoints are not supported in this build; use http:// (M6-18)"),
            ("HTTPS://127.0.0.1", "https endpoints are not supported in this build; use http:// (M6-18)"),
            ("127.0.0.1:8080/v1", "an endpoint must start with http://"),
            ("ftp://127.0.0.1", "an endpoint must start with http://"),
            ("", "an endpoint must start with http://"),
            ("http://user:pass@127.0.0.1/v1", "an endpoint may not carry a user name or password"),
            ("http://127.0.0.1/v1?key=1", "an endpoint may not carry a query or a fragment"),
            ("http://127.0.0.1/v1#x", "an endpoint may not carry a query or a fragment"),
            ("http://127.0.0.1/v 1", "an endpoint path may not hold spaces or control characters"),
            ("http://", "the endpoint's host is not valid"),
            ("http://:8080/v1", "the endpoint's host is not valid"),
            ("http://bad_host/v1", "the endpoint's host is not valid"),
            ("http://[::1/v1", "the endpoint's host is not valid"),
            ("http://a:1:2", "the endpoint's host is not valid"),
            ("http://127.0.0.1:0", "the endpoint's port is not valid"),
            ("http://127.0.0.1:65536", "the endpoint's port is not valid"),
        ];
        for case in cases {
            assert_eq!(refusal(case.0), case.1, "{:?}", case.0);
        }
        assert_eq!(refusal("http://127.0.0.1:"), "the endpoint's port is not valid");
        assert_eq!(refusal("http://h\u{e9}te"), "the endpoint's host is not valid");
    }
}
```

### 4.2 Integration test: `crates/nl/tests/openai.rs`

```rust
//! The OpenAI-compatible backend against a stub server on 127.0.0.1
//! (contract 6g1). No test uses the network or a model.

use luria_nl::backend::Backend;
use luria_nl::backend::BackendError;
use luria_nl::backend::BackendErrorKind;
use luria_nl::backend::BackendInfo;
use luria_nl::backend::Reply;
use luria_nl::endpoint::Endpoint;
use luria_nl::openai::OpenAiCompatible;
use luria_nl::prompt::Prompt;
use luria_nl::prompt::EXTRACTION_SCHEMA;
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::thread::JoinHandle;

/// What the stub saw of the one request it was sent.
struct Captured {
    request_line: String,
    headers: Vec<(String, String)>,
    body: String,
}

impl Captured {
    fn header(&self, name: &str) -> Option<String> {
        self.headers
            .iter()
            .find(|header| header.0 == name)
            .map(|header| header.1.clone())
    }
}

/// A server on a free local port that answers one request with `response`
/// and reports what it was sent. Returns the endpoint URL, `/v1` included.
fn stub(response: String) -> (String, JoinHandle<Captured>) {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("a free port");
    let port: u16 = listener.local_addr().expect("an address").port();
    let handle: JoinHandle<Captured> = std::thread::spawn(move || {
        let (stream, _) = listener.accept().expect("one connection");
        let mut reader = BufReader::new(stream.try_clone().expect("a second handle"));
        let mut request_line: String = String::new();
        reader.read_line(&mut request_line).expect("a request line");
        let mut headers: Vec<(String, String)> = Vec::new();
        let mut length: usize = 0;
        loop {
            let mut line: String = String::new();
            reader.read_line(&mut line).expect("a header line");
            if line == "\r\n" || line.is_empty() {
                break;
            }
            let (name, value) = line.split_once(':').expect("a header");
            let name: String = name.trim().to_ascii_lowercase();
            let value: String = value.trim().to_string();
            if name == "content-length" {
                length = value.parse().expect("a length");
            }
            headers.push((name, value));
        }
        let mut body: Vec<u8> = vec![0; length];
        reader.read_exact(&mut body).expect("the body");
        let mut writer = stream;
        // The client may stop reading early (the size limit); that is not
        // the stub's failure.
        let _ = writer.write_all(response.as_bytes());
        Captured {
            request_line: request_line.trim_end().to_string(),
            headers,
            body: String::from_utf8(body).expect("a UTF-8 body"),
        }
    });
    (format!("http://127.0.0.1:{}/v1", port), handle)
}

fn http(status: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        body.len(),
        body
    )
}

/// A chat completion whose message content is `content`.
fn completion(content: &str) -> String {
    serde_json::json!({
        "id": "x",
        "object": "chat.completion",
        "choices": [{"index": 0, "message": {"role": "assistant", "content": content,
            "reasoning_content": "ignored"}, "finish_reason": "stop"}],
        "usage": {"prompt_tokens": 1}
    })
    .to_string()
}

fn backend(url: &str, key: Option<&str>) -> OpenAiCompatible {
    let endpoint: Endpoint = Endpoint::parse(url).expect("a stub endpoint");
    OpenAiCompatible::new(endpoint, "qwen-test", key.map(|key| key.to_string()))
}

fn asked() -> Prompt {
    Prompt {
        system: "the system message".to_string(),
        user: "the user message".to_string(),
    }
}

#[test]
fn the_request_is_exactly_what_the_plan_says() {
    let (url, server) = stub(http("200 OK", &completion("{}")));
    let reply = backend(&url, None).complete(&asked());
    let seen: Captured = server.join().expect("the stub ran");
    assert_eq!(reply, Ok(Reply { text: "{}".to_string() }));
    assert_eq!(seen.request_line, "POST /v1/chat/completions HTTP/1.1");
    assert_eq!(seen.header("content-type"), Some("application/json".to_string()));
    assert_eq!(seen.header("authorization"), None);
    let body: Value = serde_json::from_str(&seen.body).expect("a JSON body");
    let schema: Value = serde_json::from_str(EXTRACTION_SCHEMA).expect("the schema is JSON");
    assert_eq!(
        body,
        serde_json::json!({
            "model": "qwen-test",
            "temperature": 0,
            "seed": 1,
            "messages": [
                {"role": "system", "content": "the system message"},
                {"role": "user", "content": "the user message"}
            ],
            "response_format": {
                "type": "json_schema",
                "json_schema": {"name": "extraction", "strict": true, "schema": schema}
            }
        })
    );
    assert!(seen.body.starts_with("{\"model\":\"qwen-test\",\"temperature\":0,\"seed\":1,"));
}

#[test]
fn a_key_is_sent_as_a_bearer_token_and_never_described() {
    let (url, server) = stub(http("200 OK", &completion("ok")));
    let mut keyed: OpenAiCompatible = backend(&url, Some("sk-secret"));
    assert_eq!(keyed.complete(&asked()).map(|reply| reply.text), Ok("ok".to_string()));
    let seen: Captured = server.join().expect("the stub ran");
    assert_eq!(seen.header("authorization"), Some("Bearer sk-secret".to_string()));
    assert_eq!(
        keyed.describe(),
        BackendInfo {
            kind: "openai-compatible".to_string(),
            endpoint: Some(url.clone()),
            model: "qwen-test".to_string(),
        }
    );
    let (url, server) = stub(http("200 OK", &completion("ok")));
    let _ = backend(&url, Some("")).complete(&asked());
    assert_eq!(server.join().expect("the stub ran").header("authorization"), None);
}

#[test]
fn the_content_is_returned_exactly_as_written() {
    let content: &str = "  {\"operation\": \"other\"}\n";
    let (url, server) = stub(http("200 OK", &completion(content)));
    let reply = backend(&url, None).complete(&asked());
    server.join().expect("the stub ran");
    assert_eq!(reply, Ok(Reply { text: content.to_string() }));
}

#[test]
fn a_status_other_than_success_is_reported_with_its_body() {
    let (url, server) = stub(http("503 Service Unavailable", "{\"error\":\n\"loading model\"}"));
    let reply = backend(&url, Some("sk-secret")).complete(&asked());
    server.join().expect("the stub ran");
    assert_eq!(
        reply,
        Err(BackendError {
            kind: BackendErrorKind::Status,
            message: "the server answered HTTP 503: {\"error\": \"loading model\"}".to_string(),
        })
    );
    let (url, server) = stub(http("404 Not Found", ""));
    let reply = backend(&url, None).complete(&asked());
    server.join().expect("the stub ran");
    assert_eq!(
        reply.map_err(|error| error.message),
        Err("the server answered HTTP 404".to_string())
    );
}

// A redirect would re-send the prompt to another host: it is never followed.
#[test]
fn a_redirect_is_not_followed() {
    let redirect: String = "HTTP/1.1 307 Temporary Redirect\r\nLocation: http://127.0.0.1:9/v1/chat/completions\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string();
    let (url, server) = stub(redirect);
    let reply = backend(&url, None).complete(&asked());
    server.join().expect("the stub ran");
    assert_eq!(
        reply,
        Err(BackendError {
            kind: BackendErrorKind::Status,
            message: "the server answered HTTP 307".to_string(),
        })
    );
}

#[test]
fn a_reply_that_is_not_a_chat_completion_is_refused() {
    let cases: [(&str, &str); 5] = [
        ("not json", "the response is not a chat completion: "),
        ("{\"choices\": \"none\"}", "the response is not a chat completion: "),
        ("{\"choices\": []}", "the response has no choices"),
        (
            "{\"choices\": [{\"message\": {\"content\": null}}]}",
            "the response's message has no content",
        ),
        (
            "{\"choices\": [{\"message\": {\"content\": \"  \"}}]}",
            "the response's message has no content",
        ),
    ];
    for case in cases {
        let (url, server) = stub(http("200 OK", case.0));
        let reply = backend(&url, None).complete(&asked());
        server.join().expect("the stub ran");
        match reply {
            Err(error) => {
                assert_eq!(error.kind, BackendErrorKind::Response, "{}", case.0);
                assert!(error.message.starts_with(case.1), "{}: {:?}", case.0, error.message);
            }
            Ok(found) => panic!("{} gave {:?}", case.0, found),
        }
    }
}

#[test]
fn a_response_past_the_limit_is_refused_not_read() {
    let padding: String = "x".repeat(1024 * 1024 + 1);
    let (url, server) = stub(http("200 OK", &completion(&padding)));
    let reply = backend(&url, None).complete(&asked());
    let _ = server.join();
    assert_eq!(
        reply,
        Err(BackendError {
            kind: BackendErrorKind::Response,
            message: "the response is larger than 1048576 bytes".to_string(),
        })
    );
}

#[test]
fn a_server_that_is_not_there_is_unreachable() {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("a free port");
    let port: u16 = listener.local_addr().expect("an address").port();
    drop(listener);
    let url: String = format!("http://127.0.0.1:{}/v1", port);
    match backend(&url, Some("sk-secret")).complete(&asked()) {
        Err(error) => {
            assert_eq!(error.kind, BackendErrorKind::Unreachable);
            assert!(
                error.message.starts_with(&format!("could not reach {}: ", url)),
                "{:?}",
                error.message
            );
            assert!(!error.message.contains("sk-secret"));
        }
        Ok(found) => panic!("a closed port answered {:?}", found),
    }
}
```

`cargo fmt` may reflow these; do not change content. If a test fails,
report what the code returned. If a planner-written test fails to compile
or fails clippy, stop and report it rather than editing it.

### 4.3 Proving the rules are load-bearing (AGENTS.md)

Disable each once, confirm the named test fails, restore, report both:

| Rule, disabled | Test that must fail |
| --- | --- |
| `max_redirects(0)` (leave the default) | `a_redirect_is_not_followed` |
| the response limit (read without `limit`) | `a_response_past_the_limit_is_refused_not_read` |
| the `https://` message (treat it as any other scheme) | `each_unusable_url_is_refused_with_its_reason` |
| `127.0.0.0/8` is loopback (accept only `127.0.0.1`) | `loopback_is_decided_from_the_text_alone` |
| an empty key sends no header | `a_key_is_sent_as_a_bearer_token_and_never_described` |

## 5. Do not

- Enable any `ureq` feature, or add any dependency but `ureq`.
- Read the environment, a file or the clock in `crates/nl/src/`. The key
  and endpoint come from the caller.
- Derive or implement `Debug` for `OpenAiCompatible`, or put the key in any
  message, `describe` field or log.
- Send any field not in §3.4, follow a redirect, or use a proxy.
- Resolve a host name in `is_loopback`.
- Change `extraction.rs`, `prompt.rs`, `backend.rs`, any existing test,
  `crates/cli/`, `crates/core/`, `tests/fixtures/` or [docs/](/).

## 6. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`
- `cargo tree -p luria-nl -e normal`, which must list neither `ring` nor
  `rustls`.

## 7. Handoff

Report the new and changed files, the exact commands and their results,
the §4.3 results, the `cargo tree` output, and anything that did not match
this contract. Quote command output or the file for every claim; say
plainly what you did not check.
