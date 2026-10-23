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

/// The rows the body has on an `area`-sized terminal: the frame's inner area's
/// height, that is `height - 3` (two border rows and the key row), and `0` below
/// `MIN_WIDTH` by `MIN_HEIGHT`, where nothing of the document is drawn.
fn body_rows(area: Rect) -> usize {
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        0
    } else {
        frame_block(Line::from(""))
            .inner(Rect::new(0, 0, area.width, area.height - 1))
            .height as usize
    }
}

/// The open result screen's lines, exactly the ones `draw` renders: `text`'s own,
/// by the open screen (plan §3 rule 1). The details (7c2's member text) carry the
/// provenance under them (M7-52, 7i4, M7-30).
fn screen_lines(app: &App) -> Vec<String> {
    let document = app.document();
    match app.screen() {
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
        Screen::Details(index) => match app.reading() {
            // M7-61: in a preview the model's reading sits under the variant's
            // technical lines, at the bottom of the screen `d` opens.
            Some(reading) => text::preview_details(document, index, reading),
            None => text::details(document, index),
        },
        Screen::Problem => text::problem(document),
        // 7g1: the export screen's lines are the plan's names, the table's
        // refusal when there is one, and the line the last export gave.
        Screen::Export => text::export(app.export_plan(), app.export_message()),
        // M7-49: the prompt over the export screen names the files that exist,
        // and the key that replaces them.
        Screen::Replace => replace_lines(app),
    }
}

/// The `Replace` prompt's lines (M7-49): the directory the export writes in, each
/// name that exists in it on its own line, indented as the export screen indents
/// the names it lists, then the key that answers. `taken` is what the shell set
/// before the last key, so nothing here looks at the file system (plan §3 rule 1).
fn replace_lines(app: &App) -> Vec<String> {
    match app.export_plan() {
        None => Vec::new(),
        Some(plan) => {
            let mut lines: Vec<String> = Vec::new();
            lines.push(format!(
                "these files exist in {}:",
                plan.directory.display()
            ));
            for name in app.taken() {
                lines.push(format!("  {}", name));
            }
            lines.push(String::new());
            lines.push("y replace them  any other key: write nothing".to_string());
            lines
        }
    }
}

/// The proposal screen's lines, exactly the ones `draw_proposal` renders (7e).
fn proposal_lines(app: &ProposalApp) -> Vec<String> {
    match app.screen() {
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
    }
}

/// The largest scroll the open screen's body can have on an `area`-sized terminal
/// (M7-56): the last line the body can start at, `lines - rows`, and `0` below
/// the minimum size. `terminal` clamps the app's scroll to this after every key,
/// so `k` moves the body at once after scrolling past the bottom.
pub fn max_scroll(app: &App, area: Rect) -> usize {
    let rows: usize = body_rows(area);
    if rows == 0 {
        0
    } else {
        screen_lines(app).len().saturating_sub(rows)
    }
}

/// The same rule on the proposal screen (M7-56).
pub fn proposal_max_scroll(app: &ProposalApp, area: Rect) -> usize {
    let rows: usize = body_rows(area);
    if rows == 0 {
        0
    } else {
        proposal_lines(app).len().saturating_sub(rows)
    }
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

/// The preview's title words, in place of the experimental validation (M7-60):
/// the design ran in memory and nothing is written yet (D046). The state badge
/// stays in front of them, so the state is still on screen (D020).
fn preview_words() -> Span<'static> {
    Span::raw(" · preview, nothing is saved yet ")
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

/// The style of one character of a labelled row's body, by §2.6's table (M7-54):
/// the template and bottom rows' lower case (the edited bases) bold yellow, the
/// protein's letters magenta and its `*` bold red, the primer rows' upper case
/// (annealing) light blue and lower case (the tail) yellow. Red and green are
/// never the only difference between two things.
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
                Style::default().fg(Color::Magenta)
            } else {
                Style::default()
            }
        }
        "forward" | "reverse" => {
            if ch.is_ascii_uppercase() {
                Style::default().fg(Color::LightBlue)
            } else if ch.is_ascii_lowercase() {
                yellow()
            } else {
                Style::default()
            }
        }
        _ => Style::default(),
    }
}

/// A primer table row (M7-55): `text`'s `primer_table_line`, whose body after the
/// label is `<n> nt`, two spaces, the annealing temperature `{:.1} °C`, three
/// spaces, then the sequence. The byte offsets of the temperature's number and of
/// the sequence, in `body`; `None` when the body is not such a row, that is on a
/// sequence block row, whose label is the same word.
fn primer_table_cells(body: &str) -> Option<(usize, usize, usize)> {
    let lead: usize = body.len() - body.trim_start().len();
    let rest: &str = &body[lead..];
    let digits: usize = rest.chars().take_while(|c| c.is_ascii_digit()).count();
    // A length of no digits is not a table row, and ` nt` must follow the digits.
    if digits == 0 || !rest[digits..].starts_with(" nt") {
        return None;
    }
    let tm_at: usize = body.find(" \u{00b0}C")?;
    let mut start: usize = tm_at;
    while start > 0
        && (body[start - 1..].starts_with('.')
            || body[start - 1..].starts_with(|c: char| c.is_ascii_digit()))
    {
        start -= 1;
    }
    Some((start, tm_at, tm_at + " \u{00b0}C".len() + 3))
}

/// The `[+N bp …]` marker's columns in `text`, when it holds one: from `[+` to
/// the `]` that closes it.
fn marker_range(text: &str) -> Option<(usize, usize)> {
    let start: usize = text.find("[+")?;
    let end: usize = text[start..].find(']')? + start + 1;
    Some((start, end))
}

