//! The state of the review screen (milestone 7a): the result document, and the
//! one rule this skeleton has - `q` and `Esc` quit. No I/O and no terminal
//! types are here, so the rule is testable without a terminal (plan §5.1);
//! `terminal` maps crossterm's events to `Key` and carries out the `Action`.

use crate::result::ResultDocument;

/// A key, as the app sees it. `terminal` maps crossterm's events to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Esc,
    Other,
}

/// What the app asks its shell to do after a key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Continue,
    Quit,
}

/// The screen's state: the result document to show, and nothing else.
#[derive(Clone, Debug, PartialEq)]
pub struct App {
    document: ResultDocument,
}

impl App {
    pub fn new(document: ResultDocument) -> App {
        App { document }
    }

    pub fn document(&self) -> &ResultDocument {
        &self.document
    }

    /// `q` and `Esc` quit; every other key continues.
    pub fn handle(&mut self, key: Key) -> Action {
        match key {
            Key::Char('q') | Key::Esc => Action::Quit,
            Key::Char(_) | Key::Other => Action::Continue,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A document, for the key rule to be tested on: the screen shows its
    /// state, and the key rule never reads it. Nothing scientific is here; the
    /// fields `run` fills from a prepared request are left empty.
    fn document() -> ResultDocument {
        ResultDocument {
            schema: crate::RESULT_SCHEMA,
            state: "verified",
            experimental_validation: "not_performed",
            provenance: crate::result::Provenance {
                package: crate::result::PackageView {
                    name: env!("CARGO_PKG_NAME"),
                    version: env!("CARGO_PKG_VERSION"),
                },
                algorithms: Vec::new(),
                request_sha256: None,
                operation: None,
                template: None,
                buffer: None,
            },
            request: None,
            target: None,
            clarifications: Vec::new(),
            candidates: Vec::new(),
            warnings: Vec::new(),
            results: Vec::new(),
            error: None,
        }
    }

    #[test]
    fn q_and_esc_quit() {
        let mut app: App = App::new(document());
        assert_eq!(app.handle(Key::Char('q')), Action::Quit);
        assert_eq!(app.handle(Key::Esc), Action::Quit);
    }

    #[test]
    fn other_keys_continue() {
        let mut app: App = App::new(document());
        assert_eq!(app.handle(Key::Char('x')), Action::Continue);
        assert_eq!(app.handle(Key::Char('Q')), Action::Continue);
        assert_eq!(app.handle(Key::Other), Action::Continue);
    }
}
