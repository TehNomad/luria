//! The terminal session of `luria review` (milestone 7a, 7f2): the only file in
//! the review module that does I/O (plan §5.1). It opens the screen, draws `app`
//! until the app asks to quit, and restores the terminal whatever happens: on
//! quit, on an error, and on a panic, because `try_init` installs ratatui's
//! panic hook that does that (plan §3 rule 7). There is nothing here beyond the
//! session and the event loop, and no unit test: the state and the drawing are
//! tested in `app`, `proposal` and `view`.
//!
//! `run_proposal` is 7f2's session: it carries out the proposal screen's keys,
//! and runs the approval the user confirms, in the one place the shell decides
//! to write (plan §3 rule 4).

use crate::review::app::{Action, App, Key};
use crate::review::proposal::{ProposalAction, ProposalApp};
use crate::review::view;
use crate::review::Approved;
use crossterm::event::{read, Event, KeyCode, KeyEventKind};

/// The key a crossterm press is, as the apps see it: the same mapping for the
/// result screen and the proposal screen, so both read the same keys.
fn key_of(code: KeyCode) -> Key {
    match code {
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Home => Key::Home,
        KeyCode::Enter => Key::Enter,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Delete => Key::Delete,
        KeyCode::Tab => Key::Tab,
        KeyCode::BackTab => Key::BackTab,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Esc => Key::Esc,
        _ => Key::Other,
    }
}

/// Open the screen, draw `app` until it asks to quit, and restore the terminal,
/// whatever happens (plan §3 rule 7).
///
/// `export` is called only on `Action::Export`, which `app` gives for `y` on the
/// export screen when there is a plan (7g1). Its outcome is the one line the
/// screen shows, and the session continues, so the user reads what was written
/// and goes back by choice.
pub fn run(
    mut app: App,
    export: &mut dyn FnMut(&App) -> Result<Vec<String>, String>,
) -> std::io::Result<()> {
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
                match app.handle(key_of(key.code)) {
                    Action::Quit => break,
                    // `y` asked for the write: the outcome is shown on the
                    // export screen, and the loop draws it.
                    Action::Export => app.export_done(export(&app)),
                    Action::Continue => {}
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

/// Open the screen on `app`' proposal, draw it until the user quits, which gives
/// `Ok(None)` and has written and run nothing, or an approval succeeds, which
/// gives `Ok(Some(approved))`, and restore the terminal whatever happens (plan
/// §3 rule 7) - the same session, key mapping and restore rules as `run`.
///
/// `approve` is called only on `ProposalAction::Approve`, which `app` gives for
/// `y` on the confirmation (plan §3 rule 4). Its `Err` is the one line the
/// reading screen shows, and the user may approve again or quit; its `Ok` ends
/// the session, because the request was written and designed.
pub fn run_proposal(
    mut app: ProposalApp,
    approve: &mut dyn FnMut(&ProposalApp) -> Result<Approved, String>,
) -> std::io::Result<Option<Approved>> {
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
    let mut approved: Option<Approved> = None;

    loop {
        if let Err(problem) = terminal.draw(|frame| view::draw_proposal(frame, &app)) {
            looped = Err(problem);
            break;
        }
        match read() {
            Err(problem) => {
                looped = Err(problem);
                break;
            }
            Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                match app.handle(key_of(key.code)) {
                    ProposalAction::Quit => break,
                    ProposalAction::Approve => match approve(&app) {
                        Ok(given) => {
                            approved = Some(given);
                            break;
                        }
                        // The screen goes back to the proposal with the line the
                        // approval gave, and the loop draws it.
                        Err(message) => app.approval_failed(message),
                    },
                    ProposalAction::Continue => {}
                }
            }
            // A resize, a mouse event, a release: the next draw pass redraws.
            Ok(_) => {}
        }
    }

    let restored: std::io::Result<()> = ratatui::try_restore();
    match looped {
        Err(problem) => Err(problem),
        Ok(()) => restored.map(|()| approved),
    }
}
