//! Milestone 6's proof (contract 6i): every corpus case through `interpret`
//! ends where `tests/fixtures/nl/cases.tsv` says; each proposal, run through
//! `luria q5 design`, gives the same result as the corpus's hand-written
//! request; and the same reply gives the same bytes. The model is the
//! scripted backend. `live_corpus_report` is the one test that calls a real
//! model; it is ignored unless asked for.

use luria_cli::exit::ExitStatus;
use luria_cli::interpret::document::render_interpretation;
use luria_cli::interpret::run::interpret;
use luria_cli::interpret::run::InterpretInput;
use luria_cli::interpret::run::InterpretOutcome;
use luria_cli::interpret::run::TextSource;
use luria_cli::request::parse_request;
use luria_cli::result::render;
use luria_cli::result::ResultDocument;
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

/// A fresh directory for one case: its template and, when it has one, its
/// profile, copied in under their own names.
fn workspace(test: &str, row: &[String]) -> PathBuf {
    let dir: PathBuf =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("e2e-nl-{}-{}", test, row[0]));
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

/// The codes an outcome ends with, sorted: its questions' codes, or its
/// error's code, or none for a proposal.
fn codes_of(outcome: &InterpretOutcome) -> String {
    let mut codes: Vec<String> = outcome
        .document
        .questions
        .iter()
        .map(|question| question.code.to_string())
        .collect();
    if let Some(error) = &outcome.document.error {
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

/// The corpus's `codes` column, sorted as `codes_of` sorts.
fn expected_codes(column: &str) -> String {
    let mut codes: Vec<&str> = column.split(';').collect();
    codes.sort();
    codes.join(";")
}

fn status_of(state: &str, codes: &str) -> ExitStatus {
    match (state, codes) {
        ("proposed", _) => ExitStatus::Success,
        ("needs_clarification", _) => ExitStatus::NeedsClarification,
        ("failed", "model_output_invalid") | ("failed", "ungrounded_extraction") => {
            ExitStatus::ModelOutputRefused
        }
        other => panic!("the corpus has no state {:?}", other),
    }
}

#[test]
fn every_corpus_case_ends_where_the_corpus_says() {
    let mut states: [usize; 3] = [0, 0, 0];
    for row in cases() {
        let dir: PathBuf = workspace("state", &row);
        let outcome: InterpretOutcome = interpret_case(&dir, &row, None);
        assert_eq!(outcome.document.state, row[5], "{}", row[0]);
        assert_eq!(codes_of(&outcome), expected_codes(&row[6]), "{}", row[0]);
        assert_eq!(outcome.status, status_of(&row[5], &row[6]), "{}", row[0]);
        match row[5].as_str() {
            "proposed" => states[0] += 1,
            "needs_clarification" => states[1] += 1,
            _ => states[2] += 1,
        }
    }
    assert_eq!(states, [5, 15, 4], "proposed, needs_clarification, failed");
}

// The milestone's claim: removing the model changes nothing in the science.
#[test]
fn each_proposal_designs_exactly_as_the_hand_written_request() {
    let mut compared: Vec<String> = Vec::new();
    for row in cases() {
        if row[5] != "proposed" {
            continue;
        }
        let dir: PathBuf = workspace("design", &row);
        let outcome: InterpretOutcome = interpret_case(&dir, &row, Some("proposal.json"));
        assert_eq!(outcome.status, ExitStatus::Success, "{}", row[0]);
        fs::copy(
            format!("{}request/{}", NL, row[7]),
            dir.join("expected.json"),
        )
        .expect("the expected request is copied");
        assert_eq!(
            parse_request(&read(&dir.join("proposal.json").to_string_lossy())).ok(),
            parse_request(&read(&dir.join("expected.json").to_string_lossy())).ok(),
            "{}: the two requests differ",
            row[0]
        );
        let from_words = run(&dir.join("proposal.json"));
        let by_hand = run(&dir.join("expected.json"));
        let wanted: u8 = row[8].parse().expect("a design exit status");
        assert_eq!(
            from_words.status.code(),
            wanted,
            "{}: the proposal's exit",
            row[0]
        );
        assert_eq!(
            by_hand.status.code(),
            wanted,
            "{}: the hand-written exit",
            row[0]
        );
        // The request files' own digests differ by layout; nothing else may.
        let mut aligned: ResultDocument = from_words.document.clone();
        assert!(aligned.provenance.request_sha256.is_some(), "{}", row[0]);
        aligned.provenance.request_sha256 = by_hand.document.provenance.request_sha256.clone();
        assert_eq!(render(&aligned), render(&by_hand.document), "{}", row[0]);
        compared.push(row[0].clone());
    }
    assert_eq!(
        compared,
        vec![
            "one-sub-prose",
            "three-letter",
            "two-separate",
            "two-combined",
            "codon-stated"
        ]
    );
}

#[test]
fn every_corpus_case_gives_the_same_bytes_twice() {
    for row in cases() {
        let dir: PathBuf = workspace("twice", &row);
        let first: String = render_interpretation(&interpret_case(&dir, &row, None).document);
        let second: String = render_interpretation(&interpret_case(&dir, &row, None).document);
        assert_eq!(first, second, "{}", row[0]);
        assert!(!first.contains(env!("CARGO_TARGET_TMPDIR")), "{}", row[0]);
    }
}

/// Run every corpus text through a real model and print what came back.
/// Opt in, by hand, with a model you choose:
///
/// ```text
/// LURIA_LIVE_ENDPOINT=http://127.0.0.1:8080/v1 LURIA_LIVE_MODEL=<model> \
///     cargo test -p luria-cli --test interpret_end_to_end -- --ignored --nocapture
/// ```
///
/// It asserts nothing about the model (plan §8): a live score is
/// development information. Its only failure is a setup it cannot use.
#[test]
#[ignore]
fn live_corpus_report() {
    let endpoint: String = match std::env::var("LURIA_LIVE_ENDPOINT") {
        Ok(found) => found,
        Err(_) => {
            println!("LURIA_LIVE_ENDPOINT is not set; nothing to run");
            return;
        }
    };
    let model: String =
        std::env::var("LURIA_LIVE_MODEL").expect("LURIA_LIVE_MODEL names the model");
    let parsed = luria_nl::endpoint::Endpoint::parse(&endpoint)
        .unwrap_or_else(|error| panic!("{}: {}", endpoint, error.message));
    let mut same: usize = 0;
    let mut total: usize = 0;
    println!("case\tcorpus\tlive");
    for row in cases() {
        let dir: PathBuf = workspace("live", &row);
        let mut backend = luria_nl::openai::OpenAiCompatible::new(parsed.clone(), &model, None);
        let outcome: InterpretOutcome = interpret(&input(&dir, &row, None), &mut backend);
        let corpus: String = format!("{} {}", row[5], expected_codes(&row[6]));
        let live: String = format!("{} {}", outcome.document.state, codes_of(&outcome));
        if corpus == live {
            same += 1;
        }
        total += 1;
        println!("{}\t{}\t{}", row[0], corpus, live);
    }
    println!(
        "{} of {} cases ended as the corpus's scripted reply did. The corpus scripts some \
         faulty replies on purpose (ungrounded-*, reply-*), so a good model differs there.",
        same, total
    );
}
