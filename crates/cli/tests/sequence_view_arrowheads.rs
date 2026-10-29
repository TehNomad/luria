//! A lane run's arrowhead stands at the feature's 3′ end (contract 8m11 §4,
//! M8-34). The one run that holds that end draws `▸` in its last column on the
//! plus strand and `◂ name ` before its name on the minus strand; every other
//! run reads its name, a space and the bar, with no arrow after the name. Rule 3
//! leaves the overview strip alone, which 8l2's own tests hold.
//!
//! Every value comes from contract §5, which the planner derived from the record
//! and the tables, and not from what the code answers. `pbr` is
//! `tests/fixtures/genbank/raw/J01749.1.gb` at width 78: `d = 4`, so a base's
//! column is 5 plus its zero-based offset. `syn` is
//! `tests/fixtures/genbank/synthetic.gb` at width 78: `d = 2`, so its first base
//! stands at column 3. The fourth test's record is §6's, written here.
//!
//! Each case also names the rows of `tests/fixtures/view/layout-cells.tsv`,
//! which the reference script writes: the run holding the head is kind
//! `feature_end` and the runs before it stay `feature`. The rows are quoted in
//! the checks, and one check reports where §5 and the rows disagree.
//!
//! `block_lines` writes one span per cell and one per gap, so a line's spans
//! concatenated in order give its columns, which is how §6 asks for them to be
//! read. A lane is found by what it spells, not by the line it lands on, and how
//! many lanes spell it is part of the check: §5 names the `tet` gene and CDS
//! together, and in block 49 the `bla` gene, CDS and `mat_peptide`, all three
//! named `bla`.

use luria_cli::sequence_view::layout::block_lines;
use luria_cli::sequence_view::layout::layout;
use luria_cli::sequence_view::layout::Drawing;
use luria_cli::sequence_view::layout::FeatureLevel;
use luria_cli::sequence_view::layout::LaneMode;
use luria_cli::sequence_view::layout::Layout;
use luria_cli::sequence_view::layout::LineRole;
use luria_core::identity::Identifier;
use luria_core::view_record::open_record;
use luria_core::view_record::ViewRecord;
use ratatui::text::Line;
use std::fs;
use std::path::Path;

/// 8m4's cases are drawn at 78 inner columns, and §6's fourth test at 40.
const WIDE: u16 = 78;
const NEAR: u16 = 40;

/// The head's two glyphs and the bar they sit on (rule 2).
const PLUS: char = '▸';
const MINUS: char = '◂';
const BAR: char = '═';

/// A bar of `count` columns, the fill rule 2's body ends with.
fn bar(count: usize) -> String {
    String::from(BAR).repeat(count)
}

fn pbr() -> ViewRecord {
    let path: &str = "../../tests/fixtures/genbank/raw/J01749.1.gb";
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let text: String =
        fs::read_to_string(&file).unwrap_or_else(|error| panic!("read {path}: {error}"));
    open_record(Identifier::create("synhead".to_string()).unwrap(), text)
        .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
}

fn syn() -> ViewRecord {
    let path: &str = "../../tests/fixtures/genbank/synthetic.gb";
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let text: String =
        fs::read_to_string(&file).unwrap_or_else(|error| panic!("read {path}: {error}"));
    open_record(Identifier::create("synshrt".to_string()).unwrap(), text)
        .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
}

/// §6's fourth record: 20 bp, linear, one `misc_feature` on base 5 and one on
/// base 9 written on the minus strand. With no `LONGCT` and 20 bp, `d = 2`, so
/// the gutter is 3 columns wide: base 5 stands at column 7, base 9 at column 11.
fn single() -> ViewRecord {
    let text: &str = r#"LOCUS       heads                     20 bp    DNA     linear   SYN 28-OCT-2026
FEATURES             Location/Qualifiers
     source          1..20
                     /organism="synthetic construct"
                     /mol_type="other DNA"
     misc_feature    5..5
                     /gene="plus"
     misc_feature complement(9..9)
                     /gene="minus"
ORIGIN
        1 atgtcgtaag cagctgactt
//
"#;
    open_record(
        Identifier::create("heads".to_string()).unwrap(),
        text.to_string(),
    )
    .unwrap_or_else(|error| panic!("open the 20 bp record: {error:?}"))
}

