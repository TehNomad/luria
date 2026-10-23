//! The `--write-request` rules as three functions (contract 7f1): the location
//! check, the text, and the atomic write. `luria q5 interpret` calls all three,
//! so the TUI (7f2) and the command share one implementation (plan §3 rule 4,
//! §10.5, §10.7). The directories are under `CARGO_TARGET_TMPDIR` as
//! `tests/interpret_run.rs`'s `workspace` does.

use luria_cli::interpret::run::request_location_problem;
use luria_cli::interpret::run::request_text;
use luria_cli::interpret::run::write_request_file;
use luria_cli::request::parse_request;
use luria_cli::request::RequestDocument;
use luria_cli::result::request_echo;
use luria_cli::result::RequestEcho;
use std::fs;
use std::path::PathBuf;

const GENBANK: &str = "../../tests/fixtures/genbank/";
const NL: &str = "../../tests/fixtures/nl/";

/// A fresh directory holding `synthetic-linear.gb` as the template.
fn workspace(test: &str) -> PathBuf {
    let dir: PathBuf =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("request-file-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a workspace");
    fs::copy(
        format!("{}synthetic-linear.gb", GENBANK),
        dir.join("synthetic-linear.gb"),
    )
    .expect("the record is copied");
    dir
}

/// A path that names a free file beside the template is usable, and nothing is
/// written by the check itself.
#[test]
fn a_free_name_beside_the_template_is_usable() {
    let dir: PathBuf = workspace("free");
    assert_eq!(
        request_location_problem(&dir.join("synthetic-linear.gb"), &dir.join("proposal.json")),
        None
    );
    assert!(!dir.join("proposal.json").exists());
}

/// A path outside the template's directory is refused, with the message about
/// that directory, and nothing is written.
#[test]
fn another_directory_is_unusable() {
    let dir: PathBuf = workspace("elsewhere");
    let other: PathBuf =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("request-file-elsewhere-outer");
    let _ = fs::remove_dir_all(&other);
    fs::create_dir_all(&other).expect("the other directory");
    let found: (String, String) = request_location_problem(
        &dir.join("synthetic-linear.gb"),
        &other.join("proposal.json"),
    )
    .expect("the request is not beside the template");
    assert_eq!(found.0, "request_location_unusable");
    assert_eq!(
        found.1,
        "the request must be written in the template's directory, because a request names its template relative to itself"
    );
    assert!(!other.join("proposal.json").exists());
    let _ = fs::remove_dir_all(&other);
}

/// A name something holds is `output_exists`, and the write that follows refuses
/// it as well, leaving the old content in place.
#[test]
fn an_existing_file_is_never_replaced() {
    let dir: PathBuf = workspace("taken");
    let path: PathBuf = dir.join("proposal.json");
    fs::write(&path, "old").expect("a file is there");
    let found: (String, String) = request_location_problem(&dir.join("synthetic-linear.gb"), &path)
        .expect("the name is taken");
    assert_eq!(found.0, "output_exists");
    assert_eq!(
        found.1,
        "the file proposal.json already exists; luria never replaces a file"
    );
    assert!(write_request_file(&path, "new").is_err());
    assert_eq!(
        fs::read_to_string(&path).expect("the file is readable"),
        "old".to_string()
    );
}

/// The text `--write-request` writes is the pretty echo and exactly one
/// trailing `\n`, and it reads back as the request it came from.
#[test]
fn the_text_is_the_pretty_echo_and_a_newline() {
    let contents: String = fs::read_to_string(format!("{}request/edit-a42g.json", NL))
        .expect("the corpus request is readable");
    let request: RequestDocument = parse_request(&contents).expect("the fixture is a request");
    let echo: RequestEcho = request_echo(&request);
    let text: String = request_text(&echo);
    assert_eq!(
        text,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&echo).expect("the echo serializes")
        )
    );
    assert!(text.ends_with('\n'));
    assert!(!text.ends_with("\n\n"));
    assert_eq!(parse_request(&text).ok(), Some(request));
}
