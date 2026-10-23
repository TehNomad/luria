//! The terminal session of `luria new` (milestone 7m4, contract 7m4 §2.3): the
//! only file in the `new` module that does I/O, the same session, key mapping and
//! restore rules as `review::terminal::run_proposal` (plan §3 rule 7). It draws
//! the form, answers `FormAction::Load` by reading the file the form typed
//! relative to the current directory, lists the directory typed in `File` once
//! per directory with `picker::list` before each draw (M7-50), passes a bracketed
//! paste to `NewForm::paste`, and answers `Save` the form gives at once: `plan`
//! names the files it writes, `approve_new` writes them, and the design runs from
//! the request file it wrote, so quitting before the save writes nothing (7f2's
//! rule). When a name is already there the pop-up asks, and only `y`
//! replaces (M7-49).
//! With words typed (7k7, M7-40) the same session draws the proposal screen:
//! `interpret` is called once per `[ Save and run ]`, and a reading with no
//! questions is written and run at once, `review::approve_replacing` at the path
//! the `Save as` name gives, asked to replace only when the user answered `y`
//! (M7-62, M7-64). A reading with questions, an error, a refusal, or nothing to
//! save is shown on the proposal screen and writes nothing; `q` or `esc` goes
//! back to the form with its values kept, and `ctrl-c` ends the session (M7-43).
//! The proposal's own `a`/`y` confirmation is not used in `luria new`, and the
//! form opens no save from the proposal screen: the reading's document is kept,
//! so the `Replace` pop-up's `y` writes it. There is no
//! unit test here, as there is none in `review/terminal`: the form, the pop-up
//! and the drawing are tested in `new/form`, `new/text`, `new/view` and
//! `new/confirm`.

use crate::interpret::document::InterpretationDocument;
use crate::new::confirm::{approve_new, plan, taken, Planned};
use crate::new::form::{Field, FormAction, NewForm, Popup, Source};
use crate::new::picker;
use crate::new::view::draw_form;
use crate::new::view::draw_popup;
use crate::new::view::draw_waiting;
use crate::request::ProfileDocument;
use crate::review::app::Key;
use crate::review::approve_replacing;
use crate::review::proposal::{self, ProposalAction, ProposalApp};
use crate::review::text::proposal as proposal_text;
use crate::review::view;
use crate::review::Approved;

use crossterm::event::{
    read, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers,
};
use crossterm::execute;

use std::fs;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

/// The key a crossterm press is, as the apps see it: `review::terminal`'s own
/// mapping, private there, copied so the form and the confirmation read the same
/// keys the review screens do. `c` with the control modifier is `ctrl-c` (M7-43);
/// every other key is mapped from its code, with the modifiers ignored.
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

/// The request path `plan` gives for the save pop-up's name: beside the template
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

/// The words save of `[ Save and run ]`: `approve_replacing` at the path the
/// `Save as` name gives, exactly as `luria review --template` runs it (7f2),
/// asked to replace only when the user answered `y` (M7-49), with the reading the
/// result screens show at the bottom of Details (M7-64). The request it wrote is
/// returned, since that is the file the design ran as.
fn save_words(
    form: &NewForm,
    cwd: &Path,
    words: &(InterpretationDocument, PathBuf),
    replace: bool,
) -> Result<(PathBuf, Approved, Vec<String>), String> {
    let request: PathBuf = request_path(form, cwd);
    let (document, template) = words;
    match approve_replacing(document, template, &request, replace) {
        Ok(found) => Ok((request, found, proposal_text(document))),
        Err(problem) => Err(problem),
    }
}

/// A path's file name, as the pop-up names a file whose name is already taken
/// (M7-49). `confirm`'s own rule, kept here for the words case, which builds no
/// `Planned`.
fn name_of(path: &Path) -> String {
    match path.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => path.to_string_lossy().into_owned(),
    }
}

