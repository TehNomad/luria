//! The terminal session of `luria new` (milestone 7k5, contract 7k5 §2.4): the
//! only file in the `new` module that does I/O, the same session, key mapping and
//! restore rules as `review::terminal::run_proposal` (plan §3 rule 7). It draws
//! the form, answers `FormAction::Load` by reading the file the form typed
//! relative to the current directory, passes a bracketed paste to
//! `NewForm::paste`, draws the confirmation once `[ Review ]` built, and calls
//! `approve_new` only on `y`: a refusal is the line the confirmation shows, and
//! an approval ends the session with the run it made. Quitting before `y` writes
//! nothing, which is 7f2's rule. With words typed (7k7, M7-40) the same session
//! draws the proposal screen: `interpret` is called once, `review::approve` runs
//! on `a` then `y`, and `q` goes back to the form with its values kept. There is
//! no unit test here, as there is none in `review/terminal`: the form, the
//! confirmation and the drawing are tested in `new/form`, `new/view` and
//! `new/confirm`.

use crate::interpret::document::InterpretationDocument;
use crate::new::confirm::{approve_new, draw_confirm, plan, ConfirmAction, ConfirmApp};
use crate::new::form::{FormAction, NewForm, Source};
use crate::new::view::draw_form;
use crate::review::app::Key;
use crate::review::approve;
use crate::review::proposal::{ProposalAction, ProposalApp};
use crate::review::view::draw_proposal;
use crate::review::Approved;

use crossterm::event::{
    read, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEventKind,
};
use crossterm::execute;

use std::fs::read_to_string;
use std::path::{Path, PathBuf};

/// The key a crossterm press is, as the apps see it: `review::terminal`'s own
/// mapping, private there, copied so the form and the confirmation read the same
/// keys the review screens do.
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

/// The template file the form typed, read as the loop's answer to
/// `FormAction::Load`: `path` is what the `File` field holds, read relative to
/// `cwd`, which is the rule 7k3's doc states for the loop.
fn load(path: &str, cwd: &Path) -> Result<String, String> {
    match read_to_string(cwd.join(path)) {
        Ok(text) => Ok(text),
        Err(problem) => Err(problem.to_string()),
    }
}

/// The directory of the template file the form typed, as the request is written
/// beside it (M7-37), and `None` for a paste, whose FASTA and request both go in
/// `cwd`. `Path::parent` of a bare file name is the empty path, which `cwd.join`
/// gives `cwd` itself.
fn template_dir(form: &NewForm, cwd: &Path) -> Option<PathBuf> {
    match form.source() {
        Source::File => Path::new(&form.file()).parent().map(|dir| cwd.join(dir)),
        Source::Paste => None,
    }
}

/// The request path `plan` gives for the form's `SaveAs`: beside the template
/// file, and in `cwd` for a paste (M7-37). The words path builds no `Built`, so
/// `plan`'s directory rule is applied here, which is the path the approval of a
/// proposal writes (contract §2.4).
fn request_path(form: &NewForm, cwd: &Path) -> PathBuf {
    let dir: PathBuf = match template_dir(form, cwd) {
        Some(found) => found,
        None => cwd.to_path_buf(),
    };
    dir.join(form.save_as())
}

