//! The confirmation of `luria new` (milestone 7k5, contract 7k5 §2): the files
//! one approval writes, the two screens the user passes through, the request
//! shown as the file is written, and the write and the run that follow `y`.
//!
//! 7f2's rule is the rule here (plan §3 rule 4): nothing is written before `a`
//! then `y`, quitting before `y` writes nothing, and the design runs from the
//! file the approval wrote, never from the form in memory. The request shown is
//! byte for byte the text `interpret::run::request_text` writes (plan §3 rule
//! 2, the rule 7e's proposal screen keeps), every file is written as a new file
//! that never replaces one (C-1), and the run is `review::run_approved` of that
//! file, so `new` and `review --template` design the same request the same way.
//!
//! Apart from `approve_new` nothing here is I/O: no terminal, no drawing, so the
//! screen's content and its key rule are testable without a terminal (plan
//! §5.1), and `new/terminal` stays the only file in the module that does I/O.

use crate::interpret::run::{request_location_problem, request_text, write_request_file};
use crate::new;
use crate::request::TemplateSpec;
use crate::result::request_echo;
use crate::review;
use crate::review::app::Key;
use crate::review::view::frame_block;
use crate::review::view::key_line;
use crate::review::view::too_small;
use crate::review::Approved;

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

use std::path::{Path, PathBuf};

/// The files one approval writes, and the form they came from.
#[derive(Clone, Debug, PartialEq)]
pub struct Planned {
    /// Where each file goes: for a template file, the request beside it
    /// (M7-37, as `--write-request` requires); for a paste, the pasted
    /// FASTA and the request both in the current directory.
    pub request: PathBuf,
    pub paste: Option<(PathBuf, String)>,
    pub built: new::Built,
}

/// The files a valid form's approval writes: `save_as` is the form's `Save as`
/// field, `template_dir` the directory of a template file, and `None` for a
/// paste, whose FASTA and request both go in `cwd`. The request goes beside the
/// template, as M7-37 and `prepare`'s rule that a request's template path is
/// relative to the request (plan §3.4) require.
pub fn plan(built: new::Built, template_dir: Option<&Path>, cwd: &Path, save_as: &str) -> Planned {
    let dir: &Path = match template_dir {
        Some(found) => found,
        None => cwd,
    };
    Planned {
        request: dir.join(save_as),
        paste: built
            .paste
            .as_ref()
            .map(|(name, text)| (cwd.join(name), text.clone())),
        built,
    }
}

/// Which confirmation screen is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmScreen {
    /// The request as the file will hold it, and the files the approval writes.
    Reading,
    /// The confirm question naming every file, and `y` `n`.
    Confirm,
}

/// What the app asks its shell to do after a key. Only `Approve` writes or runs
/// anything, and only after `y` on the confirmation (plan §3 rule 4); `Back`
/// returns to the form, whose values are kept, and `Quit` writes nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmAction {
    Continue,
    Back,
    Quit,
    Approve,
}

/// The confirmation's state: the files planned, the open screen, how far it is
/// scrolled, and the last approval's failure.
#[derive(Clone, Debug, PartialEq)]
pub struct ConfirmApp {
    planned: Planned,
    screen: ConfirmScreen,
    scroll: usize,
    failure: Option<String>,
}

impl ConfirmApp {
    /// The screen opens on the request itself, at its top, with no failure: the
    /// user reads what will be written before approving (plan §3 rule 4).
    pub fn new(planned: Planned) -> ConfirmApp {
        ConfirmApp {
            planned,
            screen: ConfirmScreen::Reading,
            scroll: 0,
            failure: None,
        }
    }

    /// The files the approval writes.
    pub fn planned(&self) -> &Planned {
        &self.planned
    }

    pub fn screen(&self) -> ConfirmScreen {
        self.screen
    }

    /// How far the open screen's body is scrolled, with no upper bound: `view`
    /// and `draw_confirm` clamp it to their own lines, as 7c3 §3.1 rule 5 does.
    pub fn scroll(&self) -> usize {
        self.scroll
    }

