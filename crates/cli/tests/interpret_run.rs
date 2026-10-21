//! `luria q5 interpret` as a library call (contract 6h1): each state, every
//! local refusal before anything is sent, `--write-request`, the document's
//! fixed shape and its determinism. The model is the scripted backend (6f).

use luria_cli::exit::ExitStatus;
use luria_cli::interpret::document::render_interpretation;
use luria_cli::interpret::document::InterpretationDocument;
use luria_cli::interpret::run::interpret;
use luria_cli::interpret::run::InterpretInput;
use luria_cli::interpret::run::InterpretOutcome;
use luria_cli::interpret::run::TextSource;
use luria_cli::request::parse_request;
use luria_cli::run::run;
use luria_nl::backend::BackendError;
use luria_nl::backend::BackendErrorKind;
use luria_nl::backend::BackendInfo;
use luria_nl::backend::ScriptedBackend;
use luria_nl::extraction::parse_extraction;
use luria_nl::prompt::prompt;
use luria_nl::prompt::CdsLine;
use luria_nl::prompt::CdsState;
use luria_nl::prompt::PROMPT_VERSION;
use std::fs;
use std::path::{Path, PathBuf};

const NL: &str = "../../tests/fixtures/nl/";
const GENBANK: &str = "../../tests/fixtures/genbank/";
/// One local refusal: what it is, the input, the status, the code.
type LocalCase = (&'static str, InterpretInput, ExitStatus, &'static str);

/// The linear record's digest, from `synthetic-linear-record.tsv`.
const DIGEST: &str = "a8b4ddcfa617179143d428dc103aabb7b13cf27f7b323407d11db8a01105bbd7";

/// A fresh directory holding the linear record and the full profile.
fn workspace(test: &str) -> PathBuf {
    let dir: PathBuf =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("interpret-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a workspace");
    fs::copy(
        format!("{}synthetic-linear.gb", GENBANK),
        dir.join("synthetic-linear.gb"),
    )
    .expect("the record is copied");
    fs::copy(format!("{}profile/full.json", NL), dir.join("full.json"))
        .expect("the profile is copied");
    dir
}

fn corpus(file: &str) -> String {
    let path: String = format!("{}{}", NL, file);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("cannot read {}: {}", path, error))
}

/// A corpus text, without its file's final newline.
fn text_of(case: &str) -> String {
    let text: String = corpus(&format!("text/{}.txt", case));
    text.strip_suffix('\n').unwrap_or(&text).to_string()
}

fn reply_of(case: &str) -> Result<String, BackendError> {
    Ok(corpus(&format!("reply/{}.json", case)))
}

fn input(dir: &Path, text: &str, profile: bool, write: Option<&str>) -> InterpretInput {
    InterpretInput {
        template: dir.join("synthetic-linear.gb"),
        text: TextSource::Inline(text.to_string()),
        profile: if profile {
            Some(dir.join("full.json"))
        } else {
            None
        },
        write_request: write.map(|name| dir.join(name)),
    }
}

fn scripted(replies: Vec<Result<String, BackendError>>) -> ScriptedBackend {
    ScriptedBackend::new("m", replies)
}

fn error_code(outcome: &InterpretOutcome) -> String {
    outcome
        .document
        .error
        .as_ref()
        .map(|error| error.code.clone())
        .unwrap_or_else(|| "-".to_string())
}

/// The top-level keys of a rendered document, in order.
fn top_level_keys(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| line.starts_with("  \""))
        .map(|line| line[3..].split('"').next().expect("a key").to_string())
        .collect()
}

