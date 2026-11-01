# Milestone 6h1: `interpret`, its document and exit status 8

Status: **Implemented 2026-10-19** by the local worker, accepted after one
review round. Review notes:
- **Planner defect in §4.1, fixed:** the `create_new` row could not fail,
  because step 4 refuses a dangling link before the write; `create_new` only
  guards a file appearing between steps 4 and 8, which no integration test
  can stage. The round added the unit test `a_write_never_replaces_a_file`,
  which the worker proved fails with `create(true).truncate(true)`.
- **Fixed:** `read_capped` allocated the whole cap (32 MiB for a template)
  and failed on an interrupted read; it now uses `take(cap + 1)` and
  `read_to_end`.
Planner: Claude. Parent: `docs/tasks/contracts/milestone-6-nl.md` §3, §5.5, §5.7,
M6-7, M6-10, M6-14, §6 (6h, split into 6h1 and 6h2).

## 1. What and why

Everything `luria q5 interpret` does except reading its command line and
choosing a backend: a library function that takes the input files and a
`Backend`, and returns the one JSON document the command prints, its exit
status, and an optional note for stderr. Because the backend is a
parameter, every path is tested with the scripted backend (6f), and 6h2
only has to wire arguments and a real backend to it.

Two rules shape the order of the steps:

- **Nothing is sent until every local input is valid.** A missing file, a
  FASTA template, a bad profile or an unusable `--write-request` path is
  refused before the model is called, so a mistake never costs a model call
  and never sends anything.
- **The document is the answer, in every state**, with a fixed shape, as
  `luria q5 design`'s is.

## 2. Requirements this contract applies

- **Plan §5.7:** one `luria.q5-interpretation/1` document on stdout in
  every state; the interpretation record (backend, prompt version, the exact
  messages, the raw reply, the parsed extraction) kept apart from the
  request; `--write-request` writes the bare request beside the template,
  never over an existing file; size caps on every input (review B-8):
  text 8 KiB, profile 64 KiB, template 32 MiB.
- **M6-7:** GenBank only. A FASTA template is refused as
  `interpret_fasta_unsupported`.
- **M6-14:** exit 8 for a refused model reply.
- **D012:** for the same reply, the same bytes; no absolute path anywhere in
  the document or its messages.
- **D003:** this module decides nothing about biology; `build` (6e2) and
  core do.

## 3. Changes

### 3.1 `crates/cli/src/lib.rs`

After `PROFILE_SCHEMA`, add:

```rust
/// The schema of `luria q5 interpret`'s document (milestone 6 plan §5.7).
pub const INTERPRETATION_SCHEMA: &str = "luria.q5-interpretation/1";
```

### 3.2 `crates/cli/src/exit.rs`

Add a last variant, with its arm in `code()`:

```rust
/// 8: `failed`, because the model's reply was refused: not an extraction,
/// or not grounded in the user's text (milestone 6, M6-14).
ModelOutputRefused,
```

`code()` maps it to `8`. Change nothing else; the existing tests stay as
they are.

### 3.3 `crates/cli/src/interpret.rs`

The module list becomes `build`, `catalogue`, `document`, `evidence`,
`notation`, `run`, in that order. Change nothing else.

### 3.4 `crates/cli/src/interpret/document.rs` (new)

The document's types, all `#[derive(Clone, Debug, PartialEq, Serialize)]`
and `#[serde(rename_all = "camelCase")]`, fields in exactly this order. No
field is ever skipped: an absent value is `null` or `[]`.

