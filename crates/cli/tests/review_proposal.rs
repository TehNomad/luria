//! The text of `luria review`'s proposal screen (milestone 7e): the lines
//! `luria_cli::review::text::proposal` builds from the
//! `InterpretationDocument` one `luria q5 interpret` run answers with (plan §5.2
//! screen 1). Each corpus case runs through `interpret` with the scripted
//! backend, exactly as `tests/interpret_end_to_end.rs` runs it.
//!
//! Every expected value is one the corpus or the document holds on its own: the
//! state, the codes and the case names are `tests/fixtures/nl/cases.tsv`'s own
//! columns, recorded in `tests/fixtures/nl/README.md`; the evidence quotes are the
//! tokens and quotes `tests/fixtures/nl/reply/<case>.json` holds; the request
//! shown is compared with the file `--write-request` writes; the user message, the
//! reply and the ungrounded string are compared with the document's own fields.
//! Nothing is copied from running the code, and no value is computed, parsed out
//! of a line, re-derived or summarised (plan §3 rule 1).
//!
//! One test, `questions_come_before_a_request`, uses a synthetic document: the
//! corpus never pairs a request with questions, so that test writes the question
//! and the error itself, and pins the screen's order, not a corpus fact.

use luria_cli::interpret::document::InterpretErrorView;
use luria_cli::interpret::document::InterpretationDocument;
use luria_cli::interpret::document::QuestionView;
use luria_cli::interpret::document::UngroundedView;
use luria_cli::interpret::run::interpret;
use luria_cli::interpret::run::InterpretInput;
use luria_cli::interpret::run::InterpretOutcome;
use luria_cli::interpret::run::TextSource;
use luria_cli::review::text;
use luria_nl::backend::ScriptedBackend;
use luria_nl::extraction::parse_extraction;
use luria_nl::extraction::Extraction;
use luria_nl::prompt::Prompt;
use std::fs;
use std::path::{Path, PathBuf};

const NL: &str = "../../tests/fixtures/nl/";
const GENBANK: &str = "../../tests/fixtures/genbank/";
const CASES_HEADER: &str = "case_id\ttemplate\ttext_file\treply_file\tprofile\tstate\tcodes\trequest_file\tdesign_exit\tnote";

/// The line that opens the proposed request group, `text::proposal`'s own words.
const REQUEST_HEADER: &str = "proposed request, written exactly so when you approve it:";

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
/// profile, copied in under their own names.
fn workspace(test: &str, row: &[String]) -> PathBuf {
    let dir: PathBuf = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("review-proposal-{}-{}", test, row[0]));
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

fn input(dir: &Path, row: &[String], write: Option<&str>) -> InterpretInput {
    InterpretInput {
        template: dir.join(&row[1]),
        text: TextSource::Inline(text_of(row)),
        profile: if row[4] == "-" {
            None
        } else {
            Some(dir.join(&row[4]))
        },
        write_request: write.map(|name| dir.join(name)),
    }
}

fn interpret_case(dir: &Path, row: &[String], write: Option<&str>) -> InterpretOutcome {
    let reply: String = read(&format!("{}reply/{}", NL, row[3]));
    let mut backend: ScriptedBackend = ScriptedBackend::new("scripted-corpus", vec![Ok(reply)]);
    interpret(&input(dir, row, write), &mut backend)
}

/// The codes an outcome ends with, sorted: its questions' codes, or its error's
/// code, or `-` for a proposal. As `tests/interpret_end_to_end.rs` reads them.
fn codes_of(document: &InterpretationDocument) -> String {
    let mut codes: Vec<String> = document
        .questions
        .iter()
        .map(|question| question.code.to_string())
        .collect();
    if let Some(error) = &document.error {
        codes.push(error.code.clone());
    }
    codes.sort();
    codes.dedup();
    if codes.is_empty() {
        "-".to_string()
    } else {
        codes.join(";")
    }
}

/// The proposal screen's lines for one corpus case, run through the scripted
/// backend in the test's own directory, with `write` as the `--write-request`
/// path when given. The state and the codes are checked against the corpus's own
/// columns, as `tests/interpret_end_to_end.rs` does, and the case's directory is
/// given back so a test can read the file the run wrote.
fn proposal_of(
    test: &str,
    row: &[String],
    write: Option<&str>,
) -> (PathBuf, InterpretationDocument, Vec<String>) {
    let dir: PathBuf = workspace(test, row);
    let outcome: InterpretOutcome = interpret_case(&dir, row, write);
    assert_eq!(
        outcome.document.state, row[5],
        "{}: the corpus's own state column",
        row[0]
    );
    let mut expected: Vec<&str> = row[6].split(';').collect();
    expected.sort();
    assert_eq!(
        codes_of(&outcome.document),
        expected.join(";"),
        "{}: the corpus's own codes column",
        row[0]
    );
    (
        dir,
        outcome.document.clone(),
        text::proposal(&outcome.document),
    )
}

