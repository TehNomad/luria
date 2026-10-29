//! The feature lanes and the strip's bars are drawn in their feature's colour
//! (contract 8m7 §5, M8-31): the colour the file gives them on the strand they
//! were written on, or their key's colour when the file gives them none, and the
//! selected CDS's lane is bold whether the colours are on and off.
//!
//! `pbr` is `tests/fixtures/genbank/raw/J01749.1.gb` — 4361 bp, circular by its own
//! declaration, and `tet` (`86..1276`, its `gene` and its `CDS` on the plus strand,
//! the gene first in the file) is CDS 0, so `Cds(0)` selects it. 8m4's table of
//! `tests/fixtures/view/README.md` gives the lanes of block 1 at width 78: lane 1 is
//! the `tet` gene, lane 2 the `tet` CDS, and lanes 3 and 4 the two `misc_binding`
//! sites, so the colours differ only through the key. `ape` is the 30 bp linear
//! record §5 writes, whose three `misc_feature`s name `a`, `b` and `c`: `a` and `b`
//! carry `ApEinfo_fwdcolor="#ff0000"` and `ApEinfo_revcolor="#00ff00"`, and `c`
//! carries `ApEinfo_fwdcolor="pink"`, a colour name that says nothing.
//!
//! The colour expectations come from contract §4 rule 2's table and rule 3's cube
//! arithmetic — `#ff0000` is [255,0,0], which is the cube's own red, index
//! 16 + 36·5 = 196, and `#00ff00` is its green, 16 + 30 = 46 — and not from what
//! the code answers. The screen columns of the strip are 8l2 test 5's, and the
//! screen is read back off a `ratatui` test backend as `tests/sequence_view_overview.rs`
//! reads it: row 1 is the strip's bar line, between the borders at x 1 to 78.

use luria_cli::sequence_view::layout::block_lines;
use luria_cli::sequence_view::layout::layout;
use luria_cli::sequence_view::layout::Drawing;
use luria_cli::sequence_view::layout::FeatureLevel;
use luria_cli::sequence_view::layout::LaneMode;
use luria_cli::sequence_view::layout::Layout;
use luria_cli::sequence_view::layout::LineRole;
use luria_cli::sequence_view::screen::draw;
use luria_cli::sequence_view::state::Action;
use luria_cli::sequence_view::state::Key;
use luria_cli::sequence_view::state::ViewState;
use luria_core::identity::Identifier;
use luria_core::view_record::open_record;
use luria_core::view_record::ViewRecord;
use ratatui::backend::TestBackend;
use ratatui::buffer::Cell;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::Terminal;
use std::fs;
use std::path::Path;

/// Every test draws at this size: 80 columns and 24 rows, whose frame has 78 inner
/// columns, as every other sequence-view test does.
const W: u16 = 80;
const H: u16 = 24;

/// The inner width `layout` is measured at: 80 columns less the frame's two border
/// columns (8k1 §4 rule 1).
const INNER: u16 = 78;

/// The screen row the strip's bar line is drawn on (8l2 §4 rule 2).
const BAR_ROW: usize = 1;

/// The first and last inner column of the frame, as the buffer's own `x`: the
/// borders are at x 0 and x 79 (8l2 §5).
const INSIDE: (usize, usize) = (1, 78);

fn pbr() -> ViewRecord {
    let path: &str = "../../tests/fixtures/genbank/raw/J01749.1.gb";
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let text: String =
        fs::read_to_string(&file).unwrap_or_else(|error| panic!("read {path}: {error}"));
    open_record(Identifier::create("SYNPBR322".to_string()).unwrap(), text)
        .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
}

