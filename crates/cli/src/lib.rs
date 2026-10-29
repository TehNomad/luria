//! luria's command-line interface: argument parsing, exit statuses and the
//! JSON documents the `luria` executable prints. It holds no scientific
//! logic (D003): every number comes from `luria-core`.

pub mod args;
pub mod exit;
pub mod find;
pub mod interpret;
pub mod json;
pub mod new;
pub mod prepare;
pub mod record_file;
pub mod request;
pub mod result;
pub mod review;
pub mod run;
pub mod sequence_view;
pub mod shorthand;
pub mod version;
pub mod write;

/// The request schema `luria q5 design` reads (plan §3.4).
pub const REQUEST_SCHEMA: &str = "luria.q5-request/1";
/// The design profile schema `luria q5 interpret` reads (milestone 6
/// plan §5.4).
pub const PROFILE_SCHEMA: &str = "luria.q5-design-profile/1";
/// The schema of `luria q5 interpret`'s document (milestone 6 plan §5.7).
pub const INTERPRETATION_SCHEMA: &str = "luria.q5-interpretation/1";
/// The result schema `luria q5 design` writes (plan §3.5).
pub const RESULT_SCHEMA: &str = "luria.q5-result/1";
/// The schema of `luria version`'s document.
pub const VERSION_SCHEMA: &str = "luria.version/1";
