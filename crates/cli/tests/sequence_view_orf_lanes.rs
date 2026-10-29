//! The ORF lanes `O` shows and hides (contract 8m18, M8-40). The ORFs come from
//! 8m17's `find_orfs` and are drawn as a feature lane is: a bar with a head at the
//! 3' end (8m11), labelled `ORF {residues} aa`, in its own colour, after the
//! feature lanes and before every other line.
//!
//! ## How each value was found
//!
//! `pbr` is `tests/fixtures/genbank/raw/J01749.1.gb`, 4361 bp circular by its own
//! `ORIGHT:4361 OLEFT:4361`, so `d = 4` and `g = 5`, and at width 78 a line holds
//! `⌊(78 − 2·4 − 2) / 3⌋ · 3 = 66` bases: block `k` covers the zero-based
//! `[66k, 66k + 66)`, whose bases stand at line columns 5 to 70. The nine readings,
//! their strands and residues are `tests/fixtures/view/orfs.tsv`, the table 8m17 §7
//! verified, which contract 8m18 §5 gives in zero-based form: `+ [85, 1276)` 396 aa
//! (`tet`), `+ [258, 642)` 127, `- [322, 565)` 80, `- [438, 789)` 116,
//! `- [622, 1081)` 152, `+ [1179, 1518)` 112, `- [1514, 1883)` 122,
//! `- [3292, 4153)` 286 (`bla`), `+ [3422, 3689)` 88.
//!
//! A line covers the columns the reading's bases have in the block, one run per
//! stretch of consecutive bases, cut to the block as a feature lane is (D053). The
//! label is `ORF {residues} aa ` — 11 columns for the three-digit residues these
//! blocks use — and the bar fills the run's remaining columns with `═`. A plus run
//! that holds the reading's 3' base replaces its last column with `▸`, and a minus
//! run that holds it starts with `◂ ` before the label, both cut to what the run
//! holds (8m11 rule 2, contract 8m18 §4 rule 4). A plus reading's 3' base is the
//! last base of its last interval, a minus reading's the first base of its first
//! one (8m11 rule 1). The line order is longest whole span first, then the first
//! interval's first base, then plus before minus.

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
use luria_core::orf::find_orfs;
use luria_core::orf::Orf;
use luria_core::orf::ORF_MIN_RESIDUES;
use luria_core::sequence::SequenceTopology;
use luria_core::strand::Strand;
use luria_core::view_record::open_record;
use luria_core::view_record::ViewRecord;
use ratatui::backend::TestBackend;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Terminal;
use std::fs;
use std::path::Path;

/// The size every sequence-view test draws at: 80 columns and 24 rows, whose frame
/// has 78 inner columns and whose status line is row 21.
const W: u16 = 80;
const H: u16 = 24;

/// The width `layout` is given for one block's lines: the frame's inner columns.
const INNER: u16 = 78;

/// The status line's screen row: the frame's last inner row (8k1 §4 rule 1).
const STATUS: usize = 21;

/// The body rows, where a block's lines and the strip stand: the frame's inner rows
/// less the status line's one.
const BODY: std::ops::Range<usize> = 1..21;

/// pBR322: 4361 bp, circular by its own declaration, three CDSs of which two are
/// usable — `tet` on the plus strand and `bla` on the minus — with no CDS selected
/// on open, which is why a block holds the lanes, the ruler and the two strands.
fn pbr() -> ViewRecord {
    let path: &str = "../../tests/fixtures/genbank/raw/J01749.1.gb";
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let text: String =
        fs::read_to_string(&file).unwrap_or_else(|error| panic!("read {path}: {error}"));
    open_record(Identifier::create("SYNPBR322".to_string()).unwrap(), text)
        .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
}

