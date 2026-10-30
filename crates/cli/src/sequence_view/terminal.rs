//! The terminal session of `luria FILE` (contract 8k1 §4 rule 14, D048 M8-13,
//! D055): the only file in the view module that does I/O, and the only one that
//! touches a terminal. It opens the screen, draws the view until `q` or `ctrl-c`
//! asks to leave, and restores the terminal whatever happens - on quit, on an
//! error, and on a panic, because `ratatui::try_init` installs ratatui's panic
//! hook that does that (the rule `review::terminal` follows, unchanged here).
//! The session itself has no test, as `review/terminal.rs` has none: the
//! state and the drawing are tested in `state`, `screen` and
//! `tests/sequence_view.rs`. What the test here covers is the environment's
//! colour rule, the only part of this file that answers without a terminal.
//!
//! A view is a running program and not a prompt, because `read` blocks:
//! `luria FILE` needs a terminal and refuses one it does not have (D048 M8-13).
//! Which way `luria FILE` goes - this session or a plain dump - is 8h's
//! dispatch, not this module's, and 8h is the caller this signature is written
//! for.
//!
//! Nothing here writes a file (§8), and nothing here decides what a key means:
//! `super::state::ViewState::handle` does, and this module only names the key
//! that was pressed and passes the answer on. Bracketed paste is switched on
//! and off here as well (9f §3 item 2), since only the terminal knows a paste
//! is one; what a paste means stays `state::paste_text`'s rule.

use crate::sequence_view::screen;
use crate::sequence_view::state::Action;
use crate::sequence_view::state::Key;
use crate::sequence_view::state::ViewState;
use crossterm::event::read;
use crossterm::event::DisableBracketedPaste;
use crossterm::event::EnableBracketedPaste;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use crossterm::execute;
use luria_core::view_record::ViewRecord;
use ratatui::layout::Size;
use std::ffi::OsStr;
use std::io::stdout;

/// Whether the view must open without the residue colours (contract 8m5 §4 rule
/// 5, no-color.org): `NO_COLOR` present and not empty, whatever it is set to,
/// means no colours, and an unset or empty value is the same as not asking.
pub fn no_color(value: Option<&OsStr>) -> bool {
    value.is_some_and(|text: &OsStr| !text.is_empty())
}

