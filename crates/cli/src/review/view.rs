//! The review screen's drawing (milestone 7a, 7c3, 7j2): the bordered body, the
//! title badge, the key line, colour, and `?`'s help pop-up, read from the
//! document, the app and the frame's area only. Pure: no I/O, so a test can draw
//! it on a test backend and read the rows back (plan §5.1).
//!
//! `text` builds each screen's lines (7c1, 7c2); the view draws them as they are
//! and adds only the badge, the key line, the styles and the scroll position (plan
//! §3 rule 1): no character of a text line is changed, added or removed. Nothing
//! is wrapped, so a primer's row never moves.
//!
//! At `MIN_WIDTH` by `MIN_HEIGHT` and up, rows `0` to `height - 2` are one
//! rounded block whose left title holds ` luria review `, the badge naming the
//! document's state, and the experimental validation; the body is that block's
//! inner area, `height - 3` rows; and row `height - 1` is the key line, outside
//! the border. `?` draws the help legend as a pop-up over the screen (M7-27).

use crate::review::app::{expandable, App, Screen};
use crate::review::proposal::{approvable, ProposalApp, ProposalScreen};
use crate::review::text;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

/// The smallest screen the review draws on.
pub const MIN_WIDTH: u16 = 80;
/// The smallest height the review draws on.
pub const MIN_HEIGHT: u16 = 24;

/// The label column `text` pads its rows to (`text`'s own `CONTEXT_LABEL`), so a
/// body row is recognised by its first `CONTEXT_LABEL` characters.
const CONTEXT_LABEL: usize = 10;

/// Bold, with no colour given.
fn bold() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

/// Dim, with no colour given.
fn dim() -> Style {
    Style::default().add_modifier(Modifier::DIM)
}

/// Bold red.
fn red_bold() -> Style {
    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
}

/// Yellow, not bold.
fn yellow() -> Style {
    Style::default().fg(Color::Yellow)
}

/// The size rule (plan §3.2): below `MIN_WIDTH` by `MIN_HEIGHT`, row 0 says how
/// small the screen is and nothing of the document is drawn. Returns whether the
/// screen is that small.
pub(crate) fn too_small(frame: &mut Frame, area: Rect) -> bool {
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        let notice: String = format!(
            "the screen is {}x{}; luria review needs at least {}x{}",
            area.width, area.height, MIN_WIDTH, MIN_HEIGHT
        );
        frame.render_widget(Paragraph::new(notice), Rect::new(0, 0, area.width, 1));
        return true;
    }
    false
}

/// The frame's block: rows `0` to `height - 2`, all four sides rounded, with
/// `title` at the left.
pub(crate) fn frame_block(title: Line<'static>) -> Block<'static> {
    Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .title_alignment(Alignment::Left)
}

/// A block's left title: ` luria review `, the badge, then the trailing words.
fn title_line(badge: Span<'static>, tail: Span<'static>) -> Line<'static> {
    Line::from(vec![Span::raw(" luria review "), badge, tail])
}

/// The badge, taken from `state`, the document's own value, never assumed (plan
/// §3 rule 3): the word and the symbol are the state's, and the colour is §2's.
fn state_badge(state: &str) -> Span<'static> {
    match state {
        "verified" => Span::styled(
            " ✓ verified ",
            Style::default().fg(Color::Black).bg(Color::Green),
        ),
        "failed" => Span::styled(
            " ✗ failed ",
            Style::default().fg(Color::White).bg(Color::Red),
        ),
        "needs_clarification" => Span::styled(
            " ? needs clarification ",
            Style::default().fg(Color::Black).bg(Color::Yellow),
        ),
        other => Span::styled(
            format!(" {} ", other),
            Style::default().add_modifier(Modifier::REVERSED),
        ),
    }
}

/// The proposal's badge: the screen is a proposal, not a result (plan §3 rule
/// 3), so no state word is on it.
fn proposal_badge() -> Span<'static> {
    Span::styled(
        " PROPOSAL ",
        Style::default().fg(Color::Black).bg(Color::Cyan),
    )
}