/// The record contract 8m18 §5 test 6 writes: `TAAATG` then 80 `GCT`, 246 bp, its
/// topology assumed by the FASTA file.
fn orfwrap() -> ViewRecord {
    let text: String = format!(">orfwrap\nTAAATG{}\n", "GCT".repeat(80));
    open_record(Identifier::create("orfwrap".to_string()).unwrap(), text)
        .unwrap_or_else(|error| panic!("open the 246 bp FASTA: {error:?}"))
}

/// pBR322's nine readings at 8m18's minimum, scanned from the circular topology the
/// file states and that `T` never switches off for this record.
fn pbr_orfs(record: &ViewRecord) -> Vec<Orf> {
    assert_eq!(
        record.sequence.topology(),
        SequenceTopology::Circular,
        "the file states a circular record"
    );
    find_orfs(&record.sequence, true, ORF_MIN_RESIDUES)
}

/// One block's lines, drawn in the mode `mode` at the level `features` with the
/// colours `colours`, and for the open reading frames `orfs` (contract 8m18 §4
/// rule 1).
fn lines_at(
    record: &ViewRecord,
    laid: &Layout,
    mode: &LaneMode,
    block: usize,
    features: FeatureLevel,
    colours: bool,
    orfs: &[Orf],
) -> Vec<(LineRole, Line<'static>)> {
    block_lines(
        record,
        laid,
        mode,
        block,
        &Drawing {
            colours,
            features,
            orfs,
        },
    )
    .unwrap_or_else(|problem| panic!("block {block} in {mode:?}: {problem:?}"))
}

