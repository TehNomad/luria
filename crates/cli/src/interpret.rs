//! `luria q5 interpret` (milestone 6): from what a model says the user
//! asked for, to a typed request luria can run. Everything here is
//! deterministic and holds no model I/O; the model is spoken to only in
//! `luria-nl` (milestone 6 plan §3, §5.1).

pub mod build;
pub mod catalogue;
pub mod evidence;
pub mod notation;