/// §5's `ape`: 30 bp, linear, three `misc_feature`s named by their `/gene`, the
/// first two with ApE colours and the third with a colour name. Its 30 bases give
/// `d = 2`, so one block holds the record whole.
fn ape() -> ViewRecord {
    let text: &str = r##"LOCUS       ape                       30 bp    DNA     linear   SYN 27-OCT-2026
FEATURES             Location/Qualifiers
     misc_feature    1..10
                     /gene="a"
                     /ApEinfo_fwdcolor="#ff0000"
                     /ApEinfo_revcolor="#00ff00"
     misc_feature complement(11..20)
                     /gene="b"
                     /ApEinfo_fwdcolor="#ff0000"
                     /ApEinfo_revcolor="#00ff00"
     misc_feature    21..30
                     /gene="c"
                     /ApEinfo_fwdcolor="pink"
ORIGIN
        1 atggcttggt aacccccccc cccccccccc
//
"##;
    open_record(
        Identifier::create("ape".to_string()).unwrap(),
        text.to_string(),
    )
    .unwrap_or_else(|error| panic!("open the 30 bp record: {error:?}"))
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

/// The one line of `lines` whose role is `role`.
fn line_of(lines: &[(LineRole, Line<'static>)], role: &LineRole) -> Line<'static> {
    lines
        .iter()
        .find(|(line_role, _line)| line_role == role)
        .unwrap_or_else(|| panic!("no line with role {role:?} in {:?}", roles(lines)))
        .1
        .clone()
}

/// The roles of a block's lines, in the order they are drawn — for a failure that
/// names the line a test looked for and did not find.
fn roles(lines: &[(LineRole, Line<'static>)]) -> Vec<LineRole> {
    lines.iter().map(|(role, _line)| role.clone()).collect()
}

/// The cells one line draws, as `(text, style)`: every span of the line that is not
/// a gap. `layout`'s own drawing writes one span per cell and one per gap, and a gap
/// holds nothing but spaces, so the cells are the rest (8m7 §4 rule 4).
fn cells(line: &Line<'static>) -> Vec<(String, Style)> {
    line.spans
        .iter()
        .filter(|span| !span.content.chars().all(|ch| ch == ' '))
        .map(|span| (span.content.to_string(), span.style))
        .collect()
}

/// Every cell of lane `number`, spelled out in the message of the check that reads
/// them.
fn lane_cells(lines: &[(LineRole, Line<'static>)], number: u8) -> Vec<(String, Style)> {
    let line: Line<'static> = line_of(lines, &LineRole::Lane(number));
    let found: Vec<(String, Style)> = cells(&line);
    assert!(
        !found.is_empty(),
        "lane {number} has cells to colour: {:?}",
        line.spans
            .iter()
            .map(|span| span.content.to_string())
            .collect::<Vec<String>>()
    );
    found
}

/// The screen's cells, row by row, as the test backend holds them.
fn drawn(state: &ViewState) -> Vec<Vec<Cell>> {
    let mut terminal = Terminal::new(TestBackend::new(W, H)).unwrap();
    terminal.draw(|frame| draw(frame, state)).unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = usize::from(buffer.area.width);
    buffer
        .content()
        .chunks(width)
        .map(|row: &[Cell]| row.to_vec())
        .collect()
}

/// Rule 1: every lane cell is its key's colour, and the selected CDS's own lane is
/// bold with it. 8m4's table gives the lanes of block 1 — the `tet` gene, the `tet`
/// CDS, and the two `misc_binding` sites — and `gene`, `CDS` and `misc_binding`
/// colour 179, 214 and 168 (8m7 §4 rule 2).
#[test]
fn lanes_take_their_key_colours() {
    let record: ViewRecord = pbr();
    let mode: LaneMode = LaneMode::Cds(0);
    let laid: Layout = layout(&record, INNER, &mode).expect("a layout at width 78");
    assert_eq!(
        laid.blocks[1],
        (66, 130),
        "block 1 is 8m4's, bases 67 to 130, shortened at the codon edge"
    );
    let lines: Vec<(LineRole, Line<'static>)> = lines_at(&record, &laid, &mode, 1, true);
    let roles: Vec<LineRole> = roles(&lines);
    for number in 1..=4u8 {
        assert!(
            roles.contains(&LineRole::Lane(number)),
            "lane {number} is drawn: {roles:?}"
        );
    }

    for (number, colour) in [(1u8, 179u8), (2, 214), (3, 168), (4, 168)] {
        let bold: bool = number == 2;
        for (text, style) in lane_cells(&lines, number) {
            assert_eq!(
                style.fg,
                Some(Color::Indexed(colour)),
                "lane {number}'s cell {text:?} draws the key's colour {colour}"
            );
            assert_eq!(
                style.add_modifier.contains(Modifier::BOLD),
                bold,
                "lane {number}'s cell {text:?} {}",
                if bold {
                    "is bold as the selected CDS"
                } else {
                    "is not bold"
                }
            );
        }
    }

    // Rule 4's last sentence: the `+N more` line is never coloured. pBR322's block 1
    // has exactly four lane-eligible features and no `more` line (8m4's table), so
    // the check is on block 0 — bases 1 to 66 — where the `source` features,
    // `misc_binding` sites and `regulatory` bits of the first 66 bases outnumber
    // the four lanes the level shows.
    let crowd: Vec<(LineRole, Line<'static>)> = lines_at(&record, &laid, &mode, 0, true);
    let more: Line<'static> = line_of(&crowd, &LineRole::More);
    let spelled: String = cells(&more)
        .iter()
        .map(|(text, _)| text.clone())
        .collect::<Vec<String>>()
        .join("");
    assert!(
        spelled.starts_with('+') && spelled.ends_with(" more"),
        "the line is the `+N more` line 8m4 draws: {spelled:?}"
    );
    for (text, style) in cells(&more) {
        assert_eq!(
            style.fg, None,
            "the {text:?} line keeps the terminal's colour"
        );
        assert!(
            !style.add_modifier.contains(Modifier::BOLD),
            "the {text:?} line is not bold either"
        );
    }
}

/// Rule 4: the bold follows the selection, not the colours. With colours off no
/// lane cell has a colour, and the selected CDS's lane keeps its bold; with no CDS
/// selected nothing is bold even with the colours on.
#[test]
fn bold_follows_the_selection_not_the_colours() {
    let record: ViewRecord = pbr();
    let mode: LaneMode = LaneMode::Cds(0);
    let laid: Layout = layout(&record, INNER, &mode).expect("a layout at width 78");
    let lines: Vec<(LineRole, Line<'static>)> = lines_at(&record, &laid, &mode, 1, false);
    for number in 1..=4u8 {
        let bold: bool = number == 2;
        for (text, style) in lane_cells(&lines, number) {
            assert_eq!(
                style.fg, None,
                "lane {number}'s cell {text:?} is uncoloured with the colours off"
            );
            assert_eq!(
                style.add_modifier.contains(Modifier::BOLD),
                bold,
                "lane {number}'s cell {text:?} stays {}",
                if bold { "bold" } else { "plain" }
            );
        }
    }

    // The same block with no CDS selected: the lanes are still drawn, and none of
    // them is bold, because the view selects no feature.
    let off: LaneMode = LaneMode::Off;
    let flat: Layout = layout(&record, INNER, &off).expect("a layout at width 78");
    assert_eq!(
        flat.blocks[1],
        (66, 132),
        "without a selected CDS block 1 holds bases 67 to 132"
    );
    let lines: Vec<(LineRole, Line<'static>)> = lines_at(&record, &flat, &off, 1, true);
    for number in 1..=4u8 {
        for (text, style) in lane_cells(&lines, number) {
            assert!(
                !style.add_modifier.contains(Modifier::BOLD),
                "lane {number}'s cell {text:?} is not bold with no CDS selected"
            );
        }
    }
}

/// Rule 3: the file's colour wins over the key's, and it is read from the strand
/// the feature was written on. `ape`'s three lanes: `a` on the plus strand takes
/// `forward`, `b` on the minus strand takes `reverse`, and `c`, whose value is a
/// colour name and no value at all, falls to its key — `misc_feature`, which is
/// outside the table and keeps the terminal's own colour.
#[test]
fn the_file_colour_wins() {
    let record: ViewRecord = ape();
    let mode: LaneMode = LaneMode::Off;
    let laid: Layout = layout(&record, INNER, &mode).expect("a layout at width 78");
    assert_eq!(laid.blocks.len(), 1, "one block holds the 30 bp");
    let lines: Vec<(LineRole, Line<'static>)> = lines_at(&record, &laid, &mode, 0, true);
    for (number, colour) in [(1u8, 196u8), (2, 46)] {
        for (text, style) in lane_cells(&lines, number) {
            assert_eq!(
                style.fg,
                Some(Color::Indexed(colour)),
                "lane {number}'s cell {text:?} draws the file's colour {colour}"
            );
        }
    }
    for (text, style) in lane_cells(&lines, 3) {
        assert_eq!(
            style.fg, None,
            "lane 3's cell {text:?} keeps the terminal's colour, because its value \
             is a name and its key is outside the table"
        );
    }
}

/// Rule 5: the strip's bars carry the same colours the lanes do. 8l2 test 5 gives
/// the columns: the `tet` bar at x 4 to 23 and `mobile_element` at 54 to 73. The
/// `tet` bar is the gene's, because the gene comes first in the file and the `tet`
/// CDS is skipped as overlapping (8l2 §4 rule 4) — 179, not the CDS's 214. The `─`
/// cells keep nothing, and after `C` nothing on the line has a colour.
#[test]
fn the_strip_matches_the_lanes() {
    let state: ViewState = ViewState::new(pbr(), W, H);
    assert!(state.overview(), "pbr opens with the strip drawn");
    let rows: Vec<Vec<Cell>> = drawn(&state);
    let line: Vec<Cell> = rows[BAR_ROW].clone();
    assert_eq!(
        symbols(&line, 4, 23),
        format!("tet ▸{}", "═".repeat(15)),
        "the `tet` gene's bar is 8l2's"
    );
    assert_eq!(
        symbols(&line, 54, 73),
        format!("mobile_element ▸{}", "═".repeat(4)),
        "the longest bar is 8l2's"
    );
    for column in 4..=23 {
        assert_eq!(
            fg(&line, column),
            Color::Indexed(179),
            "the `tet` bar at x {column} draws the gene's colour"
        );
    }
    for column in 54..=73 {
        assert_eq!(
            fg(&line, column),
            Color::Indexed(109),
            "the `mobile_element` bar at x {column} draws its key's colour"
        );
    }
    let gaps: Vec<usize> = (INSIDE.0..=INSIDE.1)
        .filter(|column| line[*column].symbol() == "─")
        .collect();
    assert!(!gaps.is_empty(), "the line holds some `─` cell");
    for column in gaps {
        assert_eq!(
            fg(&line, column),
            Color::Reset,
            "the `─` cell at x {column} keeps the terminal's colour"
        );
    }
    // Rule 5's other half: the two end numbers keep no style. With pBR322 at this
    // width the length sits at x 75 to 78, where 8l2's test 4 puts it.
    assert_eq!(
        symbols(&line, 75, 78),
        "4361",
        "the length is where 8l2's table puts it"
    );
    for column in 75..=78 {
        assert_eq!(
            fg(&line, column),
            Color::Reset,
            "the length `4361` at x {column} keeps the terminal's colour"
        );
    }
    // `C` turns the view's colours off, and then nothing on the line has one.
    let mut with_c: ViewState = ViewState::new(pbr(), W, H);
    assert_eq!(with_c.handle(Key::Char('C')), Action::Continue);
    let line: Vec<Cell> = drawn(&with_c)[BAR_ROW].clone();
    for column in INSIDE.0..=INSIDE.1 {
        assert_eq!(
            fg(&line, column),
            Color::Reset,
            "with the colours off, x {column} keeps the terminal's own"
        );
    }
}

/// One stretch of a row, `first` to `last` at both ends, as the symbols of its
/// cells (8l2 §5's reading of the strip).
fn symbols(row: &[Cell], first: usize, last: usize) -> String {
    row[first..=last].iter().map(|cell| cell.symbol()).collect()
}

/// The colour one cell of a drawn row carries, as the test backend holds it: the
/// same reading `tests/sequence_view_colours.rs` uses for a cell's background.
fn fg(row: &[Cell], column: usize) -> Color {
    row[column].fg
}
