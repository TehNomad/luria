//! The overview strip's three lines (contract 8l2 §4, contract 8m12 §4; the plan
//! §4.6 and proposal §2.1 of `docs/tasks/contracts/milestone-8-view.md`; D054): the whole
//! record scaled to the width with the longest features as bars, under it the part
//! on screen, the cursor and the search hits, and under them the GC content of
//! every column as a block height.
//!
//! Every column comes from core: `overview_columns` says what the columns are and
//! `overview_column` says which column a base is drawn in (D054's floor formula).
//! A feature crossing the origin is two intervals (D036), so each interval is
//! mapped on its own and the bar comes out at both ends of the strip.
//!
//! Line 1 is `1 `, the strip's `w` cells and ` {length}`; lines 2 and 3 have the
//! same columns, so a cell of one line sits over the same cell of the other. `w`
//! is what the frame's inner width has left once `1 ` and ` {length}` have taken
//! theirs, and never more columns than the record has bases (D054): pBR322's 4361
//! bases on 78 inner columns get 71 columns of 61 and 60 bases, and a 62 bp record
//! gets 62 columns of one base each and a strip that much shorter.
//!
//! Line 3 holds nothing but the `w` cells: `gc_line` is called with no head and no
//! tail, and `super::screen` puts the two blank cells and ` GC` around it. Since a
//! whole record is counted for it, the line is built once per width and kept, not
//! drawn from the bases every frame (8m12 §4 rule 5, D056).
//!
//! Bars go longest first — bases covered, ties in file order — which is the order
//! `layout::lanes` gives its lanes, and a feature any of whose columns another bar
//! already holds is skipped: what does not fit stays in the `f` list (the plan
//! §4.6). `source` has no bar, as it has no lane (D053). The bars are coloured by
//! the same rule the lanes are (M8-31, contract 8m7 §4 rule 5), and nothing else
//! on either line is coloured; nothing on them moves, and the strip is display
//! only.

use luria_core::display_location::DisplayLocation;
use luria_core::overview::overview_column;
use luria_core::overview::overview_columns;
use luria_core::strand::Strand;
use luria_core::view_record::FeatureShape;
use luria_core::view_record::ViewFeature;
use luria_core::view_record::ViewRecord;
use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::sequence_view::layout::feature_colour;

/// What names the strip's first line: the record read from its first base (D006),
/// the same `1 ` the plan's §2.1 example shows. Its two characters are taken off
/// the width before the strip is counted out.
const HEAD: &str = "1 ";

/// The cell no bar covers: the record with nothing on it, drawn unbroken.
const BAR: char = '─';

/// The bar of a feature: `═` continues a run whose name fits and fills a run
/// whose name does not (8l2 §4 rule 4).
const FILL: char = '═';

/// The columns of the bases on screen, the start of a hit, and the cursor. Where
/// two fall in one cell `▲` wins, then `·`, then `▀` (8l2 §4 rule 5), which is the
/// order `mark_line` paints them in.
const ON_SCREEN: char = '▀';
const HIT: char = '·';
const CURSOR: char = '▲';

/// The strand arrows, the same glyphs `layout` draws on a lane, for the strand as
/// the file wrote it and never inferred (D009).
const PLUS: char = '▸';
const MINUS: char = '◂';

/// The eight block heights of the GC line, indexed by the column's GC fraction on
/// the absolute scale: 0 % is `▁` and 100 % is `█` (8m12 §4 rule 2).
const HEIGHTS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// One feature a bar could be drawn for: its name as the `f` list shows it, the
/// strand it was written on, and the intervals it covers in the order the record
/// gives them (D036), which is two runs for a feature across the origin.
struct Bar {
    /// The feature's index into `ViewRecord::features`, its place in file order.
    index: usize,
    name: String,
    strand: Strand,
    runs: Vec<(u64, u64)>,
    /// Bases covered over all its runs, which is what bars are ordered by.
    length: u64,
}

/// The strip's two lines for an inner width: line 1 is `1 `, the strip, `
/// {length}`; line 2 has the same columns. `shown` is the zero-based first and
/// last base on screen; `hits` are the shown hits' starts (8l2 §4).
///
/// Both lines are exactly `2 + w + 1 + {length}'s digits` characters, which never
/// exceeds `inner_width`; `w` is 0 when the width has nothing left for the strip,
/// and then line 1 is `1 ` and ` {length}` side by side.
pub fn overview_lines(
    record: &ViewRecord,
    inner_width: u16,
    shown: (u64, u64),
    cursor: u64,
    hits: &[u64],
) -> (String, String) {
    let built: Strip = strip(record, inner_width, shown, cursor, hits, false);
    (built.text, built.marks)
}

