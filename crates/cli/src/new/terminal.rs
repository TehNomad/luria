//! The terminal session of `luria new` (milestone 7m4, contract 7m4 §2.3): the
//! only file in the `new` module that does I/O, the same session, key mapping and
//! restore rules as `review::terminal::run_proposal` (plan §3 rule 7). It draws
//! the form, answers `FormAction::Load` by reading the file the form typed
//! relative to the current directory, lists the directory typed in `File` once
//! per directory with `picker::list` before each draw (M7-50), passes a bracketed
//! paste to `NewForm::paste`, and answers the pop-up `[ Save and run ]` opens: the
//! pop-up's `enter` gives `Save`, `plan` names the files it writes,
//! `approve_new` writes them, and the design runs from the request file it wrote,
//! so nothing is written before `enter` and quitting writes nothing (7f2's rule).
//! When a name is already there the pop-up asks, and only `y` replaces (M7-49).
//! With words typed (7k7, M7-40) the same session draws the proposal screen and
//! the pop-up over it: `interpret` is called once, its approval is the same
//! pop-up, whose `enter` gives `SaveWords` and runs
//! `review::approve_replacing`, `esc` closes the pop-up so the proposal is shown
//! again with the form's values kept, and `ctrl-c` ends the session (M7-43). The
//! proposal's own `a`/`y` confirmation is not used in `luria new`. A reading that
//! can be saved is previewed (M7-60, M7-61): `preview` runs the design in memory,
//! the variant screens are drawn with the pop-up over them, `a` opens the pop-up,
//! whose `enter` gives the same `SaveWords`, and `q`, or `esc` on the overview, goes
//! back to the form with its values kept, so nothing is written before the pop-up is
//! answered. A reading with questions, an error, or nothing to save has no preview,
//! so the proposal screen opens as it did. There is no
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
use crate::review::app::{Action, App, Key};
use crate::review::approve_replacing;
use crate::review::proposal::{self, ProposalAction, ProposalApp};
use crate::review::view;
use crate::review::Approved;
use ratatui::layout::Rect;

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

