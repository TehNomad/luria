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
//! * **Line width.** The largest multiple of 10 (no translation) or of 30 (any
//!   translation mode) with `2d + 2 + N <= width`. A width with no whole step
//!   of room is refused `TooNarrow`.
//! * **A shortened line, not a shifted grid** (D055). Without a CDS selected,
//!   block `k` is `[kN, (k+1)N)`. With one, lines still start at base 1, and a
//!   line that would end inside a codon of the selected CDS ends just before
//!   that codon, so it is 1 or 2 bases short and the lines after it continue
//!   from there. Every codon fact comes from `CdsMap`: the two bases either
//!   side of the edge answer whether an edge falls inside a codon, and the
//!   base before them answers which of its bases the line must stop short of.
//! * **Lines, top to bottom.** Up to four lanes, then `more`; a plus-strand
//!   CDS's residues then its numbers; `+3 +2 +1`; top; bottom; a minus-strand
//!   CDS's numbers then its residues; `-1 -2 -3`.
//! * **Lanes** (D053). Every drawn feature except `source` that has a base in
//!   the block, longest drawn length first then file order, at most four; the
//!   rest are counted on the `more` line. A lane covers the columns of that
//!   feature's bases in the block, as one run per stretch of consecutive bases,
//!   and its text is its name, an arrow for its strand, and the bar filled
//!   with `═`, cut to the run.
//! * **Residues** (D055, D052). Every residue sits over its codon's **middle**
//!   base, on whichever line holds that base: for a CDS that is the base
//!   `CdsMap` reports as place 2 of its codon, and for a frame it is the
//!   position `frame_residues` puts in `middle`, which is also the position
//!   the frame's own window test uses. A CDS's terminal stop codon is `*`; a
//!   frame's unreadable codon is `?`.
//! * **Numbers** (D055, D010). The selected CDS only. Every residue numbered
//!   a multiple of 5, in reading order, then the line's leftmost numbered
//!   residue; each centred on its residue, moved inward to stay over the
//!   line's bases, and dropped when it would touch a number already placed.
//!   The terminal stop codon has no number.
//!
//! What this module does not draw, and 8k1 does: the cursor, underline,
//! colours other than a `*` (rule 5 below), the title bar, the status line and
//! the overview inset.

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
use luria_core::view_record::ViewRecord;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// The narrowest gutter, whatever the record length says: a 5 bp record still
/// has two number columns (`tests/fixtures/view/README.md`, "Block layout":
/// the tables cannot pin this, because every record in them is at least 10
/// bases long).
const MIN_GUTTER: u16 = 2;

/// The step a line width is a multiple of with no translation (§5.1).
const PLAIN_STEP: u64 = 10;

/// The step in any translation mode, so a whole codon can stand on one line.
const TRANSLATED_STEP: u64 = 30;

/// Plan §5.1: "up to four lanes per block, the rest as `+N more`".
const LANE_LIMIT: usize = 4;

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

