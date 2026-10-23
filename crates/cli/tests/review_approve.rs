//! Approving a proposal (milestone 7f2, plan §3 rule 4): `luria_cli::review::approve`
//! writes the proposed request at the path `--request` names, then designs from
//! that file, exactly as `luria q5 design` runs a request file. Nothing is
//! written, and nothing runs, when the run asked a question, when the path
//! already holds a file, or when the user quits before approving.
//!
//! The corpus cases run through `interpret` with the scripted backend as
//! `tests/review_proposal.rs` runs them, with `write_request: None`: the
//! approval, not the run, is what writes the file here. Every expected value is
//! one the corpus, the document, or the file system holds on its own: the state
//! and codes are `tests/fixtures/nl/cases.tsv`'s own columns, recorded in
//! `tests/fixtures/nl/README.md`; the text written is `request_text` of the
//! document's request, the same function `--write-request` writes with; and the
//! result is compared with the run of the file on disk, not with the proposal.
//! Nothing is copied from running the code.
//!
//! The three preview tests (contract 7m8a, M7-60) run the proposed request's
//! bytes with nothing written, and compare them with the same proposal saved
//! and run: the expectation for a preview is the document, digest, template
//! letters and frames the saved run gives, taken from `tests/review_approve.rs`'s
//! own rule that a run is compared with the run of the file, not with a value
//! the code printed.

use luria_cli::interpret::document::InterpretationDocument;
use luria_cli::interpret::run::interpret;
use luria_cli::interpret::run::request_text;
use luria_cli::interpret::run::InterpretInput;
use luria_cli::interpret::run::InterpretOutcome;
use luria_cli::interpret::run::TextSource;
use luria_cli::result::render;
use luria_cli::review::app::Key;
use luria_cli::review::approve;
use luria_cli::review::preview;
use luria_cli::review::proposal::{ProposalAction, ProposalApp, ProposalScreen};
use luria_cli::review::Approved;
use luria_cli::run::run;
use luria_nl::backend::ScriptedBackend;
use std::fs;
use std::path::{Path, PathBuf};

const NL: &str = "../../tests/fixtures/nl/";
const GENBANK: &str = "../../tests/fixtures/genbank/";
const CASES_HEADER: &str = "case_id\ttemplate\ttext_file\treply_file\tprofile\tstate\tcodes\trequest_file\tdesign_exit\tnote";

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("cannot read {}: {}", path, error))
}

fn cases() -> Vec<Vec<String>> {
    let text: String = read(&format!("{}cases.tsv", NL));
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], CASES_HEADER);
    lines[1..]
        .iter()
        .map(|line| line.split('\t').map(|field| field.to_string()).collect())
        .collect()
}

/// The corpus's own row for one case.
fn row_of(case: &str) -> Vec<String> {
    cases()
        .into_iter()
        .find(|row| row[0] == case)
        .expect("the corpus has this case")
}

/// A fresh directory for one case: its template and, when it has one, its
/// profile, copied in under their own names, as `tests/review_proposal.rs` does.
fn workspace(test: &str, row: &[String]) -> PathBuf {
    let dir: PathBuf = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("review-approve-{}-{}", test, row[0]));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a workspace");
    fs::copy(format!("{}{}", GENBANK, row[1]), dir.join(&row[1])).expect("the template is copied");
    if row[4] != "-" {
        fs::copy(format!("{}profile/{}", NL, row[4]), dir.join(&row[4]))
            .expect("the profile is copied");
    }
    dir
}

/// The case's text, without its file's final newline.
fn text_of(row: &[String]) -> String {
    let text: String = read(&format!("{}text/{}", NL, row[2]));
    text.strip_suffix('\n').unwrap_or(&text).to_string()
}

