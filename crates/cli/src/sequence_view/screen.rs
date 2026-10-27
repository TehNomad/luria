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
//! no gutter glyph, no `?` help and no help line: 8k1's screen is the title, the
//! blocks, the status line, the keys, the cursor and its codon. While a list is
//! open — 8k3's `f` and `c` — the same frame, the same status line and the same
//! key line hold the list's rows instead of the blocks, and nothing is drawn of
//! the record.

use crate::review::view::frame_block;
use crate::review::view::key_line;
use crate::review::view::MIN_HEIGHT;
use crate::review::view::MIN_WIDTH;
use crate::sequence_view::layout::LineRole;
use crate::sequence_view::list::Drawn;
use crate::sequence_view::state::CursorCodon;
use crate::sequence_view::state::ViewState;
use crate::sequence_view::state::GAP;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
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
    let block: Block<'static> = frame_block(Line::from(format!(" {} ", state.title())));
    let inner: Rect = block.inner(screen_area);
    frame.render_widget(block, screen_area);

    // The blocks have the frame's inner rows less the one the status line takes,
    // which is `height - 4` rows: what `ViewState::visible` measures a run of
    // blocks against (§4 rule 1).
    let body: Rect = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height - 1,
    };
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

/// What the drawing marks on a block's bases row: the cursor's base, and the
/// codon the cursor stands in when the selected CDS has one (§4 rule 12). Read
/// once per frame, and the same for every block drawn.
struct Marks {
    cursor: u64,
    codon: Option<CursorCodon>,
}

/// What the body holds: the open list's rows, or the record's blocks (8k3 §4
/// rule 7). A list is drawn in place of the blocks and not over them, so the
/// cursor and its codon mark nothing while a list is open — the bases they mark
/// are not on screen.
fn draw_body(frame: &mut Frame, state: &ViewState, body: Rect) {
    match state.list() {
        Some(_list) => draw_list(frame, state, body),
        None => draw_blocks(frame, state, body),
    }
}

/// The run of blocks `ViewState::visible` chose, from its first block, each one
/// drawn as it comes from `block_lines` with one blank row between blocks. A
/// block that does not fit in `body` stops the run: what is drawn is only what
/// fits in the area given, and the gap is never left after the last block drawn.
fn draw_blocks(frame: &mut Frame, state: &ViewState, body: Rect) {
    let gutter: u16 = match state.layout() {
        Some(built) => built.gutter,
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
        // The row the block's bases stand on, wherever in the block that row
        // falls: `layout` decides the order of a block's lines, and the drawing
        // follows it rather than assuming the bases come first.
        if let Some(top) = lines
            .iter()
            .position(|(role, _line)| *role == LineRole::Top)
        {
            if let Ok(top) = u16::try_from(top) {
                mark_bases(frame, y + top, body, gutter, block, state, &marks);
            }
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

/// The cursor's cell and the cells of its codon, on the row that carries a
/// block's bases (§4 rule 12), and the cells of the hits the search marked (8l1 §4
/// rule 8). A base `p` of block `[s, e)` stands at line column
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
fn mark_bases(
    frame: &mut Frame,
    row: u16,
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
        let marked: bool = match state.hits() {
            Some(hits) => hits.shown().iter().any(|hit| hit.covers(position, size)),
            None => false,
        };
        if !underlined && !marked && position != marks.cursor {
            continue;
        }
        let column: u16 =
            match u16::try_from(u64::from(body.x) + u64::from(gutter) + 1 + (position - start)) {
                // A base the frame cuts off on the right has no cell to mark.
                Ok(column) if column < body.right() => column,
                _ => continue,
            };
        if marked {
            // The background goes under the modifiers, which is the other way round
            // from what a cursor cell looks like: `REVERSED` swaps the cell's own
            // colours, so a cursor on a hit reads yellow too (8l1 §4 rule 8).
            frame.buffer_mut()[(column, row)].bg = Color::Yellow;
        }
        if underlined {
            // The cell's own modifiers are kept: the underline goes on top of
            // whatever `layout` styled the base with, and the cursor's cell
            // carries both marks (§4 rule 12).
            frame.buffer_mut()[(column, row)].modifier |= Modifier::UNDERLINED;
        }
        if position == marks.cursor {
            frame.buffer_mut()[(column, row)].modifier |= Modifier::REVERSED;
        }
    }
}