/// The step the line width is a multiple of: 10 bases with no translation,
/// 30 in any translation mode (§5.1).
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
/// a bar over every base says nothing.
fn lanes(record: &ViewRecord, start: u64, end: u64) -> Vec<Lane> {
    let mut out: Vec<Lane> = Vec::new();
    for feature in record.features.iter() {
        if feature.key == "source" {
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
/// Everything else is left alone — colour is 8k1's.
fn residue_style(symbol: char) -> Style {
    if symbol == '*' {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    }
}

/// A label right-aligned in the gutter: `columns [0, d)`, blank at `d`.
fn label_cell(label: &str, d: u64) -> Cell {
    Cell {
        column: d - label.chars().count() as u64,
        text: label.to_string(),
        style: Style::default(),
    }
}

/// The two strand lines of a block: the bases, their complements, and the
/// line's first and last base numbers. The right number starts one blank
/// column after the last base.
fn strand_cells(
    record: &ViewRecord,
    d: u64,
    g: u64,
    start: u64,
    end: u64,
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
    let top: Vec<Cell> = vec![
        Cell {
            column: d - left.chars().count() as u64,
            text: left,
            style: Style::default(),
        },
        Cell {
            column: g,
            text: bases.clone(),
            style: Style::default(),
        },
        Cell {
            column: g + (end - start) + 1,
            text: right,
            style: Style::default(),
        },
    ];
    let bottom: Vec<Cell> = vec![Cell {
        column: g,
        text: complement,
        style: Style::default(),
    }];
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
        let symbol: char = match place.residue {
            Some(number) => match letters.get(number as usize - 1) {
                Some(letter) => *letter,
                None => continue,
            },
            // A codon with no residue is the terminal stop codon (D010), and
            // it is drawn: `*` with no number.
            None if map.terminal_stop() => '*',
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

/// One frame's line: its label in the gutter and the residues of that frame
/// whose middle base stands in the block, at the column of that base.
fn frame_cells(
    record: &ViewRecord,
    strand: &Strand,
    frame: ReadingFrame,
    g: u64,
    d: u64,
    start: u64,
    end: u64,
) -> Result<Vec<Cell>, LayoutProblem> {
    let residues = frame_residues(&record.sequence, strand.clone(), frame, start, end)
        .map_err(LayoutProblem::Domain)?;
    let sign: char = if strand_is_plus(strand) { '+' } else { '-' };
    let mut cells: Vec<Cell> = vec![label_cell(&format!("{sign}{}", frame.index()), d)];
    for residue in residues {
        cells.push(Cell {
            column: g + residue.middle - start,
            text: residue.symbol.to_string(),
            style: residue_style(residue.symbol),
        });
    }
    Ok(cells)
}

/// The lines of one block, in the order they draw (D055, plan §5.1).
///
/// Every line's text starts at column 0 and never runs past its last item: the
/// gaps between items are spaces, and there are no trailing ones.
///
/// `block` names an entry of `layout.blocks`; the caller holds the layout it
/// got `block` from, so an index outside it is a bug in the caller and panics.
pub fn block_lines(
    record: &ViewRecord,
    layout: &Layout,
    mode: &LaneMode,
    block: usize,
) -> Result<Vec<(LineRole, Line<'static>)>, LayoutProblem> {
    let (start, end): (u64, u64) = layout.blocks[block];
    let d: u64 = u64::from(layout.gutter);
    let g: u64 = d + 1;

    let mut lines: Vec<(LineRole, Vec<Cell>)> = Vec::new();

    let lanes: Vec<Lane> = lanes(record, start, end);
    for (number, lane) in lanes.iter().take(LANE_LIMIT).enumerate() {
        let cells: Vec<Cell> = lane
            .runs
            .iter()
            .map(|(first, last)| Cell {
                column: g + first - start,
                text: lane_text(&lane.name, &lane.strand, last - first),
                style: Style::default(),
            })
            .collect();
        lines.push((LineRole::Lane(number as u8 + 1), cells));
    }
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

    let (top, bottom) = strand_cells(record, d, g, start, end)?;

    match mode {
        LaneMode::Off => {
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
            let residues: Vec<Cell> = {
                let mut by_column: Vec<&Residue> = on.iter().collect();
                by_column.sort_by_key(|residue| residue.column);
                by_column
                    .into_iter()
                    .map(|residue| Cell {
                        column: residue.column,
                        text: residue.symbol.to_string(),
                        style: residue_style(residue.symbol),
                    })
                    .collect()
            };
            let numbers: Vec<Cell> = number_cells(&on, g, g + (end - start));
            // A plus-strand CDS reads above the top strand, a minus-strand one
            // below the bottom (plan §5.1); its numbers line goes with it.
            if cds.strand().is_plus() {
                lines.push((LineRole::Residues, residues));
                lines.push((LineRole::Numbers, numbers));
                lines.push((LineRole::Top, top));
                lines.push((LineRole::Bottom, bottom));
            } else {
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
                let cells: Vec<Cell> = frame_cells(record, &Strand::Plus, frame, g, d, start, end)?;
                lines.push((LineRole::Frame(Strand::Plus, frame), cells));
            }
            lines.push((LineRole::Top, top));
            lines.push((LineRole::Bottom, bottom));
            if *mode == LaneMode::SixFrames {
                for frame in [
                    ReadingFrame::First,
                    ReadingFrame::Second,
                    ReadingFrame::Third,
                ] {
                    let cells: Vec<Cell> =
                        frame_cells(record, &Strand::Minus, frame, g, d, start, end)?;
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
        assert_eq!(laid.line_width, 30);
        assert_eq!(laid.blocks, vec![(0, 5)]);

        let lines = block_lines(&record, &laid, &LaneMode::SixFrames, 0).unwrap();
        let roles: Vec<LineRole> = lines.iter().map(|(role, _)| role.clone()).collect();
        assert_eq!(
            roles,
            vec![
                LineRole::Frame(Strand::Plus, ReadingFrame::Third),
                LineRole::Frame(Strand::Plus, ReadingFrame::Second),
                LineRole::Frame(Strand::Plus, ReadingFrame::First),
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
        assert_eq!(span(&rows[3], 1, 1), "1");
        assert_eq!(span(&rows[3], 3, 7), "ATGAA");
        assert_eq!(span(&rows[3], 9, 9), "5");
        assert_eq!(span(&rows[4], 3, 7), "TACTT");
        // The reverse frames read down from the last base, and each residue
        // still sits over the middle base of its codon in record coordinates.
        assert_eq!(span(&rows[5], 0, 1), "-1");
        assert_eq!(span(&rows[5], 6, 6), "F");
        assert_eq!(span(&rows[6], 0, 1), "-2");
        assert_eq!(span(&rows[6], 5, 5), "S");
        assert_eq!(span(&rows[7], 0, 1), "-3");
        assert_eq!(span(&rows[7], 4, 4), "H");
    }

    #[test]
    fn a_narrow_width_is_refused() {
        let record: ViewRecord = pbr322();
        // `d = 4`, `step = 30`: 39 columns leave room for no whole step.
        let narrow: Result<Layout, LayoutProblem> = layout(&record, 39, &LaneMode::ThreeFrames);
        assert_eq!(
            narrow,
            Err(LayoutProblem::TooNarrow {
                width: 39,
                needed: 40
            })
        );

        let laid: Layout = layout(&record, 40, &LaneMode::ThreeFrames).unwrap();
        assert_eq!(laid.gutter, 4);
        assert_eq!(laid.line_width, 30);
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

        let lines = block_lines(&record, &laid, &LaneMode::Cds(0), 2).unwrap();
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
            block_lines(&record, &laid, &LaneMode::Cds(1), 0),
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
        let lines = block_lines(&record, &laid, &LaneMode::Cds(2), 0).unwrap();
        let (_, styles) = drawn(&lines, 78);
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