#[test]
fn a_proposal_carries_the_request_and_everything_that_made_it() {
    let dir: PathBuf = workspace("proposal");
    let text: String = text_of("one-sub-prose");
    let mut backend: ScriptedBackend = scripted(vec![reply_of("one-sub-prose")]);
    let outcome: InterpretOutcome = interpret(&input(&dir, &text, true, None), &mut backend);
    assert_eq!(outcome.status, ExitStatus::Success);
    assert_eq!(outcome.note, None);
    let document: &InterpretationDocument = &outcome.document;
    assert_eq!(document.schema, "luria.q5-interpretation/1");
    assert_eq!(document.state, "proposed");
    assert_eq!(document.package.name, "luria-cli");
    let echo: String = serde_json::to_string(document.request.as_ref().expect("a request"))
        .expect("the request serializes");
    assert_eq!(
        parse_request(&echo).ok(),
        parse_request(&corpus("request/edit-a42g.json")).ok()
    );
    assert!(document.questions.is_empty());
    let origins: Vec<(&str, &str)> = document
        .origins
        .iter()
        .map(|origin| (origin.value.as_str(), origin.source))
        .collect();
    assert_eq!(
        origins,
        vec![
            ("design", "profile"),
            ("numbering", "text"),
            ("codon A42G", "profile")
        ]
    );
    assert_eq!(document.assumptions.len(), 1);
    assert_eq!(document.assumptions[0].code, "template_id_from_locus");
    assert_eq!(document.error, None);
    let template = document.template.as_ref().expect("the template");
    assert_eq!(
        (
            template.file.as_str(),
            template.locus_name.as_str(),
            template.sha256.as_str()
        ),
        ("synthetic-linear.gb", "synlin", DIGEST)
    );
    let syna: CdsLine = CdsLine {
        id: "synA".to_string(),
        gene: Some("synA".to_string()),
        product: Some("synthetic protein A".to_string()),
        state: CdsState::Usable {
            residues: 100,
            plus_strand: true,
        },
    };
    let sent = prompt(&text, &[syna]);
    let interpretation = &document.interpretation;
    assert_eq!(
        interpretation.backend,
        BackendInfo {
            kind: "scripted".to_string(),
            endpoint: None,
            model: "m".to_string(),
        }
    );
    assert_eq!(interpretation.prompt_version, PROMPT_VERSION);
    assert_eq!(interpretation.messages, Some(sent.clone()));
    assert_eq!(backend.prompts(), &[sent][..]);
    let reply: String = corpus("reply/one-sub-prose.json");
    assert_eq!(interpretation.reply, Some(reply.clone()));
    assert_eq!(interpretation.extraction, parse_extraction(&reply).ok());
    assert_eq!(interpretation.ungrounded, None);
    let rendered: String = render_interpretation(document);
    assert!(rendered.ends_with("}\n"));
    assert_eq!(
        top_level_keys(&rendered),
        vec![
            "schema",
            "state",
            "package",
            "request",
            "questions",
            "origins",
            "assumptions",
            "error",
            "template",
            "interpretation"
        ]
    );
}

#[test]
fn each_outcome_has_its_state_and_status() {
    let dir: PathBuf = workspace("states");
    let cases: [(&str, ExitStatus, &str, &str); 4] = [
        (
            "composition-unstated",
            ExitStatus::NeedsClarification,
            "needs_clarification",
            "-",
        ),
        (
            "ungrounded-cds",
            ExitStatus::ModelOutputRefused,
            "failed",
            "ungrounded_extraction",
        ),
        (
            "reply-extra-field",
            ExitStatus::ModelOutputRefused,
            "failed",
            "model_output_invalid",
        ),
        ("two-combined", ExitStatus::Success, "proposed", "-"),
    ];
    for case in cases {
        let mut backend: ScriptedBackend = scripted(vec![reply_of(case.0)]);
        let outcome: InterpretOutcome =
            interpret(&input(&dir, &text_of(case.0), true, None), &mut backend);
        assert_eq!(outcome.status, case.1, "{}", case.0);
        assert_eq!(outcome.document.state, case.2, "{}", case.0);
        assert_eq!(error_code(&outcome), case.3, "{}", case.0);
    }
    assert_eq!(ExitStatus::ModelOutputRefused.code(), 8);
}

