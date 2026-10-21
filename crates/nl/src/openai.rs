//! The OpenAI-compatible backend: the prompt sent to a model served behind
//! a `/chat/completions` endpoint (milestone 6 plan §5.6, §5.2 and contract
//! 6g1). llama.cpp, Ollama, vLLM and others speak that shape.
//!
//! This is the first code that sends anything off the process, so what is
//! sent is fixed here and nothing else is: the two messages of the prompt,
//! `temperature` 0, the seed, the model named by the caller (never a default
//! one), and the extraction schema as `response_format` so a server that
//! supports it constrains the reply (plan §5.2). There is no thinking
//! switch and no `max_tokens` (M6-17): the server's own settings decide how
//! long it answers. The reply text is returned exactly as the model wrote
//! it, unparsed: reading it is `extraction.rs`.
//!
//! What is not sent: the key, anywhere but in its own header; and the
//! request is never redirected (a redirect would re-send the prompt to
//! another host, which is what a server could use to get it), never routed
//! through a proxy taken from the environment, and never routed over `https://`
//! (M6-18: `ureq`'s default features are off, so this build links no TLS
//! library and cannot speak it at all). Two budgets bound the call: a whole
//! call timeout and a response size, above which the response is refused
//! rather than read.

use crate::backend::{Backend, BackendError, BackendErrorKind, BackendInfo, Reply};
use crate::endpoint::Endpoint;
use crate::prompt::{Prompt, EXTRACTION_SCHEMA};

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;
use ureq::{Agent, Error};

/// The longest a whole call may take, in seconds. A slow local model
/// answers in seconds; this bounds a server that never does.
pub const REQUEST_TIMEOUT_SECONDS: u64 = 600;
/// The largest response read, in bytes. A chat completion around one
/// extraction is far smaller; larger is refused, not read.
pub const MAX_RESPONSE_BYTES: u64 = 1024 * 1024;
/// The seed sent with every request (plan §5.6).
pub const SEED: u64 = 1;

/// A model behind an OpenAI-compatible `/chat/completions` endpoint.
/// Deliberately not `Debug`: it holds the key.
pub struct OpenAiCompatible {
    endpoint: Endpoint,
    model: String,
    key: Option<String>,
    agent: Agent,
}

impl OpenAiCompatible {
    /// `key` is sent as a bearer token when it is `Some` and not empty. The
    /// key never comes back: it is not in `describe`, in a message or in any
    /// log, and this crate reads no credential from the environment.
    pub fn new(endpoint: Endpoint, model: &str, key: Option<String>) -> OpenAiCompatible {
        OpenAiCompatible {
            endpoint: endpoint.clone(),
            model: model.to_string(),
            key: key.clone(),
            agent: agent_of(),
        }
    }
}

/// The one agent every call uses: no TLS (`ureq`'s default features are off,
/// M6-18), no proxy from the environment, no redirect followed (one
/// connection, and a `307` comes back as a status), and one global timeout
/// for the whole call.
fn agent_of() -> Agent {
    Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(REQUEST_TIMEOUT_SECONDS)))
        .http_status_as_error(false)
        .proxy(None)
        .max_redirects(0)
        .build()
        .into()
}

/// The whole request, in the field order the plan gives (§3.4). No other
/// field is ever sent: in particular no thinking switch and no `max_tokens`
/// (M6-17).
#[derive(Serialize)]
struct CompletionRequest {
    model: String,
    temperature: u8,
    seed: u64,
    messages: Vec<Turn>,
    response_format: ReplyFormat,
}

/// One message of the request: the system rule, then the user's turn.
#[derive(Serialize)]
struct Turn {
    role: String,
    content: String,
}

/// The extraction schema sent with the request, so a server that supports
/// `json_schema` constrains the reply (plan §5.2).
#[derive(Serialize)]
struct ReplyFormat {
    #[serde(rename = "type")]
    format: String,
    json_schema: SchemaOfReply,
}

/// The named schema of one extraction.
#[derive(Serialize)]
struct SchemaOfReply {
    name: String,
    strict: bool,
    schema: Value,
}

/// A reply read leniently: a server sends many fields besides these, so
/// unknown ones are allowed, and only `choices` and its message's `content`
/// are taken.
#[derive(Deserialize)]
struct Completion {
    choices: Vec<Choice>,
}

/// One choice of a chat completion.
#[derive(Deserialize)]
struct Choice {
    message: Option<ReplyMessage>,
}

/// The message of one choice: what the model wrote, or none.
#[derive(Deserialize)]
struct ReplyMessage {
    content: Option<String>,
}

