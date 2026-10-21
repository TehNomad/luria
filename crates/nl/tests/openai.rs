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

/// A server on a free local port that answers one request with the ASCII
/// `head` followed by the raw `body` bytes: a reply whose body is not UTF-8
/// cannot be written as a `String`, so the bytes are written as they are.
/// Returns the endpoint URL, `/v1` included, and the server's thread.
fn stub_raw(head: &str, body: &[u8]) -> (String, JoinHandle<()>) {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("a free port");
    let port: u16 = listener.local_addr().expect("an address").port();
    let mut response: Vec<u8> = head.as_bytes().to_vec();
    response.extend(body);
    let handle: JoinHandle<()> = std::thread::spawn(move || {
        let (stream, _) = listener.accept().expect("one connection");
        let mut reader = BufReader::new(stream.try_clone().expect("a second handle"));
        let mut request_line: String = String::new();
        reader.read_line(&mut request_line).expect("a request line");
        let mut length: usize = 0;
        loop {
            let mut line: String = String::new();
            reader.read_line(&mut line).expect("a header line");
            if line == "\r\n" || line.is_empty() {
                break;
            }
            let (name, value) = line.split_once(':').expect("a header");
            if name.trim().eq_ignore_ascii_case("content-length") {
                length = value.trim().parse().expect("a length");
            }
        }
        let mut body_sent: Vec<u8> = vec![0; length];
        reader.read_exact(&mut body_sent).expect("the body");
        let mut writer = stream;
        let _ = writer.write_all(&response);
    });
    (format!("http://127.0.0.1:{}/v1", port), handle)
}

/// A server that answers `200` with a `Transfer-Encoding: chunked` body that
/// never ends: 64 KiB chunks, written one after another until a write fails.
/// A write fails once the client stopped reading (it hit its size limit and
/// closed the connection), which is what the test is about. The thread is
/// detached and never joined: the test finishing at all is the proof, and
/// joining would wait for a writer that may still be writing.
fn endless_chunked_stub() -> String {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("a free port");
    let port: u16 = listener.local_addr().expect("an address").port();
    std::thread::spawn(move || {
        let (stream, _) = listener.accept().expect("one connection");
        let mut reader = BufReader::new(stream.try_clone().expect("a second handle"));
        let mut request_line: String = String::new();
        reader.read_line(&mut request_line).expect("a request line");
        let mut length: usize = 0;
        loop {
            let mut line: String = String::new();
            reader.read_line(&mut line).expect("a header line");
            if line == "\r\n" || line.is_empty() {
                break;
            }
            let (name, value) = line.split_once(':').expect("a header");
            if name.trim().eq_ignore_ascii_case("content-length") {
                length = value.trim().parse().expect("a length");
            }
        }
        let mut body_sent: Vec<u8> = vec![0; length];
        reader.read_exact(&mut body_sent).expect("the body");
        let mut writer = stream;
        let head: &str =
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n";
        let _ = writer.write_all(head.as_bytes());
        // One chunk of 0x10000 (65536) bytes, framed as chunked transfer: the
        // size line, the data, then CRLF, round after round. The stream never
        // gets the final `0` chunk, so the body never ends.
        let size: &str = "10000\r\n";
        let data: String = "x".repeat(65536);
        loop {
            if writer.write_all(size.as_bytes()).is_err() {
                break;
            }
            if writer.write_all(data.as_bytes()).is_err() {
                break;
            }
            if writer.write_all("\r\n".as_bytes()).is_err() {
                break;
            }
        }
    });
    format!("http://127.0.0.1:{}", port)
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
    assert_eq!(
        reply,
        Ok(Reply {
            text: "{}".to_string()
        })
    );
    assert_eq!(seen.request_line, "POST /v1/chat/completions HTTP/1.1");
    assert_eq!(
        seen.header("content-type"),
        Some("application/json".to_string())
    );
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
    assert!(seen
        .body
        .starts_with("{\"model\":\"qwen-test\",\"temperature\":0,\"seed\":1,"));
}

#[test]
fn a_key_is_sent_as_a_bearer_token_and_never_described() {
    let (url, server) = stub(http("200 OK", &completion("ok")));
    let mut keyed: OpenAiCompatible = backend(&url, Some("sk-secret"));
    assert_eq!(
        keyed.complete(&asked()).map(|reply| reply.text),
        Ok("ok".to_string())
    );
    let seen: Captured = server.join().expect("the stub ran");
    assert_eq!(
        seen.header("authorization"),
        Some("Bearer sk-secret".to_string())
    );
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
    assert_eq!(
        server.join().expect("the stub ran").header("authorization"),
        None
    );
}

#[test]
fn the_content_is_returned_exactly_as_written() {
    let content: &str = "  {\"operation\": \"other\"}\n";
    let (url, server) = stub(http("200 OK", &completion(content)));
    let reply = backend(&url, None).complete(&asked());
    server.join().expect("the stub ran");
    assert_eq!(
        reply,
        Ok(Reply {
            text: content.to_string()
        })
    );
}

#[test]
fn a_status_other_than_success_is_reported_with_its_body() {
    let (url, server) = stub(http(
        "503 Service Unavailable",
        "{\"error\":\n\"loading model\"}",
    ));
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
        (
            "{\"choices\": \"none\"}",
            "the response is not a chat completion: ",
        ),
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
                assert!(
                    error.message.starts_with(case.1),
                    "{}: {:?}",
                    case.0,
                    error.message
                );
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
                error
                    .message
                    .starts_with(&format!("could not reach {}: ", url)),
                "{:?}",
                error.message
            );
            assert!(!error.message.contains("sk-secret"));
        }
        Ok(found) => panic!("a closed port answered {:?}", found),
    }
}

// A server that answered, but with a body that is not text: the answer is
// bad, it is not a network failure (review A-4).
#[test]
fn a_body_that_is_not_utf8_is_a_bad_response() {
    let (url, server) = stub_raw(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 4\r\nConnection: close\r\n\r\n",
        &[0xff, 0xfe, 0x41, 0x42],
    );
    let reply = backend(&url, None).complete(&asked());
    server.join().expect("the stub ran");
    assert_eq!(
        reply,
        Err(BackendError {
            kind: BackendErrorKind::Response,
            message: "the response is not UTF-8".to_string(),
        }),
    );
}

// The size limit cuts the body off while it is read, not after: an endless
// body is refused, and the call returns (review A-8). The stub's thread is
// not joined: it may still be writing when the assertion is made.
#[test]
fn an_endless_body_is_cut_off_while_it_is_read() {
    let url: String = endless_chunked_stub();
    let reply = backend(&url, None).complete(&asked());
    match reply {
        Err(error) => {
            assert_eq!(
                error.kind,
                BackendErrorKind::Response,
                "endless body: {:}",
                error.message
            );
            assert!(error.message.contains("larger than"), "{:}", error.message);
        }
        Ok(found) => panic!("an endless body gave {:?}", found),
    }
}