/// The one block's lines, drawn with the colours off and every feature the level
/// shows (`FeatureLevel::All`), which is how the tables hold the level for their
/// `off` and `cds:` modes.
fn lines_at(
    record: &ViewRecord,
    laid: &Layout,
    mode: &LaneMode,
    block: usize,
) -> Vec<(LineRole, Line<'static>)> {
    block_lines(
        record,
        laid,
        mode,
        block,
        &Drawing {
            colours: false,
            features: FeatureLevel::All,
            orfs: &[],
        },
    )
    .unwrap_or_else(|problem| panic!("block {block} at {mode:?}: {problem:?}"))
}

/// A line's cells as one char per layout column: the drawing writes one span per
/// cell and one per gap, so its spans in order are its columns (8m7 §4 rule 4).
fn columns(line: &Line<'static>) -> Vec<char> {
    line.spans
        .iter()
        .flat_map(|span| span.content.chars())
        .collect()
}

/// The columns `first` to `last`, both at their ends, as a string. `block_lines`
/// trims a line's trailing gaps, so a column past the line's last cell is blank.
fn text(row: &[char], first: usize, last: usize) -> String {
    (first..=last)
        .map(|column| row.get(column).copied().unwrap_or(' '))
        .collect()
}

/// A line's drawn text, its trailing gaps trimmed off, for a failure message.
fn spelled(row: &[char]) -> String {
    row.iter().collect::<String>().trim_end().to_string()
}

/// How many head glyphs a line holds, on either strand.
fn heads(row: &[char]) -> usize {
    row.iter()
        .filter(|ch| **ch == PLUS || **ch == MINUS)
        .count()
}

