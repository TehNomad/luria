//! The three feature levels `F` cycles through (contract 8m6 §5, M8-30): all
//! the features, the CDSs only, and none. The lanes a block draws come from the
//! level's own features, so the sort, `LANE_LIMIT` and the `+N more` line all
//! count what the level leaves, and under none there is no lane line and no
//! `+N more` line. The overview strip is untouched.
//!
//! The columns come from the records' coordinates, not from what the code
//! draws. `pbr` is `tests/fixtures/genbank/raw/J01749.1.gb`, 4361 bp, so `d = 4`
//! and at width 78 a line holds `⌊(78 − 2·4 − 2) / 3⌋ · 3 = 66` bases and its
//! first base stands at line column 5. With `tet` selected the lines shorten at
//! its codon edges, and block 1 covers one-based 67 to 130 (`tet66[1]` in the
//! 8m4 §4 rule 2 table, whose four lanes are the two `tet` features and the two
//! `misc_binding` sites at 67..70 and 80..83). `tet` is `86..1276`, so its first
//! base stands at column `5 + (86 − 67) = 24` and its run covers 45 columns up
//! to the block's last base.
//!
//! ## How each value was found
//!
//! Every number above comes from the fixture and the coordinates, never from what
//! the code draws: the file's own `FEATURES` block puts `gene` and `CDS` at
//! `86..1276`, both `/gene="tet"`; the four `misc_binding` sites at 67..70 and
//! the one at 80..83 carry `/bound_moiety="echinomycin"` and no `/gene`; the
//! LOCUS line and `//ORIGHT:4361 OLEFT:4361` give the 4361 bases and their
//! circularity; and the selected-CDS edges come from the 8m4 §4 rule 2 table,
//! which was made from the Python reference (`tests/fixtures/view/cds_map.py`,
//! `frame_tables.py`): the table's `tet66[1]` is block 1 here.
//!
//! `crowd` is written here, as `sequence_view_jumps.rs` writes its 30 bp record:
//! 30 bases, `d = 2`, so a line holds `⌊(78 − 2·2 − 2) / 3⌋ · 3 = 72` bases and
//! the whole record is block 0, whose first base stands at column 3. Its five
//! `misc_feature 1..20` each cover 20 bases and its `CDS 1..9` covers 9, and the
//! lane sort is by the bases a feature covers, longest first, then file order
//! (D053) — so the five 20-base features come first, in the order `m1` to `m5`,
//! and the CDS is the sixth. At width 78 with no CDS selected a block has four
//! lanes, so `c1` is one of the two features the `+N more` line counts. Its
//! `LOCUS` line gives the 30 bases and `linear`, so `d = 2`, and its `FEATURES`
//! block holds the five `misc_feature 1..20` and the `CDS 1..9`, each named by
//! `/gene`; the lane order follows the number of bases each feature covers, the
//! D053 rule `layout::lanes` spells out.

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
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Terminal;
use std::fs;
use std::path::Path;

/// The size every sequence-view test draws at: 80 columns and 24 rows, whose
/// frame has 78 inner columns, whose status line is row 21 and whose key line is
/// row 23.
const W: u16 = 80;
const H: u16 = 24;

/// The status line's screen row: the frame's last inner row (8k1 §4 rule 1), as
/// `sequence_view_keys.rs` reads it.
const STATUS: usize = 21;

/// The body rows while the overview strip is not drawn, which is how a list is
/// drawn (8m3 §4 rule 2): the frame's inner rows less the status line's one.
const BODY_WITHOUT: std::ops::Range<usize> = 1..21;

/// pBR322: 4361 bp, three CDSs of which two are usable — `tet` at `86..1276` on
/// the plus strand and `bla` on the minus — so `tet` is CDS 0 and the view
/// selects none of them on open (M8-11).
fn pbr() -> ViewRecord {
    let path: &str = "../../tests/fixtures/genbank/raw/J01749.1.gb";
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let text: String =
        fs::read_to_string(&file).unwrap_or_else(|error| panic!("read {path}: {error}"));
    open_record(Identifier::create("SYNPBR322".to_string()).unwrap(), text)
        .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
}

/// The 30 bp linear record §5 writes: five `misc_feature 1..20` named `m1` to
/// `m5` and one `CDS 1..9` named `c1`, each named by its `/gene` (D053's name
/// rule). The bases themselves are `ATGAAATAA` and 21 `C`s.
fn crowd() -> ViewRecord {
    let text: &str = r#"LOCUS       crowd                     30 bp    DNA     linear   SYN 27-OCT-2026
FEATURES             Location/Qualifiers
     source          1..30
                     /organism="synthetic construct"
                     /mol_type="other DNA"
     misc_feature    1..20
                     /gene="m1"
     misc_feature    1..20
                     /gene="m2"
     misc_feature    1..20
                     /gene="m3"
     misc_feature    1..20
                     /gene="m4"
     misc_feature    1..20
                     /gene="m5"
     CDS             1..9
                     /gene="c1"
ORIGIN
        1 atgaaataaa cccccccccc cccccccccc
//
"#;
    open_record(
        Identifier::create("crowd".to_string()).unwrap(),
        text.to_string(),
    )
    .unwrap_or_else(|error| panic!("open the 30 bp record: {error:?}"))
}