/// One terminal session on the form: draw the form, and the pop-up that asks
/// before a file is replaced, until the user quits, which gives `Ok(None)` and
/// has written and run nothing, or an approval succeeds, which gives
/// `Ok(Some(approved))`, and restore the terminal whatever happens (plan
/// §3 rule 7). `written` is the request file the approval wrote, `None` when none
/// was, so the shell can name it in the export plan (7g1). `reading` is the
/// model's reading, the request as written, shown at the bottom of each variant's
/// Details on the result screens (M7-64): `Some` after a words save, `None` after
/// a built save, since only words have a reading to show.
///
/// `interpret` is called only on `FormAction::Interpret`, which the form gives
/// when the words box holds text with a GenBank template loaded (M7-40): the
/// template path as typed, the words as typed, the design the form shows
/// (M7-41), and the CDS the form picked (M7-59), which the closure gives
/// `interpret_with_form`. A document with no questions is written and run at
/// once (M7-62), the request at the path the `Save as` name gives; a name already
/// there is named, and only `y` replaces (M7-49). A document with questions is
/// drawn as the proposal, and writes nothing; `q` or `esc` goes back to the form,
/// and `ctrl-c` ends the session (M7-43). The loop opens no save from the
/// proposal, so the proposal's own `a` is not used in `luria new`. The
/// loop is the owner's hand test.
pub fn run_new(
    form: NewForm,
    cwd: &Path,
    written: &mut Option<PathBuf>,
    reading: &mut Option<Vec<String>>,
    interpret: &mut dyn FnMut(&str, &str, ProfileDocument, &str) -> InterpretationDocument,
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
    // The proposal screen, drawn for a reading with questions, which writes
    // nothing, and the words reading kept for the save: the document, and the
    // template path the words were typed with, which the approval reads (M7-40).
    let mut proposal: Option<ProposalApp> = None;
    let mut words: Option<(InterpretationDocument, PathBuf)> = None;
    let mut approved: Option<Approved> = None;
    let mut looped: std::io::Result<()> = Ok(());

    loop {
        // M7-50: the file list is read once, before the draw that shows it, and
        // only for the directory the form typed, so a directory is listed once
        // and the proposal's screen asks for no listing.
        if proposal.is_none() && form.focus() == Field::File {
            let dir: String = form.file_dir();
            if form.listing_dir() != Some(dir.clone()) {
                let listed = picker::list(&cwd.join(if dir.is_empty() { "." } else { &dir }));
                form.set_listing(dir, listed);
            }
        }
        // M7-49: the proposal and the form are each drawn with the pop-up over
        // them, and `draw_popup` draws nothing when no pop-up is open.
        let drawing: std::io::Result<()> = match &proposal {
            Some(app) => terminal
                .draw(|frame| {
                    view::draw_proposal(frame, app);
                    draw_popup(frame, &form);
                })
                .map(|_| ()),
            None => terminal.draw(|frame| draw_form(frame, &form)).map(|_| ()),
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
                // A paste is the form's own: the proposal takes no text, and with
                // a pop-up open its keys go to the pop-up, so a paste types
                // nothing.
                if proposal.is_none() && form.popup() == Popup::Closed {
                    form.paste(&text);
                }
            }
            Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                let pressed: Key = key_of(key);
                if form.popup() != Popup::Closed || proposal.is_some() {
                    if form.popup() != Popup::Closed {
                        // M7-49, M7-62: with a pop-up open every key goes to the
                        // form, which holds it, so `ctrl-c` quits first, the
                        // replace prompt asks, and a refused write says so on
                        // `SaveAs`.
                        match form.handle(pressed) {
                            FormAction::Quit => break,
                            FormAction::SaveWords { replace } => {
                                // The words path builds no `Built`, so the request
                                // is the path the approval writes, and `y` gives
                                // the save `[ Save and run ]` kept (M7-62).
                                if let Some(kept) = words.as_ref() {
                                    match save_words(&form, cwd, kept, replace) {
                                        // The request the approval wrote is the
                                        // file it ran, so the session ends, and the
                                        // reading goes to the result screens (M7-64).
                                        Ok((request, saved, read)) => {
                                            *written = Some(request);
                                            *reading = Some(read);
                                            approved = Some(saved);
                                            break;
                                        }
                                        // M7-63: the pop-up closes, and the line
                                        // the write gave is `SaveAs`'s blocker.
                                        Err(message) => form.save_failed(message),
                                    }
                                }
                            }
                            // `esc` and any other key closed the pop-up, so the
                            // screen under it is drawn again with the values kept.
                            _ => {}
                        }
                    } else if pressed == Key::CtrlC {
                        // `ctrl-c` ends the session on every screen (M7-43).
                        break;
                    } else if let Some(app) = proposal.as_mut() {
                        // M7-62: a reading with questions writes nothing, so `q`
                        // or `esc` goes back to the form, whose values are kept,
                        // and the loop opens no save from this screen, so `a` is
                        // not used in `luria new`.
                        if app.handle(pressed) == ProposalAction::Quit {
                            proposal = None;
                        }
                    }
                } else {
                    match form.handle(pressed) {
                        FormAction::Quit => break,
                        FormAction::Load(path) => {
                            let read: Result<String, String> = load(&path, cwd);
                            form.loaded(&path, read);
                        }
                        FormAction::Save { built, replace } => {
                            // M7-63: the first `Enter` gives the save, so the
                            // write happens at once, with no pop-up before it.
                            let directory: Option<PathBuf> = template_dir(&form, cwd);
                            let planned: Planned =
                                plan(*built, directory.as_deref(), cwd, &form.save_as());
                            if !replace {
                                let names: Vec<String> = taken(&planned);
                                if !names.is_empty() {
                                    // M7-49: a name already there is named, and `y`
                                    // is the only key that replaces it.
                                    form.ask_replace(names);
                                    continue;
                                }
                            }
                            match approve_new(&planned, replace) {
                                Ok(found) => {
                                    *written = Some(planned.request.clone());
                                    approved = Some(found);
                                    break;
                                }
                                // M7-63: the line the write gave is `SaveAs`'s
                                // blocker, and no pop-up is open.
                                Err(message) => form.save_failed(message),
                            }
                        }
                        FormAction::Interpret {
                            template,
                            text,
                            profile,
                            cds,
                        } => {
                            // The call can take minutes, so the form is drawn once
                            // with the line that says the model is being asked, and
                            // this screen answers when the reply comes back.
                            let _ = terminal.draw(|frame| draw_waiting(frame, &form));
                            // The model is spoken to once per `[ Save and run ]`,
                            // and the request is the path the approval will write.
                            let document: InterpretationDocument =
                                interpret(&template, &text, profile, &cds);
                            if proposal::approvable(&document) {
                                // M7-62, M7-64: a reading with no questions is
                                // written and run at once, so it is kept for the
                                // save, for the reading shown on the results, and
                                // for the `y` when the name is already there.
                                words = Some((document, PathBuf::from(&template)));
                                let request: PathBuf = request_path(&form, cwd);
                                if fs::symlink_metadata(&request).is_ok() {
                                    // M7-49: the name is already there, so the
                                    // pop-up asks, and nothing is written yet.
                                    form.ask_replace(vec![name_of(&request)]);
                                    continue;
                                }
                                if let Some(kept) = words.as_ref() {
                                    match save_words(&form, cwd, kept, false) {
                                        Ok((written_path, saved, read)) => {
                                            *written = Some(written_path);
                                            *reading = Some(read);
                                            approved = Some(saved);
                                            break;
                                        }
                                        Err(message) => form.save_failed(message),
                                    }
                                }
                            } else {
                                // M7-62: a reading with questions, an error, a
                                // refusal, or nothing to save writes nothing, so
                                // the proposal screen opens as it did, and `q` or
                                // `esc` goes back to the form.
                                proposal =
                                    Some(ProposalApp::new(document, request_path(&form, cwd)));
                            }
                        }
                        _ => {}
                    }
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
