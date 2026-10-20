//! luria's model adapter: the request sent to a language model and the
//! reply read back from it. This crate is the only place a model is
//! spoken to, and it holds no scientific logic and no biology (D003):
//! it stops at what the model said. `luria-cli` grounds that against the
//! user's own text and builds the typed request (milestone 6 plan §3).

pub mod backend;
pub mod extraction;
pub mod prompt;
