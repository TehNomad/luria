//! The ruler line over the top strand (contract 8m10, M8-33): a tick at every
//! tenth base of a block and that base's number at every thirtieth, standing on
//! the line directly above the top strand in every mode.
//!
//! The columns are the planner's, read off the rules with the record: pBR322 at
//! width 78 has `d = 4` and `g = 5`, so block 0 — bases 1 to 66 — puts base 10
//! at column `5 + 9 = 14`, base 30 at 34 (its number `30` covering 33-34) and
//! base 60 at 64 (`60` at 63-64). Block 1, bases 67 to 132, starts at zero-based
//! 66, so base 70 stands at column `5 + 70 - 67 + 1 = 8`, base 90 at 28 with its
//! `90` at 27-28, and base 120 at 58 with `120` at 56-58. At width 40 the lines
//! are 30 bases, and `tet` ends the 61-88, 89-118 and 119-148 lines, which
//! 8m4's table gives: there base 120 stands at column 6 and `120` would start
//! at column 4, left of `g = 5`, so the number is not drawn and the tick goes
//! in the base's column instead. None of these values comes from what the code
//! answers.

use luria_cli::sequence_view::layout::block_lines;
use luria_cli::sequence_view::layout::layout;
use luria_cli::sequence_view::layout::Drawing;
use luria_cli::sequence_view::layout::FeatureLevel;
use luria_cli::sequence_view::layout::LaneMode;
use luria_cli::sequence_view::layout::Layout;
use luria_cli::sequence_view::layout::LineRole;
use luria_cli::sequence_view::screen::draw;
use luria_cli::sequence_view::state::ViewState;
use luria_core::identity::Identifier;
use luria_core::view_record::open_record;
use luria_core::view_record::ViewRecord;
use ratatui::backend::TestBackend;
use ratatui::buffer::Cell;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::Terminal;
use std::fs;
use std::path::Path;

/// Every screen test draws at this size: 80 columns and 24 rows, whose frame has
/// 78 inner columns, as every other sequence-view test does.
const W: u16 = 80;
const H: u16 = 24;

/// The inner width `layout` is measured at: 80 columns less the frame's two
/// border columns (8k1 §4 rule 1).
const INNER: u16 = 78;

/// The tick every tenth base stands under, as the view draws it.
const TICK: &str = "╷";

fn pbr() -> ViewRecord {
    let path: &str = "../../tests/fixtures/genbank/raw/J01749.1.gb";
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let text: String =
        fs::read_to_string(&file).unwrap_or_else(|error| panic!("read {path}: {error}"));
    open_record(Identifier::create("SYNPBR322".to_string()).unwrap(), text)
        .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
}

/// One block's lines, drawn with the view's colours as `colours` says and every
/// feature lane (`FeatureLevel::All`, 8m6 §4 rule 1).
fn lines_at(
    record: &ViewRecord,
    laid: &Layout,
    mode: &LaneMode,
    block: usize,
    colours: bool,
) -> Vec<(LineRole, Line<'static>)> {
    block_lines(
        record,
        laid,
        mode,
        block,
        &Drawing {
            colours,
            features: FeatureLevel::All,
            orfs: &[],
        },
    )
    .unwrap_or_else(|problem| panic!("block {block} at {mode:?}: {problem:?}"))
}

