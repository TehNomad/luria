# Milestone 6i: the corpus end to end

Status: **Waiting on 6h1, 2026-10-19.** Start only when 6h1 is committed
(6h2 is not needed).
Planner: Claude. Parent: `docs/tasks/milestone-6-nl.md` §1 ("The milestone
succeeds when..."), §8, §6 (6i); the product brief's "Definition of
success": *natural language can produce the same typed operation and
deterministic scientific result without changing the underlying
calculation*.

## 1. What and why

This contract adds tests only. It is the milestone's proof:

1. **Every corpus case, through `interpret`**, with its scripted reply,
   ends in the state and codes `tests/fixtures/nl/cases.tsv` records.
2. **Same request, same science.** For each of the five proposals,
   `luria q5 design` on the request `interpret` wrote gives the same result
   document, byte for byte, as `luria q5 design` on the corpus's
   hand-written request, and the exit status the corpus records
   (`design_exit`). The only field allowed to differ is
   `provenance.requestSha256`, which is the digest of each request *file's*
   bytes: the two files say the same thing in different layouts.
3. **Determinism.** Each case's interpretation document is byte-identical
   when run twice (D012).
4. **A live check, opt in.** An `#[ignore]` test runs every corpus text
   through a real model and prints what came back. It asserts nothing about
   the model: a live score is development information, not a correctness
   claim (plan §8).

## 2. Requirements this contract applies

- **Plan §8** and `tests/fixtures/nl/README.md`: `codes` compared as a
  sorted set; a proposal compared as a parsed request; the end-to-end claim
  compared as `q5 design` output.
- **D012:** byte-identical output for the same inputs.
- **No network** in the required checks. The live test is `#[ignore]` and
  reads its endpoint from the environment, so `cargo test` never runs it.

## 3. Files

One new file, `crates/cli/tests/interpret_end_to_end.rs`, exactly as §4.
Change no other file.

## 4. The test file (planner-written; add verbatim)

```rust
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
    let dir: PathBuf = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("e2e-nl-{}-{}", test, row[0]));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a workspace");
    fs::copy(format!("{}{}", GENBANK, row[1]), dir.join(&row[1])).expect("the template is copied");
    if row[4] != "-" {
        fs::copy(format!("{}profile/{}", NL, row[4]), dir.join(&row[4])).expect("the profile is copied");
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
        profile: if row[4] == "-" { None } else { Some(dir.join(&row[4])) },
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
        fs::copy(format!("{}request/{}", NL, row[7]), dir.join("expected.json"))
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
        assert_eq!(from_words.status.code(), wanted, "{}: the proposal's exit", row[0]);
        assert_eq!(by_hand.status.code(), wanted, "{}: the hand-written exit", row[0]);
        // The request files' own digests differ by layout; nothing else may.
        let mut aligned: ResultDocument = from_words.document.clone();
        assert!(aligned.provenance.request_sha256.is_some(), "{}", row[0]);
        aligned.provenance.request_sha256 = by_hand.document.provenance.request_sha256.clone();
        assert_eq!(render(&aligned), render(&by_hand.document), "{}", row[0]);
        compared.push(row[0].clone());
    }
    assert_eq!(
        compared,
        vec!["one-sub-prose", "three-letter", "two-separate", "two-combined", "codon-stated"]
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
    let model: String = std::env::var("LURIA_LIVE_MODEL").expect("LURIA_LIVE_MODEL names the model");
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
```

`cargo fmt` may reflow these; do not change content. If a test fails,
report the case and what the code returned; do not change the test or a
fixture. If a planner-written test fails to compile or fails clippy, stop
and report it rather than editing it.

### 4.1 Proving the tests are load-bearing (AGENTS.md)

These change a fixture or the code temporarily; restore each and show the
restore (`git diff --exit-code` on the tracked file, or a checksum):

| Change, temporarily | Test that must fail |
| --- | --- |
| `tests/fixtures/nl/cases.tsv`: `two-combined`'s `design_exit` `6` → `0` | `each_proposal_designs_exactly_as_the_hand_written_request` |
| `tests/fixtures/nl/request/edit-a42g.json`: `"number": 42` → `43` | `each_proposal_designs_exactly_as_the_hand_written_request` |
| `tests/fixtures/nl/cases.tsv`: `cds-unnamed`'s codes `cds_unnamed` → `cds_not_found` | `every_corpus_case_ends_where_the_corpus_says` |

Also run the ignored test once with `LURIA_LIVE_ENDPOINT` unset, as
`cargo test -p luria-cli --test interpret_end_to_end -- --ignored`, and
report that it prints its "nothing to run" line and passes. **Do not run it
against a model**: the planner does that.

## 5. Do not

- Change any file but the new test file, apart from the temporary §4.1
  changes, each restored.
- Make any test but `live_corpus_report` read the environment, or touch the
  network.
- Remove `#[ignore]`.

## 6. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 7. Handoff

Report the new file, the exact commands and their results, the §4.1
results with each restore shown, and anything that did not match this
contract. Quote command output for every claim; say plainly what you did
not check.
