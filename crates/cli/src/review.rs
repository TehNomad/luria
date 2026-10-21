//! `luria review` (milestone 7a): run a request as `luria q5 design` does and
//! show its result on the terminal screen. The state and the experimental
//! validation shown are the document's own strings (plan §3 rule 1: the view
//! invents nothing), and every result screen shows both (plan §3 rule 3).
//!
//! The layout is `interpret.rs`'s: `app` is the state and its key rule,
//! `view` is the drawing, and `terminal` is the only file with I/O (plan §5.1),
//! so the screen's content is testable without a terminal. `terminal` is not
//! unit-tested; it keeps to the terminal session and the event loop.

use crate::args::UsageError;

pub mod app;
pub mod terminal;
pub mod text;
pub mod view;

/// Why `luria review` cannot open its screen here, or `None` when it can
/// (plan §3 rule 7). Both stdin (keys) and stdout (the screen) must be
/// terminals. Decided before the request is read.
pub fn terminal_problem(stdin_is_terminal: bool, stdout_is_terminal: bool) -> Option<UsageError> {
    if stdin_is_terminal && stdout_is_terminal {
        return None;
    }
    Some(UsageError {
        message: "'review' needs a terminal on stdin and stdout; 'q5 design' writes the same result as JSON".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The message §3.2 names, exactly.
    const MESSAGE: &str =
        "'review' needs a terminal on stdin and stdout; 'q5 design' writes the same result as JSON";

    #[test]
    fn review_needs_a_terminal_on_both_streams() {
        for (stdin, stdout) in [(false, true), (true, false), (false, false)] {
            assert_eq!(
                terminal_problem(stdin, stdout),
                Some(UsageError {
                    message: MESSAGE.to_string()
                }),
                "stdin {} stdout {}",
                stdin,
                stdout
            );
        }
        assert_eq!(terminal_problem(true, true), None);
    }
}
