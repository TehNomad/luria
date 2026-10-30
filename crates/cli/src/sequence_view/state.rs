//! The view's state: which CDS is selected, which translation lanes are drawn,
//! where the cursor is, and which blocks are on screen
//! (`docs/tasks/milestone-8-view.md` §5.1, §5.2; contract 8k1; D048 M8-11,
//! D049 M8-3, D052, D055).
//!
//! Pure: `new` takes a record and a size, `handle` takes a key and says whether
//! the session goes on, and nothing here reads a terminal or writes a file
//! (D003). `super::terminal` owns the I/O and `super::screen` owns the drawing,
//! exactly as `review::app`, `review::terminal` and `review::view` divide the
//! review screen.
//!
//! The columns come from 8j's `layout`: `layout` for the current width and mode
//! gives the blocks, `block_lines` gives one block's lines, and a base `p` of
//! block `[s, e)` stands at line column `gutter + 1 + (p - s)`
//! (`tests/fixtures/view/README.md`, "Block layout"). The state holds the index
//! of the block drawn on top and moves it by §4 rule 6; it works out no column
//! of its own, and the only arithmetic it does with bases is which block a base
//! lies in and how far a key moves the cursor.

use crate::review::view::MIN_HEIGHT;
use crate::review::view::MIN_WIDTH;
use crate::sequence_view::goto::parse_target;
use crate::sequence_view::goto::Target;
use crate::sequence_view::layout::block_line_count_with;
use crate::sequence_view::layout::block_lines_with;
use crate::sequence_view::layout::layout;
use crate::sequence_view::layout::Drawing;
use crate::sequence_view::layout::FeatureLevel;
use crate::sequence_view::layout::LaneMode;
use crate::sequence_view::layout::Layout;
use crate::sequence_view::layout::LayoutProblem;
use crate::sequence_view::layout::LineRole;
use crate::sequence_view::list::feature_names;
use crate::sequence_view::list::refusal_code;
use crate::sequence_view::list::Drawn;
use crate::sequence_view::list::List;
use crate::sequence_view::list::ListKind;
use crate::sequence_view::overview::gc_line;
use crate::sequence_view::search::search_record;
use crate::sequence_view::search::Hit;
use crate::sequence_view::search::Hits;
use crate::sequence_view::search::Outcome;
use crate::sequence_view::search::SearchMode;
use crate::sequence_view::search::SearchPrompt;
use luria_core::cds::Cds;
use luria_core::cds_map::CdsMap;
use luria_core::cds_map::CodonPlace;
use luria_core::display_location::DisplayLocation;
use luria_core::edit_buffer::buffer_for;
use luria_core::edit_buffer::Edit;
use luria_core::edit_buffer::EditBlocked;
use luria_core::edit_buffer::EditBuffer;
use luria_core::edit_buffer::EditFeature;
use luria_core::edit_buffer::EditOutcome;
use luria_core::edit_buffer::Undo;
use luria_core::error::DomainError;
use luria_core::name_match::cds_span;
use luria_core::name_match::match_name;
use luria_core::name_match::Matched;
use luria_core::name_match::NamedItem;
use luria_core::orf::find_orfs;
use luria_core::orf::Orf;
use luria_core::orf::ORF_MIN_RESIDUES;
use luria_core::sequence::complement_letter;
use luria_core::sequence::is_dna_letter;
use luria_core::sequence::SequenceTopology;
use luria_core::sequence::DNA_ALPHABET;
use luria_core::strand::Strand;
use luria_core::translation::translate;
use luria_core::view_record::rebuild;
use luria_core::view_record::CdsStatus;
use luria_core::view_record::FeatureShape;
use luria_core::view_record::TopologySource;
use luria_core::view_record::ViewRecord;
use ratatui::text::Line;

/// One key press, as `super::terminal` maps it (§4 rule 14, and 8k3 §4 rule 3).
/// Any press this view has no key for is `Other` and changes nothing (§4 rule
/// 10; D055 leaves `r` and `space` unbound, `g` came in 8k2, `/`, `n`, `N` and
/// `tab` in 8l1 and `o` in 8l2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// A printable key, by the character typed.
    Char(char),
    Left,
    Right,
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    /// `enter`, which acts on the highlighted row of an open list (8k3 §4 rule
    /// 3) and elsewhere changes nothing.
    Enter,
    /// `esc`, which closes an open list (8k3 §4 rule 3) or an open prompt, and
    /// with no list, prompt or key list open clears the hits the screen marks
    /// (8m15 §4 rule 4, M8-37).
    Esc,
    /// `backspace`, which shortens an open list's filter (8k3 §4 rule 3) and
    /// elsewhere changes nothing.
    Backspace,
    /// `tab`, which flips the search prompt's mode (8l1 §4 rule 2) and elsewhere
    /// changes nothing.
    Tab,
    /// `ctrl-c`, which quits from anywhere (§4 rule 10).
    CtrlC,
    /// `ctrl-d`, which moves the cursor half a screen down (8m15 §4 rules 1
    /// and 2, M8-38) and half a page down in an open list.
    CtrlD,
    /// `ctrl-u`, which moves the cursor half a screen up (8m15 §4 rules 1 and
    /// 2, M8-38) and half a page up in an open list.
    CtrlU,
    /// `ctrl-r`, which redoes the last group `u` undid (9g §4 rule 8, M9-8), and
    /// which every list or prompt swallows as nothing.
    CtrlR,
    /// Any other press.
    Other,
}

/// What the session does after a key: draw again, or leave (§4 rule 10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Continue,
    Quit,
}

/// How far `w` and `b` look for the next codon's first base (M8-35, 8m13 §4
/// rule 2): a codon is three bases, so the first base of the next one is never
/// more than three bases from the first base of this one.
const CODON_SEARCH: u64 = 3;

/// The blank row between two blocks (§4 rule 1). `super::screen` leaves exactly
/// this many, and `ViewState::visible` counts this many, so the rows the run is
/// measured in are the rows the screen fills.
pub(super) const GAP: u16 = 1;

/// The rows a run of blocks and their gaps have to fit in (§4 rule 1): the
/// frame's inner rows less the one the status line takes, which is
/// `height - 4`.
fn body_rows(height: u16) -> usize {
    usize::from(height).saturating_sub(4)
}

/// The rows the overview strip takes at the top of the frame's inner area (8l2 §4
/// rule 2, 8m12 §4 rule 4): the bars, the marks under them, and the GC line under
/// both. `super::screen` draws them there and `ViewState::block_rows` takes them
/// off the rows a run of blocks has, so the rows the state measures are the rows
/// the screen fills. While a list is open the strip is not drawn and the list
/// keeps `body_rows`, exactly as 8k3 gives it.
pub(super) const OVERVIEW_ROWS: u16 = 3;

/// The width `layout` is given: the frame's border takes one column at each
/// side (§4 rule 1).
fn inner_width(width: u16) -> u16 {
    width.saturating_sub(2)
}

/// True at a size the view draws at: below `MIN_WIDTH` by `MIN_HEIGHT` row 0
/// says the size and nothing else is drawn, and no layout is computed (§4 rules
/// 1 and 10 — the same rule `review::view::too_small` gives the review).
fn big_enough(width: u16, height: u16) -> bool {
    width >= MIN_WIDTH && height >= MIN_HEIGHT
}

/// The words a `layout` refusal puts in the status line (§4 rule 7). Of the
/// three, only `Domain` is reachable at a size the view draws at: a mode's CDS
/// is checked before it is selected, and `MIN_WIDTH` leaves room for the widest
/// gutter the size ceiling (D048 M8-14) allows. The other two are worded so the
/// line is never empty, and none of them invents a refusal of its own: a
/// `Domain` refusal is carried through in its own words (D053).
fn problem_words(problem: &LayoutProblem) -> String {
    match problem {
        LayoutProblem::TooNarrow { width, needed } => {
            format!("the screen gives {width} columns for a block; {needed} are needed")
        }
        LayoutProblem::NotUsable(index) => format!("CDS {index} cannot be selected"),
        LayoutProblem::Domain(error) => error.message(),
    }
}

/// The words a feature level names itself with in the status line, as `F` puts
/// them there (8m6 §4 rule 3): `features: all`, `features: CDSs only`,
/// `features: none`.
fn feature_words(level: FeatureLevel) -> &'static str {
    match level {
        FeatureLevel::All => "all",
        FeatureLevel::Cdss => "CDSs only",
        FeatureLevel::None => "none",
    }
}

/// What `O` says in the status line, and what `T` says while the lines are drawn
/// (M8-40, contract 8m18 §4 rule 6): `ORFs shown: 9` with the number `find_orfs`
/// kept for the topology the view reads, and `ORFs hidden` with the lines hidden.
fn orfs_words(shown: bool, count: usize) -> String {
    if shown {
        format!("ORFs shown: {count}")
    } else {
        "ORFs hidden".to_string()
    }
}

/// One codon of the selected CDS, as the underline and the status line need
/// them (§4 rules 12 and 13). `super::screen` reads these; they are not part of
/// the state's public surface.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct CursorCodon {
    /// The codon's three reference positions, in the order the codon is read:
    /// ascending on the plus strand, descending on the minus strand (D040).
    pub(super) positions: [u64; 3],
    /// The codon as read on its own strand — the complement, in reading order,
    /// for a minus-strand CDS — which is what the status line shows (§4 rule
    /// 13).
    pub(super) letters: String,
    /// The 1-based first and last base of the codon in reference order (§4 rule
    /// 13), which is not the reading order on the minus strand (D040).
    pub(super) first: u64,
    pub(super) last: u64,
    /// The residue this codon produces, `None` for a terminal stop codon, which
    /// has a codon and no residue (D010).
    pub(super) number: Option<u64>,
    /// The residue's letter, `None` when there is no residue.
    pub(super) letter: Option<char>,
}

/// The selected CDS's facts, built once per selection (D042): its `CdsMap` and
/// the residue letters of the translation it is built from. Rebuilding them per
/// key or per draw would translate the CDS again for every press.
#[derive(Clone, Debug, PartialEq)]
struct Chosen {
    map: CdsMap,
    /// The translation's one-letter residues in reading order, without a
    /// terminal stop (D010), held as `char`s so that a block's residue lines
    /// borrow them instead of making them again (8r3 §4 item 2).
    letters: Vec<char>,
    /// As the feature states it, never inferred (D009): which way the codon
    /// keys and the codon's letters run.
    strand: Strand,
}

/// The mode the keys are read in (M9-2, M9-12): `Normal` moves the cursor and
/// runs the view's own keys, `Visual` selects from the anchor to the cursor.
/// 9f adds the three modes that take text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EditMode {
    Normal,
    /// `v` anchors the selection at the cursor it was pressed on and the
    /// movement keys extend it to the cursor.
    Visual {
        anchor: u64,
    },
    /// `i` and `a` (9f §4 rule 1): bases go in before the zero-based `point`,
    /// and `typed` counts how many this spell of insert has taken in, which is
    /// the only thing `backspace` may delete (9f §4 rule 2).
    Insert {
        point: u64,
        typed: u64,
    },
    /// `R` (9f §4 rule 3): every letter substitutes the base the cursor stands
    /// on, and the cursor goes one right.
    Overtype,
    /// `r` (9f §4 rule 4): the next letter substitutes the cursor's base, or
    /// every base of the selection `r` was pressed on.
    ReplaceOne,
}

/// The view between keys: the record, the screen's size, the selection, the
/// lanes, the cursor and the top block.
pub struct ViewState {
    record: ViewRecord,
    /// The record exactly as it was opened: `rebuild` reads its format, its
    /// topology source and its GenBank header off it, and 9h's save writes the
    /// file it came from. Every edit moves `record`; this one does not.
    opened: ViewRecord,
    width: u16,
    height: u16,
    /// The topology the view moves on (D049 M8-3): what the file states, or for
    /// a FASTA record the assumption `T` may set aside for the session.
    circular: bool,
    /// The selected CDS's index into `record.cdss`, `None` for none (M8-11).
    selected: Option<usize>,
    mode: LaneMode,
    /// The cursor's zero-based base (D006).
    cursor: u64,
    /// The block drawn on top, an index into `layout.blocks`.
    top: usize,
    /// The layout for this width and mode, `None` below `MIN_WIDTH` by
    /// `MIN_HEIGHT` or when `layout` refuses.
    layout: Option<Layout>,
    /// The one line `T` or a refusal leaves in the status line, cleared by the
    /// next key (§4 rule 13).
    message: Option<String>,
    /// The open feature list or CDS picker, `None` when neither is open (8k3 §4:
    /// `f` and `c` open one and `esc` closes it; while one is open the keys go to
    /// it and its rows take the body in place of the blocks).
    list: Option<List>,
    /// The residue go-to typed with no CDS selected, remembered until the CDS
    /// picker it opened answers it (M8-22 rule 3): `enter` on a usable row takes it
    /// and `esc` drops it.
    pending: Option<(char, u64)>,
    /// The text typed at the `g` prompt, `None` while go-to is closed (8k2 §4
    /// rule 2): while it is open every printable key is text, `enter` acts on the
    /// whole line through `super::goto`, and `esc` closes it and changes nothing.
    prompt: Option<String>,
    /// The open search prompt, `None` while it is closed (8l1 §4 rules 1 and 2):
    /// while it is open the keys go to it, `tab` flips its mode, `enter` searches
    /// and `esc` closes it and leaves the hits as they were.
    search: Option<SearchPrompt>,
    /// The hits marked on the screen now, `None` when nothing is marked (8l1 §4
    /// rules 3 to 6): a search replaces them, `T` clears them, and `n` and `N` move
    /// which one is current.
    hits: Option<Hits>,
    /// Whether the overview strip is drawn (8l2 §4 rule 1): the three lines `o` hides
    /// and shows, on at open when the layout has more than twice as many blocks as
    /// the body rows hold without them. While a list is open they are not drawn,
    /// but the flag is kept, so closing the list brings them back.
    overview: bool,
    /// The strip's third line, kept from the record and the width so that drawing
    /// never counts bases again (8m12 §4 rule 5): `super::overview::gc_line`'s `w`
    /// cells for this frame's inner width, recomputed whenever the width is.
    gc: String,
    /// The level the feature lanes are drawn at (8m3 §4 rule 1, 8m6 §4 rule 3):
    /// what `F` steps through the three levels, `All` at open, and what
    /// `block_lines` draws the lanes for. Under `None` a block leaves out every
    /// lane line and the `+N more` line. The level is kept while a list or a
    /// prompt is open, so the lines come back with the blocks.
    features: FeatureLevel,
    /// Whether the ORF lines are drawn (M8-40, contract 8m18 §4 rule 6): the
    /// lanes `O` hides and shows, hidden at open, and what `drawn_lines` hands
    /// `block_lines` as `orfs`.
    orf_lanes: bool,
    /// The ORFs the shown lines draw (contract 8m18 §4 rule 6): `find_orfs` once,
    /// on the first `O` for the topology then shown, kept across every toggle and
    /// dropped by `T` when it flips the topology. `None` says they have not been
    /// computed for the topology the view reads, which holds at open and while
    /// `O` hides them, since §9 asks that nothing is computed before the first `O`.
    orfs: Option<Vec<Orf>>,
    /// Whether the residue lines draw the Zappo backgrounds (8m5 §4 rule 4): `C`
    /// turns them off and on, they are on at open unless `NO_COLOR` says
    /// otherwise (8m5 §4 rule 5), and while a list or a prompt is open `C` is
    /// text and the flag is kept.
    colours: bool,
    /// Whether the key list is open (8m3 §4 rule 2): `?` opens it, its rows take
    /// the body in place of the blocks, and any key closes it and does nothing
    /// else.
    help: bool,
    /// Whether the record can be: `buffer_for`'s answer at open (M9-12), the
    /// buffer 9e edits or the reason every editing key is refused.
    edit: Result<EditBuffer, EditBlocked>,
    /// The mode the keys are read in (M9-2): `Normal` at open, `Visual` once `v`
    /// anchors a selection at the cursor.
    editing: EditMode,
    /// The bases `r` was pressed on (9f §4 rule 4), kept because entering
    /// `ReplaceOne` drops the `Visual` that gave them: the next letter
    /// substitutes every one of them in a single `Substitute`.
    replace_selection: Option<(u64, u64)>,
    /// Whether overtype has run past the record's last base (9f §4 rule 3): the
    /// first letter there still substitutes and stays, and the next one says
    /// overtype stops at the end of the record.
    overtype_stopped: bool,
    /// The last search that ran (8l1 §4 rules 3 to 5): its mode and pattern, kept
    /// so `apply_edit` marks the same hits again after every edit (contract 9e
    /// §4 rule 6, M9-6 i). `None` until the first `enter` at the search prompt;
    /// an `esc` that closes the prompt leaves this memory untouched.
    last_search: Option<(SearchMode, String)>,
    /// The selected CDS's map and residues, `None` with no selection.
    chosen: Option<Chosen>,
    /// The bases `x`/`d` cut and `y` yank, and what `p`/`P` reads back (9g
    /// §4 rule 1): empty at open, replaced whole by every cut and yank, kept
    /// for the session (M9-5: never the system clipboard).
    register: Vec<char>,
    /// The edit groups `u` pops and `ctrl-r` pushes back (9g §4 rules 6 to 8):
    /// oldest first, each group holding one command's steps as `(Edit, Undo)`
    /// in the order they were applied.
    undo_groups: Vec<Vec<(Edit, Undo)>>,
    /// The groups `ctrl-r` re-applies in order and `u` returns (9g §4 rule 8),
    /// oldest first, with the same step order as the original command.
    redo_groups: Vec<Vec<(Edit, Undo)>>,
    /// The command whose steps are still being collected (9g §4 rule 6: each
    /// `apply_edit` adds its step here, and the command's last key closes it
    /// into `undo_groups`.
    current_group: Vec<(Edit, Undo)>,
}

impl ViewState {
    /// The view as it opens: M8-11 decides the selection — exactly one usable
    /// CDS opens selected, several select none and the title says so — the lanes
    /// follow it, the cursor is at base 0 and the first block is on top (§4 rule
    /// 2).
    pub fn new(record: ViewRecord, width: u16, height: u16) -> ViewState {
        let selected: Option<usize> = opened(&record);
        let mode: LaneMode = match selected {
            Some(index) => LaneMode::Cds(index),
            None => LaneMode::Off,
        };
        // The strip's GC line is counted from the record once here, and again on
        // every new width, and kept for the frames between (8m12 §4 rule 5).
        let gc: String = gc_line(&record, inner_width(width));
        // M9-12: the record's editability is decided once at open, and every
        // editing key reads it rather than asking again.
        let edit: Result<EditBuffer, EditBlocked> = buffer_for(&record);
        let mut state: ViewState = ViewState {
            circular: record.sequence.topology() == SequenceTopology::Circular,
            selected,
            mode,
            opened: record.clone(),
            record,
            width,
            height,
            cursor: 0,
            top: 0,
            layout: None,
            message: None,
            list: None,
            pending: None,
            prompt: None,
            search: None,
            hits: None,
            overview: false,
            gc,
            features: FeatureLevel::All,
            orf_lanes: false,
            orfs: None,
            colours: true,
            help: false,
            edit,
            editing: EditMode::Normal,
            replace_selection: None,
            overtype_stopped: false,
            last_search: None,
            chosen: None,
            register: Vec::new(),
            undo_groups: Vec::new(),
            redo_groups: Vec::new(),
            current_group: Vec::new(),
        };
        state.choose();
        state.relayout(false);
        // 8l2 §4 rule 1: the strip opens drawn on a record too long for the body,
        // measured against the rows a run of blocks has without it.
        state.overview = state.opens_with_strip();
        state
    }

