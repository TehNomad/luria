//! `luria q5 interpret` as a library call (contract 6h1): the input files and
//! a `Backend`, and the one document the command prints, its exit status and
//! an optional line for stderr. Because the backend is a parameter, every
//! path here is tested with the scripted backend (6f), and 6h2 only wires
//! arguments and a real backend to it.
//!
//! Two rules shape the order of the steps:
//!
//! - **Nothing is sent until every local input is valid.** A missing file, a
//!   FASTA template, a bad profile or an unusable `--write-request` path is
//!   refused before the model is called, so a mistake never costs a model call
//!   and never sends anything.
//! - **The document is the answer, in every state**, with a fixed shape, as
//!   `luria q5 design`'s is.
//!
//! Nothing is decided here about biology (D003): `build` (6e2), `catalogue`
//! (6c), `parse_profile` (6d), `parse_extraction` and `prompt` (6f) decide
//! what the text, the record and the model mean, and are called, never
//! re-implemented. The environment, the command line and which backend to
//! build are 6h2's.

use crate::exit::ExitStatus;
use crate::interpret::build::{
    build, Assumption, Built, Origin, Question, Source, TemplateFacts, Ungrounded,
};
use crate::interpret::catalogue::{catalogue, CatalogueEntry, Usability};
use crate::interpret::document::{
    AssumptionView, InterpretErrorView, InterpretationDocument, InterpretationView, OriginView,
    QuestionView, TemplateFileView, UngroundedView,
};
use crate::request::{parse_profile, parse_request, ProfileDocument};
use crate::result::{request_echo, rule_code, PackageView, RequestEcho};
use crate::INTERPRETATION_SCHEMA;
use luria_core::error::DomainError;
use luria_core::genbank::{read_genbank, GenBankRecord};
use luria_core::identity::Identifier;
use luria_core::strand::Strand;
use luria_nl::backend::{Backend, BackendInfo};
use luria_nl::extraction::{parse_extraction, Extraction};
use luria_nl::prompt::{prompt, CdsLine, CdsState, Prompt, PROMPT_VERSION};
use serde_json::{to_string, to_string_pretty};
use std::fs::{canonicalize, symlink_metadata, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

/// The most bytes read of each input (plan §5.7, review item B-8).
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

/// Why a capped read failed: the operating system's reason, whose `Display`
/// names no path (D012), or the file's size, which went past the cap.
#[derive(Clone, Debug, PartialEq)]
enum ReadFailure {
    Io(String),
    Over,
}

/// The document of a run that has read nothing yet: the backend and the
/// prompt version are always known, everything else is null or empty, and a
/// refusal keeps the state `failed`.
fn base(info: &BackendInfo) -> InterpretationDocument {
    InterpretationDocument {
        schema: INTERPRETATION_SCHEMA,
        state: "failed",
        package: PackageView {
            name: "luria-cli",
            version: env!("CARGO_PKG_VERSION"),
        },
        request: None,
        questions: [].to_vec(),
        origins: [].to_vec(),
        assumptions: [].to_vec(),
        error: None,
        template: None,
        interpretation: InterpretationView {
            backend: info.clone(),
            prompt_version: PROMPT_VERSION,
            messages: None,
            reply: None,
            extraction: None,
            ungrounded: None,
        },
    }
}

/// The run's answer: this document, this status, and nothing for stderr.
fn outcome(document: &InterpretationDocument, status: ExitStatus) -> InterpretOutcome {
    InterpretOutcome {
        document: document.clone(),
        status,
        note: None,
    }
}

/// A refusal: the document as it stands, this error, this status, and no
/// note. The state is already `failed`, and a step only fills a field once it
/// is known, so everything the run has not reached stays null or empty.
fn refused(
    document: &InterpretationDocument,
    code: String,
    rule: Option<String>,
    message: String,
    status: ExitStatus,
) -> InterpretOutcome {
    let error: InterpretErrorView = InterpretErrorView {
        code: code.clone(),
        rule: rule.clone(),
        message: message.clone(),
    };
    let mut answered: InterpretationDocument = document.clone();
    answered.error = Some(error.clone());
    InterpretOutcome {
        document: answered.clone(),
        status,
        note: None,
    }
}

/// A refusal core gave: the error's own code, the rule it has when it has one,
/// its message, and the status of its category (§3.6 step 3).
fn core_refusal(document: &InterpretationDocument, problem: &DomainError) -> InterpretOutcome {
    let rule: Option<String> = rule_code(problem).map(|found| found.to_string());
    refused(
        document,
        problem.code().to_string(),
        rule,
        problem.message(),
        ExitStatus::for_failure(problem.category()),
    )
}

/// The file name a path names, as UTF-8, or `None` when the name is not UTF-8
/// or the path names no file at all. Nothing here can hold a directory (D012).
fn name_of(path: &Path) -> Option<String> {
    match path.file_name() {
        None => None,
        Some(found) => found.to_str().map(|name| name.to_string()),
    }
}

/// The directory a path sits in (plan §5.7): its `parent()`, or `.` when that
/// names nothing or is empty.
fn directory_of(path: &Path) -> &Path {
    match path.parent() {
        None => Path::new("."),
        Some(found) => match found.is_empty() {
            true => Path::new("."),
            false => found,
        },
    }
}

/// The bytes of a file, reading at most `cap + 1` of them: one byte past the
/// cap is all it takes to tell "at the cap" from "over it", and a file over
/// the cap is refused before any of it is kept.
fn read_capped(path: &Path, cap: usize) -> Result<Vec<u8>, ReadFailure> {
    match OpenOptions::new().read(true).open(path) {
        Err(problem) => Err(ReadFailure::Io(problem.to_string())),
        Ok(handle) => {
            let mut buffer: Vec<u8> = [].to_vec();
            match handle.take(cap as u64 + 1).read_to_end(&mut buffer) {
                Err(problem) => Err(ReadFailure::Io(problem.to_string())),
                Ok(_) => match buffer.len() > cap {
                    true => Err(ReadFailure::Over),
                    false => Ok(buffer),
                },
            }
        }
    }
}

/// The user's text as the run reads it: one trailing `\n` removed, then one
/// trailing `\r`, so a file's final break, in either convention, is not part
/// of the request.
fn without_break(value: &str) -> String {
    let without_newline: String = match value.strip_suffix("\n") {
        None => value.to_string(),
        Some(found) => found.to_string(),
    };
    match without_newline.strip_suffix("\r") {
        None => without_newline.clone(),
        Some(found) => found.to_string(),
    }
}

/// Whether the template is FASTA: the first character that is not whitespace
/// is `>` (M6-7).
fn starts_fasta(contents: &str) -> bool {
    match contents.chars().find(|ch| !ch.is_whitespace()) {
        None => false,
        Some(ch) => ch == '>',
    }
}

/// One catalogue entry as the model is shown it (6f): whether luria can use
/// the CDS, and its strand as a yes-or-no about the plus strand.
fn line_of(entry: &CatalogueEntry) -> CdsLine {
    CdsLine {
        id: entry.id.clone(),
        gene: entry.gene.clone(),
        product: entry.product.clone(),
        state: match entry.usability {
            Usability::Usable {
                residues,
                ref strand,
            } => CdsState::Usable {
                residues,
                plus_strand: *strand == Strand::Plus,
            },
            Usability::Refused { .. } => CdsState::NotUsable,
        },
    }
}

/// One question as the document writes it: `build`'s code, rendered by the
/// code's own `code`.
fn question_view(question: &Question) -> QuestionView {
    QuestionView {
        code: question.code.code(),
        subject: question.subject.clone(),
        message: question.message.clone(),
    }
}

/// One origin as the document writes it: `text` or `profile`.
fn origin_view(origin: &Origin) -> OriginView {
    OriginView {
        value: origin.value.clone(),
        source: match origin.source {
            Source::Text => "text",
            Source::Profile => "profile",
        },
    }
}

/// One assumption as the document writes it.
fn assumption_view(assumption: &Assumption) -> AssumptionView {
    AssumptionView {
        code: assumption.code.clone(),
        message: assumption.message.clone(),
    }
}

/// One string the model reported that the text does not hold, as the
/// document writes it.
fn ungrounded_view(problem: &Ungrounded) -> UngroundedView {
    UngroundedView {
        field: problem.field.clone(),
        value: problem.value.clone(),
    }
}

/// Why the request cannot be written at `path` for a template at
/// `template`, as (code, message), or `None` when it can (plan §5.7, C-1).
/// Decided from the file system without writing anything.
pub fn request_location_problem(template: &Path, path: &Path) -> Option<(String, String)> {
    let found_request: Option<PathBuf> = canonicalize(directory_of(path)).ok();
    let found_template: Option<PathBuf> = canonicalize(directory_of(template)).ok();
    let same: bool = match found_request {
        None => false,
        Some(one) => match found_template {
            None => false,
            Some(other) => one == other,
        },
    };
    let request_name: Option<String> = name_of(path);
    // The piece the path ends with must be the name itself: `new.json/`
    // and `new.json/.` name no file, and step 8's write would fail only
    // after the model had been sent (6h1: nothing is sent until every
    // local input is valid).
    let names_a_file: bool = match path.to_str() {
        None => false,
        Some(text) => match text.rsplit(std::path::is_separator).next() {
            None => false,
            Some(piece) => match request_name.as_ref() {
                None => false,
                Some(found) => piece == found,
            },
        },
    };
    if !same || !names_a_file {
        return Some((
            "request_location_unusable".to_string(),
            "the request must be written in the template's directory, because a request names its template relative to itself".to_string(),
        ));
    }
    match symlink_metadata(path) {
        Ok(_found) => Some((
            "output_exists".to_string(),
            format!(
                "the file {} already exists; luria never replaces a file",
                request_name.expect("a file name"),
            ),
        )),
        Err(problem) => match problem.kind() {
            // Only a name nothing holds is free, as `write.rs`'s check
            // is: any other failure is the operating system's own, and
            // the write below would not have told it apart.
            ErrorKind::NotFound => None,
            _ => Some((
                "request_location_unusable".to_string(),
                format!(
                    "cannot check whether {} already exists: {}",
                    request_name.expect("a file name"),
                    problem,
                ),
            )),
        },
    }
}

/// The request as `--write-request` writes it: `to_string_pretty` of the
/// echo and one `\n`.
pub fn request_text(echo: &RequestEcho) -> String {
    match to_string_pretty(echo) {
        Err(problem) => panic!("the proposed request always serializes: {}", problem),
        Ok(found) => found + "\n",
    }
}

/// Write `text` at `path` as a new file, never replacing one, atomically with
/// creating it: `create_new` refuses a path something already holds, a symlink
/// a dangling one included, so a file that appeared after the check above is
/// still never replaced (C-1). The `String` is the operating system's reason.
pub fn write_request_file(path: &Path, text: &str) -> Result<(), String> {
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Err(problem) => Err(problem.to_string()),
        Ok(mut handle) => match handle.write_all(text.as_bytes()) {
            Err(problem) => Err(problem.to_string()),
            Ok(_) => Ok(()),
        },
    }
}

