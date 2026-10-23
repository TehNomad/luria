//! The state of `luria review`' proposal screen (milestone 7f2, plan §5.2
//! screen 1): the document one interpret run answered with, the request file an
//! approval will write, which of the two screens is open, how far the body is
//! scrolled, and the last approval's failure. No I/O and no terminal types are
//! here, so the rule is testable without a terminal (plan §5.1); `terminal` maps
//! crossterm's events to `Key` and carries out the `Action`.
//!
//! Plan §3 rule 4 is the rule here: nothing writes or runs until `y` on the
//! confirmation, and `a` reaches that confirmation only from a `proposed`
//! document that names its request. The keys of 7c3 (`j` `k` the arrows and the
//! page keys) scroll the reading screen, `q` quits from both screens, and `Esc`
//! quits only when reading: on the confirmation it goes back. A screen change
//! puts the new screen at its top (7c3 §3.1 rule 4), the same rule `app` keeps.

use crate::interpret::document::InterpretationDocument;
use crate::review::app::Key;
use std::path::{Path, PathBuf};

/// Which proposal screen is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProposalScreen {
    /// The proposal's own lines, as `text::proposal` builds them (7e).
    Reading,
    /// The confirmation: the request path named, and `y` `n`.
    Confirm,
}

/// What the app asks its shell to do after a key. Only `Approve` writes or runs
/// anything, and only after `y` on the confirmation (plan §3 rule 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProposalAction {
    Continue,
    Quit,
    Approve,
}

/// Whether a document can be approved: `proposed`, and naming its request. This
/// is plan §3 rule 4's condition, stated once: `handle` opens the confirmation
/// by it, `view` offers `a approve` by it, and `review::approve` checks it again
/// when the shell runs the approval.
pub fn approvable(document: &InterpretationDocument) -> bool {
    document.state == "proposed" && document.request.is_some()
}

/// The proposal screen's state: the document to show, the request file named by
/// `--request`, the open screen, how far it is scrolled, and the last approval's
/// failure.
#[derive(Clone, Debug, PartialEq)]
pub struct ProposalApp {
    document: InterpretationDocument,
    request: PathBuf,
    screen: ProposalScreen,
    scroll: usize,
    failure: Option<String>,
}

impl ProposalApp {
    /// The screen opens on the proposal itself, at its top, with no failure:
    /// the user reads it before deciding (plan §3 rule 4).
    pub fn new(document: InterpretationDocument, request: PathBuf) -> ProposalApp {
        ProposalApp {
            document,
            request,
            screen: ProposalScreen::Reading,
            scroll: 0,
            failure: None,
        }
    }

    pub fn document(&self) -> &InterpretationDocument {
        &self.document
    }

    /// The request file, as `--request` gave it: the confirmation names it
    /// before anything is written.
    pub fn request(&self) -> &Path {
        &self.request
    }

    pub fn screen(&self) -> ProposalScreen {
        self.screen
    }

    /// How far the open screen's body is scrolled, clamped by `terminal` to
    /// `view::proposal_max_scroll` after every key (M7-56).
    pub fn scroll(&self) -> usize {
        self.scroll
    }

    /// The scroll, clamped to `max`, the largest the open screen's body can start
    /// at (M7-56): `scroll` never goes past it, so `k` moves the body at once
    /// after scrolling past the bottom.
    pub fn clamp_scroll(&mut self, max: usize) {
        self.scroll = self.scroll.min(max);
    }

