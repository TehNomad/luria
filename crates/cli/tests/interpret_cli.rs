//! `luria q5 interpret` as a process, against a stub OpenAI-compatible
//! server on 127.0.0.1 (contract 6h2): words in, a request out over real
//! HTTP, and `luria q5 design` running the request it wrote. No test uses
//! the network or a model.

use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread::JoinHandle;

const NL: &str = "../../tests/fixtures/nl/";

fn workspace(test: &str) -> PathBuf {
    let dir: PathBuf =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("interpret-cli-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a workspace");
    fs::copy(
        "../../tests/fixtures/genbank/synthetic-linear.gb",
        dir.join("synthetic-linear.gb"),
    )
    .expect("the record is copied");
    fs::copy(format!("{}profile/full.json", NL), dir.join("full.json"))
        .expect("the profile is copied");
    dir
}

/// A server on a free local port that answers one request with `response`
/// and returns the request's headers (names lower-cased) and body.
/// What the stub saw: the request's headers, names lower-cased, and its body.
type Seen = (Vec<(String, String)>, String);

fn stub(response: String) -> (String, JoinHandle<Seen>) {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("a free port");
    let port: u16 = listener.local_addr().expect("an address").port();
    let handle = std::thread::spawn(move || {
        let (stream, _) = listener.accept().expect("one connection");
        let mut reader = BufReader::new(stream.try_clone().expect("a second handle"));
        let mut line: String = String::new();
        reader.read_line(&mut line).expect("a request line");
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
            if name == "content-length" {
                length = value.trim().parse().expect("a length");
            }
            headers.push((name, value.trim().to_string()));
        }
        let mut body: Vec<u8> = vec![0; length];
        reader.read_exact(&mut body).expect("the body");
        let mut writer = stream;
        let _ = writer.write_all(response.as_bytes());
        (headers, String::from_utf8(body).expect("a UTF-8 body"))
    });
    (format!("http://127.0.0.1:{}/v1", port), handle)
}

/// A complete HTTP response holding a chat completion whose content is
/// `content`.
fn completion(content: &str) -> String {
    let body: String = serde_json::json!({
        "choices": [{"index": 0, "message": {"role": "assistant", "content": content}}]
    })
    .to_string();
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

fn luria(dir: &Path, arguments: &[&str], key: Option<&str>) -> Output {
    let mut command: Command = Command::new(env!("CARGO_BIN_EXE_luria"));
    command
        .args(arguments)
        .current_dir(dir)
        .env_remove("LURIA_OPENAI_API_KEY");
    if let Some(key) = key {
        command.env("LURIA_OPENAI_API_KEY", key);
    }
    command.output().expect("the luria executable runs")
}

fn corpus(file: &str) -> String {
    fs::read_to_string(format!("{}{}", NL, file)).expect("a corpus file")
}

#[test]
fn words_become_a_request_over_http_and_the_request_runs() {
    let dir: PathBuf = workspace("proposal");
    fs::write(dir.join("ask.txt"), corpus("text/two-separate.txt")).expect("written");
    let (url, server) = stub(completion(&corpus("reply/two-separate.json")));
    let output: Output = luria(
        &dir,
        &[
            "q5",
            "interpret",
            "--template",
            "synthetic-linear.gb",
            "--text-file",
            "ask.txt",
            "--design",
            "full.json",
            "--backend",
            "openai-compatible",
            "--endpoint",
            url.as_str(),
            "--model",
            "qwen-test",
            "--write-request",
            "proposal.json",
        ],
        Some("sk-cli"),
    );
    let (headers, body) = server.join().expect("the stub ran");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let stdout: String = String::from_utf8(output.stdout).expect("UTF-8");
    let document: Value = serde_json::from_str(&stdout).expect("one JSON document");
    assert_eq!(document["state"], "proposed");
    assert_eq!(
        document["interpretation"]["backend"]["kind"],
        "openai-compatible"
    );
    assert_eq!(document["interpretation"]["backend"]["model"], "qwen-test");
    assert!(!stdout.contains("sk-cli"), "the key is in the document");
    assert!(headers.contains(&("authorization".to_string(), "Bearer sk-cli".to_string())));
    let sent: Value = serde_json::from_str(&body).expect("a JSON request");
    assert_eq!(sent["model"], "qwen-test");
    assert!(sent["messages"][1]["content"]
        .as_str()
        .expect("the user message")
        .ends_with(corpus("text/two-separate.txt").trim_end()));
    let design: Output = luria(&dir, &["q5", "design", "proposal.json"], None);
    assert_eq!(
        design.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&design.stderr)
    );
}

#[test]
fn a_refused_reply_exits_8_and_no_key_means_no_header() {
    let dir: PathBuf = workspace("refused");
    let (url, server) = stub(completion("not an extraction"));
    let output: Output = luria(
        &dir,
        &[
            "q5",
            "interpret",
            "--template",
            "synthetic-linear.gb",
            "--text",
            "In synA, change alanine 42 to glycine.",
            "--design",
            "full.json",
            "--backend",
            "openai-compatible",
            "--endpoint",
            url.as_str(),
            "--model",
            "m",
        ],
        None,
    );
    let (headers, _) = server.join().expect("the stub ran");
    assert_eq!(output.status.code(), Some(8));
    let document: Value = serde_json::from_slice(&output.stdout).expect("one JSON document");
    assert_eq!(document["error"]["code"], "model_output_invalid");
    assert!(headers.iter().all(|header| header.0 != "authorization"));
}

