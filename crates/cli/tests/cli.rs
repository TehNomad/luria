//! The `luria` executable, run as a process: exit codes, which stream each
//! document goes to, and the exact text of `luria version`.

use luria_core::mutation::{MUTATION_ALGORITHM_NAME, MUTATION_ALGORITHM_VERSION};
use luria_core::primer::{PRIMER_DESIGN_NAME, PRIMER_DESIGN_VERSION};
use luria_core::primer_design::{NATIVE_DESIGN_ALGORITHM_NAME, NATIVE_DESIGN_ALGORITHM_VERSION};
use luria_core::q5::{Q5_WORKFLOW_NAME, Q5_WORKFLOW_VERSION};
use luria_core::tm::{TM_FUNCTION_NAME, TM_FUNCTION_VERSION};
use std::process::{Command, Output, Stdio};

fn luria(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_luria"))
        .args(arguments)
        .output()
        .expect("the luria executable runs")
}

fn algorithm_block(role: &str, name: &str, version: &str, last: bool) -> String {
    format!(
        "    {{\n      \"role\": \"{}\",\n      \"name\": \"{}\",\n      \"version\": \"{}\"\n    }}{}\n",
        role,
        name,
        version,
        if last { "" } else { "," }
    )
}

// The whole document is pinned, so the key order and the layout are too.
#[test]
fn version_prints_the_pinned_document() {
    let output = luria(&["version"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let blocks: String = [
        algorithm_block(
            "mutation",
            MUTATION_ALGORITHM_NAME,
            MUTATION_ALGORITHM_VERSION,
            false,
        ),
        algorithm_block("primer", PRIMER_DESIGN_NAME, PRIMER_DESIGN_VERSION, false),
        algorithm_block("tm", TM_FUNCTION_NAME, TM_FUNCTION_VERSION, false),
        algorithm_block(
            "native_design",
            NATIVE_DESIGN_ALGORITHM_NAME,
            NATIVE_DESIGN_ALGORITHM_VERSION,
            false,
        ),
        algorithm_block("q5_workflow", Q5_WORKFLOW_NAME, Q5_WORKFLOW_VERSION, true),
    ]
    .concat();
    let expected: String = format!(
        "{{\n  \"schema\": \"luria.version/1\",\n  \"package\": \"luria-cli\",\n  \"version\": \"{}\",\n  \"algorithms\": [\n{}  ],\n  \"schemas\": {{\n    \"request\": \"luria.q5-request/1\",\n    \"result\": \"luria.q5-result/1\"\n  }}\n}}\n",
        env!("CARGO_PKG_VERSION"),
        blocks
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
}

#[test]
fn version_is_byte_identical_run_to_run() {
    assert_eq!(luria(&["version"]).stdout, luria(&["--version"]).stdout);
}

#[test]
fn help_prints_the_usage_to_stdout() {
    for spelling in ["help", "--help", "-h"] {
        let output = luria(&[spelling]);
        assert_eq!(output.status.code(), Some(0), "{}", spelling);
        assert!(output.stderr.is_empty(), "{}", spelling);
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            luria_cli::args::USAGE
        );
    }
}

#[test]
fn a_usage_error_exits_2_with_text_on_stderr_and_nothing_on_stdout() {
    let cases: [(&[&str], &str); 4] = [
        (&[], "no command given"),
        (&["frobnicate"], "unknown command 'frobnicate'"),
        (&["q5", "design"], "'q5 design' needs a request file"),
        (&["q5", "design", "a.json", "-v"], "unknown option '-v'"),
    ];
    for (arguments, message) in cases {
        let output = luria(arguments);
        assert_eq!(output.status.code(), Some(2), "{:?}", arguments);
        assert!(output.stdout.is_empty(), "{:?}", arguments);
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("luria: {}\n\n{}", message, luria_cli::args::USAGE),
            "{:?}",
            arguments
        );
    }
}

// Milestone 7a: `review` opens a full screen, so both streams must be
// terminals, and `missing.json` is not a file: the usage error proves the check
// runs before the request is read (plan §3 rule 7, §3.7).
#[test]
fn review_without_a_terminal_is_a_usage_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_luria"))
        .args(["review", "missing.json"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("the luria executable runs");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        format!(
            "luria: 'review' needs a terminal on stdin and stdout; 'q5 design' writes the same result as JSON\n\n{}",
            luria_cli::args::USAGE
        )
    );
}

// Milestone 7f2: the second entry point checks its arguments, its endpoint, its
// key and its terminals before the request location, and the terminal rule is
// the first one that can fail here: stdin is not a terminal, so the exit is 2
// with 7a's usage line, nothing is read, nothing is sent, and `out.json` is
// never written (7f2 §2, plan §3 rule 4).
#[test]
fn review_interpret_checks_the_request_location_first() {
    let output = Command::new(env!("CARGO_BIN_EXE_luria"))
        .args([
            "review",
            "--template",
            "t.gb",
            "--text",
            "x",
            "--backend",
            "openai-compatible",
            "--endpoint",
            "http://127.0.0.1:1/v1",
            "--model",
            "m",
            "--request",
            "out.json",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("the luria executable runs");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        format!(
            "luria: 'review' needs a terminal on stdin and stdout; 'q5 design' writes the same result as JSON\n\n{}",
            luria_cli::args::USAGE
        )
    );
}

// Added in review: a command-line word that isn't valid UTF-8 cannot be
// parsed, so it is a usage error: exit 2 (plan §3.7), not a panic (101).
#[cfg(unix)]
#[test]
fn a_word_that_is_not_utf8_exits_2_with_text_on_stderr_and_nothing_on_stdout() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use std::path::PathBuf;

    let bad_word: OsString = OsStringExt::from_vec(b"r\xff.json".to_vec());
    let arguments: Vec<OsString> = [
        PathBuf::from("q5").into_os_string(),
        PathBuf::from("design").into_os_string(),
        bad_word,
    ]
    .to_vec();
    let output = Command::new(env!("CARGO_BIN_EXE_luria"))
        .args(&arguments)
        .output()
        .expect("the luria executable runs");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let text: String = String::from_utf8(output.stderr).unwrap();
    assert!(
        text.contains("argument 3 is not valid UTF-8"),
        "stderr: {}",
        text
    );
}

// Added in review: a document that cannot be written is an I/O failure,
// exit 1 (plan §3.7), not a panic (101). /dev/full refuses every write.
#[cfg(target_os = "linux")]
#[test]
fn a_failed_write_to_stdout_exits_1() {
    for arguments in [&["version"][..], &["help"][..]] {
        let full = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .expect("/dev/full opens");
        let output = Command::new(env!("CARGO_BIN_EXE_luria"))
            .args(arguments)
            .stdout(full)
            .output()
            .expect("the luria executable runs");
        assert_eq!(output.status.code(), Some(1), "{:?}", arguments);
    }
}

// Milestone 7k5: `new` takes no file on the command line, so any word other than
// `--design` and `--export` is a usage error naming it (contract §2.1), and the
// screen needs a terminal on both streams, as 7a's rule states, checked before
// the form opens, so nothing is read and nothing is written (§3 rule 7, exit 2).
#[test]
fn new_with_an_unknown_word_is_a_usage_error_naming_it() {
    let output = luria(&["new", "--bogus"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        format!(
            "luria: 'new' takes only --design and --export, got '--bogus'\n\n{}",
            luria_cli::args::USAGE
        )
    );
}

// Milestone 7k7: `new`'s backend is all three of `--backend`, `--endpoint` and
// `--model` or none of them (contract §2.1), read by the interpret form's rules,
// so a partial one names the first option missing, and `--allow-remote`, the
// flag of `--backend`, without it names `--allow-remote`.
#[test]
fn new_with_a_backend_but_no_model_is_a_usage_error() {
    let output = luria(&[
        "new",
        "--backend",
        "openai-compatible",
        "--endpoint",
        "http://127.0.0.1:8080/v1",
    ]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        format!("luria: 'new' needs --model\n\n{}", luria_cli::args::USAGE)
    );
}

#[test]
fn new_allow_remote_without_backend_is_a_usage_error() {
    let output = luria(&["new", "--allow-remote"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        format!(
            "luria: --allow-remote needs --backend\n\n{}",
            luria_cli::args::USAGE
        )
    );
}

#[test]
fn new_without_a_terminal_names_new() {
    let output = Command::new(env!("CARGO_BIN_EXE_luria"))
        .args(["new"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("the luria executable runs");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        format!(
            "luria: 'new' needs a terminal on stdin and stdout; 'q5 design' writes the same result as JSON\n\n{}",
            luria_cli::args::USAGE
        ),
        "7a's usage error, with the command `new` names (contract §2.6)"
    );
}
