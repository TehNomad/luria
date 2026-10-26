//! Every row of the two 8a2 column tables, read by the layout and drawn on a
//! `ratatui` test backend (contract 8j; the rule text is the block-layout
//! section of `tests/fixtures/view/README.md`).
//!
//! `layout-blocks.tsv` pins how a record splits into blocks — one row per
//! block, all 645 of them — and `layout-cells.tsv` pins one row per item: the
//! line it is on, the column it starts in, how many columns it covers, and what
//! it is. Every row of both tables is checked here, not a sample: a row the
//! renderer cannot place is a failure. The cells table itself samples blocks —
//! it holds items for 24 of the blocks, a case's first block and, when the case
//! has more than one, the block holding its last bases — and the blocks table
//! carries the rest. Nothing in either table is read back from Rust: the tables
//! and `tests/fixtures/view/reference/layout_columns.py` predate this module by
//! two milestones.
//!
//! The cells table pins no glyphs, so a `feature` row records its item as
//! `"tet (+)"` while the line shows the name, an arrow and the bar cut to the
//! run (contract 8j rule 3): `check_cells` builds that text from the name and
//! strand marker the row holds, and compares it with what was drawn.
//!
//! The two tables between them pin every rule of §5.1 except four, which
//! `sequence_view::layout`'s own tests hold: the gutter's minimum of 2 (no
//! record in either table is shorter than 10 bases), an unusable CDS as a
//! selection (the tables name only usable ones), the `*` style (the tables hold
//! columns and text, not style), and the lane's glyphs — the arrow and the
//! `═` fill — which the cells table records as a name and a strand marker
//! instead.

use luria_cli::sequence_view::layout::block_lines;
use luria_cli::sequence_view::layout::layout;
use luria_cli::sequence_view::layout::LaneMode;
use luria_cli::sequence_view::layout::Layout;
use luria_cli::sequence_view::layout::LineRole;
use luria_core::frames::ReadingFrame;
use luria_core::identity::Identifier;
use luria_core::strand::Strand;
use luria_core::view_record::open_record;
use luria_core::view_record::ViewRecord;
use ratatui::backend::TestBackend;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Terminal;
use std::fs;
use std::path::Path;

/// One data row of `layout-blocks.tsv`: a case's `line_width` is recorded on
/// every one of its rows, so the table needs no line of its own for it.
#[derive(Clone)]
struct BlocksRow {
    record: String,
    width: u16,
    mode: String,
    line_width: u64,
    block: usize,
    first_base: u64,
    last_base: u64,
    bases: u64,
}

/// One data row of `layout-cells.tsv`: one item of one block. The cells table
/// samples blocks rather than listing every one: it holds rows for 24 of the
/// 645 blocks the blocks table lists, a case's first block and, when the case
/// has more than one, the block holding its last bases. Every row it does hold
/// is checked here.
#[derive(Clone)]
struct CellsRow {
    record: String,
    width: u16,
    mode: String,
    block: usize,
    line: usize,
    role: String,
    kind: String,
    column: u64,
    cells: u64,
    value: String,
}

/// The four records the tables name, opened once per test. Each path is a
/// fixture as it stands in the repository — `pbr322` is the raw GenBank file,
/// `framesamb` is FASTA — and `open_record` decides the format from the content
/// rather than the name (D049).
fn records() -> Vec<(String, ViewRecord)> {
    [
        ("synthetic", "../../tests/fixtures/genbank/synthetic.gb"),
        ("pbr322", "../../tests/fixtures/genbank/raw/J01749.1.gb"),
        ("frames62", "../../tests/fixtures/view/frames-62.gb"),
        ("framesamb", "../../tests/fixtures/view/frames-amb.fa"),
    ]
    .into_iter()
    .map(|(name, path)| {
        let full = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
        let text = fs::read_to_string(&full)
            .unwrap_or_else(|error| panic!("read {}: {error}", full.display()));
        let record = open_record(Identifier::create(name.to_string()).unwrap(), text)
            .unwrap_or_else(|error| panic!("open {name} ({path}): {error:?}"));
        (name.to_string(), record)
    })
    .collect()
}