// The gate runs before anything is read or sent: the template here does not
// exist, and `.invalid` never resolves, so only the gate can answer.
#[test]
fn a_remote_endpoint_needs_allow_remote_and_https_is_refused() {
    let dir: PathBuf = workspace("gate");
    let cases: [(&str, &str); 2] = [
        (
            "http://gpu.invalid:8080/v1",
            "the endpoint http://gpu.invalid:8080/v1 is not on this machine; sending the request there needs --allow-remote, and plain HTTP to another machine is unencrypted",
        ),
        (
            "https://api.example.invalid/v1",
            "the endpoint is not usable: https endpoints are not supported in this build; use http:// (M6-18)",
        ),
    ];
    for case in cases {
        let output: Output = luria(
            &dir,
            &[
                "q5",
                "interpret",
                "--template",
                "absent.gb",
                "--text",
                "make A42G",
                "--backend",
                "openai-compatible",
                "--endpoint",
                case.0,
                "--model",
                "m",
            ],
            None,
        );
        assert_eq!(output.status.code(), Some(2), "{}", case.0);
        assert!(output.stdout.is_empty(), "{}", case.0);
        let stderr: String = String::from_utf8(output.stderr).expect("UTF-8");
        assert!(
            stderr.starts_with(&format!("luria: {}\n\n", case.1)),
            "{}: {:?}",
            case.0,
            stderr
        );
    }
}

#[test]
fn a_local_refusal_is_a_document_and_nothing_is_sent() {
    let dir: PathBuf = workspace("local");
    // Nothing listens here: if anything were sent, the run would end
    // `backend_unreachable` (exit 1), not `template_unreadable` (exit 4).
    let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("a free port");
    let port: u16 = listener.local_addr().expect("an address").port();
    drop(listener);
    let url: String = format!("http://127.0.0.1:{}/v1", port);
    let output: Output = luria(
        &dir,
        &[
            "q5",
            "interpret",
            "--template",
            "absent.gb",
            "--text",
            "make A42G",
            "--backend",
            "openai-compatible",
            "--endpoint",
            url.as_str(),
            "--model",
            "m",
        ],
        None,
    );
    assert_eq!(output.status.code(), Some(4));
    let document: Value = serde_json::from_slice(&output.stdout).expect("one JSON document");
    assert_eq!(document["error"]["code"], "template_unreadable");
    assert_eq!(document["interpretation"]["messages"], Value::Null);
}

// D045, M6-18: a key and an endpoint that is not this machine is refused
// before anything is read or sent. `0.0.0.0` is not loopback to the gate, and
// on this machine a connection to it reaches the listener bound to 127.0.0.1,
// so an empty channel is the proof that nothing was sent. The thread is never
// joined: when nothing is sent it stays blocked in `accept`.
#[test]
fn a_key_is_not_sent_to_another_machine() {
    let dir: PathBuf = workspace("remote-key");
    fs::write(dir.join("ask.txt"), corpus("text/two-separate.txt")).expect("written");
    let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("a free port");
    let port: u16 = listener.local_addr().expect("an address").port();
    let endpoint: String = format!("http://0.0.0.0:{:}/v1", port);
    let (sent, seen) = std::sync::mpsc::channel::<()>();
    let response: String = completion(&corpus("reply/two-separate.json"));
    let _server: JoinHandle<()> = std::thread::spawn(move || {
        let (stream, _) = listener.accept().expect("one connection");
        // The connection arrived: only reached when something was sent.
        let _ = sent.send(());
        let mut reader = BufReader::new(stream.try_clone().expect("a second handle"));
        let mut line: String = String::new();
        reader.read_line(&mut line).expect("a request line");
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
        let mut body: Vec<u8> = vec![0; length];
        reader.read_exact(&mut body).expect("the body");
        let mut writer = stream;
        let _ = writer.write_all(response.as_bytes());
    });
    let output: Output = luria(
        &dir,
        &[
            "q5",
            "interpret",
            "--template",
            "synthetic-linear.gb",
            "--text-file",
            "ask.txt",
            "--design",
            "full.json",
            "--backend",
            "openai-compatible",
            "--endpoint",
            endpoint.as_str(),
            "--model",
            "qwen-test",
            "--allow-remote",
        ],
        Some("sk-remote"),
    );
    assert_eq!(output.status.code(), Some(2), "{:?}", output.stderr);
    assert!(output.stdout.is_empty());
    let stderr: String = String::from_utf8(output.stderr).expect("UTF-8");
    assert!(
        stderr.contains("luria does not send a key over plain HTTP"),
        "{:?}",
        stderr
    );
    assert_eq!(
        seen.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Empty),
        "a request was sent to another machine"
    );
}

// The same endpoint with no key still runs: the refusal is about the key, not
// about the endpoint.
#[test]
fn a_remote_endpoint_without_a_key_is_still_called() {
    let dir: PathBuf = workspace("remote-call");
    fs::write(dir.join("ask.txt"), corpus("text/two-separate.txt")).expect("written");
    let (stub_url, server) = stub(completion(&corpus("reply/two-separate.json")));
    let endpoint: String = stub_url.replace("127.0.0.1", "0.0.0.0");
    let output: Output = luria(
        &dir,
        &[
            "q5",
            "interpret",
            "--template",
            "synthetic-linear.gb",
            "--text-file",
            "ask.txt",
            "--design",
            "full.json",
            "--backend",
            "openai-compatible",
            "--endpoint",
            endpoint.as_str(),
            "--model",
            "qwen-test",
            "--allow-remote",
        ],
        None,
    );
    let (headers, _) = server.join().expect("the stub ran");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let document: Value = serde_json::from_slice(&output.stdout).expect("one JSON document");
    assert_eq!(document["state"], "proposed");
    assert!(headers.iter().all(|header| header.0 != "authorization"));
}
