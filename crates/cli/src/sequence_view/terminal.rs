//! The terminal session of `luria FILE` (contract 8k1 §4 rule 14, D048 M8-13,
//! D055): the only file in the view module that does I/O, and the only one that
//! touches a terminal. It opens the screen, draws the view until `q` or `ctrl-c`
//! asks to leave, and restores the terminal whatever happens - on quit, on an
//! error, and on a panic, because `ratatui::try_init` installs ratatui's panic
//! hook that does that (the rule `review::terminal` follows, unchanged here).
//! There is no unit test, as `review/terminal.rs` has none: the state and the
//! drawing are tested in `state`, `screen` and `tests/sequence_view.rs`.
//!
//! A view is a running program and not a prompt, because `read` blocks:
//! `luria FILE` needs a terminal and refuses one it does not have (D048 M8-13).
//! Which way `luria FILE` goes - this session or a plain dump - is 8h's
//! dispatch, not this module's, and 8h is the caller this signature is written
//! for.
//!
//! Nothing here writes a file (§8), and nothing here decides what a key means:
//! `super::state::ViewState::handle` does, and this module only names the key
//! that was pressed and passes the answer on.

use crate::sequence_view::screen;
use crate::sequence_view::state::Action;
use crate::sequence_view::state::Key;
use crate::sequence_view::state::ViewState;
use crossterm::event::read;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use luria_core::view_record::ViewRecord;
use ratatui::layout::Size;

/// The key a `crossterm` press is, as the view sees it (contract §4 rule 14): the
/// same shape as `review::terminal::key_of`. `c` with the control modifier is
/// `ctrl-c`, which quits from anywhere; a printable key is named by the character
/// typed, so `T` and `t` are different keys; the named keys are the ones the
/// plan's list gives; and every other press - a function key, a modifier on its
/// own, `esc`, a dead key - is `Other` and changes nothing (§4 rule 10, D055's
/// "a key with no rule").
fn key_of(press: KeyEvent) -> Key {
    if press.code == KeyCode::Char('c') && press.modifiers == KeyModifiers::CONTROL {
        return Key::CtrlC;
    }
    match press.code {
        KeyCode::Char(letter) => Key::Char(letter),
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        _ => Key::Other,
    }
}

/// Open the view on `record`, draw it until the view asks to leave, and restore
/// the terminal on every way out (D048 M8-13, D055): the same session, key
/// mapping and restore rule as `review::terminal::run`.
///
/// The view is laid out at the size the terminal reports when the session opens,
/// and re-laid out at every `Event::Resize` (§4 rule 14), which is the only way a
/// size change reaches it: no key asks for a size. The record is moved into the
/// view, so a session holds no reference to a caller's borrow.
pub fn run(record: ViewRecord) -> std::io::Result<()> {
    let mut terminal = match ratatui::try_init() {
        Ok(opened) => opened,
        Err(problem) => {
            // `restore` reports its own failures on stderr and returns nothing,
            // so the error this returns is `try_init`'s (`review::terminal`'s
            // comment on the same call, unchanged).
            ratatui::restore();
            return Err(problem);
        }
    };

    // The size the record is laid out at. A terminal that will not say its size
    // is not drawn on at all: the view has no width to lay blocks out in, and
    // the alternative is a screen of made-up columns.
    let size: Size = match terminal.size() {
        Ok(size) => size,
        Err(problem) => {
            let _ = ratatui::try_restore();
            return Err(problem);
        }
    };
    let mut state: ViewState = ViewState::new(record, size.width, size.height);

    let mut looped: std::io::Result<()> = Ok(());
    loop {
        if let Err(problem) = terminal.draw(|frame| screen::draw(frame, &state)) {
            looped = Err(problem);
            break;
        }
        match read() {
            Err(problem) => {
                looped = Err(problem);
                break;
            }
            // Key presses only: a release or a repeat of a press is not a key,
            // and the next draw pass draws the screen again unchanged
            // (`review::terminal`'s rule).
            Ok(Event::Key(press)) if press.kind == KeyEventKind::Press => {
                if state.handle(key_of(press)) == Action::Quit {
                    break;
                }
            }
            // A new size: the blocks are laid out again for it, the top block
            // keeps the base it covered, and the next draw shows the record at
            // the new width (§4 rule 14).
            Ok(Event::Resize(width, height)) => state.resize(width, height),
            // A mouse event, a focus change, a poll that found nothing: the next
            // draw pass redraws.
            Ok(_other) => {}
        }
    }

    let restored: std::io::Result<()> = ratatui::try_restore();
    match looped {
        Err(problem) => Err(problem),
        Ok(()) => restored,
    }
}