/// The key a `crossterm` press is, as the view sees it (contract §4 rule 14, and
/// 8k3 §4 rule 3 for `enter`, `esc` and `backspace`): the
/// same shape as `review::terminal::key_of`. `c` with the control modifier is
/// `ctrl-c`, which quits from anywhere; a printable key is named by the character
/// typed, so `T` and `t` are different keys; the named keys are the ones the
/// plan's list gives; and every other press - a function key, a modifier on its
/// own, a dead key - is `Other` and changes nothing (§4 rule 10, D055's
/// "a key with no rule").
fn key_of(press: KeyEvent) -> Key {
    if press.code == KeyCode::Char('c') && press.modifiers == KeyModifiers::CONTROL {
        return Key::CtrlC;
    }
    // 8m15's two keys, which name themselves only with exactly the control
    // modifier: `d` and `u` on their own are text, and a press with another
    // modifier is `Other` (§4 rule 1).
    if press.code == KeyCode::Char('d') && press.modifiers == KeyModifiers::CONTROL {
        return Key::CtrlD;
    }
    if press.code == KeyCode::Char('u') && press.modifiers == KeyModifiers::CONTROL {
        return Key::CtrlU;
    }
    // 9g's redo key, named the same way: `r` on its own is text, and a press
    // with another modifier is `Other`.
    if press.code == KeyCode::Char('r') && press.modifiers == KeyModifiers::CONTROL {
        return Key::CtrlR;
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
        // 8k3's three keys, which an open list uses; outside one they change
        // nothing, and that rule is `state::handle`'s, not this one.
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Esc,
        KeyCode::Backspace => Key::Backspace,
        // 8l1's key, which the open search prompt uses to flip its mode; outside
        // one it changes nothing, and that rule is `state::handle`'s.
        KeyCode::Tab => Key::Tab,
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
            // comment on the same call, unchanged). Bracketed paste is switched
            // on below, so there is nothing of that to undo at this point.
            ratatui::restore();
            return Err(problem);
        }
    };

    // 9f §3 item 2: a paste arrives as one piece of text only while the terminal
    // is asked to bracket it, which is what makes the `Event::Paste` arm below
    // fire at all. It is switched back off on every way out of `run`, because a
    // leftover `ESC[?2004h` would leave the shell eating the paste markers of
    // every later command.
    if let Err(problem) = execute!(stdout(), EnableBracketedPaste) {
        // The screen is open, so it is restored before the error is handed on.
        ratatui::restore();
        return Err(problem);
    }

    // The panic path. `try_init` installed a hook that restores raw mode and the
    // screen but knows nothing about bracketed paste, so this hook chains the one
    // ratatui installed: paste gets switched off first, then the old hook does
    // the rest of the restore.
    let previous_hook: Box<dyn Fn(&std::panic::PanicHookInfo<'_>) + Sync + Send + 'static> =
        std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(stdout(), DisableBracketedPaste);
        previous_hook(info);
    }));

    // The size the record is laid out at. A terminal that will not say its size
    // is not drawn on at all: the view has no width to lay blocks out in, and
    // the alternative is a screen of made-up columns.
    let size: Size = match terminal.size() {
        Ok(size) => size,
        Err(problem) => {
            let _ = execute!(stdout(), DisableBracketedPaste);
            let _ = ratatui::try_restore();
            return Err(problem);
        }
    };
    let mut state: ViewState = ViewState::new(record, size.width, size.height);
    // The residues are coloured on open unless the environment says not to
    // (8m5 §4 rule 5), which is decided before the first draw: no key asks.
    state.set_colours(!no_color(std::env::var_os("NO_COLOR").as_deref()));

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
            // A bracketed paste: the whole text comes in one event, and every
            // rule about what a paste means is `ViewState::paste_text`'s, not this (9f
            // §3 item 2).
            Ok(Event::Paste(text)) => state.paste_text(&text),
            // A mouse event, a focus change, a poll that found nothing: the next
            // draw pass redraws.
            Ok(_other) => {}
        }
    }

    // Every way out of the loop lands here: quit, an error from `read` and a
    // failed draw. Paste gets switched off before the screen is restored, and
    // each early return above does the same for itself; the hook installed at the
    // open covers a panic, the one way out that is not a return.
    let _ = execute!(stdout(), DisableBracketedPaste);
    let restored: std::io::Result<()> = ratatui::try_restore();
    match looped {
        Err(problem) => Err(problem),
        Ok(()) => restored,
    }
}

#[cfg(test)]
mod tests {
    use super::key_of;
    use super::no_color;
    use super::Key;
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;
    use std::ffi::OsStr;

    /// §5's U2: the three answers the environment gives, as no-color.org has
    /// them. Unset and empty both leave the residues coloured; anything else,
    /// even `0`, asks that nothing be coloured.
    #[test]
    fn no_color_is_true_only_for_a_present_and_not_empty_value() {
        assert!(!no_color(None), "nothing set asks for the colours");
        assert!(
            !no_color(Some(OsStr::new(""))),
            "an empty value asks for the colours"
        );
        assert!(
            no_color(Some(OsStr::new("1"))),
            "`NO_COLOR=1` asks for none"
        );
        assert!(
            no_color(Some(OsStr::new("0"))),
            "`NO_COLOR=0` is set and not empty, so it asks for none too"
        );
    }

    /// 9g §3 item 3: `ctrl` + `r` names the view's redo key, the way `ctrl-c`
    /// is named. A plain `r` is still the replace key.
    #[test]
    fn ctrl_r_is_the_redo_key() {
        let press: KeyEvent = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL);
        assert_eq!(key_of(press), Key::CtrlR, "ctrl-r comes in as redo");
        let plain: KeyEvent = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
        assert_eq!(key_of(plain), Key::Char('r'), "a plain r is replace");
    }
}