    /// A new terminal size: the record is laid out again for the new width, the
    /// top block keeps the base it covered before (§4 rule 6), and the scroll
    /// follows the cursor.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.gc = gc_line(&self.record, inner_width(width));
        self.relayout(true);
    }

    /// One key press: what it moves, and whether the session goes on (§4 rules
    /// 2 to 10). Whatever the key, the message a previous key left is cleared
    /// here; `q` and `ctrl-c` leave, and below `MIN_WIDTH` by `MIN_HEIGHT`
    /// nothing else does anything (§4 rule 10). While a list is open the keys go
    /// to it instead, and only `ctrl-c` leaves (8k3 §4 rule 3); while the go-to
    /// prompt is open the keys go to it, and again only `ctrl-c` leaves (8k2 §4
    /// rule 2); and while the search prompt is open its keys are the prompt's, with
    /// the same rule for `ctrl-c` and `q` (8l1 §4 rules 1 and 2).
    pub fn handle(&mut self, key: Key) -> Action {
        // `ctrl-c` quits from anywhere, `q` everywhere but inside an open list, an
        // open go-to prompt or an open search prompt, where every printable key is
        // text first (8k3 §4 rule 3, 8k2 §4 rule 2, 8l1 §4 rule 2) — except below
        // `MIN_WIDTH` by `MIN_HEIGHT`, where the list and the prompts draw nothing
        // and only `q` and `ctrl-c` do anything at all (8k1 §4 rule 10).
        let tiny: bool = !big_enough(self.width, self.height);
        let typing: bool = self.list.is_some()
            || self.prompt.is_some()
            || self.search.is_some()
            // 8m3's key list takes `q` itself: any key closes it and does nothing
            // else, and only `ctrl-c` leaves (8m3 §4 rule 2).
            || self.help;
        // The three modes that take text read every printable key as text first
        // (M9-2, 9f §4), so `q` there is a letter those modes refuse, not quit.
        // `ctrl-c` quits from anywhere as it does today (9h changes that).
        let typing_text: bool = matches!(
            self.editing,
            EditMode::Insert { .. } | EditMode::Overtype | EditMode::ReplaceOne
        );
        if key == Key::CtrlC || (key == Key::Char('q') && (!typing || tiny) && !typing_text) {
            return Action::Quit;
        }
        self.message = None;
        if tiny {
            return Action::Continue;
        }
        // Any key closes the key list and does nothing else, `q` and `F` and `?`
        // included (8m3 §4 rule 2). `ctrl-c` left above and `q` did not quit there.
        if self.help {
            self.help = false;
            return Action::Continue;
        }
        if self.list.is_some() {
            self.list_key(key);
            return Action::Continue;
        }
        // While go-to is open its keys go to the prompt, and it is `enter` there
        // that moves the cursor and closes it (8k2 §4 rule 2).
        if self.prompt.is_some() {
            self.prompt_key(key);
            return Action::Continue;
        }
        // While the search prompt is open its keys go to it, and it is `enter`
        // there that closes it and searches (8l1 §4 rule 2).
        if self.search.is_some() {
            self.search_key(key);
            return Action::Continue;
        }
        // The three modes that take text own their keys, so the view's own keys
        // are text there and not commands (M9-2, 9f §4 rules 2, 3 and 4).
        match self.editing {
            EditMode::Insert { .. } => {
                self.insert_key(key);
                return Action::Continue;
            }
            EditMode::Overtype => {
                self.overtype_key(key);
                return Action::Continue;
            }
            EditMode::ReplaceOne => {
                self.replace_one_key(key);
                return Action::Continue;
            }
            EditMode::Normal | EditMode::Visual { .. } => {}
        }
        match key {
            Key::Char('h') | Key::Left => self.step_base(-1),
            Key::Char('l') | Key::Right => self.step_base(1),
            Key::Char('j') | Key::Down => self.step_line(1),
            Key::Char('k') | Key::Up => self.step_line(-1),
            Key::Char('w') => self.step_codon(1),
            Key::Char('b') => self.step_codon(-1),
            Key::PageDown => self.page(1),
            Key::PageUp => self.page(-1),
            // `ctrl-d` and `ctrl-u`: half a screen of blocks, as in Vim
            // (8m15 §4 rules 1 and 2, M8-38).
            Key::CtrlD => self.half_page(1),
            Key::CtrlU => self.half_page(-1),
            Key::Home => self.cursor = 0,
            Key::End => {
                if let Some(last) = self.record.sequence.length().checked_sub(1) {
                    self.cursor = last;
                }
            }
            Key::Char('t') => {
                self.cycle();
                return Action::Continue;
            }
            Key::Char('T') => {
                self.flip();
                return Action::Continue;
            }
            // `o` hides and shows the strip (8l2 §4 rule 1); inside a list or a
            // prompt `o` is text, which the branches above handle. Showing it takes
            // three rows from the blocks, so the scroll follows the cursor as 8k1 §4
            // rule 6 has it: the cursor's block stays drawn, and the top moves as
            // little as the rows allow. Hiding it only makes room, so `follow` has
            // nothing to do.
            Key::Char('o') => {
                self.overview = !self.overview;
                self.follow();
                return Action::Continue;
            }
            // `f` and `c` open the two lists, with no filter typed (8k3 §4 rule
            // 1); the rows they show and the keys that go to them are 8k3's.
            // `g` opens go-to with nothing typed (8k2 §4 rule 2); while a list is
            // open `g` is filter text instead, which the branch above handles.
            Key::Char('g') => {
                self.prompt = Some(String::new());
                return Action::Continue;
            }
            // `F` steps the feature lanes through their three levels — all,
            // CDSs only, none (8m6 §4 rule 3) — and names the level reached in
            // the status line; inside a list or a prompt it is text, as every
            // printable key is there. The levels change how many lines a block
            // has, so the scroll follows the cursor as the `follow` every
            // cursor-moving key falls through to.
            // `C` colours the residues by Zappo group and uncolours them again
            // (8m5 §4 rule 4); inside a list or a prompt it is text, as `F`
            // above. Colours change no line and no row, so the scroll has
            // nothing to follow.
            Key::Char('F') => self.step_features(),
            // `O` hides the open reading frames and shows them again (M8-40,
            // contract 8m18 §4 rule 6); inside a list or a prompt it is text, as
            // `F` and `C` above. The lines change a block's rows, so the scroll
            // follows the cursor as `F` does.
            Key::Char('O') => self.toggle_orf_lanes(),
            Key::Char('C') => self.colours = !self.colours,
            Key::Char('f') => self.open_list(ListKind::Features, ""),
            Key::Char('c') => self.open_list(ListKind::Cdss, ""),
            // `/` opens the search prompt with nothing typed (8l1 §4 rule 1); in a
            // list or at the go-to prompt `/` is text, which those branches handle.
            Key::Char('/') => {
                self.search = Some(SearchPrompt::open());
                return Action::Continue;
            }
            // `n` and `N` walk the hits shown now (8l1 §4 rule 5); while the search
            // prompt is open they are text, which the branch above handles.
            Key::Char('n') => return self.step_hit(true),
            Key::Char('N') => return self.step_hit(false),
            // `?` opens the key list (8m3 §4 rule 2); in a list or at a prompt the
            // branches above make it text instead.
            Key::Char('?') => self.help = true,
            // `v` opens a selection anchored at the cursor (M9-2, M9-12: the
            // movement keys extend it, and `v` or `esc` closes it. An uneditable
            // record is refused as §4 rule 4 says, naming the feature and why.
            Key::Char('v') => {
                match self.editing {
                    EditMode::Normal => match &self.edit {
                        Ok(_buffer) => {
                            self.editing = EditMode::Visual {
                                anchor: self.cursor,
                            }
                        }
                        Err(blocked) => self.message = Some(blocked_words(blocked)),
                    },
                    // `v` in `Visual` ends the selection, as `esc` does (M9-2).
                    EditMode::Visual { .. } => self.editing = EditMode::Normal,
                    // The three modes that take text read their own keys before
                    // the view's keys, so `v` there is text and never reaches
                    // branch (`v` is a base letter in a typing mode).
                    EditMode::Insert { .. } | EditMode::Overtype | EditMode::ReplaceOne => {}
                }
                return Action::Continue;
            }
            // `esc` that ends a selection closes it alone: the hits the screen
            // marks stay, so `n` and `N` still walk them (M9-2).
            Key::Esc if matches!(self.editing, EditMode::Visual { .. }) => {
                self.editing = EditMode::Normal;
                return Action::Continue;
            }
            // `i` inserts before the cursor's base and `a` after it (9f §4 rule
            // 1): the insert point is the cursor for `i` and the cursor + 1 for
            // `a`, and either spell of insert starts with nothing typed.
            Key::Char('i') => {
                self.open_insert(self.cursor);
                return Action::Continue;
            }
            Key::Char('a') => {
                // The point goes after the cursor's base, so one base past it; on
                // the record's last base that point is its end, where the insert
                // appends instead of running past the record.
                let point: u64 = self
                    .cursor
                    .saturating_add(1)
                    .min(self.record.sequence.length());
                self.open_insert(point);
                return Action::Continue;
            }
            // `R` overtypes from the cursor (9f §4 rule 3).
            Key::Char('R') => {
                self.open_overtype();
                return Action::Continue;
            }
            // `r` replaces one base, or every base of the selection it was
            // pressed on, with the next letter typed (9f §4 rule 4).
            Key::Char('r') => {
                self.open_replace();
                return Action::Continue;
            }
            // The keys 9e edits with (M9-2, M9-12), now carrying 9g's cut,
            // yank, paste and the undo of M9-8: as 9d says, a record whose feature
            // 9b2 refuses names that feature and core's reason instead, and
            // nothing else happens. `s`, the save prompt, is 9h's.
            Key::Char('x') => {
                self.cut();
                return Action::Continue;
            }
            Key::Char('d') => {
                self.cut_or_ask();
                return Action::Continue;
            }
            Key::Char('y') => {
                self.yank();
                return Action::Continue;
            }
            Key::Char('p') => {
                self.paste_register(true);
                return Action::Continue;
            }
            Key::Char('P') => {
                self.paste_register(false);
                return Action::Continue;
            }
            Key::Char('u') => {
                self.undo_group();
                return Action::Continue;
            }
            Key::CtrlR => {
                self.redo_group();
                return Action::Continue;
            }
            Key::Char('s') => {
                if let Err(blocked) = &self.edit {
                    self.message = Some(blocked_words(blocked));
                }
                return Action::Continue;
            }
            // `esc` with no list, prompt or key list open clears the hits the
            // screen marks: the marks go, `n` and `N` have nothing to step, and
            // the cursor, the selection, the lanes and the scroll stay as they
            // are, with nothing said in the status line (8m15 §4 rule 4, M8-37).
            // With no hits it changes nothing. At a prompt or in a list the
            // branches above close them instead, leaving the hits.
            Key::Esc => {
                self.hits = None;
                return Action::Continue;
            }
            // Every other key changes nothing (§4 rule 10), `enter`,
            // `backspace` and `tab` included: outside a list or a prompt they
            // have no row and no mode to act on.
            Key::Char(_) | Key::Other | Key::CtrlC | Key::Enter | Key::Backspace | Key::Tab => {
                return Action::Continue
            }
        }
        self.follow();
        Action::Continue
    }

    /// `i` and `a` (9f §4 rule 1): the spell opens at `point` — the cursor for
    /// `i`, one base past it for `a` — with nothing typed yet. The record's
    /// editability is checked first, and a record 9d refuses answers with 9d's
    /// own words (9f §4 rule 7).
    fn open_insert(&mut self, point: u64) {
        match &self.edit {
            Ok(_buffer) => self.editing = EditMode::Insert { point, typed: 0 },
            Err(blocked) => self.message = Some(blocked_words(blocked)),
        }
    }

    /// `R` (9f §4 rule 3): overtype overwrites from the cursor on and never
    /// lengthens the record. A record 9d refuses answers with 9d's own words
    /// (9f §4 rule 7).
    fn open_overtype(&mut self) {
        match &self.edit {
            Ok(_buffer) => {
                self.editing = EditMode::Overtype;
                self.overtype_stopped = false;
            }
            Err(blocked) => self.message = Some(blocked_words(blocked)),
        }
    }

    /// `r`, from `Normal` or from `Visual` (9f §4 rule 4): the next letter
    /// substitutes one base, or every base of the selection `r` was pressed on,
    /// in one `Substitute`. The `Visual` range is kept aside because entering
    /// `ReplaceOne` drops the `Visual` that `selection` reads it from; the field
    /// goes when the spell does. A record 9d refuses answers with 9d's own
    /// words (9f §4 rule 7).
    fn open_replace(&mut self) {
        match &self.edit {
            Ok(_buffer) => {
                self.replace_selection = self.selection();
                self.editing = EditMode::ReplaceOne;
            }
            Err(blocked) => self.message = Some(blocked_words(blocked)),
        }
    }

    /// Leaves `ReplaceOne`, dropping the selection kept for it (9f §4 rule 4).
    fn close_replace(&mut self) {
        self.editing = EditMode::Normal;
        self.replace_selection = None;
    }

    /// The words a printable key that is not a base leaves in the status line
    /// (9f §4 rules 2, 3 and 4): the character itself and what can be typed.
    fn refusal_words(&mut self, letter: char) {
        self.message = Some(format!(
            "'{letter}' is not one of {DNA_ALPHABET}: only bases can be typed"
        ));
    }

    /// A key pressed in `Insert` (9f §4 rule 2): a base in the 15 IUPAC letters,
    /// either case, goes in as one `Insert` at the point and the point, the count
    /// and the cursor move on; every other printable key changes nothing and says
    /// only bases can be typed, so `q`, `x`, `f` and the view's keys are letters
    /// here and not commands (M9-2). `backspace` takes back only what this spell
    /// of insert put in, and `esc` leaves the mode. Any other press — the arrows,
    /// `enter`, `tab` — changes nothing.
    fn insert_key(&mut self, key: Key) {
        let (point, typed): (u64, u64) = match self.editing {
            EditMode::Insert { point, typed } => (point, typed),
            _ => return,
        };
        match key {
            Key::Esc => {
                // The cursor stands on the last base typed; with nothing typed such
                // a spell, it never moved and stays where the spell opened.
                if typed > 0 {
                    self.cursor = point.saturating_sub(1);
                }
                self.editing = EditMode::Normal;
                // 9g §4 rule 6: the whole spell of insert, its backspaces
                // included, is one group on the undo stack.
                self.close_group();
                self.follow();
            }
            Key::Backspace => {
                // A base that was there before the `i` was pressed is never taken
                // back: `backspace` is bounded by the bases this spell typed.
                if typed == 0 {
                    return;
                }
                // `typed > 0` implies the point is at least one base along.
                if self
                    .apply_edit(Edit::Delete {
                        start: point - 1,
                        end: point,
                    })
                    .is_err()
                {
                    // `apply_edit` has said why the base was not taken back.
                    return;
                }
                self.editing = EditMode::Insert {
                    point: point - 1,
                    typed: typed - 1,
                };
                // 9f rule 2, mirrored from the letter arm: the cursor stands on the
                // new point, clamped to the record's last base.
                let last: u64 = self.record.sequence.length().saturating_sub(1);
                self.cursor = (point - 1).min(last);
                self.follow();
            }
            Key::Char(letter) if is_dna_letter(letter.to_ascii_uppercase()) => {
                if self
                    .apply_edit(Edit::Insert {
                        at: point,
                        bases: vec![letter.to_ascii_uppercase()],
                    })
                    .is_err()
                {
                    // `apply_edit` has said why nothing was inserted.
                    return;
                }
                self.editing = EditMode::Insert {
                    point: point + 1,
                    typed: typed + 1,
                };
                // 9f rule 2: the cursor goes to the new point, clamped to the
                // record's last base.
                let last: u64 = self.record.sequence.length().saturating_sub(1);
                self.cursor = (point + 1).min(last);
                self.follow();
            }
            Key::Char(letter) => self.refusal_words(letter),
            _ => {}
        }
    }

    /// A key pressed in `Overtype` (9f §4 rule 3): a base substitutes the base
    /// the cursor stands on and the cursor goes one right; at the record's last
    /// base the letter still substitutes, but the cursor stays and the next letter
    /// says overtype stops at the record's end — overtype never lengthens the
    /// record. Any other printable key changes nothing and says only bases can be
    /// typed; `esc` leaves the mode and any other press changes nothing.
    fn overtype_key(&mut self, key: Key) {
        match key {
            Key::Esc => {
                self.editing = EditMode::Normal;
                self.overtype_stopped = false;
                // 9g §4 rule 6: every overtype letter up to here is one group.
                self.close_group();
                self.follow();
            }
            Key::Char(letter) if is_dna_letter(letter.to_ascii_uppercase()) => {
                if self.overtype_stopped {
                    self.message = Some("overtype stops at the end of the record".to_string());
                    return;
                }
                if self
                    .apply_edit(Edit::Substitute {
                        start: self.cursor,
                        bases: vec![letter.to_ascii_uppercase()],
                    })
                    .is_err()
                {
                    // `apply_edit` has said why nothing was substituted.
                    return;
                }
                let last: u64 = self.record.sequence.length().saturating_sub(1);
                if self.cursor >= last {
                    self.overtype_stopped = true;
                } else {
                    self.cursor += 1;
                    self.follow();
                }
            }
            Key::Char(letter) => self.refusal_words(letter),
            _ => {}
        }
    }

    /// A key pressed in `ReplaceOne` (9f §4 rule 4): a base substitutes every
    /// base `r` reaches, in one `Substitute` spanning the cursor's base or the
    /// whole selection. `esc` ends the spell with nothing typed; any other press
    /// ends it with a refusal, exactly as a non-base press does.
    fn replace_one_key(&mut self, key: Key) {
        match key {
            Key::Esc => self.close_replace(),
            Key::Char(letter) if is_dna_letter(letter.to_ascii_uppercase()) => {
                let (start, end): (u64, u64) = self
                    .replace_selection
                    .unwrap_or((self.cursor, self.cursor.saturating_add(1)));
                let count: usize = (end - start) as usize;
                let outcome: Result<EditOutcome, DomainError> = self.apply_edit(Edit::Substitute {
                    start,
                    bases: vec![letter.to_ascii_uppercase(); count],
                });
                self.close_replace();
                // 9g §4 rule 6: the whole `r`, whatever it replaced, is one
                // group; the refusal arms leave the group empty and change.
                self.close_group();
                if outcome.is_ok() {
                    self.follow();
                }
            }
            Key::Char(letter) => {
                self.close_replace();
                self.message = Some(format!(
                    "'{letter}' is not one of {DNA_ALPHABET}: the replace is cancelled"
                ));
            }
            _ => {
                self.close_replace();
                self.message = Some("r needs a base letter: the replace is cancelled".to_string());
            }
        }
    }

    /// The terminal's own paste, handed over as one piece of text (9f §4 rule 5,
    /// M9-4): in `Insert` the text is cleaned and inserted in one `Insert`, and
    /// nothing is inserted when the clean-up refusal hits — the whole paste goes.
    /// The rules apply in this order: a line whose first non-space character is `>`
    /// is dropped whole, then spaces, tabs, `\r`, `\n` and ASCII digits are
    /// dropped, and any other character that is not one of the 15 IUPAC
    /// bases says so, naming the character and its 1-based line and column.
    /// Outside `Insert` nothing is pasted and the line asks for `i` or `a`.
    pub(super) fn paste_text(&mut self, text: &str) {
        let (point, typed): (u64, u64) = match self.editing {
            EditMode::Insert { point, typed } => (point, typed),
            // M9-5: the terminal's paste works in insert mode only.
            _ => {
                self.message = Some("paste: press i or a first".to_string());
                return;
            }
        };
        // M9-4 in the order the contract gives: a FASTA/GenBank header line goes,
        // the whitespace and digits of a body line go, and anything else that is
        // not one of the 15 bases refuses the whole paste.
        let mut bases: Vec<char> = Vec::new();
        for (line_number, line) in text.split('\n').enumerate() {
            let is_header: bool = line
                .chars()
                .find(|ch| !ch.is_whitespace())
                .is_some_and(|ch| ch == '>');
            if is_header {
                continue;
            }
            for (column, ch) in line.chars().enumerate() {
                if ch == ' ' || ch == '\t' || ch == '\r' || ch.is_ascii_digit() {
                    continue;
                }
                let upper: char = ch.to_ascii_uppercase();
                if !is_dna_letter(upper) {
                    self.message = Some(format!(
                        "paste refused: '{ch}' at line {} column {}",
                        line_number + 1,
                        column + 1
                    ));
                    return;
                }
                bases.push(upper);
            }
        }
        if bases.is_empty() {
            return;
        }
        // What is left, if not empty, goes in as one `Insert` at the point.
        let count: u64 = bases.len() as u64;
        if self
            .apply_edit(Edit::Insert {
                at: point,
                bases: std::mem::take(&mut bases),
            })
            .is_err()
        {
            // `apply_edit` has said why nothing was pasted.
            return;
        }
        self.editing = EditMode::Insert {
            point: point + count,
            typed: typed + count,
        };
        let last: u64 = self.record.sequence.length().saturating_sub(1);
        self.cursor = (point + count).min(last);
        self.follow();
    }

    /// `x` (9g §4 rule 2): cuts the cursor's own base in `Normal`, and the
    /// selection in `Visual`, as one `Delete`. The register gets what the cut
    /// took, the cursor goes to the cut's start clamped, and a `Visual` cut
    /// leaves the mode. The bases are read off the buffer before the edit, so
    /// the register holds exactly what the cut removed.
    fn cut(&mut self) {
        let (start, end): (u64, u64) = match self.selection() {
            Some(range) => range,
            None => (self.cursor, self.cursor.saturating_add(1)),
        };
        // A record 9d refuses answers with 9d's own words, and the selection is
        // kept for another try.
        let letters: Vec<char> = match &self.edit {
            Ok(buffer) => buffer.letters()[start as usize..end as usize].to_vec(),
            Err(blocked) => {
                self.message = Some(blocked_words(blocked));
                return;
            }
        };
        if self.apply_edit(Edit::Delete { start, end }).is_err() {
            // `apply_edit` has said why the bases were not cut.
            return;
        }
        self.register = letters;
        self.editing = EditMode::Normal;
        // §4 rule 2: the cursor goes to the cut's start, clamped to the record's
        // last base when the cut took the tail of the record out.
        let last: u64 = self.record.sequence.length().saturating_sub(1);
        self.cursor = start.min(last);
        self.follow();
        self.close_group();
    }

    /// `d` (9g §4 rule 3): with a selection, the very cut `x` does; in `Normal`
    /// nothing is cut beyond a line that asks for a selection made with `v`.
    fn cut_or_ask(&mut self) {
        if self.selection().is_none() {
            self.message = Some("d needs a selection: press v first to select".to_string());
            return;
        }
        self.cut();
    }

    /// `y` (9g §4 rule 4): the selection's bases are copied to the register and
    /// the record is left as it was; the mode leaves `Visual` and the count of
    /// the yanked bases is said. In `Normal` there is nothing to yank, and the
    /// line asks for a selection made with `v`.
    fn yank(&mut self) {
        let (start, end): (u64, u64) = match self.selection() {
            Some(range) => range,
            None => {
                self.message = Some("y needs a selection: press v first to select".to_string());
                return;
            }
        };
        let letters: Vec<char> = match &self.edit {
            Ok(buffer) => buffer.letters()[start as usize..end as usize].to_vec(),
            Err(blocked) => {
                self.message = Some(blocked_words(blocked));
                return;
            }
        };
        let count: usize = letters.len();
        self.register = letters;
        self.editing = EditMode::Normal;
        self.message = Some(format!("yanked {count} bases"));
    }

    /// `p` and `P` (9g §4 rule 5): the register's bases go back in as one
    /// `Insert`, after the cursor's base (`p`, at the cursor + 1) or before it
    /// (`P`, at the cursor), and the cursor lands on the last pasted base. With
    /// an empty register the line says there is nothing to paste, and the
    /// register itself stays as it is, so `p` can go on repeating.
    fn paste_register(&mut self, after: bool) {
        if let Err(blocked) = &self.edit {
            self.message = Some(blocked_words(blocked));
            return;
        }
        if self.register.is_empty() {
            self.message = Some("paste: the register is empty".to_string());
            return;
        }
        let bases: Vec<char> = self.register.clone();
        let count: u64 = bases.len() as u64;
        let at: u64 = if after {
            self.cursor
                .saturating_add(1)
                .min(self.record.sequence.length())
        } else {
            self.cursor
        };
        if self.apply_edit(Edit::Insert { at, bases }).is_err() {
            // `apply_edit` has said why nothing was pasted.
            return;
        }
        // §4 rule 5: the cursor goes to the last pasted base, which sits
        // `count` bases after the insert point minus one step.
        let last: u64 = self.record.sequence.length().saturating_sub(1);
        self.cursor = (at + count - 1).min(last);
        self.follow();
        self.close_group();
    }

    /// `u` (9g §4 rule 7, M9-8): the last group's steps are put back in reverse
    /// order straight onto the buffer, and the view is recomputed once for the
    /// whole group. The group goes onto the redo stack for `ctrl-r` to bring
    /// back, and the cursor lands on the first base of the group's first edit,
    /// clamped. With nothing on the undo stack, the line says so.
    fn undo_group(&mut self) {
        let Some(group) = self.undo_groups.pop() else {
            self.message = Some("nothing to undo".to_string());
            return;
        };
        let mut buffer: EditBuffer = match &self.edit {
            Ok(buffer) => buffer.clone(),
            Err(blocked) => {
                self.undo_groups.push(group);
                self.message = Some(blocked_words(blocked));
                return;
            }
        };
        // §4 rule 7: the undo of the last step goes first, so every step is put
        // back on the record the steps before it left.
        for (_edit, undo) in group.iter().rev() {
            buffer.undo(undo.clone());
        }
        let start: u64 = group
            .first()
            .map_or(self.cursor, |(edit, _undo)| edit_start(edit));
        if self.rebuild_view(buffer, Vec::new(), Vec::new()).is_err() {
            // `rebuild_view` has said why; the group goes back where it was.
            self.undo_groups.push(group);
            return;
        }
        self.redo_groups.push(group);
        // §4 rule 7: the cursor goes to the group's first edit, clamped.
        let last: u64 = self.record.sequence.length().saturating_sub(1);
        self.cursor = start.min(last);
        self.follow();
    }

    /// `ctrl-r` (9g §4 rule 8, M9-8): the last group `u` undid is applied
    /// again, in the order its steps were first applied, and each step's fresh
    /// `Undo` takes the old one's place. The group goes back onto the undo
    /// stack, and the view is recomputed once. With nothing on the redo stack,
    /// the line says so.
    fn redo_group(&mut self) {
        let Some(group) = self.redo_groups.pop() else {
            self.message = Some("nothing to redo".to_string());
            return;
        };
        let mut buffer: EditBuffer = match &self.edit {
            Ok(buffer) => buffer.clone(),
            Err(blocked) => {
                self.redo_groups.push(group);
                self.message = Some(blocked_words(blocked));
                return;
            }
        };
        // §4 rule 8: the edits go on again in their original order, and each
        // one's fresh undo goes onto the group in the old one's place.
        let mut fresh: Vec<(Edit, Undo)> = Vec::new();
        let mut refused: Option<DomainError> = None;
        for (edit, _undo) in group.iter() {
            match buffer.apply_undoable(edit) {
                Ok((_outcome, undo)) => fresh.push((edit.clone(), undo)),
                Err(error) => {
                    refused = Some(error);
                    break;
                }
            }
        }
        if let Some(error) = refused {
            self.redo_groups.push(group);
            self.message = Some(error.message());
            return;
        }
        let start: u64 = fresh
            .first()
            .map_or(self.cursor, |(edit, _undo)| edit_start(edit));
        if self.rebuild_view(buffer, Vec::new(), Vec::new()).is_err() {
            // `rebuild_view` has said why; the group goes back where it was.
            self.redo_groups.push(group);
            return;
        }
        self.undo_groups.push(fresh);
        let last: u64 = self.record.sequence.length().saturating_sub(1);
        self.cursor = start.min(last);
        self.follow();
    }

    /// Ends the command group the view is collecting (9g §4 rule 6): the steps
    /// of one command go onto the undo stack together, and the redo stack goes
    /// away, since what sits on it no longer matches the record as this command
    /// leaves it. Nothing happens when the command made.
    fn close_group(&mut self) {
        if self.current_group.is_empty() {
            return;
        }
        // §4 rule 6: a new group clears the redo stack.
        self.redo_groups.clear();
        let group: Vec<(Edit, Undo)> = std::mem::take(&mut self.current_group);
        self.undo_groups.push(group);
    }

    /// The edit groups `u` can step back through (M9-16): the count 9h's save
    /// prompt stamps to say what a quit would leave behind.
    // 9h wires this in; until then the unit tests in this module are its only
    // readers. (M9-16: `edits()` is the undo-stack length the save prompt stamps.)
    #[allow(dead_code)]
    pub(super) fn edits(&self) -> usize {
        self.undo_groups.len()
    }

    /// 8k3's picker: `false`, and nothing changes, for an index out of range or
    /// a CDS that is not `CdsStatus::Usable` (D041, D042, D052). A chosen CDS is
    /// shown by its own lane, and dropping the selection takes a `Cds` mode off
    /// with it (§4 rule 9). The cursor never moves.
    pub fn select_cds(&mut self, cds: Option<usize>) -> bool {
        let picked: Option<usize> = match cds {
            Some(index) => {
                if usable(&self.record, index).is_none() {
                    return false;
                }
                Some(index)
            }
            None => None,
        };
        self.selected = picked;
        match picked {
            Some(index) => self.mode = LaneMode::Cds(index),
            None => {
                if matches!(self.mode, LaneMode::Cds(_)) {
                    self.mode = LaneMode::Off;
                }
            }
        }
        self.choose();
        self.relayout(true);
        true
    }

    /// One edit, and the view after it (contract 9e §4): `EditBuffer::apply`,
    /// `rebuild`, then everything that reads the record again — the selection and
    /// its `Chosen` map, the strip's GC line, the ORFs, the last search's hits,
    /// the cursor and the layout. A refusal from `apply` or from `rebuild`
    /// leaves the view exactly as it was, with core's own message.
    // The edit keys 9f and 9g are wired through this; the tests in this module
    // also call it directly.
    pub(super) fn apply_edit(&mut self, edit: Edit) -> Result<EditOutcome, DomainError> {
        // The edit goes onto a copy of the buffer, so that a refusal from `apply`
        // or from `rebuild` below leaves the buffer the view edits untouched.
        let edited: Result<(EditBuffer, EditOutcome, Undo), DomainError> = match self.edit.as_ref()
        {
            // A record 9d and every later edit key refuses before it gets at
            // the first place; core's own reason is what says so here.
            Ok(buffer) => {
                let mut candidate: EditBuffer = buffer.clone();
                candidate
                    .apply_undoable(&edit)
                    .map(|(outcome, undo)| (candidate, outcome, undo))
            }
            Err(blocked) => Err(blocked.error.clone()),
        };
        let (buffer, outcome, undo) = match edited {
            Ok((buffer, outcome, undo)) => (buffer, outcome, undo),
            Err(error) => {
                self.message = Some(error.message());
                return Err(error);
            }
        };
        // §4 rule 3: the features the edit left with no range, and the ones
        // M9-13 (a) took a qualifier from, are named by the names the feature
        // list shows them by — read off the record as it was before the edit,
        // which still knows both by the file line the buffer keeps.
        let removed: Vec<String> = outcome
            .removed
            .iter()
            .map(|feature| feature_name(&self.record, feature))
            .collect();
        let dropped: Vec<String> = outcome
            .dropped
            .iter()
            .map(|(index, qualifier)| {
                let name: String = match buffer.features().get(*index) {
                    Some(feature) => feature_name(&self.record, feature),
                    None => qualifier.clone(),
                };
                format!("/{qualifier} dropped from {name}")
            })
            .collect();

        self.rebuild_view(buffer, removed, dropped)?;
        // 9g §4 rule 6: an edit is never just an edit, it is an edit with the
        // step that puts it back. Both go onto the group the command now being
        // carried collects, and the command's last key closes the group. The
        // step goes on only once `rebuild_view` has said the whole edit stands:
        // a record the rebuild refuses keeps no group as it kept the buffer.
        self.current_group.push((edit, undo));
        Ok(outcome)
    }

    /// The steps an edit's aftermath takes, shared by `apply_edit`, `undo` and
    /// `redo` (9g §4 rule 7): `rebuild` gives the record the buffer's bases
    /// and features, and everything after it reads the record again — the
    /// selection and its `Chosen` map, the strip's GC line, the ORFs, the last
    /// search's hits, the cursor and the layout. `removed` and `dropped` hold
    /// the names the message is to name them by, and are empty for an undo.
    fn rebuild_view(
        &mut self,
        buffer: EditBuffer,
        removed: Vec<String>,
        dropped: Vec<String>,
    ) -> Result<(), DomainError> {
        // The selected CDS is followed by the line its feature's key was read
        // from, which no edit moves (§4 rule 2): a removed feature shifts the
        // index of every other one in `features` and in `cdss`, and its own name
        // is needed for the message before the record is replaced.
        let selected_line: Option<u64> = self
            .selected
            .and_then(|index| self.record.cdss.get(index))
            .and_then(|cds| self.record.features.get(cds.feature))
            .map(|feature| feature.line);
        let selected_name: Option<String> =
            self.selected.map(|index| display_name(&self.record, index));

        let rebuilt: ViewRecord = match rebuild(&buffer, &self.opened) {
            Ok(record) => record,
            Err(error) => {
                self.message = Some(error.message());
                return Err(error);
            }
        };
        self.record = rebuilt;
        self.edit = Ok(buffer);

        // §4 rule 2: the CDS the view selected is found again by its feature's
        // line and checked again exactly as on open (M9-6 i) — kept if it is
        // still there and `Usable`, dropped with core's reason if not.
        let mut words: Vec<String> = Vec::new();
        let found: Option<usize> = selected_line.and_then(|line| {
            let feature: usize = self
                .record
                .features
                .iter()
                .position(|feature| feature.line == line)?;
            self.record
                .cdss
                .iter()
                .position(|cds| cds.feature == feature)
        });
        if selected_line.is_some() {
            match found.filter(|index| usable(&self.record, *index).is_some()) {
                Some(index) => {
                    let previous: Option<usize> = self.selected;
                    self.selected = Some(index);
                    // The lanes still read the same CDS, under its new index.
                    if let LaneMode::Cds(old) = self.mode {
                        if previous == Some(old) {
                            self.mode = LaneMode::Cds(index);
                        }
                    }
                }
                None => {
                    let name: String = selected_name.unwrap_or_default();
                    let refusal: Option<String> = found
                        .and_then(|index| self.record.cdss.get(index))
                        .and_then(|cds| match &cds.status {
                            CdsStatus::Unusable(error) => Some(error.message()),
                            CdsStatus::Usable(_cds) => None,
                        });
                    words.push(match refusal {
                        Some(reason) => format!("CDS {name} is no longer usable: {reason}"),
                        None => format!("CDS {name} was removed by the edit"),
                    });
                    self.selected = None;
                    self.mode = LaneMode::Off;
                }
            }
        }
        self.choose();

        // §4 rule 4: the strip's third line counts the bases the width shows,
        // and the edit has just changed them.
        self.gc = gc_line(&self.record, inner_width(self.width));

        // §4 rule 5: ORFs scanned once for the record's old bases are scanned
        // again for its new ones; `None` says they were never scanned and stays.
        if self.orfs.is_some() {
            self.orfs = None;
            self.compute_orfs();
        }

        // §4 rule 7: a record shortened by the edit cannot hold such a cursor.
        let length: u64 = self.record.sequence.length();
        if length > 0 && self.cursor >= length {
            self.cursor = length - 1;
        }

        // §4 rule 6: the last search is re-run over the record as edited. The
        // hits are re-marked only: an edit never moves the cursor and leaves
        // the message alone, so a search that suddenly finds nothing or refuses
        // is silent here (`run_search`, which parks the cursor on the current
        // hit, is not what an edit does).
        if let Some((mode, pattern)) = self.last_search.clone() {
            let cds: Option<&Cds> = self.selected.and_then(|index| usable(&self.record, index));
            match search_record(
                &self.record,
                cds,
                self.circular,
                mode,
                &pattern,
                self.cursor,
            ) {
                Outcome::Found(hits) => self.hits = Some(hits),
                Outcome::Empty | Outcome::Refused(_) => self.hits = None,
            }
        }

        // §4 rule 3: the names of what the edit removed follow any CDS sentence.
        if !removed.is_empty() {
            words.push(format!("removed: {}", removed.join(", ")));
        }
        words.extend(dropped);
        if !words.is_empty() {
            self.message = Some(words.join(" · "));
        }

        // §4 rule 7: the record is laid out again, anchored on the cursor's block.
        self.relayout(true);
        Ok(())
    }

    /// Open a list over the record with this filter and the first row
    /// highlighted: `f` the features, `c` the CDSs, and 8k2 calls this with the
    /// go-to text (8k3 §4). Its rows are every item with an empty filter and the
    /// ones the shared matcher answers with one (rule 2), and while it is open
    /// its rows fill the body and its keys are the ones `handle` routes to it
    /// (rules 3 and 7). Nothing happens below `MIN_WIDTH` by `MIN_HEIGHT`, where
    /// the view draws nothing at all (§4 rule 10).
    pub fn open_list(&mut self, kind: ListKind, filter: &str) {
        if !big_enough(self.width, self.height) {
            return;
        }
        self.list = Some(List::open(&self.record, kind, filter));
    }

    /// The open list, `None` when no list is (8k3 §4).
    pub fn list(&self) -> Option<&List> {
        self.list.as_ref()
    }

    /// The text typed at the go-to prompt, `None` while it is closed (8k2 §4
    /// rule 2): `g` opens it empty, `enter` acts on the line and closes it, and
    /// `esc` closes it and changes nothing.
    pub fn prompt(&self) -> Option<&str> {
        self.prompt.as_deref()
    }

    /// The open search prompt, `None` when it is closed (8l1 §4): `/` opens one
    /// with nothing typed, `enter` closes it and searches, and `esc` closes it and
    /// leaves the hits as they were.
    pub fn search(&self) -> Option<&SearchPrompt> {
        self.search.as_ref()
    }

    /// The hits marked on the screen now, `None` when nothing is (8l1 §4 rules 3
    /// and 4): what `super::screen` paints yellow, what the status line counts, and
    /// what `n` and `N` walk through.
    pub fn hits(&self) -> Option<&Hits> {
        self.hits.as_ref()
    }

    /// The bases the selection covers, `None` with none (M9-2, M9-6 g): the
    /// zero-based half-open `[min(anchor, cursor), max(anchor, cursor) + 1)` in
    /// `Visual`. The range is always plain, so a selection never crosses the
    /// origin of a circular record: when the cursor wraps past the end the
    /// selection is the range between the two bases.
    pub(super) fn selection(&self) -> Option<(u64, u64)> {
        match self.editing {
            EditMode::Visual { anchor } => {
                let first: u64 = anchor.min(self.cursor);
                let last: u64 = anchor.max(self.cursor);
                Some((first, last.saturating_add(1)))
            }
            // `Normal` has no selection; the typing modes keep their bases aside
            // (`replace_selection` is what `r` reads; 9f §4 rule 4).
            EditMode::Normal
            | EditMode::Insert { .. }
            | EditMode::Overtype
            | EditMode::ReplaceOne => None,
        }
    }

    pub fn record(&self) -> &ViewRecord {
        &self.record
    }

    /// The selected CDS's index into `record.cdss`, `None` for none (M8-11).
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn mode(&self) -> &LaneMode {
        &self.mode
    }

    /// Whether the mode reads the bottom strand (M8-36, contract 8m14 §4
    /// rule 2): the mode is `Cds(i)` with CDS `i` on the minus strand, so the
    /// screen puts the cursor's cell and its codon's underline on the row of the
    /// complements. Every other mode — no translation, a plus-strand CDS, the
    /// three frames, the six — and an unusable CDS, which has no lines at all,
    /// read the top row.
    pub fn reads_bottom(&self) -> bool {
        match self.mode {
            LaneMode::Cds(index) => {
                self.record
                    .cdss
                    .get(index)
                    .is_some_and(|view| match &view.status {
                        CdsStatus::Usable(cds) => cds.strand() == Strand::Minus,
                        CdsStatus::Unusable(_reason) => false,
                    })
            }
            LaneMode::Off | LaneMode::ThreeFrames | LaneMode::SixFrames => false,
        }
    }

    /// The topology the view moves on: `T` flips it only for a record whose
    /// topology was assumed (D049 M8-3).
    pub fn circular(&self) -> bool {
        self.circular
    }

    /// Whether the overview strip is drawn: the three lines at the top of the
    /// frame's inner area that `o` hides and shows (8l2 §4 rule 1, 8m12 §4 rule 4).
    /// While a list is open they are not drawn, but this is what `o` left them at.
    pub fn overview(&self) -> bool {
        self.overview
    }

    /// The strip's third line as the width leaves it: the GC height of every strip
    /// column, held so that drawing never counts bases again (8m12 §4 rule 5).
    /// `super::screen` puts the frame's two cells around it.
    pub fn gc_line(&self) -> &str {
        &self.gc
    }

    /// The level the feature lanes are drawn at, as `F` left it (8m6 §4 rule 3):
    /// `All` when the view opens.
    pub fn features(&self) -> FeatureLevel {
        self.features
    }

    /// Whether the feature lanes are drawn: the lines `F` hides and shows (8m3 §4
    /// rule 1), true at every level but `None` (8m6 §4 rule 3).
    pub fn lanes_shown(&self) -> bool {
        self.features != FeatureLevel::None
    }

    /// Whether the ORF lines are drawn, as `O` left them (M8-40, contract 8m18 §4
    /// rule 6): false at open, when the view has shown no `O`.
    pub fn orf_lanes(&self) -> bool {
        self.orf_lanes
    }

    /// The ORFs the drawn lines hold, `None` while the record has not been scanned
    /// for them (M8-40, contract 8m18 §4 rule 6): `None` at open, and `Some` from
    /// the first `O` until `T` drops them for the new topology.
    pub fn orfs(&self) -> Option<&[Orf]> {
        self.orfs.as_deref()
    }

    /// Whether the residue lines are coloured by Zappo group (8m5 §4 rule 4):
    /// the backgrounds `C` turns off and on, on at open unless `NO_COLOR` says
    /// otherwise, and what `block_lines` draws them for.
    pub fn colours(&self) -> bool {
        self.colours
    }

    /// Whether the residues are coloured, as the session's environment left it
    /// (8m5 §4 rule 5): `super::terminal` opens the view with this off when
    /// `NO_COLOR` says so.
    pub fn set_colours(&mut self, on: bool) {
        self.colours = on;
    }

    /// Whether the key list is open (8m3 §4 rule 2): `?` opens it and any key
    /// closes it.
    pub fn help(&self) -> bool {
        self.help
    }

    /// The cursor's zero-based base (D006).
    pub fn cursor(&self) -> u64 {
        self.cursor
    }

    /// The blocks as laid out for the current width and mode, `None` below
    /// `MIN_WIDTH` by `MIN_HEIGHT` (§4 rule 1).
    pub fn layout(&self) -> Option<&Layout> {
        self.layout.as_ref()
    }

    /// The blocks drawn: from the top block, the longest run of whole blocks
    /// whose lines and gaps fit the frame's body rows, and always at least one
    /// block (§4 rule 1). Empty when no layout is drawn.
    pub fn visible(&self) -> std::ops::Range<usize> {
        let blocks: &Vec<(u64, u64)> = match self.layout.as_ref() {
            Some(built) => &built.blocks,
            None => return 0..0,
        };
        if self.top >= blocks.len() {
            return 0..0;
        }
        self.top..(self.top + self.run_from(self.top, self.block_rows()))
    }

    /// How many blocks from `top` fit in `room` rows, counted as §4 rule 1 does:
    /// the first one of the run whatever it costs, and every one after it with the
    /// gap before it counted. `visible` measures the drawn run this way, and
    /// `opens_with_strip` counts what the body holds with the strip off.
    fn run_from(&self, top: usize, room: usize) -> usize {
        let blocks: &Vec<(u64, u64)> = match self.layout.as_ref() {
            Some(built) => &built.blocks,
            None => return 0,
        };
        let mut rows: usize = 0;
        let mut count: usize = 0;
        for block in top..blocks.len() {
            // The gap between two blocks is counted before the block below it,
            // never after the last one drawn, and the first block of the run is
            // taken whatever it costs, so a block taller than the screen is
            // still drawn (§4 rule 1).
            let need: usize = self.line_count(block) + usize::from(GAP) * usize::from(count > 0);
            if count > 0 && rows + need > room {
                break;
            }
            rows += need;
            count += 1;
        }
        count
    }

    /// The rows a run of blocks has: the body rows, less the overview's three
    /// whenever `o` has the strip on (8l2 §4 rule 2, 8m12 §4 rule 4).
    /// `super::screen` fills exactly this many rows below the strip. A list is
    /// drawn instead of the blocks and takes all of `body_rows`, as 8k3 gives it,
    /// but this stays measured with the strip: rows change while a list is open —
    /// `enter` on a CDS and a `resize` both relayout under it — and the run chosen
    /// then has to be the one the screen will draw once the list is gone, so the
    /// cursor's block is never left off screen.
    fn block_rows(&self) -> usize {
        let strip: usize = usize::from(OVERVIEW_ROWS) * usize::from(self.overview);
        body_rows(self.height).saturating_sub(strip)
    }

    /// Whether the strip opens drawn (8l2 §4 rule 1): more than twice as many
    /// blocks as the body rows hold with the strip's three rows left out. Below
    /// `MIN_WIDTH` by `MIN_HEIGHT` no layout is built, and there is nothing to say.
    fn opens_with_strip(&self) -> bool {
        match self.layout.as_ref() {
            Some(built) => built.blocks.len() > 2 * self.run_from(0, body_rows(self.height)),
            None => false,
        }
    }

    /// The line a `T` or a refusal left in the status line, cleared by the next
    /// key (§4 rule 13).
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// The title bar's text (§4 rule 11): the record's name, its length, the
    /// topology as the file states it or as the view assumed it (M8-3), and the
    /// selected CDS's display name, strand and 1-based span — or that no CDS is
    /// selected, when the record has usable ones and M8-11 selected none.
    pub(super) fn title(&self) -> String {
        let head: String = format!(
            "{} · {} bp · {}",
            self.record.sequence.name().value(),
            self.record.sequence.length(),
            self.topology_words()
        );
        match self.cds_words() {
            Some(words) => format!("{head} · {words}"),
            // A record with no usable CDS to select has nothing to name here:
            // the CDS list 8k3 opens will say why.
            None => head,
        }
    }

    /// The status line's text (§4 rule 13): the 1-based base the cursor stands
    /// on, the codon and its residue when that base lies in one of the selected
    /// CDS's codons, and the names of the drawn features that hold it — or the
    /// message, when one is set. While a list is open its filter is what the line
    /// says, and a refusal replaces it (8k3 §4 rule 7); while go-to is open it
    /// says what has been typed there (8k2 §4 rule 2).
    pub(super) fn status(&self) -> String {
        match self.editing {
            // §4 rule 5 (M9-2): while a selection is open the line names the
            // mode, the bases it covers and their range, before the usual text.
            EditMode::Visual { .. } => format!("{} · {}", self.visual_words(), self.status_body()),
            // 9f §4 rule 6: the three modes that take text lead the line with
            // their own marker, then what the line usually says.
            EditMode::Insert { .. } => format!("-- INSERT -- · {}", self.status_body()),
            EditMode::Overtype => format!("-- OVERTYPE -- · {}", self.status_body()),
            EditMode::ReplaceOne => format!("-- REPLACE -- · {}", self.status_body()),
            EditMode::Normal => self.status_body(),
        }
    }

    /// The `-- VISUAL --` part of the status line (§4 rule 5): the mode, how
    /// many bases the selection covers and their 1-based inclusive range.
    fn visual_words(&self) -> String {
        let (first, last): (u64, u64) = match self.selection() {
            Some(range) => range,
            // The mode is `Visual` only while there is a selection, so this is
            // unreachable; a short line keeps the status well-formed anyway.
            None => return "-- VISUAL --".to_string(),
        };
        format!(
            "-- VISUAL -- {} bases, {}..{}",
            last - first,
            first + 1,
            last
        )
    }

    /// What the line says otherwise: the cursor, the message, the prompts or the
    /// hits (§4 rule 13).
    fn status_body(&self) -> String {
        if let Some(list) = self.list.as_ref() {
            return match &self.message {
                Some(words) => words.clone(),
                // A filter typed is shown as typed; with none, the line says what
                // is listed instead of leaving the line blank.
                _ if list.filter().is_empty() => format!("{} rows", list.rows().len()),
                _ => format!("filter {}", list.filter()),
            };
        }
        if let Some(text) = self.prompt.as_ref() {
            return match &self.message {
                Some(words) => words.clone(),
                // The text typed at go-to is shown as typed; with nothing typed
                // yet the line says what the prompt is for instead of leaving it
                // blank (8k2 §4 rule 2).
                _ if text.is_empty() => "go to".to_string(),
                _ => format!("go to {text}"),
            };
        }
        if let Some(prompt) = self.search.as_ref() {
            return match &self.message {
                Some(words) => words.clone(),
                // The mode badge and the text typed, as the prompt's badge has to be
                // unmistakable (8l1 §4 rule 2, proposal §4). With nothing typed yet
                // the line says what switches the mode rather than naming an empty
                // pattern.
                _ if prompt.text().is_empty() => {
                    format!("search {}  tab switches", prompt.mode().badge())
                }
                _ => format!("search {} {}", prompt.mode().badge(), prompt.text()),
            };
        }
        let mut parts: Vec<String> = vec![format!("base {}", self.cursor + 1)];
        if let Some(codon) = self.cursor_codon() {
            let residue: String = match (codon.letter, codon.number) {
                (Some(letter), Some(number)) => format!("{letter}{number}"),
                // A terminal stop codon is a codon and no residue (D010).
                _ => "stop".to_string(),
            };
            parts.push(format!(
                "{} {} {}..{}",
                residue, codon.letters, codon.first, codon.last
            ));
        }
        match &self.message {
            Some(words) => parts.push(words.clone()),
            None => {
                // While hits are marked, the line says which of the shown ones is
                // current, what core names it by and where it lies — and when core
                // found more than the view shows, how many there really are (8l1 §4
                // rule 7, M8-12).
                if let Some(hits) = self.hits.as_ref() {
                    let hit: &Hit = hits.current_hit();
                    parts.push(format!(
                        "hit {} of {} {} {}",
                        hits.current() + 1,
                        hits.shown().len(),
                        hit.label,
                        hit.span(self.record.sequence.length())
                    ));
                    if hits.total() > hits.shown().len() {
                        parts.push(format!(
                            "showing the first {} of {} hits — refine your pattern",
                            hits.shown().len(),
                            hits.total()
                        ));
                    }
                }
                let names: Vec<String> = self.features_at();
                if !names.is_empty() {
                    parts.push(names.join(" "));
                }
            }
        }
        parts.join(" · ")
    }

    /// The key line's text, row `height - 1` (§4 rule 1): every key this
    /// contract binds and no other, so the line never offers a key that does
    /// nothing (8h adds the keys it dispatches). While a list is open, the keys
    /// are its own (8k3 §4 rule 7), and while go-to is open they are the
    /// prompt's (8k2 §4 rule 2). The words are short because the line has the
    /// screen's width and no more, and at `MIN_WIDTH` by `MIN_HEIGHT` that is 80
    /// columns.
    pub(super) fn keys(&self) -> String {
        match self.list.as_ref() {
            Some(list) => match list.kind() {
                ListKind::Features => {
                    "type filter  backspace  up down PgUp PgDn Home End  enter go  esc close"
                        .to_string()
                }
                ListKind::Cdss => {
                    "type  backspace  up down PgUp PgDn Home End  enter select  esc close"
                        .to_string()
                }
            },
            None if self.prompt.is_some() => "type  backspace  enter go  esc close".to_string(),
            // The search prompt's keys (8l1 §4 rule 9): `tab` is the one that
            // switches the badge, and `enter` is what searches.
            None if self.search.is_some() => {
                "type  tab mode  backspace  enter search  esc close".to_string()
            }
            // The key list has no keys to offer, because what it offers is the keys
            // (8m3 §4 rule 2).
            None if self.help => "any key closes".to_string(),
            // The row is the screen's width and no more: 72 columns at `MIN_WIDTH`
            // by `MIN_HEIGHT`, inside the 80 that
            // `the_key_line_fits_eighty_columns` measures (8m3 §4 rule 3, M8-26).
            // `hjkl`, `PgUp PgDn`, `Home End`, `f`, `c`, `g`, `/`, `n N`, `o` and
            // `?` name themselves and go without a word, and `t frames`, `F lanes`
            // and `q quit` are the three that do not. `T` is not offered because it
            // changes nothing unless the topology was assumed, and the key list
            // leaves it out the same way (M8-25).
            None => "hjkl PgUp PgDn Home End  t frames  F lanes  f c g  / n N  ? keys  q quit"
                .to_string(),
        }
    }

    /// One block's lines as `layout` gives them, with the cursor and the codon's
    /// underline left for `super::screen` to add (§4 rule 12). A block whose
    /// lines `layout` refuses draws nothing. While the level is `None` no lane
    /// line and no `+N more` line is here (8m6 §4 rule 2).
    pub(super) fn lines(&self, block: usize) -> Vec<(LineRole, Line<'static>)> {
        self.drawn_lines(block)
    }

    /// The rows of the open list that `super::screen` draws in the body: the
    /// window holding the highlight, one entry per body row it fills, each with
    /// its text and its two marks (8k3 §4 rules 6 and 7). Empty when no list is
    /// open, when it has no rows, or below `MIN_WIDTH` by `MIN_HEIGHT`.
    pub(super) fn list_rows(&self) -> Vec<Drawn> {
        match self.list.as_ref() {
            Some(list) if big_enough(self.width, self.height) => {
                list.drawn(&self.record, body_rows(self.height))
            }
            _ => Vec::new(),
        }
    }

    /// The key list's rows, one per key the open screen has and in the order the
    /// list shows them (8m3 §4 rule 2): the key's own text, two spaces, then what
    /// the key does — each row written from the arm of `handle` that key reaches,
    /// so the `t` row starts `cycle`, the `F` row starts `cycle` as well, the
    /// `T` row `switch`. `T` has a row only where the topology was assumed, which
    /// is the only case in which it changes
    /// anything (M8-25). Empty unless the list is open: the body has 20 rows at
    /// `MIN_WIDTH` by `MIN_HEIGHT`, more than the 18 the longest list needs.
    pub(super) fn help_rows(&self) -> Vec<String> {
        if !self.help {
            return Vec::new();
        }
        let mut rows: Vec<&str> = vec![
            "hjkl  move the cursor one base left or right, one line up or down",
            "PgUp PgDn  move the cursor a screen up or down",
            "ctrl-d ctrl-u  move the cursor half a screen down or up",
            "Home End  move the cursor to the record's first base and last base",
            "w b  move the cursor to the next codon and the codon before it",
            "t  cycle what the lanes show: the CDS, three frames, six frames, off",
            "F  cycle the feature lanes: all, CDSs only, none",
            "C  colour the residues and the features, and back",
        ];
        if self.record.topology == TopologySource::Assumed {
            rows.push("T  switch the topology the view assumed for this record");
        }
        rows.extend_from_slice(&[
            "f  list the record's features",
            "c  list the CDSs and select one of them",
            "g  go to a base, a residue or a name",
            "/  search the record for bases or a protein",
            "n N  step through the hits the screen marks",
            "esc  clear the hits the screen marks",
            "o  hide the overview strip and show it again",
            "O  show the open reading frames and hide them",
            "?  this list of every key the screen has",
            "q  quit",
        ]);
        rows.into_iter().map(str::to_string).collect()
    }

    /// Where the cursor stands in the selected CDS, `None` when no CDS is
    /// selected or the base is one that CDS has no codon for (D050): which cells
    /// `screen` underlines, and what the status line says about them (§4 rules
    /// 12 and 13).
    pub(super) fn cursor_codon(&self) -> Option<CursorCodon> {
        let chosen: &Chosen = self.chosen.as_ref()?;
        let place: CodonPlace = chosen.map.base_residue(self.cursor)?;
        // The codon's other two bases lie on the side its codon is read towards
        // (D040): a plus-strand codon runs to higher coordinates, a minus-strand
        // one to lower ones. `base_residue` has already said the cursor stands
        // inside a complete codon of this CDS, and a codon of a CDS is a codon
        // of the record, so a codon that crosses the origin wraps by what the
        // record states (D036) rather than by what `T` has set for the cursor.
        let read: i64 = if chosen.strand == Strand::Plus { 1 } else { -1 };
        // The codon's first base in reading order: `place.place` is 1 for the
        // first base of the codon, so the cursor sits `place.place - 1` bases
        // into it (§4 rule 12).
        let before: i64 = match i64::try_from(place.place - 1) {
            Ok(bases) => bases,
            // A place outside a codon's three bases is not a codon base at all:
            // nothing is marked rather than a guessed span.
            Err(_error) => return None,
        };
        let start: i64 = self.cursor as i64 - before * read;
        let mut positions: [u64; 3] = [0; 3];
        let mut letters: Vec<char> = Vec::new();
        for (index, base_cell) in positions.iter_mut().enumerate() {
            let base: u64 = self.around_record(start + i64::try_from(index).unwrap_or(0) * read)?;
            let letter: char = match self.record.sequence.letter_at(base) {
                Ok(letter) if chosen.strand == Strand::Plus => letter,
                // Read on its own strand: the complement, in reading order, so
                // the status line shows the codon as the CDS reads it and not
                // the reference letters (§4 rule 13).
                Ok(letter) => complement_letter(letter)?,
                Err(_error) => return None,
            };
            *base_cell = base;
            letters.push(letter);
        }
        let letter: Option<char> = match place.residue {
            Some(number) => chosen.letters.get(number as usize - 1).copied(),
            None => None,
        };
        // The span the status line gives is in reference order (§4 rule 13): the
        // order D036 keeps a joined location's parts in, and the order
        // `cds_span` prints `fP`'s location as `56..11` in. `positions` is in
        // reading order, so the span's ends are its ends on the strand the codon
        // is read on: a plus-strand codon starts at its lowest base and runs up,
        // a minus-strand one starts at its highest and runs down. min/max gets
        // that wrong for a codon running across the origin of a circular record:
        // `fP`'s third codon, the 1-based bases 62, 1 and 2, would be `1..62`
        // instead of `62..2` (`tests/fixtures/view/frames-62.gb`).
        let (first, last): (u64, u64) = if chosen.strand == Strand::Plus {
            (positions[0], positions[2])
        } else {
            (positions[2], positions[0])
        };
        Some(CursorCodon {
            positions,
            letters: letters.iter().collect(),
            first: first + 1,
            last: last + 1,
            number: place.residue,
            letter,
        })
    }

    /// The display names, in file order, of the drawn features other than
    /// `source` whose location holds the cursor (§4 rule 13). A feature the
    /// record cannot draw states no location that could hold anything (D053),
    /// and `source` describes the whole record, so naming it would say nothing
    /// about where the cursor is.
    pub(super) fn features_at(&self) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for feature in self.record.features.iter() {
            if feature.key == "source" {
                continue;
            }
            let location: &DisplayLocation = match &feature.shape {
                FeatureShape::Drawn(shape) => shape,
                FeatureShape::NotDrawn(_refusal) => continue,
            };
            let holds: bool = location
                .location
                .intervals()
                .iter()
                .any(|(first, last)| *first <= self.cursor && self.cursor < *last);
            if holds {
                names.push(feature.name.clone());
            }
        }
        names
    }

    /// The topology as the title words it (§4 rule 11, D049 M8-3, M8-34 rule 2):
    /// the badge that goes before the words — `⟳` for a circular record, `—` for a
    /// linear one — and what the file states, or the assumption a FASTA record
    /// carries, named as an assumption until `T` sets it aside for the session.
    fn topology_words(&self) -> String {
        let assumed: bool = self.record.topology == TopologySource::Assumed;
        let badge: char = if self.circular { '⟳' } else { '—' };
        match (self.circular, assumed) {
            (true, true) => format!("{badge} circular (set with T)"),
            (true, false) => format!("{badge} circular"),
            (false, true) => format!("{badge} linear (assumed)"),
            (false, false) => format!("{badge} linear"),
        }
    }

    /// The selected CDS as the title words it: its display name (D053), the
    /// strand its location states (D009) and the 1-based span `cds_span` reads,
    /// which is what `luria find` prints (D036 order). With nothing selected,
    /// `no CDS selected` when the record has a usable CDS to select (M8-11) and
    /// nothing at all when it has none.
    fn cds_words(&self) -> Option<String> {
        let index: usize = match self.selected {
            Some(index) => index,
            None => {
                return if usable_cdss(&self.record).is_empty() {
                    None
                } else {
                    Some("no CDS selected".to_string())
                }
            }
        };
        let display: String = display_name(&self.record, index);
        let cds: &Cds = usable(&self.record, index)?;
        match cds_span(&self.record, index) {
            Some((first, last)) => Some(format!(
                "{} ({}) {first}..{last}",
                display,
                cds.strand().symbol()
            )),
            // A CDS whose location this record cannot draw states no span and no
            // strand, and the title invents neither (D005, D053).
            None => Some(format!("{display} (not drawn)")),
        }
    }

    /// The map and residues of the selected CDS, taken once per selection (§2's
    /// table): `translate` once and one `CdsMap` built from that translation, as
    /// `CdsMap::new` requires (D042). A CDS that cannot be translated — which
    /// `import_cds` has already refused, so it cannot happen for a selected CDS
    /// — leaves the state with no codon answers rather than guesses.
    fn choose(&mut self) {
        let built: Option<Chosen> = match self.selected {
            Some(index) => match usable(&self.record, index) {
                Some(cds) => match translate(&self.record.sequence, cds) {
                    Ok(made) => Some(Chosen {
                        map: CdsMap::new(cds, &made),
                        letters: made.residues_text().chars().collect(),
                        strand: cds.strand(),
                    }),
                    Err(_error) => None,
                },
                None => None,
            },
            None => None,
        };
        self.chosen = built;
    }

    /// How many rows one block takes on the screen: the lines `block_lines`
    /// gives it for the current mode, counted as they are counted there and
    /// without drawing them (8r3 §4 item 1), gaps aside — the blocks are drawn
    /// inside one frame, not fenced by their own border (§4 rule 1). A block
    /// whose lines are refused takes none, the same as `lines` gives it, so a
    /// run is never measured shorter than it draws. The reading the counting
    /// skips, and the drawing keeps, is the one `held_cds` hands over.
    fn line_count(&self, block: usize) -> usize {
        match self.layout.as_ref() {
            Some(built) => block_line_count_with(
                &self.record,
                built,
                &self.mode,
                block,
                &self.drawing(),
                self.held_cds(),
            )
            .unwrap_or_default(),
            None => 0,
        }
    }

    /// The reading the state holds for the CDS it selected, and only where the
    /// current mode names that same CDS: what `choose` translated once (D042).
    /// `drawn_lines` draws from it and `line_count` counts with it, so neither
    /// the drawing nor the counting of a block translates the whole CDS again
    /// (8r3 §4 item 2). A mode that names another CDS, or none, or a selection
    /// that failed, gets `None`, and those read it for themselves.
    fn held_cds(&self) -> Option<(&CdsMap, &[char])> {
        match (self.mode, self.selected, self.chosen.as_ref()) {
            (LaneMode::Cds(index), Some(selected), Some(chosen)) if index == selected => {
                Some((&chosen.map, chosen.letters.as_slice()))
            }
            _ => None,
        }
    }

    /// How the state's keys have left the lines to be drawn (8m3 §4 rule 1,
    /// 8m6 §4 rule 3, 8m18 §4 rule 1): the one `Drawing` a block's lines and the
    /// rows they take are both counted from.
    fn drawing(&self) -> Drawing<'_> {
        Drawing {
            colours: self.colours,
            features: self.features,
            orfs: if self.orf_lanes {
                self.orfs.as_deref().unwrap_or(&[])
            } else {
                &[]
            },
        }
    }

    /// The one place a block's lines come from, for the drawing and for the rows
    /// counted (8m3 §4 rule 1, 8m6 §4 rule 3): `layout`'s lines for the current
    /// mode, drawn with the state's own colours and feature level. `block_lines`
    /// is what leaves out the lane lines and the `+N more` line, so `lines` draws
    /// what is here and `line_count` counts it, and the rows `visible` and
    /// `follow` measure are the rows drawn. A block whose lines `layout` refuses
    /// has none, for both (8k1 §4 rule 7).
    fn drawn_lines(&self, block: usize) -> Vec<(LineRole, Line<'static>)> {
        match self.layout.as_ref() {
            Some(built) => block_lines_with(
                &self.record,
                built,
                &self.mode,
                block,
                &self.drawing(),
                self.held_cds(),
            )
            .unwrap_or_default(),
            None => Vec::new(),
        }
    }

    /// The index of the block holding the zero-based base `base`. The blocks
    /// tile the record in order, so the first one ending past `base` is the one
    /// to look at, and there is nothing to look at when no layout is drawn.
    fn block_of(&self, base: u64) -> Option<usize> {
        let blocks: &Vec<(u64, u64)> = &self.layout.as_ref()?.blocks;
        let found: usize = blocks.partition_point(|(_first, last)| *last <= base);
        match blocks.get(found) {
            Some((first, last)) if *first <= base && base < *last => Some(found),
            _ => None,
        }
    }

    /// The base `steps` away from `base` for a codon of the record: wrapping
    /// over the origin by what the record's own topology states (D036), and
    /// `None` past either end of a linear record. `around_record` is the
    /// record's arithmetic; a cursor move follows the session's topology and
    /// uses `around_session`.
    fn around_record(&self, base: i64) -> Option<u64> {
        self.wrapping(
            base,
            self.record.sequence.topology() == SequenceTopology::Circular,
        )
    }

    /// The base `steps` away from the cursor: wrapping past either end only when
    /// the view moves on a circular topology, which `T` sets for a record whose
    /// topology was assumed (§4 rule 3, D049 M8-3).
    fn around_session(&self, base: i64) -> Option<u64> {
        self.wrapping(base, self.circular)
    }

    /// One base of this record, `base` reduced into it when `wraps` and refused
    /// outside it when it does not. Zero-based coordinates, as everywhere inside
    /// the program (D006).
    fn wrapping(&self, base: i64, wraps: bool) -> Option<u64> {
        let length: i64 = self.record.sequence.length() as i64;
        if length == 0 {
            return None;
        }
        let value: i64 = if wraps { base.rem_euclid(length) } else { base };
        if value < 0 || value >= length {
            None
        } else {
            Some(value as u64)
        }
    }

    /// `l` and `h`, and the arrows: one base, wrapping past either end only when
    /// the view moves on a circular record (§4 rule 3).
    fn step_base(&mut self, steps: i64) {
        if let Some(base) = self.around_session(self.cursor as i64 + steps) {
            self.cursor = base;
        }
    }

    /// `j` and `k`: the same offset from the start of the next or previous block,
    /// clamped to that block's last base; at the first or the last block the
    /// cursor stays (§4 rule 4). A short last block is what clamping is for:
    /// pBR322's last block holds 5 bases (zero-based 4356..4361), so the line
    /// from zero-based 4310 lands on 4360, its last base, and not past the end of
    /// the record.
    fn step_line(&mut self, direction: i64) {
        if let Some(base) = self.line_target(direction) {
            self.cursor = base;
        }
    }

    /// Where `j` or `k` lands: `None`, meaning stay, when there is no block on
    /// that side.
    fn line_target(&self, direction: i64) -> Option<u64> {
        let built: &Layout = self.layout.as_ref()?;
        let block: usize = self.block_of(self.cursor)?;
        let (start, _end): (u64, u64) = *built.blocks.get(block)?;
        let target: usize = if direction > 0 {
            block + 1
        } else {
            block.checked_sub(1)?
        };
        let (first, last): (u64, u64) = *built.blocks.get(target)?;
        let offset: u64 = (self.cursor - start).min(last - first - 1);
        Some(first + offset)
    }

    /// The block move `page` and `half_page` share (8r3 §4 item 3): the
    /// cursor's block moved by `step` blocks in `direction` and clamped at the
    /// first and the last block, and the cursor moved to the same offset in the
    /// target with that offset clamped to the target's last base. `None` is
    /// where either key returns early today: no layout drawn, no block in it,
    /// or the cursor standing in no block. The target block is what each key
    /// moves the top of the screen towards.
    fn move_by_blocks(&mut self, step: usize, direction: i64) -> Option<usize> {
        let built: &Layout = self.layout.as_ref()?;
        let last: usize = built.blocks.len().checked_sub(1)?;
        let block: usize = self.block_of(self.cursor)?;
        let (start, _end): (u64, u64) = built.blocks[block];
        let target: usize = if direction > 0 {
            (block + step).min(last)
        } else {
            block.saturating_sub(step)
        };
        let (first, last_base): (u64, u64) = built.blocks[target];
        self.cursor = first + (self.cursor - start).min(last_base - first - 1);
        Some(target)
    }

    /// `PageDown` and `PageUp`: the same line move by one screen of blocks,
    /// clamped at either end, and the cursor's block goes on top (§4 rule 4).
    fn page(&mut self, direction: i64) {
        let screen: usize = self.visible().len().max(1);
        let target: usize = match self.move_by_blocks(screen, direction) {
            Some(target) => target,
            None => return,
        };
        self.top = target;
    }

    /// `ctrl-d` and `ctrl-u`: the same line move by half a screen of blocks,
    /// and the top block moves by the same distance (§4 rule 2, M8-38). The
    /// half is `max(1, ⌊visible().len() / 2⌋)`, taken from the blocks drawn
    /// before anything moves, so the key always moves and never off the screen.
    /// Clamped at the first and the last block like `page`, with no wrap even
    /// on a circular record; the cursor keeps its offset in the line and is
    /// clamped to the target block's last base, and the scroll then follows the
    /// cursor as §4 rule 6 has it.
    fn half_page(&mut self, direction: i64) {
        let half: usize = (self.visible().len() / 2).max(1);
        if self.move_by_blocks(half, direction).is_none() {
            return;
        }
        // The same last block `move_by_blocks` clamped the cursor's block at,
        // clamping `top` here (§4 rule 2, M8-38). A layout with no block got
        // `None` there and returned before this.
        let last: usize = match self
            .layout
            .as_ref()
            .and_then(|built| built.blocks.len().checked_sub(1))
        {
            Some(last) => last,
            None => return,
        };
        self.top = if direction > 0 {
            (self.top + half).min(last)
        } else {
            self.top.saturating_sub(half)
        };
        self.follow();
    }

    /// `w` and `b`: the nearest base 1 to 3 bases away that is a codon's first
    /// base in reading direction of the selected CDS, wrapping when the view
    /// moves on a circular record; the cursor stays when there is no such base
    /// or no CDS is selected (M8-35, 8m13 §4 rule 2).
    fn step_codon(&mut self, direction: i64) {
        let chosen: &Chosen = match self.chosen.as_ref() {
            Some(found) => found,
            None => return,
        };
        for distance in 1..=CODON_SEARCH {
            let steps: i64 = direction * i64::from(u32::try_from(distance).unwrap_or(u32::MAX));
            let base: u64 = match self.around_session(self.cursor as i64 + steps) {
                Some(found) => found,
                // Past either end of a record the view does not wrap: nothing
                // further in that direction can be a codon's first base.
                None => break,
            };
            let first: bool = match chosen.map.base_residue(base) {
                Some(place) => place.place == 1,
                None => false,
            };
            if first {
                self.cursor = base;
                return;
            }
        }
    }

    /// `t`: the lanes cycle `off`, the selected CDS, three frames, six frames,
    /// skipping the CDS's own lane when no CDS is selected, and the cursor stays
    /// (§4 rule 7). A mode `layout` refuses leaves the lanes off and says why in
    /// the status line.
    fn cycle(&mut self) {
        let next: LaneMode = match (self.mode, self.selected) {
            (LaneMode::Off, Some(index)) => LaneMode::Cds(index),
            (LaneMode::Off, None) => LaneMode::ThreeFrames,
            (LaneMode::Cds(_), _) => LaneMode::ThreeFrames,
            (LaneMode::ThreeFrames, _) => LaneMode::SixFrames,
            (LaneMode::SixFrames, _) => LaneMode::Off,
        };
        let anchor: Option<u64> = self.anchor();
        match layout(&self.record, inner_width(self.width), &next) {
            Ok(built) => {
                self.mode = next;
                self.layout = Some(built);
            }
            Err(problem) => {
                self.message = Some(problem_words(&problem));
                self.mode = LaneMode::Off;
                self.layout = layout(&self.record, inner_width(self.width), &LaneMode::Off).ok();
            }
        }
        self.set_top(anchor);
        self.follow();
    }

    /// `F`: the three feature levels in order — all, CDSs only, none, then all
    /// again (8m6 §4 rule 3) — and the status line names the level reached. The
    /// cursor stays.
    fn step_features(&mut self) {
        self.features = match self.features {
            FeatureLevel::All => FeatureLevel::Cdss,
            FeatureLevel::Cdss => FeatureLevel::None,
            FeatureLevel::None => FeatureLevel::All,
        };
        self.message = Some(format!("features: {}", feature_words(self.features)));
    }

    /// `O` (M8-40, contract 8m18 §4 rule 6): show the ORF lines and hide them
    /// again, with the status line naming which the key did. The first `O` that
    /// shows them asks the record for its ORFs and keeps them, and every toggle
    /// after that draws what is kept, so one scan serves any number of presses.
    fn toggle_orf_lanes(&mut self) {
        self.orf_lanes = !self.orf_lanes;
        if self.orf_lanes {
            self.compute_orfs();
        }
        self.message = Some(orfs_words(
            self.orf_lanes,
            self.orfs.as_ref().map_or(0, |orfs| orfs.len()),
        ));
    }

    /// The ORFs for the topology the view reads, scanned once and kept (contract
    /// 8m18 §4 rule 6): `find_orfs` at `ORF_MIN_RESIDUES`, on the record's own
    /// text and the topology `T` has set. This is the only point at which the
    /// sequence is scanned — nothing runs here when the lines are hidden, when
    /// they are opened, or while a block is drawn.
    fn compute_orfs(&mut self) {
        if self.orfs.is_none() {
            self.orfs = Some(find_orfs(
                &self.record.sequence,
                self.circular,
                ORF_MIN_RESIDUES,
            ));
        }
    }

    /// `T` (D049 M8-3, §4 rule 8): a FASTA record's assumed topology flips for
    /// the session, and the cursor keys wrap from then on; a record whose file
    /// states its own topology never changes, and the status line says why.
    fn flip(&mut self) {
        if self.record.topology == TopologySource::Assumed {
            self.circular = !self.circular;
            // The hits were found on the topology that has just been switched off
            // and say nothing about the other one (8l1 §4 rule 6). Where the file
            // states its own topology nothing was switched, and the hits stay.
            self.hits = None;
            // The ORFs were kept from the topology that has just been switched
            // off, and are wrong for the other one (contract 8m18 §4 rule 6). They
            // go here, and the next `O` that shows them scans the record again —
            // at once if the lines are drawn now, later if `O` shows them.
            self.orfs = None;
            if self.orf_lanes {
                self.compute_orfs();
                self.message = Some(orfs_words(
                    true,
                    self.orfs.as_ref().map_or(0, |orfs| orfs.len()),
                ));
            }
        } else {
            self.message = Some("the file states its own topology".to_string());
        }
        self.relayout(true);
    }

    /// The first base of the block on top, which the next layout has to put on
    /// top again (§4 rule 6).
    fn anchor(&self) -> Option<u64> {
        let blocks: &Vec<(u64, u64)> = &self.layout.as_ref()?.blocks;
        blocks.get(self.top).map(|(first, _last)| *first)
    }

    /// Lay the record out again for the current width and mode. `anchored` keeps
    /// the top block at the base it covered before, which §4 rule 6 asks after
    /// `t`, `T` and a resize, and then scrolls to the cursor; opening lays the
    /// record out from its first block, which is already on top.
    fn relayout(&mut self, anchored: bool) {
        let anchor: Option<u64> = if anchored { self.anchor() } else { None };
        if big_enough(self.width, self.height) {
            self.layout = layout(&self.record, inner_width(self.width), &self.mode).ok();
        } else {
            self.layout = None;
        }
        self.set_top(anchor);
        if anchored {
            self.follow();
        }
    }

    /// Put the top block back at the anchor base, then keep the index inside the
    /// blocks the new layout has.
    fn set_top(&mut self, anchor: Option<u64>) {
        if let Some(base) = anchor {
            if let Some(block) = self.block_of(base) {
                self.top = block;
            }
        }
        match self.layout.as_ref() {
            Some(built) => {
                if let Some(last) = built.blocks.len().checked_sub(1) {
                    self.top = self.top.min(last);
                }
            }
            None => self.top = 0,
        }
    }

    /// The scroll follows the cursor (§4 rule 6): a cursor above what is drawn
    /// puts its block on top; a cursor below it moves the top as little as the
    /// rows allow, so the drawn run still reaches the cursor's block.
    fn follow(&mut self) {
        let block: usize = match self.block_of(self.cursor) {
            Some(found) => found,
            None => return,
        };
        if block < self.top {
            self.top = block;
            return;
        }
        if block < self.visible().end {
            return;
        }
        let room: usize = self.block_rows();
        let mut rows: usize = self.line_count(block);
        let mut top: usize = block;
        while top > 0 {
            let need: usize = self.line_count(top - 1) + usize::from(GAP);
            if rows + need > room {
                break;
            }
            rows += need;
            top -= 1;
        }
        self.top = top;
    }

    /// A key pressed while a list is open (8k3 §4 rule 3): every printable key is
    /// filter text, `q` and `h` to `l` included, because a filter is typed with
    /// the letters the record's names use; `backspace` shortens it; `up` `down`
    /// `pageup` `pagedown` `home` `end` move the highlight; `esc` closes the list
    /// and changes nothing else; `enter` acts on the highlighted row. `ctrl-c`
    /// has already left in `handle`, and a press with no name here does nothing.
    fn list_key(&mut self, key: Key) {
        match key {
            Key::Char(ch) => {
                if let Some(list) = self.list.as_mut() {
                    list.type_char(&self.record, ch);
                }
            }
            Key::Backspace => {
                if let Some(list) = self.list.as_mut() {
                    list.backspace(&self.record);
                }
            }
            Key::Up => {
                if let Some(list) = self.list.as_mut() {
                    list.step_row(-1);
                }
            }
            Key::Down => {
                if let Some(list) = self.list.as_mut() {
                    list.step_row(1);
                }
            }
            // The page is the body the list draws in, the same number of rows
            // `visible` measures a run of blocks against (§4 rule 1).
            Key::PageUp => {
                let page: usize = body_rows(self.height);
                if let Some(list) = self.list.as_mut() {
                    list.page_row(page, -1);
                }
            }
            Key::PageDown => {
                let page: usize = body_rows(self.height);
                if let Some(list) = self.list.as_mut() {
                    list.page_row(page, 1);
                }
            }
            // `ctrl-d` and `ctrl-u` move the highlight half that page, and never
            // nil rows: the page is at least one row, so the key always moves it
            // (§4 rule 3, M8-38).
            Key::CtrlD => {
                let half: usize = body_rows(self.height) / 2;
                if let Some(list) = self.list.as_mut() {
                    list.page_row(half.max(1), 1);
                }
            }
            Key::CtrlU => {
                let half: usize = body_rows(self.height) / 2;
                if let Some(list) = self.list.as_mut() {
                    list.page_row(half.max(1), -1);
                }
            }
            Key::Home => {
                if let Some(list) = self.list.as_mut() {
                    list.go_first();
                }
            }
            Key::End => {
                if let Some(list) = self.list.as_mut() {
                    list.go_last();
                }
            }
            Key::Enter => self.act_on_row(),
            // `esc` closes: what was typed in the filter and what was highlighted
            // leave with the list, and no cursor, selection or scroll moves. A
            // residue go-to left to be picked leaves too, so a later CDS pick
            // follows M8-21 rather than M8-22 (M8-22 rule 3).
            Key::Esc => {
                self.list = None;
                self.pending = None;
            }
            // `left` and `right` have no rule while a list is open (8k3 §4 rule 3
            // names the keys that move the highlight), so they change nothing — as
            // does any other press, `ctrl-c` having left in `handle`.
            Key::Left | Key::Right | Key::Tab | Key::Other | Key::CtrlC | Key::CtrlR => {}
        }
    }

    /// `enter` on the highlighted row (8k3 §4 rules 4 and 5). With no highlight —
    /// a filter that named nothing — there is no row to act on.
    fn act_on_row(&mut self) {
        let found: Option<(ListKind, usize)> = match self.list.as_ref() {
            Some(list) => list.item().map(|item| (list.kind(), item)),
            None => None,
        };
        match found {
            Some((ListKind::Features, feature)) => self.enter_feature(feature),
            Some((ListKind::Cdss, cds)) => self.enter_cds(cds),
            None => {}
        }
    }

    /// `enter` on a feature row (8k3 §4 rule 4, M8-21). A feature the display
    /// parser refuses keeps the cursor where it is, keeps the list open, and leaves
    /// the refusal's code in the status line (plan §4.1, D053). A feature whose 5′
    /// end is not stated — a `<` on a plus-strand one, a `>` on a minus-strand one
    /// — cannot be jumped to either, because there is no first base to stand on
    /// (plan §5.3). Otherwise the cursor goes to that 5′ end: the first base of the
    /// first interval on the plus strand, the last base of the last one on the minus
    /// strand, the intervals in D036 order, zero-based as everything inside the
    /// program is (D006, D036). The list closes and the scroll follows (8k1 §4 rule
    /// 6). When the feature is a CDS's feature, a usable CDS is selected with it
    /// (M8-11) and an unusable one is not, with the code that refuses it shown
    /// beside the jump (D042, D052).
    fn enter_feature(&mut self, index: usize) {
        // The record is read here and nowhere else in this move: what it says is
        // taken out before the cursor, the selection and the scroll change.
        let (target, refusal): (Option<u64>, Option<String>) = match self.record.features.get(index)
        {
            Some(feature) => match &feature.shape {
                FeatureShape::NotDrawn(error) => (None, Some(refusal_code(error))),
                FeatureShape::Drawn(shape) if five_prime_is_unstated(shape) => {
                    (None, Some("the start is not stated".to_string()))
                }
                FeatureShape::Drawn(shape) => match five_prime_end(shape) {
                    Some(first) => (Some(first), None),
                    // A drawn shape with no interval has no base to stand on;
                    // the line says so rather than inventing one.
                    None => (None, Some("the start is not stated".to_string())),
                },
            },
            None => return,
        };
        // A `GTG` start, a missing id: the feature is jumped to and its CDS is not
        // selected, and the reason is the code core gave (D042, D052).
        let cds: Option<usize> = self
            .record
            .cdss
            .iter()
            .position(|view| view.feature == index);
        let code: Option<String> = match cds.and_then(|which| self.record.cdss.get(which)) {
            Some(view) => match &view.status {
                CdsStatus::Usable(_found) => None,
                CdsStatus::Unusable(error) => Some(refusal_code(error)),
            },
            None => None,
        };

        let base: u64 = match target {
            Some(base) => base,
            None => {
                self.message = refusal;
                return;
            }
        };
        self.cursor = base;
        if let Some(which) = cds {
            // Only a usable CDS is ever selected, and `select_cds` is the one rule
            // that says which those are (D041, M8-11).
            self.select_cds(Some(which));
        }
        self.message = code;
        self.list = None;
        self.follow();
    }

    /// `enter` on a CDS row (8k3 §4 rule 5, M8-21): a usable CDS is selected, the
    /// list closes, and the cursor goes to the first base of its residue 1's codon
    /// — its start codon, in reading direction — after the list has closed, with the
    /// scroll following (8m1 §4 rule 1). A residue go-to asked for with no CDS
    /// selected is answered by that residue's codon first base instead, and when
    /// this CDS has no such residue the cursor goes to the start codon and the
    /// status line says why (M8-22 rule 3). An unusable one selects nothing, keeps
    /// the list open, and leaves its code in the status line (D042, D052).
    fn enter_cds(&mut self, index: usize) {
        let status: Option<&CdsStatus> = self.record.cdss.get(index).map(|view| &view.status);
        match status {
            Some(CdsStatus::Usable(_found)) => {}
            Some(CdsStatus::Unusable(error)) => {
                self.message = Some(refusal_code(error));
                return;
            }
            None => return,
        }
        // The residue is answered here either way, and is not kept for a later pick
        // whether or not this CDS has it (M8-22 rule 3).
        let residue: Option<(char, u64)> = self.pending.take();
        self.select_cds(Some(index));
        self.list = None;
        // M8-22 rule 3: the residue go-to asked for, as this CDS has it or refuses
        // it; and M8-21: residue 1's first base, which a refused residue falls back
        // on. `select_cds` accepted a usable CDS, so the map and its residue 1 are
        // there; the refusals say otherwise if they ever are not.
        let asked: Option<Result<u64, String>> =
            residue.map(|(letter, number)| match self.chosen.as_ref() {
                Some(chosen) => residue_first(chosen, letter, number),
                None => Err("no CDS is selected".to_string()),
            });
        let start: Result<u64, String> = match self.chosen.as_ref() {
            Some(chosen) => match chosen.map.residue_bases(1) {
                Ok(codon) => Ok(codon[0]),
                Err(error) => Err(error.message()),
            },
            None => Err("no CDS is selected".to_string()),
        };
        let (target, refusal): (Option<u64>, Option<String>) = match (asked, start) {
            (Some(Ok(base)), _start) => (Some(base), None),
            (Some(Err(words)), Ok(base)) => (Some(base), Some(words)),
            (Some(Err(words)), Err(_start)) => (None, Some(words)),
            (None, Ok(base)) => (Some(base), None),
            (None, Err(words)) => (None, Some(words)),
        };
        if let Some(base) = target {
            self.cursor = base;
        }
        self.message = refusal;
        self.follow();
    }

    /// The go-to prompt's keys (8k2 §4 rule 2): every printable key is text, `q`
    /// included; `backspace` drops the last character typed; `esc` closes the
    /// prompt and changes nothing else; `enter` closes it and acts on the whole
    /// line through `go_to`. `ctrl-c` has already left in `handle`, and a press
    /// with no name here does nothing.
    fn prompt_key(&mut self, key: Key) {
        match key {
            Key::Char(ch) => {
                if let Some(text) = self.prompt.as_mut() {
                    text.push(ch);
                }
            }
            Key::Backspace => {
                if let Some(text) = self.prompt.as_mut() {
                    // The line is cut by the last character's own width, so a
                    // multi-byte character is dropped whole.
                    if let Some(last) = text.chars().next_back() {
                        text.truncate(text.len() - last.len_utf8());
                    }
                }
            }
            Key::Enter => {
                // `enter` closes the prompt whatever the line says, so a refusal
                // leaves the screen showing the record and not a half-typed line.
                let text: String = self.prompt.take().unwrap_or_default();
                self.go_to(&text);
            }
            Key::Esc => self.prompt = None,
            // `ctrl-d` and `ctrl-u` are not text and move nothing here: at a
            // prompt the cursor and the line typed stay as they are (§4 rule 1).
            Key::CtrlD
            | Key::CtrlU
            | Key::Left
            | Key::Right
            | Key::Up
            | Key::Down
            | Key::PageUp
            | Key::PageDown
            | Key::Home
            | Key::End
            | Key::Tab
            | Key::Other
            | Key::CtrlC
            | Key::CtrlR => {}
        }
    }

    /// A key pressed while the search prompt is open (8l1 §4 rule 2): every
    /// printable key is text — `q`, `n` and `/` included, because a pattern is
    /// typed with those letters — `backspace` shortens the text, `tab` flips the
    /// mode, `esc` closes the prompt and leaves the hits as they were, and `enter`
    /// closes it and searches. `ctrl-c` has already left in `handle`, and a press
    /// with no name here does nothing.
    fn search_key(&mut self, key: Key) {
        match key {
            Key::Char(ch) => {
                if let Some(prompt) = self.search.as_mut() {
                    prompt.type_char(ch);
                }
            }
            Key::Backspace => {
                if let Some(prompt) = self.search.as_mut() {
                    prompt.backspace();
                }
            }
            Key::Tab => {
                if let Some(prompt) = self.search.as_mut() {
                    prompt.press_tab();
                }
            }
            Key::Enter => {
                // `enter` closes the prompt whatever the text says, so a refusal
                // leaves the screen showing the record and not a half-typed line —
                // as it does at go-to (8k2 §4 rule 2).
                let prompt: SearchPrompt = match self.search.take() {
                    Some(prompt) => prompt,
                    None => return,
                };
                self.run_search(prompt.mode(), prompt.text());
            }
            Key::Esc => self.search = None,
            // `ctrl-d` and `ctrl-u` are not text and move nothing here: at the
            // prompt the cursor and the pattern typed stay as they are (§4 rule 1).
            Key::CtrlD
            | Key::CtrlU
            | Key::Left
            | Key::Right
            | Key::Up
            | Key::Down
            | Key::PageUp
            | Key::PageDown
            | Key::Home
            | Key::End
            | Key::Other
            | Key::CtrlC
            | Key::CtrlR => {}
        }
    }

    /// What `enter` at the search prompt does (8l1 §4 rule 3, plan §5.4): with an
    /// empty pattern nothing at all changes — the hits and the cursor stay as they
    /// are and no error is made — and otherwise the answer is core's. A refusal
    /// carries core's own message; a refusal and finding nothing both clear what was
    /// marked, and neither moves the cursor.
    fn run_search(&mut self, mode: SearchMode, text: &str) {
        if text.is_empty() {
            return;
        }
        // 9e §4 rule 6: what `enter` searches is kept, so every edit can mark
        // the same pattern again on the record as edited.
        self.last_search = Some((mode, text.to_string()));
        let cds: Option<&Cds> = self.selected.and_then(|index| usable(&self.record, index));
        match search_record(&self.record, cds, self.circular, mode, text, self.cursor) {
            Outcome::Refused(message) => {
                self.hits = None;
                self.message = Some(message);
            }
            Outcome::Empty => {
                self.hits = None;
                self.message = Some(format!("no hits for {text}"));
            }
            Outcome::Found(hits) => {
                // The current hit is the one the cursor goes to, and the scroll
                // follows it there (8l1 §4 rule 4).
                let start: u64 = hits.current_hit().start;
                self.cursor = start;
                self.hits = Some(hits);
                self.follow();
            }
        }
    }

    /// `n` and `N` with no prompt or list open (8l1 §4 rule 5): the current hit
    /// moves to the next or previous one shown, wrapping inside what is shown, and
    /// the cursor goes to its start with the scroll following. With nothing marked
    /// there is nothing to walk to, and nothing moves.
    fn step_hit(&mut self, forward: bool) -> Action {
        if let Some(hits) = self.hits.as_mut() {
            let start: u64 = hits.step(forward).start;
            self.cursor = start;
            self.follow();
        }
        Action::Continue
    }

    /// What one go-to line does, once `super::goto` has said which of its three
    /// shapes it has (8k2 §4 rules 3 to 5): a base moves the cursor, a residue
    /// moves it to that residue's codon in the selected CDS, and a name goes to
    /// the feature it names. A blank line changes nothing.
    fn go_to(&mut self, text: &str) {
        match parse_target(text) {
            Target::Nothing => {}
            Target::Base(number) => self.goto_base(number),
            Target::Residue { letter, number } => self.goto_residue(letter, number),
            Target::Name(name) => self.goto_name(&name),
        }
    }

    /// A base as D057 reads a bare number: the cursor stands on the 1-based base
    /// typed, the numbering the status line shows, whatever CDS is selected, and
    /// the scroll follows (8k1 §4 rule 6). A number with no base under it — `0`,
    /// one past the record's last base, or one too large for any record (D048)
    /// — keeps the cursor where it is and says what the record has (8k2 §4 rule
    /// 3).
    fn goto_base(&mut self, number: u64) {
        let length: u64 = self.record.sequence.length();
        if number == 0 || number > length {
            self.message = Some(format!("the record has bases 1 to {length}"));
            return;
        }
        self.cursor = number - 1;
        self.follow();
    }

    /// A residue of the selected CDS (8k2 §4 rule 4, M8-35): the cursor stands
    /// on the first base of its codon in reading direction, as `w` and `b` leave
    /// it (8m13 §4 rule 2), and the scroll follows; the residue letter stays over
    /// the codon's middle base (D055). Its refusals — a number outside
    /// `1..=residues()`, residue `residues + 1` being the terminal stop, which has a
    /// codon and no residue (D010), and a letter that is not this residue's own
    /// (D057)) — are what `residue_first` answers. With no CDS selected, M8-22 rule
    /// 3 takes over: the residue is remembered and the CDS picker is opened to
    /// choose one; a record with no CDS at all opens nothing and says so (8m1 §4
    /// rule 4).
    fn goto_residue(&mut self, letter: char, number: u64) {
        let chosen: &Chosen = match self.chosen.as_ref() {
            Some(chosen) => chosen,
            // M8-22 rule 3 and 8m1 §4 rule 4: no CDS is selected, so the residue
            // goes looking for one.
            None => {
                if self.record.cdss.is_empty() {
                    self.message = Some("this record has no CDS".to_string());
                } else {
                    // The picker opens with no filter, so every CDS the record has
                    // can be picked, usable or not.
                    self.pending = Some((letter, number));
                    self.open_list(ListKind::Cdss, "");
                }
                return;
            }
        };
        match residue_first(chosen, letter, number) {
            Ok(base) => {
                self.cursor = base;
                self.follow();
            }
            Err(words) => self.message = Some(words),
        }
    }

    /// A feature name, matched by core's shared matcher over the names 8k3's
    /// feature list uses (8k2 §4 rule 5, D056). One match is jumped to exactly as
    /// `enter` on that row jumps to it — the same jump, the same CDS selection and
    /// the same refusals of a feature that is not drawn or whose start is not
    /// stated (plan §5.3). Several open the feature list filtered to the text
    /// typed, and the cursor stays for the list's own keys to move. Nothing
    /// matches: the cursor stays and the status line says so.
    fn goto_name(&mut self, name: &str) {
        let named: Vec<NamedItem> = feature_names(&self.record);
        match match_name(&named, name) {
            Matched::One(found) => self.enter_feature(found.item),
            Matched::Several(_found) => self.open_list(ListKind::Features, name),
            Matched::Nothing => {
                self.message = Some(format!("no feature is named {name}"));
            }
        }
    }
}

