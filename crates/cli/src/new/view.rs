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
//!   previews green, `✗` previews red, `…` previews and the box's `needs a
//!   GenBank template` hint dim, `[ Save and run ]` bold and reversed when
//!   focused, the blocker line bold red, and the pop-up's `✗` line red bold;
//! - the file list under `File` (M7-50): the highlighted row reversed, a folder
//!   row bold, so a highlighted folder is both, and the `… <n> more` row dim, its
//!   rows told apart from the other continued rows by the position `FormText`
//!   gives in `picker_rows`; with choices showing, the key line is
//!   `↑/↓ pick  enter choose  tab next  esc quit`;
//! - `draw_popup` draws the pop-up over the form, centred in a rounded block
//!   titled ` Save `, as `text::popup_lines` writes it, with the cursor in the
//!   name typed (M7-48); `draw_form` calls it last;
//! - the cursor is `frame.set_cursor_position` at the text field's column on its
//!   line, so the terminal shows where a typed character goes;
//! - `draw_waiting` draws the same screen with the key line saying the model is
//!   being asked, and no cursor, so the screen shown while the call waits differs
//!   only in that line.

use crate::new::form::Field;
use crate::new::form::NewForm;
use crate::new::form::Popup;
use crate::new::text;
use crate::new::text::FormText;
use crate::review::view::frame_block;
use crate::review::view::key_line;
use crate::review::view::too_small;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

/// The label column `text` pads its rows to, so a row's label is its first
/// `LABEL` characters.
const LABEL: usize = 11;

/// The labels `text` writes, as §2.1 lists them. A row is a field's own row
/// exactly when its first `LABEL` characters, trimmed, are one of these: the
/// rows that continue a field start with `LABEL` spaces, so no typed value is
/// read as a label.
const LABELS: [&str; 10] = [
    "Template",
    "  File",
    "  Paste",
    "  Name",
    "  Shape",
    "  CDS",
    "  Bases",
    "  Strand",
    "Mutations",
    "Overlap",
];

/// Draw `form` into `frame`: the frame with the ` NEW DESIGN ` badge, the body
/// scrolled so the focused field's line is visible, the key line, the cursor in
/// the focused text field, and, when a pop-up is open, the pop-up over the form
/// (M7-48). Below 80x24 the screen says how small it is and shows nothing of the
/// form. While `File` has choices, the key line names the picker's keys (M7-50).
pub fn draw_form(frame: &mut Frame, form: &NewForm) {
    let keys: &str = if form.focus() == Field::File && !form.choices().is_empty() {
        "↑/↓ pick  enter choose  tab next  esc quit"
    } else {
        "tab next  shift-tab back  ←/→ choose  enter save  esc quit"
    };
    draw_form_with(frame, form, keys, true);
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
/// so the focused field's line is visible, the key line `keys`, the cursor in the
/// focused text field when `with_cursor` is set, and the pop-up over it when one
/// is open. Below 80x24 the screen says how small it is and shows nothing of the
/// form.
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
        // M7-50: the file list's rows are told apart from the other continued
        // rows by position, which `FormText` states as `picker_rows`.
        let listed: bool = matches!(body.picker_rows, Some((start, count)) if index >= start && index < start + count);
        let trimmed: &str = line.trim_start();
        let picked: bool = listed && trimmed.starts_with("> ");
        let folder: bool = listed && trimmed.ends_with('/');
        shown.push(styled_line(
            line,
            focused_label,
            focused_review,
            picked,
            folder,
        ));
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
    // M7-48: the pop-up is drawn last, over the form's own screen.
    draw_popup(frame, form);
}

