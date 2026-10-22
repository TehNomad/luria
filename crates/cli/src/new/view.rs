//! The `luria new` form's screen (milestone 7k4, contract 7k4): 7j2's rounded
//! frame, `text::form_lines` as its body, the ` NEW DESIGN ` badge, colour, the
//! scroll position and the cursor. Pure: no I/O, so a test can draw it on a test
//! backend and read the rows back, as every review screen is drawn (plan §5.1).
//!
//! # The rules this module applies (contract §3)
//!
//! - below 80x24, the review's notice, and nothing of the form is drawn;
//! - rows `0` to `height - 2` are the rounded frame, whose left title is
//!   ` luria new `, the badge ` NEW DESIGN ` (black on cyan, as `PROPOSAL`),
//!   then ` · nothing is written until you approve `; row `height - 1` is the
//!   key line; the body is the frame's inner area;
//! - the body scrolls so the focused field's line is visible, with no wrap, so
//!   a row never moves;
//! - no character of a `text` line is changed, added or removed: only styles
//!   are added (plan §3 rule 1);
//! - labels dim, the focused field's label bold and reversed, `◉` green, `✓`
//!   previews green, `✗` previews red, `needs --backend` dim, `[ Review ]` bold
//!   and reversed when focused, the blocker line bold red;
//! - the cursor is `frame.set_cursor_position` at the text field's column on its
//!   line, so the terminal shows where a typed character goes;
//! - `draw_waiting` draws the same screen with the key line saying the model is
//!   being asked, and no cursor, so the screen shown while the call waits differs
//!   only in that line.

use crate::new::form::Field;
use crate::new::form::NewForm;
use crate::new::text;
use crate::new::text::FormText;
use crate::review::view::frame_block;
use crate::review::view::key_line;
use crate::review::view::too_small;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

/// The label column `text` pads its rows to, so a row's label is its first
/// `LABEL` characters.
const LABEL: usize = 11;

/// The labels `text` writes, as §2.1 lists them. A row is a field's own row
/// exactly when its first `LABEL` characters, trimmed, are one of these: the
/// rows that continue a field start with `LABEL` spaces, so no typed value is
/// read as a label.
const LABELS: [&str; 13] = [
    "Template",
    "  File",
    "  Paste",
    "  Name",
    "  Shape",
    "  CDS",
    "  Bases",
    "  Strand",
    "Mutations",
    "In words",
    "Codon",
    "Overlap",
    "Save as",
];

/// Draw `form` into `frame`: the frame with the ` NEW DESIGN ` badge, the body
/// scrolled so the focused field's line is visible, the key line, and the cursor
/// in the focused text field. Below 80x24 the screen says how small it is and
/// shows nothing of the form.
pub fn draw_form(frame: &mut Frame, form: &NewForm) {
    draw_form_with(
        frame,
        form,
        "tab next  shift-tab back  ←/→ choose  enter review  esc quit",
        true,
    );
}

/// Draw `form` exactly as `draw_form` does, except that the key line says the
/// model is being asked, and no cursor is set: the screen the loop shows while
/// it waits for the reply, which can take minutes (`REQUEST_TIMEOUT_SECONDS`).
pub fn draw_waiting(frame: &mut Frame, form: &NewForm) {
    draw_form_with(
        frame,
        form,
        "asking the model; this screen answers when it replies",
        false,
    );
}

/// The form's screen: the frame with the ` NEW DESIGN ` badge, the body scrolled
/// so the focused field's line is visible, the key line `keys`, and, when
/// `with_cursor` is set, the cursor in the focused text field. Below 80x24 the
/// screen says how small it is and shows nothing of the form.
fn draw_form_with(frame: &mut Frame, form: &NewForm, keys: &str, with_cursor: bool) {
    let area: Rect = frame.area();
    if too_small(frame, area) {
        return;
    }

    let screen_area: Rect = Rect::new(0, 0, area.width, area.height - 1);
    let title: Line<'static> = Line::from(vec![
        Span::raw(" luria new "),
        Span::styled(
            " NEW DESIGN ",
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ),
        Span::raw(" · nothing is written until you approve "),
    ]);
    let block: Block<'static> = frame_block(title);
    let inner: Rect = block.inner(screen_area);
    frame.render_widget(block, screen_area);

    let body: FormText = text::form_lines(form);
    // The body is the frame's inner area, so `height - 3` rows: two border rows
    // and the key row. The scroll puts the focused line in view.
    let rows: usize = inner.height as usize;
    let first: usize = body.focus_line.min(body.lines.len().saturating_sub(rows));
    let label_row: usize = body.focus_line - form.focus_line();
    let mut shown: Vec<Line<'static>> = Vec::new();
    for index in first..body.lines.len() {
        let line: &str = &body.lines[index];
        let focused_label: bool = index == label_row;
        let focused_review: bool = form.focus() == Field::Review && index == body.focus_line;
        shown.push(styled_line(line, focused_label, focused_review));
    }
    frame.render_widget(Paragraph::new(shown), inner);

    frame.render_widget(
        Paragraph::new(key_line(keys)),
        Rect::new(0, area.height - 1, area.width, 1),
    );

    if with_cursor {
        if let Some(column) = body.cursor {
            frame.set_cursor_position(Position::new(
                inner.x + column as u16,
                inner.y + (body.focus_line - first) as u16,
            ));
        }
    }
}