/// The roles of a block's lines, in the order they are drawn.
fn roles(lines: &[(LineRole, Line<'static>)]) -> Vec<LineRole> {
    lines.iter().map(|(role, _line)| role.clone()).collect()
}

/// The index of the line drawn for `role`, in a failure that names the lines the
/// block has.
fn line_index(lines: &[(LineRole, Line<'static>)], role: &LineRole) -> usize {
    lines
        .iter()
        .position(|(found, _line)| found == role)
        .unwrap_or_else(|| panic!("no {role:?} line in {:?}", roles(lines)))
}

/// The one line of `lines` whose role is `role`.
fn line_of(lines: &[(LineRole, Line<'static>)], role: &LineRole) -> Line<'static> {
    lines
        .iter()
        .find(|(found, _line)| found == role)
        .unwrap_or_else(|| panic!("no {role:?} line in {:?}", roles(lines)))
        .1
        .clone()
}

/// The cells one line places, as `(first column, text, style)`: `layout`'s
/// `drawn_line` writes one span per cell and one span per gap, and a gap holds
/// only spaces, so every other span is a cell, and the widths of the spans
/// before it give the column it starts at.
fn placed(line: &Line<'static>) -> Vec<(u64, String, Style)> {
    let mut out: Vec<(u64, String, Style)> = Vec::new();
    let mut column: u64 = 0;
    for span in line.spans.iter() {
        let width: u64 = span.content.chars().count() as u64;
        if !span.content.chars().all(|ch| ch == ' ') {
            out.push((column, span.content.to_string(), span.style));
        }
        column += width;
    }
    out
}

/// The ruler line's cells as `(column, text)`: the styles are checked by their
/// own test, and the text and its column by the tests below.
fn ruler_of(lines: &[(LineRole, Line<'static>)]) -> Vec<(u64, String)> {
    placed(&line_of(lines, &LineRole::Ruler))
        .into_iter()
        .map(|(column, text, _style)| (column, text))
        .collect()
}

/// Rule 1 and rule 2: the ruler line stands directly above the top strand, and
/// holds the tick and the number of every tenth and thirtieth base of the block.
#[test]
fn the_ruler_stands_over_the_top_strand() {
    let record: ViewRecord = pbr();
    let mode: LaneMode = LaneMode::Off;
    let laid: Layout = layout(&record, INNER, &mode).expect("a layout at width 78");
    assert_eq!(
        laid.blocks[0],
        (0, 66),
        "block 0 is bases 1 to 66, 66 bases to the line at width 78"
    );

    // Block 0: bases 1 to 66, so base 10 stands at column 5 + 9 = 14, base 20 at
    // 24, base 30 at 34 and carries its number over 33-34, base 40 at 44, base
    // 50 at 54 and base 60 at 64, carrying `60` over 63-64.
    let lines: Vec<(LineRole, Line<'static>)> = lines_at(&record, &laid, &mode, 0, false);
    let top: usize = line_index(&lines, &LineRole::Top);
    let ruler: usize = line_index(&lines, &LineRole::Ruler);
    assert_eq!(
        ruler + 1,
        top,
        "the ruler line is the line directly above the top strand"
    );
    assert_eq!(
        ruler_of(lines.as_slice()),
        vec![
            (14, TICK.to_string()),
            (24, TICK.to_string()),
            (33, "30".to_string()),
            (44, TICK.to_string()),
            (54, TICK.to_string()),
            (63, "60".to_string())
        ],
        "block 0's ruler, bases 1 to 66"
    );

    // Block 1: bases 67 to 132, which starts the line at zero-based 66. Base 70
    // stands at column 5 + 69 - 66 = 8, and base 90 at 28, with its number at
    // 27-28; base 120 stands at 58, with `120` at 56-58.
    assert_eq!(laid.blocks[1], (66, 132), "block 1 is bases 67 to 132");
    let second: Vec<(LineRole, Line<'static>)> = lines_at(&record, &laid, &mode, 1, false);
    let ruler_at: usize = line_index(&second, &LineRole::Ruler);
    assert_eq!(
        ruler_at + 1,
        line_index(&second, &LineRole::Top),
        "the ruler line is the line directly above the top strand"
    );
    assert_eq!(
        ruler_of(second.as_slice()),
        vec![
            (8, TICK.to_string()),
            (18, TICK.to_string()),
            (27, "90".to_string()),
            (38, TICK.to_string()),
            (48, TICK.to_string()),
            (56, "120".to_string()),
            (68, TICK.to_string())
        ],
        "block 1's ruler, bases 67 to 132"
    );

    // Rule 1: in the Cds mode a plus-strand CDS's residues and numbers stand
    // above the ruler, which is still the line above the top strand.
    let cds: LaneMode = LaneMode::Cds(0);
    let with_tet: Layout = layout(&record, INNER, &cds).expect("a layout at width 78");
    assert_eq!(
        with_tet.blocks[1],
        (66, 130),
        "`tet` shortens block 1 to bases 67 to 130 (D055)"
    );
    let lines: Vec<(LineRole, Line<'static>)> = lines_at(&record, &with_tet, &cds, 1, false);
    let ruler: usize = line_index(&lines, &LineRole::Ruler);
    let around: &[LineRole] = &roles(lines.as_slice());
    assert_eq!(
        &around[ruler - 1..ruler + 2],
        [LineRole::Numbers, LineRole::Ruler, LineRole::Top],
        "the numbers line, then the ruler, then the top strand"
    );
}

/// Rule 2: a number that would start left of the line's first base is not drawn,
/// and the tick goes in its base's column instead.
#[test]
fn a_number_without_room_is_a_tick() {
    let record: ViewRecord = pbr();
    let mode: LaneMode = LaneMode::Cds(0);
    let laid: Layout = layout(&record, 40, &mode).expect("a layout at width 40");
    assert_eq!(laid.line_width, 30, "30 bases to the line at width 40");
    // 8m4's table: the lines are 61-88, 89-118 and 119-148, the last two tabled
    // in `layout-cells.tsv` as blocks 3 and 4.
    assert_eq!(laid.blocks[3], (88, 118), "the line 89 to 118");
    assert_eq!(laid.blocks[4], (118, 148), "the line 119 to 148");

    // The 89-118 line, whose first base is zero-based 88: base 90 stands at
    // column 6, and `90` starts at column 5, which is the line's first base, so
    // it is drawn over 5-6; bases 100 and 110 are the ticks at 16 and 26.
    let numbers: Vec<(LineRole, Line<'static>)> = lines_at(&record, &laid, &mode, 3, false);
    assert_eq!(
        ruler_of(numbers.as_slice()),
        vec![
            (5, "90".to_string()),
            (16, TICK.to_string()),
            (26, TICK.to_string())
        ],
        "the 89-118 line has room for `90` and no more"
    );

    // The 119-148 line, whose first base is zero-based 118: base 120 stands at
    // column 6, where `120` would start at column 4, left of `g = 5`. The number
    // is not drawn, and the tick goes in base 120's column instead; the line
    // holds no digit at all, and bases 130 and 140 are the ticks at 16 and 26.
    let no_room: Vec<(LineRole, Line<'static>)> = lines_at(&record, &laid, &mode, 4, false);
    let found: Vec<(u64, String)> = ruler_of(no_room.as_slice());
    assert_eq!(
        found,
        vec![
            (6, TICK.to_string()),
            (16, TICK.to_string()),
            (26, TICK.to_string())
        ],
        "the 119-148 line holds three ticks and no digit"
    );
    assert!(
        found.iter().all(|(_column, text)| text != "120"),
        "nothing on the line is the number `120`: {found:?}"
    );
}

/// Rule 3: every cell the ruler line holds is dim, with no colour on it,
/// whether the view's colours are on and off.
#[test]
fn the_ruler_is_dim_and_uncoloured() {
    let record: ViewRecord = pbr();
    let mode: LaneMode = LaneMode::Off;
    let laid: Layout = layout(&record, INNER, &mode).expect("a layout at width 78");
    for colours in [true, false] {
        let lines: Vec<(LineRole, Line<'static>)> = lines_at(&record, &laid, &mode, 0, colours);
        let held: Vec<(u64, String, Style)> = placed(&line_of(&lines, &LineRole::Ruler));
        assert!(
            !held.is_empty(),
            "the ruler line of block 0 holds ticks and numbers with the colours \
             on and off alike"
        );
        for (column, text, style) in held {
            assert!(
                style.add_modifier.contains(Modifier::DIM),
                "the ruler's {text:?} at column {column} is dim \
                 with the colours {colours}"
            );
            assert_eq!(
                style.fg, None,
                "the ruler's {text:?} at column {column} keeps the terminal's own \
                 colour with the colours {colours}"
            );
            assert_eq!(
                style.bg, None,
                "the ruler's {text:?} at column {column} has no background with \
                 the colours {colours}"
            );
        }
    }
}

/// Rule 1 and the screen: the row above the top strand's row is the ruler, and
/// its columns are the layout's, moved by the frame's border.
#[test]
fn the_screen_draws_the_ruler() {
    let state: ViewState = ViewState::new(pbr(), W, H);
    let rows: Vec<Vec<Cell>> = {
        let mut terminal = Terminal::new(TestBackend::new(W, H)).unwrap();
        terminal.draw(|frame| draw(frame, &state)).unwrap();
        let buffer = terminal.backend().buffer();
        let width: usize = usize::from(buffer.area.width);
        buffer
            .content()
            .chunks(width)
            .map(|row: &[Cell]| row.to_vec())
            .collect()
    };

    // The top strand's row, found by the bases it shows: block 0's 66 bases
    // start at the frame's first inner column plus the gutter's 4 columns and
    // the one blank column beside them.
    let bases: String = "TTCTCATGT".to_string();
    let top: usize = (0..rows.len())
        .find(|row| symbols(&rows[*row], 6, 14) == bases)
        .expect("the row block 0's top strand stands on");
    let ruler: usize = top - 1;
    assert_eq!(
        symbols(&rows[ruler], 34, 35),
        "30",
        "`30` for base 30 stands at layout columns 33-34, at the frame's 34-35"
    );
    assert_eq!(
        symbols(&rows[ruler], 15, 15),
        TICK,
        "the tick for base 10 stands at layout column 14, at the frame's 15"
    );
    // The ruler row is the ruler's, and nothing else stands on it: the tick at
    // 15, 25, 45 and 55 and the numbers at 34-35 and 64-65 are the layout's
    // 14, 24, 44, 54, 33-34, 63-64 columns, and the row holds no other symbol.
    assert_eq!(symbols(&rows[ruler], 25, 25), TICK, "the tick for base 20");
    assert_eq!(symbols(&rows[ruler], 45, 45), TICK, "the tick for base 40");
    assert_eq!(symbols(&rows[ruler], 55, 55), TICK, "the tick for base 50");
    assert_eq!(
        symbols(&rows[ruler], 64, 65),
        "60",
        "`60` for base 60 stands at layout columns 63-64"
    );
    let marked: Vec<usize> = (1..79)
        .filter(|column| rows[ruler][*column].symbol() != " ")
        .collect();
    assert_eq!(
        marked,
        vec![15, 25, 34, 35, 45, 55, 64, 65],
        "the ruler row holds the ruler's cells, at layout columns 14, 24, 33-34, \
         44, 54, 63-64 moved one column for the border"
    );
}

/// One stretch of a drawn row, `first` to `last` at both ends, as the symbols of
/// its cells.
fn symbols(row: &[Cell], first: usize, last: usize) -> String {
    row[first..=last].iter().map(|cell| cell.symbol()).collect()
}
