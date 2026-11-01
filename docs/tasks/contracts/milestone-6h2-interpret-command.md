---
type: Task Contract
title: "Milestone 6h2: the luria q5 interpret command"
description: "6h1 made interpret a library call with the backend as a parameter; 6g1 made the real backend."
status: deprecated
tags: [milestone-6]
---
# Milestone 6h2: the `luria q5 interpret` command

Status: **Implemented 2026-10-19** by the local worker; accepted on the first
review. Notes: the "gate before `interpret`" proof fails through the
stdout-is-empty assertion, not the exit status; a duplicated option is
reported before a missing value, as `--write-edited` already is; an empty
`--backend` or `--endpoint` is refused by the later rules.
Planner: Claude. Parent: [docs/tasks/contracts/milestone-6-nl.md](/tasks/contracts/milestone-6-nl.md) §3 ("Cloud use is
explicit"), §5.6, §5.7, M6-17, M6-18, §6 (6h).

## 1. What and why

6h1 made `interpret` a library call with the backend as a parameter; 6g1
made the real backend. This contract joins them on the command line:

```
luria q5 interpret --template FILE.gb (--text TEXT | --text-file FILE)
    --backend openai-compatible --endpoint URL --model NAME
    [--design PROFILE.json] [--allow-remote] [--write-request FILE.json]
```

and adds the gate of plan §3: a request leaves this machine only when the
user says so with `--allow-remote`. The gate, like every argument check,
runs before any file is read or anything is sent, and a refusal is a usage
error (exit 2, text on stderr, nothing on stdout).

## 2. Requirements this contract applies

- **Plan §5.7:** the command and its options, as above. `--backend`,
  `--endpoint` and `--model` have no defaults. One JSON document on stdout,
  exit status from `interpret`.
- **Plan §3, D023:** an endpoint whose host is not loopback
  (`Endpoint::is_loopback`, 6g1) needs `--allow-remote`. Plain HTTP to
  another machine is unencrypted, and the refusal says so (M6-18).
- **M6-18:** the only backend is `openai-compatible`; an `https://`
  endpoint is refused by `Endpoint::parse`, before anything is sent.
- **Plan §5.6:** the optional bearer key comes from the environment
  variable `LURIA_OPENAI_API_KEY`, read in `main.rs` only. It never
  appears in a document, a message or stderr.
- **5d's conventions for usage errors**, unchanged: `luria: <message>`, a
  blank line, the usage text, all on stderr; exit 2; the first error found,
  left to right, is the one reported.

## 3. Changes

### 3.1 `crates/cli/src/args.rs`

1. `Command` gains a variant:

   ```rust
   Q5Interpret {
       template: PathBuf,
       text: TextSource,
       design: Option<PathBuf>,
       endpoint: String,
       model: String,
       allow_remote: bool,
       write_request: Option<PathBuf>,
   },
   ```

   `TextSource` is `crate::interpret::run::TextSource`. `--backend` is
   checked and not stored: this build has one.

2. `USAGE` becomes exactly:

   ```
   usage:
     luria q5 design REQUEST.json [--write-edited DIR]
     luria q5 interpret --template FILE.gb (--text TEXT | --text-file FILE)
         --backend openai-compatible --endpoint URL --model NAME
         [--design PROFILE.json] [--allow-remote] [--write-request FILE.json]
     luria version
     luria help

   'q5 design' reads a Q5 site-directed mutagenesis request (JSON), runs the
   workflow and writes one JSON result document to stdout. 'q5 interpret'
   asks a language model what a request in words means and writes the
   proposed request, or its questions, as one JSON document. See docs/specs/cli.md.
   ```

   with a trailing newline, as now.

3. `parse_q5`: no subcommand is `'q5' needs a subcommand: design or
   interpret`; `interpret` goes to a new `parse_interpret`; any other word
   `W` is `unknown q5 subcommand 'W'; they are 'design' and 'interpret'`.

4. `parse_interpret(words)`, left to right, first error wins:
   - The value options are `--template`, `--text`, `--text-file`,
     `--design`, `--backend`, `--endpoint`, `--model` and
     `--write-request`. Each takes the next word as its value, whatever it
     is. No next word: `--X needs a value`. Given twice: `--X is given more
     than once`.
   - `--allow-remote` takes no value. Given twice: `--allow-remote is given
     more than once`.
   - Any other word starting with `-`: `unknown option 'W'`. Any other word:
     `'q5 interpret' takes options only, got 'W'`.
   - An empty value for `--template`, `--text-file`, `--design` or
     `--write-request`: `an empty path is not a file`. An empty `--model`:
     `the model name is empty`. An empty `--text` is accepted here;
     `interpret` refuses it as `text_empty`.
   - After all words, in this order: no `--template`: `'q5 interpret' needs
     --template`; neither `--text` nor `--text-file`: `'q5 interpret' needs
     --text or --text-file`; both: `give --text or --text-file, not both`;
     no `--backend`: `'q5 interpret' needs --backend`; a backend other than
     `openai-compatible`: `unknown backend 'B'; this build has only
     'openai-compatible' (M6-18)`; no `--endpoint`: `'q5 interpret' needs
     --endpoint`; no `--model`: `'q5 interpret' needs --model`.

5. A new public function, used by `main.rs`:

   ```rust
   /// The endpoint a `q5 interpret` command line may call, or why not
   /// (plan §3, M6-18). Decided from the text alone, before anything is
   /// read or sent.
   pub fn checked_endpoint(url: &str, allow_remote: bool) -> Result<Endpoint, UsageError>;
   ```

   `Endpoint::parse` failing: `the endpoint is not usable: <message>`. A
   parsed endpoint that is not loopback, without `allow_remote`: `the
   endpoint <url> is not on this machine; sending the request there needs
   --allow-remote, and plain HTTP to another machine is unencrypted`,
   `<url>` being `endpoint.url()`. Otherwise the endpoint.

### 3.2 `crates/cli/src/main.rs`

`Command::Q5Interpret { .. }` goes to a new `emit_interpret`, which, in
order:
1. `checked_endpoint(&endpoint, allow_remote)`; an error is
   `report_usage(problem)`, exit 2;
2. reads `LURIA_OPENAI_API_KEY` with `std::env::var(..).ok()`: absent or not
   UTF-8 is no key;
3. builds `OpenAiCompatible::new(endpoint, &model, key)` and calls
   `interpret(&InterpretInput { template, text, profile: design,
   write_request }, &mut backend)`;
4. writes `render_interpretation(&outcome.document)` with `write_out`; a
   failed write is exit 1;
5. writes `luria: <note>\n` on stderr when `outcome.note` is `Some`;
6. exits with `outcome.status`.

Nothing else in `main.rs` changes.

### 3.3 Tests to change

In `args.rs`'s `each_usage_error_says_what_is_wrong`, and nowhere else,
change exactly these two expected messages, which §3.1 step 3 changes:

- `(&["q5"], "'q5' needs a subcommand: design")` becomes
  `(&["q5"], "'q5' needs a subcommand: design or interpret")`;
- the `(&["q5", "plan"], ...)` message becomes
  `"unknown q5 subcommand 'plan'; they are 'design' and 'interpret'"`.

## 4. Tests to add (planner-written; add verbatim)

### 4.1 Unit tests, appended inside `args.rs`'s existing `mod tests`

```rust
    /// One usage case: the words, and the message they must give.
    type UsageCase<'a> = (Vec<&'a str>, &'a str);

    fn interpret_words<'a>(extra: &[&'a str]) -> Vec<&'a str> {
        let mut words: Vec<&'a str> = vec![
            "q5",
            "interpret",
            "--template",
            "r.gb",
            "--text",
            "make A42G",
            "--backend",
            "openai-compatible",
            "--endpoint",
            "http://127.0.0.1:8080/v1",
            "--model",
            "qwen",
        ];
        words.extend_from_slice(extra);
        words
    }

    /// `base` without `option` and the value after it.
    fn without<'a>(base: &[&'a str], option: &str) -> Vec<&'a str> {
        let at: usize = base.iter().position(|word| *word == option).expect("an option");
        let mut words: Vec<&'a str> = base.to_vec();
        words.drain(at..at + 2);
        words
    }

    #[test]
    fn q5_interpret_reads_every_option() {
        assert_eq!(
            parse_words(&interpret_words(&[
                "--design",
                "p.json",
                "--allow-remote",
                "--write-request",
                "out.json"
            ])),
            Ok(Command::Q5Interpret {
                template: PathBuf::from("r.gb"),
                text: TextSource::Inline("make A42G".to_string()),
                design: Some(PathBuf::from("p.json")),
                endpoint: "http://127.0.0.1:8080/v1".to_string(),
                model: "qwen".to_string(),
                allow_remote: true,
                write_request: Some(PathBuf::from("out.json")),
            })
        );
        let file: Vec<&str> = vec![
            "q5", "interpret", "--model", "m", "--text-file", "t.txt", "--endpoint", "e",
            "--backend", "openai-compatible", "--template", "r.gb",
        ];
        assert_eq!(
            parse_words(&file),
            Ok(Command::Q5Interpret {
                template: PathBuf::from("r.gb"),
                text: TextSource::File(PathBuf::from("t.txt")),
                design: None,
                endpoint: "e".to_string(),
                model: "m".to_string(),
                allow_remote: false,
                write_request: None,
            })
        );
        // A value is the next word, whatever it is.
        assert!(matches!(
            parse_words(&interpret_words(&["--design", "--allow-remote"])),
            Ok(Command::Q5Interpret { design: Some(_), allow_remote: false, .. })
        ));
    }

    #[test]
    fn each_q5_interpret_usage_error_says_what_is_wrong() {
        let base: Vec<&str> = interpret_words(&[]);
        let cases: Vec<UsageCase> = vec![
            (without(&base, "--template"), "'q5 interpret' needs --template"),
            (without(&base, "--text"), "'q5 interpret' needs --text or --text-file"),
            (without(&base, "--backend"), "'q5 interpret' needs --backend"),
            (without(&base, "--endpoint"), "'q5 interpret' needs --endpoint"),
            (without(&base, "--model"), "'q5 interpret' needs --model"),
            (interpret_words(&["--text-file", "t.txt"]), "give --text or --text-file, not both"),
            (interpret_words(&["--model", "n"]), "--model is given more than once"),
            (interpret_words(&["--allow-remote", "--allow-remote"]), "--allow-remote is given more than once"),
            (interpret_words(&["--design"]), "--design needs a value"),
            (interpret_words(&["--design", ""]), "an empty path is not a file"),
            (interpret_words(&["--write-request", ""]), "an empty path is not a file"),
            (interpret_words(&["--verbose"]), "unknown option '--verbose'"),
            (interpret_words(&["stray"]), "'q5 interpret' takes options only, got 'stray'"),
            (
                vec!["q5", "interpret", "--template", "", "--text", "t"],
                "an empty path is not a file",
            ),
            (
                vec!["q5", "interpret", "--model", ""],
                "the model name is empty",
            ),
        ];
        for case in cases {
            assert_eq!(usage(&case.0), case.1, "{:?}", case.0);
        }
        let mut other: Vec<&str> = base.clone();
        let at: usize = other.iter().position(|word| *word == "openai-compatible").expect("the backend");
        other[at] = "anthropic";
        assert_eq!(
            usage(&other),
            "unknown backend 'anthropic'; this build has only 'openai-compatible' (M6-18)"
        );
        assert_eq!(usage(&["q5", "interpret", "--text"]), "--text needs a value");
    }

    #[test]
    fn an_endpoint_leaves_this_machine_only_when_allowed() {
        assert!(checked_endpoint("http://127.0.0.1:8080/v1", false).is_ok());
        assert!(checked_endpoint("http://localhost:11434/v1", false).is_ok());
        assert_eq!(
            checked_endpoint("http://gpu.lan:8080/v1/", false).map_err(|error| error.message),
            Err("the endpoint http://gpu.lan:8080/v1 is not on this machine; sending the request there needs --allow-remote, and plain HTTP to another machine is unencrypted".to_string())
        );
        assert!(checked_endpoint("http://gpu.lan:8080/v1", true).is_ok());
        assert_eq!(
            checked_endpoint("https://api.example.com/v1", true).map_err(|error| error.message),
            Err("the endpoint is not usable: https endpoints are not supported in this build; use http:// (M6-18)".to_string())
        );
    }
```

### 4.2 Integration test: `crates/cli/tests/interpret_cli.rs`

```rust
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
    let dir: PathBuf = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("interpret-cli-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a workspace");
    fs::copy(
        "../../tests/fixtures/genbank/synthetic-linear.gb",
        dir.join("synthetic-linear.gb"),
    )
    .expect("the record is copied");
    fs::copy(format!("{}profile/full.json", NL), dir.join("full.json")).expect("the profile is copied");
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
    command.args(arguments).current_dir(dir).env_remove("LURIA_OPENAI_API_KEY");
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
            "q5", "interpret", "--template", "synthetic-linear.gb", "--text-file", "ask.txt",
            "--design", "full.json", "--backend", "openai-compatible", "--endpoint", url.as_str(),
            "--model", "qwen-test", "--write-request", "proposal.json",
        ],
        Some("sk-cli"),
    );
    let (headers, body) = server.join().expect("the stub ran");
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stderr.is_empty());
    let stdout: String = String::from_utf8(output.stdout).expect("UTF-8");
    let document: Value = serde_json::from_str(&stdout).expect("one JSON document");
    assert_eq!(document["state"], "proposed");
    assert_eq!(document["interpretation"]["backend"]["kind"], "openai-compatible");
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
    assert_eq!(design.status.code(), Some(0), "{}", String::from_utf8_lossy(&design.stderr));
}

#[test]
fn a_refused_reply_exits_8_and_no_key_means_no_header() {
    let dir: PathBuf = workspace("refused");
    let (url, server) = stub(completion("not an extraction"));
    let output: Output = luria(
        &dir,
        &[
            "q5", "interpret", "--template", "synthetic-linear.gb", "--text",
            "In synA, change alanine 42 to glycine.", "--design", "full.json",
            "--backend", "openai-compatible", "--endpoint", url.as_str(), "--model", "m",
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
                "q5", "interpret", "--template", "absent.gb", "--text", "make A42G",
                "--backend", "openai-compatible", "--endpoint", case.0, "--model", "m",
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
            "q5", "interpret", "--template", "absent.gb", "--text", "make A42G",
            "--backend", "openai-compatible", "--endpoint", url.as_str(), "--model", "m",
        ],
        None,
    );
    assert_eq!(output.status.code(), Some(4));
    let document: Value = serde_json::from_slice(&output.stdout).expect("one JSON document");
    assert_eq!(document["error"]["code"], "template_unreadable");
    assert_eq!(document["interpretation"]["messages"], Value::Null);
}
```

`cargo fmt` may reflow these; do not change content. If a test fails,
report what the code returned. If a planner-written test fails to compile
or fails clippy, stop and report it rather than editing it.

### 4.3 Proving the rules are load-bearing (AGENTS.md)

Disable each once, confirm the named test fails, restore, report both:

| Rule, disabled | Test that must fail |
| --- | --- |
| the loopback gate (ignore `allow_remote`) | `an_endpoint_leaves_this_machine_only_when_allowed` |
| the key read from `LURIA_OPENAI_API_KEY` (always pass `None`) | `words_become_a_request_over_http_and_the_request_runs` |
| `--text` and `--text-file` exclusive | `each_q5_interpret_usage_error_says_what_is_wrong` |
| the gate before `interpret` (call `interpret` first) | `a_remote_endpoint_needs_allow_remote_and_https_is_refused` |

## 5. Do not

- Read the environment anywhere but `main.rs`, or any variable but
  `LURIA_OPENAI_API_KEY`.
- Print the key, or anything derived from it, anywhere.
- Change `interpret`, `luria-nl`, `luria-core`, `tests/fixtures/` or
  [docs/](/), or any test other than the two messages of §3.3.
- Add a backend kind, a default for any option, or a dependency.

## 6. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 7. Handoff

Report the new and changed files, the exact commands and their results,
the §4.3 results, and anything that did not match this contract. Quote
command output or the file for every claim; say plainly what you did not
check.