/// One block's lines, drawn with the view's colours off and the level
/// `features` (8m6 §4 rule 1).
fn lines_at(
    record: &ViewRecord,
    laid: &Layout,
    mode: &LaneMode,
    block: usize,
    features: FeatureLevel,
) -> Vec<(LineRole, Line<'static>)> {
    block_lines(
        record,
        laid,
        mode,
        block,
        &Drawing {
            colours: false,
            features,
        },
    )
    .unwrap_or_else(|problem| panic!("block {block} at {features:?}: {problem:?}"))
}

/// The roles of `lines`, in the order they are drawn.
fn roles(lines: &[(LineRole, Line<'static>)]) -> Vec<LineRole> {
    lines.iter().map(|(role, _line)| role.clone()).collect()
}

/// Draw `lines` on a test backend `width` columns wide, and return the symbol of
/// every cell, row by row — the `layout.rs` reading of the test backend.
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

/// The screen drawn on a test backend: every row as the symbols of its cells.
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

/// Every non-blank body row in `given`, in screen order: what the body holds, and
/// nothing else.
fn body_rows(state: &ViewState, given: std::ops::Range<usize>) -> Vec<String> {
    given
        .map(|row| body_row(state, row))
        .filter(|row| !row.is_empty())
        .collect()
}

/// `key` pressed once, with a failure saying it leaves.
fn press(state: &mut ViewState, key: Key) {
    assert_eq!(
        state.handle(key),
        Action::Continue,
        "{key:?} does not leave"
    );
}

/// §5 test 1: `F` at CDSs only keeps the CDS lanes and nothing else, and at none
/// it leaves the whole block with no lane line and no `+N more` line (M8-30).
#[test]
fn cdss_only_keeps_the_cds_lanes() {
    let record: ViewRecord = pbr();
    let laid: Layout = layout(&record, 78, &LaneMode::Cds(0)).expect("tet's own lines at 78");
    // 8m4 §4 rule 2: `tet`'s codon 131..133 would straddle 132, so the line from
    // 67 ends at 130 — 64 bases — and its bases stand at columns 5 to 68.
    assert_eq!(laid.blocks[1], (66, 130));

    // All: the four lanes the 8m4 table lists — the `tet` gene, the `tet` CDS and
    // the `misc_binding` sites at 67..70 and 80..83, longest first and then file
    // order — and nothing left for the `+N more` line.
    let all = lines_at(&record, &laid, &LaneMode::Cds(0), 1, FeatureLevel::All);
    assert_eq!(
        roles(&all),
        vec![
            LineRole::Lane(1),
            LineRole::Lane(2),
            LineRole::Lane(3),
            LineRole::Lane(4),
            LineRole::Residues,
            LineRole::Numbers,
            LineRole::Ruler,
            LineRole::Top,
            LineRole::Bottom,
        ],
        "all the features fill the four lanes, and there is no more line"
    );

    // CDSs only: the `tet` CDS alone, at the same column and the same width the
    // table gives it, and the residues, numbers and strands unchanged.
    let cds = lines_at(&record, &laid, &LaneMode::Cds(0), 1, FeatureLevel::Cdss);
    assert_eq!(
        roles(&cds),
        vec![
            LineRole::Lane(1),
            LineRole::Residues,
            LineRole::Numbers,
            LineRole::Ruler,
            LineRole::Top,
            LineRole::Bottom,
        ],
        "one `CDS` is one lane, and the two `misc_binding` sites and the `gene` leave"
    );
    let rows: Vec<Vec<String>> = drawn(&cds, 78);
    // `tet`'s first base, one-based 86, is 19 bases into the block that starts at
    // 67, and bases start at column 5: its text begins at column 24.
    let lane: usize = roles(&cds)
        .iter()
        .position(|role| *role == LineRole::Lane(1))
        .expect("a lane line at CDSs only");
    assert_eq!(span(&rows[lane], 23, 23), " ", "column 23 is blank");
    assert_eq!(
        span(&rows[lane], 24, 28),
        "tet ▸",
        "the lane's text at column 24 is its name and its strand arrow"
    );

    // None: no lane line at all, and no `+N more` line to count what is not there.
    let none = lines_at(&record, &laid, &LaneMode::Cds(0), 1, FeatureLevel::None);
    assert_eq!(
        roles(&none),
        vec![
            LineRole::Residues,
            LineRole::Numbers,
            LineRole::Ruler,
            LineRole::Top,
            LineRole::Bottom,
        ],
        "no feature gets a lane, so nothing is left over to count either"
    );
}

/// §5 test 2: the level filters what the four lanes are chosen from, not what
/// they hold — a CDS the `All` lanes leave out of the four is drawn when the
/// level is `CDSs only`, and the `+N more` line goes with it (M8-30).
#[test]
fn a_cds_is_never_left_in_more() {
    let record: ViewRecord = crowd();
    let laid: Layout = layout(&record, 78, &LaneMode::Off).expect("30 bases on one line");
    // `d = 2` and a 72-base line: the record is one block, and its first base
    // stands at column 3.
    assert_eq!(laid.blocks, vec![(0, 30)]);
    assert_eq!(laid.gutter, 2);

    // All: the four `misc_feature`s in file order, the CDS counted on the
    // `+N more` line with the fifth feature.
    let all = lines_at(&record, &laid, &LaneMode::Off, 0, FeatureLevel::All);
    assert_eq!(
        roles(&all),
        vec![
            LineRole::Lane(1),
            LineRole::Lane(2),
            LineRole::Lane(3),
            LineRole::Lane(4),
            LineRole::More,
            LineRole::Ruler,
            LineRole::Top,
            LineRole::Bottom,
        ],
        "five 20-base features fill the four lanes and leave the CDS and one more"
    );
    let rows: Vec<Vec<String>> = drawn(&all, 78);
    for (lane, name) in rows[..4].iter().zip(["m1", "m2", "m3", "m4"]) {
        // Each run covers its 20 bases from base 1 at column 3.
        let text: String = span(lane, 3, 22);
        assert!(
            text.starts_with(&format!("{name} ▸")),
            "the lane of {name} spells its name from column 3: {text:?}"
        );
        assert!(
            !text.contains("c1"),
            "no `All` lane holds the CDS, which the four lanes did not choose: {text:?}"
        );
    }
    assert_eq!(
        span(&rows[4], 3, 9),
        "+2 more",
        "the four lanes leave `m5` and `c1` to be counted"
    );

    // CDSs only: `c1` takes the lane that the five equal-length features had put
    // `c1` out of the running for, and nothing is left over.
    let cds = lines_at(&record, &laid, &LaneMode::Off, 0, FeatureLevel::Cdss);
    assert_eq!(
        roles(&cds),
        vec![
            LineRole::Lane(1),
            LineRole::Ruler,
            LineRole::Top,
            LineRole::Bottom
        ],
        "one `CDS` is one lane, and no `+N more` line is drawn"
    );
    let rows: Vec<Vec<String>> = drawn(&cds, 78);
    // `c1` covers bases 1 to 9, so its run is nine columns wide from column 3.
    assert_eq!(
        span(&rows[0], 3, 11),
        "c1 ▸ ════",
        "the CDS's lane spells its name and fills its nine bases"
    );

    // None: the block's two strands, and nothing else.
    let none = lines_at(&record, &laid, &LaneMode::Off, 0, FeatureLevel::None);
    assert_eq!(
        roles(&none),
        vec![LineRole::Ruler, LineRole::Top, LineRole::Bottom],
        "no lane line, and no `+N more` line beside it"
    );
}

/// §5 test 3: `F` steps the three levels in order and names the one it reached,
/// is text inside an open list, and `?` says what the key cycles (M8-30).
#[test]
fn f_steps_through_three_levels() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert_eq!(
        state.features(),
        FeatureLevel::All,
        "the view opens with every feature getting a lane"
    );

    press(&mut state, Key::Char('F'));
    assert_eq!(
        state.features(),
        FeatureLevel::Cdss,
        "the first `F` moves to the CDSs only"
    );
    let status: String = rows(&state)[STATUS].trim_end().to_string();
    assert!(
        status.contains("CDS"),
        "the status line names the level reached: {status}"
    );

    press(&mut state, Key::Char('F'));
    assert_eq!(
        state.features(),
        FeatureLevel::None,
        "the second `F` moves to none"
    );
    press(&mut state, Key::Char('F'));
    assert_eq!(
        state.features(),
        FeatureLevel::All,
        "the third `F` wraps back to all the features"
    );

    // Inside an open list every printable key is filter text, the level aside
    // (8m3 §4 rule 1, 8m6 §4 rule 3).
    press(&mut state, Key::Char('f'));
    press(&mut state, Key::Char('F'));
    assert_eq!(
        state.list().expect("the feature list").filter(),
        "F",
        "the list's filter holds the `F` that was typed there"
    );
    assert_eq!(
        state.features(),
        FeatureLevel::All,
        "and the `F` in the filter has moved no level"
    );

    press(&mut state, Key::Esc);
    assert!(state.list().is_none(), "esc closes the list");
    press(&mut state, Key::Char('?'));
    assert!(state.help(), "? opens the key list");
    let listed: Vec<String> = body_rows(&state, BODY_WITHOUT.clone());
    let f_rows: Vec<String> = listed
        .iter()
        .filter(|row| row.starts_with("F "))
        .cloned()
        .collect();
    assert_eq!(f_rows.len(), 1, "one row starts with `F `: {listed:?}");
    assert!(
        f_rows[0].contains("CDS"),
        "the `F` row says what the key cycles through: {f_rows:?}"
    );
}