/// The pop-up, drawn over the screen `draw_form` has drawn (M7-48, contract
/// §2.4): `Clear`, then a rounded block titled ` Save `, centred, as wide as the
/// longest line plus 4, and at most the screen, with `text::popup_lines` as its
/// body and the cursor in the name on `Request`'s row while the name is typed.
/// Nothing is drawn when no pop-up is open.
pub fn draw_popup(frame: &mut Frame, form: &NewForm) {
    if form.popup() == Popup::Closed {
        return;
    }
    let area: Rect = frame.area();
    let lines: Vec<String> = text::popup_lines(form);
    let longest: usize = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);
    let width: u16 = (longest + 4).min(area.width as usize) as u16;
    let height: u16 = (lines.len() + 2).min(area.height as usize) as u16;
    let pop: Rect = Rect::new(
        (area.width - width) / 2,
        (area.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, pop);
    let title: Line<'static> = Line::from(Span::styled(
        " Save ".to_string(),
        Style::default().fg(Color::Black).bg(Color::Cyan),
    ));
    let block: Block<'static> = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title);
    let inner: Rect = block.inner(pop);
    frame.render_widget(block, pop);
    let shown: Vec<Line<'static>> = lines.iter().map(|line| popup_line(line)).collect();
    frame.render_widget(Paragraph::new(shown), inner);
    // M7-48: the typed name carries the cursor, on `Request`'s row, which is the
    // body's first row.
    if form.popup() == Popup::Naming {
        frame.set_cursor_position(Position::new(
            inner.x + LABEL as u16 + form.save_as_cursor() as u16,
            inner.y,
        ));
    }
}

/// One pop-up line: the failure line red bold, every other line as `text` wrote
/// it, with no character changed (plan §3 rule 1).
fn popup_line(line: &str) -> Line<'static> {
    if line.starts_with('✗') {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
    }
    Line::from(line.to_string())
}

/// The button's label (M7-48), which `text` writes as its own row.
const BUTTON: &str = "[ Save and run ]";

/// One `text` line as spans, with the styles §3 lists: the text itself is
/// unchanged. `focused_label` is the focused field's own row, `focused_review`
/// the `[ Save and run ]` row when focus is on it. `picked` is the highlighted
/// file list row, reversed, and `folder` a folder row, bold, so a highlighted
/// folder is both (M7-50).
fn styled_line(
    line: &str,
    focused_label: bool,
    focused_review: bool,
    picked: bool,
    folder: bool,
) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    // The file list: the highlighted row reversed, a folder row bold, and the
    // `… <n> more` row dim, which the `…` rule below writes.
    if picked {
        let mut style: Style = Style::default().add_modifier(Modifier::REVERSED);
        if folder {
            style = style.add_modifier(Modifier::BOLD);
        }
        return Line::from(Span::styled(line.to_string(), style));
    }
    if folder {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        ));
    }
    // The blocker line, which `text` writes at the left margin.
    if line.starts_with('✗') {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
    }
    if line.trim_start() == BUTTON {
        let indent: String = line[..line.chars().count() - BUTTON.chars().count()].to_string();
        let style: Style = Style::default().add_modifier(Modifier::BOLD);
        if !indent.is_empty() {
            spans.push(Span::raw(indent));
        }
        spans.push(Span::styled(
            BUTTON.to_string(),
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
    // The `Mutations` box's own rows: a preview the model will read (`…`), and
    // the hint that a GenBank template is needed, both written at the label
    // column and drawn dim (M7-47), with the text itself unchanged.
    if line.chars().take(LABEL).all(|ch| ch == ' ')
        && (trimmed.starts_with('…') || trimmed.starts_with("needs a GenBank template"))
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
    // 7m1c: the first character's style is its own, so a row that starts with
    // `◉` draws its mark green, as a `◉` anywhere else on the row does.
    let mut style: Style = match text.chars().next() {
        Some('◉') => Style::default().fg(Color::Green),
        Some(_) => base,
        None => base,
    };
    for ch in text.chars() {
        let next: Style = if ch == '◉' {
            Style::default().fg(Color::Green)
        } else {
            base
        };
        if next != style {
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
