//! The review screen's drawing (milestone 7a): the result's state, its
//! experimental validation and the quit key, read from the document and the
//! frame's area only. Pure: no I/O, so a test can draw it on a test backend and
//! read the rows back (plan §5.1).

use crate::review::app::App;
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
/// the state and the experimental validation are the first two rows and the
/// quit key is the last row. Every value is the document's own (plan §3 rule
/// 1, §3 rule 3).
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
    let lines: Vec<Line<'static>> = vec![
        Line::from(format!("state: {}", document.state)),
        Line::from(format!(
            "experimental validation: {}",
            document.experimental_validation
        )),
    ];
    frame.render_widget(Paragraph::new(lines), Rect::new(0, 0, area.width, 2));
    frame.render_widget(
        Paragraph::new("q quit"),
        Rect::new(0, area.height - 1, area.width, 1),
    );
}