impl Backend for OpenAiCompatible {
    /// Send the prompt and return the model's reply text, unparsed.
    fn complete(&mut self, prompt: &Prompt) -> Result<Reply, BackendError> {
        let schema: Value = match serde_json::from_str(EXTRACTION_SCHEMA) {
            Err(_) => {
                return Err(BackendError {
                    kind: BackendErrorKind::Response,
                    message: "the extraction schema is not JSON".to_string(),
                });
            }
            Ok(found) => found,
        };
        let request: CompletionRequest = CompletionRequest {
            model: self.model.clone(),
            temperature: 0,
            seed: SEED,
            messages: vec![
                Turn {
                    role: "system".to_string(),
                    content: prompt.system.clone(),
                },
                Turn {
                    role: "user".to_string(),
                    content: prompt.user.clone(),
                },
            ],
            response_format: ReplyFormat {
                format: "json_schema".to_string(),
                json_schema: SchemaOfReply {
                    name: "extraction".to_string(),
                    strict: true,
                    schema: schema.clone(),
                },
            },
        };
        let body: String = match serde_json::to_string(&request) {
            Err(_) => {
                return Err(BackendError {
                    kind: BackendErrorKind::Response,
                    message: "the request could not be written".to_string(),
                });
            }
            Ok(written) => written,
        };
        // An empty key is no key: no header is sent, and nothing is invented.
        let key_text: Option<String> = self.key.clone().filter(|held| !held.is_empty());
        let url: String = self.endpoint.join("/chat/completions");
        // The prompt is the only thing that leaves the process, so it goes to
        // one address only: the endpoint's, once, with no proxy.
        let answer = match key_text {
            Some(held) => self
                .agent
                .post(url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", held))
                .send(body),
            None => self
                .agent
                .post(url)
                .header("Content-Type", "application/json")
                .send(body),
        };
        match answer {
            Err(error) => Err(BackendError {
                kind: BackendErrorKind::Unreachable,
                message: format!("could not reach {}: {}", self.endpoint.url(), error),
            }),
            Ok(mut answer) => {
                let status: u16 = answer.status().as_u16();
                match answer
                    .body_mut()
                    .with_config()
                    .limit(MAX_RESPONSE_BYTES)
                    .read_to_vec()
                {
                    Err(error) => match error {
                        Error::BodyExceedsLimit(_) => Err(BackendError {
                            kind: BackendErrorKind::Response,
                            message: format!(
                                "the response is larger than {} bytes",
                                MAX_RESPONSE_BYTES,
                            ),
                        }),
                        _ => Err(BackendError {
                            kind: BackendErrorKind::Unreachable,
                            message: format!("the response broke off: {}", error),
                        }),
                    },
                    // The bytes are read first and turned into text second: a
                    // server that answered, but with a body that is not UTF-8,
                    // gave a bad answer, which is `Response`, not the network
                    // breaking off (review A-4).
                    Ok(bytes) => match String::from_utf8(bytes) {
                        Err(_) => Err(BackendError {
                            kind: BackendErrorKind::Response,
                            message: "the response is not UTF-8".to_string(),
                        }),
                        Ok(text) => match status {
                            200..=299 => completion_of(text),
                            _ => Err(status_error(status, text)),
                        },
                    },
                }
            }
        }
    }

    /// What this backend is: the endpoint it calls and the model. Never the
    /// key.
    fn describe(&self) -> BackendInfo {
        BackendInfo {
            kind: "openai-compatible".to_string(),
            endpoint: Some(self.endpoint.url()),
            model: self.model.clone(),
        }
    }
}

/// A server's own words about a call it answered badly: the status, and the
/// first 200 characters of its body, control characters turned into spaces so
/// one reason stays one line (§3.5, rule 5).
fn status_error(status: u16, body: String) -> BackendError {
    match body.is_empty() {
        true => BackendError {
            kind: BackendErrorKind::Status,
            message: format!("the server answered HTTP {}", status),
        },
        false => BackendError {
            kind: BackendErrorKind::Status,
            message: format!("the server answered HTTP {}: {}", status, excerpt(body),),
        },
    }
}

/// The first 200 characters of a server's body, with every control character
/// turned into a space: one reason stays one line.
fn excerpt(body: String) -> String {
    body.chars()
        .take(200)
        .map(|ch| match ch.is_control() {
            true => ' ',
            false => ch,
        })
        .collect()
}

/// The reply text, exactly as the model wrote it (§3.5, rules 6 and 7).
fn completion_of(text: String) -> Result<Reply, BackendError> {
    match serde_json::from_str::<Completion>(&text) {
        Err(error) => Err(BackendError {
            kind: BackendErrorKind::Response,
            message: format!("the response is not a chat completion: {}", error),
        }),
        Ok(answer) => match answer.choices.first() {
            None => Err(no_choice()),
            Some(choice) => match choice.message.as_ref() {
                None => Err(no_content()),
                Some(message) => match message.content.as_ref() {
                    None => Err(no_content()),
                    Some(content) => {
                        if content.trim().is_empty() {
                            Err(no_content())
                        } else {
                            Ok(Reply {
                                text: content.to_string(),
                            })
                        }
                    }
                },
            },
        },
    }
}

/// Why the answer held no choice to read.
fn no_choice() -> BackendError {
    BackendError {
        kind: BackendErrorKind::Response,
        message: "the response has no choices".to_string(),
    }
}

/// Why the choice held no content: a model that said nothing is not a reply.
fn no_content() -> BackendError {
    BackendError {
        kind: BackendErrorKind::Response,
        message: "the response's message has no content".to_string(),
    }
}