/// One terminal session on the form: draw the form, then the confirmation once
/// `[ Review ]` built, until the user quits, which gives `Ok(None)` and has
/// written and run nothing, or an approval succeeds, which gives
/// `Ok(Some(approved))`, and restore the terminal whatever happens (plan §3 rule
/// 7). `written` is the request file the approval wrote, `None` when none was,
/// so the shell can name it in the export plan (7g1).
///
/// `interpret` is called only on `FormAction::Interpret`, which the form gives
/// when the words box holds text with a GenBank template loaded (M7-40): the
/// template path as typed, and the words as typed. Its document is the proposal
/// drawn in this session, and the approval of it is `review::approve`, exactly as
/// `luria review --template` runs it (7f2). The loop is the owner's hand test.
pub fn run_new(
    form: NewForm,
    cwd: &Path,
    written: &mut Option<PathBuf>,
    interpret: &mut dyn FnMut(&str, &str) -> InterpretationDocument,
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

    // Bracketed paste, so a pasted sequence arrives as one `Event::Paste` and
    // reaches `NewForm::paste` as the text typed (contract §2.4). A terminal
    // that does not support it answers nothing, and a paste then arrives as the
    // keys typed, which the form reads the same way.
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let _ = execute!(handle, EnableBracketedPaste);

    let mut form = form;
    let mut confirm: Option<ConfirmApp> = None;
    // The proposal screen, and the template path the words were typed with, which
    // the approval reads (M7-40).
    let mut proposal: Option<ProposalApp> = None;
    let mut words_template: Option<PathBuf> = None;
    let mut approved: Option<Approved> = None;
    let mut looped: std::io::Result<()> = Ok(());

    loop {
        let drawing: std::io::Result<()> = match (&confirm, &proposal) {
            (Some(app), _) => terminal.draw(|frame| draw_confirm(frame, app)).map(|_| ()),
            (None, Some(app)) => terminal.draw(|frame| draw_proposal(frame, app)).map(|_| ()),
            (None, None) => terminal.draw(|frame| draw_form(frame, &form)).map(|_| ()),
        };
        if let Err(problem) = drawing {
            looped = Err(problem);
            break;
        }

        match read() {
            Err(problem) => {
                looped = Err(problem);
                break;
            }
            Ok(Event::Paste(text)) => {
                // A paste is the form's own: the screens take no text.
                if confirm.is_none() && proposal.is_none() {
                    form.paste(&text);
                }
            }
            Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                let pressed: Key = key_of(key.code);
                let mut back: bool = false;
                match (&mut confirm, &mut proposal) {
                    (Some(app), _) => match app.handle(pressed) {
                        ConfirmAction::Quit => break,
                        ConfirmAction::Approve => match approve_new(app.planned()) {
                            Ok(found) => {
                                *written = Some(app.planned().request.clone());
                                approved = Some(found);
                                break;
                            }
                            // The screen goes back to reading with the line the
                            // approval gave, as 7f2's does, and the loop draws it.
                            Err(message) => app.failed(message),
                        },
                        ConfirmAction::Back => back = true,
                        ConfirmAction::Continue => {}
                    },
                    (None, Some(app)) => match app.handle(pressed) {
                        // `q` or `esc` goes back to the form, whose values are
                        // kept, so the model is not called again until the next
                        // `[ Review ]` (contract §2.4).
                        ProposalAction::Quit => proposal = None,
                        ProposalAction::Approve => {
                            let template: PathBuf = words_template
                                .clone()
                                .expect("a proposal came from words with its template");
                            match approve(app.document(), &template, app.request()) {
                                // The request the approval wrote is the file it
                                // ran, so the loop ends as 7k5's approval does.
                                Ok(found) => {
                                    *written = Some(app.request().to_path_buf());
                                    approved = Some(found);
                                    break;
                                }
                                Err(message) => app.approval_failed(message),
                            }
                        }
                        ProposalAction::Continue => {}
                    },
                    (None, None) => match form.handle(pressed) {
                        FormAction::Quit => break,
                        FormAction::Load(path) => {
                            let read: Result<String, String> = load(&path, cwd);
                            form.loaded(&path, read);
                        }
                        FormAction::Review(built) => {
                            let directory: Option<PathBuf> = template_dir(&form, cwd);
                            confirm = Some(ConfirmApp::new(plan(
                                *built,
                                directory.as_deref(),
                                cwd,
                                &form.save_as(),
                            )));
                        }
                        FormAction::Interpret { template, text } => {
                            // The model is spoken to once per `[ Review ]`, and
                            // the request is the path the approval will write.
                            let document: InterpretationDocument = interpret(&template, &text);
                            words_template = Some(PathBuf::from(&template));
                            proposal = Some(ProposalApp::new(document, request_path(&form, cwd)));
                        }
                        FormAction::Continue => {}
                    },
                }
                // `Back` leaves the confirmation for the form, whose values are
                // kept, so the borrow above is over first.
                if back {
                    confirm = None;
                }
            }
            // A resize, a mouse event, a release: the next draw pass redraws.
            Ok(_) => {}
        }
    }

    let _ = execute!(handle, DisableBracketedPaste);
    drop(handle);

    let restored: std::io::Result<()> = ratatui::try_restore();
    match looped {
        Err(problem) => Err(problem),
        Ok(()) => restored.map(|()| approved),
    }
}