/// The words an editing key leaves when the record cannot be edited (M9-12,
/// §4 rule 4): the feature's name, key and file line, then core's own reason.
fn blocked_words(blocked: &EditBlocked) -> String {
    format!(
        "cannot edit {} ({} on line {}): {}",
        blocked.name,
        blocked.key,
        blocked.line,
        blocked.error.message()
    )
}

/// The first base one edit touches, in the coordinates the edit was made in
/// (9g §4 rule 7): where `u` and `ctrl-r` leave the cursor once a whole group
/// is put back or applied again. An insert's first base is the point it
/// inserts at, which the cursor then clamps.
fn edit_start(edit: &Edit) -> u64 {
    match edit {
        Edit::Insert { at, .. } => *at,
        Edit::Delete { start, .. }
        | Edit::Replace { start, .. }
        | Edit::Substitute { start, .. } => *start,
    }
}

/// Which CDS M8-11 selects on open (D048): exactly one usable CDS is it; several
/// usable CDSs select none, and the title says so; none selects nothing.
fn opened(record: &ViewRecord) -> Option<usize> {
    let found: Vec<usize> = usable_cdss(record);
    match found.len() {
        1 => found.first().copied(),
        _ => None,
    }
}

/// The CDS `index` as a usable one, `None` when the index names no CDS or
/// `import_cds` refused it (D041, D042, D052).
fn usable(record: &ViewRecord, index: usize) -> Option<&Cds> {
    match record.cdss.get(index) {
        Some(view) => match &view.status {
            CdsStatus::Usable(cds) => Some(cds),
            CdsStatus::Unusable(_refusal) => None,
        },
        None => None,
    }
}