fn lookup<'a>(opened: &'a [(String, ViewRecord)], name: &str) -> &'a ViewRecord {
    opened
        .iter()
        .find(|(found, _)| found == name)
        .map(|(_, record)| record)
        .unwrap_or_else(|| panic!("the tables name a record this test does not open: {name}"))
}

/// One tab-separated table, header dropped, every row the same width.
fn table(file: &str, columns: usize) -> Vec<Vec<String>> {
    let full = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/view")
        .join(file);
    let text = fs::read_to_string(&full)
        .unwrap_or_else(|error| panic!("read {}: {error}", full.display()));
    let mut lines = text.lines();
    let header = lines.next().expect("a header row");
    assert_eq!(
        header.split('\t').count(),
        columns,
        "{file}'s header names {columns} columns"
    );
    let rows: Vec<Vec<String>> = lines
        .filter(|line| !line.is_empty())
        .map(|line| line.split('\t').map(|field| field.to_string()).collect())
        .collect();
    for row in rows.iter() {
        assert_eq!(
            row.len(),
            columns,
            "{file}: expected {columns} fields, got {}:\n{}",
            row.len(),
            row.join("\t")
        );
    }
    rows
}

fn number(field: &str, file: &str, row: usize, column: usize) -> u64 {
    field.parse().unwrap_or_else(|error| {
        panic!("{file} row {row} column {column}: {field:?} is not a number: {error}")
    })
}

fn blocks_rows() -> Vec<BlocksRow> {
    table("layout-blocks.tsv", 8)
        .into_iter()
        .enumerate()
        .map(|(index, fields)| BlocksRow {
            record: fields[0].clone(),
            width: number(&fields[1], "layout-blocks.tsv", index, 1) as u16,
            mode: fields[2].clone(),
            line_width: number(&fields[3], "layout-blocks.tsv", index, 3),
            block: number(&fields[4], "layout-blocks.tsv", index, 4) as usize,
            first_base: number(&fields[5], "layout-blocks.tsv", index, 5),
            last_base: number(&fields[6], "layout-blocks.tsv", index, 6),
            bases: number(&fields[7], "layout-blocks.tsv", index, 7),
        })
        .collect()
}

fn cells_rows() -> Vec<CellsRow> {
    table("layout-cells.tsv", 10)
        .into_iter()
        .enumerate()
        .map(|(index, fields)| CellsRow {
            record: fields[0].clone(),
            width: number(&fields[1], "layout-cells.tsv", index, 1) as u16,
            mode: fields[2].clone(),
            block: number(&fields[3], "layout-cells.tsv", index, 3) as usize,
            line: number(&fields[4], "layout-cells.tsv", index, 4) as usize,
            role: fields[5].clone(),
            kind: fields[6].clone(),
            column: number(&fields[7], "layout-cells.tsv", index, 7),
            cells: number(&fields[8], "layout-cells.tsv", index, 8),
            value: fields[9].clone(),
        })
        .collect()
}

/// The mode the table's third column asks for. `off`, `three` and `six` are the
/// three modes that name no CDS; `cds:NAME` names a CDS by its D041 id, which
/// is how every other 8a2 table names one (D052, D053), and resolves to its
/// index in file order — not to a residue number.
fn mode_for(record: &ViewRecord, text: &str) -> LaneMode {
    match text {
        "off" => LaneMode::Off,
        "three" => LaneMode::ThreeFrames,
        "six" => LaneMode::SixFrames,
        _ => match text.strip_prefix("cds:") {
            Some(name) => {
                let index: usize = record
                    .cdss
                    .iter()
                    .position(|cds| cds.id.as_deref() == Some(name))
                    .unwrap_or_else(|| {
                        panic!(
                            "no CDS with D041 id {name:?} in this record; ids are {:?}",
                            record
                                .cdss
                                .iter()
                                .map(|cds| cds.id.clone())
                                .collect::<Vec<_>>()
                        )
                    });
                LaneMode::Cds(index)
            }
            None => panic!("the tables name a mode this test does not know: {text:?}"),
        },
    }
}