/// The strip's two lines as the screen draws them (8m7 §4 rule 5): the bar line
/// as spans that carry each feature's colour, and the marks line as text, which
/// carries nothing — 8m7 colours nothing on line 2. `colours` is the view's `C`:
/// with it off, no cell of the bar line is coloured.
pub fn overview_drawn(
    record: &ViewRecord,
    inner_width: u16,
    shown: (u64, u64),
    cursor: u64,
    hits: &[u64],
    colours: bool,
) -> (Line<'static>, String) {
    let built: Strip = strip(record, inner_width, shown, cursor, hits, colours);
    (built.line, built.marks)
}

/// The strip's two lines, built from the bar line's cells once: its text, its
/// styled spans and the marks line's text. `colours` is the view's `C` (8m7 §4
/// rule 5).
struct Strip {
    /// Line 1 as spans: the same text as `text`, with each feature's run
    /// carrying its colour.
    line: Line<'static>,
    /// Line 1 as text, which is what `overview_lines` returns.
    text: String,
    /// Line 2, unchanged by 8m7.
    marks: String,
}

/// The strip's cells, counted out once and read by every part of it: the bar
/// line's cells, the two ways of reading line 1 and the marks line below it.
fn strip(
    record: &ViewRecord,
    inner_width: u16,
    shown: (u64, u64),
    cursor: u64,
    hits: &[u64],
    colours: bool,
) -> Strip {
    let length: u64 = record.sequence.length();
    let label: String = length.to_string();
    // `1 ` takes two columns and ` {length}` one more than its digits; the strip
    // has the rest, and never more columns than bases (D054).
    let columns: u64 = strip_columns(record, inner_width);
    // Core decides the columns themselves; the count is what they fill, and both
    // lines are built cell by cell over it.
    let cells: usize = overview_columns(length, columns).len();
    let bars: BarCells = bar_cells(record, length, columns, cells, colours);
    Strip {
        line: Line::from(bars.spans(&label)),
        text: bars.text(&label),
        marks: mark_line(length, columns, cells, shown, cursor, hits, &label),
    }
}

/// The columns the strip has for an inner width: `w`, the same count line 1 draws
/// and `gc_line` counts bases over. `1 ` takes two columns and ` {length}` one
/// more than its digits, and the strip never has more columns than the record has
/// bases (D054).
fn strip_columns(record: &ViewRecord, inner_width: u16) -> u64 {
    let length: u64 = record.sequence.length();
    let label: usize = length.to_string().len();
    let left: u64 =
        u64::from(inner_width).saturating_sub(HEAD.chars().count() as u64 + 1 + label as u64);
    length.min(left)
}

/// The strip's third line (8m12 §4 rule 1): the strip's `w` cells for an inner
/// width, the same `w` and the same column ranges as line 1, and nothing else — no
/// head and no tail. `inner_width` is the frame's inner width line 1 is built for.
///
/// The whole record is counted for the line, so the state keeps it and does not
/// count 64 MiB of bases every frame (8m12 §4 rule 5, D056).
pub fn gc_line(record: &ViewRecord, inner_width: u16) -> String {
    let length: u64 = record.sequence.length();
    let columns: u64 = strip_columns(record, inner_width);
    overview_columns(length, columns)
        .iter()
        .map(|(first, last)| gc_glyph(record, *first, *last))
        .collect()
}