/// Step 1: the user's text, however it was given, and its cap. The `Err` is the
/// refusal this run makes, with the document as it stands, so nothing after
/// step 1 runs.
#[allow(clippy::result_large_err)]
fn read_text(
    input: &InterpretInput,
    answered: &InterpretationDocument,
) -> Result<String, InterpretOutcome> {
    let given: String = match &input.text {
        TextSource::Inline(value) => {
            if value.len() > MAX_TEXT_BYTES {
                return Err(refused(
                    answered,
                    "input_too_large".to_string(),
                    None,
                    format!(
                        "the text is more than {} bytes, the most luria reads",
                        MAX_TEXT_BYTES
                    ),
                    ExitStatus::MalformedInput,
                ));
            }
            value.clone()
        }
        TextSource::File(written) => match read_capped(written, MAX_TEXT_BYTES) {
            Err(ReadFailure::Io(reason)) => {
                return Err(refused(
                    answered,
                    "text_unreadable".to_string(),
                    None,
                    format!("cannot read the text file: {}", reason),
                    ExitStatus::MalformedInput,
                ));
            }
            Err(ReadFailure::Over) => {
                return Err(refused(
                    answered,
                    "input_too_large".to_string(),
                    None,
                    format!(
                        "the text is more than {} bytes, the most luria reads",
                        MAX_TEXT_BYTES
                    ),
                    ExitStatus::MalformedInput,
                ));
            }
            Ok(found) => match String::from_utf8(found) {
                Err(_problem) => {
                    return Err(refused(
                        answered,
                        "text_unreadable".to_string(),
                        None,
                        "the text file is not UTF-8".to_string(),
                        ExitStatus::MalformedInput,
                    ));
                }
                Ok(contents) => contents.clone(),
            },
        },
    };
    let text: String = without_break(&given);
    if text.trim().is_empty() {
        return Err(refused(
            answered,
            "text_empty".to_string(),
            None,
            "the text is empty".to_string(),
            ExitStatus::MalformedInput,
        ));
    }
    Ok(text)
}