/// The case, interpreted in its own directory with `write_request: None`, as
/// `luria review` runs it: the run writes nothing, because approving does.
fn interpret_case(dir: &Path, row: &[String]) -> InterpretOutcome {
    let reply: String = read(&format!("{}reply/{}", NL, row[3]));
    let mut backend: ScriptedBackend = ScriptedBackend::new("scripted-corpus", vec![Ok(reply)]);
    let input: InterpretInput = InterpretInput {
        template: dir.join(&row[1]),
        text: TextSource::Inline(text_of(row)),
        profile: if row[4] == "-" {
            None
        } else {
            Some(dir.join(&row[4]))
        },
        write_request: None,
    };
    interpret(&input, &mut backend)
}

/// The case run in a fresh directory, with the request path `--request` names
/// beside the template: what `emit_review_interpret` hands to `approve`.
fn proposal_of(test: &str, case: &str) -> (PathBuf, InterpretationDocument, PathBuf) {
    let row: Vec<String> = row_of(case);
    let dir: PathBuf = workspace(test, &row);
    let outcome: InterpretOutcome = interpret_case(&dir, &row);
    assert_eq!(
        outcome.document.state, row[5],
        "{}: the corpus's own state column",
        row[0]
    );
    (
        dir.clone(),
        outcome.document.clone(),
        dir.join("proposal.json"),
    )
}

/// The workspace's own entries, sorted, so a run that writes nothing can be
/// compared with the directory it started in (contract 7m8a §3).
fn entries_of(dir: &Path) -> Vec<String> {
    let mut found: Vec<String> = fs::read_dir(dir)
        .expect("the workspace is there to list")
        .map(|entry| {
            entry
                .expect("a readable entry")
                .file_name()
                .to_str()
                .expect("a UTF-8 file name")
                .to_string()
        })
        .collect();
    found.sort();
    found
}

