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
use crate::sequence_view::layout::block_lines;
use crate::sequence_view::layout::layout;
use crate::sequence_view::layout::LaneMode;
use crate::sequence_view::layout::Layout;
use crate::sequence_view::layout::LayoutProblem;
use crate::sequence_view::layout::LineRole;
use luria_core::cds::Cds;
use luria_core::cds_map::CdsMap;
use luria_core::cds_map::CodonPlace;
use luria_core::display_location::DisplayLocation;
use luria_core::name_match::cds_span;
use luria_core::sequence::complement_letter;
use luria_core::sequence::SequenceTopology;
use luria_core::strand::Strand;
use luria_core::translation::translate;
use luria_core::view_record::CdsStatus;
use luria_core::view_record::FeatureShape;
use luria_core::view_record::TopologySource;
use luria_core::view_record::ViewRecord;
use ratatui::text::Line;

/// One key press, as `super::terminal` maps it (§4 rule 14). Any press this
/// view has no key for is `Other` and changes nothing (§4 rule 10; D055 leaves
/// `r` and `space` unbound, and `g`, `o`, `f`, `c` and `/` come in 8k2, 8k3 and
/// 8l1).
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
    /// `ctrl-c`, which quits from anywhere (§4 rule 10).
    CtrlC,
    /// Any other press.
    Other,
}

/// What the session does after a key: draw again, or leave (§4 rule 10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Continue,
    Quit,
}

/// How far `w` and `b` look for the next codon's middle base (§4 rule 5): a
/// codon is three bases, so the middle base of the next one is at most two
/// bases from the middle base of this one, and three covers either edge.
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
    /// terminal stop (D010).
    letters: String,
    /// As the feature states it, never inferred (D009): which way the codon
    /// keys and the codon's letters run.
    strand: Strand,
}

/// The view between keys: the record, the screen's size, the selection, the
/// lanes, the cursor and the top block.
pub struct ViewState {
    record: ViewRecord,
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
    /// The selected CDS's map and residues, `None` with no selection.
    chosen: Option<Chosen>,
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
        let mut state: ViewState = ViewState {
            circular: record.sequence.topology() == SequenceTopology::Circular,
            selected,
            mode,
            record,
            width,
            height,
            cursor: 0,
            top: 0,
            layout: None,
            message: None,
            chosen: None,
        };
        state.choose();
        state.relayout(false);
        state
    }

    /// A new terminal size: the record is laid out again for the new width, the
    /// top block keeps the base it covered before (§4 rule 6), and the scroll
    /// follows the cursor.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.relayout(true);
    }

    /// One key press: what it moves, and whether the session goes on (§4 rules
    /// 2 to 10). Whatever the key, the message a previous key left is cleared
    /// here; `q` and `ctrl-c` leave, and below `MIN_WIDTH` by `MIN_HEIGHT`
    /// nothing else does anything (§4 rule 10).
    pub fn handle(&mut self, key: Key) -> Action {
        if key == Key::CtrlC || key == Key::Char('q') {
            return Action::Quit;
        }
        self.message = None;
        if !big_enough(self.width, self.height) {
            return Action::Continue;
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
            // Every other key changes nothing (§4 rule 10).
            Key::Char(_) | Key::Other | Key::CtrlC => return Action::Continue,
        }
        self.follow();
        Action::Continue
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

    /// The topology the view moves on: `T` flips it only for a record whose
    /// topology was assumed (D049 M8-3).
    pub fn circular(&self) -> bool {
        self.circular
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
        let room: usize = body_rows(self.height);
        let mut rows: usize = 0;
        let mut count: usize = 0;
        for block in self.top..blocks.len() {
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
        self.top..(self.top + count)
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
            "{} · {} bp {}",
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
    /// message, when one is set.
    pub(super) fn status(&self) -> String {
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
    /// nothing (8h adds the keys it dispatches).
    pub(super) fn keys(&self) -> String {
        "h j k l move  PgUp PgDn Home End  w b codon  t translation  T topology  q quit".to_string()
    }

    /// One block's lines as `layout` gives them, with the cursor and the codon's
    /// underline left for `super::screen` to add (§4 rule 12). A block whose
    /// lines `layout` refuses draws nothing.
    pub(super) fn lines(&self, block: usize) -> Vec<(LineRole, Line<'static>)> {
        match self.layout.as_ref() {
            Some(built) => block_lines(&self.record, built, &self.mode, block).unwrap_or_default(),
            None => Vec::new(),
        }
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
            Some(number) => chosen.letters.chars().nth(number as usize - 1),
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

    /// The topology as the title words it (§4 rule 11, D049 M8-3): what the file
    /// states, or the assumption a FASTA record carries, named as an assumption
    /// until `T` sets it aside for the session.
    fn topology_words(&self) -> String {
        let assumed: bool = self.record.topology == TopologySource::Assumed;
        match (self.circular, assumed) {
            (true, true) => "circular (set with T)".to_string(),
            (true, false) => "circular".to_string(),
            (false, true) => "linear (assumed)".to_string(),
            (false, false) => "linear".to_string(),
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
                        letters: made.residues_text(),
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
    /// gives it for the current mode, gaps aside — the blocks are drawn inside
    /// one frame, not fenced by their own border (§4 rule 1). A block whose
    /// lines are refused takes none, the same as `lines` gives it, so a run is
    /// never measured shorter than it draws.
    fn line_count(&self, block: usize) -> usize {
        match self.layout.as_ref() {
            Some(built) => match block_lines(&self.record, built, &self.mode, block) {
                Ok(lines) => lines.len(),
                Err(_problem) => 0,
            },
            None => 0,
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
    /// pBR322's last block holds 41 bases, so the line from zero-based 4310
    /// lands on 4360, its last base, and not past the end of the record.
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

    /// `PageDown` and `PageUp`: the same line move by one screen of blocks,
    /// clamped at either end, and the cursor's block goes on top (§4 rule 4).
    fn page(&mut self, direction: i64) {
        let screen: usize = self.visible().len().max(1);
        let built: &Layout = match self.layout.as_ref() {
            Some(built) => built,
            None => return,
        };
        let last: usize = match built.blocks.len().checked_sub(1) {
            Some(last) => last,
            None => return,
        };
        let block: usize = match self.block_of(self.cursor) {
            Some(found) => found,
            None => return,
        };
        let (start, _end): (u64, u64) = built.blocks[block];
        let target: usize = if direction > 0 {
            (block + screen).min(last)
        } else {
            block.saturating_sub(screen)
        };
        let (first, last_base): (u64, u64) = built.blocks[target];
        let cursor: u64 = first + (self.cursor - start).min(last_base - first - 1);
        self.cursor = cursor;
        self.top = target;
    }

    /// `w` and `b`: the nearest base 1 to 3 bases away that is a codon's middle
    /// base of the selected CDS, wrapping when the view moves on a circular
    /// record; the cursor stays when there is no such base or no CDS is selected
    /// (§4 rule 5).
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
                // further in that direction can be a codon's middle base.
                None => break,
            };
            let middle: bool = match chosen.map.base_residue(base) {
                Some(place) => place.place == 2,
                None => false,
            };
            if middle {
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

    /// `T` (D049 M8-3, §4 rule 8): a FASTA record's assumed topology flips for
    /// the session, and the cursor keys wrap from then on; a record whose file
    /// states its own topology never changes, and the status line says why.
    fn flip(&mut self) {
        if self.record.topology == TopologySource::Assumed {
            self.circular = !self.circular;
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
        let room: usize = body_rows(self.height);
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
            state.title().contains("4361 bp circular"),
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
            state.title().contains("393 bp linear"),
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
}
