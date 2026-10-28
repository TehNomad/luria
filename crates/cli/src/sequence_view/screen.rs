//! The view's drawing: the frame with the title in its top border, the blocks,
//! the status line and the key line, and nothing else (contract 8k1 §4 rule 1;
//! plan §5.1, proposal §2-§3).
//!
//! The frame is `review::view::frame_block` and the key line is
//! `review::view::key_line`, the same screen shape the review draws
//! (`review::view`'s module doc, `docs/tasks/milestone-8b3-review-screen.md`):
//! rows `0` to `height - 2` are one rounded block whose title sits in its top
//! border, its inner area is the body, the status line takes the block's last
//! inner row, and row `height - 1` is the keys, outside the border. Blocks are
//! not fenced by a border of their own: one blank row separates them, which is
//! the gap `ViewState::visible` counts, so the rows the state measures are the
//! rows drawn here and a row chosen for the cursor is never a separator.
//!
//! Nothing here works out where a base sits: `super::layout` places a base `p`
//! of block `[s, e)` at line column `gutter + 1 + (p - s)` and `super::state`
//! says which block is on top, which row of it carries the bases and which cells
//! the cursor and its codon mark. The only geometry done here is turning a line
//! column into a screen column by adding the frame's left border.
//!
//! Below `MIN_WIDTH` by `MIN_HEIGHT` nothing is drawn but the notice on row 0
//! (§4 rules 1 and 10). There is no second scroll bar, no rule between blocks,
//! no gutter glyph and no help line: 8k1's screen is the title, the
//! blocks, the status line, the keys, the cursor and its codon. While a list is
//! open — 8k3's `f` and `c` — the same frame, the same status line and the same
//! key line hold the list's rows instead of the blocks, and nothing is drawn of
//! the record; 8m3's key list, which `?` opens, takes the body the same way (§4
//! rule 2). While the overview strip is drawn — 8l2's `o`, 8m12's third line —
//! its three lines take the frame's first three inner rows and the blocks have
//! what is left.

use crate::review::view::frame_block;
use crate::review::view::key_line;
use crate::review::view::MIN_HEIGHT;
use crate::review::view::MIN_WIDTH;
use crate::sequence_view::layout::Layout;
use crate::sequence_view::layout::LineRole;
use crate::sequence_view::list::Drawn;
use crate::sequence_view::overview::overview_drawn;
use crate::sequence_view::state::CursorCodon;
use crate::sequence_view::state::ViewState;
use crate::sequence_view::state::GAP;
use crate::sequence_view::state::OVERVIEW_ROWS;
use luria_core::search::HitStrand;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

/// Draw the whole view: the frame with the title in its top border, the blocks
/// the state has put on screen, the status line on the frame's last inner row
/// and the key line below the frame (§4 rule 1). At a size the view does not
/// draw at, the notice on row 0 and nothing else (§4 rule 10).
pub fn draw(frame: &mut Frame, state: &ViewState) {
    let area: Rect = frame.area();
    if too_small(frame, area) {
        return;
    }

    // The frame takes every row but the key line, which sits below it as it
    // does on the review screen, so its inner area is `height - 3` rows.
    let screen_area: Rect = Rect::new(0, 0, area.width, area.height - 1);
    // M8-34 rule 2: the frame's border is the view's secondary text and dim, its
    // title in that border is not.
    let block: Block<'static> = frame_block(Line::from(format!(" {} ", state.title())))
        .border_style(Style::default().add_modifier(Modifier::DIM));
    let inner: Rect = block.inner(screen_area);
    frame.render_widget(block, screen_area);

    // The overview's three rows, when they are drawn, sit under the top border,
    // and the blocks have the frame's inner rows less those three and the one the
    // status line takes, which is `height - 7` rows: what `ViewState::visible`
    // measures a run of blocks against (8l2 §4 rule 2, 8m12 §4 rule 4). While a
    // list is open the strip is not drawn and the list keeps the `height - 4` rows
    // 8k3 gives it.
    let rows: u16 = if state.overview() && state.list().is_none() && !state.help() {
        OVERVIEW_ROWS
    } else {
        0
    };
    let body: Rect = Rect {
        x: inner.x,
        y: inner.y + rows,
        width: inner.width,
        height: inner.height - 1 - rows,
    };
    if rows > 0 {
        draw_overview(frame, state, inner);
    }
    draw_body(frame, state, body);

    frame.render_widget(
        Paragraph::new(state.status()),
        Rect::new(inner.x, inner.bottom() - 1, inner.width, 1),
    );
    frame.render_widget(
        Paragraph::new(key_line(&state.keys())),
        Rect::new(0, area.height - 1, area.width, 1),
    );
}

