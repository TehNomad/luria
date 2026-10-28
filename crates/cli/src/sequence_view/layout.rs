//! The block layout: how a record splits into lines for a width and a
//! translation mode, and what one block's lines hold
//! (`docs/tasks/milestone-8-view.md` §5.1; D050 M8-19, D055, D053, D052).
//!
//! Two pure functions and no state: `layout` splits the record into blocks,
//! `block_lines` turns one block into lines. Nothing here reads a terminal, a
//! cursor or a key — 8k1 owns the state and is the only caller that draws —
//! and the lines come back as `Line<'static>`, the same shape
//! `review::view::styled_lines` returns before a widget renders it.
//!
//! The rules are the block-layout section of `tests/fixtures/view/README.md`,
//! which `layout-blocks.tsv` and `layout-cells.tsv` pin column by column:
//!
//! * **Gutter.** `d = max(2, digits of the record length)`: a line's label —
//!   its first base's number, or a frame label — is right-aligned in columns
//!   `[0, d)`, column `d` is blank, and bases start at `d + 1`. The minimum of
//!   2 is pinned only here: every record in the tables has at least 10 bases.
//! * **Line width.** The largest multiple of 3, in every mode (M8-28), with
//!   `2d + 2 + N <= width`. A width with no whole step of room is refused
//!   `TooNarrow`.
//! * **A shortened line, not a shifted grid** (D055). Without a CDS selected,
//!   block `k` is `[kN, (k+1)N)`. With one, lines still start at base 1, and a
//!   line that would end inside a codon of the selected CDS ends just before
//!   that codon, so it is 1 or 2 bases short and the lines after it continue
//!   from there. Every codon fact comes from `CdsMap`: the two bases either
//!   side of the edge answer whether an edge falls inside a codon, and the
//!   base before them answers which of its bases the line must stop short of.
//! * **Lines, top to bottom.** Up to four lanes, then `more`; a plus-strand
//!   CDS's residues then its numbers; the ruler line; `+3 +2 +1`; top; bottom; a
//!   minus-strand CDS's numbers then its residues; `-1 -2 -3`.
//! * **Lanes** (D053). Every drawn feature except `source` that has a base in
//!   the block, longest drawn length first then file order, at most four; the
//!   rest are counted on the `more` line. A lane covers the columns of that
//!   feature's bases in the block, as one run per stretch of consecutive bases,
//!   and its text is its name, an arrow for its strand, and the bar filled
//!   with `═`, cut to the run.
//! * **Lane colours** (M8-31, contract 8m7 §4 rules 3 and 4). With `colours`
//!   on, every cell of a lane is its feature's colour: the colour the file
//!   gives it on the strand it was written on, or its key's colour when the
//!   file gives it none, and nothing when neither does. The selected CDS's own
//!   lane is bold, whether the colours are on or off, and the `more` line is
//!   never coloured.
//! * **Residues** (D055, D052). Every residue sits over its codon's **middle**
//!   base, on whichever line holds that base: for a CDS that is the base
//!   `CdsMap` reports as place 2 of its codon, and for a frame it is the
//!   position `frame_residues` puts in `middle`, which is also the position
//!   the frame's own window test uses. A CDS's terminal stop codon is `*`; a
//!   frame's unreadable codon is `?`. With `colours` on (M8-29, contract 8m5)
//!   a codon whose letter stands in one of Zappo's seven groups has that
//!   group's background behind all three of its bases, and the letter is black
//!   over its own base; the codon's other two bases are spaces with that
//!   background, and a codon two lines split is coloured on each of them for
//!   the bases the one holds.
//! * **Numbers** (D055, D010). The selected CDS only. Every residue numbered
//!   a multiple of 5, in reading order, then the line's leftmost numbered
//!   residue; each centred on its residue, moved inward to stay over the
//!   line's bases, and dropped when it would touch a number already placed.
//!   The terminal stop codon has no number.
//! * **Ruler** (M8-33, contract 8m10). The line above the top strand, in every
//!   mode. Every base whose 1-based number is a multiple of 10 gets a `╷` at
//!   its own column, and every multiple of 30 gets that number right-aligned so
//!   its last digit stands at that column — or the tick when the number would
//!   start left of the line's first base. The line carries no label, so nothing
//!   stands in the gutter, and its text is dim with no colour, the colours on
//!   and off alike (the secondary text of M8-33).
//!
//! What this module does not draw, and 8k1 does: the cursor, underline, the
//! title bar, the status line and the overview inset.

use luria_core::cds::Cds;
use luria_core::cds_map::CdsMap;
use luria_core::cds_map::CodonPlace;
use luria_core::display_location::DisplayLocation;
use luria_core::error::DomainError;
use luria_core::frames::frame_residues;
use luria_core::frames::ReadingFrame;
use luria_core::sequence::complement_letter;
use luria_core::strand::Strand;
use luria_core::translation::translate;
use luria_core::view_record::CdsStatus;
use luria_core::view_record::FeatureShape;
use luria_core::view_record::ViewFeature;
use luria_core::view_record::ViewRecord;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// The narrowest gutter, whatever the record length says: a 5 bp record still
/// has two number columns (`tests/fixtures/view/README.md`, "Block layout":
/// the tables cannot pin this, because every record in them is at least 10
/// bases long).
const MIN_GUTTER: u16 = 2;

/// The step a line width is a multiple of with no translation (M8-28).
const PLAIN_STEP: u64 = 3;

/// The same step in every translation mode (M8-28), so toggling `t` does not
/// reflow the lines and a whole codon still stands on one line.
const TRANSLATED_STEP: u64 = 3;

/// Plan §5.1: "up to four lanes per block, the rest as `+N more`".
const LANE_LIMIT: usize = 4;

/// The glyph the ruler line marks every tenth base with (M8-33).
const TICK: char = '╷';

/// The translation the view shows over a block. `Cds`'s payload is an index
/// into `ViewRecord::cdss`, which is file order (D053) — not a residue number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaneMode {
    /// No translation: the two strands only.
    Off,
    /// One CDS's residues and residue numbers.
    Cds(usize),
    /// The three forward frames.
    ThreeFrames,
    /// The three forward frames and the three reverse frames.
    SixFrames,
}

/// Which of the record's features get a lane line (M8-30, contract 8m6
/// §4 rule 2): `F` cycles the three levels, and the level says what the sort,
/// `LANE_LIMIT` and the `+N more` line apply to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeatureLevel {
    /// Every drawn feature, as today (D053).
    All,
    /// Only the features whose key is `CDS`.
    Cdss,
    /// No feature gets a lane: no lane line and no `+N more` line.
    None,
}

/// How the lines are drawn, as the view's keys set it (M8-30, contract 8m6
/// §4 rule 1): `colours` is 8m5's `C` and `features` is 8m6's `F`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drawing {
    /// Whether the residue lines draw the Zappo backgrounds (M8-29).
    pub colours: bool,
    /// Which features get lanes (M8-30).
    pub features: FeatureLevel,
}

/// What one line of a block is, in the order `block_lines` returns them.
/// `Strand` has no `Eq` (`strand.rs:6`), so this compares by `PartialEq`.
#[derive(Clone, Debug, PartialEq)]
pub enum LineRole {
    /// One feature lane, numbered from 1 in lane order (D053).
    Lane(u8),
    /// The `+N more` line, when more features overlap the block than lanes.
    More,
    /// The selected CDS's residue letters.
    Residues,
    /// The selected CDS's residue numbers (D055: that CDS only).
    Numbers,
    /// One reading frame's residue letters, with its label in the gutter.
    Frame(Strand, ReadingFrame),
    /// The ruler: a tick over every tenth base of the block, and every thirtieth
    /// base's number (M8-33). It stands directly above `Top` in every mode.
    Ruler,
    /// The strand's own bases, with the line's two base numbers.
    Top,
    /// The bottom strand: the complement of the same bases.
    Bottom,
}

/// A record split for one width and mode: the arithmetic every line needs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    /// The gutter's width `d`: labels sit in `[0, d)`, bases start at `d + 1`.
    pub gutter: u16,
    /// The line width `N`: the largest multiple of the step with room beside
    /// the gutter on both sides.
    pub line_width: u64,
    /// The blocks, in order: zero-based half-open base ranges (D006). They
    /// tile the record, and only the last one may be short by more than 2.
    pub blocks: Vec<(u64, u64)>,
}