#[test]
fn a_refused_reply_records_what_was_refused() {
    let dir: PathBuf = workspace("refused");
    let mut backend: ScriptedBackend = scripted(vec![reply_of("ungrounded-cds")]);
    let ungrounded: InterpretOutcome = interpret(
        &input(&dir, &text_of("ungrounded-cds"), true, None),
        &mut backend,
    );
    let found = ungrounded
        .document
        .interpretation
        .ungrounded
        .as_ref()
        .expect("a string");
    assert_eq!((found.field.as_str(), found.value.as_str()), ("cds", "bla"));
    assert_eq!(
        ungrounded
            .document
            .error
            .as_ref()
            .expect("an error")
            .message,
        "the model reported cds \"bla\", which the text does not contain"
    );
    assert!(ungrounded.document.interpretation.extraction.is_some());
    assert_eq!(ungrounded.document.request, None);

    let mut backend: ScriptedBackend = scripted(vec![reply_of("reply-extra-field")]);
    let invalid: InterpretOutcome = interpret(
        &input(&dir, &text_of("reply-extra-field"), true, None),
        &mut backend,
    );
    assert_eq!(invalid.document.interpretation.extraction, None);
    assert_eq!(
        invalid.document.interpretation.reply,
        Some(corpus("reply/reply-extra-field.json"))
    );
    assert!(invalid
        .document
        .error
        .as_ref()
        .expect("an error")
        .message
        .starts_with("the model's reply is not an extraction: "));

    let mut backend: ScriptedBackend = scripted(vec![reply_of("composition-unstated")]);
    let asked: InterpretOutcome = interpret(
        &input(&dir, &text_of("composition-unstated"), true, None),
        &mut backend,
    );
    let codes: Vec<&str> = asked
        .document
        .questions
        .iter()
        .map(|question| question.code)
        .collect();
    assert_eq!(codes, vec!["composition_unstated"]);
    assert_eq!(asked.document.request, None);
    assert_eq!(asked.document.error, None);
}

#[test]
fn a_backend_failure_is_reported_after_the_messages_it_was_sent() {
    let dir: PathBuf = workspace("backend");
    let failure: BackendError = BackendError {
        kind: BackendErrorKind::Unreachable,
        message: "could not reach http://127.0.0.1:9/v1: io: Connection refused".to_string(),
    };
    let mut backend: ScriptedBackend = scripted(vec![Err(failure.clone())]);
    let outcome: InterpretOutcome = interpret(
        &input(&dir, &text_of("one-sub-prose"), true, None),
        &mut backend,
    );
    assert_eq!(outcome.status, ExitStatus::Internal);
    assert_eq!(outcome.document.state, "failed");
    assert_eq!(error_code(&outcome), "backend_unreachable");
    assert_eq!(
        outcome.document.error.as_ref().expect("an error").message,
        failure.message
    );
    assert!(outcome.document.interpretation.messages.is_some());
    assert_eq!(outcome.document.interpretation.reply, None);
    assert_eq!(outcome.document.interpretation.extraction, None);
}