/// Every usable CDS's index, in file order: what 8k3's list offers, and here the
/// question of whether the record has one to offer at all.
fn usable_cdss(record: &ViewRecord) -> Vec<usize> {
    (0..record.cdss.len())
        .filter(|index| usable(record, *index).is_some())
        .collect()
}

/// A CDS's display name (D053), which is the name of the feature it came from.
fn display_name(record: &ViewRecord, index: usize) -> String {
    match record.cdss.get(index) {
        Some(view) => match record.features.get(view.feature) {
            Some(feature) => feature.name.clone(),
            None => String::new(),
        },
        None => String::new(),
    }
}

/// The name the status line names a buffered feature by (M9-6 c, M9-13 a,
/// contract 9e §4 rule 3): the name the feature list showed it by (D053),
/// found through the file line that the buffer and the record both keep
/// (`EditFeature::line`); when that does not hold, the feature's own key.
fn feature_name(record: &ViewRecord, feature: &EditFeature) -> String {
    match record
        .features
        .iter()
        .position(|found| found.line == feature.line)
    {
        Some(index) => record.features[index].name.clone(),
        None => feature.key.clone(),
    }
}

/// A drawn feature's 5′ end, where `enter` on its row jumps the cursor (M8-21,
/// D006, D036): the first base of the first interval on the plus strand, and the
/// last base of the last one on the minus strand. The intervals are in D036 order,
/// which is the same for either strand, and `None` for a shape with no interval.
fn five_prime_end(shape: &DisplayLocation) -> Option<u64> {
    match shape.location.strand() {
        Strand::Plus => shape
            .location
            .intervals()
            .first()
            .map(|(first, _last)| *first),
        Strand::Minus => shape
            .location
            .intervals()
            .last()
            .and_then(|(_first, last)| last.checked_sub(1)),
    }
}