/// Step 2: the design profile, when one was given. `Ok(None)` is no profile;
/// the `Err` is the refusal this run makes, as in step 1.
#[allow(clippy::result_large_err)]
fn read_profile(
    input: &InterpretInput,
    answered: &InterpretationDocument,
) -> Result<Option<ProfileDocument>, InterpretOutcome> {
    match &input.profile {
        None => Ok(None),
        Some(written) => match read_capped(written, MAX_PROFILE_BYTES) {
            Err(ReadFailure::Io(reason)) => Err(refused(
                answered,
                "profile_unreadable".to_string(),
                None,
                format!("cannot read the design profile: {}", reason),
                ExitStatus::MalformedInput,
            )),
            Err(ReadFailure::Over) => Err(refused(
                answered,
                "input_too_large".to_string(),
                None,
                format!(
                    "the design profile is more than {} bytes, the most luria reads",
                    MAX_PROFILE_BYTES
                ),
                ExitStatus::MalformedInput,
            )),
            Ok(found) => match String::from_utf8(found) {
                Err(_problem) => Err(refused(
                    answered,
                    "profile_unreadable".to_string(),
                    None,
                    "the design profile is not UTF-8".to_string(),
                    ExitStatus::MalformedInput,
                )),
                Ok(contents) => match parse_profile(&contents) {
                    Err(problem) => Err(refused(
                        answered,
                        "profile_invalid".to_string(),
                        None,
                        format!("{}: {}", problem.path, problem.message),
                        ExitStatus::MalformedInput,
                    )),
                    Ok(found_profile) => Ok(Some(found_profile.clone())),
                },
            },
        },
    }
}

