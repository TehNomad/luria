//! The terminal session of `luria review` (milestone 7a): the only file in the
//! review module that does I/O (plan §5.1). It opens the screen, draws `app`
//! until the app asks to quit, and restores the terminal whatever happens: on
//! quit, on an error, and on a panic, because `try_init` installs ratatui's
//! panic hook that does that (plan §3 rule 7). There is nothing here beyond the
//! session and the event loop, and no unit test: the state and the drawing are
//! tested in `app` and `view`.

use crate::review::app::{Action, App, Key};
use crate::review::view;
use crossterm::event::{read, Event, KeyCode, KeyEventKind};

/// Open the screen, draw `app` until it asks to quit, and restore the terminal,
/// whatever happens (plan §3 rule 7).
pub fn run(mut app: App) -> std::io::Result<()> {
    let mut terminal = match ratatui::try_init() {
        Ok(opened) => opened,
        Err(problem) => {
            // `restore` reports its own failures on stderr and returns nothing,
            // so the error this returns is `try_init`'s.
            ratatui::restore();
            return Err(problem);
        }
    };

    let mut looped: std::io::Result<()> = Ok(());
    loop {
        if let Err(problem) = terminal.draw(|frame| view::draw(frame, &app)) {
            looped = Err(problem);
            break;
        }
        match read() {
            Err(problem) => {
                looped = Err(problem);
                break;
            }
            Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                let key: Key = match key.code {
                    KeyCode::Char(c) => Key::Char(c),
                    KeyCode::Up => Key::Up,
                    KeyCode::Down => Key::Down,
                    KeyCode::Enter => Key::Enter,
                    KeyCode::Backspace => Key::Backspace,
                    KeyCode::Esc => Key::Esc,
                    _ => Key::Other,
                };
                if app.handle(key) == Action::Quit {
                    break;
                }
            }
            // A resize, a mouse event, a release: the next draw pass redraws.
            Ok(_) => {}
        }
    }

    let restored: std::io::Result<()> = ratatui::try_restore();
    match looped {
        Err(problem) => Err(problem),
        Ok(()) => restored,
    }
}
