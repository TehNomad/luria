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

use crate::review::app::{Action, App, Key, Screen};
use crate::review::export::taken;
use crate::review::proposal::{ProposalAction, ProposalApp};
use crate::review::view;
use crate::review::Approved;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;

/// The terminal's current size, as the area the apps draw on: `0, 0` and the
/// backend's width and height. `None` when the size cannot be read, in which
/// case the scroll is left as the app gave it.
fn area_of<B: ratatui::backend::Backend>(terminal: &ratatui::Terminal<B>) -> Option<Rect> {
    match terminal.backend().size() {
        Ok(size) => Some(Rect::new(0, 0, size.width, size.height)),
        Err(_) => None,
    }
}

/// The key a crossterm press is, as the apps see it: the same mapping for the
/// result screen and the proposal screen, so both read the same keys. `c` with
/// the control modifier is `ctrl-c` (M7-43); every other key is mapped from its
/// code, with the modifiers ignored.
fn key_of(key: KeyEvent) -> Key {
    if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
        return Key::CtrlC;
    }
    match key.code {
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

/// The write the export screen asks for: the plan the screen shows, the answer of
/// the prompt over it (`true` replaces the names it listed, `false` writes nothing
/// over a name that exists), and the names written in the order written, or the
/// one line saying why nothing was (M7-49).
///
/// The type is named here, because the closure's own form is long, and it is the
/// one `run` calls and `main` gives.
type ExportWrite = dyn FnMut(&App, bool) -> Result<Vec<String>, String>;

/// Open the screen, draw `app` until it asks to quit, and restore the terminal,
/// whatever happens (plan §3 rule 7).
///
/// `export` is called on `Action::Export`, which `app` gives for `w` `o` on the
/// export screen when there is a plan and nothing the export would write exists,
/// and on `Action::ExportReplace`, which `app` gives for `y` on the prompt naming
/// the files that exist (M7-49, M7-58); the `bool` it carries is the answer, so
/// the closure writes nothing over an existing name for `false` and replaces for
/// `true`. Its outcome is the one line the screen shows, and the session
/// continues, so the user reads what was written and goes back by choice.
pub fn run(mut app: App, export: &mut ExportWrite) -> std::io::Result<()> {
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
                // M7-49: before each key on the export screen, the names the
                // export would write that exist are set, so `w` can tell whether
                // a write is asked for, or a prompt naming a taken name. The app
                // stays pure, so this is where the file system is looked at.
                if app.screen() == Screen::Export {
                    let names: Vec<String> = match app.export_plan() {
                        Some(plan) => taken(plan),
                        None => Vec::new(),
                    };
                    app.set_taken(names);
                }
                match app.handle(key_of(key)) {
                    Action::Quit => break,
                    // The write asked for: the outcome is shown on the export
                    // screen, and the loop draws it.
                    Action::Export => app.export_done(export(&app, false)),
                    Action::ExportReplace => app.export_done(export(&app, true)),
                    Action::Continue => {}
                }
                // M7-56: scrolling stops at the bottom, so after every key the
                // scroll is clamped to the largest the open screen's body can
                // start at on this terminal, and the next draw shows it there.
                if let Some(area) = area_of(&terminal) {
                    app.clamp_scroll(view::max_scroll(&app, area));
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
                match app.handle(key_of(key)) {
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
                // M7-56: the same clamp as `run`, so scrolling stops at the
                // bottom on the proposal screen too.
                if let Some(area) = area_of(&terminal) {
                    app.clamp_scroll(view::proposal_max_scroll(&app, area));
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
