//! The document `luria q5 interpret` prints (milestone 6 plan §5.7, contract
//! 6h1): one `luria.q5-interpretation/1` document, in every state, with a
//! fixed shape and no field ever skipped: an absent value is `null` or `[]`.
//!
//! The interpretation record - what the model was asked, what it said and how
//! the reply read - is kept in `interpretation`, apart from the request
//! (D021): a proposal's request and the model's words about the user's text
//! are two different answers, and the document says which is which.
//!
//! Nothing is decided here and nothing is read: the types hold what `run.rs`
//! found, and `build` (6e2), `catalogue` (6c) and `luria-nl` decide the
//! biology and the model's words. No name here can hold a directory: a
//! template is named by its file name alone (D012), so the same reply gives
//! the same bytes wherever the files sit.

use crate::result::PackageView;
use crate::result::RequestEcho;
use luria_nl::backend::BackendInfo;
use luria_nl::extraction::Extraction;
use luria_nl::prompt::Prompt;

use serde::Serialize;
use serde_json::to_string_pretty;

/// The document one `luria q5 interpret` run answers with: always this
/// schema, always these keys, in this order.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
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

/// One question luria is asking: its stable code, what it is about, and one
/// line for a person.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionView {
    pub code: &'static str,
    pub subject: String,
    pub message: String,
}

/// One value of a proposed request and where it came from. `source` is
/// `text` or `profile`.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginView {
    pub value: String,
    pub source: &'static str,
}

/// Something luria took as given in a proposal, for the user to check.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssumptionView {
    pub code: String,
    pub message: String,
}

/// Why a run failed: its code, the rule that refused it when one has a name,
/// and one line. `rule` is null unless the refusal has a rule of its own.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InterpretErrorView {
    pub code: String,
    pub rule: Option<String>,
    pub message: String,
}

/// The template the run read: its file name alone, never a directory
/// (D012), the record's LOCUS name and the digest of its sequence.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateFileView {
    pub file: String,
    pub locus_name: String,
    pub sha256: String,
}

/// The first string the model's reply holds that the user's text does not:
/// its place in the extraction and the string itself.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UngroundedView {
    pub field: String,
    pub value: String,
}

/// What the model was asked and said: kept apart from the request (D021).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
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
pub fn render_interpretation(document: &InterpretationDocument) -> String {
    match to_string_pretty(document) {
        Ok(text) => text + "\n",
        Err(problem) => panic!("the interpretation document always serializes: {}", problem),
    }
}