/// The size rule, in this screen's words (contract §2: the review's `too_small`
/// names `luria review`, so the sentence is written here). Below the minimum the
/// view draws nothing of the record — no frame, no block, no status line — and
/// row 0 says the size it was given and the size it needs.
fn too_small(frame: &mut Frame, area: Rect) -> bool {
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        let notice: String = format!(
            "the screen is {}x{}; luria's sequence view needs at least {}x{}",
            area.width, area.height, MIN_WIDTH, MIN_HEIGHT
        );
        frame.render_widget(Paragraph::new(notice), Rect::new(0, 0, area.width, 1));
        return true;
    }
    false
}

/// The strip's three lines, on the frame's first three inner rows (8l2 §4 rule 2,
/// 8m12 §4 rules 3 and 4), as `super::overview` writes them. Nothing is added to
/// them here: the strip is display only, with no cursor cell and nothing that jumps
/// (proposal §2.1); its bars carry the same colours the lanes do (M8-31 §4 rule 5),
/// its marks line carries nothing, and its GC line is uncoloured (8m12 §4 rule 3).
/// The bases it shades are the first base of the first block `ViewState::visible`
/// chose through the last base of the last one, and the hits it dots are the starts
/// of the hits shown now (8l1 §4 rules 3 and 4). The third line is the one the
/// state keeps, so no bases are counted while drawing (8m12 §4 rule 5). Without a
/// layout there is nothing to say, and the rows stay blank.
fn draw_overview(frame: &mut Frame, state: &ViewState, inner: Rect) {
    let built: &Layout = match state.layout() {
        Some(built) => built,
        None => return,
    };
    let drawn: std::ops::Range<usize> = state.visible();
    let shown: (u64, u64) = match (
        built.blocks.get(drawn.start),
        drawn
            .end
            .checked_sub(1)
            .and_then(|last| built.blocks.get(last)),
    ) {
        // A block's range is half-open (D006), so the last base on screen is one
        // before the last block's end.
        (Some((first, _)), Some((_, last))) if *first < *last => (*first, *last - 1),
        _ => return,
    };
    let hits: Vec<u64> = match state.hits() {
        Some(hits) => hits.shown().iter().map(|hit| hit.start).collect(),
        None => Vec::new(),
    };
    let (bars, marks): (Line<'static>, String) = overview_drawn(
        state.record(),
        inner.width,
        shown,
        state.cursor(),
        &hits,
        state.colours(), // M8-34 rule 5: what is coloured, is coloured
    );
    frame.render_widget(
        Paragraph::new(bars),
        Rect::new(inner.x, inner.y, inner.width, 1),
    );
    frame.render_widget(
        Paragraph::new(marks),
        Rect::new(inner.x, inner.y + 1, inner.width, 1),
    );
    // The third line, made of the cells around the kept GC line: two blank cells
    // under `1 `, the line's own cells under the strip's, and ` GC` where the
    // length would be, all clipped to the frame's inner width (8m12 §4 rule 3).
    let gc: String = format!("  {} GC", state.gc_line());
    let clipped: String = gc.chars().take(usize::from(inner.width)).collect();
    frame.render_widget(
        Paragraph::new(clipped),
        Rect::new(inner.x, inner.y + 2, inner.width, 1),
    );
}

/// What the drawing marks on a block's bases row: the cursor's base, and the
/// codon the cursor stands in when the selected CDS has one (§4 rule 12). Read
/// once per frame, and the same for every block drawn.
struct Marks {
    cursor: u64,
    codon: Option<CursorCodon>,
}

/// The rows one block holds its bases on, as screen rows: `top` the row of the
/// bases themselves, `bottom` the row of their complements — `None` when a
/// block's lines hold no bottom strand (M8-24) — and `residues` the row of the
/// selected CDS's residue letters, `None` when the block has none (M8-32). All
/// three are found by role in the block's lines, because `layout` decides the
/// order of those lines, not the drawing.
struct Rows {
    top: u16,
    bottom: Option<u16>,
    residues: Option<u16>,
}