/// Why a layout was refused, or why one block cannot be drawn.
#[derive(Clone, Debug, PartialEq)]
pub enum LayoutProblem {
    /// No line width of at least one whole step fits: `needed` is
    /// `2 * gutter + 2 + step`, the narrowest usable width.
    TooNarrow { width: u16, needed: u16 },
    /// `LaneMode::Cds` names a `record.cdss` entry that is not
    /// `CdsStatus::Usable`, or names no entry at all. Only a usable CDS may be
    /// selected (D041, D042), and the view shows the refusal instead.
    NotUsable(usize),
    /// `translate` or `frame_residues` refused: the record's own answer about
    /// the CDS or the sequence, carried through unchanged.
    Domain(DomainError),
}

/// One item a line places: its text, the column it starts in, and its style.
struct Cell {
    column: u64,
    text: String,
    style: Style,
}

/// One feature that has a lane in this block.
struct Lane {
    /// Its file position, the tie-break after length (D053).
    index: usize,
    /// The D053 display name, which is what the lane spells out.
    name: String,
    /// As written in the location, never inferred (D009): the arrow's strand.
    strand: Strand,
    /// The feature's whole drawn length in bases, the lane order's first key.
    length: u64,
    /// The bases it covers inside the block, as half-open runs of consecutive
    /// bases in record coordinates.
    runs: Vec<(u64, u64)>,
}

/// One codon of the selected CDS whose middle base stands in this block.
struct Residue {
    /// Which codon of the CDS this is in reading order, counting from 1
    /// (`CodonPlace::codon`): the order numbers are placed in.
    codon: u64,
    /// The column its letter stands in.
    column: u64,
    /// The letter, or `*` for the terminal stop codon.
    symbol: char,
    /// Its D006 residue number; `None` for the terminal stop codon, which has
    /// a codon and no residue (D010).
    number: Option<u64>,
}

/// The gutter `d` a record's lines use: two columns, or one per digit of the
/// record length when that is more.
fn gutter(length: u64) -> u16 {
    let digits: u16 = length.to_string().chars().count() as u16;
    if digits > MIN_GUTTER {
        digits
    } else {
        MIN_GUTTER
    }
}

/// The strand as written, never inferred (D009): it decides the arrow, the
/// frame label's sign, and which side of the strands a CDS's residues sit on.
/// `Strand::is_plus` takes `self`, so this borrows it.
fn strand_is_plus(strand: &Strand) -> bool {
    *strand == Strand::Plus
}

/// The step the line width is a multiple of: 3 bases in every mode (M8-28), so
/// the same width holds the same line with and without a translation.
fn column_step(mode: &LaneMode) -> u64 {
    match mode {
        LaneMode::Off => PLAIN_STEP,
        LaneMode::Cds(_) | LaneMode::ThreeFrames | LaneMode::SixFrames => TRANSLATED_STEP,
    }
}

/// The selected CDS, or the reason it cannot be selected. An index with no
/// `cdss` entry at all is refused the same way as an unusable one: the view
/// has nothing to show for either, and `NotUsable` names the index it was
/// given either way.
fn usable_cds(record: &ViewRecord, index: usize) -> Result<&Cds, LayoutProblem> {
    match record.cdss.get(index) {
        Some(view) => match &view.status {
            CdsStatus::Usable(cds) => Ok(cds),
            CdsStatus::Unusable(_) => Err(LayoutProblem::NotUsable(index)),
        },
        None => Err(LayoutProblem::NotUsable(index)),
    }
}

/// The residue map of the selected CDS: `CdsMap::new` on `import_cds`'s CDS
/// and the translation of it against this record (D042 pins that the map and
/// the translation come from one reading of the same CDS).
fn cds_map(record: &ViewRecord, index: usize) -> Result<CdsMap, LayoutProblem> {
    let cds: &Cds = usable_cds(record, index)?;
    let translation = translate(&record.sequence, cds).map_err(LayoutProblem::Domain)?;
    Ok(CdsMap::new(cds, &translation))
}

/// Split the record into blocks (D055). Without a selected CDS, every block
/// is `line_width` long but the last. With one, an edge that would fall inside
/// a codon of that CDS moves back to just before the codon.
///
/// The move is at most 2 bases, because only the codon straddling the edge is
/// involved and it has at most two bases on the near side of it:
/// `base_residue` answers which codon the last base of the line and the first
/// base past it belong to, and whether the base before them shares that codon
/// says whether the line stops one base short or two.
///
/// A codon the record's own end splits — `frames-62.gb`'s codon of bases 62,
/// 1 and 2 — never moves an edge: it is at the end of a line, not inside one
/// (`tests/fixtures/view/README.md`, "How the values were checked").
fn split(length: u64, line_width: u64, map: Option<&CdsMap>) -> Vec<(u64, u64)> {
    let mut blocks: Vec<(u64, u64)> = Vec::new();
    let mut start: u64 = 0;
    while start < length {
        let mut end: u64 = start.saturating_add(line_width).min(length);
        if let Some(map) = map {
            // The last line ends at the record's end, and nothing is inside a
            // codon past the record.
            if end < length {
                // The codon straddling the edge, if there is one: the last base
                // of the line and the first base past it belong to the same
                // codon, so the line ends inside it.
                let straddling: Option<(CodonPlace, CodonPlace)> =
                    match (map.base_residue(end - 1), map.base_residue(end)) {
                        (Some(last), Some(next)) if last.codon == next.codon => Some((last, next)),
                        _ => None,
                    };
                if let Some((last, _next)) = straddling {
                    // The third base of that codon is on the near side too when
                    // the line covers two of its bases, and then the line stops
                    // short of two.
                    let before: Option<CodonPlace> =
                        end.checked_sub(2).and_then(|base| map.base_residue(base));
                    let two: bool = before.is_some_and(|place| place.codon == last.codon);
                    end = if two { end - 2 } else { end - 1 };
                }
            }
        }
        blocks.push((start, end));
        start = end;
    }
    blocks
}

/// Split `record` for `width` columns of lines and `mode`.
///
/// `width` is what the lines may use: no border, no status line, nothing else
/// (the 8a2 tables count columns the same way).
pub fn layout(record: &ViewRecord, width: u16, mode: &LaneMode) -> Result<Layout, LayoutProblem> {
    let length: u64 = record.sequence.length();
    let d: u16 = gutter(length);
    let step: u64 = column_step(mode);
    let needed: u64 = 2 * u64::from(d) + 2 + step;
    if u64::from(width) < needed {
        return Err(LayoutProblem::TooNarrow {
            width,
            needed: needed as u16,
        });
    }
    let line_width: u64 = (u64::from(width) - 2 * u64::from(d) - 2) / step * step;
    let map: Option<CdsMap> = match mode {
        LaneMode::Cds(index) => Some(cds_map(record, *index)?),
        LaneMode::Off | LaneMode::ThreeFrames | LaneMode::SixFrames => None,
    };
    Ok(Layout {
        gutter: d,
        line_width,
        blocks: split(length, line_width, map.as_ref()),
    })
}

/// The drawn features of `record`, in lane order for the block `[start, end)`:
/// every `Drawn` feature that is not a `source` and has a base in the block,
/// longest drawn length first, then file order (D053, D055; the plan §5.1
/// "longest first"). `source` has no lane: it describes the whole record, and
/// a bar over every base says nothing. `level` says which of them are kept
/// before the sort, so `LANE_LIMIT` and the `+N more` line count what the level
/// leaves (M8-30, §4 rule 2).
fn lanes(record: &ViewRecord, start: u64, end: u64, level: FeatureLevel) -> Vec<Lane> {
    // Under `None` no feature is kept, and with nothing kept there is nothing
    // for the `+N more` line to count either (§4 rule 2).
    if level == FeatureLevel::None {
        return Vec::new();
    }
    let mut out: Vec<Lane> = Vec::new();
    for feature in record.features.iter() {
        if feature.key == "source" {
            continue;
        }
        // CDSs only: the features whose key is `CDS`, named or not (§4 rule 2).
        if level == FeatureLevel::Cdss && feature.key != "CDS" {
            continue;
        }
        let shape: &DisplayLocation = match &feature.shape {
            FeatureShape::Drawn(shape) => shape,
            // A feature this record cannot draw has no cells to cover (D053).
            FeatureShape::NotDrawn(_) => continue,
        };
        let runs: Vec<(u64, u64)> = merged(shape.location.intervals());
        let length: u64 = runs.iter().map(|(first, last)| last - first).sum();
        let inside: Vec<(u64, u64)> = runs
            .iter()
            .filter_map(|(first, last)| {
                let first: u64 = (*first).max(start);
                let last: u64 = (*last).min(end);
                if first < last {
                    Some((first, last))
                } else {
                    None
                }
            })
            .collect();
        if inside.is_empty() {
            continue;
        }
        out.push(Lane {
            index: feature.index,
            name: feature.name.clone(),
            strand: shape.location.strand(),
            length,
            runs: inside,
        });
    }
    out.sort_by(|left, right| {
        right
            .length
            .cmp(&left.length)
            .then(left.index.cmp(&right.index))
    });
    out
}

