//! The review screen's drawing (milestone 7a, 7c3): the result's state, its
//! experimental validation, the open screen's lines and that screen's keys, read
//! from the document, the app and the frame's area only. Pure: no I/O, so a test
//! can draw it on a test backend and read the rows back (plan §5.1).
//!
//! `text` builds each screen's lines (7c1, 7c2); the view draws them as they are
//! and adds only the banner, the key line and the scroll position (plan §3 rule
//! 1). Nothing is wrapped, so a primer's row never moves.

use crate::review::app::{App, Screen};
use crate::review::proposal::{approvable, ProposalApp, ProposalScreen};
use crate::review::text;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

/// The smallest screen the review draws on.
pub const MIN_WIDTH: u16 = 80;
/// The smallest height the review draws on.
pub const MIN_HEIGHT: u16 = 24;

/// Draw `app` into `frame`. Below `MIN_WIDTH` by `MIN_HEIGHT`, the screen says
/// how small it is and shows nothing of the document; at that size or larger,
/// rows 0 and 1 are the state and the experimental validation, rows 3 to
/// `height - 2` the open screen's lines, scrolled when they are longer than the
/// body, and the last row that screen's keys. Every value is the document's own
/// (plan §3 rule 1, §3 rule 3).
pub fn draw(frame: &mut Frame, app: &App) {
    let area: Rect = frame.area();

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        let notice: String = format!(
            "the screen is {}x{}; luria review needs at least {}x{}",
            area.width, area.height, MIN_WIDTH, MIN_HEIGHT
        );
        frame.render_widget(Paragraph::new(notice), Rect::new(0, 0, area.width, 1));
        return;
    }

    let document = app.document();
    let banner: Vec<Line<'static>> = vec![
        Line::from(format!("state: {}", document.state)),
        Line::from(format!(
            "experimental validation: {}",
            document.experimental_validation
        )),
    ];
    frame.render_widget(Paragraph::new(banner), Rect::new(0, 0, area.width, 2));

    // The body is rows 3 to `height - 2`, so `height - 4` rows tall.
    let body: Rect = Rect::new(0, 3, area.width, area.height - 4);
    let rows: usize = body.height as usize;
    let lines: Vec<String> = match app.screen() {
        Screen::Overview => text::overview(document, app.selected()),
        Screen::Member(index) => text::member(document, index),
        // 7d draws the sequence context, from the document's primer parts and
        // the template `app` holds.
        Screen::Context(index) => text::context(document, index, app.template()),
        Screen::Provenance => text::provenance(document),
        Screen::Problem => text::problem(document),
    };

    // The app's scroll has no upper bound, so the clamp is here: the last line
    // the body can start at is `n - rows` (7c3 §3.1 rule 5).
    let first: usize = app.scroll().min(lines.len().saturating_sub(rows));
    let shown: Vec<Line<'static>> = lines[first..]
        .iter()
        .map(|line| Line::from(line.clone()))
        .collect();
    // No wrap, so a line longer than the width is cut at the width: a primer's
    // row never moves.
    frame.render_widget(Paragraph::new(shown), body);

    let mut keys: String = match app.screen() {
        Screen::Overview => "q quit  enter open  j/k move  p provenance".to_string(),
        Screen::Member(_) => "q quit  esc back  c context  j/k scroll".to_string(),
        Screen::Context(_) => "q quit  esc back".to_string(),
        Screen::Provenance | Screen::Problem => "q quit  esc back  j/k scroll".to_string(),
    };
    // A problem is never hidden (plan §3 rule 3), so the key that opens it is on
    // the overview's key line whenever the document has one.
    if app.screen() == Screen::Overview
        && (document.error.is_some() || !document.clarifications.is_empty())
    {
        keys.push_str("  e problem");
    }
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
        Paragraph::new(keys),
        Rect::new(0, area.height - 1, area.width, 1),
    );
}

/// Draw `app`' proposal into `frame` (7f2): the same size rule and row layout as
/// `draw` - rows 0 and 1 the banner, row 2 blank, rows 3 to `height - 2` the
/// body, scrolled when it is longer than the body, and the last row the keys -
/// with a banner that says the screen is a proposal and not a result (plan §3
/// rule 3), the proposal's own lines (7e), and the confirmation naming the
/// request file the approval writes (plan §3 rule 4). The key line offers `a
/// approve` only where `a` opens the confirmation. Nothing is added to the
/// document's lines (plan §3 rule 1).
pub fn draw_proposal(frame: &mut Frame, app: &ProposalApp) {
    let area: Rect = frame.area();

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        let notice: String = format!(
            "the screen is {}x{}; luria review needs at least {}x{}",
            area.width, area.height, MIN_WIDTH, MIN_HEIGHT
        );
        frame.render_widget(Paragraph::new(notice), Rect::new(0, 0, area.width, 1));
        return;
    }

    let banner: Vec<Line<'static>> = vec![
        Line::from(format!("proposal: {}", app.document().state)),
        Line::from("nothing has run yet: this is a proposal, not a result"),
    ];
    frame.render_widget(Paragraph::new(banner), Rect::new(0, 0, area.width, 2));

    // The body is rows 3 to `height - 2`, so `height - 4` rows tall, as `draw`.
    let body: Rect = Rect::new(0, 3, area.width, area.height - 4);
    let rows: usize = body.height as usize;
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
    let first: usize = app.scroll().min(lines.len().saturating_sub(rows));
    let shown: Vec<Line<'static>> = lines[first..]
        .iter()
        .map(|line| Line::from(line.clone()))
        .collect();
    frame.render_widget(Paragraph::new(shown), body);

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
        Paragraph::new(keys),
        Rect::new(0, area.height - 1, area.width, 1),
    );
}
