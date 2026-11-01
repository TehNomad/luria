//! The sequence view (plan §5 of `docs/tasks/milestone-8-view.md`): what is on
//! screen, where each base sits, and the keys that move the cursor. Nothing
//! here writes (D003): the view reads a record and draws it.
//!
//! 8j started this module with `layout`, the pure column arithmetic. 8k1 adds
//! the three parts of the screen: `state` holds the view between keys — which
//! CDS is selected, which translation lanes are drawn, where the cursor is and
//! which blocks are on top — `screen` draws the title, the blocks, the status
//! line and the keys, and `terminal` owns the terminal for one view and reads
//! the keys. 8k3 adds `list`, the feature list and the CDS picker that `f` and
//! `c` open: its rows and highlight live in `state`, which routes the keys to it
//! while it is open, and `screen` draws its rows where the blocks would be. 8k2
//! adds `goto`, which reads the one line typed at the `g` prompt — a base, a
//! residue or a name — while `state` moves the cursor for it. 8l1 adds `search`,
//! the prompt `/` opens with its `DNA`/`AA` badge, the hits `enter` puts on the
//! bases and the `n`/`N` that walk them. 8l2 adds `overview`, the two-line strip
//! that `o` hides and shows: `overview` writes its two lines, `state` says whether
//! they are drawn and measures the blocks against what is left of the body, and
//! `screen` puts them at the top of the frame. 9h adds `save`, the pure name,
//! date and text of a save: `state` asks for the write and `terminal` performs
//! it. The split is the one `review` draws
//! between `app`, `view` and `terminal`; 8h opens the view from `luria FILE` and
//! is the caller `terminal` is written for.

pub mod goto;
pub mod layout;
pub mod list;
pub mod overview;
pub mod save;
pub mod screen;
pub mod search;
pub mod state;
pub mod terminal;