/// The intervals of one location as runs of consecutive covered bases:
/// overlapping and abutting intervals merge, so a location written
/// `join(1..6,7..12)` is one run, as its bases are consecutive.
fn merged(intervals: Vec<(u64, u64)>) -> Vec<(u64, u64)> {
    let mut sorted: Vec<(u64, u64)> = intervals;
    sorted.sort();
    let mut runs: Vec<(u64, u64)> = Vec::new();
    for (first, last) in sorted {
        if first >= last {
            continue;
        }
        match runs.last_mut() {
            // Abutting, not just overlapping: the bases either side of the
            // join are consecutive, so they are one run of bases.
            Some(previous) if first <= previous.1 => previous.1 = previous.1.max(last),
            _ => runs.push((first, last)),
        }
    }
    runs
}

/// The text of one lane run: its name, an arrow for its strand, and the bar,
/// cut to `length` columns. Every run starts with the name, including one
/// clipped at a block edge (contract 8j rule 3), so a run shorter than its
/// name spells part of the name and nothing else.
fn lane_text(name: &str, strand: &Strand, length: u64) -> String {
    let arrow: char = if strand_is_plus(strand) { '▸' } else { '◂' };
    let mut text: String = String::new();
    text.push_str(name);
    text.push(' ');
    text.push(arrow);
    text.push(' ');
    while (text.chars().count() as u64) < length {
        text.push('═');
    }
    text.chars().take(length as usize).collect()
}

/// The style of one residue letter: a stop is bold red, the way
/// `review::view::letter_style` draws a protein's `*` (contract 8j rule 5).
/// Everything else is left alone, and 8m5's colour rule adds a background and a
/// black letter on top of what is here. With the colours off the stop is bold
/// only, its red being a colour and not a modifier (M8-34 rule 5).
fn residue_style(symbol: char, colours: bool) -> Style {
    if symbol != '*' {
        return Style::default();
    }
    let mut style: Style = Style::default().add_modifier(Modifier::BOLD);
    if colours {
        style.fg = Some(Color::Red);
    }
    style
}

/// The background one of the twenty letters draws by Zappo's seven
/// physicochemical groups (M8-29, contract 8m5 §4 rule 1), as xterm 256-colour
/// indexes: Zappo's own values or within 25 of them on every channel, so no
/// true-colour detection is needed. A symbol outside the twenty — the stop's
/// `*`, a frame's unreadable codon's `?`, an `X` from a codon the record cannot
/// read — has no group and no colour, so the cell keeps the look `block_lines`
/// gives it today.
fn zappo(symbol: char) -> Option<Color> {
    match symbol {
        // aliphatic and hydrophobic.
        'I' | 'L' | 'V' | 'A' | 'M' => Some(Color::Indexed(217)),
        // aromatic.
        'F' | 'W' | 'Y' => Some(Color::Indexed(220)),
        // positive.
        'K' | 'R' | 'H' => Some(Color::Indexed(63)),
        // negative.
        'D' | 'E' => Some(Color::Indexed(196)),
        // hydrophilic.
        'S' | 'T' | 'N' | 'Q' => Some(Color::Indexed(46)),
        // conformationally special.
        'P' | 'G' => Some(Color::Indexed(201)),
        // cysteine.
        'C' => Some(Color::Indexed(226)),
        _ => None,
    }
}

/// A label right-aligned in the gutter: `columns [0, d)`, blank at `d`. Every
/// gutter label is part of the view's secondary text and dim, the colours on and
/// off alike (M8-34 rule 1).
fn label_cell(label: &str, d: u64) -> Cell {
    Cell {
        column: d - label.chars().count() as u64,
        text: label.to_string(),
        style: Style::default().add_modifier(Modifier::DIM),
    }
}

/// The colour a base letter that is not one of the four DNA letters draws:
/// an IUPAC ambiguity code, or any other letter the record's sequence holds
/// (M8-34 rule 4). Nothing outside the four is coloured with the colours off.
const AMBIGUOUS: Color = Color::Indexed(208);

/// The style one letter of a strand line draws: `base`'s modifiers always, and
/// the ambiguous colour over them when the letter stands for anything but one of
/// `A C G T` and `colours` is on (M8-34 rules 1 and 4).
fn base_style(letter: char, colours: bool, base: Style) -> Style {
    if colours && !matches!(letter, 'A' | 'C' | 'G' | 'T') {
        let mut style: Style = base;
        style.fg = Some(AMBIGUOUS);
        style
    } else {
        base
    }
}

/// The letters of one strand line, as one cell per run of letters that share a
/// look: an ambiguous letter starts a cell of its own, and a run of ordinary
/// letters keeps the one cell the whole line used to be. `column` is where the
/// line's first letter stands.
fn base_runs(text: &str, column: u64, colours: bool, base: Style) -> Vec<Cell> {
    let letters: Vec<char> = text.chars().collect();
    let mut cells: Vec<Cell> = Vec::new();
    let mut at: usize = 0;
    while at < letters.len() {
        let style: Style = base_style(letters[at], colours, base);
        let mut end: usize = at + 1;
        while end < letters.len() && base_style(letters[end], colours, base) == style {
            end += 1;
        }
        cells.push(Cell {
            column: column + at as u64,
            text: letters[at..end].iter().collect(),
            style,
        });
        at = end;
    }
    cells
}

/// The two strand lines of a block: the bases, their complements, and the
/// line's first and last base numbers. The right number starts one blank column
/// after the last base. The bottom line and the two numbers in the gutter are the
/// view's secondary text and dim, the colours on and off alike (M8-34 rule 1);
/// the top line's letters are never dim, and `colours` gives an ambiguous letter
/// its own colour on either line (M8-34 rule 4).
fn strand_cells(
    record: &ViewRecord,
    d: u64,
    g: u64,
    start: u64,
    end: u64,
    colours: bool,
) -> Result<(Vec<Cell>, Vec<Cell>), LayoutProblem> {
    let mut bases: String = String::new();
    for position in start..end {
        bases.push(
            record
                .sequence
                .letter_at(position)
                .map_err(LayoutProblem::Domain)?,
        );
    }
    // `Sequence` accepts only letters that have a complement, so `None` here
    // would be a sequence this record never held; the letter it holds stays as
    // it is rather than inventing a gap glyph (the same fallback
    // `review::text::bottom_letter` uses).
    let complement: String = bases
        .chars()
        .map(|letter| complement_letter(letter).unwrap_or(letter))
        .collect();

    let left: String = (start + 1).to_string();
    let right: String = end.to_string();
    let dim: Style = Style::default().add_modifier(Modifier::DIM);
    let mut top: Vec<Cell> = Vec::new();
    top.push(Cell {
        column: d - left.chars().count() as u64,
        text: left,
        style: dim,
    });
    top.extend(base_runs(&bases, g, colours, Style::default()));
    top.push(Cell {
        column: g + (end - start) + 1,
        text: right,
        style: dim,
    });
    let bottom: Vec<Cell> = base_runs(&complement, g, colours, dim);
    Ok((top, bottom))
}