/// How many times `expected` appears in `lines`, so "one line" is tested as one,
/// not at least one.
fn count(lines: &[String], expected: &str) -> usize {
    lines.iter().filter(|line| *line == expected).count()
}

#[test]
fn the_proposed_request_is_what_is_written() {
    let row: Vec<String> = row_of("two-separate");
    let (dir, document, lines) = proposal_of("request", &row, Some("proposal.json"));
    assert_eq!(document.state, "proposed", "the case is a proposal");
    assert!(document.request.is_some(), "a proposal names its request");

    let header: &str = REQUEST_HEADER;
    let at: usize = lines
        .iter()
        .position(|line| line == header)
        .expect("the proposed request group");
    let mut shown: Vec<String> = Vec::new();
    for line in &lines[at + 1..] {
        if line.is_empty() {
            break;
        }
        shown.push(
            line.strip_prefix("  ")
                .expect("a request line indented two spaces")
                .to_string(),
        );
    }
    let written: String = fs::read_to_string(dir.join("proposal.json"))
        .expect("the request file the run wrote is there to read");
    assert_eq!(
        shown.join("\n") + "\n",
        written,
        "what the screen shows is byte for byte what approving writes (plan §3 rule 2)"
    );
}

#[test]
fn questions_come_first() {
    let row: Vec<String> = row_of("composition-unstated");
    let (_, document, lines) = proposal_of("questions", &row, None);
    assert_eq!(row[6], "composition_unstated", "the corpus's own code");
    assert_eq!(
        lines[0],
        "questions:".to_string(),
        "the questions come before anything else (plan §3 rule 3): {:?}",
        lines
    );
    let question: &QuestionView = &document.questions[0];
    assert!(
        lines[1].starts_with("  composition_unstated ("),
        "the question's own code opens its line: {:?}",
        lines
    );
    assert_eq!(
        lines[1],
        format!(
            "  {} ({}): {}",
            question.code, question.subject, question.message
        ),
        "code, subject and message, each the question's own"
    );
    assert!(
        !lines
            .iter()
            .any(|line| line.starts_with("proposed request")),
        "a run that asks proposes nothing, so no request is shown: {:?}",
        lines
    );
}

#[test]
fn questions_come_before_a_request() {
    // The document is synthetic: no corpus case pairs a request with questions, so
    // the question and the error this test writes exist to pin the screen's order
    // (plan §3 rule 3), not a corpus fact.
    let row: Vec<String> = row_of("two-separate");
    let (_, mut document, _) = proposal_of("order", &row, None);
    document.questions.push(QuestionView {
        code: "composition_unstated",
        subject: "variants.composition".to_string(),
        message: "say separate_variants or combined_construct".to_string(),
    });
    document.error = Some(InterpretErrorView {
        code: "ungrounded_extraction".to_string(),
        rule: Some("q5_numbering".to_string()),
        message: "the text does not state the numbering".to_string(),
    });
    let lines: Vec<String> = text::proposal(&document);
    let at_line = |wanted: &str| {
        lines
            .iter()
            .position(|line| line == wanted)
            .unwrap_or_else(|| panic!("{}: the group is not on the screen: {:?}", wanted, lines))
    };
    let questions: usize = at_line("questions:");
    let error: usize = at_line("error: ungrounded_extraction");
    let request: usize = at_line(REQUEST_HEADER);
    assert!(
        questions < request,
        "the questions come before the request (plan §3 rule 3): {:?}",
        lines
    );
    assert!(
        error < request,
        "the error comes before the request (plan §3 rule 3): {:?}",
        lines
    );
    assert!(
        questions < error,
        "the questions come before the error: {:?}",
        lines
    );
    assert!(
        request < at_line("sent to the model:"),
        "the request comes before what was sent: {:?}",
        lines
    );
}