```rust
use crate::result::PackageView;
use crate::result::RequestEcho;
use luria_nl::backend::BackendInfo;
use luria_nl::extraction::Extraction;
use luria_nl::prompt::Prompt;

pub struct InterpretationDocument {
    /// `INTERPRETATION_SCHEMA`.
    pub schema: &'static str,
    /// `proposed`, `needs_clarification` or `failed`.
    pub state: &'static str,
    /// `luria-cli` and `env!("CARGO_PKG_VERSION")`.
    pub package: PackageView,
    /// The proposed request, as `request_echo` writes it; null otherwise.
    pub request: Option<RequestEcho>,
    pub questions: Vec<QuestionView>,
    pub origins: Vec<OriginView>,
    pub assumptions: Vec<AssumptionView>,
    pub error: Option<InterpretErrorView>,
    /// The template, once it has been read and its CDSs listed.
    pub template: Option<TemplateFileView>,
    pub interpretation: InterpretationView,
}

pub struct QuestionView { pub code: &'static str, pub subject: String, pub message: String }
/// `source` is `text` or `profile`.
pub struct OriginView { pub value: String, pub source: &'static str }
pub struct AssumptionView { pub code: String, pub message: String }
pub struct InterpretErrorView { pub code: String, pub rule: Option<String>, pub message: String }
/// `file` is the template's file name alone, never a directory (D012).
pub struct TemplateFileView { pub file: String, pub locus_name: String, pub sha256: String }
pub struct UngroundedView { pub field: String, pub value: String }

/// What the model was asked and said: kept apart from the request (D021).
pub struct InterpretationView {
    pub backend: BackendInfo,
    /// `luria_nl::prompt::PROMPT_VERSION`.
    pub prompt_version: &'static str,
    /// The messages sent; null when nothing was sent.
    pub messages: Option<Prompt>,
    /// The reply text exactly as received; null when none came back.
    pub reply: Option<String>,
    /// The reply as parsed; null when it was not an extraction.
    pub extraction: Option<Extraction>,
    /// The first string the reply holds that the text does not.
    pub ungrounded: Option<UngroundedView>,
}

/// The document as printed: `serde_json` pretty form and one `\n`, as every
/// luria document is (5d).
pub fn render_interpretation(document: &InterpretationDocument) -> String;
```

`render_interpretation` mirrors `result::render`: `to_string_pretty`, plus
`"\n"`. A serialization failure cannot happen for these types; mirror
`render`'s handling of it.

### 3.5 `crates/cli/src/interpret/run.rs` (new)

```rust
use crate::exit::ExitStatus;
use crate::interpret::document::InterpretationDocument;
use luria_nl::backend::Backend;
use std::path::PathBuf;

/// The most bytes read of each input (plan §5.7, review B-8).
pub const MAX_TEXT_BYTES: usize = 8 * 1024;
pub const MAX_PROFILE_BYTES: usize = 64 * 1024;
pub const MAX_TEMPLATE_BYTES: usize = 32 * 1024 * 1024;

/// Where the user's text comes from.
#[derive(Clone, Debug, PartialEq)]
pub enum TextSource {
    Inline(String),
    File(PathBuf),
}

/// What `luria q5 interpret` was given, apart from its backend.
#[derive(Clone, Debug, PartialEq)]
pub struct InterpretInput {
    pub template: PathBuf,
    pub text: TextSource,
    pub profile: Option<PathBuf>,
    pub write_request: Option<PathBuf>,
}

/// The document, the exit status, and a line for stderr when writing the
/// request failed after the document was complete.
#[derive(Clone, Debug, PartialEq)]
pub struct InterpretOutcome {
    pub document: InterpretationDocument,
    pub status: ExitStatus,
    pub note: Option<String>,
}

pub fn interpret(input: &InterpretInput, backend: &mut dyn Backend) -> InterpretOutcome;
```

### 3.6 What `interpret` does, in order

Each refusal below ends the run with `state: "failed"`, `error: { code,
rule, message }` (`rule` null unless stated), the status given, and every
field not yet known null or empty. `interpretation.backend` is always
`backend.describe()` and `promptVersion` always `PROMPT_VERSION`. Messages
are exact; `<io>` is the `std::io::Error`'s `Display`, which names no path.

**Reading a file with a cap.** Open it and read at most `cap + 1` bytes
(`Read::take`). More than `cap` bytes: `input_too_large`, status 4,
message `the <what> is more than <cap> bytes, the most luria reads`, with
`<what>` `text`, `design profile` or `template`. The other refusals of
each file are named in its step.