/// The selected CDS's residues whose codon's middle base stands in the block,
/// in reading order: `CdsMap` names place 2 of a codon as that middle base
/// whatever the strand is, including a codon the origin splits.
fn cds_residues(map: &CdsMap, letters: &[char], g: u64, start: u64, end: u64) -> Vec<Residue> {
    let mut on: Vec<Residue> = Vec::new();
    for position in start..end {
        let place: Option<CodonPlace> = map.base_residue(position);
        let place: CodonPlace = match place {
            Some(place) if place.place == 2 => place,
            // Not in the CDS, or not the base its residue sits over.
            _ => continue,
        };
        let symbol: char = match codon_letter(map, &place, letters) {
            Some(symbol) => symbol,
            // Nothing to draw here: the codon has no residue, and this is not
            // the CDS's terminal stop codon, or the translation holds no letter
            // for the residue number it gives.
            None => continue,
        };
        on.push(Residue {
            codon: place.codon,
            column: g + position - start,
            symbol,
            number: place.residue,
        });
    }
    // Reading order, so a minus-strand CDS's numbers are considered from its
    // own first residue, not from the left edge of the line.
    on.sort_by_key(|residue| residue.codon);
    on
}

/// The letter one codon of the selected CDS draws: the letter of the residue
/// the map gives, or `*` for the terminal stop codon (D010), and `None` for a
/// codon the CDS has no residue for and for a residue number the translation
/// does not hold. The same answer `cds_residues` and `cds_cells` take.
fn codon_letter(map: &CdsMap, place: &CodonPlace, letters: &[char]) -> Option<char> {
    match place.residue {
        Some(number) => Some(*letters.get(number as usize - 1)?),
        // A codon with no residue is the terminal stop codon (D010), and it is
        // drawn: `*` with no number.
        None if map.terminal_stop() => Some('*'),
        None => None,
    }
}

/// The cells of the selected CDS's residue line: one letter per residue of
/// `on`, over its codon's middle base, and with `colours` on the other two bases
/// of every codon whose letter has a Zappo group, drawn as spaces with that
/// group's background behind them and the letter itself black over it (contract
/// 8m5 §4 rule 2). The terminal stop's `*` and any symbol `zappo` has no group
/// for keep `residue_style`'s look with no background; with colours off the line
/// is what it was before, one letter cell per residue and nothing else.
///
/// A codon split between two lines, by a line's edge or the origin, is coloured
/// on each of them for the bases that one holds, and its letter stands only on
/// the one that holds its middle base.
fn cds_cells(
    map: &CdsMap,
    on: &[Residue],
    letters: &[char],
    g: u64,
    start: u64,
    end: u64,
    colours: bool,
) -> Vec<Cell> {
    let mut cells: Vec<Cell> = Vec::new();
    for residue in on {
        let style: Style = match group_of(colours, residue.symbol) {
            Some(bg) => residue_style(residue.symbol, colours)
                .fg(Color::Black)
                .bg(bg),
            None => residue_style(residue.symbol, colours),
        };
        cells.push(Cell {
            column: residue.column,
            text: residue.symbol.to_string(),
            style,
        });
    }
    if !colours {
        return cells;
    }
    for position in start..end {
        let place: CodonPlace = match map.base_residue(position) {
            Some(place) => place,
            // A base outside the CDS, or one of the leftover bases of a CDS
            // whose length is not a multiple of three, belongs to no codon and
            // carries nothing (D050).
            None => continue,
        };
        if place.place == 2 {
            // The letter over the middle base: the residues did that one.
            continue;
        }
        // The codon's first and third base, on whichever line holds them: a
        // coloured space, and nothing at all where the group has no colour.
        let symbol: char = match codon_letter(map, &place, letters) {
            Some(symbol) => symbol,
            None => continue,
        };
        if let Some(bg) = zappo(symbol) {
            cells.push(Cell {
                column: g + position - start,
                text: " ".to_string(),
                style: Style::default().bg(bg),
            });
        }
    }
    cells
}

/// The Zappo background one symbol draws, or nothing when `colours` says the
/// residues are drawn uncoloured (contract 8m5 §4 rule 3).
fn group_of(colours: bool, symbol: char) -> Option<Color> {
    if colours {
        zappo(symbol)
    } else {
        None
    }
}

/// The six levels of the xterm 6 × 6 × 6 cube (contract 8m7 §4 rule 3), in the
/// order their indexes run: the level `0` holds, the one `95` holds, and so on.
const CUBE_LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

/// Which cube level a channel falls in: the nearest of the six levels, a tie
/// going to the lower of the two. The levels are looked at from the lowest up,
/// and only a nearer level wins, so the first of two that are the same distance
/// away from the channel is the one kept.
fn cube_level(channel: u8) -> u8 {
    let mut nearest: usize = 0;
    let mut distance: u32 = u32::MAX;
    for (index, level) in CUBE_LEVELS.iter().enumerate() {
        let between: u32 = u32::from(*level).abs_diff(u32::from(channel));
        if between < distance {
            nearest = index;
            distance = between;
        }
    }
    nearest as u8
}

/// The xterm 256-colour index of the cube colour nearest `rgb` (contract 8m7
/// §4 rule 3): each channel at its nearest cube level, and the index is
/// `16 + 36·r + 6·g + b`, which is how that cube is numbered.
pub fn cube_index(rgb: [u8; 3]) -> u8 {
    16 + 36 * cube_level(rgb[0]) + 6 * cube_level(rgb[1]) + cube_level(rgb[2])
}

/// The colour one feature key draws (contract 8m7 §4 rule 2), as xterm 256
/// indexes. A key outside the table has no colour of its own, and is drawn in
/// the terminal's own.
pub fn key_colour(key: &str) -> Option<Color> {
    let index: u8 = match key {
        // what is translated, and the parts of a transcript;
        "CDS" => 214,
        "gene" => 179,
        "mRNA" | "exon" | "intron" | "5'UTR" | "3'UTR" => 141,
        // what controls a transcript, and where reading starts;
        "promoter" | "regulatory" | "terminator" | "RBS" | "enhancer" | "polyA_signal" => 77,
        // a piece of the record that copies itself;
        "rep_origin" => 75,
        // what a protein or a primer sticks to;
        "primer_bind" | "misc_binding" | "protein_bind" => 168,
        // a run of bases repeated, and the pieces that move about;
        "repeat_region" | "mobile_element" | "LTR" => 109,
        // the parts of a translated record;
        "sig_peptide" | "mat_peptide" | "transit_peptide" => 180,
        _ => return None,
    };
    Some(Color::Indexed(index))
}

/// The colour one feature draws (contract 8m7 §4 rule 3): the colour the file
/// gives it on the strand it was written on, its colour being the nearest cube
/// colour, and failing that the colour its key draws.
/// `strand` is the strand as the location wrote it, never inferred (D009).
pub fn feature_colour(feature: &ViewFeature, strand: &Strand) -> Option<Color> {
    let given: Option<[u8; 3]> = match strand {
        Strand::Minus => feature.ape_colours.reverse,
        Strand::Plus => feature.ape_colours.forward,
    };
    match given {
        Some(rgb) => Some(Color::Indexed(cube_index(rgb))),
        None => key_colour(&feature.key),
    }
}

/// The style one lane line draws (contract 8m7 §4 rule 4): its feature's colour
/// when the view's colours are on, nothing when they are off or the feature has
/// no colour; and bold when the feature is the selected CDS's own, which is
/// bold with the colours on and off alike.
fn lane_style(feature: &ViewFeature, strand: &Strand, selected: bool, colours: bool) -> Style {
    let mut style: Style = Style::default();
    let colour: Option<Color> = if colours {
        feature_colour(feature, strand)
    } else {
        None
    };
    if let Some(colour) = colour {
        style = style.fg(colour);
    }
    if selected {
        style.add_modifier |= Modifier::BOLD;
    }
    style
}

