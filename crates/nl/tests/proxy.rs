//! The backend and the one address it may answer at: a proxy named in the
//! environment must not carry the prompt (review A-2). This is its own test
//! binary because it sets `HTTP_PROXY` and friends in the environment, and
//! those are the whole process's: another test running in parallel would see
//! them half-set and judge a call it was not meant to judge.

use luria_nl::backend::Backend;
use luria_nl::endpoint::Endpoint;
use luria_nl::openai::OpenAiCompatible;
use luria_nl::prompt::Prompt;
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
fn a_proxy_in_the_environment_is_not_used() {
    // The address the backend must answer at, and the address a proxy would
    // answer at: a listener of its own for each, so a reply is proof of the
    // door the prompt actually left by.
    let (sender, receiver) = std::sync::mpsc::channel::<()>();
    let proxy: TcpListener = TcpListener::bind("127.0.0.1:0").expect("a free port");
    let proxy_port: u16 = proxy.local_addr().expect("an address").port();
    // The proxy's one thread: it hands over a token only if a connection
    // reached it, then drops the connection at once, so a client that did use
    // it fails fast instead of waiting for the whole call timeout. The thread
    // is never joined: when nothing reaches the proxy it stays in `accept`,
    // and the process goes anyway.
    let _ = std::thread::spawn(move || {
        if let Ok((stream, _)) = proxy.accept() {
            let _ = sender.send(());
            drop(stream);
        }
    });
    // A proxy is looked up when the agent is built, not when the request is
    // sent, so the environment is in place before the backend exists: this is
    // how a real process looks, with `HTTP_PROXY` set at start-up.
    let proxy_url: String = format!("http://127.0.0.1:{}", proxy_port);
    for name in ["HTTP_PROXY", "http_proxy", "ALL_PROXY", "all_proxy"] {
        std::env::set_var(name, &proxy_url);
    }
    // A `NO_PROXY` in the environment would switch an environment proxy off
    // before the backend even looked, which would make the test tell nothing;
    // it is taken down so the four names above are what the backend sees.
    for name in ["NO_PROXY", "no_proxy"] {
        std::env::remove_var(name);
    }
    // The endpoint's own server, answering once with a completion.
    let (url, server) = stub(http("200 OK", &completion("{}")));
    let reply = backend(&url, None).complete(&asked());
    // The completion is the endpoint's: the body the proxy would have
    // answered with is not a chat completion, so a call that went through the
    // proxy could not have said this. The assertions are made before the
    // endpoint's server is joined: when the test catches the defect nothing
    // reaches that server, and joining it first would wait for a connection
    // that never comes.
    assert_eq!(reply.map(|reply| reply.text), Ok("{}".to_string()));
    // Nothing reached the proxy: its handler left no token behind.
    assert_eq!(
        receiver.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Empty)
    );
    // What the endpoint's own server received is the prompt the plan sends:
    // the call left by the endpoint's door, and the bytes of the prompt are
    // the only thing that ever leaves the process.
    let seen: Captured = server.join().expect("the stub ran");
    assert_eq!(seen.request_line, "POST /v1/chat/completions HTTP/1.1");
    assert_eq!(
        seen.header("content-type"),
        Some("application/json".to_string())
    );
    assert!(seen
        .body
        .starts_with("{\"model\":\"qwen-test\",\"temperature\":0,\"seed\":1,"));
}