/// The height of one column, over the bases `first..last` (8m12 §4 rule 2): `G`,
/// `C` and `S` count as GC and `A`, `T` and `W` as AT, in either case, and every
/// other letter — `N`, `R`, `Y`, `K`, `M`, `B`, `D`, `H`, `V` — is in neither
/// count. A column with no counted base is a space, and the scale is absolute, so
/// 0 % is `▁` and 100 % is `█` whatever the record's own range.
fn gc_glyph(record: &ViewRecord, first: u64, last: u64) -> char {
    let mut gc: u64 = 0;
    let mut at: u64 = 0;
    for base in first..last {
        let letter: char = match record.sequence.letter_at(base) {
            Ok(letter) => letter,
            // A column's range comes from `overview_columns` and so lies inside
            // the record (D054): no base asked for here is out of bounds.
            Err(_problem) => continue,
        };
        // The letters come in the case the file wrote them, and either case of a
        // letter counts the same (8m12 §4 rule 2).
        match letter.to_ascii_uppercase() {
            'G' | 'C' | 'S' => gc += 1,
            'A' | 'T' | 'W' => at += 1,
            _ => {}
        }
    }
    let counted: u64 = gc + at;
    if counted == 0 {
        return ' ';
    }
    // `⌊8·GC / (GC + AT)⌋`: the quotient is at most 8, and 100 % is the only case
    // that reaches it, where the line has no eighth height (8m12 §4 rule 2).
    let step: usize = (8 * gc / counted).min(7) as usize;
    HEIGHTS[step]
}