/// One `text` line as spans, with the styles §3 lists: the text itself is
/// unchanged. `focused_label` is the focused field's own row, `focused_review`
/// the `[ Review ]` row when focus is on it.
fn styled_line(line: &str, focused_label: bool, focused_review: bool) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    // The blocker line, which `text` writes at the left margin.
    if line.starts_with('✗') {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
    }
    if line.trim_start() == "[ Review ]" {
        let indent: String =
            line[..line.chars().count() - "[ Review ]".chars().count()].to_string();
        let style: Style = Style::default().add_modifier(Modifier::BOLD);
        if !indent.is_empty() {
            spans.push(Span::raw(indent));
        }
        spans.push(Span::styled(
            "[ Review ]".to_string(),
            if focused_review {
                style.add_modifier(Modifier::REVERSED)
            } else {
                style
            },
        ));
        return Line::from(spans);
    }
    let head: String = line.chars().take(LABEL).collect::<String>();
    let label: Option<&str> = LABELS
        .iter()
        .find(|name| name.trim_end() == head.trim_end())
        .copied();
    if label.is_some() {
        // `head` is the label's `LABEL` characters, `rest` everything after
        // them, so the padding is written once.
        let style: Style = if focused_label {
            Style::default()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED)
        } else {
            Style::default().add_modifier(Modifier::DIM)
        };
        let rest: String = line.chars().skip(LABEL).collect();
        spans.push(Span::styled(head, style));
        if rest.starts_with("needs --backend") {
            spans.push(Span::styled(
                rest,
                Style::default().add_modifier(Modifier::DIM),
            ));
            return Line::from(spans);
        }
        spans.extend(option_runs(&rest, Style::default()));
        return Line::from(spans);
    }
    // The preview lines, indented by the label column: green for a read line,
    // red for a refusal.
    let trimmed: &str = line.trim_start();
    if trimmed.starts_with('✓') {
        let indent: usize = line.chars().count() - trimmed.chars().count();
        spans.push(Span::raw(line[..indent].to_string()));
        spans.push(Span::styled(
            line[indent..].to_string(),
            Style::default().fg(Color::Green),
        ));
        return Line::from(spans);
    }
    if trimmed.starts_with('✗') {
        let indent: usize = line.chars().count() - trimmed.chars().count();
        spans.push(Span::raw(line[..indent].to_string()));
        spans.push(Span::styled(
            line[indent..].to_string(),
            Style::default().fg(Color::Red),
        ));
        return Line::from(spans);
    }
    // The `In words` box's own hint, a continued row written at the label
    // column: dim, as `needs --backend` on a label row is (7k7 §2.3, approved by
    // the owner), and the text itself is unchanged.
    if line.chars().take(LABEL).all(|ch| ch == ' ')
        && trimmed.starts_with("needs a GenBank template")
    {
        let indent: usize = line.chars().count() - trimmed.chars().count();
        spans.push(Span::raw(line[..indent].to_string()));
        spans.push(Span::styled(
            line[indent..].to_string(),
            Style::default().add_modifier(Modifier::DIM),
        ));
        return Line::from(spans);
    }
    Line::from(line.to_string())
}

/// `text` as spans: one span per run of one style, with `◉` green wherever it
/// is; the text itself is unchanged.
fn option_runs(text: &str, base: Style) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut run: String = String::new();
    let mut style: Style = base;
    for (index, ch) in text.chars().enumerate() {
        let next: Style = if ch == '◉' {
            Style::default().fg(Color::Green)
        } else {
            base
        };
        if index > 0 && next != style {
            spans.push(Span::styled(run.clone(), style));
            run.clear();
            style = next;
        }
        run.push(ch);
    }
    if !run.is_empty() {
        spans.push(Span::styled(run, style));
    }
    spans
}