#[test]
fn every_local_input_is_checked_before_anything_is_sent() {
    let dir: PathBuf = workspace("local");
    fs::copy(
        "../../tests/fixtures/example-construct/input/sid-1_cDNA.fa",
        dir.join("sid.fa"),
    )
    .expect("the FASTA is copied");
    fs::write(dir.join("broken.gb"), "not a record\n").expect("written");
    fs::write(dir.join("huge.gb"), vec![b'a'; 32 * 1024 * 1024 + 1]).expect("written");
    fs::write(dir.join("empty.txt"), "\n").expect("written");
    fs::write(
        dir.join("bad-profile.json"),
        "{\"schema\": \"luria.q5-design-profile/1\", \"codon\": \"GGC\"}",
    )
    .expect("written");
    fs::write(dir.join("taken.json"), "{}").expect("written");
    fs::create_dir_all(dir.join("elsewhere")).expect("a directory");
    let text: String = text_of("one-sub-prose");
    let base = || input(&dir, &text, true, None);
    let cases: Vec<LocalCase> = vec![
        (
            "absent template",
            InterpretInput {
                template: dir.join("absent.gb"),
                ..base()
            },
            ExitStatus::MalformedInput,
            "template_unreadable",
        ),
        (
            "FASTA template",
            InterpretInput {
                template: dir.join("sid.fa"),
                ..base()
            },
            ExitStatus::Unsupported,
            "interpret_fasta_unsupported",
        ),
        (
            "broken record",
            InterpretInput {
                template: dir.join("broken.gb"),
                ..base()
            },
            ExitStatus::MalformedInput,
            "malformed_genbank",
        ),
        (
            "huge template",
            InterpretInput {
                template: dir.join("huge.gb"),
                ..base()
            },
            ExitStatus::MalformedInput,
            "input_too_large",
        ),
        (
            "absent text",
            InterpretInput {
                text: TextSource::File(dir.join("absent.txt")),
                ..base()
            },
            ExitStatus::MalformedInput,
            "text_unreadable",
        ),
        (
            "empty text",
            InterpretInput {
                text: TextSource::File(dir.join("empty.txt")),
                ..base()
            },
            ExitStatus::MalformedInput,
            "text_empty",
        ),
        (
            "long text",
            InterpretInput {
                text: TextSource::Inline("a".repeat(8 * 1024 + 1)),
                ..base()
            },
            ExitStatus::MalformedInput,
            "input_too_large",
        ),
        (
            "absent profile",
            InterpretInput {
                profile: Some(dir.join("absent.json")),
                ..base()
            },
            ExitStatus::MalformedInput,
            "profile_unreadable",
        ),
        (
            "invalid profile",
            InterpretInput {
                profile: Some(dir.join("bad-profile.json")),
                ..base()
            },
            ExitStatus::MalformedInput,
            "profile_invalid",
        ),
        (
            "request elsewhere",
            InterpretInput {
                write_request: Some(dir.join("elsewhere").join("r.json")),
                ..base()
            },
            ExitStatus::MalformedInput,
            "request_location_unusable",
        ),
        (
            "request in a missing directory",
            InterpretInput {
                write_request: Some(dir.join("missing").join("r.json")),
                ..base()
            },
            ExitStatus::MalformedInput,
            "request_location_unusable",
        ),
        (
            "request over a file",
            InterpretInput {
                write_request: Some(dir.join("taken.json")),
                ..base()
            },
            ExitStatus::MalformedInput,
            "output_exists",
        ),
    ];
    for case in cases {
        let mut backend: ScriptedBackend = scripted(vec![reply_of("one-sub-prose")]);
        let outcome: InterpretOutcome = interpret(&case.1, &mut backend);
        assert_eq!(outcome.status, case.2, "{}", case.0);
        assert_eq!(outcome.document.state, "failed", "{}", case.0);
        assert_eq!(error_code(&outcome), case.3, "{}", case.0);
        assert!(
            backend.prompts().is_empty(),
            "{}: something was sent",
            case.0
        );
        assert_eq!(outcome.document.interpretation.messages, None, "{}", case.0);
        let rendered: String = render_interpretation(&outcome.document);
        assert!(
            !rendered.contains(env!("CARGO_TARGET_TMPDIR")),
            "{}",
            case.0
        );
    }
    let mut backend: ScriptedBackend = scripted(vec![reply_of("one-sub-prose")]);
    let broken: InterpretOutcome = interpret(
        &InterpretInput {
            template: dir.join("broken.gb"),
            ..base()
        },
        &mut backend,
    );
    assert_eq!(
        broken.document.error.as_ref().expect("an error").rule,
        Some("no_locus".to_string())
    );
    assert_eq!(
        fs::read_to_string(dir.join("taken.json")).expect("still there"),
        "{}"
    );
    // Exactly at the cap, the text reaches the model.
    let mut backend: ScriptedBackend = scripted(vec![reply_of("one-sub-prose")]);
    let _ = interpret(
        &InterpretInput {
            text: TextSource::Inline("a".repeat(8 * 1024)),
            ..base()
        },
        &mut backend,
    );
    assert_eq!(backend.prompts().len(), 1);
}