/// The residue-number cells of one line (D055, D010). Every residue numbered
/// a multiple of 5 goes first, in reading order, then the line's leftmost
/// numbered residue; each is centred on its residue, moved inward to stay over
/// the line's bases, and dropped when it would touch — even diagonally — a
/// number already placed.
///
/// `bases_end` is the column just past the line's last base.
fn number_cells(on: &[Residue], g: u64, bases_end: u64) -> Vec<Cell> {
    let numbered: Vec<(u64, u64)> = on
        .iter()
        .filter_map(|residue| Some((residue.column, residue.number?)))
        .collect();
    let mut order: Vec<(u64, u64)> = numbered
        .iter()
        .filter(|(_, number)| number % 5 == 0)
        .copied()
        .collect();
    if let Some(leftmost) = numbered.iter().min_by_key(|(column, _)| *column) {
        if !order.contains(leftmost) {
            order.push(*leftmost);
        }
    }

    let mut kept: Vec<(u64, String)> = Vec::new();
    for (column, number) in order {
        let text: String = number.to_string();
        let width: u64 = text.chars().count() as u64;
        let centre: u64 = column.saturating_sub((width - 1) / 2);
        let inward: u64 = bases_end.saturating_sub(width);
        let first: u64 = g.max(centre.min(inward));
        let last: u64 = first + width;
        // `first > end` and `last < start`, not `>=` and `<=`: one blank column
        // between two numbers is what "touch" means, so adjacent numbers drop
        // the later one.
        let clear: bool = kept.iter().all(|(start, kept_text)| {
            first > start + kept_text.chars().count() as u64 || last < *start
        });
        if clear {
            kept.push((first, text));
        }
    }
    kept.sort();
    kept.into_iter()
        .map(|(column, text)| Cell {
            column,
            text,
            style: Style::default(),
        })
        .collect()
}

/// The ruler line of a block (M8-33, contract 8m10 rule 2): the tick every
/// tenth base of the block stands under, and every thirtieth base carries its
/// own number.
///
/// The base at zero-based `position` of the block `[start, end)` is counted from
/// 1 as `count = position + 1`, and stands at column `g + position - start`. A
/// `count` that is a multiple of 10 gets a tick in that column; a multiple of
/// 30 gets its number instead, written right-aligned so that its last digit
/// stands in that column. A number whose first digit would stand left of `g`
/// has no room for its digits, and the tick goes into the base's column there.
/// Nothing else on the line holds anything — not the gutter, which holds no
/// label — and the whole line is dim and uncoloured, the colours on and off
/// alike (M8-33's secondary text).
fn ruler_cells(g: u64, start: u64, end: u64) -> Vec<Cell> {
    let dim: Style = Style::default().add_modifier(Modifier::DIM);
    let mut cells: Vec<Cell> = Vec::new();
    for position in start..end {
        let count: u64 = position + 1;
        if !count.is_multiple_of(30) && !count.is_multiple_of(10) {
            continue;
        }
        let column: u64 = g + position - start;
        if count.is_multiple_of(30) {
            let text: String = count.to_string();
            let width: u64 = text.chars().count() as u64;
            // The number ends on its own base, so its first digit stands at
            // `column - (width - 1)`, and that must not be left of `g`, where
            // the line's bases start. A number holds at most 8 digits (D056's
            // 64 MiB) and the ticks are 10 columns apart, so a number never
            // covers the tick two bases before it — `validate_cells` of the
            // column tables checks it, and `layout-cells.tsv` holds it.
            let first: u64 = column.saturating_sub(width - 1);
            if first >= g {
                cells.push(Cell {
                    column: first,
                    text,
                    style: dim,
                });
                continue;
            }
        }
        cells.push(Cell {
            column,
            text: TICK.to_string(),
            style: dim,
        });
    }
    cells
}

/// One frame's line: its label in the gutter and the residues of that frame
/// whose middle base stands in the block, at the column of that base.
///
/// When the `colours` flag is on, the window given to
/// `frame_residues` widens by one base on each side,
/// clamped to the record's length, so that a codon standing just
/// outside the block still colours the bases this line holds at its edge
/// (contract 8m5 §4 rule 2). Frames never run across the origin (D052), so a
/// codon is the three bases `middle - 1 ..= middle + 1`, whatever its strand.
// The record, the strand, the frame, the gutter widths and the block's range
// are the layout's own; `colours` is 8m5's flag. The group of arguments is the
// one `cds_cells` and the lane lines take, so no struct is added for it here.
#[allow(clippy::too_many_arguments)]
fn frame_cells(
    record: &ViewRecord,
    strand: &Strand,
    frame: ReadingFrame,
    g: u64,
    d: u64,
    start: u64,
    end: u64,
    colours: bool,
) -> Result<Vec<Cell>, LayoutProblem> {
    let length: u64 = record.sequence.length();
    let (first, last): (u64, u64) = (start.saturating_sub(1), (end + 1).min(length));
    let residues = frame_residues(&record.sequence, strand.clone(), frame, first, last)
        .map_err(LayoutProblem::Domain)?;
    let sign: char = if strand_is_plus(strand) { '+' } else { '-' };
    let mut cells: Vec<Cell> = vec![label_cell(&format!("{sign}{}", frame.index()), d)];
    for residue in residues {
        // The letters are exactly the ones before the widening: only a middle
        // base this line holds draws one.
        let draws: bool = residue.middle >= start && residue.middle < end;
        let colour: Option<Color> = group_of(colours, residue.symbol);
        if !draws && colour.is_none() {
            // A codon the widening brought in, and nothing to put on this line
            // for it.
            continue;
        }
        let middle: u64 = residue.middle;
        for base in middle.saturating_sub(1)..=middle.saturating_add(1) {
            if base < start || base >= end {
                // A base of the codon the next line holds: that line colours it.
                continue;
            }
            let column: u64 = g + base - start;
            if base == middle {
                if !draws {
                    continue;
                }
                let style: Style = match colour {
                    Some(bg) => residue_style(residue.symbol, colours)
                        .fg(Color::Black)
                        .bg(bg),
                    None => residue_style(residue.symbol, colours),
                };
                cells.push(Cell {
                    column,
                    text: residue.symbol.to_string(),
                    style,
                });
            } else if let Some(bg) = colour {
                cells.push(Cell {
                    column,
                    text: " ".to_string(),
                    style: Style::default().bg(bg),
                });
            }
        }
    }
    Ok(cells)
}