/// The line role the table's sixth column names.
fn role_for(text: &str) -> LineRole {
    if let Some(digit) = text.strip_prefix("lane") {
        return LineRole::Lane(digit.parse().expect("lane1 to lane4"));
    }
    match text {
        "more" => LineRole::More,
        "residues" => LineRole::Residues,
        "numbers" => LineRole::Numbers,
        "top" => LineRole::Top,
        "bottom" => LineRole::Bottom,
        _ => {
            for (sign, strand) in [('+', Strand::Plus), ('-', Strand::Minus)] {
                if let Some(index) = text.strip_prefix(sign) {
                    let frame: ReadingFrame = index
                        .parse::<u64>()
                        .ok()
                        .and_then(ReadingFrame::from_index)
                        .unwrap_or_else(|| panic!("{text:?} names no reading frame"));
                    return LineRole::Frame(strand, frame);
                }
            }
            panic!("the tables name a line role this test does not know: {text:?}")
        }
    }
}

/// Every non-blank cell the test backend drew, row by row, as symbols
/// (`review_view.rs`'s idiom): `lines` drawn `width` columns wide, one row per
/// line, and each row is exactly `width` symbols long.
fn drawn(lines: &[(LineRole, Line<'static>)], width: u16) -> Vec<Vec<String>> {
    let text: Vec<Line<'static>> = lines.iter().map(|(_, line)| line.clone()).collect();
    let mut terminal = Terminal::new(TestBackend::new(width, text.len() as u16)).unwrap();
    terminal
        .draw(move |frame| {
            frame.render_widget(Paragraph::new(text.clone()), frame.area());
        })
        .unwrap();
    let buffer = terminal.backend().buffer();
    let columns: usize = buffer.area.width as usize;
    assert_eq!(
        columns, width as usize,
        "the backend drew the width it was given"
    );
    buffer
        .content()
        .chunks(columns)
        .map(|row| row.iter().map(|cell| cell.symbol().to_string()).collect())
        .collect()
}

/// Check the rows of one case against `layout`: its line width, its blocks, and
/// the block's two base numbers as D006 half-open ranges.
fn check_blocks(case: &BlocksCase, opened: &[(String, ViewRecord)]) {
    let record: &ViewRecord = lookup(opened, &case.record);
    let mode: LaneMode = mode_for(record, &case.mode);
    let laid: Layout = layout(record, case.width, &mode).unwrap_or_else(|problem| {
        panic!("{} {} {}: {problem:?}", case.record, case.width, case.mode)
    });

    // The gutter is the one rule of §5.1 the tables cannot pin: they hold no
    // record shorter than 10 bases, so `max(2, digits)` is never exercised
    // here beyond the digits themselves. `sequence_view::layout`'s own tests
    // hold the minimum.
    let digits: u64 = record.sequence.length().to_string().chars().count() as u64;
    let gutter: u64 = digits.max(2);
    assert_eq!(
        u64::from(laid.gutter),
        gutter,
        "{} {} {}: the gutter holds the record length's {} digits",
        case.record,
        case.width,
        case.mode,
        record.sequence.length()
    );

    assert_eq!(
        laid.line_width, case.line_width,
        "{} {} {}: the line width the table records on every row",
        case.record, case.width, case.mode
    );
    assert_eq!(
        laid.blocks.len(),
        case.rows.len(),
        "{} {} {}: the layout has {} blocks, the table has rows for {}",
        case.record,
        case.width,
        case.mode,
        laid.blocks.len(),
        case.rows.len()
    );
    for (index, row) in case.rows.iter().enumerate() {
        assert_eq!(
            row.block, index,
            "{} {} {}: the table's block column counts from 0 in order",
            case.record, case.width, case.mode
        );
        assert_eq!(
            row.last_base - row.first_base + 1,
            row.bases,
            "{} {} {} block {}: the table's own base count and its two base numbers",
            case.record,
            case.width,
            case.mode,
            index
        );
        // The table's base numbers are 1-based and inclusive (D055); the
        // layout's range is zero-based and half-open (D006).
        assert_eq!(
            laid.blocks[index],
            (row.first_base - 1, row.last_base),
            "{} {} {} block {}",
            case.record,
            case.width,
            case.mode,
            index
        );
    }

    // The tiling the table shows, checked from its own rows rather than from
    // the split: every base of the record is in exactly one block, in order.
    let mut next: u64 = 1;
    for row in case.rows.iter() {
        assert_eq!(
            row.first_base, next,
            "{} {} {} block {}: the blocks tile the record",
            case.record, case.width, case.mode, row.block
        );
        next = row.last_base + 1;
    }
    assert_eq!(
        next - 1,
        record.sequence.length(),
        "{} {} {}: the last block ends at the record's last base",
        case.record,
        case.width,
        case.mode
    );
}

/// The text the view must draw for one `feature` item: the table records the
/// item as its display name and its strand, `"tet (+)"`, which is not what the
/// line shows (the tables pin no glyphs). The line shows the name, an arrow for
/// that strand and the bar, cut to the run's columns (contract 8j rule 3).
fn run_text(value: &str, cells: u64) -> String {
    let (name, marker) = match value.strip_suffix(')') {
        Some(rest) => match rest.rsplit_once(" (") {
            Some((name, marker @ ("+" | "-"))) => (name, marker),
            _ => panic!("the cells table records a feature item as {{name}} (+) or (-): {value:?}"),
        },
        None => panic!("the cells table records a feature item as {{name}} (+) or (-): {value:?}"),
    };
    let arrow: char = if marker == "+" { '▸' } else { '◂' };
    let mut text: String = format!("{name} {arrow} ");
    while text.chars().count() < cells as usize {
        text.push('═');
    }
    text.chars().take(cells as usize).collect()
}

/// Check the rows of one block against `block_lines` drawn on a test backend:
/// each row's item at its own column and width, each row's role at its own line
/// index, and no other non-blank cell on the line.
fn check_cells(group: &CellsGroup, opened: &[(String, ViewRecord)]) {
    let record: &ViewRecord = lookup(opened, &group.record);
    let mode: LaneMode = mode_for(record, &group.mode);
    let laid: Layout = layout(record, group.width, &mode).unwrap_or_else(|problem| {
        panic!(
            "{} {} {} block {}: {problem:?}",
            group.record, group.width, group.mode, group.block
        )
    });
    assert!(
        group.block < laid.blocks.len(),
        "{} {} {}: the cells table names block {} of {} blocks",
        group.record,
        group.width,
        group.mode,
        group.block,
        laid.blocks.len()
    );
    let lines = block_lines(record, &laid, &mode, group.block).unwrap_or_else(|problem| {
        panic!(
            "{} {} {} block {}: {problem:?}",
            group.record, group.width, group.mode, group.block
        )
    });

    // The lines the table lists, in the order it lists them, and no others.
    let listed: usize = group.roles.len();
    assert_eq!(
        lines.len(),
        listed,
        "{} {} {} block {}: the renderer draws {} lines, the table lists {}",
        group.record,
        group.width,
        group.mode,
        group.block,
        lines.len(),
        listed
    );
    for (line, role) in group.roles.iter().enumerate() {
        assert_eq!(
            lines[line].0,
            role_for(role),
            "{} {} {} block {} line {}: the role of the line",
            group.record,
            group.width,
            group.mode,
            group.block,
            line
        );
    }

    let width: usize = group.width as usize;
    let rows: Vec<Vec<String>> = drawn(&lines, group.width);
    assert_eq!(rows.len(), listed, "one drawn row per line");

    // Every item the table lists lands exactly where it says, in the line's own
    // role line. A row the renderer cannot place fails here.
    for row in group.rows.iter() {
        let first: usize = row.column as usize;
        let last: usize = first + row.cells as usize;
        assert!(
            last <= width,
            "{} {} {} block {} line {} ({}) {} at column {} covers {} columns, past the line's {}",
            group.record,
            group.width,
            group.mode,
            group.block,
            row.line,
            row.role,
            row.kind,
            row.column,
            row.cells,
            width
        );
        let shown: String = rows[row.line][first..last].concat();
        // A `feature` row's `value` names the item and its strand; the line
        // shows rule 3's bar text for that name and strand, cut to the run.
        // Every other kind's `value` is the text the line shows.
        let expected: String = if row.kind == "feature" {
            run_text(&row.value, row.cells)
        } else {
            row.value.clone()
        };
        assert_eq!(
            shown,
            expected,
            "{} {} {} block {} line {} ({}) {} at column {} covering {} columns",
            group.record,
            group.width,
            group.mode,
            group.block,
            row.line,
            row.role,
            row.kind,
            row.column,
            row.cells
        );
    }

    // Containment: every non-blank cell the renderer drew on that line belongs
    // to an item the table lists for it, so a line has no text the table does
    // not account for, and nothing spills past the item that holds it.
    let mut covered: Vec<Vec<bool>> = vec![vec![false; width]; listed];
    for row in group.rows.iter() {
        for column in (row.column..row.column + row.cells).take(width) {
            covered[row.line][column as usize] = true;
        }
    }
    for (line, shown) in rows.iter().enumerate() {
        for (column, symbol) in shown.iter().enumerate() {
            if symbol == " " || symbol.is_empty() {
                continue;
            }
            assert!(
                covered[line][column],
                "{} {} {} block {} line {}: the renderer drew {symbol:?} at column {}, which no item that table lists covers",
                group.record,
                group.width,
                group.mode,
                group.block,
                line,
                column
            );
        }
    }

    // Rule 2's other half: a line's text never runs past its last item, so no
    // line carries a trailing space. The drawn buffer cannot show that (it pads
    // every cell with a space), so it is checked on the `Line`s themselves: a
    // line is exactly as wide as the last item the table lists for it, and a
    // line the table lists no item for carries no text at all.
    for (line, (_, text)) in lines.iter().enumerate() {
        let end: u64 = group
            .rows
            .iter()
            .filter(|row| row.line == line)
            .map(|row| row.column + row.cells)
            .max()
            .unwrap_or(0);
        let shown: String = text
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert_eq!(
            shown.chars().count() as u64,
            end,
            "{} {} {} block {} line {}: the line's text is {} columns, to its last item's {}",
            group.record,
            group.width,
            group.mode,
            group.block,
            line,
            shown.chars().count(),
            end
        );
    }
}

/// The blocks table grouped by case, in the order the table lists its cases.
struct BlocksCase {
    record: String,
    width: u16,
    mode: String,
    /// The table's `line_width` column, which it repeats on every row of the
    /// case; `check_blocks` reads it from the case rather than from a row.
    line_width: u64,
    rows: Vec<BlocksRow>,
}

fn blocks_cases(rows: &[BlocksRow]) -> Vec<BlocksCase> {
    let mut cases: Vec<BlocksCase> = Vec::new();
    for row in rows {
        match cases.iter_mut().find(|case| {
            case.record == row.record && case.width == row.width && case.mode == row.mode
        }) {
            Some(case) => {
                assert_eq!(
                    case.line_width, row.line_width,
                    "{} {} {}: the table records a different line width on two rows of one case",
                    case.record, case.width, case.mode
                );
                case.rows.push(row.clone());
            }
            None => cases.push(BlocksCase {
                record: row.record.clone(),
                width: row.width,
                mode: row.mode.clone(),
                line_width: row.line_width,
                rows: vec![row.clone()],
            }),
        }
    }
    cases
}

/// The cells table grouped by block, in the order the table lists its blocks.
struct CellsGroup {
    record: String,
    width: u16,
    mode: String,
    block: usize,
    /// Every row of the block.
    rows: Vec<CellsRow>,
    /// The role of each line, in line order.
    roles: Vec<String>,
}

fn cells_groups(rows: &[CellsRow]) -> Vec<CellsGroup> {
    let mut groups: Vec<CellsGroup> = Vec::new();
    for row in rows {
        let group: &mut CellsGroup = match groups.iter_mut().find(|group| {
            group.record == row.record
                && group.width == row.width
                && group.mode == row.mode
                && group.block == row.block
        }) {
            Some(group) => group,
            None => {
                groups.push(CellsGroup {
                    record: row.record.clone(),
                    width: row.width,
                    mode: row.mode.clone(),
                    block: row.block,
                    rows: Vec::new(),
                    roles: Vec::new(),
                });
                groups.last_mut().unwrap()
            }
        };
        group.rows.push(row.clone());
        while group.roles.len() <= row.line {
            // Every line the block has is listed in order, and a line with no
            // room for any text is still listed with its role.
            assert_eq!(
                group.roles.len(),
                row.line,
                "{} {} {} block {}: line {} is listed before line {}",
                row.record,
                row.width,
                row.mode,
                row.block,
                group.roles.len(),
                row.line
            );
            group.roles.push(row.role.clone());
        }
        assert_eq!(
            group.roles[row.line], row.role,
            "{} {} {} block {} line {}: two roles for one line",
            row.record, row.width, row.mode, row.block, row.line
        );
    }
    groups
}

#[test]
fn every_case_splits_into_the_tabled_blocks() {
    let rows: Vec<BlocksRow> = blocks_rows();
    // The tables' own row counts, from `tests/fixtures/view/README.md` (§
    // "Block layout": 645 and 845 data rows). Reading them all is the contract,
    // so the count is asserted rather than assumed.
    assert_eq!(rows.len(), 645, "layout-blocks.tsv's data rows");
    let opened: Vec<(String, ViewRecord)> = records();
    let cases: Vec<BlocksCase> = blocks_cases(&rows);
    assert_eq!(
        cases.len(),
        16,
        "the tables hold 4 records x their widths x their modes"
    );
    for case in cases.iter() {
        check_blocks(case, &opened);
    }
}

#[test]
fn every_tabled_item_is_where_the_table_says() {
    let rows: Vec<CellsRow> = cells_rows();
    assert_eq!(rows.len(), 845, "layout-cells.tsv's data rows");
    let opened: Vec<(String, ViewRecord)> = records();
    let groups: Vec<CellsGroup> = cells_groups(&rows);
    for group in groups.iter() {
        check_cells(group, &opened);
    }

    // The two tables cover the same cases: every case the blocks table splits
    // has its cells checked block by block, and the cells table names no case
    // the blocks table does not split.
    let mut cells_cases: Vec<(String, u16, String)> = groups
        .iter()
        .map(|group| (group.record.clone(), group.width, group.mode.clone()))
        .collect();
    cells_cases.sort();
    cells_cases.dedup();
    let mut blocks_cases: Vec<(String, u16, String)> = blocks_rows()
        .iter()
        .map(|row| (row.record.clone(), row.width, row.mode.clone()))
        .collect();
    blocks_cases.sort();
    blocks_cases.dedup();
    assert_eq!(
        cells_cases, blocks_cases,
        "the cells table names the same cases the blocks table splits"
    );
}