/// The drawn text of every lane line, in the order they are drawn.
fn lanes(lines: &[(LineRole, Line<'static>)]) -> Vec<(LineRole, Vec<char>)> {
    lines
        .iter()
        .filter(|(role, _line)| matches!(role, LineRole::Lane(_)))
        .map(|(role, line)| (role.clone(), columns(line)))
        .collect()
}

/// The lanes that spell `want` from column `first` to column `last`, both at
/// their ends: §6 asks that a lane be found by its text, and how many match says
/// whether §5's named features are the ones drawing it.
fn spellers(
    lines: &[(LineRole, Line<'static>)],
    first: usize,
    last: usize,
    want: &str,
) -> Vec<LineRole> {
    lanes(lines)
        .into_iter()
        .filter(|(_role, row)| text(row, first, last) == want)
        .map(|(role, _row)| role)
        .collect()
}

/// The lanes, spelled out for a failure that should name them all.
fn shown(lines: &[(LineRole, Line<'static>)]) -> Vec<(LineRole, String)> {
    lanes(lines)
        .iter()
        .map(|(role, row)| (role.clone(), spelled(row)))
        .collect()
}

/// §5's first and second hand values: the two `tet` features on the plus strand,
/// `86..1276`, hold their 3′ end at base 1276 — column 28 of block 19 and no
/// column of block 1 — and the join's second part opens block 19, so the same
/// run opens a block too.
#[test]
fn the_head_is_at_the_three_prime_end() {
    let record: ViewRecord = pbr();
    let mode: LaneMode = LaneMode::Cds(0);
    assert_eq!(
        record.cdss[0].id.as_deref(),
        Some("tet"),
        "CDS 0 is `tet`, §5's `cds:tet`"
    );
    let laid: Layout = layout(&record, WIDE, &mode).expect("pbr at width 78");
    assert_eq!(laid.gutter, 4, "the record reads 4361 long, so d = 4");
    assert_eq!(
        laid.blocks[19],
        (1252, 1318),
        "block 19 is §5's, bases 1253 to 1318"
    );
    assert_eq!(
        laid.blocks[1],
        (66, 130),
        "block 1 is §5's, bases 67 to 130"
    );

    // §5's first value: the `tet` gene and CDS each have a 24-column run at
    // columns 5 to 28, the head at its last column, where base 1276 stands. The
    // row is `pbr322 78 cds:tet 19 0 lane1 feature_end 5 24 tet (+)`.
    let block: Vec<(LineRole, Line<'static>)> = lines_at(&record, &laid, &mode, 19);
    let head: String = format!("tet {}{PLUS}", bar(19));
    assert_eq!(
        spellers(&block, 5, 28, &head),
        vec![LineRole::Lane(1), LineRole::Lane(2)],
        "the `tet` gene and CDS lanes both run from column 5 with the head at \
         column 28: {:?}",
        shown(&block)
    );
    for (role, row) in lanes(&block) {
        assert_eq!(
            heads(&row),
            1,
            "lane {role:?} of §5's block 19 holds one head: {row:?}"
        );
    }

    // §5's second value: the same two runs in block 1, 45 columns wide from
    // column 24, hold no head, because base 1276 lies in block 19. The row is
    // `pbr322 78 cds:tet 1 0 lane1 feature 24 45 tet (+)`.
    let block: Vec<(LineRole, Line<'static>)> = lines_at(&record, &laid, &mode, 1);
    let run: String = format!("tet {}", bar(41));
    assert_eq!(
        spellers(&block, 24, 68, &run),
        vec![LineRole::Lane(1), LineRole::Lane(2)],
        "the `tet` runs read their name and the bar, with no arrow: {:?}",
        shown(&block)
    );
    for (role, row) in lanes(&block).into_iter().take(2) {
        assert_eq!(
            heads(&row),
            0,
            "the `tet` lane {role:?} holds no head: {row:?}"
        );
    }

    // §5's second value adds "No `▸` or `◂` appears on any lane line of that
    // block", and the two `misc_binding` sites that 8m7 puts in lanes 3 and 4 of
    // that block contradict it: `misc_binding 67..70` and `80..83` are whole
    // features, so rule 1 puts their 3′ ends at bases 70 and 83, and their rows,
    // `pbr322 78 cds:tet 1 2 lane3 feature_end 5 4 misc_binding (+)` and
    // `1 3 lane4 feature_end 18 4 misc_binding (+)`, are `feature_end`.
    assert_eq!(
        spellers(&block, 5, 8, &format!("mis{PLUS}")),
        vec![LineRole::Lane(3)],
        "the `misc_binding 67..70` run ends at base 70, its own 3′ end: {:?}",
        shown(&block)
    );
    assert_eq!(
        spellers(&block, 18, 21, &format!("mis{PLUS}")),
        vec![LineRole::Lane(4)],
        "the `misc_binding 80..83` run ends at base 83, its own 3′ end: {:?}",
        shown(&block)
    );
}

/// §5's third and fourth hand values: `complement(3293..4153)` ends at base
/// 3293, the first interval's start, so its 3′ end opens block 49, and the run
/// that fills block 62 holds its 5′ end and draws no head.
#[test]
fn a_minus_strand_head_points_left() {
    let record: ViewRecord = pbr();
    let mode: LaneMode = LaneMode::Cds(2);
    assert_eq!(
        record.cdss[2].id.as_deref(),
        Some("bla"),
        "CDS 2 is `bla`, §5's `cds:bla`: `tet`, the unnamed CDS, then `bla`"
    );
    let laid: Layout = layout(&record, WIDE, &mode).expect("pbr at width 78");
    assert_eq!(
        laid.blocks[49],
        (3234, 3298),
        "block 49 is §5's, bases 3235 to 3298"
    );
    assert_eq!(
        laid.blocks[62],
        (4090, 4156),
        "block 62 is §5's, bases 4091 to 4156"
    );

    // §5's third value: `◂ bla ` in six columns at 63 to 68, for the gene and the
    // CDS. Their rows are `pbr322 78 cds:bla 49 1 lane2 feature_end 63 6 bla (-)`
    // and `pbr322 78 cds:bla 49 2 lane3 feature_end 63 6 bla (-)`.
    let block: Vec<(LineRole, Line<'static>)> = lines_at(&record, &laid, &mode, 49);
    assert_eq!(
        spellers(&block, 63, 68, "◂ bla "),
        vec![LineRole::Lane(2), LineRole::Lane(3)],
        "the `bla` gene and CDS run into the block's end and point left: {:?}",
        shown(&block)
    );
    // Lane 4 is `mat_peptide`, `complement(3296..4084)`, whose 3′ end, base 3296,
    // stands at column 66 where its 3-column run opens: the head, a space, and the
    // name — `bla`, its D053 display name — cut to the one letter the run holds.
    // The row is `pbr322 78 cds:bla 49 3 lane4 feature_end 66 3 bla (-)`.
    assert_eq!(
        spellers(&block, 66, 68, "◂ b"),
        vec![LineRole::Lane(4)],
        "the head leads, and the name is cut to the three columns: {:?}",
        shown(&block)
    );

    // §5's fourth value: the three `bla` runs that fill block 62 hold the 5′ end,
    // not the 3′ end, and read `bla ` and the bar with no head: the gene, the CDS
    // and `sig_peptide`, `complement(4085..4153)`, whose 3′ end lies outside the
    // block. The rows are `pbr322 78 cds:bla 62 1 lane2 feature 5 63 bla (-)`,
    // `62 2 lane3 feature 5 63 bla (-)` and `62 3 lane4 feature 5 63 bla (-)`.
    let block: Vec<(LineRole, Line<'static>)> = lines_at(&record, &laid, &mode, 62);
    let run: String = format!("bla {}", bar(59));
    assert_eq!(
        spellers(&block, 5, 67, &run),
        vec![LineRole::Lane(2), LineRole::Lane(3), LineRole::Lane(4)],
        "the `bla` runs that open the block hold no head: {:?}",
        shown(&block)
    );
    for (role, row) in lanes(&block) {
        assert_eq!(
            heads(&row),
            0,
            "no head glyph stands on lane {role:?} of §5's block 62: {row:?}"
        );
    }
}

/// §5's fifth hand value and rule 1's origin join: `join(55..60,1..12)` ends at
/// zero-based 11, in the run that opens the record, and its other run at the
/// block's far end holds no head.
#[test]
fn an_origin_join_has_one_head() {
    let record: ViewRecord = syn();
    let mode: LaneMode = LaneMode::Off;
    let laid: Layout = layout(&record, WIDE, &mode).expect("synthetic at width 78");
    assert_eq!(laid.gutter, 2, "the 60 bp record has d = 2, so g = 3");
    assert_eq!(laid.blocks[0], (0, 60), "one block holds the record");
    assert_eq!(
        laid.blocks.len(),
        1,
        "the join breaks at 60 and 0, which are the block's own ends"
    );

    // §5: `pj `, eight `═`, then the head at column 14, where base 12 stands. The
    // row is `synthetic 78 off 0 0 lane1 feature_end 3 12 pj (+)`.
    let block: Vec<(LineRole, Line<'static>)> = lines_at(&record, &laid, &mode, 0);
    let head: String = format!("pj {}{PLUS}", bar(8));
    assert_eq!(
        spellers(&block, 3, 14, &head),
        vec![LineRole::Lane(1)],
        "the run of bases 1 to 12 holds the join's 3′ end: {:?}",
        shown(&block)
    );
    // Its other run, the join's first part, reads `pj ` and three `═` with no head
    // at all. The row is `synthetic 78 off 0 0 lane1 feature 57 6 pj (+)`.
    assert_eq!(
        spellers(&block, 57, 62, &format!("pj {}", bar(3))),
        vec![LineRole::Lane(1)],
        "the run of bases 55 to 60 is the join's 5′ end: {:?}",
        shown(&block)
    );
    for (role, row) in lanes(&block) {
        assert_eq!(
            heads(&row),
            1,
            "lane {role:?} holds one head, at the 3′ end: {row:?}"
        );
    }
}

/// §6's fourth test: a run of one column with the head is the head alone, on both
/// strands — the single-base feature is a whole feature, so its one base is at
/// once its 5′ end and its 3′ end.
#[test]
fn a_one_column_run_is_its_head() {
    let record: ViewRecord = single();
    let mode: LaneMode = LaneMode::Off;
    let laid: Layout = layout(&record, NEAR, &mode).expect("20 bases at width 40");
    assert_eq!(laid.gutter, 2, "d = 2, so the gutter is 3 columns wide");
    assert_eq!(laid.blocks[0], (0, 20), "one block holds the record");

    let block: Vec<(LineRole, Line<'static>)> = lines_at(&record, &laid, &mode, 0);
    assert_eq!(
        spellers(&block, 7, 7, "▸"),
        vec![LineRole::Lane(1)],
        "the plus strand's one-column run is its head alone: {:?}",
        shown(&block)
    );
    assert_eq!(
        spellers(&block, 11, 11, "◂"),
        vec![LineRole::Lane(2)],
        "the minus strand's one-column run is its head alone: {:?}",
        shown(&block)
    );
    for (role, row) in lanes(&block) {
        assert_eq!(
            heads(&row),
            1,
            "lane {role:?} draws nothing but its head: {row:?}"
        );
    }
}