/// The lines of one block, in the order they draw (D055, plan §5.1).
///
/// Every line's text starts at column 0 and never runs past its last item: the
/// gaps between items are spaces, and there are no trailing ones, except the
/// coloured spaces of a codon's outer bases (M8-29).
///
/// `block` names an entry of `layout.blocks`; the caller holds the layout it
/// got `block` from, so an index outside it is a bug in the caller and panics.
/// `drawing` is how the lines are drawn: `drawing.colours` whether the residue
/// lines draw the Zappo backgrounds (M8-29, contract 8m5 §4 rule 3), and
/// `drawing.features` which features get lanes (M8-30, contract 8m6 §4 rule 2).
pub fn block_lines(
    record: &ViewRecord,
    layout: &Layout,
    mode: &LaneMode,
    block: usize,
    drawing: &Drawing,
) -> Result<Vec<(LineRole, Line<'static>)>, LayoutProblem> {
    let (start, end): (u64, u64) = layout.blocks[block];
    let d: u64 = u64::from(layout.gutter);
    let g: u64 = d + 1;

    let mut lines: Vec<(LineRole, Vec<Cell>)> = Vec::new();

    let lanes: Vec<Lane> = lanes(record, start, end, drawing.features);
    // The feature whose lane marks the selection (contract 8m7 §4 rule 4): the
    // CDS mode's own `cdss` entry's feature, and no feature when the mode
    // selects nothing. The lane is bold whether the colours are on or off.
    let selected: Option<usize> = match mode {
        LaneMode::Cds(index) => record.cdss.get(*index).map(|cds| cds.feature),
        LaneMode::Off | LaneMode::ThreeFrames | LaneMode::SixFrames => None,
    };
    for (number, lane) in lanes.iter().take(LANE_LIMIT).enumerate() {
        // The lane's own feature: `lanes` built this lane from one of
        // `record.features`, keeping its `index`, so the index is one of them.
        let feature: &ViewFeature = &record.features[lane.index];
        let style: Style = lane_style(
            feature,
            &lane.strand,
            selected == Some(lane.index),
            drawing.colours,
        );
        let cells: Vec<Cell> = lane
            .runs
            .iter()
            .map(|(first, last)| Cell {
                column: g + first - start,
                text: lane_text(&lane.name, &lane.strand, last - first),
                style,
            })
            .collect();
        lines.push((LineRole::Lane(number as u8 + 1), cells));
    }
    // What is counted here is what the level left, so under `None` the line is
    // not drawn at all (§4 rule 2).
    if lanes.len() > LANE_LIMIT {
        let text: String = format!("+{} more", lanes.len() - LANE_LIMIT);
        lines.push((
            LineRole::More,
            vec![Cell {
                column: g,
                text: text.clone(),
                style: Style::default(),
            }],
        ));
    }

    let (top, bottom) = strand_cells(record, d, g, start, end, drawing.colours)?;
    // The ruler line stands directly above the top strand in every mode, and the
    // lines below it move down by one row with it (M8-33).
    let ruler: Vec<Cell> = ruler_cells(g, start, end);

    match mode {
        LaneMode::Off => {
            lines.push((LineRole::Ruler, ruler));
            lines.push((LineRole::Top, top));
            lines.push((LineRole::Bottom, bottom));
        }
        LaneMode::Cds(index) => {
            // One reading of the CDS: its map and its residue letters come from
            // the same translation (D042).
            let cds: &Cds = usable_cds(record, *index)?;
            let translation = translate(&record.sequence, cds).map_err(LayoutProblem::Domain)?;
            let map: CdsMap = CdsMap::new(cds, &translation);
            let letters: Vec<char> = translation.residues_text().chars().collect();
            let on: Vec<Residue> = cds_residues(&map, &letters, g, start, end);
            let residues: Vec<Cell> =
                cds_cells(&map, &on, &letters, g, start, end, drawing.colours);
            let numbers: Vec<Cell> = number_cells(&on, g, g + (end - start));
            // A plus-strand CDS reads above the top strand, a minus-strand one
            // below the bottom (plan §5.1); its numbers line goes with it.
            if cds.strand().is_plus() {
                lines.push((LineRole::Residues, residues));
                lines.push((LineRole::Numbers, numbers));
                lines.push((LineRole::Ruler, ruler));
                lines.push((LineRole::Top, top));
                lines.push((LineRole::Bottom, bottom));
            } else {
                lines.push((LineRole::Ruler, ruler));
                lines.push((LineRole::Top, top));
                lines.push((LineRole::Bottom, bottom));
                lines.push((LineRole::Numbers, numbers));
                lines.push((LineRole::Residues, residues));
            }
        }
        LaneMode::ThreeFrames | LaneMode::SixFrames => {
            // Forward frames above the top strand, `+1` nearest it; reverse
            // frames below the bottom strand, `-1` nearest it (D055).
            for frame in [
                ReadingFrame::Third,
                ReadingFrame::Second,
                ReadingFrame::First,
            ] {
                let cells: Vec<Cell> = frame_cells(
                    record,
                    &Strand::Plus,
                    frame,
                    g,
                    d,
                    start,
                    end,
                    drawing.colours,
                )?;
                lines.push((LineRole::Frame(Strand::Plus, frame), cells));
            }
            lines.push((LineRole::Ruler, ruler));
            lines.push((LineRole::Top, top));
            lines.push((LineRole::Bottom, bottom));
            if *mode == LaneMode::SixFrames {
                for frame in [
                    ReadingFrame::First,
                    ReadingFrame::Second,
                    ReadingFrame::Third,
                ] {
                    let cells: Vec<Cell> = frame_cells(
                        record,
                        &Strand::Minus,
                        frame,
                        g,
                        d,
                        start,
                        end,
                        drawing.colours,
                    )?;
                    lines.push((LineRole::Frame(Strand::Minus, frame), cells));
                }
            }
        }
    }

    Ok(lines
        .into_iter()
        .map(|(role, cells)| (role, drawn_line(cells)))
        .collect())
}