/// The roles of `lines`, in the order the lines draw.
fn roles(lines: &[(LineRole, Line<'static>)]) -> Vec<LineRole> {
    lines.iter().map(|(role, _line)| role.clone()).collect()
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

/// The cells of one line, as `(text, style)`: every span that is not a gap
/// (`sequence_view_lane_colours.rs`'s reading of the lines `layout` builds).
fn cells(line: &Line<'static>) -> Vec<(String, Style)> {
    line.spans
        .iter()
        .filter(|span| !span.content.chars().all(|ch| ch == ' '))
        .map(|span| (span.content.to_string(), span.style))
        .collect()
}

/// The texts of one line, the gaps aside.
fn texts(line: &Line<'static>) -> Vec<String> {
    cells(line).iter().map(|(text, _)| text.clone()).collect()
}

/// Draw `lines` on a test backend `width` columns wide, and return the symbol of
/// every cell, row by row — the way `layout.rs` reads the test backend.
fn drawn(lines: &[(LineRole, Line<'static>)], width: u16) -> Vec<Vec<String>> {
    let height: u16 = lines.len() as u16;
    let text: Vec<Line<'static>> = lines.iter().map(|(_role, line)| line.clone()).collect();
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(move |frame| {
            frame.render_widget(Paragraph::new(text.clone()), frame.area());
        })
        .unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = buffer.area.width as usize;
    buffer
        .content()
        .chunks(width)
        .map(|row| row.iter().map(|cell| cell.symbol().to_string()).collect())
        .collect()
}

/// The text of one drawn row between two columns, inclusive.
fn span(row: &[String], first: usize, last: usize) -> String {
    row[first..=last].concat()
}

/// The screen as the test backend holds it: every row as the symbols of its cells.
fn rows(state: &ViewState) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(W, H)).unwrap();
    terminal.draw(|frame| draw(frame, state)).unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = usize::from(buffer.area.width);
    buffer
        .content
        .chunks(width)
        .map(|cells| cells.iter().map(|cell| cell.symbol()).collect())
        .collect()
}

/// The cells between the frame's borders on one screen row, trimmed at both ends.
fn body_row(state: &ViewState, row: usize) -> String {
    let line: Vec<char> = rows(state)[row].chars().collect();
    let right: usize = usize::from(W) - 1;
    line[1..right].iter().collect::<String>().trim().to_string()
}

/// Every non-blank body row, in screen order.
fn body_rows(state: &ViewState) -> Vec<String> {
    BODY.map(|row| body_row(state, row))
        .filter(|row| !row.is_empty())
        .collect()
}

/// The status line as the screen draws it.
fn status(state: &ViewState) -> String {
    rows(state)[STATUS].trim_end().to_string()
}

/// `key` pressed once, with a failure saying it leaves.
fn press(state: &mut ViewState, key: Key) {
    assert_eq!(
        state.handle(key),
        Action::Continue,
        "{key:?} does not leave"
    );
}

/// The block holding the cursor, as the layout the state carries answers it.
fn cursor_block(state: &ViewState) -> usize {
    let laid: &Layout = state.layout().expect("a drawn layout");
    laid.blocks
        .iter()
        .position(|(_first, last)| *last > state.cursor())
        .expect("the block holding the cursor")
}

/// §5 test 1: blocks 0, 1 and 19 as contract 8m18 §5 gives them. A block no reading
/// reaches has no ORF line; blocks 1 and 19 draw their runs at the columns the
/// coordinates give, and the head stands only where the 3' base stands (§4 rules 1
/// to 4).
#[test]
fn orf_lanes_hold_their_orfs() {
    let record: ViewRecord = pbr();
    let ors: Vec<Orf> = pbr_orfs(&record);
    assert_eq!(ors.len(), 9, "8m17's table keeps nine readings");
    let laid: Layout = layout(&record, INNER, &LaneMode::Off).expect("66 bases at width 78");
    assert_eq!(
        laid.gutter, 4,
        "the record's 4361 bases make a four-column gutter"
    );
    assert_eq!(laid.blocks[0], (0, 66));
    assert_eq!(laid.blocks[1], (66, 132));
    assert_eq!(laid.blocks[19], (1254, 1320));

    // Block 0: no reading reaches base 66, so no ORF line, and nothing to count.
    let zero: Vec<(LineRole, Line<'static>)> = lines_at(
        &record,
        &laid,
        &LaneMode::Off,
        0,
        FeatureLevel::None,
        false,
        &ors,
    );
    assert_eq!(
        roles(&zero),
        vec![LineRole::Ruler, LineRole::Top, LineRole::Bottom],
        "before `tet`'s first reading a block has no ORF line and no `+N more ORFs` line"
    );

    // Block 1: `tet`'s reading, which starts at 85 and runs past the block, covers
    // its 47 bases at the columns 24 to 70 — `5 + (85 − 66)` — and draws no head,
    // because its 3' base is 1275 and the block's last base is 131. The label is 11
    // columns, so the bar holds the other 36.
    let one: Vec<(LineRole, Line<'static>)> = lines_at(
        &record,
        &laid,
        &LaneMode::Off,
        1,
        FeatureLevel::None,
        false,
        &ors,
    );
    assert_eq!(
        roles(&one),
        vec![
            LineRole::Orf(1),
            LineRole::Ruler,
            LineRole::Top,
            LineRole::Bottom
        ],
        "one reading overlaps the block, and it is the block's only ORF line"
    );
    let rows: Vec<Vec<String>> = drawn(&one, 78);
    assert_eq!(span(&rows[0], 23, 23), " ", "column 23 is blank");
    assert_eq!(
        span(&rows[0], 24, 70),
        "ORF 396 aa ".to_string() + &"═".repeat(36),
        "the label opens the bar, which fills the rest of the run"
    );
    assert_eq!(
        span(&rows[0], 71, 71),
        " ",
        "the run ends at the block's last base"
    );

    // Block 19: `tet`'s run covers the bases 1254 to 1276, 22 columns from column 5,
    // and its 3' base 1275 stands in that run's last column, 26: ten `═` and the
    // head. `+ [1179, 1518)` covers the whole block and ends 184 bases past its own
    // 3' base at 1517.
    let nineteen: Vec<(LineRole, Line<'static>)> = lines_at(
        &record,
        &laid,
        &LaneMode::Off,
        19,
        FeatureLevel::None,
        false,
        &ors,
    );
    assert_eq!(
        roles(&nineteen),
        vec![
            LineRole::Orf(1),
            LineRole::Orf(2),
            LineRole::Ruler,
            LineRole::Top,
            LineRole::Bottom,
        ],
        "the 1191-base reading outranks the 339-base one, and no third reading reaches the block",
    );
    let rows: Vec<Vec<String>> = drawn(&nineteen, 78);
    assert_eq!(
        span(&rows[0], 4, 4),
        " ",
        "the run begins at the block's first base"
    );
    assert_eq!(
        span(&rows[0], 5, 26),
        "ORF 396 aa ".to_string() + &"═".repeat(10) + "▸",
        "the run that holds the 3' end puts its head in its last column",
    );
    assert_eq!(
        span(&rows[0], 27, 27),
        " ",
        "the run ends past base 1276, and the head stands at base 1275's column"
    );
    assert_eq!(
        span(&rows[1], 5, 70),
        "ORF 112 aa ".to_string() + &"═".repeat(55),
        "the second reading covers the whole block and draws no head",
    );
    assert!(
        !span(&rows[1], 0, 77).contains(['▸', '◂']),
        "no head stands in a run that does not hold the 3' end"
    );
    assert_eq!(
        span(&rows[1], 4, 4),
        " ",
        "that run too begins at the block's first base"
    );
}

/// §5 test 2: the four readings that reach block 9 come in that block's order by
/// whole span, longest first, right after any feature lines and before every other
/// line, each run cut to the block (§4 rules 2 and 3).
#[test]
fn orf_lanes_go_longest_first() {
    let record: ViewRecord = pbr();
    let ors: Vec<Orf> = pbr_orfs(&record);
    let laid: Layout = layout(&record, INNER, &LaneMode::Off).expect("66 bases at width 78");
    assert_eq!(laid.blocks[9], (594, 660));

    // The whole spans, read off the table: `tet`'s reading 1191 bases, `1081 − 622`
    // = 459, `642 − 258` = 384, `789 − 438` = 351. No other reading reaches the
    // block: `- [322, 565)` ends at 565 and `+ [1179, 1518)` begins at 1179.
    let lines: Vec<(LineRole, Line<'static>)> = lines_at(
        &record,
        &laid,
        &LaneMode::Off,
        9,
        FeatureLevel::None,
        false,
        &ors,
    );
    assert_eq!(
        roles(&lines),
        vec![
            LineRole::Orf(1),
            LineRole::Orf(2),
            LineRole::Orf(3),
            LineRole::Orf(4),
            LineRole::Ruler,
            LineRole::Top,
            LineRole::Bottom,
        ],
        "four readings fill four ORF lines in that order, and nothing is left to count"
    );

    // With every drawn feature getting a lane, the ORF lines still stand after the
    // lanes and their `+N more` line, before the ruler (8m18 §4 rule 3).
    let crowded: Vec<(LineRole, Line<'static>)> = lines_at(
        &record,
        &laid,
        &LaneMode::Off,
        9,
        FeatureLevel::All,
        false,
        &ors,
    );
    let crowded_roles: Vec<LineRole> = roles(&crowded);
    let first: usize = crowded_roles
        .iter()
        .position(|role| matches!(role, LineRole::Orf(_)))
        .expect("a block of four readings draws an ORF line");
    assert!(
        crowded_roles[..first]
            .iter()
            .all(|role| matches!(role, LineRole::Lane(_) | LineRole::More)),
        "every line above the ORF lines is a feature line: {crowded_roles:?}"
    );
    assert_eq!(
        &crowded_roles[first..],
        &[
            LineRole::Orf(1),
            LineRole::Orf(2),
            LineRole::Orf(3),
            LineRole::Orf(4),
            LineRole::Ruler,
            LineRole::Top,
            LineRole::Bottom,
        ],
        "and every other line stands below them"
    );

    // The four runs, each cut to the block: `tet` covers its 66 bases with no head;
    // `1081 − 622` begins at the block's base 622, where a minus reading's 3' end
    // stands, so `◂ ` stands before its label and the bar holds the run's other 25
    // columns; `642 − 258` ends inside the block at base 641, so `▸` stands in the
    // run's last column; `789 − 438` covers the block with its 3' end at 438, past
    // the block, and draws no head.
    for (number, expected) in [
        (1usize, "ORF 396 aa ".to_string() + &"═".repeat(55)),
        (2, "◂ ORF 152 aa ".to_string() + &"═".repeat(25)),
        (3, "ORF 127 aa ".to_string() + &"═".repeat(36) + "▸"),
        (4, "ORF 116 aa ".to_string() + &"═".repeat(55)),
    ] {
        let line: Line<'static> = line_of(&lines, &LineRole::Orf(number));
        assert_eq!(
            texts(&line),
            vec![expected],
            "orf{number} stands as one run of its label and its bar"
        );
    }
}

/// §5 test 3: six readings that all cover one block leave four lines and count the
/// two that do not fit on one more line, as the feature `+N more` line does (§4
/// rule 2).
#[test]
fn more_orfs_than_lanes() {
    let record: ViewRecord = pbr();
    let laid: Layout = layout(&record, INNER, &LaneMode::Off).expect("66 bases at width 78");
    assert_eq!(laid.blocks[9], (594, 660));

    // Hand-built: six plus-strand readings that each begin at the block's first base
    // and run past it, 600, 500, 400, 300, 200 and 100 bases long. Their residues
    // are the codons each span holds, the stop not counted as 8m17 counts it.
    let hand: Vec<Orf> = [600u64, 500, 400, 300, 200, 100]
        .iter()
        .map(|bases| Orf {
            strand: Strand::Plus,
            intervals: vec![(594, 594 + bases)],
            residues: bases / 3 - 1,
        })
        .collect();
    assert_eq!(hand.len(), 6, "six readings all reaching block 9");

    let lines: Vec<(LineRole, Line<'static>)> = lines_at(
        &record,
        &laid,
        &LaneMode::Off,
        9,
        FeatureLevel::None,
        false,
        &hand,
    );
    assert_eq!(
        roles(&lines),
        vec![
            LineRole::Orf(1),
            LineRole::Orf(2),
            LineRole::Orf(3),
            LineRole::Orf(4),
            LineRole::MoreOrfs,
            LineRole::Ruler,
            LineRole::Top,
            LineRole::Bottom,
        ],
        "four lines, then the one line counting what they leave behind"
    );

    // The four lines hold the four longest readings, in that order, every one of
    // them covering the block's 66 columns from column 5.
    let rows: Vec<Vec<String>> = drawn(&lines, 78);
    for (line, bases) in rows[..4].iter().zip([600u64, 500, 400, 300]) {
        // The label `ORF {residues} aa ` and the run's own 66 columns: the bar
        // holds the columns the label leaves.
        let label: String = format!("ORF {} aa ", bases / 3 - 1);
        assert_eq!(
            span(line, 5, 70),
            format!("{}{}", label, "═".repeat(66 - label.chars().count())),
            "orf line 1 to 4 hold the {bases}-base reading, its label from column 5"
        );
    }
    assert_eq!(
        span(&rows[4], 5, 16),
        "+2 more ORFs",
        "the four lines leave the 200-base and the 100-base readings to be counted",
    );
}

/// §5 test 4: every ORF cell is the view's own ORF colour with the colours on, and
/// uncoloured with them off; no cell is bold (§4 rule 5).
#[test]
fn orf_lanes_have_their_own_colour() {
    let record: ViewRecord = pbr();
    let ors: Vec<Orf> = pbr_orfs(&record);
    let laid: Layout = layout(&record, INNER, &LaneMode::Off).expect("66 bases at width 78");
    assert_eq!(laid.blocks[49], (3234, 3300));

    for colours in [true, false] {
        let lines: Vec<(LineRole, Line<'static>)> = lines_at(
            &record,
            &laid,
            &LaneMode::Off,
            49,
            FeatureLevel::None,
            colours,
            &ors,
        );
        assert_eq!(
            roles(&lines),
            vec![
                LineRole::Orf(1),
                LineRole::Ruler,
                LineRole::Top,
                LineRole::Bottom
            ],
            "block 49 holds one reading, `bla`'s",
        );
        let line: Line<'static> = line_of(&lines, &LineRole::Orf(1));
        // `bla` runs from 3292 to 4154: the block's last base, 3299, is its 68th
        // base, so the run is 8 columns wide, at columns 63 to 70, and the 3' end
        // stands at its first column. The label and the head are cut to the run's
        // eight columns (8m11 rule 1, contract 8m18 §4 rule 4).
        assert_eq!(
            texts(&line),
            vec!["◂ ORF 28".to_string()],
            "the label and the head hold the run's eight columns",
        );
        for (text, style) in cells(&line) {
            assert_eq!(
                style.fg,
                if colours {
                    Some(Color::Indexed(151))
                } else {
                    None
                },
                "the {text:?} cell {}, no bold",
                if colours {
                    "draws the ORF colour"
                } else {
                    "keeps the terminal's, which a drawn cell reads as Reset"
                },
            );
            assert!(
                !style.add_modifier.contains(Modifier::BOLD),
                "the {text:?} cell is never bold"
            );
        }
    }
}

/// §5 test 5: `O` shows the lines and hides them, says how many it keeps, follows
/// the cursor, is text inside a list, and has its row after `o`'s in the key list.
#[test]
fn o_shows_and_hides_the_orfs() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert!(
        !state.orf_lanes(),
        "the view opens with the lines hidden (§4 rule 6)"
    );
    assert!(
        state.orfs().is_none(),
        "and nothing is computed at open (§9)"
    );
    assert!(
        state.circular(),
        "and pBR322's file declares the circular reading the scan takes"
    );
    assert!(
        !body_rows(&state).iter().any(|row| row.contains("ORF")),
        "no block line names a reading at open: {:?}",
        body_rows(&state)
    );

    // Put the cursor in the first block a reading reaches: `g` then `100`, which
    // stands in block 1, the zero-based [66, 132).
    press(&mut state, Key::Char('g'));
    for ch in "100".chars() {
        press(&mut state, Key::Char(ch));
    }
    press(&mut state, Key::Enter);
    let block: usize = cursor_block(&state);
    assert_eq!(block, 1, "base 100 stands in pBR322's block 1");
    assert!(
        state.visible().contains(&block),
        "and that block is one of the drawn ones"
    );

    press(&mut state, Key::Char('O'));
    assert!(state.orf_lanes(), "the first `O` shows the lines");
    assert_eq!(
        state
            .orfs()
            .expect("the first `O` scanned the record")
            .len(),
        9,
        "8m17's table keeps nine for the topology the file states"
    );
    assert!(
        status(&state).contains("ORFs shown: 9"),
        "the status line names what is shown: {}",
        status(&state)
    );

    // The block holding the cursor draws its one ORF line at the column the
    // coordinates give it: `tet`'s run begins at base 85, 19 bases into the block,
    // so line column 24 stands at screen column 25 beside the ruler line.
    let drawn_rows: Vec<String> = rows(&state);
    let orf_row: usize = BODY
        .clone()
        .find(|row| drawn_rows[*row].contains("ORF 396 aa"))
        .expect("the drawn block holding the cursor shows the reading's line");
    let line: Vec<char> = drawn_rows[orf_row].chars().collect();
    assert_eq!(
        line[24], ' ',
        "column 24 of the frame's inner area is blank"
    );
    assert_eq!(
        line[25..35].iter().collect::<String>(),
        "ORF 396 aa",
        "the label stands at the block's own column 24",
    );

    press(&mut state, Key::Char('O'));
    assert!(!state.orf_lanes(), "the second `O` hides them again");
    assert!(
        status(&state).contains("ORFs hidden"),
        "and the status line says so: {}",
        status(&state)
    );
    assert!(
        state.orfs().is_some(),
        "the scan's result is kept for the next `O`, which needs no second scan"
    );
    assert!(
        !body_rows(&state).iter().any(|row| row.contains("ORF")),
        "no block line names a reading while they are hidden"
    );

    // Inside an open list every printable key is filter text, the flag aside (8m3 §4
    // rule 1), as `F` and `C` leave theirs.
    press(&mut state, Key::Char('f'));
    press(&mut state, Key::Char('O'));
    assert_eq!(
        state.list().expect("the feature list").filter(),
        "O",
        "the list's filter holds the `O` that was typed there"
    );
    assert!(
        !state.orf_lanes(),
        "and the `O` in the filter has moved no line"
    );
    press(&mut state, Key::Esc);
    assert!(state.list().is_none(), "esc closes the list");

    press(&mut state, Key::Char('?'));
    assert!(state.help(), "? opens the key list");
    let listed: Vec<String> = body_rows(&state);
    let position: usize = listed
        .iter()
        .position(|row| row.starts_with("o "))
        .expect("the key list has an `o` row");
    assert_eq!(
        listed[position + 1],
        "O  show the open reading frames and hide them",
        "the `O` row follows the `o` row and says what the key does: {listed:?}"
    );
}

/// §5 test 6: the linear scan finds nothing for a record whose `ATG` reading runs
/// off the end, and `T` recomputes at once for the circular reading, which finds
/// the one crossing the origin.
#[test]
fn t_recomputes_for_the_new_topology() {
    let record: ViewRecord = orfwrap();
    assert_eq!(record.sequence.length(), 246, "`TAAATG` then 80 `GCT`");
    assert_eq!(
        record.sequence.topology(),
        SequenceTopology::Linear,
        "a FASTA file's record carries an assumed topology, read linearly at open",
    );

    let mut state: ViewState = ViewState::new(record, W, H);
    assert!(!state.circular(), "the view opens on the linear reading");
    press(&mut state, Key::Char('O'));
    assert!(
        status(&state).contains("ORFs shown: 0"),
        "the `ATG` reading runs to the record's end and reaches no stop, and a linear
         reading that never reaches one is no open reading frame: {}",
        status(&state)
    );
    assert_eq!(state.orfs().expect("scanned once").len(), 0);

    press(&mut state, Key::Char('T'));
    assert!(
        state.circular(),
        "`T` switched the assumed topology to circular"
    );
    assert_eq!(cursor_block(&state), 0, "the cursor stayed where it was");
    assert!(
        status(&state).contains("ORFs shown: 1"),
        "the circular reading wraps into the `TAA` at the record's start: {}",
        status(&state)
    );
    let ors: &[Orf] = state.orfs().expect("scanned again");
    assert_eq!(
        ors.len(),
        1,
        "`T` recomputes for the topology it switched to"
    );
    assert_eq!(ors[0].strand, Strand::Plus);
    // The one reading that crosses the origin: its `ATG` stands at 1-based 4 and its
    // stop at 1-based 1 to 3, so the span runs from there to the record's end and
    // then on through base 3 — `join(4..246,1..3)` of contract 8m18 §6, written as
    // 8m17's fixture table writes its own crossing joins (`orf_fixtures.rs`'s
    // `join(55..60,1..12)`). In D036's half-open zero-based form, the order
    // `find_orfs` leaves them in, that is `(3, 246)` then `(0, 3)`, as the table's
    // `86..1276` is `(85, 1276)`.
    assert_eq!(
        ors[0].intervals,
        vec![(3, 246), (0, 3)],
        "contract 8m18 §5's `join(4..246, 1..3)`, in D036 order",
    );
    assert_eq!(ors[0].residues, 81, "the 82nd codon is the stop codon");
}