/// `luria q5 interpret`, in the order of §3.6: every local input is checked
/// first, the model is spoken to once, its reply is read, the request is
/// built, and only a proposed request is written.
pub fn interpret(input: &InterpretInput, backend: &mut dyn Backend) -> InterpretOutcome {
    let info: BackendInfo = backend.describe();
    let answered: InterpretationDocument = base(&info);
    let text: String = match read_text(input, &answered) {
        Err(outcome) => return outcome,
        Ok(found) => found,
    };
    let profile: Option<ProfileDocument> = match read_profile(input, &answered) {
        Err(outcome) => return outcome,
        Ok(found) => found,
    };
    run(input, &text, profile, answered, backend)
}

/// `luria q5 interpret` with the design profile given as a value, as the `luria
/// new` form passes the values it shows (M7-41): steps 1 and 3 to 8 exactly as
/// `interpret` runs them, with step 2 skipped, so no profile file is read. The
/// profile in the input and one given here are the same value read twice.
pub fn interpret_with_profile(
    input: &InterpretInput,
    profile: ProfileDocument,
    backend: &mut dyn Backend,
) -> InterpretOutcome {
    assert!(input.profile.is_none(), "a profile is given twice");
    let info: BackendInfo = backend.describe();
    let answered: InterpretationDocument = base(&info);
    let text: String = match read_text(input, &answered) {
        Err(outcome) => return outcome,
        Ok(found) => found,
    };
    run(input, &text, Some(profile), answered, backend)
}