/// What the body holds: the open list's rows, 8m3's key list, or the record's
/// blocks (8k3 §4 rule 7, 8m3 §4 rule 2). Each list is drawn in place of the blocks
/// and not over them, so the cursor and its codon mark nothing while a list is
/// open — the bases they mark are not on screen.
fn draw_body(frame: &mut Frame, state: &ViewState, body: Rect) {
    match state.list() {
        Some(_list) => draw_list(frame, state, body),
        None if state.help() => draw_help(frame, state, body),
        None => draw_blocks(frame, state, body),
    }
}

/// The run of blocks `ViewState::visible` chose, from its first block, each one
/// drawn as it comes from `block_lines` with one blank row between blocks, and
/// that row holds the dotted rule M8-34 rule 3 writes into it. A block that does
/// not fit in `body` stops the run: what is drawn is only what fits in the area
/// given, and the gap is never left after the last block drawn.
fn draw_blocks(frame: &mut Frame, state: &ViewState, body: Rect) {
    let (gutter, line_width): (u16, u64) = match state.layout() {
        Some(built) => (built.gutter, built.line_width),
        None => return,
    };
    let marks: Marks = Marks {
        cursor: state.cursor(),
        codon: state.cursor_codon(),
    };
    let rows: usize = usize::from(body.height);
    let mut used: usize = 0;
    let mut drawn: usize = 0;
    for block in state.visible() {
        let lines: Vec<(LineRole, Line<'static>)> = state.lines(block);
        let need: usize = lines.len();
        // A block whose lines `layout` refuses has no rows to draw, and the run
        // goes on without it: `line_count` measured it as none too.
        if need == 0 {
            continue;
        }
        let gap: usize = usize::from(GAP) * usize::from(drawn > 0);
        if used + gap + need > rows {
            break;
        }
        // The row that separates this block from the one before it is the body's
        // own blank row, and it carries nothing but the rule (M8-34 rule 3): no
        // row is added or removed, and the first block has no rule above it.
        if gap > 0 {
            for row in used..used + gap {
                draw_rule(frame, body, gutter, line_width, row);
            }
        }
        used += gap;
        // A run longer than the frame counts in `u16` is not drawn: inventing a
        // height would put a later block over the status line.
        let height: u16 = match u16::try_from(need) {
            Ok(height) => height,
            Err(_error) => break,
        };
        let y: u16 = match u16::try_from(used) {
            Ok(y) => body.y + y,
            Err(_error) => break,
        };
        let text: Vec<Line<'static>> = lines.iter().map(|(_role, line)| line.clone()).collect();
        frame.render_widget(
            Paragraph::new(text),
            Rect {
                x: body.x,
                y,
                width: body.width,
                height,
            },
        );
        // The row the block's bases stand on, and the row their complements
        // stand on, wherever in the block those rows fall: `layout` decides the
        // order of a block's lines, and the drawing follows it rather than
        // assuming the bases come first. A hit is marked on the row its own
        // strand names (M8-24), so the bottom row is found the same way the top
        // row is, by its role.
        let top: Option<u16> = lines
            .iter()
            .position(|(role, _line)| *role == LineRole::Top)
            .and_then(|at| u16::try_from(at).ok())
            .map(|at| y + at);
        let bottom: Option<u16> = lines
            .iter()
            .position(|(role, _line)| *role == LineRole::Bottom)
            .and_then(|at| u16::try_from(at).ok())
            .map(|at| y + at);
        // The selected CDS's residue line, the only one a block has, sits
        // wherever `layout` puts it (M8-32): a block with no residue line — any
        // mode but `Cds` — has nothing to mark there.
        let residues: Option<u16> = lines
            .iter()
            .position(|(role, _line)| *role == LineRole::Residues)
            .and_then(|at| u16::try_from(at).ok())
            .map(|at| y + at);
        if let Some(top) = top {
            mark_bases(
                frame,
                Rows {
                    top,
                    bottom,
                    residues,
                },
                body,
                gutter,
                block,
                state,
                &marks,
            );
        }
        used += need;
        drawn += 1;
    }
}

/// The open list's rows, one per body row from the body's first row down (§4
/// rule 7): the window `ViewState::list_rows` chose, which always holds the
/// highlighted row. The highlighted row is REVERSED and a row `enter` would
/// refuse is DIM, across the cells its text fills; a row can be both, and a row
/// that is neither is drawn with the screen's default style. Nothing here works
/// out what a row says or which rows are drawn: `list` gives the text and the
/// two marks, and `state` gives the window.
fn draw_list(frame: &mut Frame, state: &ViewState, body: Rect) {
    let rows: Vec<Drawn> = state.list_rows();
    for (row, drawn) in rows.iter().enumerate() {
        let y: u16 = match u16::try_from(row) {
            // The rows are counted against the body's height already; a row that
            // would fall past it is not drawn, rather than drawn over the status
            // line.
            Ok(y) if body.y + y < body.bottom() => body.y + y,
            _ => break,
        };
        frame.render_widget(
            Paragraph::new(drawn.text.clone()),
            Rect {
                x: body.x,
                y,
                width: body.width,
                height: 1,
            },
        );
        if !drawn.dim && !drawn.highlighted {
            continue;
        }
        let mut modifier: Modifier = Modifier::empty();
        if drawn.dim {
            modifier |= Modifier::DIM;
        }
        if drawn.highlighted {
            modifier |= Modifier::REVERSED;
        }
        let width: usize = drawn.text.chars().count().min(usize::from(body.width));
        for column in 0..width {
            let x: u16 = match u16::try_from(usize::from(body.x) + column) {
                Ok(x) if x < body.right() => x,
                _ => break,
            };
            frame.buffer_mut()[(x, y)].modifier |= modifier;
        }
    }
}

/// The key list's rows, one per body row from the body's first row down as
/// `ViewState::help_rows` writes them (8m3 §4 rule 2). Nothing is marked: the list
/// is text only, with no highlighted row and no row `enter` acts on. A row that
/// does not fit the body is not drawn, rather than drawn over the status line, and
/// the body's rows hold every row the list has at the minimum size.
fn draw_help(frame: &mut Frame, state: &ViewState, body: Rect) {
    for (row, text) in state.help_rows().iter().enumerate() {
        let y: u16 = match u16::try_from(row) {
            Ok(y) if body.y + y < body.bottom() => body.y + y,
            _ => break,
        };
        frame.render_widget(
            Paragraph::new(text.clone()),
            Rect {
                x: body.x,
                y,
                width: body.width,
                height: 1,
            },
        );
    }
}

/// The one blank row between two blocks, filled with the dim, uncoloured `┄` of
/// M8-34 rule 3 across the columns a full line's bases stand in — the inner
/// columns `d + 1` to `d + N` — and nowhere else in the row: the gutter, the one
/// column after the last base and whatever the frame does not draw stay blank.
/// A shortened last line still has the rule spanning the width a full line has,
/// because the columns are the layout's and not the block's.
fn draw_rule(frame: &mut Frame, body: Rect, gutter: u16, line_width: u64, row: usize) {
    let y: u16 = match u16::try_from(row) {
        // The gap row is inside the body already: `ViewState::visible` counted it.
        Ok(y) if body.y + y < body.bottom() => body.y + y,
        _ => return,
    };
    let first: u64 = u64::from(body.x) + u64::from(gutter) + 1;
    // The last base a full line holds, the column the rule ends in.
    let last: u64 = first + line_width - 1;
    for column in first..=last {
        let x: u16 = match u16::try_from(column) {
            // A column the frame does not draw holds nothing of the rule.
            Ok(x) if x < body.right() => x,
            _ => break,
        };
        frame.buffer_mut()[(x, y)].set_symbol("┄");
        frame.buffer_mut()[(x, y)].modifier |= Modifier::DIM;
    }
}

/// The cursor's cell and the cells of its codon, on the row that carries a
/// block's bases (§4 rule 12), the cells of the hits the search marked (8l1 §4
/// rule 8, M8-24), the same on the row of a block's complements, and the codon's
/// three cells on the selected CDS's residue row, reversed and bold (M8-32 §4
/// rule 2). A base `p`
/// of block `[s, e)` stands at line column
/// `gutter + 1 + (p - s)` — `super::layout` puts it there — and the only geometry
/// done here is moving that line column to the screen by the frame's left border
/// and the block's place in the body. Only the cells on screen are marked: a
/// codon the frame cuts off on the right has the cells it shows marked and the
/// rest of it nowhere.
///
/// The cursor is reversed on the base it stands on, never on its complement, and
/// the whole codon is underlined rather than only the cursor's cell. A cursor
/// with no selected CDS, and a CDS whose codon the cursor is not in, underline
/// nothing. A hit's cells go yellow behind all of that: every base of every hit
/// shown, both runs of a hit whose span crosses the origin, and the cursor's
/// reversal and the codon's underline stay as 8k1 draws them over the yellow.
///
/// Which row a hit's yellow goes on is the hit's strand, and `rows` holds those
/// rows: the top row for a `Plus` or `Both` hit, the bottom row for a `Minus` or
/// `Both` one (M8-24). A hit of either strand goes nowhere on a block that has no
/// bottom row; every block `layout` draws has one.
fn mark_bases(
    frame: &mut Frame,
    rows: Rows,
    body: Rect,
    gutter: u16,
    block: usize,
    state: &ViewState,
    marks: &Marks,
) {
    let (start, end): (u64, u64) = match state
        .layout()
        .and_then(|built| built.blocks.get(block).copied())
    {
        Some(pair) => pair,
        None => return,
    };
    // The hits are marked against the record's length, because a hit whose run
    // passes the origin covers bases at both ends of the record (D052, 8l1 §4 rule
    // 8); which of them this block holds is the loop's question.
    let size: u64 = state.record().sequence.length();
    for position in start..end {
        let underlined: bool = marks
            .codon
            .as_ref()
            .is_some_and(|codon| codon.positions.contains(&position));
        let (top_mark, bottom_mark): (bool, bool) = match state.hits() {
            // A covered base is marked on the row the hit was found on, and on
            // both rows of a `Both` hit (M8-24).
            Some(hits) => hits
                .shown()
                .iter()
                .fold((false, false), |(top_row, bottom_row), hit| {
                    if !hit.covers(position, size) {
                        return (top_row, bottom_row);
                    }
                    match hit.strand {
                        HitStrand::Plus => (true, bottom_row),
                        HitStrand::Minus => (top_row, true),
                        HitStrand::Both => (true, true),
                    }
                }),
            None => (false, false),
        };
        if !underlined && !top_mark && !bottom_mark && position != marks.cursor {
            continue;
        }
        let column: u16 =
            match u16::try_from(u64::from(body.x) + u64::from(gutter) + 1 + (position - start)) {
                // A base the frame cuts off on the right has no cell to mark.
                Ok(column) if column < body.right() => column,
                _ => continue,
            };
        if top_mark {
            if state.colours() {
                // The background goes under the modifiers, which is the other way round
                // from what a cursor cell looks like: `REVERSED` swaps the cell's own
                // colours, so a cursor on a hit reads yellow too (8l1 §4 rule 8).
                frame.buffer_mut()[(column, rows.top)].bg = Color::Yellow;
            } else {
                // M8-34 rule 5: with no colour to mark a hit, the mark is bold and
                // underlined, on the row the hit's strand names.
                frame.buffer_mut()[(column, rows.top)].modifier |=
                    Modifier::BOLD | Modifier::UNDERLINED;
            }
        }
        if let (true, Some(bottom)) = (bottom_mark, rows.bottom) {
            if state.colours() {
                frame.buffer_mut()[(column, bottom)].bg = Color::Yellow;
            } else {
                frame.buffer_mut()[(column, bottom)].modifier |=
                    Modifier::BOLD | Modifier::UNDERLINED;
            }
        }
        if underlined {
            // The cell's own modifiers are kept: the underline goes on top of
            // whatever `layout` styled the base with, and the cursor's cell
            // carries both marks (§4 rule 12).
            frame.buffer_mut()[(column, rows.top)].modifier |= Modifier::UNDERLINED;
        }
        if position == marks.cursor {
            frame.buffer_mut()[(column, rows.top)].modifier |= Modifier::REVERSED;
        }
        if let (Some(residues), true) = (rows.residues, underlined) {
            // The codon's three cells on the residue line are reversed and bold
            // as well, one cell per position and never the span between the
            // lowest and the highest, so a codon the origin splits marks its two
            // pieces and nothing between them (M8-32 §4 rule 2). The cell's own
            // letter and colours stay under the mark, and the mark is there with
            // the colours on and off alike (M8-32 §4 rule 3).
            frame.buffer_mut()[(column, residues)].modifier |= Modifier::REVERSED | Modifier::BOLD;
        }
    }
}