#[test]
fn every_capped_or_odd_input_is_checked_before_anything_is_sent() {
    let dir: PathBuf = workspace("caps");
    fs::write(dir.join("long.txt"), vec![b'a'; 8 * 1024 + 1]).expect("written");
    fs::write(dir.join("big-profile.json"), vec![b' '; 64 * 1024 + 1]).expect("written");
    let latin1: Vec<u8> = [0x41, 0xe9, 0x0a].to_vec();
    for name in ["latin1.txt", "latin1-profile.json", "latin1.gb"] {
        fs::write(dir.join(name), latin1.clone()).expect("written");
    }
    let text: String = text_of("one-sub-prose");
    let base = || input(&dir, &text, true, None);
    let cases: Vec<LocalCase> = vec![
        (
            "text file over the cap",
            InterpretInput {
                text: TextSource::File(dir.join("long.txt")),
                ..base()
            },
            ExitStatus::MalformedInput,
            "input_too_large",
        ),
        (
            "profile over the cap",
            InterpretInput {
                profile: Some(dir.join("big-profile.json")),
                ..base()
            },
            ExitStatus::MalformedInput,
            "input_too_large",
        ),
        (
            "text not UTF-8",
            InterpretInput {
                text: TextSource::File(dir.join("latin1.txt")),
                ..base()
            },
            ExitStatus::MalformedInput,
            "text_unreadable",
        ),
        (
            "profile not UTF-8",
            InterpretInput {
                profile: Some(dir.join("latin1-profile.json")),
                ..base()
            },
            ExitStatus::MalformedInput,
            "profile_unreadable",
        ),
        (
            "template not UTF-8",
            InterpretInput {
                template: dir.join("latin1.gb"),
                ..base()
            },
            ExitStatus::MalformedInput,
            "template_unreadable",
        ),
        (
            "template names no file",
            InterpretInput {
                template: dir.join(".."),
                ..base()
            },
            ExitStatus::MalformedInput,
            "template_unreadable",
        ),
        (
            "request ends in a separator",
            InterpretInput {
                write_request: Some(dir.join("new.json/")),
                ..base()
            },
            ExitStatus::MalformedInput,
            "request_location_unusable",
        ),
        (
            "request ends in a dot",
            InterpretInput {
                write_request: Some(dir.join("new.json").join(".")),
                ..base()
            },
            ExitStatus::MalformedInput,
            "request_location_unusable",
        ),
    ];
    for case in cases {
        let mut backend: ScriptedBackend = scripted(vec![reply_of("one-sub-prose")]);
        let outcome: InterpretOutcome = interpret(&case.1, &mut backend);
        assert_eq!(outcome.status, case.2, "{}", case.0);
        assert_eq!(outcome.document.state, "failed", "{}", case.0);
        assert_eq!(error_code(&outcome), case.3, "{}", case.0);
        assert!(
            backend.prompts().is_empty(),
            "{}: something was sent",
            case.0
        );
        assert_eq!(outcome.document.interpretation.messages, None, "{}", case.0);
        let rendered: String = render_interpretation(&outcome.document);
        assert!(
            !rendered.contains(env!("CARGO_TARGET_TMPDIR")),
            "{}",
            case.0
        );
    }
    let mut backend: ScriptedBackend = scripted(vec![reply_of("one-sub-prose")]);
    let names_no_file: InterpretOutcome = interpret(
        &InterpretInput {
            template: dir.join(".."),
            ..base()
        },
        &mut backend,
    );
    assert_eq!(
        names_no_file
            .document
            .error
            .as_ref()
            .expect("an error")
            .message,
        "the template path names no file".to_string()
    );
    assert!(
        backend.prompts().is_empty(),
        "template names no file: something was sent"
    );
}

#[cfg(unix)]
#[test]
fn a_proposal_whose_write_fails_is_still_shown_and_exits_1() {
    use std::os::unix::fs::PermissionsExt;
    let dir: PathBuf = workspace("write-fails");
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o555))
        .expect("the workspace is not writable");
    let mut backend: ScriptedBackend = scripted(vec![reply_of("one-sub-prose")]);
    let outcome: InterpretOutcome = interpret(
        &input(&dir, &text_of("one-sub-prose"), true, Some("proposal.json")),
        &mut backend,
    );
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755))
        .expect("the workspace is writable again");
    assert_eq!(outcome.status, ExitStatus::Internal);
    assert_eq!(outcome.document.state, "proposed");
    assert!(outcome.document.request.is_some());
    let note = outcome.note.as_ref().expect("a line for stderr");
    assert!(
        note.starts_with("could not write proposal.json"),
        "the note names the file that could not be written"
    );
    assert!(!dir.join("proposal.json").exists());
}