/// The features a bar could be drawn for, in the order they are drawn: every
/// `Drawn` feature that is not a `source` — which describes the whole record and
/// would cover every cell (D053) — longest by bases covered first, ties in file
/// order (8l2 §4 rule 4).
fn bars(record: &ViewRecord) -> Vec<Bar> {
    let mut out: Vec<Bar> = Vec::new();
    for feature in record.features.iter() {
        if feature.key == "source" {
            continue;
        }
        let shape: &DisplayLocation = match &feature.shape {
            FeatureShape::Drawn(shape) => shape,
            // A feature this record cannot draw has no cells to cover (D053).
            FeatureShape::NotDrawn(_problem) => continue,
        };
        let runs: Vec<(u64, u64)> = shape.location.intervals();
        out.push(Bar {
            index: feature.index,
            name: feature.name.clone(),
            strand: shape.location.strand(),
            length: runs.iter().map(|(first, last)| last - first).sum(),
            runs,
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

/// The bar line's cells: one glyph and one style per overview column. The bars
/// are placed once, and the text and the colours are read from the same cells,
/// so the two cannot disagree (8m7 §4 rule 5).
struct BarCells {
    glyphs: Vec<char>,
    /// The style the cell draws, `Style::default()` for the `─` cells: no cell
    /// that is not a feature's run carries one (8m7 §4 rule 5).
    styles: Vec<Style>,
}

impl BarCells {
    /// The whole line as text: `1 `, one cell per overview column, ` {length}`.
    fn text(&self, label: &str) -> String {
        format!("{HEAD}{} {label}", self.glyphs.iter().collect::<String>())
    }

    /// The whole line as spans: the same text, every run of cells that draw the
    /// same style in one span.
    fn spans(&self, label: &str) -> Vec<Span<'static>> {
        let mut spans: Vec<Span<'static>> = vec![Span::from(HEAD)];
        let mut at: usize = 0;
        while at < self.glyphs.len() {
            let style: Style = self.styles[at];
            let mut run: String = String::new();
            while at < self.glyphs.len() && self.styles[at] == style {
                run.push(self.glyphs[at]);
                at += 1;
            }
            spans.push(Span::styled(run, style));
        }
        spans.push(Span::from(format!(" {label}")));
        spans
    }
}

/// The style one feature's bar draws (8m7 §4 rule 5): its colour when the view's
/// colours are on, as `layout` colours a lane, and no colour when they are off.
fn bar_style(feature: &ViewFeature, strand: &Strand, colours: bool) -> Style {
    match (colours, feature_colour(feature, strand)) {
        (true, Some(colour)) => Style::default().fg(colour),
        _ => Style::default(),
    }
}

/// The bar line's cells: `BAR` everywhere, and over each bar the columns of each
/// of its intervals — two runs for one across the origin — and a feature any of
/// whose columns another bar already holds is skipped whole (8l2 §4 rule 4). The
/// colours come from the feature's own rule, read once with the placement
/// (8m7 §4 rule 5).
fn bar_cells(
    record: &ViewRecord,
    length: u64,
    columns: u64,
    cells: usize,
    colours: bool,
) -> BarCells {
    let mut line: Vec<char> = vec![BAR; cells];
    let mut styles: Vec<Style> = vec![Style::default(); cells];
    let mut taken: Vec<bool> = vec![false; cells];
    for bar in bars(record) {
        // A feature with an interval that falls outside the record is not drawn at
        // all: drawing the rest of it would promise a bar that is really broken.
        let spans: Option<Vec<(usize, usize)>> = bar
            .runs
            .iter()
            .map(|run| span(length, columns, *run))
            .collect();
        let runs: Vec<(usize, usize)> = match spans {
            Some(runs) if !runs.is_empty() => runs,
            _ => continue,
        };
        if runs
            .iter()
            .any(|(first, last)| (*first..=*last).any(|cell| taken[cell]))
        {
            continue;
        }
        // The feature this bar stands for: `bars` takes every bar from one of
        // `record.features` and keeps its `index`, so the index is one of them.
        let style: Style = bar_style(&record.features[bar.index], &bar.strand, colours);
        for (first, last) in runs {
            for (cell, glyph) in
                (first..=last).zip(run_text(&bar.name, &bar.strand, last - first + 1))
            {
                line[cell] = glyph;
            }
            styles[first..=last].fill(style);
            taken[first..=last].fill(true);
        }
    }
    BarCells {
        glyphs: line,
        styles,
    }
}

/// The marks line over the same columns: `▀` for every base on screen, `·` at the
/// column of every shown hit's start and `▲` at the cursor's. They are painted in
/// that order, so the cursor wins a cell it shares, then a hit, then the screen
/// (8l2 §4 rule 5). What the label takes on line 1 is a space here.
fn mark_line(
    length: u64,
    columns: u64,
    cells: usize,
    shown: (u64, u64),
    cursor: u64,
    hits: &[u64],
    label: &str,
) -> String {
    let mut line: Vec<char> = vec![' '; cells];
    if shown.0 <= shown.1 {
        for base in shown.0..=shown.1 {
            paint(&mut line, length, columns, base, ON_SCREEN);
        }
    }
    for start in hits {
        paint(&mut line, length, columns, *start, HIT);
    }
    paint(&mut line, length, columns, cursor, CURSOR);
    let blank: String = " ".repeat(label.len());
    format!("  {} {}", line.iter().collect::<String>(), blank)
}

/// Put `glyph` in the column base `base` is drawn in, doing nothing for a base
/// outside the record or when the strip has no columns (D054).
fn paint(line: &mut [char], length: u64, columns: u64, base: u64, glyph: char) {
    if let Some(column) = overview_column(length, columns, base) {
        // The column is one of `columns` and `line` has `columns` cells, so the
        // index is in range; `overview_column` never answers past the last one.
        line[column as usize] = glyph;
    }
}

/// The cells one interval is drawn in: the column its first base falls in through
/// the column its last base falls in, both from `overview_column` (D054), so the
/// columns of a run are contiguous even when its bases are not. `None` for an
/// interval with no base of the record in it.
fn span(length: u64, columns: u64, (first, last): (u64, u64)) -> Option<(usize, usize)> {
    if first >= last || first >= length {
        return None;
    }
    let left: u64 = overview_column(length, columns, first)?;
    let right: u64 = overview_column(length, columns, last.min(length) - 1)?;
    Some((left as usize, right as usize))
}

/// What one run of `cells` columns shows: the name, a space and the strand arrow,
/// then `═` to fill — but only when the run is at least two cells longer than the
/// name, which is what the name, the space and the arrow need (8l2 §4 rule 4). A
/// shorter run is all `═`, so a bar never spells part of a name.
fn run_text(name: &str, strand: &Strand, cells: usize) -> Vec<char> {
    let arrow: char = if *strand == Strand::Plus { PLUS } else { MINUS };
    let mut out: Vec<char> = Vec::new();
    if cells >= name.chars().count() + 2 {
        out.extend(name.chars());
        out.push(' ');
        out.push(arrow);
    }
    while out.len() < cells {
        out.push(FILL);
    }
    out.truncate(cells);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use luria_core::identity::Identifier;
    use luria_core::view_record::open_record;

    /// The hand-written record the unit tests are written out against: 20 bp,
    /// circular, one `source`, a 6 bp plus-strand `gene` named `A` at bases
    /// 3..8 (1-based) and a 6 bp `repeat_region` written
    /// `complement(join(17..20,1..2))`, which crosses the origin and so covers
    /// bases `[16, 20)` and `[0, 2)` (D036). The two are the same length, so file
    /// order decides which is drawn first (8l2 §4 rule 4).
    fn twenty() -> ViewRecord {
        let text: String = concat!(
            "LOCUS       hand20                  20 bp    DNA     circular SYN 27-OCT-2026\n",
            "DEFINITION  Hand-written record for the overview strip's unit tests.\n",
            "ACCESSION   SYN0099\n",
            "VERSION     SYN0099.1\n",
            "KEYWORDS    .\n",
            "SOURCE      synthetic construct\n",
            "  ORGANISM  synthetic construct\n",
            "FEATURES             Location/Qualifiers\n",
            "     source          1..20\n",
            "                     /organism=\"synthetic construct\"\n",
            "                     /mol_type=\"other DNA\"\n",
            "     gene            3..8\n",
            "                     /gene=\"A\"\n",
            "     repeat_region   complement(join(17..20,1..2))\n",
            "ORIGIN\n",
            "        1 ACGTACGTAC GTACGTACGT\n",
            "//\n",
        )
        .to_string();
        open_record(Identifier::create("hand20".to_string()).unwrap(), text)
            .expect("the 20 bp hand-written record")
    }

    /// 20 bases on 15 inner columns: `1 ` and ` 20` take 5 columns, so the strip
    /// has 10 columns of two bases each (D054). `A` is drawn first and takes
    /// columns 1..3 of its bases 2..8, three cells: its name, the space and the
    /// arrow fill them exactly; `repeat_region`, 13 characters long and
    /// with no room for its name in the 2 cells and 1 cell of its two runs, is all
    /// `═` and sits at columns 8..9 and column 0 (D036: two runs, both ends).
    ///
    /// Bases 0..11 (inclusive) are on screen, so columns 0..5 are `▀`; the cursor
    /// at base 5 goes in column 2 and wins it from `▀`, and the hit starts at bases
    /// 9 and 17 take columns 4 and 8, the first of them winning it from `▀`.
    #[test]
    fn twenty_bases_on_fifteen_columns() {
        let (bars, marks): (String, String) = overview_lines(&twenty(), 15, (0, 11), 5, &[9, 17]);
        assert_eq!(bars, "1 ═A ▸────══ 20");
        assert_eq!(marks, "  ▀▀▲▀·▀  ·    ");
        assert_eq!(bars.chars().count(), 15);
        assert_eq!(marks.chars().count(), 15);
    }

    /// The two 6 bp features tie on bases covered, so file order draws the `gene`
    /// first; a bar whose columns another bar holds is skipped whole, which is what
    /// keeps `source` out and keeps one of a pair of identical bars from covering
    /// the same cell twice (8l2 §4 rule 4, D053).
    #[test]
    fn a_bar_already_taken_is_skipped() {
        // `source` covers every base and gets no bar at all, and the strip is all
        // `─` apart from the two features above it.
        let bars: Vec<Bar> = bars(&twenty());
        let names: Vec<&str> = bars.iter().map(|bar| bar.name.as_str()).collect();
        assert_eq!(names, ["A", "repeat_region"], "source has no bar");
        assert_eq!(bars[0].length, 6, "gene 3..8");
        assert_eq!(bars[1].length, 6, "join(17..20,1..2) is 4 + 2 bases");
        assert_eq!(
            bars[1].runs,
            vec![(16, 20), (0, 2)],
            "the origin-crossing interval pair, in the order the record gives it"
        );
    }

    /// A record shorter than the strip is drawn narrow (D054): 4 bases on 20 inner
    /// columns give 4 columns of one base and a strip 4 cells wide, and a record
    /// with nothing in it has no columns at all. Line 1 spells the length both
    /// times and line 2 has the same columns.
    #[test]
    fn a_short_record_is_drawn_narrow() {
        let record: ViewRecord = open_record(
            Identifier::create("tiny".to_string()).unwrap(),
            ">tiny\nACGT\n".to_string(),
        )
        .expect("the 4 bp FASTA record");
        let (bars, marks): (String, String) = overview_lines(&record, 20, (1, 2), 3, &[0]);
        assert_eq!(bars, "1 ──── 4");
        assert_eq!(marks, "  ·▀▀▲  ");
        // Nothing left for the strip: the label sits next to the `1` with nothing
        // between them, and line 2 has no cell to mark anything in.
        let (bare, blank): (String, String) = overview_lines(&record, 4, (0, 3), 0, &[]);
        assert_eq!(bare, "1  4");
        assert_eq!(blank, "    ");
    }
}