/// Steps 3 to 8, from step 1's text and step 2's profile: the body both public
/// functions run, so the two differ only in where the profile came from.
fn run(
    input: &InterpretInput,
    text: &str,
    profile: Option<ProfileDocument>,
    mut answered: InterpretationDocument,
    backend: &mut dyn Backend,
) -> InterpretOutcome {
    // Step 3: the template: its name alone, its bytes, its FASTA refusal, then
    // the record and its CDS catalogue.
    let file: String = match input.template.file_name() {
        None => {
            return refused(
                &answered,
                "template_unreadable".to_string(),
                None,
                "the template path names no file".to_string(),
                ExitStatus::MalformedInput,
            );
        }
        Some(found) => match found.to_str() {
            None => {
                return refused(
                    &answered,
                    "template_unreadable".to_string(),
                    None,
                    "the template file's name is not UTF-8".to_string(),
                    ExitStatus::MalformedInput,
                );
            }
            Some(kept) => kept.to_string(),
        },
    };
    let contents: String = match read_capped(&input.template, MAX_TEMPLATE_BYTES) {
        Err(ReadFailure::Io(reason)) => {
            return refused(
                &answered,
                "template_unreadable".to_string(),
                None,
                format!("cannot read the template file: {}", reason),
                ExitStatus::MalformedInput,
            );
        }
        Err(ReadFailure::Over) => {
            return refused(
                &answered,
                "input_too_large".to_string(),
                None,
                format!(
                    "the template is more than {} bytes, the most luria reads",
                    MAX_TEMPLATE_BYTES
                ),
                ExitStatus::MalformedInput,
            );
        }
        Ok(found) => match String::from_utf8(found) {
            Err(_problem) => {
                return refused(
                    &answered,
                    "template_unreadable".to_string(),
                    None,
                    "the template file is not UTF-8".to_string(),
                    ExitStatus::MalformedInput,
                );
            }
            Ok(body) => body.clone(),
        },
    };
    if starts_fasta(&contents) {
        return refused(
            &answered,
            "interpret_fasta_unsupported".to_string(),
            None,
            "the template is FASTA; luria q5 interpret reads GenBank only, so that CDSs come from the record (M6-7). Write the request by hand for a FASTA template".to_string(),
            ExitStatus::Unsupported,
        );
    }
    let name: Identifier = Identifier::create("template".to_string())
        .expect("the identifier `template` is a valid identifier");
    let record: GenBankRecord = match read_genbank(name, contents) {
        Err(problem) => {
            return core_refusal(&answered, &problem);
        }
        Ok(found) => found.clone(),
    };
    let listing: Vec<CatalogueEntry> = match catalogue(&record) {
        Err(problem) => {
            return core_refusal(&answered, &problem);
        }
        Ok(found) => found.clone(),
    };
    let facts: TemplateFacts = TemplateFacts {
        locus_name: record.locus_name(),
        path: file.clone(),
        sha256: record.sequence().content_hash().hex_digest().to_string(),
    };
    answered.template = Some(TemplateFileView {
        file: file.clone(),
        locus_name: facts.locus_name.clone(),
        sha256: facts.sha256.clone(),
    });

    // Step 4: where the request would be written, checked before anything is
    // sent: it sits beside the template, and its name is free.
    let request: Option<PathBuf> = match &input.write_request {
        None => None,
        Some(written) => match request_location_problem(&input.template, written) {
            Some((code, message)) => {
                return refused(&answered, code, None, message, ExitStatus::MalformedInput);
            }
            None => Some(written.clone()),
        },
    };

    // Step 5: the prompt the model is given, and the one call. `messages` is
    // set before the call, so a refusal still shows what would have been sent.
    let lines: Vec<CdsLine> = listing.iter().map(line_of).collect();
    let sent: Prompt = prompt(text, &lines);
    answered.interpretation.messages = Some(sent.clone());
    let reply: String = match backend.complete(&sent) {
        Err(problem) => {
            return refused(
                &answered,
                problem.kind.code().to_string(),
                None,
                problem.message.clone(),
                ExitStatus::Internal,
            );
        }
        Ok(answer) => {
            answered.interpretation.reply = Some(answer.text.clone());
            answer.text.clone()
        }
    };

    // Step 6: the reply as an extraction.
    let extraction: Extraction = match parse_extraction(&reply) {
        Err(problem) => {
            return refused(
                &answered,
                "model_output_invalid".to_string(),
                None,
                format!(
                    "the model's reply is not an extraction: {}",
                    problem.message
                ),
                ExitStatus::ModelOutputRefused,
            );
        }
        Ok(found) => {
            answered.interpretation.extraction = Some(found.clone());
            found.clone()
        }
    };

    // Step 7: what luria makes of what the model said, which it does not
    // believe on its own (D003).
    match build(text, &extraction, &listing, profile.as_ref(), &facts) {
        Built::Ungrounded(problem) => {
            answered.interpretation.ungrounded = Some(ungrounded_view(&problem));
            refused(
                &answered,
                "ungrounded_extraction".to_string(),
                None,
                format!(
                    "the model reported {} \"{}\", which the text does not contain",
                    problem.field, problem.value,
                ),
                ExitStatus::ModelOutputRefused,
            )
        }
        Built::NeedsClarification { questions } => {
            answered.state = "needs_clarification";
            answered.questions = questions.iter().map(question_view).collect();
            outcome(&answered, ExitStatus::NeedsClarification)
        }
        Built::Proposed {
            request: proposal,
            origins,
            assumptions,
        } => {
            let echo: RequestEcho = request_echo(&proposal);
            let echoed: String = match to_string(&echo) {
                Err(problem) => panic!("the proposed request always serializes: {}", problem),
                Ok(found) => found.clone(),
            };
            if parse_request(&echoed).ok() != Some(proposal) {
                return refused(
                    &answered,
                    "internal_request_round_trip".to_string(),
                    None,
                    "the proposed request does not read back as itself; this is a bug in luria"
                        .to_string(),
                    ExitStatus::Internal,
                );
            }
            answered.state = "proposed";
            answered.request = Some(echo.clone());
            answered.origins = origins.iter().map(origin_view).collect();
            answered.assumptions = assumptions.iter().map(assumption_view).collect();

            // Step 8: the request, beside the template, as a new file only. A
            // failure keeps the document as it is and leaves one line for
            // stderr: the document is still the run's answer.
            let pretty: String = request_text(&echo);
            match request {
                None => outcome(&answered, ExitStatus::Success),
                Some(written) => match write_request_file(&written, &pretty) {
                    Err(reason) => InterpretOutcome {
                        document: answered.clone(),
                        status: ExitStatus::Internal,
                        note: Some(format!(
                            "could not write {}: {}",
                            name_of(&written).expect("a file name"),
                            reason,
                        )),
                    },
                    Ok(_) => outcome(&answered, ExitStatus::Success),
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // `create_new` is what keeps a file that appears after step 4's check
    // from being replaced (§3.6 step 8). The race cannot be staged, so the
    // write is checked on its own.
    #[test]
    fn a_write_never_replaces_a_file() {
        let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/tmp")
            .join(format!("luria-write-new-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a directory");
        let path: PathBuf = dir.join("request.json");
        assert_eq!(write_request_file(&path, "first"), Ok(()));
        assert!(write_request_file(&path, "second").is_err());
        assert_eq!(std::fs::read_to_string(&path).expect("readable"), "first");
        #[cfg(unix)]
        {
            let link: PathBuf = dir.join("link.json");
            std::os::unix::fs::symlink(dir.join("nowhere.json"), &link).expect("a link");
            assert!(write_request_file(&link, "x").is_err());
            assert!(!dir.join("nowhere.json").exists());
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