/// The validation words: `not tested at the bench` when the document's
/// `experimental_validation` is `not_performed`, the value itself for any other.
fn validation_words(value: &str) -> Span<'static> {
    if value == "not_performed" {
        Span::raw(" · not tested at the bench ")
    } else {
        Span::raw(format!(" · experimental validation: {} ", value))
    }
}

/// A body row's label, when it is one of the member block's rows (the first
/// `CONTEXT_LABEL` characters, trimmed), as §3's table names them.
fn row_label(line: &str) -> Option<String> {
    let label: String = line
        .chars()
        .take(CONTEXT_LABEL)
        .collect::<String>()
        .trim_end()
        .to_string();
    match label.as_str() {
        "template" | "bottom" | "protein" | "forward" | "reverse" => Some(label),
        _ => None,
    }
}

/// The style of one character of a labelled row's body, by §3's table: the
/// template and bottom rows' lower case (the edited bases) bold yellow, the
/// protein's letters cyan and its `*` bold red, the primer rows' upper case
/// (annealing) green and lower case (not annealing) yellow.
fn letter_style(label: &str, ch: char) -> Style {
    match label {
        "template" | "bottom" => {
            if ch.is_ascii_lowercase() {
                yellow().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }
        }
        "protein" => {
            if ch == '*' {
                red_bold()
            } else if ch.is_ascii_alphabetic() {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            }
        }
        "forward" | "reverse" => {
            if ch.is_ascii_uppercase() {
                Style::default().fg(Color::Green)
            } else if ch.is_ascii_lowercase() {
                yellow()
            } else {
                Style::default()
            }
        }
        _ => Style::default(),
    }
}

/// The `[+N bp …]` marker's columns in `text`, when it holds one: from `[+` to
/// the `]` that closes it.
fn marker_range(text: &str) -> Option<(usize, usize)> {
    let start: usize = text.find("[+")?;
    let end: usize = text[start..].find(']')? + start + 1;
    Some((start, end))
}

/// `text` as spans: one span per run of one style, and the `[+N bp …]` marker
/// bold magenta wherever it is (§3's last row). The text itself is unchanged.
fn runs(text: &str, style_of: &dyn Fn(char) -> Style) -> Vec<Span<'static>> {
    if let Some((start, end)) = marker_range(text) {
        let mut spans: Vec<Span<'static>> = runs(&text[..start], style_of);
        spans.push(Span::styled(
            text[start..end].to_string(),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));
        spans.extend(runs(&text[end..], style_of));
        return spans;
    }
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut run: String = String::new();
    let mut style: Style = Style::default();
    for (index, ch) in text.chars().enumerate() {
        let next: Style = style_of(ch);
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

/// A heading line: the heading word bold, the rest as it is.
fn heading_spans(line: &str, heading: &str) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = vec![Span::styled(heading.to_string(), bold())];
    let rest: String = line
        .chars()
        .skip(heading.chars().count())
        .collect::<String>();
    if !rest.is_empty() {
        spans.push(Span::raw(rest));
    }
    spans
}

/// The columns at which a row's outcome word (`no primer pair`, `no design`)
/// starts, when it has one.
fn outcome_at(line: &str) -> Option<usize> {
    line.find("no primer pair")
        .or_else(|| line.find("no design"))
}

/// The selected overview row: reverse video over the whole row, with the outcome
/// word red on top of it.
fn selected_spans(line: &str) -> Vec<Span<'static>> {
    let reversed: Style = Style::default().add_modifier(Modifier::REVERSED);
    match outcome_at(line) {
        Some(at) => vec![
            Span::styled(line[..at].to_string(), reversed),
            Span::styled(line[at..].to_string(), reversed.fg(Color::Red)),
        ],
        None => vec![Span::styled(line.to_string(), reversed)],
    }
}