    /// The last approval's failure, shown on the reading screen.
    pub fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }

    /// An approval that refused: one line for the screen, and back to reading,
    /// where the user can quit, and the screen starts at its top.
    pub fn approval_failed(&mut self, message: String) {
        self.failure = Some(message);
        self.screen = ProposalScreen::Reading;
        self.scroll = 0;
    }

    /// `q` quits from every screen, and so does `ctrl-c` (M7-43), which writes
    /// nothing. `Esc` quits only when reading. The keys of
    /// 7c3 scroll the reading screen. `a` opens the confirmation only for a
    /// `proposed` document that names its request; `y` asks for the approval,
    /// and the confirmation stays open until the shell reports, so the user sees
    /// what was named while it runs. `n` and `Backspace` go back. Every other
    /// key changes nothing. A screen change puts the new screen at its top.
    pub fn handle(&mut self, key: Key) -> ProposalAction {
        // M7-43: `ctrl-c` quits from every screen, writing nothing.
        if key == Key::CtrlC {
            return ProposalAction::Quit;
        }
        if key == Key::Char('q') {
            return ProposalAction::Quit;
        }

        let opened: ProposalScreen = self.screen;

        match (self.screen, key) {
            (ProposalScreen::Reading, Key::Esc) => return ProposalAction::Quit,
            (ProposalScreen::Reading, Key::Char('a')) => {
                if approvable(&self.document) {
                    self.screen = ProposalScreen::Confirm;
                }
            }
            (ProposalScreen::Reading, Key::Down | Key::Char('j')) => {
                self.scroll += 1;
            }
            (ProposalScreen::Reading, Key::Up | Key::Char('k')) => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            (ProposalScreen::Reading, Key::PageUp | Key::PageDown | Key::Home) => self.page(key),
            (ProposalScreen::Confirm, Key::Char('y')) => return ProposalAction::Approve,
            (ProposalScreen::Confirm, Key::Char('n') | Key::Esc | Key::Backspace) => {
                self.screen = ProposalScreen::Reading;
            }
            _ => {}
        }

        if self.screen != opened {
            self.scroll = 0;
        }
        ProposalAction::Continue
    }

    /// One page key's effect on the scroll, as `app`'s: `PageDown` adds 10,
    /// `PageUp` subtracts 10 and never goes below 0, `Home` is 0.
    fn page(&mut self, key: Key) {
        self.scroll = match key {
            Key::PageDown => self.scroll.saturating_add(10),
            Key::PageUp => self.scroll.saturating_sub(10),
            Key::Home => 0,
            _ => self.scroll,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpret::document::InterpretationView;
    use crate::request::parse_request;
    use crate::result::{request_echo, PackageView, RequestEcho};
    use luria_nl::backend::BackendInfo;
    use luria_nl::prompt::PROMPT_VERSION;
    use std::fs;
    use std::path::Path;

    /// A document as one interpret run's answer has its shape, with nothing
    /// read: no template, no messages sent, no reply, so the only fields the key
    /// rule uses are the two under test, `state` and `request`.
    fn base() -> InterpretationDocument {
        InterpretationDocument {
            schema: crate::INTERPRETATION_SCHEMA,
            state: "proposed",
            package: PackageView {
                name: "luria-cli",
                version: env!("CARGO_PKG_VERSION"),
            },
            request: None,
            questions: Vec::new(),
            origins: Vec::new(),
            assumptions: Vec::new(),
            error: None,
            template: None,
            interpretation: InterpretationView {
                backend: BackendInfo {
                    kind: "openai-compatible".to_string(),
                    endpoint: None,
                    model: "scripted".to_string(),
                },
                prompt_version: PROMPT_VERSION,
                messages: None,
                reply: None,
                extraction: None,
                ungrounded: None,
            },
        }
    }

    /// The request echo, built from the corpus's own request fixture, so the
    /// `request` field holds a real request the screen can show.
    fn echo() -> RequestEcho {
        let written: String = fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures/nl/request/variants-separate.json"),
        )
        .expect("the request fixture is readable");
        request_echo(&parse_request(&written).expect("the fixture is a request"))
    }

    /// A document with `state` and, when `with_request`, the corpus request: the
    /// two fields `handle` reads.
    fn hand(state: &'static str, with_request: bool) -> InterpretationDocument {
        let mut document: InterpretationDocument = base();
        document.state = state;
        document.request = match with_request {
            true => Some(echo()),
            false => None,
        };
        document
    }

    /// The app of a document, with the request path the confirmation names.
    fn app_of(state: &'static str, with_request: bool) -> ProposalApp {
        ProposalApp::new(hand(state, with_request), PathBuf::from("out.json"))
    }

    #[test]
    fn a_opens_confirm_only_for_a_proposal() {
        let mut reading: ProposalApp = app_of("proposed", true);
        assert_eq!(reading.screen(), ProposalScreen::Reading);
        assert_eq!(reading.handle(Key::Char('a')), ProposalAction::Continue);
        assert_eq!(
            reading.screen(),
            ProposalScreen::Confirm,
            "a proposal that names its request reaches the confirmation"
        );

        let mut asked: ProposalApp = app_of("needs_clarification", false);
        asked.handle(Key::Char('a'));
        assert_eq!(
            asked.screen(),
            ProposalScreen::Reading,
            "a run that asks has nothing to approve (plan §3 rule 4)"
        );

        let mut nameless: ProposalApp = app_of("proposed", false);
        nameless.handle(Key::Char('a'));
        assert_eq!(
            nameless.screen(),
            ProposalScreen::Reading,
            "a proposal naming no request has nothing to write"
        );

        // The rule is the document's own, so `a` keeps working on the same
        // document, and the screen opens at its top.
        let mut proposed: ProposalApp = app_of("proposed", true);
        proposed.handle(Key::Char('j'));
        assert_eq!(proposed.scroll(), 1);
        for _ in 0..2 {
            proposed.handle(Key::Char('a'));
        }
        assert_eq!(proposed.screen(), ProposalScreen::Confirm);
        assert_eq!(proposed.scroll(), 0, "the new screen opens at its top");
    }

    #[test]
    fn y_approves_and_n_goes_back() {
        let mut confirming: ProposalApp = app_of("proposed", true);
        assert_eq!(confirming.handle(Key::Char('a')), ProposalAction::Continue);
        assert_eq!(confirming.handle(Key::Char('y')), ProposalAction::Approve);
        assert_eq!(
            confirming.screen(),
            ProposalScreen::Confirm,
            "the confirmation stays open until the shell reports"
        );

        let mut back: ProposalApp = app_of("proposed", true);
        back.handle(Key::Char('a'));
        assert_eq!(back.handle(Key::Char('n')), ProposalAction::Continue);
        assert_eq!(back.screen(), ProposalScreen::Reading);
        assert_eq!(back.scroll(), 0, "going back opens the screen at its top");
        back.handle(Key::Char('j'));
        assert_eq!(back.handle(Key::Char('a')), ProposalAction::Continue);
        assert_eq!(back.handle(Key::Backspace), ProposalAction::Continue);
        assert_eq!(back.screen(), ProposalScreen::Reading);
        assert_eq!(back.scroll(), 0);

        // `y` is the approval key on the confirmation only: on the reading
        // screen it changes nothing.
        let mut reading: ProposalApp = app_of("proposed", true);
        assert_eq!(reading.handle(Key::Char('y')), ProposalAction::Continue);
        assert_eq!(reading.screen(), ProposalScreen::Reading);
    }

    // M7-43: `ctrl-c` quits from every screen the proposal app has, writing
    // nothing, where `Esc` quits only when reading and goes back on the
    // confirmation. A run that asks has nothing to approve, so its `ctrl-c` quits
    // from the reading screen it stays on.
    #[test]
    fn ctrl_c_quits_from_every_screen() {
        for (state, with_request) in [("proposed", true), ("needs_clarification", false)] {
            let mut reading: ProposalApp = app_of(state, with_request);
            assert_eq!(
                reading.handle(Key::CtrlC),
                ProposalAction::Quit,
                "{}: `ctrl-c` quits from the proposal",
                state
            );
        }

        // `a` opens the confirmation only for a proposal naming its request, so
        // the confirmation is reached on the document that can be approved.
        let mut confirming: ProposalApp = app_of("proposed", true);
        confirming.handle(Key::Char('a'));
        assert_eq!(confirming.screen(), ProposalScreen::Confirm);
        assert_eq!(
            confirming.handle(Key::CtrlC),
            ProposalAction::Quit,
            "`ctrl-c` quits from the confirmation too, writing nothing"
        );

        let mut confirming: ProposalApp = app_of("proposed", true);
        confirming.handle(Key::Char('a'));
        assert_eq!(
            confirming.handle(Key::Esc),
            ProposalAction::Continue,
            "`Esc` goes back from the confirmation, where `ctrl-c` quits"
        );
        assert_eq!(confirming.screen(), ProposalScreen::Reading);
    }

    #[test]
    fn q_quits_everywhere_and_esc_quits_only_when_reading() {
        for (state, with_request) in [("proposed", true), ("needs_clarification", false)] {
            let mut reading: ProposalApp = app_of(state, with_request);
            assert_eq!(
                reading.handle(Key::Char('q')),
                ProposalAction::Quit,
                "{}: `q` quits from the proposal",
                state
            );

            let mut confirming: ProposalApp = app_of(state, with_request);
            confirming.handle(Key::Char('a'));
            assert_eq!(
                confirming.handle(Key::Char('q')),
                ProposalAction::Quit,
                "{}: `q` quits from the confirmation too, and from a screen `a` did not open",
                state
            );
        }

        let mut reading: ProposalApp = app_of("proposed", true);
        assert_eq!(reading.handle(Key::Esc), ProposalAction::Quit);

        let mut confirming: ProposalApp = app_of("proposed", true);
        confirming.handle(Key::Char('a'));
        assert_eq!(confirming.handle(Key::Esc), ProposalAction::Continue);
        assert_eq!(
            confirming.screen(),
            ProposalScreen::Reading,
            "`Esc` goes back from the confirmation, never quits it"
        );
        assert_eq!(confirming.handle(Key::Esc), ProposalAction::Quit);
    }

    #[test]
    fn a_failure_returns_to_reading_and_is_kept() {
        let mut app: ProposalApp = app_of("proposed", true);
        assert_eq!(app.handle(Key::Char('a')), ProposalAction::Continue);
        assert_eq!(app.failure(), None);
        app.approval_failed(
            "the file out.json already exists; luria never replaces a file".to_string(),
        );
        assert_eq!(
            app.screen(),
            ProposalScreen::Reading,
            "the failure is shown on the reading screen (7f2 §3.3)"
        );
        assert_eq!(
            app.failure(),
            Some("the file out.json already exists; luria never replaces a file"),
            "the message is the one the approval gave, kept for the user to read"
        );
        assert_eq!(app.scroll(), 0, "the reading screen is at its top");
        // The document is unchanged, so the user can read it and try again.
        assert_eq!(app.handle(Key::Char('a')), ProposalAction::Continue);
        assert_eq!(app.screen(), ProposalScreen::Confirm);
        assert_eq!(app.handle(Key::Char('q')), ProposalAction::Quit);
        assert_eq!(
            app.failure(),
            Some("the file out.json already exists; luria never replaces a file"),
            "quitting does not clear what was shown"
        );
    }

    /// The scrolling keys move the reading screen as 7c3's do, and the page keys
    /// never take the scroll below 0.
    #[test]
    fn the_reading_screen_scrolls_as_7c3s_does() {
        let mut app: ProposalApp = app_of("proposed", true);
        for _ in 0..3 {
            app.handle(Key::Down);
        }
        assert_eq!(app.scroll(), 3);
        app.handle(Key::Char('k'));
        assert_eq!(app.scroll(), 2);
        app.handle(Key::PageUp);
        assert_eq!(app.scroll(), 0, "`PageUp` never goes below 0");
        app.handle(Key::PageDown);
        assert_eq!(app.scroll(), 10);
        app.handle(Key::Home);
        assert_eq!(app.scroll(), 0, "`Home` is the top");
        for _ in 0..5 {
            app.handle(Key::Up);
        }
        assert_eq!(app.scroll(), 0);
        // A run that asks has nothing to approve, so its keys scroll only.
        let mut asked: ProposalApp = app_of("needs_clarification", false);
        for key in [Key::Char('j'), Key::PageDown, Key::Char('a')] {
            assert_eq!(asked.handle(key), ProposalAction::Continue);
        }
        assert_eq!(asked.screen(), ProposalScreen::Reading);
        assert_eq!(asked.scroll(), 11);
    }
}