/// `text` as spans: one span per run of one style, and the `[+N bp …]` marker
/// bold wherever it is (§2.6's last row, M7-54: it takes the protein's colour).
/// The text itself is unchanged.
fn runs(text: &str, style_of: &dyn Fn(char) -> Style) -> Vec<Span<'static>> {
    if let Some((start, end)) = marker_range(text) {
        let mut spans: Vec<Span<'static>> = runs(&text[..start], style_of);
        spans.push(Span::styled(
            text[start..end].to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.extend(runs(&text[end..], style_of));
        return spans;
    }
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut run: String = String::new();
    // 7m1c: the first character's style is its own, so every character is drawn
    // in the style `style_of` gives it, the first one included.
    let mut style: Style = match text.chars().next() {
        Some(first) => style_of(first),
        None => Style::default(),
    };
    for ch in text.chars() {
        let next: Style = style_of(ch);
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
            // M7-55: on a primer table row, `nt` and `°C` take no colour, the
            // annealing temperature's number is bold and its unit plain, and only
            // the sequence after the three spaces is coloured as a primer row.
            match primer_table_cells(&body) {
                Some((tm_start, tm_at, seq_start)) => {
                    spans.push(Span::raw(body[..tm_start].to_string()));
                    spans.push(Span::styled(body[tm_start..tm_at].to_string(), bold()));
                    spans.push(Span::raw(body[tm_at..seq_start].to_string()));
                    spans.extend(runs(&body[seq_start..], &|ch| letter_style(&label, ch)));
                }
                None => spans.extend(runs(&body, &|ch| letter_style(&label, ch))),
            }
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

/// `?`'s help as a pop-up over the screen (M7-27, M7-53): centred, `Clear` under
/// it so nothing of the screen shows through, a rounded block titled ` Help `,
/// sized to `text::help`'s longest line + 4 by its line count + 2, clamped to
/// `area`. The legend is on the variant screen only, and the keys named are the
/// open screen's own.
fn draw_help(frame: &mut Frame, area: Rect, app: &App) {
    let document = app.document();
    let can_expand: bool = match app.screen() {
        Screen::Member(index) => expandable(document, index),
        _ => false,
    };
    let lines: Vec<String> = match app.reading() {
        // A preview's legend names the key its own screens add (M7-60).
        Some(_) => text::preview_help(
            app.screen(),
            document.error.is_some() || !document.clarifications.is_empty(),
            can_expand,
        ),
        None => text::help(
            app.screen(),
            document.error.is_some() || !document.clarifications.is_empty(),
            can_expand,
        ),
    };
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
    // A preview names itself in place of the validation, and keeps the badge
    // (M7-60, D020); without a reading the title is as it was.
    let tail: Span<'static> = match app.reading() {
        Some(_) => preview_words(),
        None => validation_words(document.experimental_validation),
    };
    let block: Block<'static> = frame_block(title_line(state_badge(document.state), tail));
    let inner: Rect = block.inner(screen_area);
    frame.render_widget(block, screen_area);

    // The body is the frame's inner area, so `height - 3` rows: two border rows
    // and the key row.
    let lines: Vec<String> = screen_lines(app);

    // The app's scroll is clamped by `terminal` to `max_scroll`, and the draw
    // clamps it here as well, so the body never starts past the last line
    // (7c3 §3.1 rule 5, M7-56).
    let rows: usize = inner.height as usize;
    let first: usize = app.scroll().min(max_scroll(app, area));
    let shown: Vec<Line<'static>> = styled_lines(&lines[first..], Some(app.screen()));
    // No wrap, so a line longer than the body's width is cut at that width: a
    // primer's row never moves.
    frame.render_widget(Paragraph::new(shown), inner);

    let mut keys: String = if app.help() {
        // M7-27: with the legend open, the only keys are `q` and the one that
        // closes it.
        "q quit  esc close".to_string()
    } else {
        let preview: bool = app.reading().is_some();
        let mut keys: String = match app.screen() {
            Screen::Overview => {
                // A preview has no export, so `x` is not on its key line (M7-60).
                if preview {
                    "q quit  enter open  j/k move".to_string()
                } else {
                    "q quit  enter open  j/k move  x export".to_string()
                }
            }
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
            Screen::Problem => "q quit  esc back  j/k scroll".to_string(),
            // M7-58: `w` is offered only where `w` writes, that is where a plan
            // exists, so the key line never offers an action that does nothing
            // (7g1).
            Screen::Export => {
                let mut keys: String = "q quit".to_string();
                if app.export_plan().is_some() {
                    keys.push_str("  w write");
                }
                keys.push_str("  esc back");
                keys
            }
            // M7-49: the prompt's own keys, `y` answering the question the body
            // asks, `esc` going back to the export screen the prompt sits over.
            Screen::Replace => "q quit  y replace them  esc back".to_string(),
        };
        // A problem is never hidden (plan §3 rule 3), so the key that opens it is on
        // the overview's key line whenever the document has one.
        if app.screen() == Screen::Overview
            && (document.error.is_some() || !document.clarifications.is_empty())
        {
            keys.push_str("  e problem");
        }
        // M7-60: `a` opens the save pop-up on a preview's variant screens, so
        // their key lines name it, and the other screens' do not.
        if preview
            && matches!(
                app.screen(),
                Screen::Overview | Screen::Member(_) | Screen::Details(_)
            )
        {
            keys.push_str("  a save");
        }
        // M7-27, M7-53: `?` opens the legend on every screen, Export included, so
        // it is offered on every key line.
        keys.push_str("  ? help");
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
        draw_help(frame, area, app);
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

    let lines: Vec<String> = proposal_lines(app);

    // The same clamp as `draw`: the last line the body can start at (M7-56).
    let rows: usize = inner.height as usize;
    let first: usize = app.scroll().min(proposal_max_scroll(app, area));
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