1. **The text.** `Inline(s)`: its bytes, with the same cap. `File(p)`:
   unreadable is `text_unreadable`, `cannot read the text file: <io>`;
   not UTF-8 is `text_unreadable`, `the text file is not UTF-8`. Either
   way, remove one trailing `\n` if there is one, then one trailing `\r` if
   there is one. If what is left is empty after `trim`: `text_empty`, `the
   text is empty`. All status 4.
2. **The profile**, when given. Unreadable: `profile_unreadable`, `cannot
   read the design profile: <io>`; not UTF-8: `profile_unreadable`, `the
   design profile is not UTF-8`; `parse_profile` refuses it:
   `profile_invalid`, message `<path>: <message>` from its `RequestError`.
   All status 4.
3. **The template.** Its file name (`Path::file_name`) as UTF-8, or
   `template_unreadable`, `the template file's name is not UTF-8`.
   Unreadable: `template_unreadable`, `cannot read the template file:
   <io>`; not UTF-8: `template_unreadable`, `the template file is not
   UTF-8`. All status 4. If its first char that is not whitespace is `>`:
   `interpret_fasta_unsupported`, status 6, message `the template is FASTA;
   luria q5 interpret reads GenBank only, so that CDSs come from the record
   (M6-7). Write the request by hand for a FASTA template`. Then
   `read_genbank(Identifier "template", text)` and `catalogue(&record)`;
   either's `DomainError` gives `code: error.code()`, `rule:
   result::rule_code(&error)`, `message: error.message()`, status
   `ExitStatus::for_failure(error.category())`. From here `template` is
   `{ file, locusName: record.locus_name(), sha256: the sequence's
   content_hash().hex_digest() }`.
4. **The request path**, when `write_request` is given. The directory of a
   path is its `parent()`, or `.` when that is `None` or empty. If
   `canonicalize` of the request's directory fails, or differs from
   `canonicalize` of the template's directory, or the request's file name is
   not UTF-8: `request_location_unusable`, `the request must be written in
   the template's directory, because a request names its template relative
   to itself`. If `std::fs::symlink_metadata` of the request path succeeds
   (anything is there, a dangling link included): `output_exists`, `the
   file <name> already exists; luria never replaces a file`, `<name>` the
   file name alone. Both status 4.
5. **The prompt and the call.** Convert each catalogue entry to a
   `CdsLine` (`Usable { residues, strand }` becomes `CdsState::Usable {
   residues, plus_strand: strand == Strand::Plus }`, `Refused` becomes
   `NotUsable`), build `prompt(text, &lines)`, and set
   `interpretation.messages` to it. Then `backend.complete(&prompt)`. An
   error: `code: error.kind.code()`, `message: error.message`, status 1.
   Otherwise set `interpretation.reply` to the reply text.
6. **The extraction.** `parse_extraction(&reply)`. A refusal:
   `model_output_invalid`, `the model's reply is not an extraction:
   <message>`, status 8. Otherwise set `interpretation.extraction`.
7. **Building.** `build(text, &extraction, &catalogue, profile.as_ref(),
   &TemplateFacts { locus_name, path: the template's file name, sha256 })`:
   - `Ungrounded(u)`: set `interpretation.ungrounded`; error
     `ungrounded_extraction`, `the model reported <field> "<value>", which
     the text does not contain`, status 8;
   - `NeedsClarification { questions }`: `state: "needs_clarification"`,
     `questions` as views, `error` null, status 3;
   - `Proposed { request, origins, assumptions }`: `echo =
     request_echo(&request)`. If `parse_request` of
     `serde_json::to_string(&echo)` is not `Ok(request)`: error
     `internal_request_round_trip`, `the proposed request does not read
     back as itself; this is a bug in luria`, status 1. Otherwise `state:
     "proposed"`, `request: echo`, `origins` with `source` `text` or
     `profile`, `assumptions`, status 0.
8. **Writing**, only for a proposal and only when `write_request` is given:
   `OpenOptions::new().write(true).create_new(true)`, then the text
   `serde_json::to_string_pretty(&echo)` plus `"\n"`. A failure keeps the
   document as it is (still `proposed`), sets status 1 and `note: could not
   write <name>: <io>`.

`note` is `None` in every other case.

## 4. Tests (planner-written; add verbatim)

New file `crates/cli/tests/interpret_run.rs`:

```rust
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
    let dir: PathBuf = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("interpret-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a workspace");
    fs::copy(format!("{}synthetic-linear.gb", GENBANK), dir.join("synthetic-linear.gb"))
        .expect("the record is copied");
    fs::copy(format!("{}profile/full.json", NL), dir.join("full.json")).expect("the profile is copied");
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
        profile: if profile { Some(dir.join("full.json")) } else { None },
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
        vec![("design", "profile"), ("numbering", "text"), ("codon A42G", "profile")]
    );
    assert_eq!(document.assumptions.len(), 1);
    assert_eq!(document.assumptions[0].code, "template_id_from_locus");
    assert_eq!(document.error, None);
    let template = document.template.as_ref().expect("the template");
    assert_eq!(
        (template.file.as_str(), template.locus_name.as_str(), template.sha256.as_str()),
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
            "schema", "state", "package", "request", "questions", "origins", "assumptions",
            "error", "template", "interpretation"
        ]
    );
}

#[test]
fn each_outcome_has_its_state_and_status() {
    let dir: PathBuf = workspace("states");
    let cases: [(&str, ExitStatus, &str, &str); 4] = [
        ("composition-unstated", ExitStatus::NeedsClarification, "needs_clarification", "-"),
        ("ungrounded-cds", ExitStatus::ModelOutputRefused, "failed", "ungrounded_extraction"),
        ("reply-extra-field", ExitStatus::ModelOutputRefused, "failed", "model_output_invalid"),
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
    let ungrounded: InterpretOutcome =
        interpret(&input(&dir, &text_of("ungrounded-cds"), true, None), &mut backend);
    let found = ungrounded.document.interpretation.ungrounded.as_ref().expect("a string");
    assert_eq!((found.field.as_str(), found.value.as_str()), ("cds", "bla"));
    assert_eq!(
        ungrounded.document.error.as_ref().expect("an error").message,
        "the model reported cds \"bla\", which the text does not contain"
    );
    assert!(ungrounded.document.interpretation.extraction.is_some());
    assert_eq!(ungrounded.document.request, None);

    let mut backend: ScriptedBackend = scripted(vec![reply_of("reply-extra-field")]);
    let invalid: InterpretOutcome =
        interpret(&input(&dir, &text_of("reply-extra-field"), true, None), &mut backend);
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
    let asked: InterpretOutcome =
        interpret(&input(&dir, &text_of("composition-unstated"), true, None), &mut backend);
    let codes: Vec<&str> = asked.document.questions.iter().map(|question| question.code).collect();
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
    let outcome: InterpretOutcome =
        interpret(&input(&dir, &text_of("one-sub-prose"), true, None), &mut backend);
    assert_eq!(outcome.status, ExitStatus::Internal);
    assert_eq!(outcome.document.state, "failed");
    assert_eq!(error_code(&outcome), "backend_unreachable");
    assert_eq!(outcome.document.error.as_ref().expect("an error").message, failure.message);
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
            InterpretInput { template: dir.join("absent.gb"), ..base() },
            ExitStatus::MalformedInput,
            "template_unreadable",
        ),
        (
            "FASTA template",
            InterpretInput { template: dir.join("sid.fa"), ..base() },
            ExitStatus::Unsupported,
            "interpret_fasta_unsupported",
        ),
        (
            "broken record",
            InterpretInput { template: dir.join("broken.gb"), ..base() },
            ExitStatus::MalformedInput,
            "malformed_genbank",
        ),
        (
            "huge template",
            InterpretInput { template: dir.join("huge.gb"), ..base() },
            ExitStatus::MalformedInput,
            "input_too_large",
        ),
        (
            "absent text",
            InterpretInput { text: TextSource::File(dir.join("absent.txt")), ..base() },
            ExitStatus::MalformedInput,
            "text_unreadable",
        ),
        (
            "empty text",
            InterpretInput { text: TextSource::File(dir.join("empty.txt")), ..base() },
            ExitStatus::MalformedInput,
            "text_empty",
        ),
        (
            "long text",
            InterpretInput { text: TextSource::Inline("a".repeat(8 * 1024 + 1)), ..base() },
            ExitStatus::MalformedInput,
            "input_too_large",
        ),
        (
            "absent profile",
            InterpretInput { profile: Some(dir.join("absent.json")), ..base() },
            ExitStatus::MalformedInput,
            "profile_unreadable",
        ),
        (
            "invalid profile",
            InterpretInput { profile: Some(dir.join("bad-profile.json")), ..base() },
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
            InterpretInput { write_request: Some(dir.join("taken.json")), ..base() },
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
        assert!(backend.prompts().is_empty(), "{}: something was sent", case.0);
        assert_eq!(outcome.document.interpretation.messages, None, "{}", case.0);
        let rendered: String = render_interpretation(&outcome.document);
        assert!(!rendered.contains(env!("CARGO_TARGET_TMPDIR")), "{}", case.0);
    }
    let mut backend: ScriptedBackend = scripted(vec![reply_of("one-sub-prose")]);
    let broken: InterpretOutcome =
        interpret(&InterpretInput { template: dir.join("broken.gb"), ..base() }, &mut backend);
    assert_eq!(
        broken.document.error.as_ref().expect("an error").rule,
        Some("no_locus".to_string())
    );
    assert_eq!(fs::read_to_string(dir.join("taken.json")).expect("still there"), "{}");
    // Exactly at the cap, the text reaches the model.
    let mut backend: ScriptedBackend = scripted(vec![reply_of("one-sub-prose")]);
    let _ = interpret(
        &InterpretInput { text: TextSource::Inline("a".repeat(8 * 1024)), ..base() },
        &mut backend,
    );
    assert_eq!(backend.prompts().len(), 1);
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
    assert_eq!(written, serde_json::to_string_pretty(echo).expect("serializes") + "\n");
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
        &input(&dir, &text_of("composition-unstated"), true, Some("proposal.json")),
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
            backend.prompts()[0].user.ends_with(&format!("Request:\n{}", text)),
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
            &interpret(&input(&dir, &text_of("two-separate"), true, None), &mut backend).document,
        )
    };
    let first: String = once();
    assert_eq!(first, once());
    assert!(!first.contains(env!("CARGO_TARGET_TMPDIR")));
}
```

`cargo fmt` may reflow these; do not change content. If a test fails,
report what `interpret` returned. If a planner-written test fails to compile
or fails clippy, stop and report it rather than editing it.

### 4.1 Proving the rules are load-bearing (AGENTS.md)

Disable each once, confirm the named test fails, restore, report both:

| Rule, disabled | Test that must fail |
| --- | --- |
| the request-path check runs before the call (move step 4 after step 5) | `every_local_input_is_checked_before_anything_is_sent` |
| the FASTA check | `every_local_input_is_checked_before_anything_is_sent` |
| `create_new` (use `create(true)`, truncating) | `a_dangling_link_is_a_file_already_there` |
| the text's final line break removed | `a_text_file_loses_one_final_line_break` |
| exit 8 for an ungrounded reply (use 4) | `each_outcome_has_its_state_and_status` |

## 5. Do not

- Read the environment, choose a backend, or parse arguments: that is 6h2.
- Re-implement anything `build`, `catalogue`, `parse_profile`,
  `parse_extraction`, `prompt` or `request_echo` does.
- Write any path but `write_request`, or write it anywhere but through
  `create_new`.
- Put an absolute path, a directory or a key in the document or a note.
- Change `luria-core`, `luria-nl`, `tests/fixtures/`, `docs/`, or any
  existing file other than `lib.rs`, `exit.rs` and the one line in
  `interpret.rs`.

## 6. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 7. Handoff

Report the new and changed files, the exact commands and their results,
the §4.1 results, and anything that did not match this contract. Quote
command output or the file for every claim; say plainly what you did not
check.