/// The terminal's current size, as the area the preview draws on: `0, 0` and the
/// backend's width and height, so `view::max_scroll` clamps the preview's scroll
/// the way `review::terminal` clamps the review's (M7-56). `None` when the size
/// cannot be read, in which case the scroll is left as the app gave it. Copied
/// from `review::terminal`, where it is private.
fn area_of<B: ratatui::backend::Backend>(terminal: &ratatui::Terminal<B>) -> Option<Rect> {
    match terminal.backend().size() {
        Ok(size) => Some(Rect::new(0, 0, size.width, size.height)),
        Err(_) => None,
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

/// A path's file name, as the pop-up names a file whose name is already taken
/// (M7-49). `confirm`'s own rule, kept here for the words case, which builds no
/// `Planned`.
fn name_of(path: &Path) -> String {
    match path.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => path.to_string_lossy().into_owned(),
    }
}

/// One terminal session on the form: draw the form, then the pop-up the form
/// opens once `[ Save and run ]` built, until the user quits, which gives
/// `Ok(None)` and has written and run nothing, or an approval succeeds, which
/// gives `Ok(Some(approved))`, and restore the terminal whatever happens (plan
/// §3 rule 7). `written` is the request file the approval wrote, `None` when none
/// was, so the shell can name it in the export plan (7g1).
///
/// `interpret` is called only on `FormAction::Interpret`, which the form gives
/// when the words box holds text with a GenBank template loaded (M7-40): the
/// template path as typed, the words as typed, the design the form shows
/// (M7-41), and the CDS the form picked (M7-59), which the closure gives
/// `interpret_with_form`. Its document is the proposal drawn in this session, and
/// its approval is the pop-up over it, whose
/// `enter` runs `review::approve_replacing`, exactly as `luria review --template`
/// runs it (7f2), asked to replace only when the user answered `y` (M7-49). The
/// loop is the owner's hand test.
pub fn run_new(
    form: NewForm,
    cwd: &Path,
    written: &mut Option<PathBuf>,
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
    // The proposal screen, and the template path the words were typed with, which
    // the approval reads (M7-40).
    let mut proposal: Option<ProposalApp> = None;
    let mut words_template: Option<PathBuf> = None;
    // M7-60, M7-61: the preview of a reading that can be saved, the design run in
    // memory, drawn on the variant screens with the reading at the bottom of
    // Details. The proposal is kept as well, because the save uses its document.
    let mut preview: Option<App> = None;
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
        // M7-48: the proposal is drawn with the pop-up over it, and `draw_popup`
        // draws nothing when no pop-up is open.
        // M7-48, M7-60: the preview, the proposal and the form are each drawn
        // with the pop-up over them, and `draw_popup` draws nothing when no pop-up
        // is open.
        let drawing: std::io::Result<()> = match (&preview, &proposal) {
            (Some(app), _) => terminal
                .draw(|frame| {
                    view::draw(frame, app);
                    draw_popup(frame, &form);
                })
                .map(|_| ()),
            (None, Some(app)) => terminal
                .draw(|frame| {
                    view::draw_proposal(frame, app);
                    draw_popup(frame, &form);
                })
                .map(|_| ()),
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
                // A paste is the form's own: the proposal and its preview take no
                // text, and with a pop-up open its keys go to the pop-up, so a
                // paste types nothing (M7-60).
                if proposal.is_none() && preview.is_none() && form.popup() == Popup::Closed {
                    form.paste(&text);
                }
            }
            Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                let pressed: Key = key_of(key);
                if preview.is_some() || proposal.is_some() {
                    // M7-48, M7-60: with a pop-up open over the preview or over
                    // the proposal every key goes to the form, which holds it, so
                    // the pop-up's `enter`, the replace prompt and a refused write
                    // behave exactly as they do now.
                    if form.popup() != Popup::Closed {
                        match form.handle(pressed) {
                            FormAction::Quit => break,
                            FormAction::SaveWords { replace } => {
                                // The words path builds no `Built`, so the
                                // request is the path the approval writes.
                                let request: PathBuf = request_path(&form, cwd);
                                if !replace && fs::symlink_metadata(&request).is_ok() {
                                    // M7-49: the name is already there, so the
                                    // pop-up asks, and nothing is written yet.
                                    form.ask_replace(vec![name_of(&request)]);
                                    continue;
                                }
                                let template: PathBuf = words_template
                                    .clone()
                                    .expect("a proposal came from words with its template");
                                // The preview is a run in memory, so the save uses
                                // the proposal's document, which is kept for it.
                                let reading: &InterpretationDocument = proposal
                                    .as_ref()
                                    .expect("the save pop-up sits over a proposal")
                                    .document();
                                match approve_replacing(reading, &template, &request, replace) {
                                    // The request the approval wrote is the
                                    // file it ran, so the loop ends.
                                    Ok(found) => {
                                        *written = Some(request);
                                        approved = Some(found);
                                        break;
                                    }
                                    // The pop-up names the request and keeps
                                    // the screen under it, as 7f2's does.
                                    Err(message) => form.save_failed(message),
                                }
                            }
                            // `esc` closed the pop-up, so the screen under it is
                            // drawn again with the form's values kept.
                            _ => {}
                        }
                    } else if pressed == Key::CtrlC {
                        // `ctrl-c` ends the session on every screen (M7-43), the
                        // preview included.
                        break;
                    } else if preview.is_some() {
                        // M7-60: `a` on a preview's variant screens asks for the
                        // save, so it opens the pop-up that names the file, and
                        // `q`, or `esc` on the overview, goes back to the form.
                        let mut quit: bool = false;
                        if let Some(app) = preview.as_mut() {
                            match app.handle(pressed) {
                                Action::Approve => form.open_words_popup(),
                                Action::Quit => quit = true,
                                _ => {}
                            }
                            if !quit {
                                // M7-56: as on the review screens, the scroll stops
                                // at the bottom, so it is clamped after every key.
                                if let Some(area) = area_of(&terminal) {
                                    app.clamp_scroll(view::max_scroll(app, area));
                                }
                            }
                        }
                        if quit {
                            // Back to the form with its values kept, and nothing
                            // written (M7-60), so the model is not called again
                            // until the next `[ Save and run ]`.
                            preview = None;
                            proposal = None;
                        }
                    } else if pressed == Key::Char('a')
                        && proposal::approvable(
                            proposal.as_ref().expect("a proposal is open").document(),
                        )
                    {
                        // M7-48: `a` opens the pop-up that names the file, so
                        // the proposal's own `a`/`y` confirmation is not used
                        // in `luria new`.
                        form.open_words_popup();
                    } else {
                        // `q` or `esc` goes back to the form, whose values
                        // are kept, so the model is not called again until
                        // the next `[ Save and run ]` (contract §2.4).
                        if let Some(app) = proposal.as_mut() {
                            if app.handle(pressed) == ProposalAction::Quit {
                                proposal = None;
                            }
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
                                // The pop-up stays open with the line the write
                                // gave, as 7f2's confirmation showed it.
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
                            words_template = Some(PathBuf::from(&template));
                            let request: PathBuf = request_path(&form, cwd);
                            // M7-60, M7-61: a reading that can be saved is designed
                            // in memory and kept as the preview the variant screens
                            // draw, with the reading itself at the bottom of
                            // Details. A reading with questions, an error, a
                            // refusal, or nothing to save is not previewed, and the
                            // proposal screen opens as it does now.
                            preview = if proposal::approvable(&document) {
                                match crate::review::preview(&document, &request) {
                                    Ok(found) => Some(
                                        App::new(found.document)
                                            .with_template(found.template)
                                            .with_frames(found.frames)
                                            .with_reading(crate::review::text::proposal(&document)),
                                    ),
                                    Err(_problem) => None,
                                }
                            } else {
                                None
                            };
                            proposal = Some(ProposalApp::new(document, request));
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