    /// The last approval's failure, shown on the reading screen.
    pub fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }

    /// An approval that refused: one line for the screen, and back to reading,
    /// where the user can quit, or `a` then `y` again, exactly as 7f2's
    /// `approval_failed` does. Nothing was written by a refusal, and a screen
    /// change puts the new screen at its top.
    pub fn failed(&mut self, message: String) {
        self.failure = Some(message);
        self.screen = ConfirmScreen::Reading;
        self.scroll = 0;
    }

    /// `q` quits from every screen, writing nothing, and so does `ctrl-c` (M7-43).
    /// `esc` on reading goes back to the form; `a` opens the confirmation, which
    /// `y` approves and `n` or `esc` leaves. The keys of 7c3 scroll the reading
    /// screen. Every other key changes nothing, and a screen change puts the new
    /// screen at its top.
    pub fn handle(&mut self, key: Key) -> ConfirmAction {
        // M7-43: `ctrl-c` quits from every screen, writing nothing, where `Esc`
        // goes back.
        if key == Key::CtrlC {
            return ConfirmAction::Quit;
        }
        if key == Key::Char('q') {
            return ConfirmAction::Quit;
        }

        let opened: ConfirmScreen = self.screen;

        match (self.screen, key) {
            (ConfirmScreen::Reading, Key::Esc) => return ConfirmAction::Back,
            (ConfirmScreen::Reading, Key::Char('a')) => {
                self.screen = ConfirmScreen::Confirm;
            }
            (ConfirmScreen::Reading, Key::Down | Key::Char('j')) => {
                self.scroll += 1;
            }
            (ConfirmScreen::Reading, Key::Up | Key::Char('k')) => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            (ConfirmScreen::Reading, Key::PageUp | Key::PageDown | Key::Home) => self.page(key),
            (ConfirmScreen::Confirm, Key::Char('y')) => return ConfirmAction::Approve,
            (ConfirmScreen::Confirm, Key::Char('n') | Key::Esc | Key::Backspace) => {
                self.screen = ConfirmScreen::Reading;
            }
            _ => {}
        }

        if self.screen != opened {
            self.scroll = 0;
        }
        ConfirmAction::Continue
    }

    /// One page key's effect on the scroll, as `app`'s and `ProposalApp`'s:
    /// `PageDown` adds 10, `PageUp` subtracts 10 and never goes below 0,
    /// `Home` is 0.
    fn page(&mut self, key: Key) {
        self.scroll = match key {
            Key::PageDown => self.scroll.saturating_add(10),
            Key::PageUp => self.scroll.saturating_sub(10),
            Key::Home => 0,
            _ => self.scroll,
        };
    }
}

/// The confirmation's lines: on reading, `This request will be written:`, the
/// request file's path, then the request exactly as the file will hold it, then
/// for a paste the FASTA file it writes first, and any failure line. On the
/// confirmation, one question naming every file, as 7f2 names the request file,
/// and `y` `n`.
pub fn confirm_lines(app: &ConfirmApp) -> Vec<String> {
    match app.screen() {
        ConfirmScreen::Reading => {
            let mut lines: Vec<String> = Vec::new();
            lines.push("This request will be written:".to_string());
            lines.push(app.planned().request.display().to_string());
            lines.push(String::new());
            // The request byte for byte as `request_text` writes it (plan §3
            // rule 2): `request_echo` of the built document, pretty-printed.
            let text: String = request_text(&request_echo(&app.planned().built.request));
            for line in text.lines() {
                lines.push(line.to_string());
            }
            if let Some((paste, _)) = &app.planned().paste {
                lines.push(String::new());
                lines.push("and the template:".to_string());
                lines.push(paste.display().to_string());
            }
            if let Some(message) = app.failure() {
                lines.push(String::new());
                lines.push(message.to_string());
            }
            lines
        }
        ConfirmScreen::Confirm => {
            let question: String = match &app.planned().paste {
                Some((paste, _)) => format!(
                    "write the template to {} and the request to {} and run luria q5 design on it?",
                    paste.display(),
                    app.planned().request.display()
                ),
                None => format!(
                    "write the request to {} and run luria q5 design on it?",
                    app.planned().request.display()
                ),
            };
            vec![question, String::new(), "y yes  n no".to_string()]
        }
    }
}

/// Draw `app`' confirmation into `frame`: 7j2's rounded frame with the
/// ` PROPOSAL ` badge, black on cyan, as the review's proposal screen carries
/// it, the confirmation's lines as they are, scrolled, and the key line, which
/// offers `a approve` only on reading and `y yes` only on the confirmation.
/// Below 80x24 the review's notice, and nothing of the confirmation is drawn.
pub fn draw_confirm(frame: &mut Frame, app: &ConfirmApp) {
    let area: Rect = frame.area();

    if too_small(frame, area) {
        return;
    }

    let screen_area: Rect = Rect::new(0, 0, area.width, area.height - 1);
    let title: Line<'static> = Line::from(vec![
        Span::raw(" luria new "),
        Span::styled(
            " PROPOSAL ",
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ),
        Span::raw(" · nothing is written until you approve "),
    ]);
    let block: Block<'static> = frame_block(title);
    let inner: Rect = block.inner(screen_area);
    frame.render_widget(block, screen_area);

    let lines: Vec<String> = confirm_lines(app);
    // The same clamp as `draw` and `draw_proposal`: the last line the body can
    // start at.
    let rows: usize = inner.height as usize;
    let first: usize = app.scroll().min(lines.len().saturating_sub(rows));
    let shown: Vec<Line<'static>> = lines[first..]
        .iter()
        .map(|line| Line::from(line.clone()))
        .collect();
    frame.render_widget(Paragraph::new(shown), inner);

    let mut keys: String = match app.screen() {
        ConfirmScreen::Reading => "q quit  j/k scroll  a approve".to_string(),
        ConfirmScreen::Confirm => "q quit  y yes  n no".to_string(),
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
        Paragraph::new(key_line(&keys)),
        Rect::new(0, area.height - 1, area.width, 1),
    );
}