/// An overview row that is not selected: as it is, with the outcome word red.
fn outcome_spans(line: &str) -> Vec<Span<'static>> {
    match outcome_at(line) {
        Some(at) => vec![
            Span::raw(line[..at].to_string()),
            Span::styled(line[at..].to_string(), Style::default().fg(Color::Red)),
        ],
        None => vec![Span::raw(line.to_string())],
    }
}

/// The body's lines as styled spans, by §3's table. The text is `text`' own and
/// stays exactly as it is: only styles are added. `screen` is the open result
/// screen; a proposal's lines name no result screen, so it is `None`.
fn styled_lines(lines: &[String], screen: Option<Screen>) -> Vec<Line<'static>> {
    let mut out: Vec<Line<'static>> = Vec::new();
    let mut warnings: bool = false;
    for (index, line) in lines.iter().enumerate() {
        // The warning lines are the ones under the `Warnings` heading, up to the
        // blank line that ends the block.
        warnings = line == "Warnings" || (warnings && !line.is_empty());
        let spans: Vec<Span<'static>> = if screen == Some(Screen::Problem) && index == 0 {
            vec![Span::styled(line.clone(), red_bold())]
        } else if line == "Warnings" {
            vec![Span::styled(line.clone(), bold())]
        } else if warnings && matches!(screen, Some(Screen::Member(_))) {
            vec![Span::styled(line.clone(), yellow())]
        } else if line == "No primer pair could be designed" {
            vec![Span::styled(line.clone(), red_bold())]
        } else if line.starts_with("Template  ") {
            heading_spans(line, "Template")
        } else if line.starts_with("Target    ") {
            heading_spans(line, "Target")
        } else if line.starts_with("Variants (") {
            vec![Span::styled(line.clone(), bold())]
        } else if line.ends_with("sequence 5'→3'") {
            // The primer table's column header.
            vec![Span::styled(line.clone(), dim())]
        } else if line.starts_with("▶ ") {
            selected_spans(line)
        } else if let Some(label) = row_label(line) {
            let label_text: String = line.chars().take(CONTEXT_LABEL).collect::<String>();
            let body: String = line.chars().skip(CONTEXT_LABEL).collect::<String>();
            let mut spans: Vec<Span<'static>> = vec![Span::styled(label_text, dim())];
            spans.extend(runs(&body, &|ch| letter_style(&label, ch)));
            spans
        } else if line.starts_with("  ") {
            outcome_spans(line)
        } else {
            runs(line, &|_ch| Style::default())
        };
        out.push(Line::from(spans));
    }
    out
}

/// The key line as spans: `keys` split on the two spaces between its items, each
/// key letter (the text up to the item's first space) bold, the rest dim. The
/// key text itself is unchanged.
pub(crate) fn key_line(keys: &str) -> Line<'static> {
    let dim: Style = Style::default().add_modifier(Modifier::DIM);
    let mut spans: Vec<Span<'static>> = Vec::new();
    for (index, item) in keys.split("  ").enumerate() {
        if index > 0 {
            spans.push(Span::styled("  ", dim));
        }
        match item.find(' ') {
            Some(at) => {
                spans.push(Span::styled(
                    item[..at].to_string(),
                    dim.add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::styled(item[at..].to_string(), dim));
            }
            None => spans.push(Span::styled(
                item.to_string(),
                dim.add_modifier(Modifier::BOLD),
            )),
        }
    }
    Line::from(spans)
}

/// `?`'s help legend as a pop-up over the screen (M7-27, plan §5.3b): centred,
/// `Clear` under it so nothing of the screen shows through, a rounded block
/// titled ` Help `, sized to `text::help()`'s longest line + 4 by its line count
/// + 2, clamped to `area`.
fn draw_help(frame: &mut Frame, area: Rect) {
    let lines: Vec<String> = text::help();
    let width: u16 = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0) as u16
        + 4;
    let height: u16 = lines.len() as u16 + 2;
    let width: u16 = width.min(area.width);
    let height: u16 = height.min(area.height);
    let pop: Rect = Rect::new(
        (area.width - width) / 2,
        (area.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, pop);
    let block: Block<'static> = frame_block(Line::from(" Help "));
    let inner: Rect = block.inner(pop);
    frame.render_widget(block, pop);
    let shown: Vec<Line<'static>> = lines.iter().map(|line| Line::from(line.clone())).collect();
    frame.render_widget(Paragraph::new(shown), inner);
}