#[test]
fn approving_writes_the_request_and_designs_from_it() {
    let (dir, document, request) = proposal_of("written", "two-separate");
    assert_eq!(document.state, "proposed", "the case is a proposal");
    let echo = document
        .request
        .as_ref()
        .expect("a proposal names its request");

    let approved: Approved = match approve(&document, &dir.join("synthetic-linear.gb"), &request) {
        Ok(approved) => approved,
        Err(problem) => panic!("the approval of a proposed request must run: {}", problem),
    };

    // The file the approval wrote is the request, byte for byte, as
    // `--write-request` writes it (plan §3 rule 2), and the proposal shown on
    // the screen is the same text (`tests/review_proposal.rs`).
    let written: String =
        fs::read_to_string(&request).expect("the written request is there to read");
    assert_eq!(
        written,
        request_text(echo),
        "the file names a request on disk"
    );

    // The design ran from that file: the result the approval gives is the
    // result `luria q5 design` prints for the same path, byte for byte, so the
    // screen shows what the file gave and the request digest names a file
    // (plan §3 rule 4).
    let rerun = run(&request);
    assert_eq!(
        render(&approved.document),
        render(&rerun.document),
        "the result is the run of the written file"
    );
    assert_eq!(approved.document.state, "verified", "the set is designed");
    assert_eq!(
        approved.document.provenance.request_sha256, rerun.document.provenance.request_sha256,
        "and its digest is the written file's"
    );
    assert!(
        approved.template.is_some(),
        "the template beside the request is the one the result names (D005)"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn approving_never_replaces_a_file() {
    let (dir, document, request) = proposal_of("replace", "two-separate");
    let kept: &str = "the file that is already there\n";
    fs::write(&request, kept).expect("a file is placed at the request path");

    let outcome: Result<Approved, String> =
        approve(&document, &dir.join("synthetic-linear.gb"), &request);
    let problem: String = match outcome {
        Err(problem) => problem,
        Ok(_approved) => panic!("a path that already holds a file is never written through"),
    };
    assert!(
        problem.contains("proposal.json"),
        "the refusal names the file the user was told about: {}",
        problem
    );
    assert_eq!(
        fs::read_to_string(&request).expect("the file is readable"),
        kept,
        "the file that was there is unchanged"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_clarification_cannot_be_approved() {
    let (dir, document, request) = proposal_of("clarification", "composition-unstated");
    assert_eq!(document.state, "needs_clarification", "the case asks");
    assert!(
        document.request.is_none(),
        "a run that asks proposes nothing"
    );

    let outcome: Result<Approved, String> =
        approve(&document, &dir.join("synthetic-linear.gb"), &request);
    assert_eq!(
        outcome,
        Err("there is no proposed request to approve".to_string()),
        "there is nothing to write, so nothing is written and nothing runs"
    );
    assert!(
        !request.exists(),
        "no file exists at the path the user named: {:?}",
        request
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn quitting_before_approval_writes_nothing() {
    let (dir, document, request) = proposal_of("quit", "two-separate");
    let mut app: ProposalApp = ProposalApp::new(document, request.clone());
    assert_eq!(
        app.handle(Key::Char('q')),
        ProposalAction::Quit,
        "`q` on the proposal quits"
    );
    assert_eq!(
        app.screen(),
        ProposalScreen::Reading,
        "the proposal screen is open"
    );
    assert!(
        !request.exists(),
        "quitting writes nothing: no file at {:?} (plan §3 rule 4)",
        request
    );
    let _ = fs::remove_dir_all(&dir);
}

/// The preview is the saved run (contract 7m8a §1, M7-60): the bytes the save
/// writes, `request_text` of the proposal, run as though they were already at
/// the path the save will write, give the document the run of the written file
/// gives, byte for byte, the request digest included, and the same template
/// letters and mutant frames.
#[test]
fn a_preview_is_the_saved_run() {
    let (dir, document, request) = proposal_of("preview-run", "two-separate");
    let template: PathBuf = dir.join("synthetic-linear.gb");

    let previewed: Approved = match preview(&document, &request) {
        Ok(approved) => approved,
        Err(problem) => panic!("a proposed request has a preview: {}", problem),
    };
    let saved: Approved = match approve(&document, &template, &request) {
        Ok(approved) => approved,
        Err(problem) => panic!("the same proposal must be savable: {}", problem),
    };

    assert_eq!(
        render(&previewed.document),
        render(&saved.document),
        "the preview gives the document the saved run gives"
    );
    assert_eq!(
        previewed.document.provenance.request_sha256, saved.document.provenance.request_sha256,
        "and its digest is the bytes the save writes, not other bytes"
    );
    assert_eq!(
        previewed.template, saved.template,
        "and the same template letters"
    );
    assert_eq!(previewed.frames, saved.frames, "and the same mutant frames");
    let _ = fs::remove_dir_all(&dir);
}

/// A preview writes nothing (contract 7m8a §2.3): the path it names stays
/// empty, and the workspace lists the entries it had before the preview ran.
#[test]
fn a_preview_writes_nothing() {
    let (dir, document, request) = proposal_of("preview-write", "two-separate");
    let before: Vec<String> = entries_of(&dir);
    assert!(
        !request.exists(),
        "the path the preview names has no file to begin with: {:?}",
        request
    );

    let previewed: Approved = match preview(&document, &request) {
        Ok(approved) => approved,
        Err(problem) => panic!("a proposed request has a preview: {}", problem),
    };
    assert_eq!(
        previewed.document.state, "verified",
        "the preview is the design run of the proposed request"
    );
    assert!(
        !request.exists(),
        "the preview writes no file at {:?}",
        request
    );
    assert_eq!(
        entries_of(&dir),
        before,
        "and the workspace lists the same entries as before the preview"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// A run that asked a question has nothing to approve, so it has no preview
/// either (contract 7m8a §2.3): the same refusal `approve` gives, in the same
/// words, and nothing written.
#[test]
fn a_clarification_has_no_preview() {
    let (dir, document, request) = proposal_of("preview-ask", "composition-unstated");
    assert_eq!(document.state, "needs_clarification", "the case asks");
    assert_eq!(
        preview(&document, &request),
        Err("there is no proposed request to approve".to_string()),
        "there is no proposed request, so there is no run to preview"
    );
    assert!(
        !request.exists(),
        "and the refusal writes nothing: no file at {:?}",
        request
    );
    let _ = fs::remove_dir_all(&dir);
}
