//! luria's command-line interface: argument parsing, exit statuses and the
//! JSON documents the `luria` executable prints. It holds no scientific
//! logic (D003): every number comes from `luria-core`.

pub mod args;
pub mod exit;
pub mod json;
pub mod request;
pub mod version;

/// The request schema `luria q5 design` reads (plan §3.4).
pub const REQUEST_SCHEMA: &str = "luria.q5-request/1";
/// The result schema `luria q5 design` writes (plan §3.5).
pub const RESULT_SCHEMA: &str = "luria.q5-result/1";
/// The schema of `luria version`'s document.
pub const VERSION_SCHEMA: &str = "luria.version/1";
