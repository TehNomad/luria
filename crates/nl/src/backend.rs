//! The interface every model backend implements, and the scripted backend
//! tests use (plan §5.6). A backend turns one `Prompt` into one `Reply`,
//! unparsed: reading the reply is `extraction.rs`, and judging what the
//! model said belongs to `luria-cli`.
//!
//! `describe` is what the interpretation document records about a call: the
//! backend's kind, the endpoint it calls if any, and the model. It never
//! holds a key or any other credential, and this crate reads no credential
//! from the environment (D003): a key stays with whatever constructs the
//! backend, and it never comes back through `describe`.
//!
//! Three reasons no reply came back, each with the code the CLI reports:
//! `Unreachable` (no answer at all: the connection failed, timed out or
//! broke), `Status` (a server answered, with a status other than success)
//! and `Response` (an answer that is not a reply in its protocol's shape).
//!
//! `ScriptedBackend` replays canned replies, in order, and records every
//! prompt it is given, so everything above a backend is tested with no
//! model and no network. It is for tests: the command line does not offer
//! it, and no HTTP lives here (that is 6g1).

use crate::prompt::Prompt;
use serde::Serialize;

/// A language model luria can send a prompt to (plan §5.6).
pub trait Backend {
    /// Send the prompt; return the model's reply text, unparsed.
    fn complete(&mut self, prompt: &Prompt) -> Result<Reply, BackendError>;
    /// What this backend is, for the interpretation document. Never a key.
    fn describe(&self) -> BackendInfo;
}

/// The model's reply: the text `parse_extraction` will read, exactly as the
/// model wrote it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Reply {
    pub text: String,
}

/// What a backend is: its kind, the endpoint it calls if any, and the
/// model. Never a key or any other credential.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BackendInfo {
    pub kind: String,
    pub endpoint: Option<String>,
    pub model: String,
}

/// Why no reply came back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendErrorKind {
    /// No answer: the connection failed, timed out, or broke.
    Unreachable,
    /// The server answered with a status other than success.
    Status,
    /// The server answered, but not with a reply in its protocol's shape.
    Response,
}

impl BackendErrorKind {
    /// `backend_unreachable`, `backend_status`, `backend_response`.
    pub fn code(self) -> &'static str {
        match self {
            BackendErrorKind::Unreachable => "backend_unreachable",
            BackendErrorKind::Status => "backend_status",
            BackendErrorKind::Response => "backend_response",
        }
    }
}

/// A failed call. `message` is one line, no trailing newline, and holds no
/// credential.
#[derive(Clone, Debug, PartialEq)]
pub struct BackendError {
    pub kind: BackendErrorKind,
    pub message: String,
}

/// A backend that replays canned replies, in order, and records every
/// prompt it is given. For tests; the command line does not offer it.
#[derive(Clone, Debug)]
pub struct ScriptedBackend {
    model: String,
    replies: Vec<Result<String, BackendError>>,
    given: Vec<Prompt>,
    next: usize,
}

impl ScriptedBackend {
    /// Replies are given back in this order, one per `complete`.
    pub fn new(model: &str, replies: Vec<Result<String, BackendError>>) -> ScriptedBackend {
        ScriptedBackend {
            model: model.to_string(),
            replies,
            given: vec![],
            next: 0,
        }
    }

    /// Every prompt `complete` was given, in order.
    pub fn prompts(&self) -> &[Prompt] {
        &self.given
    }
}

impl Backend for ScriptedBackend {
    /// Records the prompt first, then gives back the next canned reply.
    fn complete(&mut self, prompt: &Prompt) -> Result<Reply, BackendError> {
        self.given.push(prompt.clone());
        match self.replies.get(self.next) {
            Some(reply) => {
                self.next += 1;
                match reply {
                    Ok(text) => Ok(Reply { text: text.clone() }),
                    Err(error) => Err(error.clone()),
                }
            }
            None => Err(BackendError {
                kind: BackendErrorKind::Response,
                message: "the scripted backend has no reply left".to_string(),
            }),
        }
    }

    fn describe(&self) -> BackendInfo {
        BackendInfo {
            kind: "scripted".to_string(),
            endpoint: None,
            model: self.model.clone(),
        }
    }
}