/// The template file the request is written beside: the file name the request
/// states, in the directory the request goes in. For a paste that is the FASTA
/// file the approval writes first, so the location check below sees the pair as
/// `prepare` does (plan §3.4), and for a template file it is that file itself
/// (M7-37).
fn template_of(planned: &Planned) -> PathBuf {
    let name: &str = match &planned.built.request.template {
        TemplateSpec::Fasta { path, .. } | TemplateSpec::GenBank { path, .. } => path,
    };
    let dir: &Path = match planned.request.parent() {
        Some(found) => found,
        None => Path::new(""),
    };
    dir.join(name)
}

/// A path's file name, as the screen and the messages name a file.
fn name_of(path: &Path) -> String {
    match path.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => path.to_string_lossy().into_owned(),
    }
}

/// Write the paste (if any), then the request, each as a new file never
/// replacing one (`interpret::run::write_request_file`), then run the
/// design from the request file exactly as `review::approve` does after its
/// write, giving `review::Approved`.
///
/// The location is checked before anything is written, as `approve` checks it:
/// for a template file, `request_location_problem` is the rule 7f1 states, and
/// a paste and its request share the current directory, so the check passes by
/// construction, and is run anyway. The `Err` is one line for the screen; a
/// paste already written stands on disk, nothing is deleted, and the line names
/// it.
pub fn approve_new(planned: &Planned) -> Result<Approved, String> {
    if let Some((_code, message)) =
        request_location_problem(&template_of(planned), &planned.request)
    {
        return Err(message);
    }

    // The paste first: the template the request names, as a new file.
    if let Some((paste, text)) = &planned.paste {
        if let Err(reason) = write_request_file(paste, text) {
            return Err(format!("could not write {}: {}", name_of(paste), reason));
        }
    }

    // The request, byte for byte the text the confirmation showed.
    match write_request_file(
        &planned.request,
        &request_text(&request_echo(&planned.built.request)),
    ) {
        Err(reason) => Err(match &planned.paste {
            Some((paste, _)) => format!(
                "{} is written, but luria cannot write {}: {}",
                name_of(paste),
                name_of(&planned.request),
                reason
            ),
            None => format!("could not write {}: {}", name_of(&planned.request), reason),
        }),
        Ok(()) => Ok(review::run_approved(&planned.request)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::new::{CdsChoice, CodonChoice, Form, TemplateInput};
    use crate::request::{StrandSpec, TopologySpec};

    /// A planned approval from a filled form: `plan` is pure and writes nothing,
    /// so `cwd` is a path no test touches, and the key rule uses only `screen`.
    fn planned() -> Planned {
        let form: Form = Form {
            template: TemplateInput::Paste {
                text: "ATGGCT".to_string(),
                name: "syn".to_string(),
                save_as: "syn.fa".to_string(),
                topology: TopologySpec::Linear,
                cds: CdsChoice::Whole(StrandSpec::Plus),
            },
            lines: vec!["M1A".to_string()],
            codon: CodonChoice::DefaultPolicy,
            overlap: Some(15),
        };
        plan(
            new::build(&form).expect("the form builds"),
            None,
            Path::new("here"),
            "syn.json",
        )
    }

    // M7-43: `ctrl-c` quits from both screens the confirmation has, writing
    // nothing. `Esc` does not quit on either: on reading it goes back to the
    // form, and on the confirmation it goes back to reading.
    #[test]
    fn ctrl_c_quits_from_every_screen_where_esc_goes_back() {
        let mut reading: ConfirmApp = ConfirmApp::new(planned());
        assert_eq!(
            reading.handle(Key::CtrlC),
            ConfirmAction::Quit,
            "`ctrl-c` quits from the request screen"
        );

        let mut confirming: ConfirmApp = ConfirmApp::new(planned());
        confirming.handle(Key::Char('a'));
        assert_eq!(confirming.screen(), ConfirmScreen::Confirm);
        assert_eq!(
            confirming.handle(Key::CtrlC),
            ConfirmAction::Quit,
            "`ctrl-c` quits from the confirmation too, writing nothing"
        );

        let mut reading: ConfirmApp = ConfirmApp::new(planned());
        assert_eq!(
            reading.handle(Key::Esc),
            ConfirmAction::Back,
            "`Esc` on the request screen goes back to the form, not quit"
        );

        let mut confirming: ConfirmApp = ConfirmApp::new(planned());
        confirming.handle(Key::Char('a'));
        assert_eq!(
            confirming.handle(Key::Esc),
            ConfirmAction::Continue,
            "`Esc` on the confirmation goes back to reading, not quit"
        );
        assert_eq!(confirming.screen(), ConfirmScreen::Reading);
    }
}