/// Draw `app` into `frame`. Below `MIN_WIDTH` by `MIN_HEIGHT`, the screen says
/// how small it is and shows nothing of the document; at that size or larger,
/// rows `0` to `height - 2` are the rounded frame, whose left title holds the
/// badge built from the document's own `state` and its `experimental_validation`,
/// the body is the frame's inner area (`height - 3` rows), scrolled when the
/// open screen's lines are longer than the body, and row `height - 1` is the key
/// line. While `?` is open, the help legend is drawn as a pop-up over the body.
/// Every value is the document's own (plan §3 rule 1, §3 rule 3).
pub fn draw(frame: &mut Frame, app: &App) {
    let area: Rect = frame.area();

    if too_small(frame, area) {
        return;
    }

    let document = app.document();
    let screen_area: Rect = Rect::new(0, 0, area.width, area.height - 1);
    let block: Block<'static> = frame_block(title_line(
        state_badge(document.state),
        validation_words(document.experimental_validation),
    ));
    let inner: Rect = block.inner(screen_area);
    frame.render_widget(block, screen_area);

    // The body is the frame's inner area, so `height - 3` rows: two border rows
    // and the key row.
    let lines: Vec<String> = match app.screen() {
        Screen::Overview => text::overview(document, app.selected()),
        // 7i4 (plan §5.3a): `enter` on a member opens its sequence screen, 7i3's
        // block with the short primer lines under it, expanded when `i` asked for
        // the insertion to be written out.
        Screen::Member(index) => text::member_screen(
            document,
            index,
            app.template(),
            app.frame(index),
            app.expanded(),
        ),
        // 7i4, M7-30: the details (7c2's member text) are behind `d`.
        Screen::Details(index) => text::member(document, index),
        Screen::Provenance => text::provenance(document),
        Screen::Problem => text::problem(document),
        // 7g1: the export screen's lines are the plan's names, the table's
        // refusal when there is one, and the line the last export gave.
        Screen::Export => text::export(app.export_plan(), app.export_message()),
    };

    // The app's scroll has no upper bound, so the clamp is here: the last line
    // the body can start at is `n - rows` (7c3 §3.1 rule 5).
    let rows: usize = inner.height as usize;
    let first: usize = app.scroll().min(lines.len().saturating_sub(rows));
    let shown: Vec<Line<'static>> = styled_lines(&lines[first..], Some(app.screen()));
    // No wrap, so a line longer than the body's width is cut at that width: a
    // primer's row never moves.
    frame.render_widget(Paragraph::new(shown), inner);

    let mut keys: String = if app.help() {
        // M7-27: with the legend open, the only keys are `q` and the one that
        // closes it.
        "q quit  esc close".to_string()
    } else {
        let mut keys: String = match app.screen() {
            Screen::Overview => "q quit  enter open  j/k move  p provenance  x export".to_string(),
            Screen::Member(index) => {
                // `i` is offered only where `i` expands, that is on a member whose
                // design writes more than the inline limit, and it names the state the
                // key brings about, so `expand` while collapsed and `collapse` while
                // expanded (7i4).
                let mut keys: String = "q quit  esc back  d details  j/k scroll".to_string();
                if expandable(document, index) {
                    keys.push_str(if app.expanded() {
                        "  i collapse"
                    } else {
                        "  i expand"
                    });
                }
                keys
            }
            Screen::Details(_) => "q quit  esc back  j/k scroll".to_string(),
            Screen::Provenance | Screen::Problem => "q quit  esc back  j/k scroll".to_string(),
            // `y` is offered only where `y` writes, that is where a plan exists, so
            // the key line never offers an action that does nothing (7g1).
            Screen::Export => {
                let mut keys: String = "q quit".to_string();
                if app.export_plan().is_some() {
                    keys.push_str("  y write");
                }
                keys.push_str("  n back");
                keys
            }
        };
        // A problem is never hidden (plan §3 rule 3), so the key that opens it is on
        // the overview's key line whenever the document has one.
        if app.screen() == Screen::Overview
            && (document.error.is_some() || !document.clarifications.is_empty())
        {
            keys.push_str("  e problem");
        }
        // M7-27: `?` opens the legend on every result screen bar `Export`, where it
        // does nothing, so it is offered only where it works.
        if app.screen() != Screen::Export {
            keys.push_str("  ? help");
        }
        keys
    };
    if lines.len() > rows {
        // Where the body sits in the screen's lines, 1-based and inclusive.
        keys.push_str(&format!(
            "  lines {}-{} of {}",
            first + 1,
            first + rows,
            lines.len()
        ));
    }
    frame.render_widget(
        Paragraph::new(key_line(&keys)),
        Rect::new(0, area.height - 1, area.width, 1),
    );

    if app.help() {
        // The screen is drawn as usual first, so the pop-up goes over it (M7-27).
        draw_help(frame, area);
    }
}

