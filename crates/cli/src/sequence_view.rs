//! The sequence view (plan §5 of `docs/tasks/milestone-8-view.md`): what is on
//! screen, where each base sits, and the keys that move the cursor. Nothing
//! here writes (D003): the view reads a record and draws it.
//!
//! 8j started this module with `layout`, the pure column arithmetic. 8k1 adds
//! the three parts of the screen: `state` holds the view between keys — which
//! CDS is selected, which translation lanes are drawn, where the cursor is and
//! which blocks are on top — `screen` draws the title, the blocks, the status
//! line and the keys, and `terminal` owns the terminal for one view and reads
//! the keys. The split is the one `review` draws between `app`, `view` and
//! `terminal`; 8h opens the view from `luria FILE` and is the caller `terminal`
//! is written for.

pub mod layout;
pub mod screen;
pub mod state;
pub mod terminal;