/// One line's cells as a `Line`: text from column 0, a span of spaces for each
/// gap, and nothing past the last item.
fn drawn_line(mut cells: Vec<Cell>) -> Line<'static> {
    cells.sort_by_key(|cell| cell.column);
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut column: u64 = 0;
    for cell in cells {
        if cell.column > column {
            spans.push(Span::from(" ".repeat((cell.column - column) as usize)));
        }
        column = cell.column + cell.text.chars().count() as u64;
        spans.push(Span::styled(cell.text, cell.style));
    }
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use luria_core::identity::Identifier;
    use luria_core::view_record::open_record;
    use ratatui::backend::TestBackend;
    use ratatui::widgets::Paragraph;
    use ratatui::Terminal;
    use std::fs;
    use std::path::Path;

    /// One fixture record, as `open_record` reads it: format by content, never
    /// by name (D049), so these paths name files and the sniff decides.
    fn record(path: &str, name: &str) -> ViewRecord {
        let text: String = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
            .unwrap_or_else(|error| panic!("read {path}: {error}"));
        open_record(Identifier::create(name.to_string()).unwrap(), text)
            .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
    }

    fn pbr322() -> ViewRecord {
        record(
            "../../tests/fixtures/genbank/raw/J01749.1.gb",
            "J01749.1.gb",
        )
    }

    fn synthetic() -> ViewRecord {
        record("../../tests/fixtures/genbank/synthetic.gb", "synthetic.gb")
    }

    /// Draw `lines` `width` columns wide on a test backend, and return the
    /// symbol of every cell, row by row (the `review_view.rs` idiom).
    fn drawn_cells(lines: &[(LineRole, Line<'static>)], width: u16) -> Vec<Vec<String>> {
        drawn(lines, width).0
    }

    /// The same rows with the style of every cell, for a style assertion.
    fn drawn(
        lines: &[(LineRole, Line<'static>)],
        width: u16,
    ) -> (Vec<Vec<String>>, Vec<Vec<Style>>) {
        let height: u16 = lines.len() as u16;
        let text: Vec<Line<'static>> = lines.iter().map(|(_, line)| line.clone()).collect();
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(move |frame| {
                frame.render_widget(Paragraph::new(text.clone()), frame.area());
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        let width: usize = buffer.area.width as usize;
        let symbols: Vec<Vec<String>> = buffer
            .content()
            .chunks(width)
            .map(|row| row.iter().map(|cell| cell.symbol().to_string()).collect())
            .collect();
        let styles: Vec<Vec<Style>> = buffer
            .content()
            .chunks(width)
            .map(|row| row.iter().map(|cell| cell.style()).collect())
            .collect();
        (symbols, styles)
    }

    /// The text of one drawn row between two columns, inclusive.
    fn span(row: &[String], first: usize, last: usize) -> String {
        row[first..=last].concat()
    }

    /// The index of the line drawn for `role`.
    fn line_of(lines: &[(LineRole, Line<'static>)], role: &LineRole) -> usize {
        lines
            .iter()
            .position(|(found, _)| found == role)
            .unwrap_or_else(|| panic!("no {} line", describe(role)))
    }

    fn describe(role: &LineRole) -> String {
        match role {
            LineRole::Lane(number) => format!("lane{number}"),
            LineRole::More => "more".to_string(),
            LineRole::Residues => "residues".to_string(),
            LineRole::Numbers => "numbers".to_string(),
            LineRole::Frame(strand, frame) => {
                let sign: char = if strand_is_plus(strand) { '+' } else { '-' };
                format!("{sign}{}", frame.index())
            }
            LineRole::Ruler => "ruler".to_string(),
            LineRole::Top => "top".to_string(),
            LineRole::Bottom => "bottom".to_string(),
        }
    }

    #[test]
    fn the_gutter_is_at_least_two_columns() {
        // A 5 bp FASTA: `digits(5)` is 1, and the gutter's minimum of 2 is a
        // rule the column tables cannot pin, because every record in them has
        // at least 10 bases (`tests/fixtures/view/README.md`, "Block layout").
        let record: ViewRecord = open_record(
            Identifier::create("t".to_string()).unwrap(),
            ">t\nATGAA\n".to_string(),
        )
        .unwrap();
        assert_eq!(record.sequence.length(), 5);

        let laid: Layout = layout(&record, 40, &LaneMode::SixFrames).unwrap();
        assert_eq!(laid.gutter, 2);
        // `40 - 2 * 2 - 2 = 34` free columns, and 34 / 3 = 11 whole steps of 3:
        // 33 bases (M8-28).
        assert_eq!(laid.line_width, 33);
        assert_eq!(laid.blocks, vec![(0, 5)]);

        let lines = block_lines(
            &record,
            &laid,
            &LaneMode::SixFrames,
            0,
            &Drawing {
                colours: false,
                features: FeatureLevel::All,
            },
        )
        .unwrap();
        let roles: Vec<LineRole> = lines.iter().map(|(role, _)| role.clone()).collect();
        assert_eq!(
            roles,
            vec![
                LineRole::Frame(Strand::Plus, ReadingFrame::Third),
                LineRole::Frame(Strand::Plus, ReadingFrame::Second),
                LineRole::Frame(Strand::Plus, ReadingFrame::First),
                LineRole::Ruler,
                LineRole::Top,
                LineRole::Bottom,
                LineRole::Frame(Strand::Minus, ReadingFrame::First),
                LineRole::Frame(Strand::Minus, ReadingFrame::Second),
                LineRole::Frame(Strand::Minus, ReadingFrame::Third),
            ]
        );

        let rows: Vec<Vec<String>> = drawn_cells(&lines, 40);
        // Bases start at column 3, so `ATGAA` holds columns 3 to 7: a residue
        // sits over its codon's middle base (D055), `E` over base 4, `*` over
        // base 3 and `M` over base 2.
        assert_eq!(span(&rows[0], 0, 1), "+3");
        assert_eq!(span(&rows[0], 6, 6), "E");
        assert_eq!(span(&rows[1], 0, 1), "+2");
        assert_eq!(span(&rows[1], 5, 5), "*");
        assert_eq!(span(&rows[2], 0, 1), "+1");
        assert_eq!(span(&rows[2], 4, 4), "M");
        // The line's left number is right-aligned in columns `[0, 2)` and its
        // right number one blank column past the last base, 5 being a multiple
        // of 5 and the line's last base.
        assert_eq!(span(&rows[4], 1, 1), "1");
        assert_eq!(span(&rows[4], 3, 7), "ATGAA");
        assert_eq!(span(&rows[4], 9, 9), "5");
        assert_eq!(span(&rows[5], 3, 7), "TACTT");
        // The reverse frames read down from the last base, and each residue
        // still sits over the middle base of its codon in record coordinates.
        assert_eq!(span(&rows[6], 0, 1), "-1");
        assert_eq!(span(&rows[6], 6, 6), "F");
        assert_eq!(span(&rows[7], 0, 1), "-2");
        assert_eq!(span(&rows[7], 5, 5), "S");
        assert_eq!(span(&rows[8], 0, 1), "-3");
        assert_eq!(span(&rows[8], 4, 4), "H");
        // The ruler line is drawn and holds nothing: a 5 bp record has no tenth
        // base to mark and no thirtieth to number, so the line is blank. The
        // tables record the same for a ruler that holds nothing, no rows at all,
        // and `layout_fixtures` still counts the line it gives no rows for.
        assert!(
            rows[3][0..10].iter().all(|symbol| symbol == " "),
            "the ruler line over a 5 bp record holds nothing: {:?}",
            rows[3][0..10].concat()
        );
    }

    #[test]
    fn a_narrow_width_is_refused() {
        let record: ViewRecord = pbr322();
        // `d = 4`, `step = 3`: 12 columns leave room for no whole step, and 13
        // is the narrowest usable width (M8-28).
        let narrow: Result<Layout, LayoutProblem> = layout(&record, 12, &LaneMode::ThreeFrames);
        assert_eq!(
            narrow,
            Err(LayoutProblem::TooNarrow {
                width: 12,
                needed: 13
            })
        );

        let laid: Layout = layout(&record, 13, &LaneMode::ThreeFrames).unwrap();
        assert_eq!(laid.gutter, 4);
        assert_eq!(laid.line_width, 3);
        // 4361 bases in blocks of 3 make 1454 blocks, the last holding the 2
        // bases 4361 = 1453 * 3 + 2 leaves over: never a whole step, never 0.
        assert_eq!(laid.blocks.len(), 1454);
        assert_eq!(laid.blocks[1453], (4359, 4361));
    }

    #[test]
    fn a_lane_name_longer_than_its_run_is_cut() {
        // pBR322, 40 columns, `tet` on 30-base lines: block 2 is the line from
        // 61, shortened to 88 (D055). Its `tet` lanes cover 3 columns each and
        // `misc_binding`'s covers 4, and each run spells its name from the
        // first column of the run.
        let record: ViewRecord = pbr322();
        let laid: Layout = layout(&record, 40, &LaneMode::Cds(0)).unwrap();
        assert_eq!(laid.blocks[2], (60, 88));

        let lines = block_lines(
            &record,
            &laid,
            &LaneMode::Cds(0),
            2,
            &Drawing {
                colours: false,
                features: FeatureLevel::All,
            },
        )
        .unwrap();
        let rows: Vec<Vec<String>> = drawn_cells(&lines, 40);
        let lane1: usize = line_of(&lines, &LineRole::Lane(1));
        assert_eq!(span(&rows[lane1], 30, 32), "tet");
        let lane3: usize = line_of(&lines, &LineRole::Lane(3));
        assert_eq!(span(&rows[lane3], 11, 14), "misc");
    }

    #[test]
    fn an_unusable_cds_is_refused() {
        // pBR322's ROP CDS has no id, so `import_cds` refuses it and the view
        // cannot select it (`tests/fixtures/genbank/pbr322-cds.tsv`, D041).
        let record: ViewRecord = pbr322();
        assert!(matches!(record.cdss[1].status, CdsStatus::Unusable(_)));
        assert_eq!(
            layout(&record, 78, &LaneMode::Cds(1)),
            Err(LayoutProblem::NotUsable(1))
        );
        let laid: Layout = layout(&record, 78, &LaneMode::Off).unwrap();
        assert_eq!(
            block_lines(
                &record,
                &laid,
                &LaneMode::Cds(1),
                0,
                &Drawing {
                    colours: false,
                    features: FeatureLevel::All,
                },
            ),
            Err(LayoutProblem::NotUsable(1))
        );
    }

    #[test]
    fn the_stop_is_bold_red() {
        // pj = join(55..60,1..12) crosses the origin (D036), so one line holds
        // its codons in two stretches: codon 1 (bases 55-57) puts its `M` at
        // column 58, codon 3 (bases 1-3) puts another at column 4, and codon 6
        // (bases 10-12) is the terminal stop whose `*` sits over base 11 at
        // column 13. Neither letter style comes from the strand or the column.
        let record: ViewRecord = synthetic();
        assert_eq!(record.cdss[2].id.as_deref(), Some("pj"));
        let laid: Layout = layout(&record, 78, &LaneMode::Cds(2)).unwrap();
        let lines = block_lines(
            &record,
            &laid,
            &LaneMode::Cds(2),
            0,
            &Drawing {
                colours: true,
                features: FeatureLevel::All,
            },
        )
        .unwrap();
        let (symbols, styles) = drawn(&lines, 78);
        let residues: usize = lines
            .iter()
            .position(|(role, _)| *role == LineRole::Residues)
            .unwrap();
        let stop: Style = styles[residues][13];
        assert_eq!(stop.fg, Some(Color::Red));
        assert!(stop.add_modifier.contains(Modifier::BOLD));
        // Codon 1 (bases 55-57) crosses the origin and puts its `M` at column
        // 58; codon 3 (bases 1-3) puts another at column 4. Neither is styled:
        // the rule is about the codon, not the letter, the column or the line.
        for column in [4usize, 58] {
            let letter: Style = styles[residues][column];
            assert_ne!(
                letter.fg,
                Some(Color::Red),
                "the `M` at column {column} is not red: {letter:?}"
            );
            assert!(
                !letter.add_modifier.contains(Modifier::BOLD),
                "the `M` at column {column} is not bold: {letter:?}"
            );
        }
        let styled: Vec<usize> = (0..70usize)
            .filter(|column| {
                let style: Style = styles[residues][*column];
                style.fg == Some(Color::Red) && style.add_modifier.contains(Modifier::BOLD)
            })
            .collect();
        assert_eq!(
            styled,
            vec![13],
            "the `*` at column 13 is the residues line's only bold red cell"
        );

        // Rule 5: with the colours off the stop keeps its bold and loses its red,
        // and nothing else on the line gains a colour. The block is the same one,
        // so its residues line keeps the row and the column it had.
        let off: Vec<(LineRole, Line<'static>)> = block_lines(
            &record,
            &laid,
            &LaneMode::Cds(2),
            0,
            &Drawing {
                colours: false,
                features: FeatureLevel::All,
            },
        )
        .unwrap();
        let (off_symbols, off_styles) = drawn(&off, 78);
        let off_residues: usize = line_of(&off, &LineRole::Residues);
        let off_stop: Style = off_styles[off_residues][13];
        // What the drawn cell holds is what a terminal resolves, so an unset
        // foreground reads as `Reset`; what matters is that no colour is drawn.
        assert!(
            matches!(off_stop.fg, None | Some(Color::Reset)),
            "the stop has no colour to lose: {off_stop:?}"
        );
        assert!(
            matches!(off_stop.bg, None | Some(Color::Reset)),
            "and no background either: {off_stop:?}"
        );
        assert!(
            off_stop.add_modifier.contains(Modifier::BOLD),
            "the stop keeps its bold: {off_stop:?}"
        );
        assert_eq!(
            off_symbols[off_residues][13], "*",
            "the same letter in the same column"
        );
        // Rule 5 over the whole line: with the colours off, no cell of the residues
        // line carries a colour, and the letters themselves are still there.
        let coloured: Vec<usize> = (0..70usize)
            .filter(|column| {
                let style: Style = off_styles[off_residues][*column];
                matches!(style.fg, Some(colour) if colour != Color::Reset)
                    || matches!(style.bg, Some(colour) if colour != Color::Reset)
            })
            .collect();
        assert_eq!(
            coloured,
            Vec::<usize>::new(),
            "the residues line of a view with the colours off holds no colour cell"
        );
        assert_eq!(
            off_symbols, symbols,
            "the colours-off drawing holds the same characters in the same places as \
             the colours-on drawing, and differs in colour only"
        );
    }

    /// §5's U1: the twenty letters, each with the colour the Zappo group of §4
    /// rule 1's table draws for it, and no colour at all for a symbol outside
    /// the twenty — the stop's `*`, an unreadable codon's `?`, a letter the
    /// record cannot read a codon for.
    #[test]
    fn every_letter_of_the_seven_groups_has_its_colour() {
        let table: [(char, u8); 20] = [
            ('I', 217),
            ('L', 217),
            ('V', 217),
            ('A', 217),
            ('M', 217),
            ('F', 220),
            ('W', 220),
            ('Y', 220),
            ('K', 63),
            ('R', 63),
            ('H', 63),
            ('D', 196),
            ('E', 196),
            ('S', 46),
            ('T', 46),
            ('N', 46),
            ('Q', 46),
            ('P', 201),
            ('G', 201),
            ('C', 226),
        ];
        for (letter, index) in table {
            assert_eq!(
                zappo(letter),
                Some(Color::Indexed(index)),
                "`{letter}` is the group colour {index}"
            );
        }
        let mut once: Vec<char> = table.iter().map(|(letter, _)| *letter).collect();
        once.sort();
        once.dedup();
        assert_eq!(
            once.len(),
            20,
            "the table holds all twenty letters, each once: {once:?}"
        );
        for symbol in ['*', '?', 'X', 'B', 'Z', 'J'] {
            assert_eq!(zappo(symbol), None, "no colour for `{symbol}`");
        }
    }

    // §5's U1 (contract 8m7 §4 rules 2 and 3). Every index here is worked out
    // from §4 rule 3's own formula, not from what the code answers: the levels
    // are 0, 95, 135, 175, 215 and 255, and the index is `16 + 36·r + 6·g + b`.
    //
    //   * `[255,0,0]`: r = 5, g = 0, b = 0 → 16 + 180 = 196.
    //   * `[0,255,0]`: g = 5 → 16 + 30 = 46.
    //   * `[0x84,0xb0,0xdc]` is [132,176,220]: 132 is nearest 135 (3 away, not
    //     37 from 95), 176 is nearest 175, 220 nearest 215, so r = 2, g = 3,
    //     b = 4 → 16 + 72 + 18 + 4 = 110.
    //   * `[0,0,0]` is the cube's first colour, 16; `[255,255,255]` is its
    //     last, 16 + 180 + 30 + 5 = 231.
    //     135 and 175, 175 and 215), and a tie goes to the lower level, so
    //     `[115,155,195]` keeps 95, 135 and 175 — r = 1, g = 2, b = 3 — and the
    //     index is 16 + 36 + 12 + 3 = 67; and `[235,115,0]` keeps 215, 95 and
    //     0, which is 16 + 144 + 6 + 0 = 166.
    #[test]
    fn the_nearest_cube_colour_and_the_key_colours() {
        assert_eq!(cube_index([255, 0, 0]), 196, "pure red is the cube's red");
        assert_eq!(
            cube_index([0, 255, 0]),
            46,
            "pure green is the cube's green"
        );
        assert_eq!(
            cube_index([0x84, 0xb0, 0xdc]),
            110,
            "132, 176 and 220 each go to their nearest level"
        );
        assert_eq!(cube_index([0, 0, 0]), 16, "black is the cube's first");
        assert_eq!(cube_index([255, 255, 255]), 231, "white is the cube's last");
        assert_eq!(
            cube_index([115, 155, 195]),
            67,
            "a tie between two levels goes to the lower of them"
        );
        assert_eq!(
            cube_index([235, 115, 0]),
            166,
            "235 ties down to 215 as the others do"
        );

        // §4 rule 2's table, every key of it, and nothing outside it.
        for (key, index) in [
            ("CDS", 214),
            ("gene", 179),
            ("mRNA", 141),
            ("exon", 141),
            ("intron", 141),
            ("5'UTR", 141),
            ("3'UTR", 141),
            ("promoter", 77),
            ("regulatory", 77),
            ("terminator", 77),
            ("RBS", 77),
            ("enhancer", 77),
            ("polyA_signal", 77),
            ("rep_origin", 75),
            ("primer_bind", 168),
            ("misc_binding", 168),
            ("protein_bind", 168),
            ("repeat_region", 109),
            ("mobile_element", 109),
            ("LTR", 109),
            ("sig_peptide", 180),
            ("mat_peptide", 180),
            ("transit_peptide", 180),
        ] {
            assert_eq!(
                key_colour(key),
                Some(Color::Indexed(index)),
                "`{key}` draws colour {index}"
            );
        }
        for key in [
            "old_sequence",
            "source",
            "misc_feature",
            "primersite",
            "CDS ",
            "genex",
        ] {
            assert_eq!(
                key_colour(key),
                None,
                "`{key}` has no colour of its own, the terminal's own"
            );
        }
    }

    /// The cells `number_cells` places, as the column they start in and the
    /// text they place, in the order the function sorts them into.
    fn placed(cells: &[Cell]) -> Vec<(u64, &str)> {
        cells
            .iter()
            .map(|cell| (cell.column, cell.text.as_str()))
            .collect()
    }

    #[test]
    fn a_number_touching_a_placed_one_is_dropped() {
        // D050's touch rule, which no block in `layout-cells.tsv` reaches: no
        // tabled line holds two numbered residues close enough together. The
        // residues are given here, not read from a record: residue 120 is a
        // multiple of five so it is placed first, centred on its residue at
        // column 13 and covering 12-14. Residue 119 is the line's leftmost
        // numbered residue, and centred on column 10 it would cover 9-11: its
        // last column is the column `120` starts in, and one blank column
        // between two numbers is what a touch means, so `119` is the one
        // dropped rather than the one that was placed first.
        let on: [Residue; 2] = [
            Residue {
                codon: 1,
                column: 10,
                symbol: 'A',
                number: Some(119),
            },
            Residue {
                codon: 2,
                column: 13,
                symbol: 'A',
                number: Some(120),
            },
        ];
        let cells: Vec<Cell> = number_cells(&on, 4, 64);
        assert_eq!(
            placed(&cells),
            vec![(12, "120")],
            "`120` keeps column 12 and `119`, which would end where it starts, is dropped"
        );
    }

    #[test]
    fn a_number_is_clamped_inside_the_bases() {
        // D050 again, and again no tabled block reaches it: `bases_end` is the
        // column just past the line's bases, so a line whose last base sits at
        // column 64 has `bases_end` of 65. Residue 1000 sits at column 63, and
        // centring four digits on it would start at 62 and cover 62-65, one
        // column past the last base, so the number is moved inward to start at
        // 65 - 4 = 61 and covers 61-64.
        let on: [Residue; 1] = [Residue {
            codon: 1,
            column: 63,
            symbol: 'A',
            number: Some(1000),
        }];
        let cells: Vec<Cell> = number_cells(&on, 5, 65);
        assert_eq!(
            placed(&cells),
            vec![(61, "1000")],
            "`1000` is clamped to the four columns that stay over the bases"
        );
    }
}