/// Whether a drawn feature's 5′ end is not stated (M8-21, D053): a `<` before the
/// first interval on the plus strand, a `>` after the last one on the minus strand.
/// The other marker marks the far end and says nothing about this one. It is
/// `pub(super)` because `super::list::refused` asks the same rule (M8-21, D053,
/// contract 8r1 item 1).
pub(super) fn five_prime_is_unstated(shape: &DisplayLocation) -> bool {
    match shape.location.strand() {
        Strand::Plus => shape.open_before.is_some(),
        Strand::Minus => shape.open_after.is_some(),
    }
}

/// The residue rule of a selected CDS (8k2 §4 rule 4, D010, D057, M8-35): the
/// first base of its codon in reading direction, which is the codon's highest
/// base on the minus strand, and otherwise the words the status line says — a
/// number outside `1..=residues()`, or a letter that is not this residue's own.
fn residue_first(chosen: &Chosen, letter: char, number: u64) -> Result<u64, String> {
    let count: u64 = chosen.map.residues();
    if number == 0 || number > count {
        return Err(format!("the selected CDS has residues 1 to {count}"));
    }
    // The residues are held in reading order, so residue `number` is the
    // `number - 1`th letter, and the range above keeps that index inside them.
    match chosen.letters.get(number as usize - 1).copied() {
        Some(found) if found != letter => Err(format!("residue {number} is {found}, not {letter}")),
        Some(_found) => chosen
            .map
            .residue_bases(number)
            .map(|codon| codon[0])
            .map_err(|error| error.message()),
        // The map's count and the translation have the same length, so this is
        // reached only if they ever disagree; either way there is no residue here
        // to go to.
        None => Err(format!("the selected CDS has residues 1 to {count}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luria_core::identity::Identifier;
    use luria_core::view_record::open_record;
    use std::fs;
    use std::path::Path;

    /// One fixture record, as `open_record` reads it — the same helper as
    /// `layout.rs`'s tests: the content decides the format, never the name
    /// (D049).
    fn record(path: &str, name: &str) -> ViewRecord {
        let text: String = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
            .unwrap_or_else(|error| panic!("read {path}: {error}"));
        open_record(Identifier::create(name.to_string()).unwrap(), text)
            .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
    }

    fn pbr() -> ViewRecord {
        record("../../tests/fixtures/genbank/raw/J01749.1.gb", "SYNPBR322")
    }

    fn lin() -> ViewRecord {
        record("../../tests/fixtures/genbank/synthetic-linear.gb", "synlin")
    }

    /// The 16 bp FASTA of contract §5: no features, no CDSs, and an assumed
    /// topology (D049).
    fn w16() -> ViewRecord {
        open_record(
            Identifier::create("w".to_string()).unwrap(),
            ">w\nTTCAAAAAAAAAAGAA\n".to_string(),
        )
        .expect("the 16 bp FASTA record")
    }

    // M8-11 (D048), §4 rule 2: a lone usable CDS is selected on open and several
    // are not. `opened` answers before any mode exists.
    #[test]
    fn m8_11_selects_only_a_lone_usable_cds() {
        // pBR322: `tet` (0) and `bla` (2) are usable, ROP (1) is not — so two
        // usable CDSs, and none is selected.
        assert_eq!(usable_cdss(&pbr()), vec![0, 2], "tet and bla");
        assert_eq!(opened(&pbr()), None, "two usable CDSs select none");

        // One usable CDS is selected, and its own lane is drawn on open.
        assert_eq!(opened(&lin()), Some(0));
        let state: ViewState = ViewState::new(lin(), 80, 24);
        assert_eq!(state.selected(), Some(0));
        assert_eq!(state.mode(), &LaneMode::Cds(0));

        // Nothing to select says nothing in the title, and the lanes are off.
        let state: ViewState = ViewState::new(w16(), 80, 24);
        assert_eq!(state.selected(), None);
        assert_eq!(state.mode(), &LaneMode::Off);
        assert!(!state.title().contains("CDS"), "title: {}", state.title());
    }

    // §4 rule 11 and D049 M8-3: the topology words the title carries, and the
    // span `cds_span` prints.
    #[test]
    fn the_title_names_the_topology_the_view_moves_on() {
        // pBR322's LOCUS line states circular: worded plainly, with no
        // assumption and nothing to set.
        let state: ViewState = ViewState::new(pbr(), 80, 24);
        assert!(
            state.title().contains("4361 bp · ⟳ circular"),
            "title: {}",
            state.title()
        );
        assert!(
            !state.title().contains("assumed"),
            "title: {}",
            state.title()
        );
        assert!(
            state.title().contains("no CDS selected"),
            "title: {}",
            state.title()
        );

        // A GenBank record that states linear says `linear`, and its one CDS is
        // named with the strand its location states (D009) and the span `find`
        // prints (D036 order).
        let state: ViewState = ViewState::new(lin(), 80, 24);
        assert!(
            state.title().contains("393 bp · — linear"),
            "title: {}",
            state.title()
        );
        assert!(
            !state.title().contains("assumed"),
            "title: {}",
            state.title()
        );
        assert!(
            state.title().contains("synA (+) 43..345"),
            "title: {}",
            state.title()
        );
    }

    // §4 rule 13: the message a key leaves is the last part of the status line,
    // and the next key clears it.
    #[test]
    fn the_message_replaces_the_features_until_the_next_key() {
        let mut state: ViewState = ViewState::new(pbr(), 80, 24);
        // Base 1 (zero-based 0) lies in no CDS — M8-11 selected none — and the
        // only features holding it are the two `source` features, which the
        // status line leaves out. So the line says where the cursor is, and
        // nothing else.
        assert_eq!(state.status(), "base 1");
        assert_eq!(state.handle(Key::Char('T')), Action::Continue);
        assert_eq!(state.status(), "base 1 · the file states its own topology");

        // Any key clears it, and the line is back to what it was.
        assert_eq!(state.handle(Key::Char('x')), Action::Continue);
        assert_eq!(state.message(), None);
        assert_eq!(state.status(), "base 1");
    }

    // §4 rule 13 and D040: the codon in the status line is read on its own
    // strand, and its span is in reference order.
    #[test]
    fn a_codon_is_read_on_its_own_strand() {
        let mut state: ViewState = ViewState::new(pbr(), 80, 24);
        assert!(state.select_cds(Some(2)), "bla is usable");
        state.cursor = 4151;
        let codon: CursorCodon = state
            .cursor_codon()
            .expect("zero-based 4151 is bla's codon 1 middle base");
        assert_eq!(codon.letters, "ATG", "read on the minus strand");
        assert_eq!(codon.positions, [4152, 4151, 4150], "reading order (D040)");
        assert_eq!((codon.first, codon.last), (4151, 4153), "reference order");
        assert_eq!((codon.number, codon.letter), (Some(1), Some('M')));
        // The reference letters in reference order are CAT; the status line
        // shows the codon as the CDS reads it, then the features holding the
        // base in file order. J01749.1.gb's table gives `mobile_element
        // 3148..4361`, `gene`, `CDS` and `sig_peptide` all `/gene="bla"` over
        // zero-based 4151 — and `source`, which the line leaves out.
        assert_eq!(
            state.status(),
            "base 4152 · M1 ATG 4151..4153 · mobile_element bla bla bla"
        );
    }

    // §4 rule 1: below 80 × 24 there is no layout and no block is drawn; a
    // resize back to the minimum lays the record out again.
    #[test]
    fn no_layout_below_the_minimum() {
        let mut state: ViewState = ViewState::new(pbr(), 79, 24);
        assert!(state.layout().is_none());
        assert_eq!(state.visible(), 0..0);
        state.resize(80, 24);
        let visible: std::ops::Range<usize> = state.visible();
        assert!(state.layout().is_some());
        assert_eq!(visible.start, 0);
        assert!(!visible.is_empty(), "at least one block: {visible:?}");
        state.resize(120, 23);
        assert!(state.layout().is_none());
        assert_eq!(state.visible(), 0..0);
    }

    // §4 rules 4 and 6: the block on top moves with the cursor, and the cursor's
    // block is always inside the drawn run.
    #[test]
    fn the_top_block_follows_the_cursor() {
        let mut state: ViewState = ViewState::new(pbr(), 80, 24);
        assert_eq!(
            state.visible().start,
            0,
            "the first block is on top at open"
        );
        for _ in 0..400 {
            state.handle(Key::Char('l'));
        }
        let block: usize = state
            .block_of(state.cursor)
            .expect("the cursor stands in a block");
        let visible: std::ops::Range<usize> = state.visible();
        assert!(visible.start > 0, "the top has moved: {visible:?}");
        assert!(
            visible.start <= block && block < visible.end,
            "cursor block {block}, visible {visible:?}"
        );
    }

    // 9d §4 rule 1 and rule 3 (M9-2): `v` anchors a selection at the cursor, and
    // the movement keys extend it while the cursor moves.
    #[test]
    fn v_selects_from_the_cursor() {
        let mut state: ViewState = ViewState::new(pbr(), 80, 24);
        state.cursor = 9;
        assert_eq!(state.selection(), None, "no selection yet in Normal");
        assert_eq!(state.handle(Key::Char('v')), Action::Continue);
        assert_eq!(state.selection(), Some((9, 10)), "the cursor's own base");
        for _ in 0..3 {
            assert_eq!(state.handle(Key::Char('l')), Action::Continue);
        }
        assert_eq!(state.cursor(), 12);
        assert_eq!(state.selection(), Some((9, 13)), "half-open [9, 13)");
        // `h` walks the cursor back through the anchor, and the range stays a
        // plain one on either side of it (§4 rule 1).
        for _ in 0..5 {
            state.handle(Key::Char('h'));
        }
        assert_eq!(state.cursor(), 7);
        assert_eq!(state.selection(), Some((7, 10)), "half-open [7, 10)");
        for _ in 0..2 {
            state.handle(Key::Char('h'));
        }
        assert_eq!(state.cursor(), 5);
        assert_eq!(state.selection(), Some((5, 10)), "half-open [5, 10)");
        assert_eq!(state.handle(Key::Esc), Action::Continue);
        assert_eq!(state.selection(), None);
        assert_eq!(
            state.cursor(),
            5,
            "closing selects nothing and moves nothing"
        );
    }

    // 9d §4 rule 2 (M9-2): an `esc` that ends a selection is not the `esc` that
    // clears the hits the screen marks.
    #[test]
    fn esc_ending_a_selection_keeps_hits() {
        let mut state: ViewState = ViewState::new(pbr(), 80, 24);
        // A search that finds hits (8l1 §4 rules 1 and 3).
        state.handle(Key::Char('/'));
        for ch in ['A', 'T', 'G'] {
            state.handle(Key::Char(ch));
        }
        state.handle(Key::Enter);
        assert!(state.hits().is_some(), "ATG has hits in pBR322");
        assert_eq!(state.handle(Key::Char('v')), Action::Continue);
        assert!(state.selection().is_some(), "v selects");
        assert_eq!(state.handle(Key::Esc), Action::Continue);
        assert_eq!(state.selection(), None, "the selection has gone");
        assert!(state.hits().is_some(), "the marks stay for n and N to walk");
    }

    // 9d §4 rule 3: the keys that move the cursor move it as in Normal, and the
    // selection follows; opening and closing a list keeps it.
    #[test]
    fn selection_survives_goto_and_lists() {
        let mut state: ViewState = ViewState::new(pbr(), 80, 24);
        assert_eq!(state.handle(Key::Char('v')), Action::Continue);
        state.handle(Key::Char('g'));
        for ch in ['2', '0', '0'] {
            state.handle(Key::Char(ch));
        }
        state.handle(Key::Enter);
        assert_eq!(state.cursor(), 199, "go-to 200 (8k2 §4 rule 3)");
        assert_eq!(state.selection(), Some((0, 200)), "half-open [0, 200)");
        state.handle(Key::Char('f'));
        assert!(state.list().is_some(), "f opens the feature list");
        assert_eq!(state.selection(), Some((0, 200)), "while the list is open");
        state.handle(Key::Esc);
        assert!(state.list().is_none(), "esc closed the list");
        assert_eq!(state.selection(), Some((0, 200)), "and kept the selection");
    }

    // 9d §4 rule 4 and M9-12: a record whose feature 9b2 refuses (synthetic.gb
    // holds `order(1..6,7..12)` on line 61) leaves every editing key inert.
    #[test]
    fn blocked_record_refuses_edit_keys() {
        let path: &str = "../../tests/fixtures/genbank/synthetic.gb";
        let mut state: ViewState = ViewState::new(record(path, "synthetic"), 80, 24);
        for ch in "viarRxdypPus".chars() {
            assert_eq!(state.handle(Key::Char(ch)), Action::Continue, "{ch}");
            assert_eq!(state.selection(), None, "{ch} selects nothing");
        }
        let message: Option<String> = state.message().map(str::to_string);
        let message: String = message.expect("the key leaves a message");
        assert!(message.contains("ord"), "names the feature: {message}");
        assert!(message.contains("61"), "and its line: {message}");
    }

    // 9d §4 rule 5: the status line names the mode and counts the selection.
    #[test]
    fn status_names_visual() {
        let mut state: ViewState = ViewState::new(pbr(), 80, 24);
        state.cursor = 100;
        assert_eq!(state.handle(Key::Char('v')), Action::Continue);
        for _ in 0..11 {
            state.handle(Key::Char('l'));
        }
        let line: String = state.status();
        assert!(line.starts_with("-- VISUAL --"), "line: {line}");
        assert!(
            line.contains("12 bases, 101..112"),
            "count and 1-based range: {line}"
        );
        assert!(line.contains("base 112"), "then what it says today: {line}");
    }

    // 9d §4 rule 1 and M9-6 (g): a selection on a circular record never wraps
    // over the origin; `End` leaves the plain range from the anchor to the last.
    #[test]
    fn circular_end_selects_the_plain_range() {
        let mut state: ViewState = ViewState::new(pbr(), 80, 24);
        assert!(state.circular(), "the file states circular (D049 M8-3)");
        assert_eq!(state.handle(Key::Char('v')), Action::Continue);
        state.handle(Key::End);
        assert_eq!(state.cursor(), 4360, "the record's last base");
        assert_eq!(state.selection(), Some((0, 4361)), "half-open [0, 4361)");
    }

    /// The file line of the feature that a CDS came from: what `apply_edit`
    /// follows to find the selected CDS again after every edit (§4 rule 2).
    fn cds_feature_line(record: &ViewRecord, cds: usize) -> u64 {
        record.features[record.cdss[cds].feature].line
    }

    // 9e §5 test 4 (M9-6 a): an edit before the selected CDS keeps it selected,
    // with its whole range shifted by the bases the insert took in.
    #[test]
    fn selected_cds_stays_through_a_neutral_edit() {
        let opened: ViewRecord = pbr();
        let tet_line: u64 = cds_feature_line(&opened, 0);
        let mut state: ViewState = ViewState::new(opened, 80, 24);
        assert!(state.select_cds(Some(0)), "tet is usable (M8-11)");
        assert!(state.chosen.is_some(), "the map and residues are held");

        let outcome: EditOutcome = state
            .apply_edit(Edit::Insert {
                at: 50,
                bases: "AAA".chars().collect(),
            })
            .expect("three bases before tet");
        assert!(outcome.removed.is_empty(), "no feature is emptied");
        assert!(outcome.dropped.is_empty(), "no CDS loses bases");

        let index: usize = state.selected().expect("tet stays selected");
        assert_eq!(
            cds_feature_line(state.record(), index),
            tet_line,
            "found again by the feature's file line"
        );
        assert_eq!(state.mode(), &LaneMode::Cds(index), "the same CDS is shown");
        // tet's own range moves three bases right: 86..1276 becomes 89..1279.
        let cds: Cds = usable(state.record(), index)
            .expect("tet stays usable")
            .clone();
        assert_eq!(
            cds.intervals()[0].start(),
            88,
            "its first codon now sits at 1-based 89"
        );
        assert!(
            state.title().contains("tet (+) 89..1279"),
            "title: {}",
            state.title()
        );
        assert!(state.chosen.is_some(), "its map is rebuilt anew");
    }

    // 9f §5 test 1: `i` inserts before the base the cursor stands on and `a`
    // after it, and the letter typed goes in upper case however it is typed.
    #[test]
    fn insert_before_and_after() {
        let opened: ViewRecord = pbr();
        // The `i` half: `i` puts the point at the cursor, so the two letters
        // typed land at 1-based 10 and 11.
        let mut state: ViewState = ViewState::new(opened.clone(), 80, 24);
        state.cursor = 9;
        assert_eq!(state.handle(Key::Char('i')), Action::Continue, "i");
        state.handle(Key::Char('g'));
        state.handle(Key::Char('c'));
        state.handle(Key::Esc);
        assert_eq!(state.editing, EditMode::Normal, "`esc` leaves insert");
        assert_eq!(state.record().sequence.length(), 4363, "two bases took in");
        let after: Vec<char> = state.record().sequence.letters();
        assert_eq!(after[9], 'G', "the 10th base");
        assert_eq!(after[10], 'C', "the 11th base");
        // 9f §4 rule 2: `esc` leaves the cursor on the last base typed.
        assert_eq!(state.cursor(), 10, "the cursor on the last base typed");

        // The `a` half starts from a fresh view with the cursor at zero-based 9:
        // `a` puts the point after the cursor's base, so the letter typed takes
        // 1-based 11 and every base from the record's old 11th on shifts right.
        let mut state: ViewState = ViewState::new(opened.clone(), 80, 24);
        let before: Vec<char> = opened.sequence.letters();
        state.cursor = 9;
        assert_eq!(state.handle(Key::Char('a')), Action::Continue, "a");
        state.handle(Key::Char('t'));
        assert_eq!(state.record().sequence.length(), 4362, "one base took in");
        let after: Vec<char> = state.record().sequence.letters();
        assert_eq!(after[10], 'T', "the 11th base, upper case");
        assert_eq!(&after[11..], &before[10..], "the rest shifts right");
    }

    // 9f §5 test 2 (M9-2): in `Insert` every other printable key is a letter the
    // view refuses, not a command, so the record is untouched and the session
    // goes on.
    #[test]
    fn insert_refuses_non_bases() {
        let mut state: ViewState = ViewState::new(pbr(), 80, 24);
        let before: Vec<char> = state.record().sequence.letters();
        state.cursor = 9;
        state.handle(Key::Char('i'));
        for ch in ['q', '1', ' ', 'x'] {
            assert_eq!(state.handle(Key::Char(ch)), Action::Continue, "{ch}");
            assert_eq!(
                state.record().sequence.letters(),
                before,
                "{ch} changed the record"
            );
            assert!(state.message().is_some(), "{ch} leaves a message");
            assert!(
                state.status().starts_with("-- INSERT --"),
                "and the mode is still insert: {}",
                state.status()
            );
        }
        // `q` there is a refused letter and not quit (9f §4).
        assert_eq!(
            state.handle(Key::Char('q')),
            Action::Continue,
            "q in insert"
        );
    }

    // 9f §5 test 3: `backspace` takes back only the bases this spell of insert
    // put in, and never a base that was there before the `i`.
    #[test]
    fn backspace_only_takes_what_was_typed() {
        let opened: ViewRecord = pbr();
        let mut state: ViewState = ViewState::new(opened.clone(), 80, 24);
        state.cursor = 9;
        state.handle(Key::Char('i'));
        for _ in 0..2 {
            state.handle(Key::Char('a'));
        }
        for _ in 0..3 {
            state.handle(Key::Backspace);
        }
        assert_eq!(
            state.record().sequence.length(),
            4361,
            "the third backspace took nothing away"
        );
        assert_eq!(
            state.record().sequence.letters(),
            opened.sequence.letters(),
            "and the letters are pBR322's again"
        );
        assert_eq!(
            state.cursor(),
            9,
            "the cursor followed the point back to where it stood when `i` was pressed"
        );
    }

    // 9f §5 test 4: overtype substitutes to the last base and stops there, so
    // the record is never lengthened and the second letter past the end is said
    // so.
    #[test]
    fn overtype_stops_at_the_end() {
        let opened: ViewRecord = pbr();
        let last: u64 = opened.sequence.length() - 1;
        assert_ne!(
            opened.sequence.letter_at(last).unwrap(),
            'G',
            "the record's last base is not the one typed, so the substitution bites"
        );
        let mut state: ViewState = ViewState::new(opened, 80, 24);
        state.handle(Key::End);
        assert_eq!(state.cursor(), last, "`End` on the record's last base");
        state.handle(Key::Char('R'));
        state.handle(Key::Char('g'));
        assert_eq!(
            state.record().sequence.length(),
            4361,
            "overtype never makes the record longer"
        );
        assert_eq!(
            state.record().sequence.letter_at(last),
            Ok('G'),
            "the last base is substituted"
        );
        assert_eq!(state.cursor(), last, "and the cursor stays on it");
        let after_first: Vec<char> = state.record().sequence.letters();
        state.handle(Key::Char('g'));
        assert_eq!(
            state.record().sequence.letters(),
            after_first,
            "the second letter past the end is refused"
        );
        assert!(
            state
                .message()
                .expect("the refusal is said")
                .contains("overtype"),
            "and says overtype stops: {:?}",
            state.message()
        );
    }

    // 9f §5 test 5: `r` from `Visual` substitutes every base of the selection in
    // one `Substitute` and leaves `Normal`.
    #[test]
    fn r_replaces_a_selection() {
        let opened: ViewRecord = pbr();
        let mut state: ViewState = ViewState::new(opened.clone(), 80, 24);
        state.handle(Key::Char('v'));
        for _ in 0..2 {
            state.handle(Key::Char('l'));
        }
        state.handle(Key::Char('r'));
        state.handle(Key::Char('n'));
        assert_eq!(state.editing, EditMode::Normal, "the mode is Normal");
        for index in 0..3 {
            assert_eq!(
                state.record().sequence.letter_at(index),
                Ok('N'),
                "base {}",
                index + 1
            );
        }
        assert_eq!(
            state.record().sequence.length(),
            opened.sequence.length(),
            "and the length is unchanged"
        );
    }

    // 9f §5 test 9 (through 9e): three bases typed with `i` shift the whole
    // record after them, and the features the record keeps with it.
    #[test]
    fn insert_moves_features() {
        let opened: ViewRecord = pbr();
        let tet_line: u64 = cds_feature_line(&opened, 0);
        let mut state: ViewState = ViewState::new(opened, 80, 24);
        assert!(state.select_cds(Some(0)), "tet is usable (M8-11)");
        state.cursor = 50;
        state.handle(Key::Char('i'));
        for ch in ['g', 'c', 'a'] {
            state.handle(Key::Char(ch));
        }
        state.handle(Key::Esc);
        assert_eq!(
            state.record().sequence.length(),
            4364,
            "three bases took in"
        );
        let index: usize = state.selected().expect("tet stays selected");
        assert_eq!(
            cds_feature_line(state.record(), index),
            tet_line,
            "found again by the feature's file line"
        );
        let cds: Cds = usable(state.record(), index)
            .expect("tet stays usable")
            .clone();
        assert_eq!(
            cds.intervals()[0].start(),
            88,
            "its first codon sits at 1-based 89"
        );
        assert!(
            state.title().contains("tet (+) 89..1279"),
            "title: {}",
            state.title()
        );
    }

    // 9f §5 test 6 (M9-4): a paste of a header line plus a body line keeps the
    // bases, and the header line, the spaces and the digits go.
    #[test]
    fn paste_drops_what_m9_4_drops() {
        let opened: ViewRecord = pbr();
        let mut state: ViewState = ViewState::new(opened.clone(), 80, 24);
        state.cursor = 9;
        state.handle(Key::Char('i'));
        state.paste_text(">x\nACGT 12\nacgt\n");
        assert_eq!(
            state.record().sequence.length(),
            opened.sequence.length() + 8,
            "eight bases went in"
        );
        let letters: Vec<char> = state.record().sequence.letters();
        let pasted: String = letters[9..17].iter().collect();
        assert_eq!(pasted, "ACGTACGT", "the letters kept, upper case");
        assert_eq!(
            state.editing,
            EditMode::Insert {
                point: 17,
                typed: 8
            },
            "point and typed went up by the length of what was pasted"
        );
    }

    // 9f §5 test 7: one bad character refuses the whole paste, and the refusal
    // names the character and its 1-based line and column in the pasted text.
    #[test]
    fn paste_refuses_a_bad_character_whole() {
        let opened: ViewRecord = pbr();
        let mut state: ViewState = ViewState::new(opened.clone(), 80, 24);
        state.cursor = 9;
        state.handle(Key::Char('i'));
        state.paste_text("ACGT\nAC!T");
        assert_eq!(
            state.record().sequence.length(),
            opened.sequence.length(),
            "nothing at all was inserted"
        );
        assert_eq!(
            state.record().sequence.letters(),
            opened.sequence.letters(),
            "not even the first line went inserted"
        );
        let message: String = state.message().expect("the refusal is said").to_string();
        assert!(message.contains('!'), "names the character: {message}");
        assert!(message.contains("line 2"), "names the line: {message}");
        assert!(message.contains("column 3"), "names the column: {message}");
    }

    // 9f §5 test 8 (M9-5): a paste outside insert mode is refused and says what
    // to press first.
    #[test]
    fn paste_outside_insert_does_nothing() {
        let opened: ViewRecord = pbr();
        let mut state: ViewState = ViewState::new(opened.clone(), 80, 24);
        state.paste_text("ACGT");
        assert_eq!(
            state.record().sequence.letters(),
            opened.sequence.letters(),
            "nothing was inserted"
        );
        let message: String = state.message().expect("the refusal is said").to_string();
        assert!(
            message.contains("press i or a"),
            "says to press `i` or `a`: {message}"
        );
    }

    // 9e §5 test 5 (full tier, M9-6 i): one base inside the selected CDS frames
    // it, so the view deselects it, the lanes go `Off`, and the status line
    // names the CDS and gives core's own refusal.
    #[test]
    fn a_frameshift_deselects_with_the_reason() {
        let opened: ViewRecord = pbr();
        let tet_line: u64 = cds_feature_line(&opened, 0);
        let mut state: ViewState = ViewState::new(opened, 80, 24);
        assert!(state.select_cds(Some(0)), "tet is usable");

        state
            .apply_edit(Edit::Insert {
                at: 200,
                bases: vec!['G'],
            })
            .expect("one base inside tet");
        assert_eq!(state.selected(), None, "the frameshifted tet is dropped");
        assert_eq!(state.mode(), &LaneMode::Off, "the lanes go Off");
        assert!(state.chosen.is_none(), "and its map goes with it");

        // The refusal is core's own, read off the rebuilt record.
        let rebuilt: &ViewRecord = state.record();
        let feature: usize = rebuilt
            .features
            .iter()
            .position(|found| found.line == tet_line)
            .expect("tet is still a feature of the record");
        let index: usize = rebuilt
            .cdss
            .iter()
            .position(|cds| cds.feature == feature)
            .expect("tet is still a CDS of the record");
        let reason: String = match &rebuilt.cdss[index].status {
            CdsStatus::Unusable(error) => error.message(),
            CdsStatus::Usable(_cds) => panic!("one base inside tet frames it"),
        };
        let message: String = state
            .message()
            .expect("the edit leaves a message")
            .to_string();
        assert!(message.contains("tet"), "names the CDS: {message}");
        assert!(message.contains(&reason), "and gives the reason: {message}");
    }

    // 9e §5 test 6 (M9-6 a, 8l1 §4 rules 3 to 5): the hits of the last search
    // follow the edit, and stay marked for `n` and `N` to walk.
    #[test]
    fn hits_follow_the_edit() {
        let mut state: ViewState = ViewState::new(pbr(), 80, 24);
        state.handle(Key::Char('/'));
        for ch in "CTGCAG".chars() {
            state.handle(Key::Char(ch));
        }
        state.handle(Key::Enter);
        let before: Vec<u64> = state
            .hits()
            .expect("pBR322 has one PstI site")
            .shown()
            .iter()
            .map(|hit| hit.start)
            .collect();
        // J01749.1.gb's CTGCAG sits at zero-based 3606 (1-based 3607), read
        // out of the fixture's ORIGIN block by hand.
        assert_eq!(before, vec![3606], "the site as opened");
        // The cursor sits where `enter` parked it, on the hit's start.
        assert_eq!(state.cursor(), 3606, "the cursor on the hit");
        let message_before: Option<String> = state.message().map(str::to_string);

        state
            .apply_edit(Edit::Insert {
                at: 0,
                bases: "AAA".chars().collect(),
            })
            .expect("three bases before base one");
        // Contract 9e §4 rule 6: the edit re-marks the hits and moves nothing.
        assert_eq!(state.cursor(), 3606, "the edit did not move the cursor");
        assert_eq!(
            state.message().map(str::to_string),
            message_before,
            "the re-run search wrote nothing into the message"
        );
        let hits: &Hits = state
            .hits()
            .expect("the search ran again over the edited record");
        let after: Vec<u64> = hits.shown().iter().map(|hit| hit.start).collect();
        assert_eq!(
            after,
            vec![3609],
            "the hit moved by the three bases the insert took in"
        );
        let current: &Hit = &hits.shown()[hits.current()];
        assert_eq!(current.start, 3609, "the current hit is after it");
    }

    // 9e §5 test 7 (M9-6 c): a feature the edit leaves with no range goes, and
    // the status line names it.
    #[test]
    fn removal_is_named() {
        let mut state: ViewState = ViewState::new(pbr(), 80, 24);
        let outcome: EditOutcome = state
            .apply_edit(Edit::Delete { start: 23, end: 27 })
            .expect("the four bases of misc_binding 24..27");
        assert_eq!(outcome.removed.len(), 1, "that one feature only");
        assert_eq!(outcome.removed[0].key, "misc_binding", "its key");
        assert_eq!(
            state.message().expect("the status line names it"),
            "removed: misc_binding"
        );
    }

    // 9g §5 test 3 (M9-2, M9-5): `x` cuts the selection into the register and
    // `p` puts the three bases back after the cursor base; the record ends up
    // just as long as it was.
    #[test]
    fn cut_then_paste_moves_bases() {
        let opened: ViewRecord = pbr();
        let first: Vec<char> = opened.sequence.letters()[..3].to_vec();
        let mut state: ViewState = ViewState::new(opened.clone(), 80, 24);
        assert_eq!(state.handle(Key::Char('v')), Action::Continue);
        for _ in 0..2 {
            state.handle(Key::Char('l'));
        }
        assert_eq!(state.selection(), Some((0, 3)), "three bases selected");
        state.handle(Key::Char('x'));
        assert_eq!(
            state.record().sequence.length(),
            opened.sequence.length() - 3,
            "three bases went"
        );
        assert_eq!(state.selection(), None, "the Visual cut leaves the mode");
        assert_eq!(state.cursor(), 0, "the cursor stands at the cut start");
        state.handle(Key::Char('p'));
        assert_eq!(
            state.record().sequence.length(),
            opened.sequence.length(),
            "the paste puts them back, the length is unchanged overall"
        );
        let letters: Vec<char> = state.record().sequence.letters();
        assert_eq!(
            &letters[1..4],
            first.as_slice(),
            "the three bases sit after the cursor base"
        );
        assert_eq!(
            state.cursor(),
            3,
            "the cursor stands on the last pasted base"
        );
        assert_eq!(state.edits(), 2, "the cut and the paste are two groups");
    }

    // 9g §5 test 4 (M9-5): `y` copies the selection to the register without
    // touching the record, and `P` puts it in before the cursor base.
    #[test]
    fn yank_keeps_the_record() {
        let opened: ViewRecord = pbr();
        let pair: Vec<char> = opened.sequence.letters()[..2].to_vec();
        let mut state: ViewState = ViewState::new(opened.clone(), 80, 24);
        assert_eq!(state.handle(Key::Char('v')), Action::Continue);
        state.handle(Key::Char('l'));
        assert_eq!(state.handle(Key::Char('y')), Action::Continue);
        assert_eq!(
            state.record().sequence.length(),
            opened.sequence.length(),
            "yank leaves the record alone"
        );
        assert_eq!(
            state.message().expect("y names how many bases"),
            "yanked 2 bases"
        );
        state.handle(Key::Char('P'));
        assert_eq!(
            state.record().sequence.length(),
            opened.sequence.length() + 2,
            "P added the pair back"
        );
        let letters: Vec<char> = state.record().sequence.letters();
        assert_eq!(
            &letters[1..3],
            pair.as_slice(),
            "the pair sits before the cursor base"
        );
        assert_eq!(state.edits(), 1, "the paste is one group, yank is not");
    }

    // 9g §4 rule 6 and §5 test 5 (M9-16): the insert from `i` to `esc` is one
    // group, so one `u` takes the whole spell of letters back at once.
    #[test]
    fn an_insert_session_is_one_undo() {
        let opened: ViewRecord = pbr();
        let mut state: ViewState = ViewState::new(opened.clone(), 80, 24);
        state.cursor = 100;
        state.handle(Key::Char('i'));
        for ch in ['a', 'c', 'g'] {
            state.handle(Key::Char(ch));
        }
        state.handle(Key::Esc);
        assert_eq!(
            state.record().sequence.length(),
            opened.sequence.length() + 3,
            "three bases took in"
        );
        assert_eq!(state.edits(), 1, "one group for the whole spell");
        state.handle(Key::Char('u'));
        // The letters and the feature list must come back exactly as the file
        // had them. The whole `ViewRecord` cannot be compared this way, as a
        // rebuild carries the record as the writer wrote it and not its header
        // lines (9e's `rebuild` builds a GenBank record from parts).
        let record: &ViewRecord = state.record();
        assert_eq!(
            record.sequence.letters(),
            opened.sequence.letters(),
            "one `u` took back the whole spell"
        );
        assert_eq!(record.features, opened.features, "the features are back");
        assert_eq!(record.cdss, opened.cdss, "the CDSs are back");
        assert_eq!(
            record.sequence.content_hash(),
            opened.sequence.content_hash(),
            "the same sequence"
        );
        assert_eq!(state.edits(), 0, "nothing is left on the undo stack");
    }

    // 9g §5 test 6 (M9-8, M9-16): `ctrl-r` puts the undone group back on the
    // undo stack, and any new command clears what sat there.
    #[test]
    fn redo_after_undo() {
        let opened: ViewRecord = pbr();
        let mut state: ViewState = ViewState::new(opened.clone(), 80, 24);
        state.handle(Key::Char('x'));
        assert_eq!(
            state.record().sequence.length(),
            opened.sequence.length() - 1,
            "the cursor base is cut"
        );
        state.handle(Key::Char('u'));
        assert_eq!(
            state.record().sequence.letters(),
            opened.sequence.letters(),
            "the undo puts the base back"
        );
        assert_eq!(state.edits(), 0, "the undo stack is empty");
        state.handle(Key::CtrlR);
        assert_eq!(state.edits(), 1, "the group is back on the undo stack");
        assert_eq!(
            state.record().sequence.length(),
            opened.sequence.length() - 1,
            "the base is cut again"
        );
        // `u` moves the undone group onto the redo stack, and the new `x`
        // starts a new group, which clears the redo (M9-16). Only the second
        // cut is left standing, and its group is the only one on the undo stack.
        state.handle(Key::Char('u'));
        state.handle(Key::Char('x'));
        assert_eq!(state.edits(), 1, "the new cut replaces the undone one");
        state.handle(Key::CtrlR);
        let message: String = state
            .message()
            .expect("the new cut cleared the redo stack")
            .to_string();
        assert!(message.contains("nothing"), "nothing to redo: {message}");
        assert_eq!(
            state.record().sequence.length(),
            opened.sequence.length() - 1,
            "the cleared redo changed nothing"
        );
    }

    // 9g §5 test 7 (M9-6 c, M9-8): undoing a `Delete` that had removed a
    // feature brings that feature back with its location text.
    #[test]
    fn undo_brings_back_a_removed_feature() {
        let opened: ViewRecord = pbr();
        let mut state: ViewState = ViewState::new(opened.clone(), 80, 24);
        let outcome: EditOutcome = state
            .apply_edit(Edit::Delete { start: 23, end: 27 })
            .expect("the four bases of misc_binding 24..27");
        assert_eq!(outcome.removed.len(), 1, "that one feature only");
        state.close_group();
        assert_eq!(state.edits(), 1, "the Delete is a group of one");
        assert_eq!(
            state.record().features.len(),
            opened.features.len() - 1,
            "the feature list lost the binding"
        );
        state.handle(Key::Char('u'));
        assert_eq!(
            state.record().sequence.letters(),
            opened.sequence.letters(),
            "the four bases are back"
        );
        let buffer: &EditBuffer = state.edit.as_ref().expect("an editable record");
        let feature: &EditFeature = buffer
            .features()
            .iter()
            .find(|feature| feature.key == "misc_binding" && feature.line == 242)
            .expect("the binding feature is back");
        assert_eq!(
            feature.location_text.as_deref(),
            Some("24..27"),
            "and its location text came back as it was read"
        );
        assert_eq!(
            state.record().features,
            opened.features,
            "the view features are the opened ones again"
        );
    }

    // 9g §5 test 8 (§4 rules 3 and 4): `d` and `y` with no selection do nothing
    // and say the selection is made with `v`.
    #[test]
    fn d_and_y_need_a_selection() {
        let opened: ViewRecord = pbr();
        let mut state: ViewState = ViewState::new(opened.clone(), 80, 24);
        state.cursor = 20;
        for key in [Key::Char('d'), Key::Char('y')] {
            assert_eq!(state.handle(key), Action::Continue, "{key:?}");
            let message: String = state.message().expect("the refusal is said").to_string();
            assert!(message.contains('v'), "{key:?} asks for v: {message}");
        }
        assert_eq!(
            state.record().sequence.letters(),
            opened.sequence.letters(),
            "either key left the record as it was"
        );
        assert_eq!(state.cursor(), 20, "the cursor stays put");
        assert_eq!(state.edits(), 0, "nothing was edited");
    }

    // 9e §5 test 8 (M9-6 d): an edit on pBR322 with the ORF lanes shown, in a
    // debug build, is far under one second. The 10 000 000-base case is not
    // asserted but its timing is printed, so the cost of one edit against the
    // ceiling (D048, M9-14) is visible too.
    #[test]
    fn edits_are_fast_enough() {
        use std::time::{Duration, Instant};

        let mut state: ViewState = ViewState::new(pbr(), 80, 24);
        state.handle(Key::Char('O'));
        assert!(state.orfs.is_some(), "the ORF lanes are shown");

        let mut slowest: Duration = Duration::ZERO;
        for round in 0..20 {
            let started: Instant = Instant::now();
            state
                .apply_edit(Edit::Insert {
                    at: round,
                    bases: vec!['A'],
                })
                .unwrap_or_else(|error| panic!("insert {round}: {}", error.code()));
            let elapsed: Duration = started.elapsed();
            assert!(
                elapsed < Duration::from_secs(1),
                "edit {round} on pBR322 took {elapsed:?}, the contract asks for under 1 s"
            );
            slowest = slowest.max(elapsed);
        }
        eprintln!("pBR322 with ORFs shown, 20 one-base inserts: slowest apply_edit {slowest:?}");

        // One edit on a record at the ceiling minus a base (D048): the insert
        // brings it to `MAX_RECORD_BASES`, so it is accepted; the timing is
        // reported without an assertion limit.
        use luria_core::view_record::MAX_RECORD_BASES;
        let bases: String = "A".repeat(MAX_RECORD_BASES as usize - 1);
        let big: ViewRecord = open_record(
            Identifier::create("big".to_string()).unwrap(),
            format!(">big\n{bases}\n"),
        )
        .unwrap_or_else(|error| panic!("{} bases refused: {:?}", bases.len(), error.code()));
        assert_eq!(big.sequence.length(), MAX_RECORD_BASES - 1, "one under");

        let mut state: ViewState = ViewState::new(big, 80, 24);
        let started: Instant = Instant::now();
        state
            .apply_edit(Edit::Insert {
                at: 0,
                bases: vec!['A'],
            })
            .expect("the insert fits one base under the ceiling");
        let ceiling_edit: Duration = started.elapsed();
        eprintln!(
            "one apply_edit on {} bases: {ceiling_edit:?}",
            MAX_RECORD_BASES - 1
        );
        assert_eq!(
            state.record().sequence.length(),
            MAX_RECORD_BASES,
            "the edited record is at the ceiling"
        );
    }

    /// 8r3 §5 test 3: not an assertion, only the timings the contract asks for,
    /// printed in release mode, where the view is built as the binary builds it.
    /// The sequence is the one §5 names: reading the record, opening the view,
    /// 50 presses of `j`, 10 of `PageDown` and nothing else. `j` moves the
    /// cursor a base at a time, so `visible` measures the rows of every block on
    /// screen before each press. `PageDown` moves the cursor a whole screen of
    /// blocks and counts the rows of the block it puts on top. `pBR322` opens the
    /// view with no CDS selected — two are usable, so M8-11 selects none — and
    /// these keys keep it in `Off` mode, where the state holds no reading; the
    /// mode that does hold one, `Cds`, is what `block_line_count_with` counts
    /// without translating again, and test 1 pins that count.
    /// It prints; it asserts nothing.
    ///
    /// ```text
    /// cargo test --release -p luria-cli --lib -- view_timing --ignored --nocapture
    /// ```
    #[test]
    #[ignore]
    fn view_timing_of_pbr322() {
        use std::time::Instant;

        let started: Instant = Instant::now();
        let record: ViewRecord = pbr();
        let reading: std::time::Duration = started.elapsed();

        let started: Instant = Instant::now();
        let mut state: ViewState = ViewState::new(record, 80, 24);
        let opening: std::time::Duration = started.elapsed();
        let base: u64 = state.record().sequence.length();

        let started: Instant = Instant::now();
        for _ in 0..50 {
            state.handle(Key::Char('j'));
        }
        let lines: std::time::Duration = started.elapsed();

        let started: Instant = Instant::now();
        for _ in 0..10 {
            state.handle(Key::PageDown);
        }
        let pages: std::time::Duration = started.elapsed();

        println!(
            "{base} bp: read {reading:?}, built the view {opening:?}, 50 `j` {lines:?}, \
             10 `PageDown` {pages:?}"
        );
    }
}
