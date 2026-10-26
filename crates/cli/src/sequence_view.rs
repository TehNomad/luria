//! The sequence view (plan §5 of `docs/tasks/milestone-8-view.md`): what is on
//! screen, where each base sits, and the keys that move the cursor. Nothing
//! here writes (D003): the view reads a record and draws it.
//!
//! 8j starts this module with `layout`, the pure column arithmetic. The state,
//! cursor and status line come in 8k1, and only they draw.

pub mod layout;