#[cfg(unix)]
#[test]
fn a_dangling_link_is_a_file_already_there() {
    let dir: PathBuf = workspace("link");
    std::os::unix::fs::symlink(dir.join("nowhere.json"), dir.join("link.json")).expect("a link");
    let mut backend: ScriptedBackend = scripted(vec![reply_of("one-sub-prose")]);
    let outcome: InterpretOutcome = interpret(
        &input(&dir, &text_of("one-sub-prose"), true, Some("link.json")),
        &mut backend,
    );
    assert_eq!(error_code(&outcome), "output_exists");
    assert!(backend.prompts().is_empty());
    assert!(!dir.join("nowhere.json").exists());
}

// The proposal is written beside the template, and `luria q5 design` runs it
// there: the template path in it is relative to the request (§3.6 step 4).
#[test]
fn a_written_request_runs_where_it_was_written() {
    let dir: PathBuf = workspace("write");
    let mut backend: ScriptedBackend = scripted(vec![reply_of("two-separate")]);
    let outcome: InterpretOutcome = interpret(
        &input(&dir, &text_of("two-separate"), true, Some("proposal.json")),
        &mut backend,
    );
    assert_eq!(outcome.status, ExitStatus::Success);
    let written: String = fs::read_to_string(dir.join("proposal.json")).expect("written");
    let echo = outcome.document.request.as_ref().expect("a request");
    assert_eq!(
        written,
        serde_json::to_string_pretty(echo).expect("serializes") + "\n"
    );
    assert_eq!(
        parse_request(&written).ok(),
        parse_request(&corpus("request/variants-separate.json")).ok()
    );
    assert_eq!(run(&dir.join("proposal.json")).status, ExitStatus::Success);
}

#[test]
fn nothing_is_written_unless_a_request_is_proposed() {
    let dir: PathBuf = workspace("no-write");
    let mut backend: ScriptedBackend = scripted(vec![reply_of("composition-unstated")]);
    let outcome: InterpretOutcome = interpret(
        &input(
            &dir,
            &text_of("composition-unstated"),
            true,
            Some("proposal.json"),
        ),
        &mut backend,
    );
    assert_eq!(outcome.status, ExitStatus::NeedsClarification);
    assert!(!dir.join("proposal.json").exists());
}

#[test]
fn a_text_file_loses_one_final_line_break() {
    let dir: PathBuf = workspace("text-file");
    let text: String = text_of("one-sub-prose");
    fs::write(dir.join("unix.txt"), format!("{}\n", text)).expect("written");
    fs::write(dir.join("dos.txt"), format!("{}\r\n", text)).expect("written");
    for name in ["unix.txt", "dos.txt"] {
        let mut backend: ScriptedBackend = scripted(vec![reply_of("one-sub-prose")]);
        let request: InterpretInput = InterpretInput {
            text: TextSource::File(dir.join(name)),
            ..input(&dir, "", true, None)
        };
        let outcome: InterpretOutcome = interpret(&request, &mut backend);
        assert_eq!(outcome.status, ExitStatus::Success, "{}", name);
        assert!(
            backend.prompts()[0]
                .user
                .ends_with(&format!("Request:\n{}", text)),
            "{}",
            name
        );
    }
}

#[test]
fn the_same_reply_gives_the_same_bytes() {
    let dir: PathBuf = workspace("determinism");
    let once = || -> String {
        let mut backend: ScriptedBackend = scripted(vec![reply_of("two-separate")]);
        render_interpretation(
            &interpret(
                &input(&dir, &text_of("two-separate"), true, None),
                &mut backend,
            )
            .document,
        )
    };
    let first: String = once();
    assert_eq!(first, once());
    assert!(!first.contains(env!("CARGO_TARGET_TMPDIR")));
}