/// Draw `app`' proposal into `frame` (7f2, 7j2): the same size rule and frame as
/// `draw` - rows `0` to `height - 2` the rounded block, its title the badge
/// saying the screen is a proposal and not a result (plan §3 rule 3), the body
/// the block's inner area, scrolled when it is longer than the body, and row
/// `height - 1` the keys - with the proposal's own lines (7e) and the
/// confirmation naming the request file the approval writes (plan §3 rule 4). The
/// key line offers `a approve` only where `a` opens the confirmation. Nothing is
/// added to the document's lines (plan §3 rule 1).
pub fn draw_proposal(frame: &mut Frame, app: &ProposalApp) {
    let area: Rect = frame.area();

    if too_small(frame, area) {
        return;
    }

    let screen_area: Rect = Rect::new(0, 0, area.width, area.height - 1);
    let block: Block<'static> = frame_block(title_line(
        proposal_badge(),
        Span::raw(" · nothing has run yet "),
    ));
    let inner: Rect = block.inner(screen_area);
    frame.render_widget(block, screen_area);

    let lines: Vec<String> = match app.screen() {
        ProposalScreen::Reading => {
            let mut lines: Vec<String> = Vec::new();
            if let Some(message) = app.failure() {
                lines.push(format!("approval failed: {}", message));
                lines.push(String::new());
            }
            lines.extend(text::proposal(app.document()));
            lines
        }
        ProposalScreen::Confirm => vec![
            format!(
                "write the request to {} and run luria q5 design on it?",
                app.request().display()
            ),
            String::new(),
            "y yes  n no".to_string(),
        ],
    };

    // The same clamp as `draw`: the last line the body can start at.
    let rows: usize = inner.height as usize;
    let first: usize = app.scroll().min(lines.len().saturating_sub(rows));
    let shown: Vec<Line<'static>> = styled_lines(&lines[first..], None);
    frame.render_widget(Paragraph::new(shown), inner);

    let mut keys: String = match app.screen() {
        ProposalScreen::Reading => {
            let mut keys: String = "q quit  j/k scroll".to_string();
            // `a` is offered only where `a` opens the confirmation (7f2 §3.3).
            if approvable(app.document()) {
                keys.push_str("  a approve");
            }
            keys
        }
        ProposalScreen::Confirm => "q quit  y yes  n no".to_string(),
    };
    if lines.len() > rows {
        keys.push_str(&format!(
            "  lines {}-{} of {}",
            first + 1,
            first + rows,
            lines.len()
        ));
    }
    frame.render_widget(
        Paragraph::new(key_line(&keys)),
        Rect::new(0, area.height - 1, area.width, 1),
    );
}