#[test]
fn an_ungrounded_reply_names_the_string() {
    let row: Vec<String> = row_of("ungrounded-mutation");
    let (_, document, lines) = proposal_of("ungrounded", &row, None);
    assert_eq!(row[6], "ungrounded_extraction", "the corpus's own code");
    let ungrounded: &UngroundedView = document
        .interpretation
        .ungrounded
        .as_ref()
        .expect("the document names the string the text does not hold");
    assert_eq!(
        format!("error: {}", row[6]),
        lines[0],
        "the error comes first (plan §3 rule 3): {:?}",
        lines
    );
    let named: Vec<&String> = lines
        .iter()
        .filter(|line| line.starts_with("  not in your text: "))
        .collect();
    assert_eq!(
        named.len(),
        1,
        "the string the reply made up is named once: {:?}",
        lines
    );
    assert!(
        named[0].starts_with(&format!("  not in your text: {}", ungrounded.field)),
        "the line names the place in the extraction the reply got wrong: {:?}",
        lines
    );
    assert_eq!(
        named[0].as_str(),
        format!(
            "  not in your text: {} \"{}\"",
            ungrounded.field, ungrounded.value
        ),
        "and quotes the string as the reply wrote it"
    );
}

#[test]
fn a_reply_that_is_not_an_extraction_shows_the_reply() {
    let row: Vec<String> = row_of("reply-extra-field");
    let (_, document, lines) = proposal_of("not-an-extraction", &row, None);
    assert_eq!(row[6], "model_output_invalid", "the corpus's own code");
    assert_eq!(
        format!("error: {}", row[6]),
        lines[0],
        "the error comes first (plan §3 rule 3): {:?}",
        lines
    );
    assert!(
        document.interpretation.extraction.is_none(),
        "the reply was not read as an extraction"
    );
    assert!(
        !lines
            .iter()
            .any(|line| line.starts_with("the model's reading")),
        "a reply that is not an extraction is never shown as one: {:?}",
        lines
    );
    let reply: &String = document
        .interpretation
        .reply
        .as_ref()
        .expect("the reply as received is in the document");
    let expected: Vec<String> = reply.lines().map(|line| format!("  {}", line)).collect();
    let header: &str = "the model's reply, as received:";
    let at: usize = lines
        .iter()
        .position(|line| line == header)
        .expect("the reply group");
    assert_eq!(
        &lines[at + 1..],
        expected,
        "the reply, line by line, as the document holds it (D021): {:?}",
        lines
    );
}

#[test]
fn what_was_sent_is_shown() {
    let row: Vec<String> = row_of("two-separate");
    let (_, document, lines) = proposal_of("sent", &row, None);
    let messages: &Prompt = document
        .interpretation
        .messages
        .as_ref()
        .expect("the messages sent are in the document");
    let user: &str = "  user message:";
    let at: usize = lines
        .iter()
        .position(|line| line == user)
        .expect("the user message line");
    let shown: Vec<String> = messages
        .user
        .lines()
        .map(|line| format!("    {}", line))
        .collect();
    assert_eq!(
        &lines[at + 1..at + 1 + shown.len()],
        shown,
        "the user message in full, four spaces in (plan §3 rule 6): {:?}",
        lines
    );
    assert_eq!(
        shown.last().unwrap(),
        &format!("    {}", text_of(&row)),
        "the user's own prose is the last line of what was sent"
    );
    assert_eq!(
        count(
            &lines,
            &format!(
                "  system prompt: {} lines, not shown",
                messages.system.lines().count()
            )
        ),
        1,
        "the system prompt is counted by its own lines, not shown: {:?}",
        lines
    );
}

#[test]
fn the_evidence_quotes_each_mutation() {
    let row: Vec<String> = row_of("two-separate");
    let (_, document, lines) = proposal_of("evidence", &row, None);
    // The corpus reply file is the source of the tokens and quotes the screen
    // quotes, so the document's extraction is checked against the file first.
    let reply: String = read(&format!("{}reply/{}", NL, row[3]));
    let from_file: Extraction =
        parse_extraction(&reply).expect("the corpus reply is an extraction");
    assert_eq!(
        Some(&from_file),
        document.interpretation.extraction.as_ref(),
        "the document holds the corpus reply as parsed"
    );
    assert_eq!(
        count(&lines, "the model's reading, checked against your text:"),
        1,
        "the reading is one group: {:?}",
        lines
    );
    assert_eq!(count(&lines, "  A42G  quote \"A42G\""), 1, "{:?}", lines);
    assert_eq!(count(&lines, "  W87A  quote \"W87A\""), 1, "{:?}", lines);
    for mutation in &from_file.mutations {
        assert_eq!(
            count(
                &lines,
                &format!("  {}  quote \"{}\"", mutation.token, mutation.quote)
            ),
            1,
            "one row per mutation, the user's words quoted: {:?}",
            lines
        );
    }
    assert_eq!(
        count(&lines, "  composition: separate_variants"),
        1,
        "{:?}",
        lines
    );
    assert!(
        reply.contains("\"composition\": \"separate_variants\""),
        "the reply file states that composition"
    );
}
